//! Table and footnote layout methods for the layout engine.
//!
//! Handles `fo:table`, `fo:table-header`, `fo:table-body`, `fo:table-footer`,
//! `fo:table-row`, `fo:table-cell`, and `fo:footnote` elements.

use crate::area::{Area, AreaTree, AreaType, TraitSet};
use crate::layout::{
    BorderCollapse, ColumnWidth, PageNumberResolver, TableLayout, TableLayoutMode,
};
use fop_core::{FoArena, FoNodeData, NodeId, PropertyId, PropertyList};
use fop_types::{Color, Length, Point, Rect, Result, Size};

use super::LayoutEngine;

impl LayoutEngine {
    /// Lay out an `fo:table` node: resolve the table-layout/border model, compute
    /// the column widths, create the outer table area, and place every section.
    ///
    /// This is the single entry point shared by both dispatch routes:
    /// * `layout_node` (tables under fo:root / fo:block-container), and
    /// * `layout_block` (tables that are direct children of an fo:flow),
    ///
    /// so a table renders identically wherever it appears.  `available_width` is
    /// the table's inline-progression-dimension and `y_offset` is its block-start
    /// position within `parent_area` (the flow stacks tables like blocks; the
    /// layout_node route passes `0`).  Returns the outer table area id.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn layout_table_node(
        &self,
        fo_tree: &FoArena,
        node_id: NodeId,
        properties: &PropertyList,
        area_tree: &mut AreaTree,
        parent_area: Option<crate::area::AreaId>,
        available_width: Length,
        y_offset: Length,
        resolver: &mut PageNumberResolver,
    ) -> Result<Option<crate::area::AreaId>> {
        // Resolve table-layout (EN_AUTO = 9, EN_FIXED = 51); default fixed.
        let layout_mode = if let Ok(prop) = properties.get(PropertyId::TableLayout) {
            if let Some(enum_val) = prop.as_enum() {
                if enum_val == 9 {
                    TableLayoutMode::Auto
                } else {
                    TableLayoutMode::Fixed
                }
            } else if prop.is_auto() {
                TableLayoutMode::Auto
            } else {
                TableLayoutMode::Fixed
            }
        } else {
            TableLayoutMode::Fixed
        };

        // Resolve border-collapse (EN_COLLAPSE = 28); default separate.
        let border_collapse = if let Ok(prop) = properties.get(PropertyId::BorderCollapse) {
            if let Some(enum_val) = prop.as_enum() {
                if enum_val == 28 {
                    BorderCollapse::Collapse
                } else {
                    BorderCollapse::Separate
                }
            } else if let Some(string_val) = prop.as_string() {
                if string_val == "collapse" {
                    BorderCollapse::Collapse
                } else {
                    BorderCollapse::Separate
                }
            } else {
                BorderCollapse::Separate
            }
        } else {
            BorderCollapse::Separate
        };

        // border-spacing only affects the separate model; default 0pt per spec.
        let border_spacing = if let Ok(prop) = properties.get(PropertyId::BorderSpacing) {
            prop.as_length().unwrap_or(Length::from_pt(0.0))
        } else {
            Length::from_pt(0.0)
        };

        let table_layout = TableLayout::new(available_width)
            .with_border_spacing(border_spacing)
            .with_layout_mode(layout_mode)
            .with_border_collapse(border_collapse);

        // Gather the declared column specs (in document order) and the sections.
        let mut column_specs = Vec::new();
        let mut header_id = None;
        let mut footer_id = None;
        let mut body_ids = Vec::new();

        for child_id in fo_tree.children(node_id) {
            if let Some(child) = fo_tree.get(child_id) {
                match &child.data {
                    FoNodeData::TableColumn { .. } => {
                        if let Some(props) = child.data.properties() {
                            if let Ok(width) = props.get(PropertyId::ColumnWidth) {
                                if let Some(len) = width.as_length() {
                                    column_specs.push(ColumnWidth::Fixed(len));
                                } else if width.is_auto() {
                                    column_specs.push(ColumnWidth::Auto);
                                }
                            } else {
                                column_specs.push(ColumnWidth::Auto);
                            }
                        }
                    }
                    FoNodeData::TableHeader { .. } => header_id = Some(child_id),
                    FoNodeData::TableFooter { .. } => footer_id = Some(child_id),
                    FoNodeData::TableBody { .. } => body_ids.push(child_id),
                    _ => {}
                }
            }
        }

        // Compute the final column widths per the resolved layout mode.
        let computed_widths = match layout_mode {
            TableLayoutMode::Fixed => {
                // Fixed layout is unchanged: undeclared tables collapse to a single
                // proportional column that fills the available width.
                let specs = if column_specs.is_empty() {
                    vec![ColumnWidth::Proportional(1.0)]
                } else {
                    column_specs.clone()
                };
                table_layout.compute_fixed_widths(&specs)
            }
            TableLayoutMode::Auto => {
                // Auto layout measures real cell content (GAP 1).  Undeclared
                // columns are treated as auto and sized from the cells themselves.
                self.measure_auto_column_widths(fo_tree, node_id, &column_specs, &table_layout)
            }
        };

        // Create the outer table area (zero-height placeholder; the real height is
        // computed by layout_table and written back below).
        let table_rect = Rect::new(Length::ZERO, y_offset, available_width, Length::ZERO);

        let mut traits = TraitSet::default();
        if let Ok(color) = properties.get(PropertyId::BackgroundColor) {
            traits.background_color = color.as_color();
        }

        let area = Area::new(AreaType::Block, table_rect).with_traits(traits);
        let table_id = area_tree.add_area(area);

        if let Some(parent) = parent_area {
            area_tree
                .append_child(parent, table_id)
                .map_err(fop_types::FopError::Generic)?;
        }

        // layout_table returns the real total height (sum of all section/row
        // heights); write it back so stacking / float-clear / pagination see the
        // correct bounding box.
        let real_table_height = self.layout_table(
            fo_tree,
            area_tree,
            table_id,
            header_id,
            footer_id,
            &body_ids,
            &computed_widths,
            resolver,
        )?;

        if let Some(table_area_node) = area_tree.get_mut(table_id) {
            table_area_node.area.geometry.height = real_table_height;
        }

        Ok(Some(table_id))
    }

    /// Layout a complete table with header, body, and footer sections.
    ///
    /// Returns the total height of the table (sum of all section heights), which
    /// the caller must use to update the outer table area's `geometry.height`.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn layout_table(
        &self,
        fo_tree: &FoArena,
        area_tree: &mut AreaTree,
        table_id: crate::area::AreaId,
        header_id: Option<NodeId>,
        footer_id: Option<NodeId>,
        body_ids: &[NodeId],
        column_widths: &[Length],
        resolver: &mut PageNumberResolver,
    ) -> Result<Length> {
        let mut current_y = Length::ZERO;

        // Layout header once and cache the areas
        let header_areas = if let Some(hid) = header_id {
            // layout_table_section returns the absolute y-position after all header rows,
            // starting from current_y (= 0).  That value equals the header's total height
            // when current_y starts at zero.
            let header_end_y = self.layout_table_section(
                fo_tree,
                hid,
                area_tree,
                table_id,
                current_y,
                column_widths,
                resolver,
            )?;
            current_y = header_end_y;

            // Collect header area IDs for potential repetition
            let children = area_tree.children(table_id);
            children.into_iter().collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        // Layout all body sections; each call advances current_y to the end of that section.
        for body_id in body_ids {
            current_y = self.layout_table_section(
                fo_tree,
                *body_id,
                area_tree,
                table_id,
                current_y,
                column_widths,
                resolver,
            )?;
        }

        // Layout footer and include its rows in the total height.
        if let Some(fid) = footer_id {
            current_y = self.layout_table_section(
                fo_tree,
                fid,
                area_tree,
                table_id,
                current_y,
                column_widths,
                resolver,
            )?;
        }

        // Note: In a full implementation with multi-page tables, we would:
        // 1. Detect when body content exceeds page height
        // 2. Create a new page
        // 3. Clone and place header_areas at the top of the new page
        // 4. Continue laying out body content
        // 5. Clone and place footer areas at the bottom of each page

        // Store header areas for page breaking (future enhancement)
        let _ = header_areas; // Suppress unused warning

        // current_y is now the absolute y-coordinate of the bottom edge of the last
        // row in the table (measured from the table's own origin at y = 0), which is
        // exactly the real height of the table.
        Ok(current_y)
    }

    /// Layout a table section (header, body, or footer)
    #[allow(clippy::too_many_arguments)]
    pub(super) fn layout_table_section(
        &self,
        fo_tree: &FoArena,
        section_id: NodeId,
        area_tree: &mut AreaTree,
        table_id: crate::area::AreaId,
        y_offset: Length,
        column_widths: &[Length],
        resolver: &mut PageNumberResolver,
    ) -> Result<Length> {
        self.layout_table_body(
            fo_tree,
            section_id,
            area_tree,
            table_id,
            y_offset,
            column_widths,
            resolver,
        )
    }

    /// Layout a table body
    #[allow(clippy::too_many_arguments)]
    pub(super) fn layout_table_body(
        &self,
        fo_tree: &FoArena,
        tbody_id: NodeId,
        area_tree: &mut AreaTree,
        table_id: crate::area::AreaId,
        y_offset: Length,
        column_widths: &[Length],
        resolver: &mut PageNumberResolver,
    ) -> Result<Length> {
        let mut current_y = y_offset;
        let row_height = Length::from_pt(30.0); // Default row height

        // Build a grid to track cell spanning
        let rows = fo_tree.children(tbody_id);
        let row_count = rows.len();
        let col_count = column_widths.len();

        // Grid to track which cells are occupied (None = free, Some(original_cell_id) = occupied)
        let mut grid: Vec<Vec<Option<NodeId>>> = vec![vec![None; col_count]; row_count];

        // First pass: Build the grid with span information
        for (row_idx, row_id) in rows.iter().enumerate() {
            if let Some(row_node) = fo_tree.get(*row_id) {
                if matches!(row_node.data, FoNodeData::TableRow { .. }) {
                    let cells = fo_tree.children(*row_id);
                    let mut col_idx = 0;

                    for cell_id in cells {
                        // Find the next free column
                        while col_idx < col_count && grid[row_idx][col_idx].is_some() {
                            col_idx += 1;
                        }

                        if col_idx >= col_count {
                            break;
                        }

                        if let Some(cell_node) = fo_tree.get(cell_id) {
                            if matches!(cell_node.data, FoNodeData::TableCell { .. }) {
                                // Extract colspan and rowspan from properties
                                let (colspan, rowspan) =
                                    if let Some(props) = cell_node.data.properties() {
                                        let cols = props
                                            .get(PropertyId::NumberColumnsSpanned)
                                            .ok()
                                            .and_then(|v| v.as_number())
                                            .map(|n| n.max(1.0) as usize)
                                            .unwrap_or(1);
                                        let rows_span = props
                                            .get(PropertyId::NumberRowsSpanned)
                                            .ok()
                                            .and_then(|v| v.as_number())
                                            .map(|n| n.max(1.0) as usize)
                                            .unwrap_or(1);
                                        (cols, rows_span)
                                    } else {
                                        (1, 1)
                                    };

                                // Mark the grid cells as occupied
                                #[allow(clippy::needless_range_loop)]
                                for r in row_idx..(row_idx + rowspan).min(row_count) {
                                    for c in col_idx..(col_idx + colspan).min(col_count) {
                                        grid[r][c] = Some(cell_id);
                                    }
                                }

                                col_idx += colspan;
                            }
                        }
                    }
                }
            }
        }

        // Second pass: Layout the cells using the grid
        for (row_idx, row_id) in rows.iter().enumerate() {
            if let Some(row_node) = fo_tree.get(*row_id) {
                if matches!(row_node.data, FoNodeData::TableRow { .. }) {
                    // Create row area
                    let row_width: Length =
                        column_widths.iter().fold(Length::ZERO, |acc, w| acc + *w);
                    let row_rect = Rect::new(Length::ZERO, current_y, row_width, row_height);
                    let row_area = Area::new(AreaType::Block, row_rect);
                    let row_area_id = area_tree.add_area(row_area);

                    area_tree
                        .append_child(table_id, row_area_id)
                        .map_err(fop_types::FopError::Generic)?;

                    // Layout cells for this row
                    let cells = fo_tree.children(*row_id);

                    for cell_id in cells {
                        if let Some(cell_node) = fo_tree.get(cell_id) {
                            if matches!(cell_node.data, FoNodeData::TableCell { .. }) {
                                // Find this cell's position in the grid
                                let mut cell_col_idx = 0;
                                let mut found = false;
                                for c in 0..col_count {
                                    if grid[row_idx][c] == Some(cell_id) {
                                        // Check if this is the origin cell (not a spanned cell)
                                        let is_origin = if row_idx > 0 {
                                            grid[row_idx - 1][c] != Some(cell_id)
                                        } else {
                                            true
                                        } && if c > 0 {
                                            grid[row_idx][c - 1] != Some(cell_id)
                                        } else {
                                            true
                                        };

                                        if is_origin {
                                            cell_col_idx = c;
                                            found = true;
                                            break;
                                        }
                                    }
                                }

                                if !found {
                                    continue; // This cell was already laid out in a previous row (rowspan)
                                }

                                // Extract colspan and rowspan
                                let (colspan, rowspan) =
                                    if let Some(props) = cell_node.data.properties() {
                                        let cols = props
                                            .get(PropertyId::NumberColumnsSpanned)
                                            .ok()
                                            .and_then(|v| v.as_number())
                                            .map(|n| n.max(1.0) as usize)
                                            .unwrap_or(1);
                                        let rows_span = props
                                            .get(PropertyId::NumberRowsSpanned)
                                            .ok()
                                            .and_then(|v| v.as_number())
                                            .map(|n| n.max(1.0) as usize)
                                            .unwrap_or(1);
                                        (cols, rows_span)
                                    } else {
                                        (1, 1)
                                    };

                                // Calculate cell position and dimensions
                                let cell_x: Length = column_widths[..cell_col_idx]
                                    .iter()
                                    .fold(Length::ZERO, |acc, w| acc + *w);

                                let cell_width: Length = column_widths
                                    [cell_col_idx..(cell_col_idx + colspan).min(col_count)]
                                    .iter()
                                    .fold(Length::ZERO, |acc, w| acc + *w);

                                let cell_height = row_height * rowspan as i32;

                                // Create cell area
                                let cell_rect =
                                    Rect::new(cell_x, Length::ZERO, cell_width, cell_height);
                                let mut traits = TraitSet::default();

                                if let Some(props) = cell_node.data.properties() {
                                    if let Ok(color) = props.get(PropertyId::BackgroundColor) {
                                        traits.background_color = color.as_color();
                                    }
                                }

                                let cell_area =
                                    Area::new(AreaType::Block, cell_rect).with_traits(traits);
                                let cell_area_id = area_tree.add_area(cell_area);

                                area_tree
                                    .append_child(row_area_id, cell_area_id)
                                    .map_err(fop_types::FopError::Generic)?;

                                // Layout cell content (blocks)
                                let cell_children = fo_tree.children(cell_id);
                                for child_id in cell_children {
                                    self.layout_block(
                                        fo_tree,
                                        child_id,
                                        area_tree,
                                        cell_area_id,
                                        Length::ZERO,
                                        cell_width,
                                        resolver,
                                    )?;
                                }
                            }
                        }
                    }

                    current_y += row_height;
                }
            }
        }

        Ok(current_y)
    }

    /// Place collected footnotes at the bottom of the page body region.
    ///
    /// After all flow content has been laid out, this method retrieves all footnote
    /// bodies registered for the page, adds a separator line, and positions each
    /// footnote at the bottom of the body rect.
    pub(super) fn place_footnotes_for_page(
        &self,
        area_tree: &mut AreaTree,
        page_id: crate::area::AreaId,
        body_rect: Rect,
    ) -> Result<()> {
        use crate::area::{AreaId, BorderStyle};

        // Collect footnote IDs (clone to avoid borrow issues)
        let footnotes: Vec<AreaId> = match area_tree.get_footnotes(page_id) {
            Some(f) if !f.is_empty() => f.clone(),
            _ => return Ok(()), // No footnotes on this page
        };

        // Compute total footnote height (all footnotes + separator)
        let footnote_total_height = area_tree.footnote_height(page_id);

        // Separator Y: bottom of body rect minus total footnote block height
        let separator_y = body_rect.y + body_rect.height - footnote_total_height;

        // Create a thin horizontal footnote separator line (1pt thick, 1/3 page width)
        let separator_width = body_rect.width / 3;
        let separator_rect = Rect::from_point_size(
            Point::new(body_rect.x, separator_y),
            Size::new(separator_width, Length::from_pt(1.0)),
        );
        let mut separator_area = Area::new(AreaType::FootnoteSeparator, separator_rect);
        separator_area.traits = crate::area::TraitSet {
            border_width: Some([
                Length::from_pt(1.0),
                Length::ZERO,
                Length::ZERO,
                Length::ZERO,
            ]),
            border_color: Some([Color::BLACK, Color::BLACK, Color::BLACK, Color::BLACK]),
            border_style: Some([
                BorderStyle::Solid,
                BorderStyle::None,
                BorderStyle::None,
                BorderStyle::None,
            ]),
            ..Default::default()
        };
        let separator_id = area_tree.add_area(separator_area);
        area_tree
            .append_child(page_id, separator_id)
            .map_err(fop_types::FopError::Generic)?;

        // Place each footnote below the separator
        // separator_y + 1pt line + 6pt gap = separator_y + 7pt
        let mut current_y = separator_y + Length::from_pt(7.0);

        for footnote_id in footnotes {
            let footnote_height = if let Some(fn_node) = area_tree.get(footnote_id) {
                fn_node.area.height()
            } else {
                continue;
            };

            // Reposition the footnote to its final location
            if let Some(fn_node) = area_tree.get_mut(footnote_id) {
                fn_node.area.geometry.x = body_rect.x;
                fn_node.area.geometry.y = current_y;
                fn_node.area.geometry.width = body_rect.width;
            }

            // Attach footnote to the page as a top-level sibling of regions
            area_tree
                .append_child(page_id, footnote_id)
                .map_err(fop_types::FopError::Generic)?;

            current_y += footnote_height;
        }

        Ok(())
    }

    /// Layout a fo:footnote element.
    ///
    /// Renders the fo:inline reference mark inline within the current block, and
    /// collects the fo:footnote-body content as a Footnote area at the page bottom.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn layout_footnote(
        &self,
        fo_tree: &FoArena,
        footnote_node_id: NodeId,
        area_tree: &mut AreaTree,
        block_area_id: crate::area::AreaId,
        block_parent_area_id: crate::area::AreaId,
        available_width: Length,
        line_height: Length,
        parent_traits: &TraitSet,
        first_line_indent: Length,
        y_offset: Length,
        resolver: &mut PageNumberResolver,
    ) -> Result<()> {
        let children = fo_tree.children(footnote_node_id);

        let mut inline_child: Option<NodeId> = None;
        let mut body_child: Option<NodeId> = None;

        // Separate the fo:inline reference mark from the fo:footnote-body
        for child_id in children {
            if let Some(child_node) = fo_tree.get(child_id) {
                match &child_node.data {
                    FoNodeData::Inline { .. } => {
                        inline_child = Some(child_id);
                    }
                    FoNodeData::FootnoteBody { .. } => {
                        body_child = Some(child_id);
                    }
                    _ => {}
                }
            }
        }

        // 1. Render the reference mark (fo:inline) inline within the current block
        if let Some(inline_id) = inline_child {
            self.layout_inline(
                fo_tree,
                inline_id,
                area_tree,
                block_area_id,
                available_width,
                line_height,
                parent_traits,
                first_line_indent,
                y_offset,
            )?;
        }

        // 2. Layout the footnote body as a Footnote area and register it with the page
        if let Some(body_id) = body_child {
            let footnote_width = available_width;
            let footnote_line_height = Length::from_pt(10.0); // Default 10pt for footnote text

            // Create a container area for the footnote body
            let footnote_rect = Rect::from_point_size(
                Point::ZERO, // Position will be set by place_footnotes later
                Size::new(footnote_width, footnote_line_height),
            );
            let footnote_area = Area::new(AreaType::Footnote, footnote_rect);
            let footnote_area_id = area_tree.add_area(footnote_area);

            // Layout children of fo:footnote-body (usually fo:block elements)
            let body_children = fo_tree.children(body_id);
            let mut current_y = Length::ZERO;

            for body_child_id in body_children {
                if let Some(child_area_id) = self.layout_block(
                    fo_tree,
                    body_child_id,
                    area_tree,
                    footnote_area_id,
                    current_y,
                    footnote_width,
                    resolver,
                )? {
                    if let Some(child_area) = area_tree.get(child_area_id) {
                        current_y = child_area.area.geometry.y + child_area.area.height();
                    }
                }
            }

            // Update footnote area height to match content
            if current_y > Length::ZERO {
                if let Some(fn_node) = area_tree.get_mut(footnote_area_id) {
                    fn_node.area.geometry.height = current_y;
                }
            }

            // Find the page ancestor and register the footnote
            // Walk up: block_area_id -> parent (region) -> page
            let page_id = area_tree
                .find_page_ancestor(block_parent_area_id)
                .or_else(|| area_tree.find_page_ancestor(block_area_id));

            if let Some(pid) = page_id {
                area_tree.add_footnote(pid, footnote_area_id);
            }
        }

        Ok(())
    }
}
