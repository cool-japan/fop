//! List layout algorithm
//!
//! Implements layout for lists with labels and bodies positioned side-by-side.

use crate::area::{Area, AreaContent, AreaId, AreaTree, AreaType};
use fop_types::{Length, Point, Rect, Result, Size};

/// List layout engine
pub struct ListLayout {
    /// Available width for the list
    available_width: Length,

    /// End position of label area: start-indent + provisional-distance-between-starts - provisional-label-separation
    /// This corresponds to `label-end()` in XSL-FO
    label_width: Length,

    /// Gap between label end and body start (provisional-label-separation)
    label_separation: Length,

    /// Start position of body: start-indent + provisional-distance-between-starts
    /// This corresponds to `body-start()` in XSL-FO
    body_start_offset: Length,
}

/// Layout information for a list item
#[derive(Debug, Clone)]
pub struct ListItemLayout {
    /// Y position of the item
    pub y_position: Length,

    /// Height of the item
    pub height: Length,

    /// Label area ID
    pub label_id: Option<AreaId>,

    /// Body area ID
    pub body_id: Option<AreaId>,
}

impl ListLayout {
    /// Create a new list layout engine
    pub fn new(available_width: Length) -> Self {
        Self {
            available_width,
            label_width: Length::from_pt(18.0), // 24pt - 6pt = label_end by default
            label_separation: Length::from_pt(6.0), // Default provisional-label-separation
            body_start_offset: Length::from_pt(24.0), // Default provisional-distance-between-starts
        }
    }

    /// Set label end position (provisional-distance-between-starts - provisional-label-separation)
    /// This corresponds to the `label-end()` function value in XSL-FO.
    pub fn with_label_width(mut self, width: Length) -> Self {
        self.label_width = width;
        self
    }

    /// Set label separation (provisional-label-separation)
    pub fn with_label_separation(mut self, separation: Length) -> Self {
        self.label_separation = separation;
        self
    }

    /// Set body start position (provisional-distance-between-starts)
    /// This corresponds to the `body-start()` function value in XSL-FO.
    pub fn with_body_start(mut self, body_start: Length) -> Self {
        self.body_start_offset = body_start;
        self
    }

    /// Calculate body width (available_width - body_start)
    pub fn body_width(&self) -> Length {
        (self.available_width - self.body_start_offset).max(Length::from_pt(10.0))
    }

    /// Calculate body start x position — corresponds to `body-start()` in XSL-FO
    pub fn body_start(&self) -> Length {
        self.body_start_offset
    }

    /// Calculate label end position — corresponds to `label-end()` in XSL-FO
    pub fn label_end(&self) -> Length {
        self.label_width
    }

    /// Deprecated: use body_start() instead
    pub fn body_start_x(&self) -> Length {
        self.body_start_offset
    }

    /// Layout a single list item
    pub fn layout_item(
        &self,
        area_tree: &mut AreaTree,
        y_position: Length,
        label_content: Option<&str>,
        body_height: Length,
    ) -> Result<ListItemLayout> {
        let item_height = body_height.max(Length::from_pt(12.0)); // Minimum height

        // Create label area
        let label_id = if let Some(label_text) = label_content {
            let label_rect = Rect::from_point_size(
                Point::new(Length::ZERO, y_position),
                Size::new(self.label_width, item_height),
            );
            let mut label_area = Area::new(AreaType::Block, label_rect);
            label_area.content = Some(AreaContent::Text(label_text.to_string()));

            Some(area_tree.add_area(label_area))
        } else {
            None
        };

        // Create body area
        let body_rect = Rect::from_point_size(
            Point::new(self.body_start_x(), y_position),
            Size::new(self.body_width(), item_height),
        );
        let body_area = Area::new(AreaType::Block, body_rect);
        let body_id = Some(area_tree.add_area(body_area));

        Ok(ListItemLayout {
            y_position,
            height: item_height,
            label_id,
            body_id,
        })
    }

