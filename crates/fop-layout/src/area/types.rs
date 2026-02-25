//! Area types and traits
//!
//! Defines the different types of areas and their associated traits.

use crate::layout::{properties::OverflowBehavior, TextAlign};
use fop_types::{Color, Length, Rect};

/// Content stored in an area
#[derive(Debug, Clone)]
pub enum AreaContent {
    /// Text content
    Text(String),

    /// Binary image data (raw bytes)
    ImageData(Vec<u8>),
}

/// An area represents a rectangular region on a page
#[derive(Debug, Clone)]
pub struct Area {
    /// Area type
    pub area_type: AreaType,

    /// Position and size
    pub geometry: Rect,

    /// Rendering traits (color, background, borders, etc.)
    pub traits: TraitSet,

    /// Content (text or image data)
    pub content: Option<AreaContent>,

    /// Keep constraints for page breaking
    pub keep_constraint: Option<crate::layout::KeepConstraint>,

    /// Break-before property value
    pub break_before: Option<crate::layout::BreakValue>,

    /// Break-after property value
    pub break_after: Option<crate::layout::BreakValue>,

    /// Widows constraint - minimum lines at top of page after break
    pub widows: i32,

    /// Orphans constraint - minimum lines at bottom of page before break
    pub orphans: i32,
}

impl Area {
    /// Create a new area
    pub fn new(area_type: AreaType, geometry: Rect) -> Self {
        Self {
            area_type,
            geometry,
            traits: TraitSet::default(),
            content: None,
            keep_constraint: None,
            break_before: None,
            break_after: None,
            widows: 2,
            orphans: 2,
        }
    }

    /// Create a text area
    pub fn text(geometry: Rect, content: String) -> Self {
        Self {
            area_type: AreaType::Text,
            geometry,
            traits: TraitSet::default(),
            content: Some(AreaContent::Text(content)),
            keep_constraint: None,
            break_before: None,
            break_after: None,
            widows: 2,
            orphans: 2,
        }
    }

    /// Create a viewport area for an image
    pub fn viewport_with_image(geometry: Rect, image_data: Vec<u8>) -> Self {
        Self {
            area_type: AreaType::Viewport,
            geometry,
            traits: TraitSet::default(),
            content: Some(AreaContent::ImageData(image_data)),
            keep_constraint: None,
            break_before: None,
            break_after: None,
            widows: 2,
            orphans: 2,
        }
    }

    /// Set the area's traits
    pub fn with_traits(mut self, traits: TraitSet) -> Self {
        self.traits = traits;
        self
    }

    /// Set the area's keep constraint
    pub fn with_keep_constraint(mut self, keep_constraint: crate::layout::KeepConstraint) -> Self {
        self.keep_constraint = Some(keep_constraint);
        self
    }

    /// Set the area's break-before value
    pub fn with_break_before(mut self, break_before: crate::layout::BreakValue) -> Self {
        self.break_before = Some(break_before);
        self
    }

    /// Set the area's break-after value
    pub fn with_break_after(mut self, break_after: crate::layout::BreakValue) -> Self {
        self.break_after = Some(break_after);
        self
    }

    /// Set the area's widows constraint
    pub fn with_widows(mut self, widows: i32) -> Self {
        self.widows = widows;
        self
    }

    /// Set the area's orphans constraint
    pub fn with_orphans(mut self, orphans: i32) -> Self {
        self.orphans = orphans;
        self
    }

    /// Check if this area contains text
    pub fn has_text(&self) -> bool {
        matches!(self.content, Some(AreaContent::Text(_)))
    }

    /// Check if this area contains image data
    pub fn has_image_data(&self) -> bool {
        matches!(self.content, Some(AreaContent::ImageData(_)))
    }

    /// Get text content if this is a text area
    pub fn text_content(&self) -> Option<&str> {
        match &self.content {
            Some(AreaContent::Text(s)) => Some(s),
            _ => None,
        }
    }

    /// Get image data if this is an image area
    pub fn image_data(&self) -> Option<&[u8]> {
        match &self.content {
            Some(AreaContent::ImageData(data)) => Some(data),
            _ => None,
        }
    }

