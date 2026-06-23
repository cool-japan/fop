//! Content measurement for automatic table layout (`table-layout="auto"`).
//!
//! XSL-FO §7.26.17 requires the columns of an automatic-layout table to be
//! sized to their content: a column holding a single short word must end up
//! narrower than a column holding a long sentence.  Achieving that means
//! measuring, for every cell, two intrinsic widths:
//!
//! * the **minimum content width** — the widest *unbreakable* unit (the widest
//!   word, or widest atomic inline) below which the content would overflow even
//!   with unlimited line breaking; and
//! * the **maximum content width** — the width the content would occupy if laid
//!   out on a single line with no wrapping at all.
//!
//! These are folded, column by column, into [`ColumnInfo`] min/max widths and
//! handed to the already-correct CSS2.1 distributor
//! [`TableLayout::compute_auto_widths`].  Cells that span several columns
//! (`number-columns-spanned`) impose their intrinsic widths on the *set* of
//! columns they cover, per the standard distribution rule.
//!
//! All glyph advances come from the real per-variant Standard-14 metrics in
//! [`fop_types::FontRegistry`] via
//! [`crate::layout::properties::measure_text_metrics`], so the column widths
//! agree with the Knuth-Plass line breaker and the rendered area geometry.

use crate::area::TraitSet;
use crate::layout::properties::measure_text_metrics;
use crate::layout::{extract_traits, ColumnInfo, ColumnWidth, TableLayout};
use fop_core::{FoArena, FoNodeData, NodeId, PropertyId};
use fop_types::Length;

use super::LayoutEngine;

/// Intrinsic (min, max) content widths measured for one spanning cell, deferred
/// until all single-column cells have established the per-column baselines.
struct SpanMeasure {
    /// First column the cell covers.
    start: usize,
    /// Number of columns the cell covers (already clamped to the grid).
    span: usize,
    /// Minimum content width of the cell.
    min: Length,
    /// Maximum content width of the cell.
    max: Length,
}

impl LayoutEngine {
    /// Compute final column widths for an automatic-layout table by measuring
    /// the real content of every cell.
    ///
    /// `column_specs` carries the explicitly declared `<fo:table-column>` widths
    /// (in document order).  Undeclared trailing columns — needed when rows hold
    /// more cells than there are column declarations — are treated as `Auto`.
    ///
    /// The returned vector always has one entry per grid column and, whenever the
    /// content fits, sums to exactly the table's content width so the table fills
    /// its inline-progression-dimension (leftover space is distributed across the
    /// auto columns in proportion to their maximum content width).
    pub(super) fn measure_auto_column_widths(
        &self,
        fo_tree: &FoArena,
        table_node_id: NodeId,
        column_specs: &[ColumnWidth],
        table_layout: &TableLayout,
    ) -> Vec<Length> {
        let declared = column_specs.len();
        let section_rows = collect_section_rows(fo_tree, table_node_id);
        let measured_cols = max_columns(fo_tree, &section_rows);
        let n_cols = declared.max(measured_cols).max(1);

        // Per-column intrinsic widths gathered from non-spanning cells.
        let mut col_min = vec![Length::ZERO; n_cols];
        let mut col_max = vec![Length::ZERO; n_cols];
        let mut spanning: Vec<SpanMeasure> = Vec::new();

        // Walk every section independently — row spans never cross a header/body/
        // footer boundary, so the occupancy bookkeeping resets per section.
        for rows in &section_rows {
            let mut occupied = vec![0usize; n_cols];

            for &row_id in rows {
                let mut col = 0usize;

                for cell_id in fo_tree.children(row_id) {
                    let is_cell = fo_tree
                        .get(cell_id)
                        .map(|n| matches!(n.data, FoNodeData::TableCell { .. }))
                        .unwrap_or(false);
                    if !is_cell {
                        continue;
                    }

                    // Skip columns still occupied by a row-spanning cell from above.
                    while col < n_cols && occupied[col] > 0 {
                        col += 1;
                    }
                    if col >= n_cols {
                        break;
                    }

                    let (colspan, rowspan) = cell_spans(fo_tree, cell_id);
                    let span = colspan.min(n_cols - col).max(1);
                    let (cell_min, cell_max) = self.measure_cell_content_widths(fo_tree, cell_id);

                    if span == 1 {
                        col_min[col] = col_min[col].max(cell_min);
                        col_max[col] = col_max[col].max(cell_max);
                    } else {
                        spanning.push(SpanMeasure {
                            start: col,
                            span,
                            min: cell_min,
                            max: cell_max,
                        });
                    }

                    // Mark the covered columns as occupied for the next rowspan-1 rows.
                    let occ = rowspan.max(1);
                    for slot in occupied.iter_mut().skip(col).take(span) {
                        *slot = occ;
                    }
                    col += span;
                }

                // Advance the rowspan occupancy to the next row.
                for slot in occupied.iter_mut() {
                    if *slot > 0 {
                        *slot -= 1;
                    }
                }
            }
        }

        // Spanning cells raise the *sum* of the columns they cover to at least
        // their own intrinsic width (the standard distribution rule).
        for span in &spanning {
            distribute_span(&mut col_min, span.start, span.span, span.min);
            distribute_span(&mut col_max, span.start, span.span, span.max);
        }
        for i in 0..n_cols {
            if col_max[i] < col_min[i] {
                col_max[i] = col_min[i];
            }
        }

        // Build column info: declared specs first, undeclared columns are auto.
        let column_info: Vec<ColumnInfo> = (0..n_cols)
            .map(|i| {
                let spec = if i < declared {
                    column_specs[i].clone()
                } else {
                    ColumnWidth::Auto
                };
                ColumnInfo::with_widths(spec, col_min[i], col_max[i])
            })
            .collect();

        let mut widths = table_layout.compute_auto_widths(&column_info);
        self.fill_auto_leftover(&mut widths, &column_info, table_layout);
        widths
    }