    /// Layout a complete list
    pub fn layout_list(
        &self,
        area_tree: &mut AreaTree,
        items: &[(Option<String>, Length)], // (label, body_height)
        start_y: Length,
    ) -> Result<Vec<ListItemLayout>> {
        let mut layouts = Vec::new();
        let mut current_y = start_y;

        for (label, body_height) in items {
            let layout = self.layout_item(area_tree, current_y, label.as_deref(), *body_height)?;

            current_y += layout.height;
            layouts.push(layout);
        }

        Ok(layouts)
    }

    /// Generate marker text based on list style
    pub fn generate_marker(&self, index: usize, style: ListMarkerStyle) -> String {
        match style {
            ListMarkerStyle::Disc => "•".to_string(),
            ListMarkerStyle::Circle => "○".to_string(),
            ListMarkerStyle::Square => "■".to_string(),
            ListMarkerStyle::Decimal => index.to_string(),
            ListMarkerStyle::LowerAlpha => Self::to_alpha(index, false),
            ListMarkerStyle::UpperAlpha => Self::to_alpha(index, true),
            ListMarkerStyle::LowerRoman => Self::to_roman(index, false),
            ListMarkerStyle::UpperRoman => Self::to_roman(index, true),
            ListMarkerStyle::None => String::new(),
        }
    }

    /// Convert number to alphabetic
    fn to_alpha(mut n: usize, uppercase: bool) -> String {
        if n == 0 {
            return String::new();
        }

        let mut result = String::new();
        while n > 0 {
            n -= 1;
            let ch = if uppercase {
                (b'A' + (n % 26) as u8) as char
            } else {
                (b'a' + (n % 26) as u8) as char
            };
            result.insert(0, ch);
            n /= 26;
        }
        result
    }

    /// Convert number to Roman numerals
    fn to_roman(n: usize, uppercase: bool) -> String {
        if n == 0 || n > 3999 {
            return n.to_string();
        }

        let values = [1000, 900, 500, 400, 100, 90, 50, 40, 10, 9, 5, 4, 1];
        let symbols_lower = [
            "m", "cm", "d", "cd", "c", "xc", "l", "xl", "x", "ix", "v", "iv", "i",
        ];
        let symbols_upper = [
            "M", "CM", "D", "CD", "C", "XC", "L", "XL", "X", "IX", "V", "IV", "I",
        ];

        let symbols = if uppercase {
            symbols_upper
        } else {
            symbols_lower
        };

        let mut result = String::new();
        let mut num = n;

        for (i, &value) in values.iter().enumerate() {
            while num >= value {
                result.push_str(symbols[i]);
                num -= value;
            }
        }

        result
    }
}

