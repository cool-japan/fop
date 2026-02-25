//! Spacing and indent property extraction

use fop_core::{PropertyId, PropertyList};
use fop_types::Length;

/// Extract space-before from properties
pub fn extract_space_before(properties: &PropertyList) -> Length {
    properties
        .get(PropertyId::SpaceBefore)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO)
}

/// Extract space-after from properties
pub fn extract_space_after(properties: &PropertyList) -> Length {
    properties
        .get(PropertyId::SpaceAfter)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO)
}

/// Extract start-indent from properties
///
/// start-indent specifies the left margin (in left-to-right writing mode)
/// for a block-level element.
pub fn extract_start_indent(properties: &PropertyList) -> Length {
    properties
        .get(PropertyId::StartIndent)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO)
}

/// Extract end-indent from properties
///
/// end-indent specifies the right margin (in left-to-right writing mode)
/// for a block-level element.
pub fn extract_end_indent(properties: &PropertyList) -> Length {
    properties
        .get(PropertyId::EndIndent)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO)
}

/// Extract text-indent from properties
///
/// text-indent specifies the indentation of the first line of text
/// in a block-level element.
pub fn extract_text_indent(properties: &PropertyList) -> Length {
    properties
        .get(PropertyId::TextIndent)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO)
}

/// Extract line-height from properties
///
/// Line height controls the height of line boxes. It can be specified as:
/// - A length value (e.g., 14pt)
/// - A number multiplied by the font size (e.g., 1.2)
/// - "normal" which typically means 1.2 times the font size
///
/// Returns None if not specified or set to "normal", allowing default behavior.
pub fn extract_line_height(properties: &PropertyList) -> Option<Length> {
    properties.get(PropertyId::LineHeight).ok().and_then(|v| {
        // Check if it's a length value
        if let Some(len) = v.as_length() {
            Some(len)
        } else if let Some(num) = v.as_number() {
            // Multiplier of font size - get font size and multiply
            let font_size = properties
                .get(PropertyId::FontSize)
                .ok()
                .and_then(|fs| fs.as_length())
                .unwrap_or(Length::from_pt(12.0));
            Some(Length::from_millipoints(
                (font_size.millipoints() as f64 * num) as i32,
            ))
        } else {
            // "normal" or other keyword - use default
            None
        }
    })
}

/// Extract letter-spacing from properties
///
/// Letter spacing controls the spacing between characters.
/// Positive values increase spacing, negative values decrease it.
/// Returns None if not specified or set to "normal".
pub fn extract_letter_spacing(properties: &PropertyList) -> Option<Length> {
    properties
        .get(PropertyId::LetterSpacing)
        .ok()
        .and_then(|v| v.as_length())
}

/// Extract word-spacing from properties
///
/// Word spacing controls the spacing between words (space characters).
/// Positive values increase spacing, negative values decrease it.
/// Returns None if not specified or set to "normal".
pub fn extract_word_spacing(properties: &PropertyList) -> Option<Length> {
    properties
        .get(PropertyId::WordSpacing)
        .ok()
        .and_then(|v| v.as_length())
}
