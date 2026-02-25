//! Column width calculation algorithms (fixed and auto layout)

use fop_types::Length;

use super::types::{BorderCollapse, ColumnInfo, ColumnWidth, GridCell, TableLayout};

impl TableLayout {
    /// Compute column widths using fixed layout algorithm
    pub fn compute_fixed_widths(&self, columns: &[ColumnWidth]) -> Vec<Length> {
        let n_cols = columns.len();
        if n_cols == 0 {
            return Vec::new();
        }

        // For collapsed borders, there's no spacing between cells
        let total_spacing = match self.border_collapse {
            BorderCollapse::Separate => self.border_spacing * (n_cols + 1) as i32,
            BorderCollapse::Collapse => Length::ZERO,
        };
        let available_for_cols = self.available_width - total_spacing;

        // Count proportional units
        let total_proportional: f64 = columns
            .iter()
            .filter_map(|w| {
                if let ColumnWidth::Proportional(p) = w {
                    Some(*p)
                } else {
                    None
                }
            })
            .sum();

        // Calculate fixed total
        let fixed_total: Length = columns
            .iter()
            .filter_map(|w| {
                if let ColumnWidth::Fixed(len) = w {
                    Some(*len)
                } else {
                    None
                }
            })
            .fold(Length::ZERO, |acc, len| acc + len);

        // Remaining width for proportional/auto
        let remaining = available_for_cols - fixed_total;

        // Compute each column width
        columns
            .iter()
            .map(|spec| match spec {
                ColumnWidth::Fixed(len) => *len,
                ColumnWidth::Proportional(p) => {
                    if total_proportional > 0.0 {
                        Length::from_pt(remaining.to_pt() * p / total_proportional)
                    } else {
                        Length::ZERO
                    }
                }
                ColumnWidth::Auto => {
                    // In fixed layout, auto gets equal share of remaining
                    let auto_count = columns
                        .iter()
                        .filter(|w| matches!(w, ColumnWidth::Auto))
                        .count();
                    if auto_count > 0 {
                        remaining / auto_count as i32
                    } else {
                        Length::ZERO
                    }
                }
            })
            .collect()
    }

    /// Compute column widths using auto layout algorithm
    ///
    /// This implements the CSS2.1 automatic table layout algorithm:
    /// 1. Calculate minimum and maximum width for each column based on content
    /// 2. Assign fixed widths first
    /// 3. Distribute remaining width to auto columns based on their min/max widths
    /// 4. Handle proportional widths
    ///
    /// Algorithm details:
    /// - **Fixed columns**: Use their specified width
    /// - **Proportional columns**: Distribute remaining width by ratio
    /// - **Auto columns**: Use content-based min/max widths
    ///   - If space >= total max: Use max widths (no line breaking)
    ///   - If min <= space < max: Interpolate between min and max
    ///   - If space < min: Scale down from min (may overflow)
    ///
    /// This matches Apache FOP's AutoLayoutAlgorithm.java behavior.
    pub fn compute_auto_widths(&self, column_info: &[ColumnInfo]) -> Vec<Length> {
        let n_cols = column_info.len();
        if n_cols == 0 {
            return Vec::new();
        }

        // For collapsed borders, there's no spacing between cells
        let total_spacing = match self.border_collapse {
            BorderCollapse::Separate => self.border_spacing * (n_cols + 1) as i32,
            BorderCollapse::Collapse => Length::ZERO,
        };
        let available_for_cols = self.available_width - total_spacing;

        let mut widths = vec![Length::ZERO; n_cols];

        // Step 1: Assign fixed widths
        let mut fixed_total = Length::ZERO;
        let mut auto_count = 0;
        let mut proportional_count = 0;

        for (i, info) in column_info.iter().enumerate() {
            match info.width_spec {
                ColumnWidth::Fixed(len) => {
                    widths[i] = len;
                    fixed_total += len;
                }
                ColumnWidth::Auto => {
                    auto_count += 1;
                }
                ColumnWidth::Proportional(_) => {
                    proportional_count += 1;
                }
            }
        }

        let remaining = available_for_cols - fixed_total;

        // Step 2: Handle proportional columns
        if proportional_count > 0 {
            let total_proportional: f64 = column_info
                .iter()
                .filter_map(|info| {
                    if let ColumnWidth::Proportional(p) = info.width_spec {
                        Some(p)
                    } else {
                        None
                    }
                })
                .sum();

            if total_proportional > 0.0 {
                for (i, info) in column_info.iter().enumerate() {
                    if let ColumnWidth::Proportional(p) = info.width_spec {
                        widths[i] = Length::from_pt(remaining.to_pt() * p / total_proportional);
                    }
                }
                return widths;
            }
        }

        // Step 3: Auto layout algorithm
        if auto_count > 0 {
            // Calculate total min and max widths for auto columns
            let mut total_min = Length::ZERO;
            let mut total_max = Length::ZERO;

            for info in column_info.iter() {
                if matches!(info.width_spec, ColumnWidth::Auto) {
                    total_min += info.min_width;
                    total_max += info.max_width;
                }
            }

            // Distribute remaining width based on min/max constraints
            if remaining >= total_max {
                // Plenty of space - use max widths
                for (i, info) in column_info.iter().enumerate() {
                    if matches!(info.width_spec, ColumnWidth::Auto) {
                        widths[i] = info.max_width;
                    }
                }
            } else if remaining >= total_min {
                // Between min and max - distribute proportionally
                let range = total_max - total_min;
                if range > Length::ZERO {
                    for (i, info) in column_info.iter().enumerate() {
                        if matches!(info.width_spec, ColumnWidth::Auto) {
                            let col_range = info.max_width - info.min_width;
                            let ratio = col_range.to_pt() / range.to_pt();
                            let extra = Length::from_pt((remaining - total_min).to_pt() * ratio);
                            widths[i] = info.min_width + extra;
                        }
                    }
                } else {
                    // All columns have same min/max - distribute equally
                    let per_col = remaining / auto_count;
                    for (i, info) in column_info.iter().enumerate() {
                        if matches!(info.width_spec, ColumnWidth::Auto) {
                            widths[i] = per_col;
                        }
                    }
                }
            } else {
                // Not enough space - use min widths and scale down if needed
                if total_min > Length::ZERO {
                    let scale = remaining.to_pt() / total_min.to_pt();
                    for (i, info) in column_info.iter().enumerate() {
                        if matches!(info.width_spec, ColumnWidth::Auto) {
                            widths[i] = Length::from_pt(info.min_width.to_pt() * scale);
                        }
                    }
                } else {
                    // Distribute equally
                    let per_col = remaining / auto_count;
                    for (i, info) in column_info.iter().enumerate() {
                        if matches!(info.width_spec, ColumnWidth::Auto) {
                            widths[i] = per_col;
                        }
                    }
                }
            }
        }

        widths
    }

