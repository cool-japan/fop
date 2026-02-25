//! Background shorthand property expansion
//!
//! Handles the `background` shorthand which expands to background-color,
//! background-image, background-repeat, background-attachment, background-position.

use crate::properties::{PropertyId, PropertyList, PropertyValue};
use crate::Result;

/// Expand the background shorthand property
///
/// Format: [color] [image] [repeat] [attachment] [position]
/// All components are optional and order-flexible
/// Example: "#FF0000 url(bg.png) no-repeat fixed center"
pub(super) fn expand_background_shorthand(
    properties: &mut PropertyList,
    value: &PropertyValue,
) -> Result<()> {
    match value {
        PropertyValue::List(values) => {
            // Parse list of values and categorize them
            for val in values {
                match val {
                    PropertyValue::Color(_) => {
                        properties.set(PropertyId::BackgroundColor, val.clone());
                    }
                    PropertyValue::String(s) => {
                        let s_str = s.as_ref();
                        // Check if it's a url() function
                        if s_str.starts_with("url(") && s_str.ends_with(')') {
                            properties.set(PropertyId::BackgroundImage, val.clone());
                        } else if is_background_repeat(s_str) {
                            properties.set(PropertyId::BackgroundRepeat, val.clone());
                        } else if is_background_attachment(s_str) {
                            properties.set(PropertyId::BackgroundAttachment, val.clone());
                        } else if is_background_position(s_str) {
                            properties.set(PropertyId::BackgroundPosition, val.clone());
                        }
                    }
                    PropertyValue::Percentage(_) | PropertyValue::Length(_) => {
                        // Likely a position value
                        properties.set(PropertyId::BackgroundPosition, val.clone());
                    }
                    _ => {}
                }
            }
        }
        PropertyValue::Color(_) => {
            properties.set(PropertyId::BackgroundColor, value.clone());
        }
        PropertyValue::Percentage(_) | PropertyValue::Length(_) => {
            // Position value
            properties.set(PropertyId::BackgroundPosition, value.clone());
        }
        PropertyValue::String(s) => {
            let s_str = s.as_ref();
            if s_str.starts_with("url(") && s_str.ends_with(')') {
                properties.set(PropertyId::BackgroundImage, value.clone());
            } else if is_background_repeat(s_str) {
                properties.set(PropertyId::BackgroundRepeat, value.clone());
            } else if is_background_attachment(s_str) {
                properties.set(PropertyId::BackgroundAttachment, value.clone());
            } else if is_background_position(s_str) {
                properties.set(PropertyId::BackgroundPosition, value.clone());
            }
        }
        _ => {}
    }

    Ok(())
}

/// Check if a string is a valid background-repeat value
pub(super) fn is_background_repeat(s: &str) -> bool {
    matches!(
        s,
        "repeat" | "repeat-x" | "repeat-y" | "no-repeat" | "space" | "round"
    )
}

/// Check if a string is a valid background-attachment value
pub(super) fn is_background_attachment(s: &str) -> bool {
    matches!(s, "scroll" | "fixed" | "local")
}

/// Check if a string is a valid background-position keyword
pub(super) fn is_background_position(s: &str) -> bool {
    matches!(s, "top" | "bottom" | "left" | "right" | "center")
}
