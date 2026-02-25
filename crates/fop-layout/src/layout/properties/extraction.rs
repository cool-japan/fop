//! Core trait extraction: extract_traits, font size resolution, text measurement

use crate::area::{
    BorderStyle, Direction, DisplayAlign, FontStretch, FontStyle, FontVariant, Span, TextTransform,
    TraitSet, WritingMode,
};
use crate::layout::TextAlign;
use fop_core::{PropertyId, PropertyList};
use fop_types::Length;

use super::misc::{extract_border_radius, extract_opacity, extract_overflow, OverflowBehavior};
use super::spacing::{extract_letter_spacing, extract_line_height, extract_word_spacing};

/// Extract font-size from a property value, resolving relative sizes
///
/// If the value is a relative font size (larger, smaller, or size keywords),
/// it will be resolved using the parent font size from the property list.
/// Returns None if the value cannot be converted to a length.
pub(super) fn extract_font_size(
    properties: &PropertyList,
    value: &fop_core::PropertyValue,
) -> Option<Length> {
    // First check if it's a direct length
    if let Some(len) = value.as_length() {
        return Some(len);
    }

    // Check if it's a relative font size
    if value.as_relative_font_size().is_some() {
        // Get parent font size for resolution
        let parent_font_size = if let Some(parent) = properties.parent() {
            parent
                .get(PropertyId::FontSize)
                .ok()
                .and_then(|v| extract_font_size(parent, &v))
                .unwrap_or(Length::from_pt(12.0)) // Default to 12pt if no parent
        } else {
            Length::from_pt(12.0) // Default to 12pt if no parent
        };

        return value.resolve_font_size(parent_font_size);
    }

    None
}

/// Parse border style from string value
pub(super) fn parse_border_style(s: Option<&str>) -> Option<BorderStyle> {
    match s? {
        "none" => Some(BorderStyle::None),
        "solid" => Some(BorderStyle::Solid),
        "dashed" => Some(BorderStyle::Dashed),
        "dotted" => Some(BorderStyle::Dotted),
        "double" => Some(BorderStyle::Double),
        "groove" => Some(BorderStyle::Groove),
        "ridge" => Some(BorderStyle::Ridge),
        "inset" => Some(BorderStyle::Inset),
        "outset" => Some(BorderStyle::Outset),
        "hidden" => Some(BorderStyle::Hidden),
        _ => None,
    }
}

