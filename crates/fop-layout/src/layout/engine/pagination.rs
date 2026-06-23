//! Height-overflow pagination for page-sequence flow content.
//!
//! This module owns the live pagination path used by [`LayoutEngine::layout`].
//! It drives a per-page-sequence loop that:
//!
//! * instantiates pages from the page master geometry,
//! * repeats the static-content regions (header / footer / start / end sidebar)
//!   on every page,
//! * stacks the flow's block content in the region-body while tracking the
//!   cumulative height against the body box,
//! * starts a new page and **reparents** the overflowing block (and any blocks
//!   glued to it by keep constraints) onto the new page's region-body when the
//!   next block would not fit, and
//! * honours forced `break-before` / `break-after` page breaks (including the
//!   even/odd-page variants).
//!
//! Block geometry is stored relative to its parent region, so relocating a
//! block between pages only requires fixing the block's own `y` — its laid-out
//! descendants keep their region-relative positions (see
//! [`AreaTree::reparent`](crate::area::AreaTree::reparent)).
//!
//! Multi-column flows (`column-count > 1`) are paginated by a sibling driver,
//! [`LayoutEngine::paginate_multicolumn_flow`], which fills the columns of each
//! page left to right and starts a new page (reusing the same page-construction
//! machinery, so static content repeats) once the last column is full.  Because
//! the destination page is chosen *before* each block is emitted, multi-column
//! blocks are born under the correct region-body and need no reparenting.  Full
//! newspaper-style column *balancing* is intentionally out of scope.

use crate::area::{Area, AreaId, AreaTree, AreaType, TraitSet};
use crate::layout::PageNumberResolver;
use crate::layout::{
    extract_break_after, extract_break_before, extract_clear, extract_column_count,
    extract_column_gap, extract_space_after, extract_space_before, extract_traits, BreakValue,
    PageBreaker,
};
use fop_core::{FoArena, FoNodeData, NodeId, PropertyId};
use fop_types::{Length, Point, Rect, Result, Size};

use super::markers::{collect_sequence_markers, DocumentMarkerState, PageMarkerView};
use super::types::{FloatManager, MultiColumnLayout, PageRegionGeometry};
use super::LayoutEngine;

/// Owned, borrow-free description of everything needed to build each page of a
/// single `fo:page-sequence`. Built once up front so the pagination loop never
/// holds a borrow into the FO tree across the area-tree mutations.
struct SequenceLayout {
    /// Page/region geometry resolved from the page master.
    geom: PageRegionGeometry,
    /// Traits applied to every page area (e.g. page background colour).
    page_traits: TraitSet,
    /// Traits applied to every region-body area (e.g. inherited text colour).
    body_traits: TraitSet,
    /// `static-content` node for `xsl-region-before` (header), if present.
    static_before: Option<NodeId>,
    /// `static-content` node for `xsl-region-after` (footer), if present.
    static_after: Option<NodeId>,
    /// `static-content` node for `xsl-region-start` (start sidebar), if present.
    static_start: Option<NodeId>,
    /// `static-content` node for `xsl-region-end` (end sidebar), if present.
    static_end: Option<NodeId>,
}

/// The flow-control measurements of a single multi-column block, pulled off its
/// FO node up front so the multi-column paginator can decide column/page breaks
/// without holding a borrow into the FO tree across area-tree mutations.
///
/// These mirror exactly the values the single-page placement primitive
/// ([`LayoutEngine::emit_multicolumn_block`]) and column flow-control
/// ([`LayoutEngine::layout_block_multicolumn`]) derive, so the paginated and
/// single-page column paths agree on where a column fills.
struct MultiColumnMetrics {
    /// `space-before` applied before the block.
    space_before: Length,
    /// `space-after` applied after the block.
    space_after: Length,
    /// Resolved per-line height (the height reserved for the column-fit check).
    line_height: Length,
    /// Forced `break-before` value.
    break_before: BreakValue,
    /// Forced `break-after` value.
    break_after: BreakValue,
}

impl LayoutEngine {
    /// Lay out a whole `fo:page-sequence`, producing one or more page areas.
    ///
    /// Returns the id of the **first** page area (the page-sequence's principal
    /// area, used for id registration). Subsequent pages are appended to the
    /// area tree as additional top-level `Page` areas.
    pub(in crate::layout::engine) fn layout_page_sequence(
        &self,
        fo_tree: &FoArena,
        node_id: NodeId,
        area_tree: &mut AreaTree,
        resolver: &mut PageNumberResolver,
        doc_markers: &mut DocumentMarkerState,
    ) -> Result<Option<AreaId>> {
        let node = fo_tree
            .get(node_id)
            .ok_or_else(|| fop_types::FopError::Generic(format!("Node {} not found", node_id)))?;

        let (master_reference, node_user_id, page_traits) = match &node.data {
            FoNodeData::PageSequence {
                properties,
                master_reference,
                ..
            } => {
                let mut page_traits = TraitSet::default();
                if let Ok(color) = properties.get(PropertyId::BackgroundColor) {
                    page_traits.background_color = color.as_color();
                }
                (master_reference.clone(), node.id.clone(), page_traits)
            }
            _ => return Ok(None),
        };

        let geom = self.extract_page_region_geometry(fo_tree, &master_reference);

        // Classify the page-sequence children into the static-content regions
        // and the principal flow.
        let mut static_before = None;
        let mut static_after = None;
        let mut static_start = None;
        let mut static_end = None;
        let mut flow_id = None;
        for child_id in fo_tree.children(node_id) {
            if let Some(child) = fo_tree.get(child_id) {
                match &child.data {
                    FoNodeData::StaticContent { flow_name, .. } => match flow_name.as_str() {
                        "xsl-region-before" => static_before = Some(child_id),
                        "xsl-region-after" => static_after = Some(child_id),
                        "xsl-region-start" => static_start = Some(child_id),
                        "xsl-region-end" => static_end = Some(child_id),
                        _ => {}
                    },
                    FoNodeData::Flow { .. } => flow_id = Some(child_id),
                    _ => {}
                }
            }
        }

        // Body region inherits the flow's text colour (matching the previous
        // single-page behaviour).
        let mut body_traits = TraitSet::default();
        if let Some(flow_node_id) = flow_id {
            if let Some(flow_node) = fo_tree.get(flow_node_id) {
                if let FoNodeData::Flow { properties, .. } = &flow_node.data {
                    if let Ok(color) = properties.get(PropertyId::Color) {
                        body_traits.color = color.as_color();
                    }
                }
            }
        }

        let seq = SequenceLayout {
            geom,
            page_traits,
            body_traits,
            static_before,
            static_after,
            static_start,
            static_end,
        };

        // The page number the sequence opens on (the resolver is positioned on
        // it by the previous sequence / its initial value of 1).  Pages are
        // created sequentially incrementing the resolver by exactly one each,
        // so page `i` of this sequence carries page number `first_page_number +
        // i` — used below to re-establish the per-page number when the deferred
        // static content (running headers/footers) is laid out.
        let first_page_number = resolver.current_page();

        // PASS 1 — flow.  Build the pages and lay out the flow with
        // height-overflow pagination, recording each placed top-level flow item
        // (`placements`) so the markers it carries can later be attributed to
        // the page it actually landed on.  Static content is intentionally NOT
        // laid out yet: a page's running header needs the markers that occur in
        // that page's flow, which are only known once the flow is placed.
        let mut page_ids: Vec<AreaId> = Vec::new();
        let mut placements: Vec<(AreaId, NodeId)> = Vec::new();
        let (first_page_id, first_region_id) = self.new_page_with_regions(area_tree, &seq)?;
        page_ids.push(first_page_id);

        // Register the page-sequence's own id against its first page.
        if let Some(id) = &node_user_id {
            resolver.register_element(id.clone(), first_page_id);
        }

        if let Some(flow_node_id) = flow_id {
            self.paginate_flow(
                fo_tree,
                flow_node_id,
                area_tree,
                &seq,
                first_region_id,
                &mut page_ids,
                resolver,
                &mut placements,
            )?;
        }

        // Place collected footnotes at the bottom of every page's body region.
        for page_id in &page_ids {
            self.place_footnotes_for_page(area_tree, *page_id, seq.geom.body_rect)?;
        }

        // Build the page-accurate marker context from the finished flow.
        let seq_markers = collect_sequence_markers(
            fo_tree,
            area_tree,
            &page_ids,
            &placements,
            doc_markers.snapshot(),
        );

        // PASS 2 — static content.  Now that every page's markers are known,
        // lay out the repeated static-content regions on each page, resolving
        // `fo:retrieve-marker` against that page's marker context.  The page
        // number is re-established first so per-page `fo:page-number` resolves
        // correctly in headers/footers.
        for (page_idx, page_id) in page_ids.iter().enumerate() {
            resolver.set_current_page(first_page_number + page_idx);
            let view = PageMarkerView::new(&seq_markers, page_idx);
            self.layout_page_static_content(fo_tree, area_tree, &seq, *page_id, resolver, &view)?;
        }

        // Carry the markers in effect at the sequence end into the next
        // sequence (for `retrieve-boundary="document"`).
        doc_markers.set_trailing(seq_markers.trailing_markers());

        // Advance the page counter to the first page of the next sequence.
        resolver.set_current_page(first_page_number + page_ids.len());

        Ok(Some(first_page_id))
    }

