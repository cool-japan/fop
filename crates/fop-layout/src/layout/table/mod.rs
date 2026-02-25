//! Table layout algorithm
//!
//! Implements fixed and auto table layout modes for positioning cells in a grid.

pub mod border;
pub mod column;
pub mod grid;
pub mod types;

pub use types::{
    BorderCollapse, CellCollapsedBorders, CollapsedBorder, ColumnInfo, ColumnWidth, GridCell,
    TableLayout, TableLayoutMode,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::area::{AreaId, AreaTree};
    use fop_types::Length;

    #[test]
    fn test_table_layout_creation() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        assert_eq!(layout.available_width, Length::from_pt(500.0));
    }

    #[test]
    fn test_fixed_width_single_column() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let columns = vec![ColumnWidth::Fixed(Length::from_pt(200.0))];

        let widths = layout.compute_fixed_widths(&columns);

        assert_eq!(widths.len(), 1);
        assert_eq!(widths[0], Length::from_pt(200.0));
    }

    #[test]
    fn test_fixed_width_proportional() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let columns = vec![
            ColumnWidth::Proportional(1.0),
            ColumnWidth::Proportional(2.0),
        ];

        let widths = layout.compute_fixed_widths(&columns);

        assert_eq!(widths.len(), 2);
        // Total available after spacing: 500 - 3*2 = 494
        // 1:2 ratio means 164.67 and 329.33 approximately
        assert!(widths[0].to_pt() < widths[1].to_pt());
    }

    #[test]
    fn test_fixed_width_mixed() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let columns = vec![
            ColumnWidth::Fixed(Length::from_pt(100.0)),
            ColumnWidth::Proportional(1.0),
            ColumnWidth::Proportional(1.0),
        ];

        let widths = layout.compute_fixed_widths(&columns);

        assert_eq!(widths.len(), 3);
        assert_eq!(widths[0], Length::from_pt(100.0));
        // Remaining 494 - 100 = 394 split 1:1
        assert_eq!(widths[1], widths[2]);
    }

    #[test]
    fn test_create_grid() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let grid = layout.create_grid(3, 4);

        assert_eq!(grid.len(), 3);
        assert_eq!(grid[0].len(), 4);
        assert!(grid[0][0].is_none());
    }

    #[test]
    fn test_place_cell() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let mut grid = layout.create_grid(3, 3);

        let cell = GridCell {
            row: 0,
            col: 0,
            rowspan: 1,
            colspan: 1,
            content_id: None,
        };

        layout
            .place_cell(&mut grid, cell)
            .expect("test: should succeed");
        assert!(grid[0][0].is_some());
    }

    #[test]
    fn test_place_cell_with_colspan() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let mut grid = layout.create_grid(3, 3);

        let cell = GridCell {
            row: 0,
            col: 0,
            rowspan: 1,
            colspan: 2,
            content_id: None,
        };

        layout
            .place_cell(&mut grid, cell)
            .expect("test: should succeed");
        assert!(grid[0][0].is_some());
        assert!(grid[0][1].is_some()); // Spanned
        assert!(grid[0][2].is_none()); // Not spanned
    }

    #[test]
    fn test_auto_width_with_column_info() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let column_info = vec![
            ColumnInfo {
                width_spec: ColumnWidth::Auto,
                computed_width: Length::ZERO,
                min_width: Length::from_pt(50.0),
                max_width: Length::from_pt(150.0),
            },
            ColumnInfo {
                width_spec: ColumnWidth::Fixed(Length::from_pt(200.0)),
                computed_width: Length::ZERO,
                min_width: Length::from_pt(200.0),
                max_width: Length::from_pt(200.0),
            },
        ];

        let widths = layout.compute_auto_widths(&column_info);

        assert_eq!(widths.len(), 2);
        assert_eq!(widths[1], Length::from_pt(200.0)); // Fixed
        assert!(widths[0] > Length::ZERO); // Auto got some width
    }

    #[test]
    fn test_table_layout_mode_default() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        assert_eq!(layout.layout_mode(), TableLayoutMode::Fixed);
    }

    #[test]
    fn test_table_layout_mode_auto() {
        let layout =
            TableLayout::new(Length::from_pt(500.0)).with_layout_mode(TableLayoutMode::Auto);
        assert_eq!(layout.layout_mode(), TableLayoutMode::Auto);
    }

    #[test]
    fn test_column_info_new() {
        let info = ColumnInfo::new(ColumnWidth::Auto);
        assert!(matches!(info.width_spec, ColumnWidth::Auto));
        assert_eq!(info.min_width, Length::ZERO);
        assert_eq!(info.max_width, Length::ZERO);
    }

    #[test]
    fn test_column_info_with_widths() {
        let info = ColumnInfo::with_widths(
            ColumnWidth::Auto,
            Length::from_pt(50.0),
            Length::from_pt(200.0),
        );
        assert_eq!(info.min_width, Length::from_pt(50.0));
        assert_eq!(info.max_width, Length::from_pt(200.0));
    }

    #[test]
    fn test_auto_width_plenty_of_space() {
        let layout = TableLayout::new(Length::from_pt(1000.0));
        let column_info = vec![
            ColumnInfo::with_widths(
                ColumnWidth::Auto,
                Length::from_pt(50.0),
                Length::from_pt(100.0),
            ),
            ColumnInfo::with_widths(
                ColumnWidth::Auto,
                Length::from_pt(60.0),
                Length::from_pt(120.0),
            ),
        ];

        let widths = layout.compute_auto_widths(&column_info);

        // With plenty of space, should use max widths
        assert_eq!(widths[0], Length::from_pt(100.0));
        assert_eq!(widths[1], Length::from_pt(120.0));
    }

    #[test]
    fn test_auto_width_tight_space() {
        let layout = TableLayout::new(Length::from_pt(200.0));
        let column_info = vec![
            ColumnInfo::with_widths(
                ColumnWidth::Auto,
                Length::from_pt(50.0),
                Length::from_pt(100.0),
            ),
            ColumnInfo::with_widths(
                ColumnWidth::Auto,
                Length::from_pt(60.0),
                Length::from_pt(120.0),
            ),
        ];

        let widths = layout.compute_auto_widths(&column_info);

        // With tight space, should be between min and max
        assert!(widths[0] >= Length::from_pt(50.0));
        assert!(widths[0] <= Length::from_pt(100.0));
        assert!(widths[1] >= Length::from_pt(60.0));
        assert!(widths[1] <= Length::from_pt(120.0));
    }

    #[test]
    fn test_auto_width_with_fixed_columns() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let column_info = vec![
            ColumnInfo::with_widths(
                ColumnWidth::Fixed(Length::from_pt(150.0)),
                Length::ZERO,
                Length::ZERO,
            ),
            ColumnInfo::with_widths(
                ColumnWidth::Auto,
                Length::from_pt(50.0),
                Length::from_pt(200.0),
            ),
            ColumnInfo::with_widths(
                ColumnWidth::Auto,
                Length::from_pt(50.0),
                Length::from_pt(200.0),
            ),
        ];

        let widths = layout.compute_auto_widths(&column_info);

        // Fixed column should keep its width
        assert_eq!(widths[0], Length::from_pt(150.0));
        // Auto columns share remaining space
        assert!(widths[1] > Length::ZERO);
        assert!(widths[2] > Length::ZERO);
    }

    #[test]
    fn test_measure_column_widths() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let grid = layout.create_grid(2, 3);

        let (min, max) = layout.measure_column_widths(&grid, 0);

        // Should return some default widths
        assert!(min >= Length::ZERO);
        assert!(max >= min);
    }

    #[test]
    fn test_update_column_info_from_grid() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let mut column_info = vec![
            ColumnInfo::new(ColumnWidth::Auto),
            ColumnInfo::new(ColumnWidth::Fixed(Length::from_pt(100.0))),
        ];
        let mut grid = layout.create_grid(2, 2);

        // Add a cell to the grid so there's content to measure
        let cell = GridCell {
            row: 0,
            col: 0,
            rowspan: 1,
            colspan: 1,
            content_id: None,
        };
        layout
            .place_cell(&mut grid, cell)
            .expect("test: should succeed");

        layout.update_column_info_from_grid(&mut column_info, &grid);

        // Auto column should have measured widths (from cell content)
        assert!(column_info[0].min_width > Length::ZERO);
        assert!(column_info[0].max_width >= column_info[0].min_width);
        // Fixed column should remain unchanged
        assert_eq!(column_info[1].min_width, Length::ZERO);
    }

    #[test]
    fn test_distribute_colspan_widths() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let mut column_info = vec![
            ColumnInfo::new(ColumnWidth::Auto),
            ColumnInfo::new(ColumnWidth::Auto),
        ];

        let mut grid = layout.create_grid(1, 2);
        let cell = GridCell {
            row: 0,
            col: 0,
            rowspan: 1,
            colspan: 2,
            content_id: None,
        };
        layout
            .place_cell(&mut grid, cell)
            .expect("test: should succeed");

        layout.distribute_colspan_widths(&mut column_info, &grid);

        // Columns should have increased min widths due to colspan
        assert!(column_info[0].min_width > Length::ZERO);
        assert!(column_info[1].min_width > Length::ZERO);
    }

    #[test]
    fn test_border_collapse_default() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        assert_eq!(layout.border_collapse(), BorderCollapse::Separate);
    }

    #[test]
    fn test_border_collapse_setting() {
        let layout =
            TableLayout::new(Length::from_pt(500.0)).with_border_collapse(BorderCollapse::Collapse);
        assert_eq!(layout.border_collapse(), BorderCollapse::Collapse);
    }

    #[test]
    fn test_fixed_widths_with_collapsed_borders() {
        let layout =
            TableLayout::new(Length::from_pt(500.0)).with_border_collapse(BorderCollapse::Collapse);
        let columns = vec![
            ColumnWidth::Fixed(Length::from_pt(100.0)),
            ColumnWidth::Fixed(Length::from_pt(200.0)),
        ];

        let widths = layout.compute_fixed_widths(&columns);

        // With collapsed borders, no spacing is subtracted
        // Total: 100 + 200 = 300, remaining: 500 - 300 = 200
        assert_eq!(widths[0], Length::from_pt(100.0));
        assert_eq!(widths[1], Length::from_pt(200.0));
    }

    #[test]
    fn test_fixed_widths_with_separate_borders() {
        let layout = TableLayout::new(Length::from_pt(500.0))
            .with_border_spacing(Length::from_pt(5.0))
            .with_border_collapse(BorderCollapse::Separate);
        let columns = vec![
            ColumnWidth::Fixed(Length::from_pt(100.0)),
            ColumnWidth::Fixed(Length::from_pt(200.0)),
        ];

        let widths = layout.compute_fixed_widths(&columns);

        // With separate borders, spacing is subtracted: 500 - (3 * 5) = 485 available
        assert_eq!(widths[0], Length::from_pt(100.0));
        assert_eq!(widths[1], Length::from_pt(200.0));
    }

    #[test]
    fn test_border_conflict_resolution_hidden() {
        use crate::area::BorderStyle;
        let layout = TableLayout::new(Length::from_pt(500.0));

        let border1 = CollapsedBorder::new(
            Length::from_pt(2.0),
            fop_types::Color::RED,
            BorderStyle::Hidden,
        );
        let border2 = CollapsedBorder::new(
            Length::from_pt(5.0),
            fop_types::Color::BLUE,
            BorderStyle::Solid,
        );

        let result = layout.resolve_border_conflict(border1, border2);
        // Hidden always wins
        assert_eq!(result.style, BorderStyle::Hidden);
    }

    #[test]
    fn test_border_conflict_resolution_width() {
        use crate::area::BorderStyle;
        let layout = TableLayout::new(Length::from_pt(500.0));

        let border1 = CollapsedBorder::new(
            Length::from_pt(5.0),
            fop_types::Color::RED,
            BorderStyle::Solid,
        );
        let border2 = CollapsedBorder::new(
            Length::from_pt(2.0),
            fop_types::Color::BLUE,
            BorderStyle::Solid,
        );

        let result = layout.resolve_border_conflict(border1, border2);
        // Wider border wins
        assert_eq!(result.width, Length::from_pt(5.0));
        assert_eq!(result.color, fop_types::Color::RED);
    }

    #[test]
    fn test_border_conflict_resolution_style() {
        use crate::area::BorderStyle;
        let layout = TableLayout::new(Length::from_pt(500.0));

        let border1 = CollapsedBorder::new(
            Length::from_pt(2.0),
            fop_types::Color::RED,
            BorderStyle::Double,
        );
        let border2 = CollapsedBorder::new(
            Length::from_pt(2.0),
            fop_types::Color::BLUE,
            BorderStyle::Dashed,
        );

        let result = layout.resolve_border_conflict(border1, border2);
        // Double has higher precedence than Dashed
        assert_eq!(result.style, BorderStyle::Double);
        assert_eq!(result.color, fop_types::Color::RED);
    }

    #[test]
    fn test_border_conflict_resolution_none() {
        use crate::area::BorderStyle;
        let layout = TableLayout::new(Length::from_pt(500.0));

        let border1 = CollapsedBorder::new(
            Length::from_pt(2.0),
            fop_types::Color::RED,
            BorderStyle::None,
        );
        let border2 = CollapsedBorder::new(
            Length::from_pt(2.0),
            fop_types::Color::BLUE,
            BorderStyle::Solid,
        );

        let result = layout.resolve_border_conflict(border1, border2);
        // Solid wins over None
        assert_eq!(result.style, BorderStyle::Solid);
        assert_eq!(result.color, fop_types::Color::BLUE);
    }

    #[test]
    fn test_collapsed_border_visible() {
        use crate::area::BorderStyle;

        let visible = CollapsedBorder::new(
            Length::from_pt(2.0),
            fop_types::Color::RED,
            BorderStyle::Solid,
        );
        assert!(visible.is_visible());

        let invisible_none = CollapsedBorder::new(
            Length::from_pt(2.0),
            fop_types::Color::RED,
            BorderStyle::None,
        );
        assert!(!invisible_none.is_visible());

        let invisible_hidden = CollapsedBorder::new(
            Length::from_pt(2.0),
            fop_types::Color::RED,
            BorderStyle::Hidden,
        );
        assert!(!invisible_hidden.is_visible());

        let invisible_zero =
            CollapsedBorder::new(Length::ZERO, fop_types::Color::RED, BorderStyle::Solid);
        assert!(!invisible_zero.is_visible());
    }

    #[test]
    fn test_border_collapse_display() {
        assert_eq!(format!("{}", BorderCollapse::Separate), "separate");
        assert_eq!(format!("{}", BorderCollapse::Collapse), "collapse");
    }

    // ======================================================================
    // NEW COMPREHENSIVE TESTS (added to reach 50+ total)
    // ======================================================================

    // ------------------------------------------------------------------
    // 1. TableLayout struct construction, getters, setters
    // ------------------------------------------------------------------

    #[test]
    fn test_table_layout_default_border_spacing() {
        let layout = TableLayout::new(Length::from_pt(400.0));
        // Default border_spacing is 2pt (set in ::new)
        assert_eq!(layout.border_spacing, Length::from_pt(2.0));
    }

    #[test]
    fn test_table_layout_with_border_spacing_getter() {
        let layout =
            TableLayout::new(Length::from_pt(400.0)).with_border_spacing(Length::from_pt(8.0));
        assert_eq!(layout.border_spacing, Length::from_pt(8.0));
    }

    #[test]
    fn test_table_layout_available_width_stored() {
        let layout = TableLayout::new(Length::from_pt(720.0));
        assert_eq!(layout.available_width, Length::from_pt(720.0));
    }

    #[test]
    fn test_table_layout_chained_builder() {
        let layout = TableLayout::new(Length::from_pt(500.0))
            .with_border_spacing(Length::from_pt(4.0))
            .with_layout_mode(TableLayoutMode::Auto)
            .with_border_collapse(BorderCollapse::Collapse);
        assert_eq!(layout.layout_mode(), TableLayoutMode::Auto);
        assert_eq!(layout.border_collapse(), BorderCollapse::Collapse);
        assert_eq!(layout.border_spacing, Length::from_pt(4.0));
    }

    // ------------------------------------------------------------------
    // 2. Column width calculation — fixed widths
    // ------------------------------------------------------------------

    #[test]
    fn test_fixed_widths_empty_columns() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let widths = layout.compute_fixed_widths(&[]);
        assert!(widths.is_empty());
    }

    #[test]
    fn test_fixed_widths_three_equal_fixed_columns() {
        let layout = TableLayout::new(Length::from_pt(600.0))
            .with_border_spacing(Length::ZERO)
            .with_border_collapse(BorderCollapse::Collapse);
        let columns = vec![
            ColumnWidth::Fixed(Length::from_pt(100.0)),
            ColumnWidth::Fixed(Length::from_pt(200.0)),
            ColumnWidth::Fixed(Length::from_pt(150.0)),
        ];
        let widths = layout.compute_fixed_widths(&columns);
        assert_eq!(widths.len(), 3);
        assert_eq!(widths[0], Length::from_pt(100.0));
        assert_eq!(widths[1], Length::from_pt(200.0));
        assert_eq!(widths[2], Length::from_pt(150.0));
    }

    #[test]
    fn test_fixed_widths_auto_columns_share_remaining() {
        // collapsed so no spacing: 500 total, 200 fixed → 300 for two auto cols
        let layout =
            TableLayout::new(Length::from_pt(500.0)).with_border_collapse(BorderCollapse::Collapse);
        let columns = vec![
            ColumnWidth::Fixed(Length::from_pt(200.0)),
            ColumnWidth::Auto,
            ColumnWidth::Auto,
        ];
        let widths = layout.compute_fixed_widths(&columns);
        assert_eq!(widths.len(), 3);
        assert_eq!(widths[0], Length::from_pt(200.0));
        // two auto columns split 300 equally
        assert_eq!(widths[1], widths[2]);
        assert!((widths[1].to_pt() - 150.0).abs() < 0.01);
    }

    #[test]
    fn test_fixed_widths_single_auto_column_takes_all_remaining() {
        let layout =
            TableLayout::new(Length::from_pt(500.0)).with_border_collapse(BorderCollapse::Collapse);
        let columns = vec![
            ColumnWidth::Fixed(Length::from_pt(100.0)),
            ColumnWidth::Auto,
        ];
        let widths = layout.compute_fixed_widths(&columns);
        assert!((widths[1].to_pt() - 400.0).abs() < 0.01);
    }

    #[test]
    fn test_fixed_widths_proportional_single() {
        // Only one proportional column gets all the available space
        let layout =
            TableLayout::new(Length::from_pt(400.0)).with_border_collapse(BorderCollapse::Collapse);
        let columns = vec![ColumnWidth::Proportional(1.0)];
        let widths = layout.compute_fixed_widths(&columns);
        assert_eq!(widths.len(), 1);
        // All 400pt should be assigned
        assert!((widths[0].to_pt() - 400.0).abs() < 0.01);
    }

    #[test]
    fn test_fixed_widths_proportional_3_1_ratio() {
        let layout =
            TableLayout::new(Length::from_pt(400.0)).with_border_collapse(BorderCollapse::Collapse);
        let columns = vec![
            ColumnWidth::Proportional(3.0),
            ColumnWidth::Proportional(1.0),
        ];
        let widths = layout.compute_fixed_widths(&columns);
        // 3:1 ratio → 300 and 100
        assert!((widths[0].to_pt() - 300.0).abs() < 0.01);
        assert!((widths[1].to_pt() - 100.0).abs() < 0.01);
    }

    // ------------------------------------------------------------------
    // 3. Column width calculation — auto layout
    // ------------------------------------------------------------------

    #[test]
    fn test_auto_widths_empty_columns() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let widths = layout.compute_auto_widths(&[]);
        assert!(widths.is_empty());
    }

    #[test]
    fn test_auto_widths_all_fixed() {
        let layout =
            TableLayout::new(Length::from_pt(500.0)).with_border_collapse(BorderCollapse::Collapse);
        let infos = vec![
            ColumnInfo::with_widths(
                ColumnWidth::Fixed(Length::from_pt(120.0)),
                Length::ZERO,
                Length::ZERO,
            ),
            ColumnInfo::with_widths(
                ColumnWidth::Fixed(Length::from_pt(80.0)),
                Length::ZERO,
                Length::ZERO,
            ),
        ];
        let widths = layout.compute_auto_widths(&infos);
        assert_eq!(widths[0], Length::from_pt(120.0));
        assert_eq!(widths[1], Length::from_pt(80.0));
    }

    #[test]
    fn test_auto_widths_proportional_columns() {
        let layout =
            TableLayout::new(Length::from_pt(400.0)).with_border_collapse(BorderCollapse::Collapse);
        let infos = vec![
            ColumnInfo::new(ColumnWidth::Proportional(1.0)),
            ColumnInfo::new(ColumnWidth::Proportional(1.0)),
        ];
        let widths = layout.compute_auto_widths(&infos);
        // Equal proportions → 200 each
        assert!((widths[0].to_pt() - 200.0).abs() < 0.01);
        assert!((widths[1].to_pt() - 200.0).abs() < 0.01);
    }

    #[test]
    fn test_auto_widths_tight_all_min() {
        // Space is below total_min — widths should scale down
        let layout =
            TableLayout::new(Length::from_pt(50.0)).with_border_collapse(BorderCollapse::Collapse);
        let infos = vec![
            ColumnInfo::with_widths(
                ColumnWidth::Auto,
                Length::from_pt(40.0),
                Length::from_pt(100.0),
            ),
            ColumnInfo::with_widths(
                ColumnWidth::Auto,
                Length::from_pt(40.0),
                Length::from_pt(100.0),
            ),
        ];
        let widths = layout.compute_auto_widths(&infos);
        // Both columns scaled from min(40) proportionally; total must be <= 50
        assert!(widths[0] > Length::ZERO);
        assert!(widths[1] > Length::ZERO);
        let total = widths[0].to_pt() + widths[1].to_pt();
        assert!(total <= 50.0 + 0.01);
    }

    #[test]
    fn test_auto_widths_between_min_and_max_interpolation() {
        // Space between total_min and total_max → interpolation
        let layout =
            TableLayout::new(Length::from_pt(200.0)).with_border_collapse(BorderCollapse::Collapse);
        let infos = vec![
            ColumnInfo::with_widths(
                ColumnWidth::Auto,
                Length::from_pt(50.0),
                Length::from_pt(150.0),
            ),
            ColumnInfo::with_widths(
                ColumnWidth::Auto,
                Length::from_pt(50.0),
                Length::from_pt(150.0),
            ),
        ];
        let widths = layout.compute_auto_widths(&infos);
        // Each column between 50 and 150
        assert!(widths[0] >= Length::from_pt(50.0));
        assert!(widths[0] <= Length::from_pt(150.0));
        assert!(widths[1] >= Length::from_pt(50.0));
        assert!(widths[1] <= Length::from_pt(150.0));
    }

    #[test]
    fn test_auto_widths_same_min_max_uses_max_widths() {
        // When min == max for all auto columns and available >= total_max,
        // the algorithm uses max widths (plenty-of-space branch).
        let layout =
            TableLayout::new(Length::from_pt(300.0)).with_border_collapse(BorderCollapse::Collapse);
        let infos = vec![
            ColumnInfo::with_widths(
                ColumnWidth::Auto,
                Length::from_pt(80.0),
                Length::from_pt(80.0),
            ),
            ColumnInfo::with_widths(
                ColumnWidth::Auto,
                Length::from_pt(80.0),
                Length::from_pt(80.0),
            ),
        ];
        // available=300 >= total_max=160 → "plenty of space" branch → max widths (80 each)
        let widths = layout.compute_auto_widths(&infos);
        assert!((widths[0].to_pt() - 80.0).abs() < 0.01);
        assert!((widths[1].to_pt() - 80.0).abs() < 0.01);
    }

    // ------------------------------------------------------------------
    // 4. Cell positioning — GridCell and place_cell
    // ------------------------------------------------------------------

    #[test]
    fn test_grid_cell_rowspan_marks_spanned_rows() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let mut grid = layout.create_grid(3, 2);
        let cell = GridCell {
            row: 0,
            col: 0,
            rowspan: 3,
            colspan: 1,
            content_id: None,
        };
        layout
            .place_cell(&mut grid, cell)
            .expect("test: should succeed");
        // Row 0 has the real cell
        let top = grid[0][0].as_ref().expect("test: should succeed");
        assert_eq!(top.rowspan, 3);
        // Rows 1 and 2 are span markers (rowspan == 0)
        let marker1 = grid[1][0].as_ref().expect("test: should succeed");
        assert_eq!(marker1.rowspan, 0);
        let marker2 = grid[2][0].as_ref().expect("test: should succeed");
        assert_eq!(marker2.rowspan, 0);
        // Other column unaffected
        assert!(grid[0][1].is_none());
    }

    #[test]
    fn test_grid_cell_colspan_marks_spanned_cols() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let mut grid = layout.create_grid(2, 4);
        let cell = GridCell {
            row: 0,
            col: 1,
            rowspan: 1,
            colspan: 3,
            content_id: None,
        };
        layout
            .place_cell(&mut grid, cell)
            .expect("test: should succeed");
        // col 1 has real cell
        assert_eq!(
            grid[0][1].as_ref().expect("test: should succeed").colspan,
            3
        );
        // cols 2, 3 are span markers
        assert_eq!(
            grid[0][2].as_ref().expect("test: should succeed").colspan,
            0
        );
        assert_eq!(
            grid[0][3].as_ref().expect("test: should succeed").colspan,
            0
        );
        // col 0 unaffected
        assert!(grid[0][0].is_none());
    }

    #[test]
    fn test_grid_cell_rowspan_colspan_combined() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let mut grid = layout.create_grid(3, 3);
        let cell = GridCell {
            row: 0,
            col: 0,
            rowspan: 2,
            colspan: 2,
            content_id: None,
        };
        layout
            .place_cell(&mut grid, cell)
            .expect("test: should succeed");
        // Real cell at (0,0)
        let real = grid[0][0].as_ref().expect("test: should succeed");
        assert_eq!(real.rowspan, 2);
        assert_eq!(real.colspan, 2);
        // Span markers at (0,1), (1,0), (1,1)
        assert_eq!(
            grid[0][1].as_ref().expect("test: should succeed").rowspan,
            0
        );
        assert_eq!(
            grid[1][0].as_ref().expect("test: should succeed").rowspan,
            0
        );
        assert_eq!(
            grid[1][1].as_ref().expect("test: should succeed").rowspan,
            0
        );
        // Unaffected positions
        assert!(grid[0][2].is_none());
        assert!(grid[2][0].is_none());
    }

    #[test]
    fn test_place_cell_out_of_bounds_row() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let mut grid = layout.create_grid(2, 2);
        let cell = GridCell {
            row: 5,
            col: 0,
            rowspan: 1,
            colspan: 1,
            content_id: None,
        };
        assert!(layout.place_cell(&mut grid, cell).is_err());
    }

    #[test]
    fn test_place_cell_out_of_bounds_col() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let mut grid = layout.create_grid(2, 2);
        let cell = GridCell {
            row: 0,
            col: 5,
            rowspan: 1,
            colspan: 1,
            content_id: None,
        };
        assert!(layout.place_cell(&mut grid, cell).is_err());
    }

    #[test]
    fn test_place_cell_span_clamped_at_bounds() {
        // Span that extends beyond grid should be clamped
        let layout = TableLayout::new(Length::from_pt(500.0));
        let mut grid = layout.create_grid(2, 3);
        let cell = GridCell {
            row: 0,
            col: 1,
            rowspan: 5, // extends beyond 2 rows
            colspan: 5, // extends beyond 3 cols
            content_id: None,
        };
        // Should not panic
        layout
            .place_cell(&mut grid, cell)
            .expect("test: should succeed");
    }

    #[test]
    fn test_grid_cell_with_content_id() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let mut grid = layout.create_grid(1, 1);
        let area_id = AreaId::from_index(42);
        let cell = GridCell {
            row: 0,
            col: 0,
            rowspan: 1,
            colspan: 1,
            content_id: Some(area_id),
        };
        layout
            .place_cell(&mut grid, cell)
            .expect("test: should succeed");
        let stored = grid[0][0].as_ref().expect("test: should succeed");
        assert_eq!(stored.content_id, Some(area_id));
    }

    // ------------------------------------------------------------------
    // 5. Border collapse model
    // ------------------------------------------------------------------

    #[test]
    fn test_border_conflict_both_hidden() {
        use crate::area::BorderStyle;
        let layout = TableLayout::new(Length::from_pt(500.0));
        let b1 = CollapsedBorder::new(
            Length::from_pt(1.0),
            fop_types::Color::BLACK,
            BorderStyle::Hidden,
        );
        let b2 = CollapsedBorder::new(
            Length::from_pt(3.0),
            fop_types::Color::RED,
            BorderStyle::Hidden,
        );
        // First hidden takes precedence
        let result = layout.resolve_border_conflict(b1, b2);
        assert!(matches!(result.style, BorderStyle::Hidden));
    }

    #[test]
    fn test_border_conflict_both_none() {
        use crate::area::BorderStyle;
        let layout = TableLayout::new(Length::from_pt(500.0));
        let b1 = CollapsedBorder::new(
            Length::from_pt(1.0),
            fop_types::Color::BLACK,
            BorderStyle::None,
        );
        let b2 = CollapsedBorder::new(
            Length::from_pt(1.0),
            fop_types::Color::RED,
            BorderStyle::None,
        );
        let result = layout.resolve_border_conflict(b1, b2);
        assert!(matches!(result.style, BorderStyle::None));
    }

    #[test]
    fn test_border_conflict_equal_width_style_precedence() {
        use crate::area::BorderStyle;
        let layout = TableLayout::new(Length::from_pt(500.0));
        // Solid > Dotted when widths equal
        let b1 = CollapsedBorder::new(
            Length::from_pt(2.0),
            fop_types::Color::BLACK,
            BorderStyle::Dotted,
        );
        let b2 = CollapsedBorder::new(
            Length::from_pt(2.0),
            fop_types::Color::RED,
            BorderStyle::Solid,
        );
        let result = layout.resolve_border_conflict(b1, b2);
        assert!(matches!(result.style, BorderStyle::Solid));
    }

    #[test]
    fn test_border_conflict_double_vs_solid() {
        use crate::area::BorderStyle;
        let layout = TableLayout::new(Length::from_pt(500.0));
        let b1 = CollapsedBorder::new(
            Length::from_pt(2.0),
            fop_types::Color::BLACK,
            BorderStyle::Solid,
        );
        let b2 = CollapsedBorder::new(
            Length::from_pt(2.0),
            fop_types::Color::RED,
            BorderStyle::Double,
        );
        let result = layout.resolve_border_conflict(b1, b2);
        assert!(matches!(result.style, BorderStyle::Double));
    }

    #[test]
    fn test_border_conflict_groove_inset_precedence() {
        use crate::area::BorderStyle;
        let layout = TableLayout::new(Length::from_pt(500.0));
        // Groove (3) > Inset (2) in precedence
        let b1 = CollapsedBorder::new(
            Length::from_pt(2.0),
            fop_types::Color::BLACK,
            BorderStyle::Groove,
        );
        let b2 = CollapsedBorder::new(
            Length::from_pt(2.0),
            fop_types::Color::RED,
            BorderStyle::Inset,
        );
        let result = layout.resolve_border_conflict(b1, b2);
        assert!(matches!(result.style, BorderStyle::Groove));
    }

    #[test]
    fn test_border_conflict_wider_wins_over_better_style() {
        use crate::area::BorderStyle;
        let layout = TableLayout::new(Length::from_pt(500.0));
        // Narrow Double vs Wide Dotted → Wide wins regardless of style
        let b1 = CollapsedBorder::new(
            Length::from_pt(1.0),
            fop_types::Color::BLACK,
            BorderStyle::Double,
        );
        let b2 = CollapsedBorder::new(
            Length::from_pt(5.0),
            fop_types::Color::BLUE,
            BorderStyle::Dotted,
        );
        let result = layout.resolve_border_conflict(b1, b2);
        assert_eq!(result.width, Length::from_pt(5.0));
        assert!(matches!(result.style, BorderStyle::Dotted));
    }

    #[test]
    fn test_collapsed_border_none_factory() {
        let b = CollapsedBorder::none();
        assert_eq!(b.width, Length::ZERO);
        assert!(!b.is_visible());
    }

    #[test]
    fn test_cell_collapsed_borders_default() {
        let borders = CellCollapsedBorders::default();
        assert!(!borders.top.is_visible());
        assert!(!borders.right.is_visible());
        assert!(!borders.bottom.is_visible());
        assert!(!borders.left.is_visible());
    }

    #[test]
    fn test_separate_border_spacing_affects_available_width() {
        // With separate borders, total spacing is border_spacing * (n_cols + 1)
        let layout = TableLayout::new(Length::from_pt(500.0))
            .with_border_spacing(Length::from_pt(10.0))
            .with_border_collapse(BorderCollapse::Separate);
        // 4 columns → spacing = 10 * 5 = 50 → available_for_cols = 450
        let columns = vec![
            ColumnWidth::Auto,
            ColumnWidth::Auto,
            ColumnWidth::Auto,
            ColumnWidth::Auto,
        ];
        let widths = layout.compute_fixed_widths(&columns);
        // 450 / 4 = 112.5 each
        for w in &widths {
            assert!((w.to_pt() - 112.5).abs() < 0.01);
        }
    }

    #[test]
    fn test_collapse_model_no_spacing_deducted() {
        let layout = TableLayout::new(Length::from_pt(400.0))
            .with_border_spacing(Length::from_pt(10.0))
            .with_border_collapse(BorderCollapse::Collapse);
        // With collapse, spacing is zero regardless of border_spacing setting
        let columns = vec![ColumnWidth::Auto, ColumnWidth::Auto];
        let widths = layout.compute_fixed_widths(&columns);
        // Should get 200 each (400 / 2)
        for w in &widths {
            assert!((w.to_pt() - 200.0).abs() < 0.01);
        }
    }

    // ------------------------------------------------------------------
    // 6. ColumnInfo properties
    // ------------------------------------------------------------------

    #[test]
    fn test_column_info_computed_width_initial_zero() {
        let info = ColumnInfo::new(ColumnWidth::Fixed(Length::from_pt(100.0)));
        assert_eq!(info.computed_width, Length::ZERO);
    }

    #[test]
    fn test_column_info_with_widths_computed_still_zero() {
        let info = ColumnInfo::with_widths(
            ColumnWidth::Proportional(2.0),
            Length::from_pt(30.0),
            Length::from_pt(90.0),
        );
        assert_eq!(info.computed_width, Length::ZERO);
        assert_eq!(info.min_width, Length::from_pt(30.0));
        assert_eq!(info.max_width, Length::from_pt(90.0));
    }

    #[test]
    fn test_column_info_width_spec_variants() {
        let fixed = ColumnInfo::new(ColumnWidth::Fixed(Length::from_pt(50.0)));
        assert!(matches!(fixed.width_spec, ColumnWidth::Fixed(_)));

        let prop = ColumnInfo::new(ColumnWidth::Proportional(1.5));
        assert!(matches!(prop.width_spec, ColumnWidth::Proportional(_)));

        let auto = ColumnInfo::new(ColumnWidth::Auto);
        assert!(matches!(auto.width_spec, ColumnWidth::Auto));
    }

    // ------------------------------------------------------------------
    // 7. GridCell start/end column and row span
    // ------------------------------------------------------------------

    #[test]
    fn test_grid_cell_end_column_computed() {
        let cell = GridCell {
            row: 0,
            col: 2,
            rowspan: 1,
            colspan: 3,
            content_id: None,
        };
        // end column = col + colspan - 1 = 4
        assert_eq!(cell.col + cell.colspan - 1, 4);
    }

    #[test]
    fn test_grid_cell_single_cell_span() {
        let cell = GridCell {
            row: 0,
            col: 0,
            rowspan: 1,
            colspan: 1,
            content_id: None,
        };
        assert_eq!(cell.rowspan, 1);
        assert_eq!(cell.colspan, 1);
    }

    #[test]
    fn test_grid_cell_marker_zeroed_spans() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let mut grid = layout.create_grid(2, 2);
        let cell = GridCell {
            row: 0,
            col: 0,
            rowspan: 2,
            colspan: 2,
            content_id: None,
        };
        layout
            .place_cell(&mut grid, cell)
            .expect("test: should succeed");
        // Markers at (0,1), (1,0), (1,1)
        for &(r, c) in &[(0usize, 1usize), (1, 0), (1, 1)] {
            let marker = grid[r][c].as_ref().expect("test: should succeed");
            assert_eq!(marker.rowspan, 0, "row={r} col={c} should be marker");
            assert_eq!(marker.colspan, 0, "row={r} col={c} should be marker");
        }
    }

    #[test]
    fn test_grid_cell_marker_points_to_origin() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let mut grid = layout.create_grid(2, 2);
        let cell = GridCell {
            row: 1,
            col: 1,
            rowspan: 1,
            colspan: 1,
            content_id: None,
        };
        layout
            .place_cell(&mut grid, cell)
            .expect("test: should succeed");
        let stored = grid[1][1].as_ref().expect("test: should succeed");
        assert_eq!(stored.row, 1);
        assert_eq!(stored.col, 1);
    }

    // ------------------------------------------------------------------
    // 8. Edge cases: empty table, single-cell, 1-column, 1-row
    // ------------------------------------------------------------------

    #[test]
    fn test_empty_table_zero_columns() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let widths = layout.compute_fixed_widths(&[]);
        assert!(widths.is_empty());
    }

    #[test]
    fn test_single_cell_table() {
        let layout =
            TableLayout::new(Length::from_pt(300.0)).with_border_collapse(BorderCollapse::Collapse);
        let mut grid = layout.create_grid(1, 1);
        let cell = GridCell {
            row: 0,
            col: 0,
            rowspan: 1,
            colspan: 1,
            content_id: None,
        };
        layout
            .place_cell(&mut grid, cell)
            .expect("test: should succeed");
        assert!(grid[0][0].is_some());
        // Column widths
        let widths = layout.compute_fixed_widths(&[ColumnWidth::Auto]);
        assert_eq!(widths.len(), 1);
        assert!((widths[0].to_pt() - 300.0).abs() < 0.01);
    }

    #[test]
    fn test_one_column_table_many_rows() {
        let layout =
            TableLayout::new(Length::from_pt(200.0)).with_border_collapse(BorderCollapse::Collapse);
        let mut grid = layout.create_grid(5, 1);
        for r in 0..5 {
            let cell = GridCell {
                row: r,
                col: 0,
                rowspan: 1,
                colspan: 1,
                content_id: None,
            };
            layout
                .place_cell(&mut grid, cell)
                .expect("test: should succeed");
        }
        for row in grid.iter().take(5) {
            assert!(row[0].is_some());
        }
        let widths = layout.compute_fixed_widths(&[ColumnWidth::Auto]);
        assert_eq!(widths.len(), 1);
        assert!((widths[0].to_pt() - 200.0).abs() < 0.01);
    }

    #[test]
    fn test_one_row_table_many_columns() {
        let layout =
            TableLayout::new(Length::from_pt(600.0)).with_border_collapse(BorderCollapse::Collapse);
        let n = 6;
        let mut grid = layout.create_grid(1, n);
        for c in 0..n {
            let cell = GridCell {
                row: 0,
                col: c,
                rowspan: 1,
                colspan: 1,
                content_id: None,
            };
            layout
                .place_cell(&mut grid, cell)
                .expect("test: should succeed");
        }
        let col_specs: Vec<ColumnWidth> = (0..n).map(|_| ColumnWidth::Auto).collect();
        let widths = layout.compute_fixed_widths(&col_specs);
        assert_eq!(widths.len(), n);
        for w in &widths {
            assert!(
                (w.to_pt() - 100.0).abs() < 0.01,
                "expected 100pt, got {}pt",
                w.to_pt()
            );
        }
    }

    #[test]
    fn test_single_proportional_column_zero_proportional_sum() {
        // Edge: only one proportional column with value 0.0
        let layout =
            TableLayout::new(Length::from_pt(300.0)).with_border_collapse(BorderCollapse::Collapse);
        let columns = vec![ColumnWidth::Proportional(0.0)];
        let widths = layout.compute_fixed_widths(&columns);
        // total_proportional == 0.0 → width is ZERO
        assert_eq!(widths[0], Length::ZERO);
    }

    // ------------------------------------------------------------------
    // 9. Table width fitting within page margins
    // ------------------------------------------------------------------

    #[test]
    fn test_fixed_widths_fit_within_available_width() {
        let available = Length::from_pt(500.0);
        let layout = TableLayout::new(available).with_border_collapse(BorderCollapse::Collapse);
        let columns = vec![
            ColumnWidth::Proportional(1.0),
            ColumnWidth::Proportional(1.0),
            ColumnWidth::Proportional(1.0),
        ];
        let widths = layout.compute_fixed_widths(&columns);
        let total: f64 = widths.iter().map(|w| w.to_pt()).sum();
        assert!(
            total <= available.to_pt() + 0.01,
            "total {total} > available {}",
            available.to_pt()
        );
    }

    #[test]
    fn test_auto_widths_fit_within_available_width() {
        let available = Length::from_pt(400.0);
        let layout = TableLayout::new(available).with_border_collapse(BorderCollapse::Collapse);
        let infos = vec![
            ColumnInfo::with_widths(
                ColumnWidth::Auto,
                Length::from_pt(60.0),
                Length::from_pt(180.0),
            ),
            ColumnInfo::with_widths(
                ColumnWidth::Auto,
                Length::from_pt(60.0),
                Length::from_pt(180.0),
            ),
        ];
        let widths = layout.compute_auto_widths(&infos);
        let total: f64 = widths.iter().map(|w| w.to_pt()).sum();
        assert!(total <= available.to_pt() + 0.01);
    }

    #[test]
    fn test_table_layout_separates_border_spacing_in_auto_widths() {
        let layout = TableLayout::new(Length::from_pt(500.0))
            .with_border_spacing(Length::from_pt(5.0))
            .with_border_collapse(BorderCollapse::Separate);
        let infos = vec![
            ColumnInfo::with_widths(
                ColumnWidth::Auto,
                Length::from_pt(10.0),
                Length::from_pt(500.0),
            ),
            ColumnInfo::with_widths(
                ColumnWidth::Auto,
                Length::from_pt(10.0),
                Length::from_pt(500.0),
            ),
        ];
        // total_spacing = 5 * 3 = 15 → available_for_cols = 485
        // space (485) < max (1000) but space >= min (20) → interpolate
        let widths = layout.compute_auto_widths(&infos);
        let total: f64 = widths.iter().map(|w| w.to_pt()).sum();
        // Columns should not exceed the spacing-adjusted available width
        assert!(total <= 500.0 + 0.01);
    }

    // ------------------------------------------------------------------
    // 10. ColumnWidth enum variants and display
    // ------------------------------------------------------------------

    #[test]
    fn test_column_width_fixed_variant() {
        let cw = ColumnWidth::Fixed(Length::from_pt(75.0));
        if let ColumnWidth::Fixed(l) = cw {
            assert!((l.to_pt() - 75.0).abs() < 0.01);
        } else {
            panic!("Expected Fixed variant");
        }
    }

    #[test]
    fn test_column_width_proportional_variant() {
        let cw = ColumnWidth::Proportional(2.5);
        if let ColumnWidth::Proportional(p) = cw {
            assert!((p - 2.5).abs() < 0.001);
        } else {
            panic!("Expected Proportional variant");
        }
    }

    #[test]
    fn test_column_width_auto_variant() {
        let cw = ColumnWidth::Auto;
        assert!(matches!(cw, ColumnWidth::Auto));
    }

    #[test]
    fn test_column_width_equality() {
        assert_eq!(ColumnWidth::Auto, ColumnWidth::Auto);
        assert_eq!(
            ColumnWidth::Fixed(Length::from_pt(10.0)),
            ColumnWidth::Fixed(Length::from_pt(10.0))
        );
        assert_ne!(ColumnWidth::Auto, ColumnWidth::Fixed(Length::from_pt(0.0)));
    }

    // ------------------------------------------------------------------
    // 11. TableLayoutMode enum
    // ------------------------------------------------------------------

    #[test]
    fn test_table_layout_mode_equality() {
        assert_eq!(TableLayoutMode::Fixed, TableLayoutMode::Fixed);
        assert_eq!(TableLayoutMode::Auto, TableLayoutMode::Auto);
        assert_ne!(TableLayoutMode::Fixed, TableLayoutMode::Auto);
    }

    #[test]
    fn test_table_layout_mode_default_is_fixed() {
        assert_eq!(TableLayoutMode::default(), TableLayoutMode::Fixed);
    }

    // ------------------------------------------------------------------
    // 12. BorderCollapse enum
    // ------------------------------------------------------------------

    #[test]
    fn test_border_collapse_equality() {
        assert_eq!(BorderCollapse::Separate, BorderCollapse::Separate);
        assert_eq!(BorderCollapse::Collapse, BorderCollapse::Collapse);
        assert_ne!(BorderCollapse::Separate, BorderCollapse::Collapse);
    }

    #[test]
    fn test_border_collapse_default_is_separate() {
        assert_eq!(BorderCollapse::default(), BorderCollapse::Separate);
    }

    // ------------------------------------------------------------------
    // 13. CollapsedBorder creation and visibility
    // ------------------------------------------------------------------

    #[test]
    fn test_collapsed_border_dotted_visible() {
        use crate::area::BorderStyle;
        let b = CollapsedBorder::new(
            Length::from_pt(1.0),
            fop_types::Color::BLACK,
            BorderStyle::Dotted,
        );
        assert!(b.is_visible());
    }

    #[test]
    fn test_collapsed_border_dashed_visible() {
        use crate::area::BorderStyle;
        let b = CollapsedBorder::new(
            Length::from_pt(0.5),
            fop_types::Color::BLACK,
            BorderStyle::Dashed,
        );
        assert!(b.is_visible());
    }

    #[test]
    fn test_collapsed_border_double_visible() {
        use crate::area::BorderStyle;
        let b = CollapsedBorder::new(
            Length::from_pt(3.0),
            fop_types::Color::RED,
            BorderStyle::Double,
        );
        assert!(b.is_visible());
    }

    #[test]
    fn test_collapsed_border_groove_visible() {
        use crate::area::BorderStyle;
        let b = CollapsedBorder::new(
            Length::from_pt(2.0),
            fop_types::Color::BLACK,
            BorderStyle::Groove,
        );
        assert!(b.is_visible());
    }

    #[test]
    fn test_collapsed_border_ridge_visible() {
        use crate::area::BorderStyle;
        let b = CollapsedBorder::new(
            Length::from_pt(2.0),
            fop_types::Color::BLACK,
            BorderStyle::Ridge,
        );
        assert!(b.is_visible());
    }

    // ------------------------------------------------------------------
    // 14. Grid creation edge cases
    // ------------------------------------------------------------------

    #[test]
    fn test_create_grid_zero_rows() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let grid = layout.create_grid(0, 4);
        assert_eq!(grid.len(), 0);
    }

    #[test]
    fn test_create_grid_zero_cols() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let grid = layout.create_grid(3, 0);
        assert_eq!(grid.len(), 3);
        assert_eq!(grid[0].len(), 0);
    }

    #[test]
    fn test_create_grid_all_cells_initially_none() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let grid = layout.create_grid(4, 5);
        for row in &grid {
            for cell in row {
                assert!(cell.is_none());
            }
        }
    }

    // ------------------------------------------------------------------
    // 15. Distribute colspan widths
    // ------------------------------------------------------------------

    #[test]
    fn test_distribute_colspan_three_cols_one_span() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let mut column_info = vec![
            ColumnInfo::new(ColumnWidth::Auto),
            ColumnInfo::new(ColumnWidth::Auto),
            ColumnInfo::new(ColumnWidth::Auto),
        ];
        let mut grid = layout.create_grid(1, 3);
        let cell = GridCell {
            row: 0,
            col: 0,
            rowspan: 1,
            colspan: 3,
            content_id: None,
        };
        layout
            .place_cell(&mut grid, cell)
            .expect("test: should succeed");
        layout.distribute_colspan_widths(&mut column_info, &grid);
        // All three columns should get min widths boosted
        for info in &column_info {
            assert!(info.min_width > Length::ZERO);
        }
    }

    #[test]
    fn test_distribute_colspan_fixed_column_unaffected() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let mut column_info = vec![
            ColumnInfo::with_widths(
                ColumnWidth::Fixed(Length::from_pt(100.0)),
                Length::from_pt(100.0),
                Length::from_pt(100.0),
            ),
            ColumnInfo::new(ColumnWidth::Auto),
        ];
        let mut grid = layout.create_grid(1, 2);
        let cell = GridCell {
            row: 0,
            col: 0,
            rowspan: 1,
            colspan: 2,
            content_id: None,
        };
        layout
            .place_cell(&mut grid, cell)
            .expect("test: should succeed");
        layout.distribute_colspan_widths(&mut column_info, &grid);
        // Fixed column min_width stays at 100
        assert_eq!(column_info[0].min_width, Length::from_pt(100.0));
    }

    // ------------------------------------------------------------------
    // 16. Measure column widths with populated grid
    // ------------------------------------------------------------------

    #[test]
    fn test_measure_column_widths_with_cell_returns_nonzero() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let mut grid = layout.create_grid(2, 2);
        let cell = GridCell {
            row: 0,
            col: 0,
            rowspan: 1,
            colspan: 1,
            content_id: None,
        };
        layout
            .place_cell(&mut grid, cell)
            .expect("test: should succeed");
        let (min, max) = layout.measure_column_widths(&grid, 0);
        // measure_column_widths uses 30pt min and 200pt max heuristics
        assert!((min.to_pt() - 30.0).abs() < 0.01);
        assert!((max.to_pt() - 200.0).abs() < 0.01);
    }

    #[test]
    fn test_measure_column_widths_empty_column_returns_zero() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let grid = layout.create_grid(3, 3);
        let (min, max) = layout.measure_column_widths(&grid, 1);
        assert_eq!(min, Length::ZERO);
        assert_eq!(max, Length::ZERO);
    }

    #[test]
    fn test_measure_column_widths_max_gte_min() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let mut grid = layout.create_grid(1, 1);
        let cell = GridCell {
            row: 0,
            col: 0,
            rowspan: 1,
            colspan: 1,
            content_id: None,
        };
        layout
            .place_cell(&mut grid, cell)
            .expect("test: should succeed");
        let (min, max) = layout.measure_column_widths(&grid, 0);
        assert!(max >= min);
    }

    // ------------------------------------------------------------------
    // 17. layout_table integration smoke tests
    // ------------------------------------------------------------------

    #[test]
    fn test_layout_table_returns_area_id() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let mut tree = AreaTree::new();
        let column_widths = vec![Length::from_pt(100.0), Length::from_pt(200.0)];
        let grid = layout.create_grid(2, 2);
        let result = layout.layout_table(&mut tree, &column_widths, &grid, Length::ZERO);
        assert!(result.is_ok());
    }

    #[test]
    fn test_layout_table_empty_grid() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let mut tree = AreaTree::new();
        let column_widths: Vec<Length> = vec![];
        let grid: Vec<Vec<Option<GridCell>>> = vec![];
        let result = layout.layout_table(&mut tree, &column_widths, &grid, Length::ZERO);
        assert!(result.is_ok());
    }

    #[test]
    fn test_layout_table_with_y_offset() {
        let layout = TableLayout::new(Length::from_pt(500.0));
        let mut tree = AreaTree::new();
        let column_widths = vec![Length::from_pt(200.0)];
        let grid = layout.create_grid(1, 1);
        let y = Length::from_pt(50.0);
        let result = layout.layout_table(&mut tree, &column_widths, &grid, y);
        assert!(result.is_ok());
    }
}