/// Extract rendering traits from a property list
pub fn extract_traits(properties: &PropertyList) -> TraitSet {
    let mut traits = TraitSet::default();

    // Color
    if let Ok(value) = properties.get(PropertyId::Color) {
        traits.color = value.as_color();
    }

    // Background color
    if let Ok(value) = properties.get(PropertyId::BackgroundColor) {
        traits.background_color = value.as_color();
    }

    // Font family
    if let Ok(value) = properties.get(PropertyId::FontFamily) {
        traits.font_family = value.as_string().map(|s| s.to_string());
    }

    // Font size
    if let Ok(value) = properties.get(PropertyId::FontSize) {
        traits.font_size = extract_font_size(properties, &value);
    }

    // Font weight
    if let Ok(value) = properties.get(PropertyId::FontWeight) {
        if let Some(num) = value.as_integer() {
            traits.font_weight = Some(num as u16);
        }
    }

    // Font style
    if let Ok(value) = properties.get(PropertyId::FontStyle) {
        if let Some(enum_val) = value.as_enum() {
            traits.font_style = match enum_val {
                1 => Some(FontStyle::Italic),
                2 => Some(FontStyle::Oblique),
                _ => Some(FontStyle::Normal),
            };
        }
    }

    // Margins (for spacing calculations - reserved for future use)
    let _margin_top = properties
        .get(PropertyId::MarginTop)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);
    let _margin_right = properties
        .get(PropertyId::MarginRight)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);
    let _margin_bottom = properties
        .get(PropertyId::MarginBottom)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);
    let _margin_left = properties
        .get(PropertyId::MarginLeft)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);

    // Padding
    let padding_top = properties
        .get(PropertyId::PaddingTop)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);
    let padding_right = properties
        .get(PropertyId::PaddingRight)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);
    let padding_bottom = properties
        .get(PropertyId::PaddingBottom)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);
    let padding_left = properties
        .get(PropertyId::PaddingLeft)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);

    traits.padding = Some([padding_top, padding_right, padding_bottom, padding_left]);

    // Border width
    let border_top = properties
        .get(PropertyId::BorderTopWidth)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);
    let border_right = properties
        .get(PropertyId::BorderRightWidth)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);
    let border_bottom = properties
        .get(PropertyId::BorderBottomWidth)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);
    let border_left = properties
        .get(PropertyId::BorderLeftWidth)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);

    traits.border_width = Some([border_top, border_right, border_bottom, border_left]);

    // Border colors
    let border_top_color = properties
        .get(PropertyId::BorderTopColor)
        .ok()
        .and_then(|v| v.as_color())
        .unwrap_or(fop_types::Color::BLACK);
    let border_right_color = properties
        .get(PropertyId::BorderRightColor)
        .ok()
        .and_then(|v| v.as_color())
        .unwrap_or(fop_types::Color::BLACK);
    let border_bottom_color = properties
        .get(PropertyId::BorderBottomColor)
        .ok()
        .and_then(|v| v.as_color())
        .unwrap_or(fop_types::Color::BLACK);
    let border_left_color = properties
        .get(PropertyId::BorderLeftColor)
        .ok()
        .and_then(|v| v.as_color())
        .unwrap_or(fop_types::Color::BLACK);

    traits.border_color = Some([
        border_top_color,
        border_right_color,
        border_bottom_color,
        border_left_color,
    ]);

    // Border styles
    let border_top_style = properties
        .get(PropertyId::BorderTopStyle)
        .ok()
        .and_then(|v| parse_border_style(v.as_string()))
        .unwrap_or(BorderStyle::Solid);
    let border_right_style = properties
        .get(PropertyId::BorderRightStyle)
        .ok()
        .and_then(|v| parse_border_style(v.as_string()))
        .unwrap_or(BorderStyle::Solid);
    let border_bottom_style = properties
        .get(PropertyId::BorderBottomStyle)
        .ok()
        .and_then(|v| parse_border_style(v.as_string()))
        .unwrap_or(BorderStyle::Solid);
    let border_left_style = properties
        .get(PropertyId::BorderLeftStyle)
        .ok()
        .and_then(|v| parse_border_style(v.as_string()))
        .unwrap_or(BorderStyle::Solid);

    traits.border_style = Some([
        border_top_style,
        border_right_style,
        border_bottom_style,
        border_left_style,
    ]);

    // Text alignment
    if let Ok(value) = properties.get(PropertyId::TextAlign) {
        if let Some(s) = value.as_string() {
            traits.text_align = match s {
                "left" => Some(TextAlign::Left),
                "right" => Some(TextAlign::Right),
                "center" => Some(TextAlign::Center),
                "justify" => Some(TextAlign::Justify),
                _ => None,
            };
        }
    }

    // Line height
    traits.line_height = extract_line_height(properties);

    // Letter spacing
    traits.letter_spacing = extract_letter_spacing(properties);

    // Word spacing
    traits.word_spacing = extract_word_spacing(properties);

    // Border radius
    traits.border_radius = extract_border_radius(properties);

    // Overflow
    let overflow = extract_overflow(properties);
    // Only set if not the default (Visible)
    if overflow != OverflowBehavior::Visible {
        traits.overflow = Some(overflow);
    }

    // Opacity
    let opacity = extract_opacity(properties);
    // Only set if not the default (1.0)
    if (opacity - 1.0).abs() > f64::EPSILON {
        traits.opacity = Some(opacity);
    }

    // Text transform
    if let Ok(value) = properties.get(PropertyId::TextTransform) {
        if let Some(s) = value.as_string() {
            traits.text_transform = match s {
                "uppercase" => Some(TextTransform::Uppercase),
                "lowercase" => Some(TextTransform::Lowercase),
                "capitalize" => Some(TextTransform::Capitalize),
                _ => None,
            };
        }
    }

    // Font variant
    if let Ok(value) = properties.get(PropertyId::FontVariant) {
        if let Some(s) = value.as_string() {
            traits.font_variant = match s {
                "small-caps" => Some(FontVariant::SmallCaps),
                _ => None,
            };
        }
    }

    // Display align
    if let Ok(value) = properties.get(PropertyId::DisplayAlign) {
        if let Some(s) = value.as_string() {
            traits.display_align = match s {
                "center" => Some(DisplayAlign::Center),
                "after" => Some(DisplayAlign::After),
                _ => Some(DisplayAlign::Before),
            };
        }
    }

    // Baseline shift (super/sub/length)
    if let Ok(value) = properties.get(PropertyId::BaselineShift) {
        if let Some(s) = value.as_string() {
            traits.baseline_shift = match s {
                "super" => Some(0.5), // 50% of font-size upward
                "sub" => Some(-0.3),  // 30% of font-size downward
                "baseline" | "0" => Some(0.0),
                _ => None,
            };
        } else if let Some(len) = value.as_length() {
            // Store as pt value; will be divided by font-size later
            traits.baseline_shift = Some(len.to_pt() / 12.0); // relative to 12pt default
        }
    }

    // Hyphenation
    if let Ok(value) = properties.get(PropertyId::Hyphenate) {
        if let Some(s) = value.as_string() {
            traits.hyphenate = Some(s == "true");
        } else if let Some(b) = value.as_boolean() {
            traits.hyphenate = Some(b);
        }
    }

    if let Ok(value) = properties.get(PropertyId::HyphenationPushCharacterCount) {
        if let Some(i) = value.as_integer() {
            traits.hyphenation_push_chars = Some(i as u32);
        }
    }

    if let Ok(value) = properties.get(PropertyId::HyphenationRemainCharacterCount) {
        if let Some(i) = value.as_integer() {
            traits.hyphenation_remain_chars = Some(i as u32);
        }
    }

    // Font stretch
    if let Ok(value) = properties.get(PropertyId::FontStretch) {
        if let Some(s) = value.as_string() {
            traits.font_stretch = match s {
                "ultra-condensed" => Some(FontStretch::UltraCondensed),
                "extra-condensed" => Some(FontStretch::ExtraCondensed),
                "condensed" => Some(FontStretch::Condensed),
                "semi-condensed" => Some(FontStretch::SemiCondensed),
                "semi-expanded" => Some(FontStretch::SemiExpanded),
                "expanded" => Some(FontStretch::Expanded),
                "extra-expanded" => Some(FontStretch::ExtraExpanded),
                "ultra-expanded" => Some(FontStretch::UltraExpanded),
                _ => None, // "normal" -> None (default)
            };
        }
    }

    // Text align last
    if let Ok(value) = properties.get(PropertyId::TextAlignLast) {
        if let Some(s) = value.as_string() {
            traits.text_align_last = match s {
                "left" | "start" => Some(TextAlign::Left),
                "right" | "end" => Some(TextAlign::Right),
                "center" => Some(TextAlign::Center),
                "justify" => Some(TextAlign::Justify),
                _ => None,
            };
        }
    }

    // Change bar color
    if let Ok(value) = properties.get(PropertyId::ChangeBarColor) {
        traits.change_bar_color = value.as_color();
    }

    // span property (for multi-column spanning)
    if let Ok(value) = properties.get(PropertyId::Span) {
        if value.as_string() == Some("all") {
            traits.span = Span::All;
        }
    }

    // role attribute for accessibility tagging
    if let Ok(value) = properties.get(PropertyId::Role) {
        traits.role = value.as_string().map(|s| s.to_string());
    }

    // xml:lang for language tagging
    if let Ok(value) = properties.get(PropertyId::XmlLang) {
        traits.xml_lang = value.as_string().map(|s| s.to_string());
    }

    // writing-mode
    if let Ok(value) = properties.get(PropertyId::WritingMode) {
        if let Some(s) = value.as_string() {
            traits.writing_mode = match s {
                "rl-tb" | "rl" => WritingMode::RlTb,
                "tb-rl" | "tb" => WritingMode::TbRl,
                "tb-lr" => WritingMode::TbLr,
                _ => WritingMode::LrTb,
            };
        }
    }

    // direction
    if let Ok(value) = properties.get(PropertyId::Direction) {
        if let Some(s) = value.as_string() {
            traits.direction = match s {
                "rtl" => Direction::Rtl,
                _ => Direction::Ltr,
            };
        }
    }

    traits
}

