//! Four-sided property expansion (margin, padding, border-width, etc.)
//!
//! XSL-FO follows CSS conventions for multi-value shorthand properties.

use crate::properties::{PropertyId, PropertyList, PropertyValue};
use crate::Result;

/// Expand a four-sided property (margin, padding, border-width, etc.)
///
/// XSL-FO follows CSS conventions:
/// - 1 value: all four sides
/// - 2 values: top/bottom, left/right
/// - 3 values: top, left/right, bottom
/// - 4 values: top, right, bottom, left (clockwise)
pub(super) fn expand_four_sides(
    properties: &mut PropertyList,
    value: &PropertyValue,
    top_id: PropertyId,
    right_id: PropertyId,
    bottom_id: PropertyId,
    left_id: PropertyId,
) -> Result<()> {
    match value {
        PropertyValue::List(values) => {
            match values.len() {
                1 => {
                    // All four sides same value
                    let val = values[0].clone();
                    properties.set(top_id, val.clone());
                    properties.set(right_id, val.clone());
                    properties.set(bottom_id, val.clone());
                    properties.set(left_id, val);
                }
                2 => {
                    // Top/bottom, left/right
                    let tb = values[0].clone();
                    let lr = values[1].clone();
                    properties.set(top_id, tb.clone());
                    properties.set(bottom_id, tb);
                    properties.set(right_id, lr.clone());
                    properties.set(left_id, lr);
                }
                3 => {
                    // Top, left/right, bottom
                    let top = values[0].clone();
                    let lr = values[1].clone();
                    let bottom = values[2].clone();
                    properties.set(top_id, top);
                    properties.set(right_id, lr.clone());
                    properties.set(left_id, lr);
                    properties.set(bottom_id, bottom);
                }
                4 => {
                    // Top, right, bottom, left
                    properties.set(top_id, values[0].clone());
                    properties.set(right_id, values[1].clone());
                    properties.set(bottom_id, values[2].clone());
                    properties.set(left_id, values[3].clone());
                }
                _ => {
                    // Invalid number of values, set all to first value
                    if let Some(first) = values.first() {
                        let val = first.clone();
                        properties.set(top_id, val.clone());
                        properties.set(right_id, val.clone());
                        properties.set(bottom_id, val.clone());
                        properties.set(left_id, val);
                    }
                }
            }
        }
        _ => {
            // Single value, apply to all sides
            properties.set(top_id, value.clone());
            properties.set(right_id, value.clone());
            properties.set(bottom_id, value.clone());
            properties.set(left_id, value.clone());
        }
    }

    Ok(())
}
