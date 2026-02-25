//! Table and footnote layout methods for the layout engine.
//!
//! Handles `fo:table`, `fo:table-header`, `fo:table-body`, `fo:table-footer`,
//! `fo:table-row`, `fo:table-cell`, and `fo:footnote` elements.

use crate::area::{Area, AreaTree, AreaType, TraitSet};
use crate::layout::PageNumberResolver;
use fop_core::{FoArena, FoNodeData, NodeId, PropertyId};
use fop_types::{Color, Length, Point, Rect, Result, Size};

use super::LayoutEngine;

impl LayoutEngine {
    /// Layout a complete table with header, body, and footer sections
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
    ) -> Result<()> {
        let mut current_y = Length::ZERO;

        // Layout header once and cache the areas
        let header_areas = if let Some(hid) = header_id {
            let header_height = self.layout_table_section(
                fo_tree,
                hid,
                area_tree,
                table_id,
                current_y,
                column_widths,
                resolver,
            )?;
            current_y += header_height;

            // Collect header area IDs for potential repetition
            let children = area_tree.children(table_id);
            children.into_iter().collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        // Layout all body sections
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

        // Layout footer once and cache the areas
        if let Some(fid) = footer_id {
            let _footer_height = self.layout_table_section(
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

        Ok(())
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
