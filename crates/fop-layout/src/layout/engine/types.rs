//! Supporting types for the layout engine.
//!
//! Contains marker tracking, multi-column layout, float management,
//! page context, and page region geometry structures.

use fop_types::Length;

/// Geometry of all page regions derived from a simple-page-master
///
/// Holds the computed rectangles for each of the five XSL-FO page regions.
/// Dimensions are computed from the page-master's page size, margins, and
/// region extents.
#[derive(Debug, Clone, Copy)]
pub(super) struct PageRegionGeometry {
    /// Total page width (from page-master page-width attribute)
    pub page_width: Length,
    /// Total page height (from page-master page-height attribute)
    pub page_height: Length,
    /// Rectangle for region-before (header)
    pub before_rect: fop_types::Rect,
    /// Rectangle for region-after (footer)
    pub after_rect: fop_types::Rect,
    /// Rectangle for region-start (left sidebar)
    pub start_rect: fop_types::Rect,
    /// Rectangle for region-end (right sidebar)
    pub end_rect: fop_types::Rect,
    /// Rectangle for region-body (main content)
    pub body_rect: fop_types::Rect,
}

/// Multi-column layout configuration
///
/// Handles layout of content across multiple columns per CSS Multi-column
/// Layout Module Level 1 specification.
#[derive(Debug, Clone)]
pub struct MultiColumnLayout {
    /// Number of columns
    pub column_count: i32,
    /// Gap between columns
    pub column_gap: Length,
    /// Total available width
    pub available_width: Length,
    /// Width of each column
    pub column_width: Length,
    /// Current column index (0-based)
    pub current_column: i32,
    /// Current Y position within the current column
    pub column_y: Length,
    /// Maximum height per column (when page height is known)
    pub max_column_height: Option<Length>,
}

impl MultiColumnLayout {
    /// Create a new multi-column layout
    pub fn new(column_count: i32, column_gap: Length, available_width: Length) -> Self {
        // Calculate column width: (page_width - (n-1)*gap) / n
        let total_gap = column_gap * (column_count - 1);
        let column_width = (available_width - total_gap) / column_count;

        Self {
            column_count,
            column_gap,
            available_width,
            column_width,
            current_column: 0,
            column_y: Length::ZERO,
            max_column_height: None,
        }
    }

    /// Set the maximum column height (for balancing and page breaks)
    pub fn with_max_height(mut self, max_height: Length) -> Self {
        self.max_column_height = Some(max_height);
        self
    }

    /// Get the X offset for the current column
    pub fn current_column_x(&self) -> Length {
        (self.column_width + self.column_gap) * self.current_column
    }

    /// Check if the current column is filled (exceeds max height)
    pub fn is_column_filled(&self, content_height: Length) -> bool {
        if let Some(max_height) = self.max_column_height {
            self.column_y + content_height > max_height
        } else {
            false
        }
    }

    /// Move to the next column
    pub fn next_column(&mut self) -> bool {
        if self.current_column + 1 < self.column_count {
            self.current_column += 1;
            self.column_y = Length::ZERO;
            true
        } else {
            // All columns are filled - need new page
            false
        }
    }

    /// Allocate space in the current column
    pub fn allocate(&mut self, height: Length) -> (Length, Length) {
        let x = self.current_column_x();
        let y = self.column_y;
        self.column_y += height;
        (x, y)
    }

    /// Reset for a new page
    pub fn reset(&mut self) {
        self.current_column = 0;
        self.column_y = Length::ZERO;
    }

    /// Get the number of columns
    pub fn column_count(&self) -> i32 {
        self.column_count
    }

    /// Get the width of each column
    pub fn column_width(&self) -> Length {
        self.column_width
    }
}

/// Float side values for float property
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatSide {
    /// Float to the left
    Left,
    /// Float to the right
    Right,
    /// Float to the start edge (left in LTR, right in RTL)
    Start,
    /// Float to the end edge (right in LTR, left in RTL)
    End,
    /// Float inside (start on left pages, end on right pages)
    Inside,
    /// Float outside (end on left pages, start on right pages)
    Outside,
    /// No float
    None,
}

/// Clear values for clear property
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClearSide {
    /// Clear past left floats
    Left,
    /// Clear past right floats
    Right,
    /// Clear past both left and right floats
    Both,
    /// Clear past start floats
    Start,
    /// Clear past end floats
    End,
    /// No clearing
    None,
}

/// Information about a floating element
#[derive(Debug, Clone)]
pub(super) struct FloatInfo {
    /// The area ID of the float
    #[allow(dead_code)]
    pub(super) area_id: crate::area::AreaId,
    /// Side the float is on (left or right)
    pub(super) side: FloatSide,
    /// Top Y position of the float
    pub(super) top: Length,
    /// Bottom Y position of the float
    pub(super) bottom: Length,
    /// Width of the float
    pub(super) width: Length,
}

/// Manages active floating elements and calculates available space
#[derive(Debug, Default)]
pub(super) struct FloatManager {
    /// Currently active left floats
    pub(super) left_floats: Vec<FloatInfo>,
    /// Currently active right floats
    pub(super) right_floats: Vec<FloatInfo>,
}

impl FloatManager {
    /// Create a new empty float manager
    pub(super) fn new() -> Self {
        Self {
            left_floats: Vec::new(),
            right_floats: Vec::new(),
        }
    }