    /// Grow auto columns so the column boxes exactly span the table's content
    /// width when the content fits (CSS2.1 only assigns max widths in that case,
    /// leaving the table narrower than its IPD).  Leftover space is shared among
    /// the auto columns in proportion to their maximum content width, so wider
    /// content keeps getting the wider column.  Never shrinks a column, so the
    /// min-content guarantee established by `compute_auto_widths` is preserved.
    fn fill_auto_leftover(
        &self,
        widths: &mut [Length],
        column_info: &[ColumnInfo],
        table_layout: &TableLayout,
    ) {
        let n = widths.len();
        if n == 0 {
            return;
        }
        let target = table_layout.content_width_for_columns(n);
        let used = widths.iter().fold(Length::ZERO, |acc, w| acc + *w);
        let leftover = target - used;
        // Only ever distribute genuine slack; a tiny epsilon avoids fighting f64
        // rounding when the columns already fill the table.
        if leftover <= Length::from_pt(0.01) {
            return;
        }

        let auto_idx: Vec<usize> = (0..n)
            .filter(|&i| matches!(column_info[i].width_spec, ColumnWidth::Auto))
            .collect();
        if auto_idx.is_empty() {
            return;
        }

        let total_max: f64 = auto_idx
            .iter()
            .map(|&i| column_info[i].max_width.to_pt())
            .sum();

        // Distribute the leftover; the last auto column absorbs the running
        // remainder so the totals add up to `target` exactly (no rounding drift).
        let mut distributed = Length::ZERO;
        let last = auto_idx.len() - 1;
        for (k, &i) in auto_idx.iter().enumerate() {
            let add = if k == last {
                leftover - distributed
            } else if total_max > 0.0 {
                Length::from_pt(leftover.to_pt() * column_info[i].max_width.to_pt() / total_max)
            } else {
                Length::from_pt(leftover.to_pt() / auto_idx.len() as f64)
            };
            widths[i] += add;
            distributed += add;
        }
    }

    /// Measure the intrinsic `(min, max)` content widths of a single table cell.
    ///
    /// `min` is the widest unbreakable unit anywhere in the cell; `max` is the
    /// widest single-line width of any block-level descendant (blocks stack
    /// vertically, so the cell only needs to be as wide as its widest block).
    pub(super) fn measure_cell_content_widths(
        &self,
        fo_tree: &FoArena,
        cell_id: NodeId,
    ) -> (Length, Length) {
        let cell_props = fo_tree.get(cell_id).and_then(|n| n.data.properties());
        let cell_traits = resolve_text_traits(&TraitSet::default(), cell_props);
        self.measure_block_context(fo_tree, cell_id, &cell_traits)
    }