    /// Get the area's width
    pub fn width(&self) -> Length {
        self.geometry.width
    }

    /// Get the area's height
    pub fn height(&self) -> Length {
        self.geometry.height
    }
}

/// Types of areas in the area tree
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AreaType {
    /// Page area - represents a physical page
    Page,

    /// Region area - represents a page region (body, before, after, start, end)
    Region,

    /// Header area - static content at top of page
    Header,

    /// Footer area - static content at bottom of page
    Footer,

    /// Block area - block-level formatting context
    Block,

    /// Line area - contains inline areas
    Line,

    /// Inline area - inline-level content
    Inline,

    /// Text area - actual text content
    Text,

    /// Space area - whitespace
    Space,

    /// Viewport area - for images, SVG, etc.
    Viewport,

    /// Footnote area - footnote content
    Footnote,

    /// Footnote separator area - line above footnotes
    FootnoteSeparator,

    /// Column area - represents a column in multi-column layout
    Column,

    /// Float area - floating element (like CSS floats)
    FloatArea,

    /// Sidebar start area - static content on the start (left) side of the page
    SidebarStart,

    /// Sidebar end area - static content on the end (right) side of the page
    SidebarEnd,
}

/// Rendering traits for an area
///
/// These are the properties that affect rendering (color, background, borders, etc.)
#[derive(Debug, Clone, Default)]
pub struct TraitSet {
    /// Text color
    pub color: Option<Color>,

    /// Background color
    pub background_color: Option<Color>,

    /// Font family
    pub font_family: Option<String>,

    /// Font size
    pub font_size: Option<Length>,

    /// Font weight (100-900, 400 = normal, 700 = bold)
    pub font_weight: Option<u16>,

    /// Font style (normal, italic, oblique)
    pub font_style: Option<FontStyle>,

    /// Text decoration
    pub text_decoration: Option<TextDecoration>,

    /// Border widths (top, right, bottom, left)
    pub border_width: Option<[Length; 4]>,

    /// Border colors (top, right, bottom, left)
    pub border_color: Option<[Color; 4]>,

    /// Border styles (top, right, bottom, left)
    pub border_style: Option<[BorderStyle; 4]>,

    /// Padding (top, right, bottom, left)
    pub padding: Option<[Length; 4]>,

    /// Text alignment
    pub text_align: Option<TextAlign>,

    /// Link destination (for hyperlinks)
    pub link_destination: Option<String>,

    /// Leader pattern (dots, rule, space, use-content)
    pub is_leader: Option<String>,

    /// Rule thickness (for rule leaders)
    pub rule_thickness: Option<Length>,

    /// Rule style (for rule leaders: solid, dashed, dotted)
    pub rule_style: Option<String>,

    /// Line height
    pub line_height: Option<Length>,

    /// Letter spacing (extra space between characters)
    pub letter_spacing: Option<Length>,

    /// Word spacing (extra space between words)
    pub word_spacing: Option<Length>,

    /// Border radius for rounded corners (top-left, top-right, bottom-right, bottom-left)
    /// Each corner can have independent radius values
    pub border_radius: Option<[Length; 4]>,

    /// Overflow behavior - controls clipping of content
    pub overflow: Option<OverflowBehavior>,

    /// Opacity - transparency level (0.0 = transparent, 1.0 = opaque)
    pub opacity: Option<f64>,

    /// Text transformation
    pub text_transform: Option<TextTransform>,

    /// Font variant
    pub font_variant: Option<FontVariant>,

    /// Display alignment (vertical alignment)
    pub display_align: Option<DisplayAlign>,

    /// Baseline shift for inline positioning (positive = up, negative = down, as fraction of font-size)
    pub baseline_shift: Option<f64>,

    /// Whether hyphenation is enabled
    pub hyphenate: Option<bool>,

    /// Minimum word length before hyphenation (hyphenation-minimum-word-count)
    pub hyphenation_min_word_chars: Option<u32>,

    /// Characters before hyphen (hyphenation-push-character-count)
    pub hyphenation_push_chars: Option<u32>,

    /// Characters after hyphen (hyphenation-remain-character-count)
    pub hyphenation_remain_chars: Option<u32>,

