//! Table layout types: enums, structs for table, column, cell, and border data

use crate::area::AreaId;
use fop_types::Length;
use std::fmt;

/// Table layout algorithm mode
///
/// Corresponds to the XSL-FO table-layout property.
/// - "auto": Column widths are determined by content (CSS2.1 auto table layout)
/// - "fixed": Column widths are determined by explicit widths (default)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TableLayoutMode {
    /// Auto layout - column widths determined by content
    /// Implements CSS2.1 automatic table layout algorithm
    Auto,
    /// Fixed layout - column widths from table-column or first row
    /// Faster and more predictable than auto layout
    #[default]
    Fixed,
}

/// Border collapse model for tables
///
/// Corresponds to the CSS/XSL-FO border-collapse property.
/// - "separate": Borders are drawn around each cell with spacing between them
/// - "collapse": Adjacent cell borders are merged into a single border
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BorderCollapse {
    /// Separate borders (default) - each cell has its own borders with spacing
    #[default]
    Separate,
    /// Collapsed borders - adjacent borders merge with conflict resolution
    Collapse,
}

impl fmt::Display for BorderCollapse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BorderCollapse::Separate => write!(f, "separate"),
            BorderCollapse::Collapse => write!(f, "collapse"),
        }
    }
}

/// Column width specification
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnWidth {
    /// Fixed width
    Fixed(Length),

    /// Proportional width (e.g., 2* = 2 units)
    Proportional(f64),

    /// Auto width (calculated from content)
    Auto,
}

/// Information about a table column
///
/// Used in auto layout algorithm to track column width constraints.
#[derive(Debug, Clone)]
pub struct ColumnInfo {
    /// Column width specification
    pub width_spec: ColumnWidth,

    /// Computed width (after layout)
    pub computed_width: Length,

    /// Minimum content width
    /// Smallest width that prevents content overflow
    pub min_width: Length,

    /// Maximum content width
    /// Preferred width without any line breaking
    pub max_width: Length,
}

impl ColumnInfo {
    /// Create a new ColumnInfo with the given width specification
    pub fn new(width_spec: ColumnWidth) -> Self {
        Self {
            width_spec,
            computed_width: Length::ZERO,
            min_width: Length::ZERO,
            max_width: Length::ZERO,
        }
    }

    /// Create a ColumnInfo with explicit min/max widths
    pub fn with_widths(width_spec: ColumnWidth, min_width: Length, max_width: Length) -> Self {
        Self {
            width_spec,
            computed_width: Length::ZERO,
            min_width,
            max_width,
        }
    }
}

/// A cell in the table grid
#[derive(Debug, Clone)]
pub struct GridCell {
    /// Row index
    pub row: usize,

    /// Column index
    pub col: usize,

    /// Number of rows spanned
    pub rowspan: usize,

    /// Number of columns spanned
    pub colspan: usize,

    /// Content area ID
    pub content_id: Option<AreaId>,
}

/// Collapsed border information for a cell edge
///
/// In the collapsed border model, adjacent cell borders merge into a single border.
/// This structure stores the winning border after conflict resolution.
#[derive(Debug, Clone, Copy)]
pub struct CollapsedBorder {
    /// Border width
    pub width: Length,

    /// Border color
    pub color: fop_types::Color,

    /// Border style
    pub style: crate::area::BorderStyle,
}

impl CollapsedBorder {
    /// Create a new collapsed border
    pub fn new(width: Length, color: fop_types::Color, style: crate::area::BorderStyle) -> Self {
        Self {
            width,
            color,
            style,
        }
    }

    /// Create a none/hidden border (no border)
    pub fn none() -> Self {
        Self {
            width: Length::ZERO,
            color: fop_types::Color::BLACK,
            style: crate::area::BorderStyle::None,
        }
    }

    /// Check if this border is visible
    pub fn is_visible(&self) -> bool {
        self.width > Length::ZERO
            && !matches!(
                self.style,
                crate::area::BorderStyle::None | crate::area::BorderStyle::Hidden
            )
    }
}

/// Table layout engine
pub struct TableLayout {
    /// Available width for the table
    pub(super) available_width: Length,

    /// Border spacing (for separate border model)
    pub(super) border_spacing: Length,

    /// Table layout mode (auto or fixed)
    pub(super) layout_mode: TableLayoutMode,

    /// Border collapse model
    pub(super) border_collapse: BorderCollapse,
}

impl TableLayout {
    /// Create a new table layout engine
    pub fn new(available_width: Length) -> Self {
        Self {
            available_width,
            border_spacing: Length::from_pt(2.0),
            layout_mode: TableLayoutMode::Fixed,
            border_collapse: BorderCollapse::Separate,
        }
    }

    /// Set border spacing
    pub fn with_border_spacing(mut self, spacing: Length) -> Self {
        self.border_spacing = spacing;
        self
    }

    /// Set table layout mode
    pub fn with_layout_mode(mut self, mode: TableLayoutMode) -> Self {
        self.layout_mode = mode;
        self
    }

    /// Set border collapse model
    pub fn with_border_collapse(mut self, collapse: BorderCollapse) -> Self {
        self.border_collapse = collapse;
        self
    }

    /// Get the current table layout mode
    pub fn layout_mode(&self) -> TableLayoutMode {
        self.layout_mode
    }

    /// Get the current border collapse model
    pub fn border_collapse(&self) -> BorderCollapse {
        self.border_collapse
    }

    /// Get the available width budget for the whole table (all columns plus any
    /// inter-cell spacing).  This is the width passed to [`TableLayout::new`].
    pub fn available_width(&self) -> Length {
        self.available_width
    }

    /// Get the configured border spacing (used only in the separate-border model).
    pub fn border_spacing(&self) -> Length {
        self.border_spacing
    }

    /// Width left for the column boxes themselves once inter-cell spacing has been
    /// subtracted.  In the collapsed-border model there is no spacing, so this is
    /// the full [`TableLayout::available_width`]; in the separate model it removes
    /// `border_spacing × (n_cols + 1)`.
    ///
    /// This mirrors the budget that [`TableLayout::compute_auto_widths`] and
    /// [`TableLayout::compute_fixed_widths`] distribute across the columns, so the
    /// engine can grow auto columns to exactly fill the table.
    pub fn content_width_for_columns(&self, n_cols: usize) -> Length {
        let total_spacing = match self.border_collapse {
            BorderCollapse::Separate => self.border_spacing * (n_cols + 1) as i32,
            BorderCollapse::Collapse => Length::ZERO,
        };
        self.available_width - total_spacing
    }
}

/// Collapsed borders for all four edges of a cell
///
/// Stores the winning border after conflict resolution for each edge.
#[derive(Debug, Clone, Copy)]
pub struct CellCollapsedBorders {
    /// Top border
    pub top: CollapsedBorder,
    /// Right border
    pub right: CollapsedBorder,
    /// Bottom border
    pub bottom: CollapsedBorder,
    /// Left border
    pub left: CollapsedBorder,
}

impl Default for CellCollapsedBorders {
    fn default() -> Self {
        Self {
            top: CollapsedBorder::none(),
            right: CollapsedBorder::none(),
            bottom: CollapsedBorder::none(),
            left: CollapsedBorder::none(),
        }
    }
}
