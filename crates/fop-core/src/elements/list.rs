//! List-related FO elements
//!
//! Implements fo:list-block and fo:list-item

use crate::properties::PropertyList;
use crate::Length;

/// List block element (fo:list-block)
#[derive(Debug, Clone)]
pub struct ListBlock<'a> {
    /// Property list
    pub properties: PropertyList<'a>,

    /// List items
    pub items: Vec<ListItem<'a>>,

    /// Provisional distance between starts (space for label)
    pub provisional_distance_between_starts: Length,

    /// Provisional label separation (space between label and body)
    pub provisional_label_separation: Length,
}

/// List item element (fo:list-item)
#[derive(Debug, Clone)]
pub struct ListItem<'a> {
    /// Property list
    pub properties: PropertyList<'a>,

    /// List item label (fo:list-item-label)
    pub label: ListItemLabel<'a>,

    /// List item body (fo:list-item-body)
    pub body: ListItemBody<'a>,
}

/// List item label (fo:list-item-label)
#[derive(Debug, Clone)]
pub struct ListItemLabel<'a> {
    /// Property list
    pub properties: PropertyList<'a>,

    /// Label content (blocks)
    pub blocks: Vec<super::Block<'a>>,
}

/// List item body (fo:list-item-body)
#[derive(Debug, Clone)]
pub struct ListItemBody<'a> {
    /// Property list
    pub properties: PropertyList<'a>,

    /// Body content (blocks)
    pub blocks: Vec<super::Block<'a>>,
}

impl<'a> ListBlock<'a> {
    /// Create a new list block
    pub fn new(properties: PropertyList<'a>) -> Self {
        Self {
            properties,
            items: Vec::new(),
            provisional_distance_between_starts: Length::from_pt(24.0),  // Default
            provisional_label_separation: Length::from_pt(6.0),  // Default
        }
    }

    /// Get the number of items in the list
    pub fn item_count(&self) -> usize {
        self.items.len()
    }

    /// Add an item to the list
    pub fn add_item(&mut self, item: ListItem<'a>) {
        self.items.push(item);
    }
}

impl<'a> ListItem<'a> {
    /// Create a new list item
    pub fn new(properties: PropertyList<'a>, label: ListItemLabel<'a>, body: ListItemBody<'a>) -> Self {
        Self {
            properties,
            label,
            body,
        }
    }
}

impl<'a> ListItemLabel<'a> {
    /// Create a new list item label
    pub fn new(properties: PropertyList<'a>) -> Self {
        Self {
            properties,
            blocks: Vec::new(),
        }
    }

    /// Add a block to the label
    pub fn add_block(&mut self, block: super::Block<'a>) {
        self.blocks.push(block);
    }
}

impl<'a> ListItemBody<'a> {
    /// Create a new list item body
    pub fn new(properties: PropertyList<'a>) -> Self {
        Self {
            properties,
            blocks: Vec::new(),
        }
    }

    /// Add a block to the body
    pub fn add_block(&mut self, block: super::Block<'a>) {
        self.blocks.push(block);
    }
}

/// List marker type for common list styles
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListMarkerType {
    /// Bullet (•)
    Disc,

    /// Circle (○)
    Circle,

    /// Square (■)
    Square,

    /// Decimal numbers (1, 2, 3, ...)
    Decimal,

    /// Lower-alpha (a, b, c, ...)
    LowerAlpha,

    /// Upper-alpha (A, B, C, ...)
    UpperAlpha,

    /// Lower-roman (i, ii, iii, ...)
    LowerRoman,

    /// Upper-roman (I, II, III, ...)
    UpperRoman,

    /// No marker
    None,
}

impl ListMarkerType {
    /// Generate marker text for a given index (1-based)
    pub fn marker_text(&self, index: usize) -> String {
        match self {
            Self::Disc => "•".to_string(),
            Self::Circle => "○".to_string(),
            Self::Square => "■".to_string(),
            Self::Decimal => index.to_string(),
            Self::LowerAlpha => Self::to_alpha(index, false),
            Self::UpperAlpha => Self::to_alpha(index, true),
            Self::LowerRoman => Self::to_roman(index, false),
            Self::UpperRoman => Self::to_roman(index, true),
            Self::None => String::new(),
        }
    }

    /// Convert number to alphabetic (a-z, aa-zz, ...)
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
            return n.to_string();  // Fallback to decimal
        }

        let values = [1000, 900, 500, 400, 100, 90, 50, 40, 10, 9, 5, 4, 1];
        let symbols_lower = ["m", "cm", "d", "cd", "c", "xc", "l", "xl", "x", "ix", "v", "iv", "i"];
        let symbols_upper = ["M", "CM", "D", "CD", "C", "XC", "L", "XL", "X", "IX", "V", "IV", "I"];

        let symbols = if uppercase { symbols_upper } else { symbols_lower };

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_block_creation() {
        let props = PropertyList::new();
        let list = ListBlock::new(props);

        assert_eq!(list.item_count(), 0);
    }

    #[test]
    fn test_add_list_item() {
        let props = PropertyList::new();
        let mut list = ListBlock::new(props);

        let label = ListItemLabel::new(PropertyList::new());
        let body = ListItemBody::new(PropertyList::new());
        let item = ListItem::new(PropertyList::new(), label, body);

        list.add_item(item);

        assert_eq!(list.item_count(), 1);
    }

    #[test]
    fn test_list_marker_disc() {
        assert_eq!(ListMarkerType::Disc.marker_text(1), "•");
        assert_eq!(ListMarkerType::Disc.marker_text(5), "•");
    }

    #[test]
    fn test_list_marker_decimal() {
        assert_eq!(ListMarkerType::Decimal.marker_text(1), "1");
        assert_eq!(ListMarkerType::Decimal.marker_text(42), "42");
    }

    #[test]
    fn test_list_marker_lower_alpha() {
        assert_eq!(ListMarkerType::LowerAlpha.marker_text(1), "a");
        assert_eq!(ListMarkerType::LowerAlpha.marker_text(2), "b");
        assert_eq!(ListMarkerType::LowerAlpha.marker_text(26), "z");
        assert_eq!(ListMarkerType::LowerAlpha.marker_text(27), "aa");
    }

    #[test]
    fn test_list_marker_upper_alpha() {
        assert_eq!(ListMarkerType::UpperAlpha.marker_text(1), "A");
        assert_eq!(ListMarkerType::UpperAlpha.marker_text(26), "Z");
    }

    #[test]
    fn test_list_marker_lower_roman() {
        assert_eq!(ListMarkerType::LowerRoman.marker_text(1), "i");
        assert_eq!(ListMarkerType::LowerRoman.marker_text(4), "iv");
        assert_eq!(ListMarkerType::LowerRoman.marker_text(9), "ix");
        assert_eq!(ListMarkerType::LowerRoman.marker_text(2023), "mmxxiii");
    }

    #[test]
    fn test_list_marker_upper_roman() {
        assert_eq!(ListMarkerType::UpperRoman.marker_text(1), "I");
        assert_eq!(ListMarkerType::UpperRoman.marker_text(4), "IV");
        assert_eq!(ListMarkerType::UpperRoman.marker_text(1994), "MCMXCIV");
    }

    #[test]
    fn test_list_marker_none() {
        assert_eq!(ListMarkerType::None.marker_text(1), "");
    }
}