/// List marker styles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListMarkerStyle {
    /// Bullet (•)
    Disc,

    /// Circle (○)
    Circle,

    /// Square (■)
    Square,

    /// Decimal numbers
    Decimal,

    /// Lowercase letters
    LowerAlpha,

    /// Uppercase letters
    UpperAlpha,

    /// Lowercase Roman numerals
    LowerRoman,

    /// Uppercase Roman numerals
    UpperRoman,

    /// No marker
    None,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_layout_creation() {
        let layout = ListLayout::new(Length::from_pt(400.0));
        assert_eq!(layout.available_width, Length::from_pt(400.0));
        // Default: provisional-distance=24pt, separation=6pt => label_end=18pt
        assert_eq!(layout.label_end(), Length::from_pt(18.0));
        // Default body_start = provisional-distance = 24pt
        assert_eq!(layout.body_start(), Length::from_pt(24.0));
    }

    #[test]
    fn test_body_width_calculation() {
        // provisional-distance=60pt, separation=10pt
        // label_end = 60-10 = 50pt, body_start = 60pt
        // body_width = 400 - 60 = 340pt
        let layout = ListLayout::new(Length::from_pt(400.0))
            .with_label_width(Length::from_pt(50.0))
            .with_label_separation(Length::from_pt(10.0))
            .with_body_start(Length::from_pt(60.0));

        assert_eq!(layout.body_width(), Length::from_pt(340.0));
    }

    #[test]
    fn test_body_start_position() {
        let layout = ListLayout::new(Length::from_pt(400.0))
            .with_label_width(Length::from_pt(50.0))
            .with_label_separation(Length::from_pt(10.0))
            .with_body_start(Length::from_pt(60.0));

        // body-start() = provisional-distance-between-starts = 60pt
        assert_eq!(layout.body_start(), Length::from_pt(60.0));
        assert_eq!(layout.body_start_x(), Length::from_pt(60.0));
    }

    #[test]
    fn test_layout_single_item() {
        let layout = ListLayout::new(Length::from_pt(400.0));
        let mut area_tree = AreaTree::new();

        let item_layout = layout
            .layout_item(
                &mut area_tree,
                Length::ZERO,
                Some("1."),
                Length::from_pt(20.0),
            )
            .expect("test: should succeed");

        assert_eq!(item_layout.y_position, Length::ZERO);
        assert_eq!(item_layout.height, Length::from_pt(20.0));
        assert!(item_layout.label_id.is_some());
        assert!(item_layout.body_id.is_some());
    }

    #[test]
    fn test_layout_item_without_label() {
        let layout = ListLayout::new(Length::from_pt(400.0));
        let mut area_tree = AreaTree::new();

        let item_layout = layout
            .layout_item(&mut area_tree, Length::ZERO, None, Length::from_pt(20.0))
            .expect("test: should succeed");

        assert!(item_layout.label_id.is_none());
        assert!(item_layout.body_id.is_some());
    }

    #[test]
    fn test_layout_complete_list() {
        let layout = ListLayout::new(Length::from_pt(400.0));
        let mut area_tree = AreaTree::new();

        let items = vec![
            (Some("1.".to_string()), Length::from_pt(20.0)),
            (Some("2.".to_string()), Length::from_pt(30.0)),
            (Some("3.".to_string()), Length::from_pt(20.0)),
        ];

        let layouts = layout
            .layout_list(&mut area_tree, &items, Length::ZERO)
            .expect("test: should succeed");

        assert_eq!(layouts.len(), 3);
        assert_eq!(layouts[0].y_position, Length::ZERO);
        assert_eq!(layouts[1].y_position, Length::from_pt(20.0));
        assert_eq!(layouts[2].y_position, Length::from_pt(50.0));
    }

    #[test]
    fn test_marker_disc() {
        let layout = ListLayout::new(Length::from_pt(400.0));
        assert_eq!(layout.generate_marker(1, ListMarkerStyle::Disc), "•");
    }

    #[test]
    fn test_marker_decimal() {
        let layout = ListLayout::new(Length::from_pt(400.0));
        assert_eq!(layout.generate_marker(5, ListMarkerStyle::Decimal), "5");
    }

    #[test]
    fn test_marker_lower_alpha() {
        let layout = ListLayout::new(Length::from_pt(400.0));
        assert_eq!(layout.generate_marker(1, ListMarkerStyle::LowerAlpha), "a");
        assert_eq!(layout.generate_marker(26, ListMarkerStyle::LowerAlpha), "z");
        assert_eq!(
            layout.generate_marker(27, ListMarkerStyle::LowerAlpha),
            "aa"
        );
    }

    #[test]
    fn test_marker_lower_roman() {
        let layout = ListLayout::new(Length::from_pt(400.0));
        assert_eq!(layout.generate_marker(1, ListMarkerStyle::LowerRoman), "i");
        assert_eq!(layout.generate_marker(4, ListMarkerStyle::LowerRoman), "iv");
        assert_eq!(
            layout.generate_marker(1994, ListMarkerStyle::LowerRoman),
            "mcmxciv"
        );
    }

    #[test]
    fn test_marker_upper_roman() {
        let layout = ListLayout::new(Length::from_pt(400.0));
        assert_eq!(layout.generate_marker(1, ListMarkerStyle::UpperRoman), "I");
        assert_eq!(
            layout.generate_marker(2023, ListMarkerStyle::UpperRoman),
            "MMXXIII"
        );
    }

    #[test]
    fn test_marker_none() {
        let layout = ListLayout::new(Length::from_pt(400.0));
        assert_eq!(layout.generate_marker(1, ListMarkerStyle::None), "");
    }
    #[test]
    fn test_roman_zero_returns_zero_string() {
        // to_roman(0) returns "0" (can't represent zero in Roman numerals)
        let layout = ListLayout::new(Length::from_pt(400.0));
        let result = layout.generate_marker(0, ListMarkerStyle::LowerRoman);
        assert_eq!(
            result, "0",
            "Zero should return '0' since it has no Roman form"
        );
    }

    #[test]
    fn test_roman_large_over_3999_returns_decimal() {
        // to_roman(4000) returns "4000" (out of range for Roman numerals)
        let layout = ListLayout::new(Length::from_pt(400.0));
        let result = layout.generate_marker(4000, ListMarkerStyle::LowerRoman);
        assert_eq!(result, "4000", "Numbers > 3999 should fall back to decimal");
    }

    #[test]
    fn test_roman_3999_max_valid() {
        let layout = ListLayout::new(Length::from_pt(400.0));
        let result = layout.generate_marker(3999, ListMarkerStyle::UpperRoman);
        assert_eq!(result, "MMMCMXCIX");
    }

    #[test]
    fn test_roman_subtractive_notation() {
        let layout = ListLayout::new(Length::from_pt(400.0));
        assert_eq!(layout.generate_marker(4, ListMarkerStyle::LowerRoman), "iv");
        assert_eq!(layout.generate_marker(9, ListMarkerStyle::LowerRoman), "ix");
        assert_eq!(
            layout.generate_marker(40, ListMarkerStyle::LowerRoman),
            "xl"
        );
        assert_eq!(
            layout.generate_marker(90, ListMarkerStyle::LowerRoman),
            "xc"
        );
        assert_eq!(
            layout.generate_marker(400, ListMarkerStyle::LowerRoman),
            "cd"
        );
        assert_eq!(
            layout.generate_marker(900, ListMarkerStyle::LowerRoman),
            "cm"
        );
    }

    #[test]
    fn test_alpha_zero_returns_empty() {
        // to_alpha(0) returns "" (undefined)
        let layout = ListLayout::new(Length::from_pt(400.0));
        let result = layout.generate_marker(0, ListMarkerStyle::LowerAlpha);
        assert_eq!(result, "");
    }

    #[test]
    fn test_alpha_overflow_27_is_aa() {
        let layout = ListLayout::new(Length::from_pt(400.0));
        assert_eq!(
            layout.generate_marker(27, ListMarkerStyle::LowerAlpha),
            "aa"
        );
    }

    #[test]
    fn test_alpha_overflow_52_is_az() {
        // 52 = 26 + 26, so it's "az"
        let layout = ListLayout::new(Length::from_pt(400.0));
        assert_eq!(
            layout.generate_marker(52, ListMarkerStyle::LowerAlpha),
            "az"
        );
    }

    #[test]
    fn test_alpha_overflow_53_is_ba() {
        // 53 = 2*26 + 1 -> "ba"
        let layout = ListLayout::new(Length::from_pt(400.0));
        assert_eq!(
            layout.generate_marker(53, ListMarkerStyle::LowerAlpha),
            "ba"
        );
    }

    #[test]
    fn test_alpha_upper_case() {
        let layout = ListLayout::new(Length::from_pt(400.0));
        assert_eq!(layout.generate_marker(1, ListMarkerStyle::UpperAlpha), "A");
        assert_eq!(layout.generate_marker(26, ListMarkerStyle::UpperAlpha), "Z");
        assert_eq!(
            layout.generate_marker(27, ListMarkerStyle::UpperAlpha),
            "AA"
        );
    }

    #[test]
    fn test_marker_circle_and_square() {
        let layout = ListLayout::new(Length::from_pt(400.0));
        assert_eq!(layout.generate_marker(1, ListMarkerStyle::Circle), "○");
        assert_eq!(layout.generate_marker(1, ListMarkerStyle::Square), "■");
    }
}