    /// Lay out a single page's repeated static-content regions (header / footer
    /// / start & end sidebars), resolving each `fo:retrieve-marker` against
    /// `view` (this page's marker context), then restore the canonical page
    /// child order (static regions, then region-body, then footnotes).
    ///
    /// Static content is laid out here — after the flow — rather than at page
    /// construction, because the markers a page's header retrieves are only
    /// known once that page's flow has been placed.  The reorder compensates for
    /// the append-only area tree so the static areas precede the region-body
    /// exactly as if they had been built first.
    fn layout_page_static_content(
        &self,
        fo_tree: &FoArena,
        area_tree: &mut AreaTree,
        seq: &SequenceLayout,
        page_id: AreaId,
        resolver: &mut PageNumberResolver,
        view: &PageMarkerView,
    ) -> Result<()> {
        let geom = &seq.geom;
        if let Some(header_id) = seq.static_before {
            self.layout_static_content_in_rect(
                fo_tree,
                header_id,
                area_tree,
                page_id,
                geom.before_rect,
                AreaType::Header,
                resolver,
                view,
            )?;
        }
        if let Some(footer_id) = seq.static_after {
            self.layout_static_content_in_rect(
                fo_tree,
                footer_id,
                area_tree,
                page_id,
                geom.after_rect,
                AreaType::Footer,
                resolver,
                view,
            )?;
        }
        if let Some(start_id) = seq.static_start {
            self.layout_static_content_in_rect(
                fo_tree,
                start_id,
                area_tree,
                page_id,
                geom.start_rect,
                AreaType::SidebarStart,
                resolver,
                view,
            )?;
        }
        if let Some(end_id) = seq.static_end {
            self.layout_static_content_in_rect(
                fo_tree,
                end_id,
                area_tree,
                page_id,
                geom.end_rect,
                AreaType::SidebarEnd,
                resolver,
                view,
            )?;
        }

        area_tree.reorder_children(page_id, page_child_priority);
        Ok(())
    }

    /// Create a new top-level page area and its (empty) region-body.
    ///
    /// Returns `(page_id, region_body_id)`. The repeated static content
    /// (headers / footers / sidebars) is **not** built here: it is laid out in
    /// a second pass once the flow's per-page markers are known (see
    /// [`Self::layout_page_static_content`]).
    fn new_page_with_regions(
        &self,
        area_tree: &mut AreaTree,
        seq: &SequenceLayout,
    ) -> Result<(AreaId, AreaId)> {
        let geom = &seq.geom;

        let page_rect =
            Rect::from_point_size(Point::ZERO, Size::new(geom.page_width, geom.page_height));
        let page_area = Area::new(AreaType::Page, page_rect).with_traits(seq.page_traits.clone());
        let page_id = area_tree.add_area(page_area);

        // Region-body that will receive flow content.
        let region =
            Area::new(AreaType::Region, geom.body_rect).with_traits(seq.body_traits.clone());
        let region_id = area_tree.add_area(region);
        area_tree
            .append_child(page_id, region_id)
            .map_err(fop_types::FopError::Generic)?;

        Ok((page_id, region_id))
    }

    /// Lay out a flow's block children with height-overflow pagination.
    #[allow(clippy::too_many_arguments)]
    fn paginate_flow(
        &self,
        fo_tree: &FoArena,
        flow_node_id: NodeId,
        area_tree: &mut AreaTree,
        seq: &SequenceLayout,
        first_region_id: AreaId,
        page_ids: &mut Vec<AreaId>,
        resolver: &mut PageNumberResolver,
        placements: &mut Vec<(AreaId, NodeId)>,
    ) -> Result<()> {
        let flow_node = fo_tree.get(flow_node_id).ok_or_else(|| {
            fop_types::FopError::Generic(format!("Flow node {} not found", flow_node_id))
        })?;
        let column_count = match &flow_node.data {
            FoNodeData::Flow { properties, .. } => extract_column_count(properties),
            _ => return Ok(()),
        };
        let column_gap = match &flow_node.data {
            FoNodeData::Flow { properties, .. } => extract_column_gap(properties),
            _ => Length::ZERO,
        };

        let body_rect = seq.geom.body_rect;
        let body_width = body_rect.width;
        let body_height = body_rect.height;
        let children = fo_tree.children(flow_node_id);

        // Multi-column flows are paginated across pages: columns fill left to
        // right on each page, and a new page is started (repeating static
        // content) once the last column of the current page is full.  Handled by
        // a sibling driver that shares this module's page-construction machinery.
        if column_count > 1 {
            return self.paginate_multicolumn_flow(
                fo_tree,
                &children,
                column_count,
                column_gap,
                area_tree,
                seq,
                first_region_id,
                page_ids,
                resolver,
                placements,
            );
        }

        // Single-column paginated layout. `page_blocks` tracks the blocks placed
        // on the *current* page (in order, floats excluded) for keep-group
        // migration; `current_y` is the region-relative stacking cursor.
        let breaker = self.page_breaker_for(&seq.geom);
        let mut current_region_id = first_region_id;
        let mut page_blocks: Vec<AreaId> = Vec::new();
        let mut current_y = Length::ZERO;
        let mut float_manager = FloatManager::new();

        for child_id in children {
            float_manager.remove_floats_above(current_y);

            // Pull everything we need off the FO node up front so we hold no
            // borrow into `fo_tree` across the area-tree mutations below.
            let (is_float, break_before, break_after) = match fo_tree.get(child_id) {
                Some(n) => {
                    let is_float = matches!(n.data, FoNodeData::Float { .. });
                    let (bb, ba) = match n.data.properties() {
                        Some(p) => (extract_break_before(p), extract_break_after(p)),
                        None => (BreakValue::Auto, BreakValue::Auto),
                    };
                    (is_float, bb, ba)
                }
                None => continue,
            };

            if is_float {
                let is_odd_page = resolver.current_page() % 2 == 1;
                if let Some(float_area_id) = self.layout_float_in_flow(
                    fo_tree,
                    child_id,
                    area_tree,
                    current_region_id,
                    current_y,
                    body_width,
                    is_odd_page,
                    &mut float_manager,
                    resolver,
                )? {
                    // A float's content can also carry markers; attribute them to
                    // the page the float landed on.
                    placements.push((float_area_id, child_id));
                }
                continue;
            }

            // Forced break-before: move to a fresh page before laying out this
            // block (only if the current page already has content).
            if break_before.forces_page_break()
                && (!page_blocks.is_empty() || current_y > Length::ZERO)
            {
                current_region_id = self.start_new_page_for_break(
                    area_tree,
                    seq,
                    page_ids,
                    resolver,
                    break_before,
                )?;
                page_blocks.clear();
                current_y = Length::ZERO;
                float_manager.clear();
            }

            // Apply `clear`: advance past active floats if requested.
            if let Some(props) = fo_tree.get(child_id).and_then(|n| n.data.properties()) {
                current_y = float_manager.get_clear_position(extract_clear(props), current_y);
            }

            // Float-aware horizontal offset and available width.
            let (left_offset, avail_width) = float_manager.available_width(current_y, body_width);

            let block_id_opt = self.layout_block_float_aware(
                fo_tree,
                child_id,
                area_tree,
                current_region_id,
                current_y,
                avail_width,
                left_offset,
                resolver,
            )?;

            if let Some(block_id) = block_id_opt {
                page_blocks.push(block_id);
                placements.push((block_id, child_id));

                let (block_y, block_h) = match area_tree.get(block_id) {
                    Some(n) => (n.area.geometry.y, n.area.height()),
                    None => (current_y, Length::ZERO),
                };
                let block_bottom = block_y + block_h;

                // Overflow when the block's bottom exceeds the body box and the
                // page already holds earlier content (a block can never overflow
                // off an empty page — it would just clip, see block-splitting
                // deferral in the report).
                if block_bottom > body_height && current_y > Length::ZERO {
                    // Determine the keep-group: the trailing run of blocks that
                    // must travel together to the next page.
                    let group_start = keep_group_start(&breaker, area_tree, &page_blocks);
                    // If the whole page is one glued group we cannot honour the
                    // keep without overflowing; fall back to moving the last
                    // block alone so layout still makes progress.
                    let effective_start = if group_start == 0 && page_blocks.len() > 1 {
                        page_blocks.len() - 1
                    } else {
                        group_start
                    };

                    let group: Vec<AreaId> = page_blocks[effective_start..].to_vec();

                    current_region_id = self.start_new_page(area_tree, seq, page_ids, resolver)?;
                    float_manager.clear();

                    current_y = migrate_blocks(area_tree, &group, current_region_id)?;
                    page_blocks = group;
                } else {
                    current_y = block_bottom;
                }
            }

            // Forced break-after: subsequent content starts on a fresh page.
            if break_after.forces_page_break() {
                current_region_id =
                    self.start_new_page_for_break(area_tree, seq, page_ids, resolver, break_after)?;
                page_blocks.clear();
                current_y = Length::ZERO;
                float_manager.clear();
            }
        }

        float_manager.clear();
        Ok(())
    }