    /// Font stretch (condensed/expanded)
    pub font_stretch: Option<FontStretch>,

    /// Text alignment for last line (used with justify)
    pub text_align_last: Option<TextAlign>,

    /// Change bar color (for margin rule rendering)
    pub change_bar_color: Option<fop_types::Color>,

    /// Span property (none or all columns)
    pub span: Span,

    /// Role attribute for accessibility tagging (PDF/UA)
    pub role: Option<String>,

    /// Language attribute (xml:lang) for language tagging
    pub xml_lang: Option<String>,

    /// Writing mode (lr-tb, rl-tb, tb-rl, tb-lr)
    pub writing_mode: WritingMode,

    /// Text direction (ltr, rtl)
    pub direction: Direction,
}

/// Writing mode values for XSL-FO
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WritingMode {
    /// Left-to-right, top-to-bottom (default Western text)
    #[default]
    LrTb,
    /// Right-to-left, top-to-bottom (Arabic/Hebrew)
    RlTb,
    /// Top-to-bottom, right-to-left (Traditional CJK vertical)
    TbRl,
    /// Top-to-bottom, left-to-right (Modern CJK vertical)
    TbLr,
}

/// Text direction values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Direction {
    /// Left-to-right (default)
    #[default]
    Ltr,
    /// Right-to-left (Arabic/Hebrew)
    Rtl,
}

/// Span values for fo:block column spanning
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Span {
    /// Block stays in current column (default)
    #[default]
    None,
    /// Block spans all columns
    All,
}

/// Font style values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

/// Border style values (XSL-FO and CSS compatible)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderStyle {
    /// No border
    None,
    /// Solid line
    Solid,
    /// Dashed line
    Dashed,
    /// Dotted line
    Dotted,
    /// Double line
    Double,
    /// 3D grooved border
    Groove,
    /// 3D ridged border
    Ridge,
    /// 3D inset border
    Inset,
    /// 3D outset border
    Outset,
    /// Hidden border (same as none, but for border conflict resolution)
    Hidden,
}

/// Text decoration values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextDecoration {
    pub underline: bool,
    pub overline: bool,
    pub line_through: bool,
}

impl TextDecoration {
    pub const NONE: Self = Self {
        underline: false,
        overline: false,
        line_through: false,
    };

    pub const UNDERLINE: Self = Self {
        underline: true,
        overline: false,
        line_through: false,
    };
}

/// Text transformation mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TextTransform {
    #[default]
    None,
    Uppercase,
    Lowercase,
    Capitalize,
}

/// Font variant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontVariant {
    #[default]
    Normal,
    SmallCaps,
}

/// Font stretch values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FontStretch {
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    #[default]
    Normal,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

/// Display alignment (vertical alignment within a block area)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DisplayAlign {
    #[default]
    Before,
    Center,
    After,
}

#[cfg(test)]
mod tests {
    use super::*;
    use fop_types::{Length, Point, Size};

    #[test]
    fn test_area_creation() {
        let rect = Rect::from_point_size(
            Point::new(Length::from_pt(10.0), Length::from_pt(20.0)),
            Size::new(Length::from_pt(100.0), Length::from_pt(50.0)),
        );

        let area = Area::new(AreaType::Block, rect);

        assert_eq!(area.area_type, AreaType::Block);
        assert_eq!(area.width(), Length::from_pt(100.0));
        assert_eq!(area.height(), Length::from_pt(50.0));
        assert!(!area.has_text());
    }

    #[test]
    fn test_text_area() {
        let rect = Rect::from_point_size(
            Point::ZERO,
            Size::new(Length::from_pt(50.0), Length::from_pt(12.0)),
        );

        let area = Area::text(rect, "Hello".to_string());

        assert_eq!(area.area_type, AreaType::Text);
        assert!(area.has_text());
        assert_eq!(area.text_content().expect("test: should succeed"), "Hello");
    }

    #[test]
    fn test_traits() {
        let traits = TraitSet {
            color: Some(Color::RED),
            font_size: Some(Length::from_pt(12.0)),
            ..Default::default()
        };

        assert_eq!(traits.color, Some(Color::RED));
        assert_eq!(traits.font_size, Some(Length::from_pt(12.0)));
    }
}