    /// Measure a block formatting context rooted at `node_id`.
    ///
    /// Inline siblings accumulate into a running line; block-level siblings flush
    /// that line and recurse, because each block starts on a fresh line.  Returns
    /// `(min, max)` where `max` is the widest completed line.
    fn measure_block_context(
        &self,
        fo_tree: &FoArena,
        node_id: NodeId,
        traits: &TraitSet,
    ) -> (Length, Length) {
        let mut min = Length::ZERO;
        let mut block_max = Length::ZERO;
        let mut run = Length::ZERO;

        for child_id in fo_tree.children(node_id) {
            let child = match fo_tree.get(child_id) {
                Some(c) => c,
                None => continue,
            };

            match &child.data {
                FoNodeData::Text(text) => {
                    let (word_min, run_width) = self.measure_text_run(text, traits);
                    min = min.max(word_min);
                    run += run_width;
                }
                data if is_inline_level(data) => {
                    let child_traits = resolve_text_traits(traits, data.properties());
                    let (child_min, child_run) =
                        self.measure_inline_run(fo_tree, child_id, &child_traits);
                    min = min.max(child_min);
                    run += child_run;
                }
                data => {
                    // Block-level (or unknown container): flush the current line.
                    block_max = block_max.max(run);
                    run = Length::ZERO;
                    let child_traits = resolve_text_traits(traits, data.properties());
                    let (child_min, child_max) =
                        self.measure_block_context(fo_tree, child_id, &child_traits);
                    min = min.max(child_min);
                    block_max = block_max.max(child_max);
                }
            }
        }

        block_max = block_max.max(run);
        (min, block_max)
    }

    /// Measure an inline formatting context rooted at `node_id`, flattening the
    /// whole subtree onto one notional line.  Returns `(min, total)` where `min`
    /// is the widest unbreakable word and `total` is the un-wrapped line width.
    fn measure_inline_run(
        &self,
        fo_tree: &FoArena,
        node_id: NodeId,
        traits: &TraitSet,
    ) -> (Length, Length) {
        let mut min = Length::ZERO;
        let mut total = Length::ZERO;

        // fo:character is itself the atomic content (it has no Text children).
        if let Some(node) = fo_tree.get(node_id) {
            if let FoNodeData::Character { character, .. } = &node.data {
                let glyph = character.to_string();
                let width = measure_text_metrics(&glyph, traits, &self.font_registry);
                return (width, width);
            }
        }

        for child_id in fo_tree.children(node_id) {
            let child = match fo_tree.get(child_id) {
                Some(c) => c,
                None => continue,
            };

            match &child.data {
                FoNodeData::Text(text) => {
                    let (word_min, run_width) = self.measure_text_run(text, traits);
                    min = min.max(word_min);
                    total += run_width;
                }
                FoNodeData::Character { character, .. } => {
                    let glyph = character.to_string();
                    let child_traits = resolve_text_traits(traits, child.data.properties());
                    let width = measure_text_metrics(&glyph, &child_traits, &self.font_registry);
                    min = min.max(width);
                    total += width;
                }
                FoNodeData::Leader { .. } => {
                    // Leaders are elastic: they contribute no intrinsic width.
                }
                data => {
                    let child_traits = resolve_text_traits(traits, data.properties());
                    let (child_min, child_total) =
                        self.measure_inline_run(fo_tree, child_id, &child_traits);
                    min = min.max(child_min);
                    total += child_total;
                }
            }
        }

        (min, total)
    }

    /// Measure a single text run with the real font metrics.
    ///
    /// Returns `(min_word, full)`:
    /// * `min_word` — the widest single whitespace-delimited word (the smallest
    ///   width at which the run never overflows); and
    /// * `full` — the advance width of the entire run on one line (its max width).
    fn measure_text_run(&self, text: &str, traits: &TraitSet) -> (Length, Length) {
        if text.trim().is_empty() {
            return (Length::ZERO, Length::ZERO);
        }
        let full = measure_text_metrics(text, traits, &self.font_registry);
        let mut min_word = Length::ZERO;
        for word in text.split_whitespace() {
            let w = measure_text_metrics(word, traits, &self.font_registry);
            min_word = min_word.max(w);
        }
        (min_word, full)
    }
}