    /// Lay out a multi-column flow with cross-page pagination.
    ///
    /// Columns of the current page fill left to right; when a block does not fit
    /// in the current column the cursor advances to the next column, and when the
    /// **last** column of the page is full a new page is started (via
    /// [`Self::start_new_page`], so static content repeats) and the flow resumes
    /// in the first column of the new page's region-body.  Forced page breaks
    /// (`break-before` / `break-after` = `page` / `even-page` / `odd-page`) start
    /// a fresh page (mirroring the single-column path); column breaks advance to
    /// the next column (or a new page when the last column is reached).
    ///
    /// Blocks are placed directly under the region-body of the page they belong
    /// to, so their geometry is page-relative and correct without any later
    /// reparenting (the page is chosen *before* the block is emitted).
    #[allow(clippy::too_many_arguments)]
    fn paginate_multicolumn_flow(
        &self,
        fo_tree: &FoArena,
        children: &[NodeId],
        column_count: i32,
        column_gap: Length,
        area_tree: &mut AreaTree,
        seq: &SequenceLayout,
        first_region_id: AreaId,
        page_ids: &mut Vec<AreaId>,
        resolver: &mut PageNumberResolver,
        placements: &mut Vec<(AreaId, NodeId)>,
    ) -> Result<()> {
        let body_rect = seq.geom.body_rect;
        let mut multi_col = MultiColumnLayout::new(column_count, column_gap, body_rect.width)
            .with_max_height(body_rect.height);
        let mut current_region_id = first_region_id;

        for &child_id in children {
            // Non-block flow children (e.g. floats/tables) have no column
            // flow-control; emit them in place (the placement primitive returns
            // `None` for them, matching the legacy single-page behaviour).
            let metrics = match self.measure_multicolumn_block(fo_tree, child_id) {
                Some(metrics) => metrics,
                None => {
                    if let Some(area_id) = self.emit_multicolumn_block(
                        fo_tree,
                        child_id,
                        area_tree,
                        current_region_id,
                        &mut multi_col,
                        resolver,
                    )? {
                        placements.push((area_id, child_id));
                    }
                    continue;
                }
            };

            // Forced break-before. A page break starts a fresh page (only when the
            // current page already holds content); a column break advances to the
            // next column (starting a new page when the last column is reached).
            if metrics.break_before.forces_page_break() && multicolumn_page_has_content(&multi_col)
            {
                current_region_id = self.start_new_page_for_break(
                    area_tree,
                    seq,
                    page_ids,
                    resolver,
                    metrics.break_before,
                )?;
                multi_col.reset();
            } else if matches!(metrics.break_before, BreakValue::Column)
                && multi_col.column_y > Length::ZERO
            {
                current_region_id = self.advance_multicolumn(
                    area_tree,
                    seq,
                    page_ids,
                    resolver,
                    &mut multi_col,
                    current_region_id,
                )?;
            }

            // Apply space-before, then decide the column fit.  This mirrors the
            // single-page column placement exactly, extended so that a full last
            // column starts a new page rather than overflowing.
            multi_col.allocate(metrics.space_before);
            let total_height = metrics.space_before + metrics.line_height + metrics.space_after;
            if multi_col.is_column_filled(total_height) {
                current_region_id = self.advance_multicolumn(
                    area_tree,
                    seq,
                    page_ids,
                    resolver,
                    &mut multi_col,
                    current_region_id,
                )?;
            }

            // Place the block in the current column of the current page.
            if let Some(area_id) = self.emit_multicolumn_block(
                fo_tree,
                child_id,
                area_tree,
                current_region_id,
                &mut multi_col,
                resolver,
            )? {
                placements.push((area_id, child_id));
            }

            // Apply space-after.
            multi_col.allocate(metrics.space_after);

            // Forced break-after: a page break starts a fresh page for the next
            // block; a column break advances to the next column.
            if metrics.break_after.forces_page_break() {
                current_region_id = self.start_new_page_for_break(
                    area_tree,
                    seq,
                    page_ids,
                    resolver,
                    metrics.break_after,
                )?;
                multi_col.reset();
            } else if matches!(metrics.break_after, BreakValue::Column) {
                current_region_id = self.advance_multicolumn(
                    area_tree,
                    seq,
                    page_ids,
                    resolver,
                    &mut multi_col,
                    current_region_id,
                )?;
            }
        }

        Ok(())
    }

    /// Advance the multi-column cursor to the next column, or — when the current
    /// column is the last one on the page — start a new page (repeating static
    /// content) and reset to its first column.  Returns the region-body the flow
    /// should continue placing blocks into.
    fn advance_multicolumn(
        &self,
        area_tree: &mut AreaTree,
        seq: &SequenceLayout,
        page_ids: &mut Vec<AreaId>,
        resolver: &mut PageNumberResolver,
        multi_col: &mut MultiColumnLayout,
        current_region_id: AreaId,
    ) -> Result<AreaId> {
        if multi_col.next_column() {
            // Moved to the next column on the same page.
            Ok(current_region_id)
        } else {
            // The last column was full: start a new page and reset to column one.
            let new_region = self.start_new_page(area_tree, seq, page_ids, resolver)?;
            multi_col.reset();
            Ok(new_region)
        }
    }

    /// Pull the column flow-control measurements off a flow child, or `None` when
    /// the child is not a block (and therefore takes part in no column logic).
    ///
    /// The resolved values mirror exactly those derived by the single-page column
    /// placement so the paginated and single-page paths agree on column fill.
    fn measure_multicolumn_block(
        &self,
        fo_tree: &FoArena,
        node_id: NodeId,
    ) -> Option<MultiColumnMetrics> {
        let node = fo_tree.get(node_id)?;
        let properties = match &node.data {
            FoNodeData::Block { properties } | FoNodeData::BlockContainer { properties } => {
                properties
            }
            _ => return None,
        };
        let traits = extract_traits(properties);
        let line_height = traits
            .line_height
            .or(traits.font_size)
            .unwrap_or(Length::from_pt(12.0));
        Some(MultiColumnMetrics {
            space_before: extract_space_before(properties),
            space_after: extract_space_after(properties),
            line_height,
            break_before: extract_break_before(properties),
            break_after: extract_break_after(properties),
        })
    }

    /// Increment the page counter, build the next page (empty region-body, no
    /// static content yet), record it, and return its region-body id.
    fn start_new_page(
        &self,
        area_tree: &mut AreaTree,
        seq: &SequenceLayout,
        page_ids: &mut Vec<AreaId>,
        resolver: &mut PageNumberResolver,
    ) -> Result<AreaId> {
        resolver.set_current_page(resolver.current_page() + 1);
        let (page_id, region_id) = self.new_page_with_regions(area_tree, seq)?;
        page_ids.push(page_id);
        Ok(region_id)
    }

