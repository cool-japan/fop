//! Border shorthand property expansion
//!
//! Handles `border`, `border-top`, `border-right`, `border-bottom`, `border-left`,
//! `border-width`, `border-color`, `border-style`.

use crate::properties::{PropertyId, PropertyList, PropertyValue};
use crate::Result;

/// Expand a border shorthand property (border, border-top, etc.)
///
/// Format: width style color (all optional, order-independent)
/// Example: "1pt solid red" or "red 2pt" or "dashed"
pub(super) fn expand_border_shorthand(
    properties: &mut PropertyList,
    value: &PropertyValue,
    width_id: PropertyId,
    style_id: PropertyId,
    color_id: PropertyId,
) -> Result<()> {
    // For a single non-list value, set all properties to that value
    match value {
        PropertyValue::List(values) => {
            // Parse list of values and categorize them
            for val in values {
                match val {
                    PropertyValue::Length(_) => {
                        properties.set(width_id, val.clone());
                    }
                    PropertyValue::Color(_) => {
                        properties.set(color_id, val.clone());
                    }
                    PropertyValue::Enum(_) | PropertyValue::String(_) => {
                        // Assume it's a style
                        properties.set(style_id, val.clone());
                    }
                    _ => {}
                }
            }
        }
        PropertyValue::Length(_) => {
            properties.set(width_id, value.clone());
        }
        PropertyValue::Color(_) => {
            properties.set(color_id, value.clone());
        }
        _ => {
            // Default: set width, style, and color to the same value
            properties.set(width_id, value.clone());
            properties.set(style_id, value.clone());
            properties.set(color_id, value.clone());
        }
    }

    Ok(())
}