#[cfg(test)]
mod extended_tests {
    use super::*;
    use fop_types::{Length, Point, Rect, Size};

    // ---- Area builder method tests ----

    #[test]
    fn test_area_with_traits() {
        let rect = Rect::from_point_size(
            Point::ZERO,
            Size::new(Length::from_pt(100.0), Length::from_pt(50.0)),
        );
        let traits = TraitSet {
            font_size: Some(Length::from_pt(14.0)),
            ..Default::default()
        };
        let area = Area::new(AreaType::Block, rect).with_traits(traits);
        assert_eq!(area.traits.font_size, Some(Length::from_pt(14.0)));
    }

    #[test]
    fn test_area_widows_orphans_defaults() {
        let rect = Rect::from_point_size(
            Point::ZERO,
            Size::new(Length::from_pt(100.0), Length::from_pt(50.0)),
        );
        let area = Area::new(AreaType::Block, rect);
        assert_eq!(area.widows, 2);
        assert_eq!(area.orphans, 2);
    }

    #[test]
    fn test_area_with_widows_and_orphans() {
        let rect = Rect::from_point_size(
            Point::ZERO,
            Size::new(Length::from_pt(100.0), Length::from_pt(50.0)),
        );
        let area = Area::new(AreaType::Block, rect)
            .with_widows(4)
            .with_orphans(3);
        assert_eq!(area.widows, 4);
        assert_eq!(area.orphans, 3);
    }

    #[test]
    fn test_area_break_before_none_by_default() {
        let rect = Rect::from_point_size(
            Point::ZERO,
            Size::new(Length::from_pt(100.0), Length::from_pt(50.0)),
        );
        let area = Area::new(AreaType::Block, rect);
        assert!(area.break_before.is_none());
        assert!(area.break_after.is_none());
    }

    #[test]
    fn test_area_with_break_before() {
        use crate::layout::BreakValue;
        let rect = Rect::from_point_size(
            Point::ZERO,
            Size::new(Length::from_pt(100.0), Length::from_pt(50.0)),
        );
        let area = Area::new(AreaType::Block, rect).with_break_before(BreakValue::Page);
        assert!(area.break_before.is_some());
    }

    #[test]
    fn test_area_with_break_after() {
        use crate::layout::BreakValue;
        let rect = Rect::from_point_size(
            Point::ZERO,
            Size::new(Length::from_pt(100.0), Length::from_pt(50.0)),
        );
        let area = Area::new(AreaType::Block, rect).with_break_after(BreakValue::Page);
        assert!(area.break_after.is_some());
    }

    #[test]
    fn test_area_viewport_with_image() {
        let rect = Rect::from_point_size(
            Point::ZERO,
            Size::new(Length::from_pt(50.0), Length::from_pt(50.0)),
        );
        let image_data = vec![0u8, 1, 2, 3, 4];
        let area = Area::viewport_with_image(rect, image_data.clone());
        assert_eq!(area.area_type, AreaType::Viewport);
        assert!(area.has_image_data());
        assert_eq!(
            area.image_data().expect("test: should succeed"),
            image_data.as_slice()
        );
    }

    #[test]
    fn test_area_text_content_none_for_non_text() {
        let rect = Rect::from_point_size(
            Point::ZERO,
            Size::new(Length::from_pt(100.0), Length::from_pt(50.0)),
        );
        let area = Area::new(AreaType::Block, rect);
        assert!(area.text_content().is_none());
        assert!(!area.has_text());
        assert!(!area.has_image_data());
    }

    #[test]
    fn test_area_image_data_none_for_text_area() {
        let rect = Rect::from_point_size(
            Point::ZERO,
            Size::new(Length::from_pt(50.0), Length::from_pt(12.0)),
        );
        let area = Area::text(rect, "Test".to_string());
        assert!(area.image_data().is_none());
    }

    // ---- TraitSet field tests ----