/// Resolve the font traits used for measuring a node's text, inheriting the
/// parent's font and overlaying only the font properties the node sets itself.
///
/// `PropertyList::get` falls back to a property's *initial* value when it is
/// absent, so the overlay is gated on [`PropertyList::is_explicit`]: a font
/// property is taken from the node only when the author actually set it,
/// otherwise the inherited (parent) value is kept.  This makes inheritance work
/// for both real parsed trees (which also chain their property lists) and
/// hand-built ones (which do not).
fn resolve_text_traits(parent: &TraitSet, props: Option<&fop_core::PropertyList>) -> TraitSet {
    let mut traits = parent.clone();
    if let Some(props) = props {
        let own = extract_traits(props);
        if props.is_explicit(PropertyId::FontFamily) {
            traits.font_family = own.font_family;
        }
        if props.is_explicit(PropertyId::FontSize) {
            traits.font_size = own.font_size;
        }
        if props.is_explicit(PropertyId::FontWeight) {
            traits.font_weight = own.font_weight;
        }
        if props.is_explicit(PropertyId::FontStyle) {
            traits.font_style = own.font_style;
        }
    }
    traits
}

/// Is this node laid out inline (contributing to the current line) rather than
/// starting a new block-level line?
fn is_inline_level(data: &FoNodeData) -> bool {
    matches!(
        data,
        FoNodeData::Inline { .. }
            | FoNodeData::InlineContainer { .. }
            | FoNodeData::BasicLink { .. }
            | FoNodeData::Character { .. }
            | FoNodeData::PageNumber { .. }
            | FoNodeData::PageNumberCitation { .. }
            | FoNodeData::PageNumberCitationLast { .. }
            | FoNodeData::Leader { .. }
            | FoNodeData::Wrapper { .. }
            | FoNodeData::BidiOverride { .. }
    )
}

/// Extract `(number-columns-spanned, number-rows-spanned)` for a cell, mirroring
/// the extraction used by the cell-area layout pass.
fn cell_spans(fo_tree: &FoArena, cell_id: NodeId) -> (usize, usize) {
    let props = match fo_tree.get(cell_id).and_then(|n| n.data.properties()) {
        Some(p) => p,
        None => return (1, 1),
    };
    let cols = props
        .get(PropertyId::NumberColumnsSpanned)
        .ok()
        .and_then(|v| v.as_number())
        .map(|n| n.max(1.0) as usize)
        .unwrap_or(1);
    let rows = props
        .get(PropertyId::NumberRowsSpanned)
        .ok()
        .and_then(|v| v.as_number())
        .map(|n| n.max(1.0) as usize)
        .unwrap_or(1);
    (cols, rows)
}

/// Collect the table's rows grouped by section, in visual order
/// (header, then each body, then footer).  Row spans are confined to a section,
/// so each group can be processed with its own occupancy state.
fn collect_section_rows(fo_tree: &FoArena, table_node_id: NodeId) -> Vec<Vec<NodeId>> {
    let mut header: Option<NodeId> = None;
    let mut footer: Option<NodeId> = None;
    let mut bodies: Vec<NodeId> = Vec::new();

    for child_id in fo_tree.children(table_node_id) {
        if let Some(child) = fo_tree.get(child_id) {
            match child.data {
                FoNodeData::TableHeader { .. } => header = Some(child_id),
                FoNodeData::TableFooter { .. } => footer = Some(child_id),
                FoNodeData::TableBody { .. } => bodies.push(child_id),
                _ => {}
            }
        }
    }

    let rows_of = |section_id: NodeId| -> Vec<NodeId> {
        fo_tree
            .children(section_id)
            .into_iter()
            .filter(|row_id| {
                fo_tree
                    .get(*row_id)
                    .map(|n| matches!(n.data, FoNodeData::TableRow { .. }))
                    .unwrap_or(false)
            })
            .collect()
    };

    let mut sections = Vec::new();
    if let Some(h) = header {
        sections.push(rows_of(h));
    }
    for b in bodies {
        sections.push(rows_of(b));
    }
    if let Some(f) = footer {
        sections.push(rows_of(f));
    }
    sections
}

