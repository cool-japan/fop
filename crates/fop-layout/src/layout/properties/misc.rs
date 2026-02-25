//! Miscellaneous property extraction: overflow, border-radius, column layout, opacity, clear

use fop_core::{PropertyId, PropertyList};
use fop_types::Length;

pub use super::types::OverflowBehavior;

/// Extract overflow property from properties
///
/// Returns the overflow behavior for a container. This controls whether
/// content that exceeds the container bounds should be clipped.
///
/// # Examples
///
/// ```
/// use fop_core::{PropertyList, PropertyId, PropertyValue};
/// use fop_layout::layout::extract_overflow;
/// use std::borrow::Cow;
///
/// let mut props = PropertyList::new();
/// props.set(PropertyId::Overflow, PropertyValue::String(Cow::Borrowed("hidden")));
///
/// let overflow = extract_overflow(&props);
/// assert!(overflow.clips_content());
/// ```
pub fn extract_overflow(properties: &PropertyList) -> OverflowBehavior {
    properties
        .get(PropertyId::Overflow)
        .ok()
        .map(|v| OverflowBehavior::from_property_value(&v))
        .unwrap_or(OverflowBehavior::Visible)
}

/// Extract border-radius from properties
///
/// Border radius controls the curvature of corners. Returns an array of
/// [top-left, top-right, bottom-right, bottom-left] radii.
/// Returns None if no border radius is specified.
pub fn extract_border_radius(properties: &PropertyList) -> Option<[Length; 4]> {
    // Check for shorthand x-border-radius property first
    if let Ok(value) = properties.get(PropertyId::XBorderRadius) {
        if let Some(len) = value.as_length() {
            // Single value applies to all corners
            return Some([len, len, len, len]);
        }
    }

    // Extract individual corner radii
    // CSS/FOP uses: top-left, top-right, bottom-right, bottom-left
    let top_left = properties
        .get(PropertyId::XBorderBeforeStartRadius)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);

    let top_right = properties
        .get(PropertyId::XBorderBeforeEndRadius)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);

    let bottom_right = properties
        .get(PropertyId::XBorderAfterEndRadius)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);

    let bottom_left = properties
        .get(PropertyId::XBorderAfterStartRadius)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::ZERO);

    // Only return Some if at least one corner has a non-zero radius
    if top_left > Length::ZERO
        || top_right > Length::ZERO
        || bottom_right > Length::ZERO
        || bottom_left > Length::ZERO
    {
        Some([top_left, top_right, bottom_right, bottom_left])
    } else {
        None
    }
}

/// Extract column-count from properties
///
/// column-count specifies the number of columns for multi-column layout.
/// The default is 1 (single column).
///
/// # Examples
///
/// ```
/// use fop_core::{PropertyList, PropertyId, PropertyValue};
/// use fop_layout::layout::extract_column_count;
///
/// let mut props = PropertyList::new();
/// props.set(PropertyId::ColumnCount, PropertyValue::Integer(2));
///
/// let count = extract_column_count(&props);
/// assert_eq!(count, 2);
/// ```
pub fn extract_column_count(properties: &PropertyList) -> i32 {
    properties
        .get(PropertyId::ColumnCount)
        .ok()
        .and_then(|v| v.as_integer())
        .unwrap_or(1) // Default to single column
        .max(1) // Ensure at least 1 column
}

/// Extract column-gap from properties
///
/// column-gap specifies the space between columns in multi-column layout.
/// The default is 1em (approximately 12pt for typical fonts).
///
/// # Examples
///
/// ```
/// use fop_core::{PropertyList, PropertyId, PropertyValue};
/// use fop_layout::layout::extract_column_gap;
/// use fop_types::Length;
///
/// let mut props = PropertyList::new();
/// props.set(PropertyId::ColumnGap, PropertyValue::Length(Length::from_pt(20.0)));
///
/// let gap = extract_column_gap(&props);
/// assert_eq!(gap, Length::from_pt(20.0));
/// ```
pub fn extract_column_gap(properties: &PropertyList) -> Length {
    properties
        .get(PropertyId::ColumnGap)
        .ok()
        .and_then(|v| v.as_length())
        .unwrap_or(Length::from_pt(12.0)) // Default to 1em (approximately)
}

/// Extract opacity from properties
///
/// opacity specifies the transparency level for rendering.
/// Valid values are 0.0 (fully transparent) to 1.0 (fully opaque).
/// The default is 1.0 (opaque).
///
/// # Examples
///
/// ```
/// use fop_core::{PropertyList, PropertyId, PropertyValue};
/// use fop_layout::layout::extract_opacity;
///
/// let props = PropertyList::new();
/// // Extract opacity returns 1.0 (fully opaque) as default
/// let opacity = extract_opacity(&props);
/// assert_eq!(opacity, 1.0);
/// ```
pub fn extract_opacity(properties: &PropertyList) -> f64 {
    properties
        .get(PropertyId::Opacity)
        .ok()
        .and_then(|v| v.as_number())
        .unwrap_or(1.0) // Default to fully opaque
        .clamp(0.0, 1.0) // Clamp to [0.0, 1.0]
}

/// Extract the `clear` property from a block's PropertyList.
///
/// Returns the XSL-FO clear value that controls float clearing behaviour.
/// Valid values are `left`, `right`, `both`, `start`, `end`, and `none`.
///
/// The enum values used here map to XSL-FO integer codes:
/// EN_NONE = 77, EN_LEFT = 66, EN_RIGHT = 96, EN_BOTH = 19,
/// EN_START = 104, EN_END = 45.
pub fn extract_clear(properties: &PropertyList) -> super::super::engine::ClearSide {
    use super::super::engine::ClearSide;

    if let Ok(prop) = properties.get(PropertyId::Clear) {
        if let Some(enum_val) = prop.as_enum() {
            return match enum_val {
                66 => ClearSide::Left,
                96 => ClearSide::Right,
                19 => ClearSide::Both,
                104 => ClearSide::Start,
                45 => ClearSide::End,
                _ => ClearSide::None,
            };
        }
        if let Some(s) = prop.as_string() {
            return match s {
                "left" => ClearSide::Left,
                "right" => ClearSide::Right,
                "both" => ClearSide::Both,
                "start" => ClearSide::Start,
                "end" => ClearSide::End,
                _ => ClearSide::None,
            };
        }
    }
    ClearSide::None
}