    #[test]
    fn test_traitset_default_all_none() {
        let traits = TraitSet::default();
        assert!(traits.color.is_none());
        assert!(traits.background_color.is_none());
        assert!(traits.font_family.is_none());
        assert!(traits.font_size.is_none());
        assert!(traits.font_weight.is_none());
        assert!(traits.font_style.is_none());
        assert!(traits.text_decoration.is_none());
        assert!(traits.border_width.is_none());
        assert!(traits.padding.is_none());
        assert!(traits.text_align.is_none());
        assert!(traits.line_height.is_none());
        assert!(traits.letter_spacing.is_none());
        assert!(traits.word_spacing.is_none());
    }

    #[test]
    fn test_traitset_writing_mode_default_lr_tb() {
        let traits = TraitSet::default();
        assert_eq!(traits.writing_mode, WritingMode::LrTb);
    }

    #[test]
    fn test_traitset_direction_default_ltr() {
        let traits = TraitSet::default();
        assert_eq!(traits.direction, Direction::Ltr);
    }

    #[test]
    fn test_traitset_span_default_none() {
        let traits = TraitSet::default();
        assert_eq!(traits.span, Span::None);
    }

    #[test]
    fn test_traitset_display_align_default_before() {
        let traits = TraitSet::default();
        // display_align is Option<DisplayAlign>, defaults to None
        assert_eq!(traits.display_align, None);
    }

    #[test]
    fn test_traitset_font_variant_default_normal() {
        let traits = TraitSet::default();
        // font_variant is Option<FontVariant>, defaults to None
        assert_eq!(traits.font_variant, None);
    }

    #[test]
    fn test_traitset_text_transform_default_none() {
        let traits = TraitSet::default();
        // text_transform is Option<TextTransform>, defaults to None
        assert_eq!(traits.text_transform, None);
    }

    #[test]
    fn test_traitset_font_stretch_default_normal() {
        let traits = TraitSet::default();
        // font_stretch is Option<FontStretch>, defaults to None
        assert_eq!(traits.font_stretch, None);
    }

    // ---- AreaType coverage tests ----

    #[test]
    fn test_area_types_are_distinct() {
        assert_ne!(AreaType::Page, AreaType::Region);
        assert_ne!(AreaType::Block, AreaType::Line);
        assert_ne!(AreaType::Inline, AreaType::Text);
        assert_ne!(AreaType::Header, AreaType::Footer);
        assert_ne!(AreaType::Column, AreaType::Footnote);
    }

    #[test]
    fn test_area_width_and_height() {
        let rect = Rect::from_point_size(
            Point::new(Length::from_pt(5.0), Length::from_pt(10.0)),
            Size::new(Length::from_pt(200.0), Length::from_pt(100.0)),
        );
        let area = Area::new(AreaType::Page, rect);
        assert_eq!(area.width(), Length::from_pt(200.0));
        assert_eq!(area.height(), Length::from_pt(100.0));
    }

    // ---- TextDecoration tests ----

    #[test]
    fn test_text_decoration_none() {
        let td = TextDecoration::NONE;
        assert!(!td.underline);
        assert!(!td.overline);
        assert!(!td.line_through);
    }

    #[test]
    fn test_text_decoration_underline() {
        let td = TextDecoration::UNDERLINE;
        assert!(td.underline);
        assert!(!td.overline);
        assert!(!td.line_through);
    }

    #[test]
    fn test_text_decoration_custom() {
        let td = TextDecoration {
            underline: false,
            overline: true,
            line_through: true,
        };
        assert!(!td.underline);
        assert!(td.overline);
        assert!(td.line_through);
    }

    // ---- FontStyle / BorderStyle enum tests ----

    #[test]
    fn test_font_style_variants() {
        assert_ne!(FontStyle::Normal, FontStyle::Italic);
        assert_ne!(FontStyle::Italic, FontStyle::Oblique);
        assert_ne!(FontStyle::Normal, FontStyle::Oblique);
    }

    #[test]
    fn test_border_style_variants() {
        assert_ne!(BorderStyle::None, BorderStyle::Solid);
        assert_ne!(BorderStyle::Dashed, BorderStyle::Dotted);
        assert_ne!(BorderStyle::Hidden, BorderStyle::None);
    }
}