    /// Add a float to the manager
    ///
    /// # Parameters
    /// - `float`: The float information to add
    /// - `is_odd_page`: Whether the current page is odd-numbered (used for inside/outside positioning)
    pub(super) fn add_float(&mut self, float: FloatInfo, is_odd_page: bool) {
        match float.side {
            FloatSide::Left | FloatSide::Start => {
                self.left_floats.push(float);
            }
            FloatSide::Right | FloatSide::End => {
                self.right_floats.push(float);
            }
            FloatSide::Inside => {
                // Inside = verso (left page) uses right side, recto (right page) uses left side
                // Odd pages are recto (right pages), even pages are verso (left pages)
                if is_odd_page {
                    // Recto page (right) → inside is left
                    self.left_floats.push(float);
                } else {
                    // Verso page (left) → inside is right
                    self.right_floats.push(float);
                }
            }
            FloatSide::Outside => {
                // Outside = verso (left page) uses left side, recto (right page) uses right side
                if is_odd_page {
                    // Recto page (right) → outside is right
                    self.right_floats.push(float);
                } else {
                    // Verso page (left) → outside is left
                    self.left_floats.push(float);
                }
            }
            FloatSide::None => {}
        }
    }

    /// Get available width at a given Y position
    pub(super) fn available_width(&self, y: Length, container_width: Length) -> (Length, Length) {
        let left_offset = self.get_left_offset(y);
        let right_offset = self.get_right_offset(y);
        let available = container_width - left_offset - right_offset;
        (left_offset, available)
    }

    /// Get the left offset (space taken by left floats) at a given Y position
    pub(super) fn get_left_offset(&self, y: Length) -> Length {
        self.left_floats
            .iter()
            .filter(|f| f.top <= y && y < f.bottom)
            .map(|f| f.width)
            .fold(Length::ZERO, |acc, w| acc + w)
    }

    /// Get the right offset (space taken by right floats) at a given Y position
    pub(super) fn get_right_offset(&self, y: Length) -> Length {
        self.right_floats
            .iter()
            .filter(|f| f.top <= y && y < f.bottom)
            .map(|f| f.width)
            .fold(Length::ZERO, |acc, w| acc + w)
    }

    /// Get the Y position to clear past floats
    #[allow(dead_code)]
    pub(super) fn get_clear_position(&self, clear: ClearSide, current_y: Length) -> Length {
        match clear {
            ClearSide::Left | ClearSide::Start => self
                .left_floats
                .iter()
                .filter(|f| f.bottom > current_y)
                .map(|f| f.bottom)
                .max()
                .unwrap_or(current_y),
            ClearSide::Right | ClearSide::End => self
                .right_floats
                .iter()
                .filter(|f| f.bottom > current_y)
                .map(|f| f.bottom)
                .max()
                .unwrap_or(current_y),
            ClearSide::Both => {
                let left_bottom = self
                    .left_floats
                    .iter()
                    .filter(|f| f.bottom > current_y)
                    .map(|f| f.bottom)
                    .max()
                    .unwrap_or(current_y);
                let right_bottom = self
                    .right_floats
                    .iter()
                    .filter(|f| f.bottom > current_y)
                    .map(|f| f.bottom)
                    .max()
                    .unwrap_or(current_y);
                left_bottom.max(right_bottom)
            }
            ClearSide::None => current_y,
        }
    }

    /// Remove floats that are above the given Y position (no longer affecting layout)
    pub(super) fn remove_floats_above(&mut self, y: Length) {
        self.left_floats.retain(|f| f.bottom > y);
        self.right_floats.retain(|f| f.bottom > y);
    }

    /// Clear all floats
    pub(super) fn clear(&mut self) {
        self.left_floats.clear();
        self.right_floats.clear();
    }
}

/// Page context for tracking page position within a sequence
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(super) struct PageContext {
    /// Current page number within the sequence (1-based)
    pub(super) page_number: usize,
    /// Total number of pages in the sequence (if known)
    pub(super) total_pages: Option<usize>,
    /// Whether this is the first page
    pub(super) is_first: bool,
    /// Whether this is the last page (only known if total_pages is known)
    pub(super) is_last: bool,
}

impl PageContext {
    /// Create a new page context for the first page
    #[allow(dead_code)]
    pub(super) fn new() -> Self {
        Self {
            page_number: 1,
            total_pages: None,
            is_first: true,
            is_last: false,
        }
    }

    /// Check if this is an odd-numbered page
    #[allow(dead_code)]
    pub(super) fn is_odd_page(&self) -> bool {
        self.page_number % 2 == 1
    }

    /// Check if this is an even-numbered page
    #[allow(dead_code)]
    pub(super) fn is_even_page(&self) -> bool {
        self.page_number.is_multiple_of(2)
    }

    /// Check if this is the first page
    #[allow(dead_code)]
    pub(super) fn is_first_page(&self) -> bool {
        self.is_first
    }

    /// Check if this is the last page
    #[allow(dead_code)]
    pub(super) fn is_last_page(&self) -> bool {
        self.is_last
    }
}

/// Parse an XSL-FO length string (e.g., "10mm", "72pt") to a Length value.
#[allow(dead_code)]
pub(super) fn parse_fo_length(s: &str) -> Option<Length> {
    if let Some(v) = s.strip_suffix("pt") {
        v.parse::<f64>().ok().map(Length::from_pt)
    } else if let Some(v) = s.strip_suffix("mm") {
        v.parse::<f64>().ok().map(Length::from_mm)
    } else if let Some(v) = s.strip_suffix("cm") {
        v.parse::<f64>().ok().map(Length::from_cm)
    } else if let Some(v) = s.strip_suffix("in") {
        v.parse::<f64>().ok().map(Length::from_inch)
    } else if let Some(v) = s.strip_suffix("px") {
        v.parse::<f64>().ok().map(|px| Length::from_pt(px * 0.75))
    } else {
        None
    }
}