/// Estimate the pixel width of a text string given font properties.
/// Uses approximate character width metrics — a full implementation would
/// use actual TTF glyph advances via `ttf-parser`.
///
/// Returns estimated width in points.
///
/// # Examples
///
/// ```
/// use fop_layout::measure_text_width;
/// use fop_types::Length;
///
/// let width = measure_text_width("Hello", Length::from_pt(12.0), None);
/// assert!(width.to_pt() > 0.0);
/// ```
pub fn measure_text_width(text: &str, font_size: Length, font_weight: Option<u16>) -> Length {
    if text.is_empty() {
        return Length::ZERO;
    }

    // Average character width approximation:
    // - Normal weight: ~0.5 × font-size per character (for Latin)
    // - Bold: ~0.55 × font-size
    // - CJK characters: ~1.0 × font-size (full-width)
    let weight_factor = match font_weight {
        Some(w) if w >= 600 => 0.55,
        _ => 0.50,
    };

    let mut total_width = 0.0_f64;
    for ch in text.chars() {
        let char_factor = if is_cjk(ch) {
            1.0
        } else if ch == ' ' {
            0.25
        } else if r#"iIl1!|.,;'"#.contains(ch) {
            0.3
        } else if "mMwW".contains(ch) {
            0.7
        } else {
            weight_factor
        };
        total_width += char_factor;
    }

    Length::from_pt(total_width * font_size.to_pt())
}

/// Check if a character is a CJK (Chinese/Japanese/Korean) character
fn is_cjk(ch: char) -> bool {
    matches!(ch,
        '\u{3000}'..='\u{9FFF}'   // CJK Unified Ideographs + common CJK
        | '\u{AC00}'..='\u{D7AF}' // Korean Hangul syllables
        | '\u{F900}'..='\u{FAFF}' // CJK Compatibility Ideographs
        | '\u{FF00}'..='\u{FFEF}' // Halfwidth and Fullwidth Forms
    )
}