    /// Like [`Self::start_new_page`], but additionally inserts a blank page when
    /// the break requires a specific page parity (`break-before/after =
    /// even-page | odd-page`) that the freshly created page does not satisfy.
    fn start_new_page_for_break(
        &self,
        area_tree: &mut AreaTree,
        seq: &SequenceLayout,
        page_ids: &mut Vec<AreaId>,
        resolver: &mut PageNumberResolver,
        break_value: BreakValue,
    ) -> Result<AreaId> {
        let mut region_id = self.start_new_page(area_tree, seq, page_ids, resolver)?;

        let current_is_odd = resolver.current_page() % 2 == 1;
        let needs_extra = (break_value.requires_even_page() && current_is_odd)
            || (break_value.requires_odd_page() && !current_is_odd);
        if needs_extra {
            // The just-created page becomes a blank intermediate; content lands
            // on the following (correct-parity) page.
            region_id = self.start_new_page(area_tree, seq, page_ids, resolver)?;
        }

        Ok(region_id)
    }

    /// Build a [`PageBreaker`] carrying this sequence's page geometry, used for
    /// its keep-constraint break decisions (`can_break_before`).
    fn page_breaker_for(&self, geom: &PageRegionGeometry) -> PageBreaker {
        let body = geom.body_rect;
        let margin_top = body.y;
        let margin_left = body.x;
        let margin_bottom = geom.page_height - body.y - body.height;
        let margin_right = geom.page_width - body.x - body.width;
        PageBreaker::new(
            geom.page_width,
            geom.page_height,
            [margin_top, margin_right, margin_bottom, margin_left],
        )
    }
}

/// Whether the current page already holds multi-column content.  A forced page
/// break before the very first block of a page (first column, top) must be a
/// no-op — otherwise it would emit a spurious blank leading page — exactly as
/// the single-column path only breaks when the page already has content.
fn multicolumn_page_has_content(multi_col: &MultiColumnLayout) -> bool {
    multi_col.current_column > 0 || multi_col.column_y > Length::ZERO
}

/// Canonical sibling order of a page's direct child areas, used to restore the
/// natural order after the deferred static content is appended.  Static regions
/// (header / footer / sidebars) paint first, then the region-body, then any
/// footnote separator and footnote areas — matching the order produced when
/// static content was built before the flow.
fn page_child_priority(area_type: AreaType) -> u8 {
    match area_type {
        AreaType::Header | AreaType::Footer | AreaType::SidebarStart | AreaType::SidebarEnd => 0,
        AreaType::Region | AreaType::Column => 1,
        _ => 2,
    }
}

/// Find the smallest index `s` such that `blocks[s..]` must move to the next
/// page together, i.e. for every `k` in `(s, len)` a break before `blocks[k]`
/// is forbidden by a keep constraint. Scans backward from the last block.
fn keep_group_start(breaker: &PageBreaker, area_tree: &AreaTree, blocks: &[AreaId]) -> usize {
    let n = blocks.len();
    if n == 0 {
        return 0;
    }
    let mut start = n - 1;
    while start > 0 {
        // A legal break before `blocks[start]` ends the glued run.
        if breaker.can_break_before(area_tree, blocks[start], start, blocks) {
            break;
        }
        start -= 1;
    }
    start
}

