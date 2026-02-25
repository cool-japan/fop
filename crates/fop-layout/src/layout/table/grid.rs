//! Grid creation, cell placement, and table-level layout

use crate::area::{Area, AreaId, AreaTree, AreaType};
use fop_types::{Length, Point, Rect, Result, Size};

use super::types::{BorderCollapse, GridCell, TableLayout};

impl TableLayout {
    /// Create table grid from rows and cells
    pub fn create_grid(&self, rows: usize, cols: usize) -> Vec<Vec<Option<GridCell>>> {
        vec![vec![None; cols]; rows]
    }

    /// Place a cell in the grid
    pub fn place_cell(&self, grid: &mut [Vec<Option<GridCell>>], cell: GridCell) -> Result<()> {
        let rows = grid.len();
        let cols = if rows > 0 { grid[0].len() } else { 0 };

        // Check bounds
        if cell.row >= rows || cell.col >= cols {
            return Err(fop_types::FopError::Generic(
                "Cell position out of bounds".to_string(),
            ));
        }

        // Place cell and mark spanned cells
        #[allow(clippy::needless_range_loop)]
        for r in cell.row..(cell.row + cell.rowspan).min(rows) {
            for c in cell.col..(cell.col + cell.colspan).min(cols) {
                if r == cell.row && c == cell.col {
                    grid[r][c] = Some(cell.clone());
                } else {
                    // Mark as occupied by span
                    grid[r][c] = Some(GridCell {
                        row: cell.row,
                        col: cell.col,
                        rowspan: 0, // Marker: this is a spanned cell
                        colspan: 0,
                        content_id: None,
                    });
                }
            }
        }

        Ok(())
    }

    /// Layout table into area tree
    pub fn layout_table(
        &self,
        area_tree: &mut AreaTree,
        column_widths: &[Length],
        grid: &[Vec<Option<GridCell>>],
        y_offset: Length,
    ) -> Result<AreaId> {
        let rows = grid.len();
        let cols = if rows > 0 { grid[0].len() } else { 0 };

        // Calculate table dimensions based on border-collapse model
        let table_width: Length = column_widths
            .iter()
            .copied()
            .fold(Length::ZERO, |acc, len| acc + len);

        // For collapsed borders, no spacing between cells
        let total_spacing = match self.border_collapse {
            BorderCollapse::Separate => self.border_spacing * (cols + 1) as i32,
            BorderCollapse::Collapse => Length::ZERO,
        };
        let total_width = table_width + total_spacing;

        // Create table area
        let table_rect = Rect::from_point_size(
            Point::new(Length::ZERO, y_offset),
            Size::new(total_width, Length::from_pt(100.0)), // Height calculated later
        );
        let table_area = Area::new(AreaType::Block, table_rect);
        let table_id = area_tree.add_area(table_area);

        // Layout cells with appropriate spacing
        let (initial_x, initial_y, cell_spacing_x, cell_spacing_y) = match self.border_collapse {
            BorderCollapse::Separate => (
                self.border_spacing,
                self.border_spacing,
                self.border_spacing,
                self.border_spacing,
            ),
            BorderCollapse::Collapse => (Length::ZERO, Length::ZERO, Length::ZERO, Length::ZERO),
        };

        let mut current_y = initial_y;
        for row in grid {
            let mut current_x = initial_x;
            let row_height = Length::from_pt(20.0); // Default row height

            for (col_idx, cell_opt) in row.iter().enumerate() {
                if let Some(cell) = cell_opt {
                    // Only process actual cells (not span markers)
                    if cell.rowspan > 0 && cell.colspan > 0 {
                        let cell_width = column_widths[col_idx];
                        let cell_rect = Rect::from_point_size(
                            Point::new(current_x, current_y),
                            Size::new(cell_width, row_height),
                        );
                        let cell_area = Area::new(AreaType::Block, cell_rect);
                        area_tree.add_area(cell_area);
                    }
                }

                if col_idx < column_widths.len() {
                    current_x += column_widths[col_idx] + cell_spacing_x;
                }
            }

            current_y += row_height + cell_spacing_y;
        }

        Ok(table_id)
    }
}