/// The number of grid columns implied by the cells: the maximum, over all rows,
/// of the sum of `number-columns-spanned` of that row's cells.
fn max_columns(fo_tree: &FoArena, section_rows: &[Vec<NodeId>]) -> usize {
    let mut max_cols = 0usize;
    for rows in section_rows {
        for &row_id in rows {
            let mut count = 0usize;
            for cell_id in fo_tree.children(row_id) {
                let is_cell = fo_tree
                    .get(cell_id)
                    .map(|n| matches!(n.data, FoNodeData::TableCell { .. }))
                    .unwrap_or(false);
                if is_cell {
                    count += cell_spans(fo_tree, cell_id).0;
                }
            }
            max_cols = max_cols.max(count);
        }
    }
    max_cols
}

/// Raise the sum of `arr[start..start+span]` to at least `required`, distributing
/// any deficit across the covered columns (proportionally to their current
/// widths, or equally when they are all zero).  The last covered column absorbs
/// the running remainder so the post-condition `sum >= required` holds exactly.
fn distribute_span(arr: &mut [Length], start: usize, span: usize, required: Length) {
    let end = (start + span).min(arr.len());
    if start >= end {
        return;
    }
    let current = arr[start..end].iter().fold(Length::ZERO, |acc, w| acc + *w);
    if required <= current {
        return;
    }
    let deficit = required - current;
    let sum_pt = current.to_pt();
    let count = end - start;

    let mut distributed = Length::ZERO;
    for (k, i) in (start..end).enumerate() {
        let add = if k + 1 == count {
            deficit - distributed
        } else if sum_pt > 0.0 {
            Length::from_pt(deficit.to_pt() * arr[i].to_pt() / sum_pt)
        } else {
            Length::from_pt(deficit.to_pt() / count as f64)
        };
        arr[i] += add;
        distributed += add;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fop_core::{FoNode, PropertyList, PropertyValue};

    /// Build a `block → text` cell and return its node id, appended to `parent`.
    fn add_text_cell(fo: &mut FoArena<'static>, parent: NodeId, text: &str) -> NodeId {
        let cell = fo.add_node(FoNode::new(FoNodeData::TableCell {
            properties: PropertyList::new(),
        }));
        fo.append_child(parent, cell).expect("append cell");
        let block = fo.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo.append_child(cell, block).expect("append block");
        let t = fo.add_node(FoNode::new(FoNodeData::Text(text.to_string())));
        fo.append_child(block, t).expect("append text");
        cell
    }

    /// A 2-column auto table: column A has short text, column B long text.
    fn build_two_column_table(fo: &mut FoArena<'static>, short: &str, long: &str) -> NodeId {
        let mut table_props = PropertyList::new();
        // table-layout="auto" (EN_AUTO = 9)
        table_props.set(PropertyId::TableLayout, PropertyValue::Enum(9));
        let table = fo.add_node(FoNode::new(FoNodeData::Table {
            properties: table_props,
        }));

        for _ in 0..2 {
            // <fo:table-column/> with no column-width → auto column.
            let col = fo.add_node(FoNode::new(FoNodeData::TableColumn {
                properties: PropertyList::new(),
            }));
            fo.append_child(table, col).expect("append column");
        }

        let body = fo.add_node(FoNode::new(FoNodeData::TableBody {
            properties: PropertyList::new(),
        }));
        fo.append_child(table, body).expect("append body");
        let row = fo.add_node(FoNode::new(FoNodeData::TableRow {
            properties: PropertyList::new(),
        }));
        fo.append_child(body, row).expect("append row");
        add_text_cell(fo, row, short);
        add_text_cell(fo, row, long);
        table
    }

    #[test]
    fn test_measure_text_run_min_is_widest_word() {
        let engine = LayoutEngine::new();
        let traits = TraitSet::default();
        let (min, max) = engine.measure_text_run("hi there everyone", &traits);
        // The full single-line width must exceed the widest individual word.
        assert!(max > min, "max {:?} should exceed min {:?}", max, min);
        // The widest word ("everyone") is wider than the narrowest ("hi").
        let (hi_min, _) = engine.measure_text_run("hi", &traits);
        assert!(min > hi_min, "widest word should beat a 2-char word");
    }

    #[test]
    fn test_cell_min_le_max_and_positive() {
        let mut fo = FoArena::new();
        let table = fo.add_node(FoNode::new(FoNodeData::Table {
            properties: PropertyList::new(),
        }));
        let row = fo.add_node(FoNode::new(FoNodeData::TableRow {
            properties: PropertyList::new(),
        }));
        fo.append_child(table, row).expect("append row");
        let cell = add_text_cell(&mut fo, row, "longer phrase here");

        let engine = LayoutEngine::new();
        let (min, max) = engine.measure_cell_content_widths(&fo, cell);
        assert!(min > Length::ZERO, "min should be positive");
        assert!(max >= min, "max {:?} >= min {:?}", max, min);
    }

    #[test]
    fn test_auto_widths_short_lt_long_and_fill_table() {
        let mut fo = FoArena::new();
        let short = "Hi";
        let long = "a much longer cell value that needs more width";
        let table = build_two_column_table(&mut fo, short, long);

        let engine = LayoutEngine::new();
        let available = engine.page_width - Length::from_pt(144.0);
        // Mirror the Table arm: separate borders, 0pt border-spacing.
        let table_layout = TableLayout::new(available)
            .with_border_spacing(Length::ZERO)
            .with_layout_mode(crate::layout::TableLayoutMode::Auto);

        let specs = vec![ColumnWidth::Auto, ColumnWidth::Auto];
        let widths = engine.measure_auto_column_widths(&fo, table, &specs, &table_layout);

        assert_eq!(widths.len(), 2);
        // GAP 1 core guarantee: content-sized columns, narrow < wide.
        assert!(
            widths[0] < widths[1],
            "short column {:?} must be narrower than long column {:?}",
            widths[0],
            widths[1]
        );

        // Both columns are at least their min content width.
        let cells = collect_section_rows(&fo, table);
        let row = cells[0][0];
        let cell_ids: Vec<NodeId> = fo.children(row);
        let (min_a, _) = engine.measure_cell_content_widths(&fo, cell_ids[0]);
        let (min_b, _) = engine.measure_cell_content_widths(&fo, cell_ids[1]);
        assert!(widths[0] >= min_a, "col A below its min content width");
        assert!(widths[1] >= min_b, "col B below its min content width");

        // The columns fill the whole content width (border-spacing is 0).
        let total = widths[0] + widths[1];
        let target = table_layout.content_width_for_columns(2);
        assert!(
            (total.to_pt() - target.to_pt()).abs() < 0.05,
            "columns ({:?}) should sum to the table content width {:?}",
            total,
            target
        );
    }

    #[test]
    fn test_undeclared_columns_default_to_auto() {
        // No <fo:table-column> elements: the column count comes from the cells.
        let mut fo = FoArena::new();
        let table = fo.add_node(FoNode::new(FoNodeData::Table {
            properties: PropertyList::new(),
        }));
        let body = fo.add_node(FoNode::new(FoNodeData::TableBody {
            properties: PropertyList::new(),
        }));
        fo.append_child(table, body).expect("append body");
        let row = fo.add_node(FoNode::new(FoNodeData::TableRow {
            properties: PropertyList::new(),
        }));
        fo.append_child(body, row).expect("append row");
        add_text_cell(&mut fo, row, "x");
        add_text_cell(&mut fo, row, "a long stretch of words here");

        let engine = LayoutEngine::new();
        let available = engine.page_width - Length::from_pt(144.0);
        let table_layout = TableLayout::new(available)
            .with_border_spacing(Length::ZERO)
            .with_layout_mode(crate::layout::TableLayoutMode::Auto);

        // No declared specs at all.
        let widths = engine.measure_auto_column_widths(&fo, table, &[], &table_layout);
        assert_eq!(widths.len(), 2, "column count derived from cells");
        assert!(widths[0] < widths[1]);
    }

    #[test]
    fn test_colspan_raises_covered_columns() {
        // A header cell spanning two columns must lift the *sum* of the two
        // covered columns to at least the spanning cell's content width.
        let mut fo = FoArena::new();
        let mut table_props = PropertyList::new();
        table_props.set(PropertyId::TableLayout, PropertyValue::Enum(9));
        let table = fo.add_node(FoNode::new(FoNodeData::Table {
            properties: table_props,
        }));
        for _ in 0..2 {
            let col = fo.add_node(FoNode::new(FoNodeData::TableColumn {
                properties: PropertyList::new(),
            }));
            fo.append_child(table, col).expect("append column");
        }
        let body = fo.add_node(FoNode::new(FoNodeData::TableBody {
            properties: PropertyList::new(),
        }));
        fo.append_child(table, body).expect("append body");

        // Row 1: a single cell spanning both columns with a long phrase.
        let row1 = fo.add_node(FoNode::new(FoNodeData::TableRow {
            properties: PropertyList::new(),
        }));
        fo.append_child(body, row1).expect("append row1");
        let mut span_props = PropertyList::new();
        span_props.set(PropertyId::NumberColumnsSpanned, PropertyValue::Number(2.0));
        let span_cell = fo.add_node(FoNode::new(FoNodeData::TableCell {
            properties: span_props,
        }));
        fo.append_child(row1, span_cell).expect("append span cell");
        let block = fo.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo.append_child(span_cell, block).expect("append block");
        let long = "a wide spanning header that covers both columns";
        let t = fo.add_node(FoNode::new(FoNodeData::Text(long.to_string())));
        fo.append_child(block, t).expect("append text");

        // Row 2: two narrow cells.
        let row2 = fo.add_node(FoNode::new(FoNodeData::TableRow {
            properties: PropertyList::new(),
        }));
        fo.append_child(body, row2).expect("append row2");
        add_text_cell(&mut fo, row2, "a");
        add_text_cell(&mut fo, row2, "b");

        let engine = LayoutEngine::new();
        let available = engine.page_width - Length::from_pt(144.0);
        let table_layout = TableLayout::new(available)
            .with_border_spacing(Length::ZERO)
            .with_layout_mode(crate::layout::TableLayoutMode::Auto);
        let specs = vec![ColumnWidth::Auto, ColumnWidth::Auto];
        let widths = engine.measure_auto_column_widths(&fo, table, &specs, &table_layout);

        // The spanning content's max width is imposed on the two columns' sum.
        let (_, span_max) = engine.measure_cell_content_widths(&fo, span_cell);
        let total = widths[0] + widths[1];
        assert!(
            total.to_pt() + 0.05 >= span_max.to_pt(),
            "covered columns ({:?}) must hold the spanning cell max {:?}",
            total,
            span_max
        );
    }

    #[test]
    fn test_distribute_span_helper_meets_requirement() {
        let mut arr = vec![Length::from_pt(10.0), Length::from_pt(30.0)];
        distribute_span(&mut arr, 0, 2, Length::from_pt(100.0));
        let sum = arr[0] + arr[1];
        assert!((sum.to_pt() - 100.0).abs() < 0.01, "sum {:?}", sum);
        // Proportional to the originals (10:30) → col0 < col1.
        assert!(arr[0] < arr[1]);
    }

    #[test]
    fn test_font_size_widens_measurement() {
        // A larger font-size on the cell must widen the measured content.
        let mut fo = FoArena::new();
        let table = fo.add_node(FoNode::new(FoNodeData::Table {
            properties: PropertyList::new(),
        }));
        let row = fo.add_node(FoNode::new(FoNodeData::TableRow {
            properties: PropertyList::new(),
        }));
        fo.append_child(table, row).expect("append row");

        let mut big_props = PropertyList::new();
        big_props.set(
            PropertyId::FontSize,
            PropertyValue::Length(Length::from_pt(24.0)),
        );
        let big_cell = fo.add_node(FoNode::new(FoNodeData::TableCell {
            properties: big_props,
        }));
        fo.append_child(row, big_cell).expect("append big cell");
        let big_block = fo.add_node(FoNode::new(FoNodeData::Block {
            properties: PropertyList::new(),
        }));
        fo.append_child(big_cell, big_block).expect("append block");
        let bt = fo.add_node(FoNode::new(FoNodeData::Text("Word".to_string())));
        fo.append_child(big_block, bt).expect("append text");

        let small_cell = add_text_cell(&mut fo, row, "Word");

        let engine = LayoutEngine::new();
        let (_, big_max) = engine.measure_cell_content_widths(&fo, big_cell);
        let (_, small_max) = engine.measure_cell_content_widths(&fo, small_cell);
        assert!(
            big_max > small_max,
            "24pt cell ({:?}) should be wider than 12pt cell ({:?})",
            big_max,
            small_max
        );
    }
}