    /// Measure content widths for cells in a column
    /// Returns (min_width, max_width) for the column
    pub fn measure_column_widths(
        &self,
        grid: &[Vec<Option<GridCell>>],
        col_idx: usize,
    ) -> (Length, Length) {
        let mut min_width = Length::ZERO;
        let mut max_width = Length::ZERO;

        for row in grid {
            if let Some(Some(cell)) = row.get(col_idx) {
                // Only process actual cells (not span markers)
                if cell.rowspan > 0 && cell.colspan > 0 {
                    // For now, use simple heuristics
                    // In a full implementation, this would measure actual text content
                    let cell_min = Length::from_pt(30.0); // Minimum cell width
                    let cell_max = Length::from_pt(200.0); // Maximum preferred width

                    min_width = min_width.max(cell_min);
                    max_width = max_width.max(cell_max);
                }
            }
        }

        // Ensure max >= min
        if max_width < min_width {
            max_width = min_width;
        }

        (min_width, max_width)
    }

    /// Update column info with measured widths from grid
    pub fn update_column_info_from_grid(
        &self,
        column_info: &mut [ColumnInfo],
        grid: &[Vec<Option<GridCell>>],
    ) {
        for (col_idx, info) in column_info.iter_mut().enumerate() {
            if matches!(info.width_spec, ColumnWidth::Auto) {
                let (min, max) = self.measure_column_widths(grid, col_idx);
                info.min_width = min;
                info.max_width = max;
            }
        }
    }

    /// Handle cells with colspan in width calculation
    /// This distributes the cell's required width across spanned columns
    pub fn distribute_colspan_widths(
        &self,
        column_info: &mut [ColumnInfo],
        grid: &[Vec<Option<GridCell>>],
    ) {
        for row in grid {
            for (col_idx, cell_opt) in row.iter().enumerate() {
                if let Some(cell) = cell_opt {
                    // Process cells with colspan > 1
                    if cell.rowspan > 0 && cell.colspan > 1 && cell.col == col_idx {
                        let end_col = (cell.col + cell.colspan).min(column_info.len());

                        // Calculate total width of spanned columns
                        let mut total_min = Length::ZERO;
                        let mut auto_cols = 0;

                        for i in cell.col..end_col {
                            if let Some(info) = column_info.get(i) {
                                if matches!(info.width_spec, ColumnWidth::Auto) {
                                    total_min += info.min_width;
                                    auto_cols += 1;
                                }
                            }
                        }

                        // For colspan cells, ensure minimum width
                        // In a full implementation, this would measure the cell content
                        let cell_min = Length::from_pt(50.0 * cell.colspan as f64);

                        if cell_min > total_min && auto_cols > 0 {
                            // Distribute extra width needed across auto columns
                            let extra_per_col = (cell_min - total_min) / auto_cols;

                            for i in cell.col..end_col {
                                if let Some(info) = column_info.get_mut(i) {
                                    if matches!(info.width_spec, ColumnWidth::Auto) {
                                        info.min_width += extra_per_col;
                                        info.max_width = info.max_width.max(info.min_width);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