/// Reparent each block in `group` (in order) under `new_region`, restacking
/// them from the body top while preserving the original inter-block spacing.
/// Returns the resulting stacking cursor (the new page's `current_y`).
fn migrate_blocks(
    area_tree: &mut AreaTree,
    group: &[AreaId],
    new_region: AreaId,
) -> Result<Length> {
    let mut new_y = Length::ZERO;
    let mut prev_old_bottom: Option<Length> = None;

    for &block_id in group {
        let (old_y, height) = match area_tree.get(block_id) {
            Some(n) => (n.area.geometry.y, n.area.height()),
            None => continue,
        };

        // Preserve the gap (space-before) that separated this block from its
        // predecessor; the first block of the group drops its leading space and
        // sits flush at the body top.
        let gap = match prev_old_bottom {
            Some(prev_bottom) => (old_y - prev_bottom).max(Length::ZERO),
            None => Length::ZERO,
        };
        new_y += gap;

        area_tree
            .reparent(block_id, new_region)
            .map_err(fop_types::FopError::Generic)?;
        if let Some(node) = area_tree.get_mut(block_id) {
            node.area.geometry.y = new_y;
        }

        new_y += height;
        prev_old_bottom = Some(old_y + height);
    }

    Ok(new_y)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::area::AreaType;
    use fop_core::{FoNode, PropertyList, PropertyValue};
    use std::borrow::Cow;

    /// Specification for one flow block: its (line-)height in points and an
    /// optional forced `break-before` value (e.g. `"page"`).
    struct BlockSpec {
        height_pt: f64,
        break_before: Option<&'static str>,
    }

    fn block(height_pt: f64) -> BlockSpec {
        BlockSpec {
            height_pt,
            break_before: None,
        }
    }

    fn block_break_before(height_pt: f64, value: &'static str) -> BlockSpec {
        BlockSpec {
            height_pt,
            break_before: Some(value),
        }
    }

    /// Build a single-page-sequence document with an explicit simple-page-master
    /// so the region geometry is fully deterministic.
    ///
    /// * `page_w_pt` / `page_h_pt` — page size; all four page margins are 0.
    /// * `before_extent_pt` — `region-before` extent (0 ⇒ no header region).
    /// * `header` — if true, a `static-content` (region-before) with one block.
    /// * `blocks` — the flow's blocks; each block's height equals its line-height
    ///   (the blocks carry no text, so block height == resolved line-height).
    fn build_doc(
        page_w_pt: f64,
        page_h_pt: f64,
        before_extent_pt: f64,
        header: bool,
        blocks: &[BlockSpec],
    ) -> FoArena<'static> {
        build_doc_columns(
            page_w_pt,
            page_h_pt,
            before_extent_pt,
            header,
            1,
            0.0,
            blocks,
        )
    }

    /// Like [`build_doc`], but sets `column-count` / `column-gap` on the flow so
    /// the multi-column paginator is exercised.  `column_count <= 1` yields a
    /// single-column flow identical to [`build_doc`].
    #[allow(clippy::too_many_arguments)]
    fn build_doc_columns(
        page_w_pt: f64,
        page_h_pt: f64,
        before_extent_pt: f64,
        header: bool,
        column_count: i32,
        column_gap_pt: f64,
        blocks: &[BlockSpec],
    ) -> FoArena<'static> {
        let mut fo = FoArena::new();
        let root = fo.add_node(FoNode::new(FoNodeData::Root));

        // --- layout-master-set / simple-page-master ---
        let lms = fo.add_node(FoNode::new(FoNodeData::LayoutMasterSet));
        fo.append_child(root, lms).expect("test: append lms");

        let mut spm_props = PropertyList::new();
        spm_props.set(
            PropertyId::PageWidth,
            PropertyValue::Length(Length::from_pt(page_w_pt)),
        );
        spm_props.set(
            PropertyId::PageHeight,
            PropertyValue::Length(Length::from_pt(page_h_pt)),
        );
        for margin in [
            PropertyId::MarginTop,
            PropertyId::MarginBottom,
            PropertyId::MarginLeft,
            PropertyId::MarginRight,
        ] {
            spm_props.set(margin, PropertyValue::Length(Length::ZERO));
        }
        let spm = fo.add_node(FoNode::new(FoNodeData::SimplePageMaster {
            master_name: "pm".to_string(),
            properties: spm_props,
        }));
        fo.append_child(lms, spm).expect("test: append spm");

        if before_extent_pt > 0.0 {
            let mut rb_props = PropertyList::new();
            rb_props.set(
                PropertyId::Extent,
                PropertyValue::Length(Length::from_pt(before_extent_pt)),
            );
            let rb = fo.add_node(FoNode::new(FoNodeData::RegionBefore {
                properties: rb_props,
            }));
            fo.append_child(spm, rb)
                .expect("test: append region-before");
        }
        let body = fo.add_node(FoNode::new(FoNodeData::RegionBody {
            properties: PropertyList::new(),
        }));
        fo.append_child(spm, body)
            .expect("test: append region-body");

        // --- page-sequence ---
        let ps = fo.add_node(FoNode::new(FoNodeData::PageSequence {
            master_reference: "pm".to_string(),
            format: "1".to_string(),
            grouping_separator: None,
            grouping_size: None,
            properties: PropertyList::new(),
        }));
        fo.append_child(root, ps)
            .expect("test: append page-sequence");

        if header {
            let sc = fo.add_node(FoNode::new(FoNodeData::StaticContent {
                flow_name: "xsl-region-before".to_string(),
                properties: PropertyList::new(),
            }));
            fo.append_child(ps, sc)
                .expect("test: append static-content");
            let mut hp = PropertyList::new();
            hp.set(
                PropertyId::LineHeight,
                PropertyValue::Length(Length::from_pt(20.0)),
            );
            let hb = fo.add_node(FoNode::new(FoNodeData::Block { properties: hp }));
            fo.append_child(sc, hb).expect("test: append header block");
            let ht = fo.add_node(FoNode::new(FoNodeData::Text("HEADER".to_string())));
            fo.append_child(hb, ht).expect("test: append header text");
        }

        let mut flow_props = PropertyList::new();
        if column_count > 1 {
            flow_props.set(
                PropertyId::ColumnCount,
                PropertyValue::Integer(column_count),
            );
            flow_props.set(
                PropertyId::ColumnGap,
                PropertyValue::Length(Length::from_pt(column_gap_pt)),
            );
        }
        let flow = fo.add_node(FoNode::new(FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: flow_props,
        }));
        fo.append_child(ps, flow).expect("test: append flow");

        for spec in blocks {
            let mut bp = PropertyList::new();
            bp.set(
                PropertyId::LineHeight,
                PropertyValue::Length(Length::from_pt(spec.height_pt)),
            );
            if let Some(bb) = spec.break_before {
                bp.set(
                    PropertyId::BreakBefore,
                    PropertyValue::String(Cow::Borrowed(bb)),
                );
            }
            let b = fo.add_node(FoNode::new(FoNodeData::Block { properties: bp }));
            fo.append_child(flow, b).expect("test: append flow block");
        }

        fo
    }

    /// All top-level `Page` areas in tree order.
    fn page_ids(tree: &AreaTree) -> Vec<AreaId> {
        tree.iter()
            .filter(|(_, node)| node.area.area_type == AreaType::Page)
            .map(|(id, _)| id)
            .collect()
    }

    /// The region-body of a page (its single `Region`-typed child).
    fn region_of(tree: &AreaTree, page_id: AreaId) -> AreaId {
        tree.children(page_id)
            .into_iter()
            .find(|c| {
                tree.get(*c)
                    .map(|n| n.area.area_type == AreaType::Region)
                    .unwrap_or(false)
            })
            .expect("test: page must have a region-body")
    }

    /// The `Block` children of a region-body.
    fn blocks_of(tree: &AreaTree, region_id: AreaId) -> Vec<AreaId> {
        tree.children(region_id)
            .into_iter()
            .filter(|c| {
                tree.get(*c)
                    .map(|n| n.area.area_type == AreaType::Block)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Tall flow ⇒ ≥2 pages, real reparenting, and every page's content fits
    /// inside its region-body.
    ///
    /// Geometry: page 200×250pt, margins 0, no header ⇒ body-rect = 200×250pt.
    /// Blocks: 4 × 100pt. Two blocks fill 200pt (≤250); a third would reach
    /// 300pt (>250) ⇒ 2 blocks per page ⇒ ceil(4/2) = 2 pages.
    #[test]
    fn test_overflow_produces_multiple_pages_with_reparenting() {
        let doc = build_doc(
            200.0,
            250.0,
            0.0,
            false,
            &[block(100.0), block(100.0), block(100.0), block(100.0)],
        );
        let engine = LayoutEngine::new();
        let tree = engine.layout(&doc).expect("test: layout should succeed");

        let pages = page_ids(&tree);
        assert_eq!(
            pages.len(),
            2,
            "4×100pt blocks in a 250pt body must paginate to 2 pages"
        );

        // Every page's region-body content must fit within the body height.
        let mut total_blocks = 0;
        for page_id in &pages {
            let region_id = region_of(&tree, *page_id);
            let body_height = tree
                .get(region_id)
                .expect("test: region exists")
                .area
                .height();
            for block_id in blocks_of(&tree, region_id) {
                let b = tree.get(block_id).expect("test: block exists");
                let bottom = b.area.geometry.y + b.area.height();
                assert!(
                    bottom <= body_height,
                    "block bottom {}pt must not exceed body height {}pt",
                    bottom.to_pt(),
                    body_height.to_pt()
                );
                total_blocks += 1;
            }
        }
        assert_eq!(total_blocks, 4, "all 4 blocks must be placed exactly once");

        // Real reparenting: the second page's blocks must genuinely live under
        // the second page's region-body (verified through the parent links).
        let page2 = pages[1];
        let region2 = region_of(&tree, page2);
        let page2_blocks = blocks_of(&tree, region2);
        assert_eq!(page2_blocks.len(), 2, "page 2 holds the 2 overflow blocks");
        for block_id in page2_blocks {
            let block = tree.get(block_id).expect("test: block exists");
            assert_eq!(
                block.parent,
                Some(region2),
                "overflow block must be parented to page 2's region-body"
            );
            // The first overflow block restarts at the body top (y = 0).
        }
        assert_eq!(
            tree.get(region2).expect("test: region2 exists").parent,
            Some(page2),
            "region-body must be parented to its page"
        );
        // The first overflow block of page 2 sits flush at the body top.
        let first_p2_block = blocks_of(&tree, region2)[0];
        assert_eq!(
            tree.get(first_p2_block)
                .expect("test: block exists")
                .area
                .geometry
                .y,
            Length::ZERO,
            "first block on a new page restarts at the body top"
        );
    }

    /// Static content (a header) repeats on every page of a multi-page sequence.
    ///
    /// Geometry: page 200×300pt, margins 0, region-before extent 40pt ⇒
    /// body height = 300 − 40 = 260pt. Blocks: 5 × 100pt ⇒ 2 per page
    /// (3rd would reach 300pt > 260) ⇒ ceil(5/2) = 3 pages ⇒ 3 headers.
    #[test]
    fn test_header_repeats_on_every_page() {
        let doc = build_doc(
            200.0,
            300.0,
            40.0,
            true,
            &[
                block(100.0),
                block(100.0),
                block(100.0),
                block(100.0),
                block(100.0),
            ],
        );
        let engine = LayoutEngine::new();
        let tree = engine.layout(&doc).expect("test: layout should succeed");

        let pages = page_ids(&tree);
        assert_eq!(
            pages.len(),
            3,
            "5×100pt blocks in a 260pt body must paginate to 3 pages"
        );

        // Exactly one Header area per page, each parented to a distinct page.
        let mut header_pages = Vec::new();
        for (id, node) in tree.iter() {
            if node.area.area_type == AreaType::Header {
                header_pages.push(node.parent.expect("test: header has a parent"));
                // Sanity: the header area id is real.
                let _ = id;
            }
        }
        assert_eq!(
            header_pages.len(),
            3,
            "the header static-content must repeat on all 3 pages"
        );
        header_pages.sort_by_key(|p| p.index());
        header_pages.dedup();
        assert_eq!(
            header_pages.len(),
            3,
            "each repeated header must belong to a distinct page"
        );
    }

    /// A short document still produces exactly one page (regression guard).
    ///
    /// Geometry: page 200×250pt, margins 0 ⇒ body 250pt. Two 100pt blocks total
    /// 200pt ≤ 250pt ⇒ a single page.
    #[test]
    fn test_short_document_is_single_page() {
        let doc = build_doc(200.0, 250.0, 0.0, false, &[block(100.0), block(100.0)]);
        let engine = LayoutEngine::new();
        let tree = engine.layout(&doc).expect("test: layout should succeed");

        let pages = page_ids(&tree);
        assert_eq!(pages.len(), 1, "200pt of content fits one 250pt body");

        let region_id = region_of(&tree, pages[0]);
        assert_eq!(
            blocks_of(&tree, region_id).len(),
            2,
            "both blocks live on the single page"
        );
    }

    /// A forced `break-before="page"` starts a new page even when the content
    /// would otherwise fit, and the block is reparented onto the new page.
    ///
    /// Geometry: page 200×600pt, margins 0 ⇒ body 600pt (no height overflow).
    /// Two 50pt blocks easily fit, but block 2 carries break-before=page.
    #[test]
    fn test_forced_break_before_starts_new_page() {
        let doc = build_doc(
            200.0,
            600.0,
            0.0,
            false,
            &[block(50.0), block_break_before(50.0, "page")],
        );
        let engine = LayoutEngine::new();
        let tree = engine.layout(&doc).expect("test: layout should succeed");

        let pages = page_ids(&tree);
        assert_eq!(
            pages.len(),
            2,
            "break-before=page must force a 2nd page despite the content fitting"
        );

        let region1 = region_of(&tree, pages[0]);
        let region2 = region_of(&tree, pages[1]);
        assert_eq!(
            blocks_of(&tree, region1).len(),
            1,
            "the first block stays on page 1"
        );
        let p2_blocks = blocks_of(&tree, region2);
        assert_eq!(p2_blocks.len(), 1, "the break-before block moves to page 2");
        assert_eq!(
            tree.get(p2_blocks[0]).expect("test: block exists").parent,
            Some(region2),
            "the break-before block is parented under page 2's region-body"
        );
    }

    /// Keep-with-previous drags the preceding block onto the new page so the
    /// pair is not split by a height-overflow break.
    ///
    /// Geometry: page 200×250pt, margins 0 ⇒ body 250pt. Blocks: b1=100pt,
    /// b2=100pt, b3=100pt with keep-with-previous. Naively b1,b2 fill page 1
    /// (200pt) and b3 overflows alone; but keep-with-previous glues b3 to b2, so
    /// the b2+b3 pair migrates together ⇒ page 1 = [b1], page 2 = [b2, b3].
    #[test]
    fn test_keep_with_previous_migrates_pair() {
        let mut fo = FoArena::new();
        let root = fo.add_node(FoNode::new(FoNodeData::Root));
        let lms = fo.add_node(FoNode::new(FoNodeData::LayoutMasterSet));
        fo.append_child(root, lms).expect("test: append lms");

        let mut spm_props = PropertyList::new();
        spm_props.set(
            PropertyId::PageWidth,
            PropertyValue::Length(Length::from_pt(200.0)),
        );
        spm_props.set(
            PropertyId::PageHeight,
            PropertyValue::Length(Length::from_pt(250.0)),
        );
        for margin in [
            PropertyId::MarginTop,
            PropertyId::MarginBottom,
            PropertyId::MarginLeft,
            PropertyId::MarginRight,
        ] {
            spm_props.set(margin, PropertyValue::Length(Length::ZERO));
        }
        let spm = fo.add_node(FoNode::new(FoNodeData::SimplePageMaster {
            master_name: "pm".to_string(),
            properties: spm_props,
        }));
        fo.append_child(lms, spm).expect("test: append spm");
        let body = fo.add_node(FoNode::new(FoNodeData::RegionBody {
            properties: PropertyList::new(),
        }));
        fo.append_child(spm, body)
            .expect("test: append region-body");

        let ps = fo.add_node(FoNode::new(FoNodeData::PageSequence {
            master_reference: "pm".to_string(),
            format: "1".to_string(),
            grouping_separator: None,
            grouping_size: None,
            properties: PropertyList::new(),
        }));
        fo.append_child(root, ps)
            .expect("test: append page-sequence");
        let flow = fo.add_node(FoNode::new(FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: PropertyList::new(),
        }));
        fo.append_child(ps, flow).expect("test: append flow");

        // b1, b2 (plain 100pt) and b3 (100pt, keep-with-previous=always).
        for keep in [false, false, true] {
            let mut bp = PropertyList::new();
            bp.set(
                PropertyId::LineHeight,
                PropertyValue::Length(Length::from_pt(100.0)),
            );
            if keep {
                bp.set(
                    PropertyId::KeepWithPrevious,
                    PropertyValue::String(Cow::Borrowed("always")),
                );
            }
            let b = fo.add_node(FoNode::new(FoNodeData::Block { properties: bp }));
            fo.append_child(flow, b).expect("test: append block");
        }

        let engine = LayoutEngine::new();
        let tree = engine.layout(&fo).expect("test: layout should succeed");

        let pages = page_ids(&tree);
        assert_eq!(pages.len(), 2, "the glued pair forces a 2-page layout");

        let region1 = region_of(&tree, pages[0]);
        let region2 = region_of(&tree, pages[1]);
        assert_eq!(
            blocks_of(&tree, region1).len(),
            1,
            "page 1 keeps only b1 — b2 is dragged forward by b3's keep-with-previous"
        );
        assert_eq!(
            blocks_of(&tree, region2).len(),
            2,
            "page 2 holds the glued b2+b3 pair"
        );
    }

    // -----------------------------------------------------------------------
    // Multi-column cross-page pagination
    // -----------------------------------------------------------------------

    /// The x (column) offset of a block area, in points.
    fn block_x_pt(tree: &AreaTree, block_id: AreaId) -> f64 {
        tree.get(block_id)
            .expect("test: block exists")
            .area
            .geometry
            .x
            .to_pt()
    }

    /// The y (in-column) offset of a block area, in points.
    fn block_y_pt(tree: &AreaTree, block_id: AreaId) -> f64 {
        tree.get(block_id)
            .expect("test: block exists")
            .area
            .geometry
            .y
            .to_pt()
    }

    /// A 2-column flow with more content than fits both columns of one page must
    /// paginate to ≥2 pages, placing every block exactly once.
    ///
    /// Geometry: page 200×250pt, margins 0, no header ⇒ body 200×250pt, 2 columns
    /// gap 0 ⇒ each column is 100pt wide and 250pt tall ⇒ holds two 100pt blocks
    /// (a third reaches 300 > 250).  6 blocks ⇒ page 1 fills both columns (4
    /// blocks) and 2 spill onto page 2.
    #[test]
    fn test_multicolumn_overflow_produces_multiple_pages() {
        let doc = build_doc_columns(
            200.0,
            250.0,
            0.0,
            false,
            2,
            0.0,
            &[
                block(100.0),
                block(100.0),
                block(100.0),
                block(100.0),
                block(100.0),
                block(100.0),
            ],
        );
        let engine = LayoutEngine::new();
        let tree = engine.layout(&doc).expect("test: layout should succeed");

        let pages = page_ids(&tree);
        assert_eq!(
            pages.len(),
            2,
            "6×100pt blocks in two 250pt columns must paginate to 2 pages"
        );

        let region1 = region_of(&tree, pages[0]);
        let region2 = region_of(&tree, pages[1]);
        assert_eq!(
            blocks_of(&tree, region1).len(),
            4,
            "page 1 fills both columns (2 blocks each)"
        );
        assert_eq!(
            blocks_of(&tree, region2).len(),
            2,
            "the 2 overflow blocks land on page 2"
        );

        // Every block placed exactly once across both pages.
        let total: usize = pages
            .iter()
            .map(|p| blocks_of(&tree, region_of(&tree, *p)).len())
            .sum();
        assert_eq!(total, 6, "all 6 blocks placed exactly once");
    }

    /// Columns fill left then right before the page breaks: on page 1 the first
    /// two blocks sit in the left column (x = 0) and the next two in the right
    /// column (x = 100pt), with the right column restarting at the body top.
    #[test]
    fn test_multicolumn_fills_left_then_right_before_break() {
        let doc = build_doc_columns(
            200.0,
            250.0,
            0.0,
            false,
            2,
            0.0,
            &[
                block(100.0),
                block(100.0),
                block(100.0),
                block(100.0),
                block(100.0),
                block(100.0),
            ],
        );
        let engine = LayoutEngine::new();
        let tree = engine.layout(&doc).expect("test: layout should succeed");

        let pages = page_ids(&tree);
        let region1 = region_of(&tree, pages[0]);
        let p1 = blocks_of(&tree, region1);
        assert_eq!(p1.len(), 4, "page 1 holds 4 blocks (2 per column)");

        // Left column (x = 0) fills first: blocks 0 and 1.
        assert!(
            block_x_pt(&tree, p1[0]).abs() < 0.01,
            "block 1 is in the left column"
        );
        assert!(
            block_x_pt(&tree, p1[1]).abs() < 0.01,
            "block 2 is in the left column"
        );
        assert!(
            block_y_pt(&tree, p1[0]).abs() < 0.01,
            "block 1 sits at the column top"
        );
        assert!(
            (block_y_pt(&tree, p1[1]) - 100.0).abs() < 0.01,
            "block 2 stacks below block 1 in the left column"
        );

        // Right column (x = 100pt) only after the left column is full.
        assert!(
            (block_x_pt(&tree, p1[2]) - 100.0).abs() < 0.01,
            "block 3 starts the right column"
        );
        assert!(
            (block_x_pt(&tree, p1[3]) - 100.0).abs() < 0.01,
            "block 4 is in the right column"
        );
        assert!(
            block_y_pt(&tree, p1[2]).abs() < 0.01,
            "the right column restarts at the body top"
        );
        assert!(
            (block_y_pt(&tree, p1[3]) - 100.0).abs() < 0.01,
            "block 4 stacks below block 3 in the right column"
        );
    }

    /// Static content (a header) repeats on every page of a multi-column,
    /// multi-page sequence.
    ///
    /// Geometry: page 200×300pt, region-before extent 40 ⇒ body 200×260pt, 2
    /// columns ⇒ each column holds two 100pt blocks (third reaches 300 > 260).
    /// 6 blocks ⇒ page 1 fills both columns (4 blocks), 2 spill to page 2 ⇒ 2
    /// pages ⇒ 2 repeated headers.
    #[test]
    fn test_multicolumn_header_repeats_on_every_page() {
        let doc = build_doc_columns(
            200.0,
            300.0,
            40.0,
            true,
            2,
            0.0,
            &[
                block(100.0),
                block(100.0),
                block(100.0),
                block(100.0),
                block(100.0),
                block(100.0),
            ],
        );
        let engine = LayoutEngine::new();
        let tree = engine.layout(&doc).expect("test: layout should succeed");

        let pages = page_ids(&tree);
        assert_eq!(
            pages.len(),
            2,
            "6 blocks across two 2-column pages ⇒ 2 pages"
        );

        let mut header_pages = Vec::new();
        for (_, node) in tree.iter() {
            if node.area.area_type == AreaType::Header {
                header_pages.push(node.parent.expect("test: header has a parent"));
            }
        }
        assert_eq!(
            header_pages.len(),
            2,
            "the header static-content must repeat on both pages"
        );
        header_pages.sort_by_key(|p| p.index());
        header_pages.dedup();
        assert_eq!(
            header_pages.len(),
            2,
            "each repeated header must belong to a distinct page"
        );
    }

    /// A short 2-column document stays on a single page; the second column is
    /// used once the first is full, but no new page is started.
    ///
    /// Geometry: page 200×250pt, 2 columns ⇒ each column holds two 100pt blocks.
    /// 3 blocks ⇒ left column holds blocks 1 & 2, block 3 starts the right
    /// column — still one page.
    #[test]
    fn test_multicolumn_short_document_single_page() {
        let doc = build_doc_columns(
            200.0,
            250.0,
            0.0,
            false,
            2,
            0.0,
            &[block(100.0), block(100.0), block(100.0)],
        );
        let engine = LayoutEngine::new();
        let tree = engine.layout(&doc).expect("test: layout should succeed");

        let pages = page_ids(&tree);
        assert_eq!(pages.len(), 1, "3 blocks fit within one 2-column page");

        let region = region_of(&tree, pages[0]);
        let blocks = blocks_of(&tree, region);
        assert_eq!(blocks.len(), 3, "all 3 blocks live on the single page");

        // Blocks 1 & 2 in the left column, block 3 in the right column.
        assert!(
            block_x_pt(&tree, blocks[0]).abs() < 0.01,
            "block 1 in left column"
        );
        assert!(
            block_x_pt(&tree, blocks[1]).abs() < 0.01,
            "block 2 in left column"
        );
        assert!(
            (block_x_pt(&tree, blocks[2]) - 100.0).abs() < 0.01,
            "block 3 spills into the right column"
        );
    }

    /// Regression: an explicit `column-count="1"` flow is routed through the
    /// single-column paginator and behaves exactly like the default
    /// single-column path — vertical stacking (every block at x = 0) with
    /// height-overflow pagination.
    ///
    /// Geometry: page 200×250pt, body 250pt, 4×100pt blocks ⇒ 2 blocks per page
    /// ⇒ 2 pages, all blocks in a single column at x = 0.
    #[test]
    fn test_column_count_one_uses_single_column_pagination() {
        let doc = build_doc_columns(
            200.0,
            250.0,
            0.0,
            false,
            1,
            0.0,
            &[block(100.0), block(100.0), block(100.0), block(100.0)],
        );
        let engine = LayoutEngine::new();
        let tree = engine.layout(&doc).expect("test: layout should succeed");

        let pages = page_ids(&tree);
        assert_eq!(
            pages.len(),
            2,
            "column-count=1 must paginate by height like the single-column path"
        );

        for page_id in &pages {
            let region = region_of(&tree, *page_id);
            let blocks = blocks_of(&tree, region);
            assert_eq!(blocks.len(), 2, "2 blocks per page in a single column");
            for block_id in blocks {
                assert!(
                    block_x_pt(&tree, block_id).abs() < 0.01,
                    "single-column blocks all stack at x = 0"
                );
            }
        }
    }

    // -----------------------------------------------------------------------
    // Page-accurate fo:marker / fo:retrieve-marker (running headers)
    // -----------------------------------------------------------------------

    use fop_core::tree::RetrievePosition;

    /// Build a paginated document whose region-before header retrieves the
    /// `sec` marker, and whose flow stacks one block per `block_markers` entry —
    /// each `Some(text)` block carrying an `fo:marker marker-class-name="sec"`
    /// whose content is that `text`, each `None` block carrying no marker.
    ///
    /// * page margins are 0 and `before_extent_pt` sizes the header region, so
    ///   the body height is `page_h_pt - before_extent_pt`; with `block_height_pt`
    ///   chosen against it the test controls how many marker blocks land per page.
    /// * the header's `fo:retrieve-marker` is a direct child of the
    ///   `fo:static-content` (the form the engine resolves) using
    ///   `retrieve_position` and `retrieve_boundary`.
    #[allow(clippy::too_many_arguments)]
    fn build_marker_doc(
        page_w_pt: f64,
        page_h_pt: f64,
        before_extent_pt: f64,
        retrieve_position: RetrievePosition,
        retrieve_boundary: &'static str,
        block_height_pt: f64,
        block_markers: &[Option<&str>],
    ) -> FoArena<'static> {
        let mut fo = FoArena::new();
        let root = fo.add_node(FoNode::new(FoNodeData::Root));

        let lms = fo.add_node(FoNode::new(FoNodeData::LayoutMasterSet));
        fo.append_child(root, lms).expect("test: append lms");

        let mut spm_props = PropertyList::new();
        spm_props.set(
            PropertyId::PageWidth,
            PropertyValue::Length(Length::from_pt(page_w_pt)),
        );
        spm_props.set(
            PropertyId::PageHeight,
            PropertyValue::Length(Length::from_pt(page_h_pt)),
        );
        for margin in [
            PropertyId::MarginTop,
            PropertyId::MarginBottom,
            PropertyId::MarginLeft,
            PropertyId::MarginRight,
        ] {
            spm_props.set(margin, PropertyValue::Length(Length::ZERO));
        }
        let spm = fo.add_node(FoNode::new(FoNodeData::SimplePageMaster {
            master_name: "pm".to_string(),
            properties: spm_props,
        }));
        fo.append_child(lms, spm).expect("test: append spm");

        let mut rb_props = PropertyList::new();
        rb_props.set(
            PropertyId::Extent,
            PropertyValue::Length(Length::from_pt(before_extent_pt)),
        );
        let rb = fo.add_node(FoNode::new(FoNodeData::RegionBefore {
            properties: rb_props,
        }));
        fo.append_child(spm, rb)
            .expect("test: append region-before");

        let body = fo.add_node(FoNode::new(FoNodeData::RegionBody {
            properties: PropertyList::new(),
        }));
        fo.append_child(spm, body)
            .expect("test: append region-body");

        let ps = fo.add_node(FoNode::new(FoNodeData::PageSequence {
            master_reference: "pm".to_string(),
            format: "1".to_string(),
            grouping_separator: None,
            grouping_size: None,
            properties: PropertyList::new(),
        }));
        fo.append_child(root, ps)
            .expect("test: append page-sequence");

        // Header: a static-content whose direct child is the retrieve-marker.
        let sc = fo.add_node(FoNode::new(FoNodeData::StaticContent {
            flow_name: "xsl-region-before".to_string(),
            properties: PropertyList::new(),
        }));
        fo.append_child(ps, sc)
            .expect("test: append static-content");
        let mut rm_props = PropertyList::new();
        rm_props.set(
            PropertyId::RetrieveBoundary,
            PropertyValue::String(Cow::Borrowed(retrieve_boundary)),
        );
        let rm = fo.add_node(FoNode::new(FoNodeData::RetrieveMarker {
            retrieve_class_name: "sec".to_string(),
            retrieve_position,
            properties: rm_props,
        }));
        fo.append_child(sc, rm)
            .expect("test: append retrieve-marker");

        // Flow: one block per entry, optionally carrying a `sec` marker.
        let flow = fo.add_node(FoNode::new(FoNodeData::Flow {
            flow_name: "xsl-region-body".to_string(),
            properties: PropertyList::new(),
        }));
        fo.append_child(ps, flow).expect("test: append flow");

        for marker_text in block_markers {
            let mut bp = PropertyList::new();
            bp.set(
                PropertyId::LineHeight,
                PropertyValue::Length(Length::from_pt(block_height_pt)),
            );
            let block = fo.add_node(FoNode::new(FoNodeData::Block { properties: bp }));
            fo.append_child(flow, block)
                .expect("test: append flow block");

            if let Some(text) = marker_text {
                let marker = fo.add_node(FoNode::new(FoNodeData::Marker {
                    marker_class_name: "sec".to_string(),
                    properties: PropertyList::new(),
                }));
                fo.append_child(block, marker).expect("test: append marker");
                let mut mbp = PropertyList::new();
                mbp.set(
                    PropertyId::LineHeight,
                    PropertyValue::Length(Length::from_pt(12.0)),
                );
                let marker_block = fo.add_node(FoNode::new(FoNodeData::Block { properties: mbp }));
                fo.append_child(marker, marker_block)
                    .expect("test: append marker block");
                let marker_text_node = fo.add_node(FoNode::new(FoNodeData::Text(text.to_string())));
                fo.append_child(marker_block, marker_text_node)
                    .expect("test: append marker text");
            }
        }

        fo
    }

    /// Concatenated, trimmed text rendered into a page's `Header` area (the
    /// content the running header's retrieve-marker resolved to).
    fn header_text(tree: &AreaTree, page_id: AreaId) -> String {
        let header = tree.children(page_id).into_iter().find(|&child| {
            tree.get(child)
                .map(|n| n.area.area_type == AreaType::Header)
                .unwrap_or(false)
        });
        let mut out = String::new();
        if let Some(header_id) = header {
            collect_area_text(tree, header_id, &mut out);
        }
        out.trim().to_string()
    }

    /// Append all `Text` content under `id` (depth-first) to `out`.
    fn collect_area_text(tree: &AreaTree, id: AreaId, out: &mut String) {
        if let Some(node) = tree.get(id) {
            if let Some(text) = node.area.text_content() {
                out.push_str(text);
            }
            for child_id in tree.children(id) {
                collect_area_text(tree, child_id, out);
            }
        }
    }

    /// Each page's running header shows the marker that starts on *that* page —
    /// not a single marker repeated on every page (the bug this fixes).
    ///
    /// Geometry: page 200×300pt, region-before extent 40 ⇒ body 260pt; 100pt
    /// marker blocks ⇒ 2 per page.  Blocks carry Alpha, Bravo, Charlie, Delta ⇒
    /// page 1 = {Alpha, Bravo}, page 2 = {Charlie, Delta}.  With
    /// `first-starting-within-page` page 1's header is Alpha and page 2's is
    /// Charlie.
    #[test]
    fn test_marker_resolves_per_page_first_starting() {
        let doc = build_marker_doc(
            200.0,
            300.0,
            40.0,
            RetrievePosition::FirstStartingWithinPage,
            "page-sequence",
            100.0,
            &[Some("Alpha"), Some("Bravo"), Some("Charlie"), Some("Delta")],
        );
        let engine = LayoutEngine::new();
        let tree = engine.layout(&doc).expect("test: layout should succeed");

        let pages = page_ids(&tree);
        assert_eq!(
            pages.len(),
            2,
            "4×100pt marker blocks in a 260pt body ⇒ 2 pages"
        );

        let p1 = header_text(&tree, pages[0]);
        let p2 = header_text(&tree, pages[1]);
        assert!(
            p1.contains("Alpha") && !p1.contains("Charlie"),
            "page 1 header must show the marker starting on page 1 (Alpha), got {:?}",
            p1
        );
        assert!(
            p2.contains("Charlie") && !p2.contains("Alpha"),
            "page 2 header must show the marker starting on page 2 (Charlie), got {:?}",
            p2
        );
        assert_ne!(p1, p2, "the two pages must show different markers");
    }

    /// On a page with two markers of the class, `first-starting-within-page` and
    /// `last-starting-within-page` select different markers.
    ///
    /// Same geometry/flow as above: page 1 = {Alpha, Bravo}.  `first-starting`
    /// yields Alpha; `last-starting` yields Bravo.
    #[test]
    fn test_marker_first_vs_last_starting_within_page() {
        let blocks = [Some("Alpha"), Some("Bravo"), Some("Charlie"), Some("Delta")];

        let first_doc = build_marker_doc(
            200.0,
            300.0,
            40.0,
            RetrievePosition::FirstStartingWithinPage,
            "page-sequence",
            100.0,
            &blocks,
        );
        let last_doc = build_marker_doc(
            200.0,
            300.0,
            40.0,
            RetrievePosition::LastStartingWithinPage,
            "page-sequence",
            100.0,
            &blocks,
        );
        let engine = LayoutEngine::new();
        let first_tree = engine
            .layout(&first_doc)
            .expect("test: layout should succeed");
        let last_tree = engine
            .layout(&last_doc)
            .expect("test: layout should succeed");

        let first_p1 = header_text(&first_tree, page_ids(&first_tree)[0]);
        let last_p1 = header_text(&last_tree, page_ids(&last_tree)[0]);

        assert!(
            first_p1.contains("Alpha") && !first_p1.contains("Bravo"),
            "first-starting must pick the first of two same-page markers (Alpha), got {:?}",
            first_p1
        );
        assert!(
            last_p1.contains("Bravo") && !last_p1.contains("Alpha"),
            "last-starting must pick the last of two same-page markers (Bravo), got {:?}",
            last_p1
        );
        assert_ne!(
            first_p1, last_p1,
            "first-starting and last-starting must differ on a 2-marker page"
        );
    }

    /// A page that sets no marker carries over the previous page's marker under
    /// `last-ending-within-page` (the position that, like a running "current
    /// section" header, shows the page's own marker when present and otherwise
    /// the one still in effect).  `last-starting-within-page` leaves the
    /// markerless page's header empty (starting positions do not carry over).
    ///
    /// Geometry: page 200×180pt, region-before extent 40 ⇒ body 140pt; 100pt
    /// blocks ⇒ one per page.  Blocks carry Alpha, (none), Bravo ⇒ page 1 sets
    /// Alpha, page 2 sets nothing, page 3 sets Bravo.  `last-ending-within-page`
    /// ⇒ page 1 Alpha, page 2 carries over Alpha, page 3 shows its own Bravo.
    #[test]
    fn test_marker_carryover_when_page_has_no_fresh_marker() {
        let flow = [Some("Alpha"), None, Some("Bravo")];

        let carry_doc = build_marker_doc(
            200.0,
            180.0,
            40.0,
            RetrievePosition::LastEndingWithinPage,
            "page-sequence",
            100.0,
            &flow,
        );
        let starting_doc = build_marker_doc(
            200.0,
            180.0,
            40.0,
            RetrievePosition::LastStartingWithinPage,
            "page-sequence",
            100.0,
            &flow,
        );
        let engine = LayoutEngine::new();
        let carry_tree = engine
            .layout(&carry_doc)
            .expect("test: layout should succeed");
        let starting_tree = engine
            .layout(&starting_doc)
            .expect("test: layout should succeed");

        let carry_pages = page_ids(&carry_tree);
        assert_eq!(
            carry_pages.len(),
            3,
            "3×100pt blocks in a 140pt body ⇒ 3 pages"
        );

        let p1 = header_text(&carry_tree, carry_pages[0]);
        let p2 = header_text(&carry_tree, carry_pages[1]);
        let p3 = header_text(&carry_tree, carry_pages[2]);
        assert!(
            p1.contains("Alpha") && !p1.contains("Bravo"),
            "page 1 header shows its own marker (Alpha), got {:?}",
            p1
        );
        assert!(
            p2.contains("Alpha") && !p2.contains("Bravo"),
            "page 2 sets no marker and must carry over Alpha, got {:?}",
            p2
        );
        assert!(
            p3.contains("Bravo") && !p3.contains("Alpha"),
            "page 3 sets and shows its own marker (Bravo), got {:?}",
            p3
        );

        // last-starting-within-page: page 2 has no qualifying marker ⇒ empty
        // (starting positions never carry over).
        let starting_pages = page_ids(&starting_tree);
        assert!(
            header_text(&starting_tree, starting_pages[1]).is_empty(),
            "last-starting must not carry over: page 2's header is empty, got {:?}",
            header_text(&starting_tree, starting_pages[1])
        );
    }
}
