//! Shorthand property expansion
//!
//! Many XSL-FO properties have shorthand forms that expand to multiple properties.
//! For example, `margin="10pt"` expands to margin-top, margin-right, margin-bottom, margin-left.
//!
//! This module is split into submodules by shorthand category:
//! - `four_sides`: Generic four-sided expansion (margin, padding, border-width, etc.)
//! - `border`: Border shorthand expansion (border, border-top, etc.)
//! - `background`: Background shorthand expansion
//! - `font`: Font shorthand expansion

mod background;
mod border;
mod font;
mod four_sides;

#[cfg(test)]
mod tests;

use crate::properties::{PropertyId, PropertyList, PropertyValue};
use crate::Result;

/// Expand shorthand properties into their component properties
pub struct ShorthandExpander;

impl ShorthandExpander {
    /// Expand a property if it's a shorthand
    ///
    /// Returns true if the property was expanded, false otherwise
    pub fn expand(
        properties: &mut PropertyList,
        name: &str,
        value: &PropertyValue,
    ) -> Result<bool> {
        match name {
            "margin" => {
                four_sides::expand_four_sides(
                    properties,
                    value,
                    PropertyId::MarginTop,
                    PropertyId::MarginRight,
                    PropertyId::MarginBottom,
                    PropertyId::MarginLeft,
                )?;
                Ok(true)
            }
            "padding" => {
                four_sides::expand_four_sides(
                    properties,
                    value,
                    PropertyId::PaddingTop,
                    PropertyId::PaddingRight,
                    PropertyId::PaddingBottom,
                    PropertyId::PaddingLeft,
                )?;
                Ok(true)
            }
            "border-width" => {
                four_sides::expand_four_sides(
                    properties,
                    value,
                    PropertyId::BorderTopWidth,
                    PropertyId::BorderRightWidth,
                    PropertyId::BorderBottomWidth,
                    PropertyId::BorderLeftWidth,
                )?;
                Ok(true)
            }
            "border-color" => {
                four_sides::expand_four_sides(
                    properties,
                    value,
                    PropertyId::BorderTopColor,
                    PropertyId::BorderRightColor,
                    PropertyId::BorderBottomColor,
                    PropertyId::BorderLeftColor,
                )?;
                Ok(true)
            }
            "border-style" => {
                four_sides::expand_four_sides(
                    properties,
                    value,
                    PropertyId::BorderTopStyle,
                    PropertyId::BorderRightStyle,
                    PropertyId::BorderBottomStyle,
                    PropertyId::BorderLeftStyle,
                )?;
                Ok(true)
            }
            "border" => {
                // border: width style color
                // Apply to all four sides
                border::expand_border_shorthand(
                    properties,
                    value,
                    PropertyId::BorderTopWidth,
                    PropertyId::BorderTopStyle,
                    PropertyId::BorderTopColor,
                )?;
                border::expand_border_shorthand(
                    properties,
                    value,
                    PropertyId::BorderRightWidth,
                    PropertyId::BorderRightStyle,
                    PropertyId::BorderRightColor,
                )?;
                border::expand_border_shorthand(
                    properties,
                    value,
                    PropertyId::BorderBottomWidth,
                    PropertyId::BorderBottomStyle,
                    PropertyId::BorderBottomColor,
                )?;
                border::expand_border_shorthand(
                    properties,
                    value,
                    PropertyId::BorderLeftWidth,
                    PropertyId::BorderLeftStyle,
                    PropertyId::BorderLeftColor,
                )?;
                Ok(true)
            }
            "border-top" => {
                border::expand_border_shorthand(
                    properties,
                    value,
                    PropertyId::BorderTopWidth,
                    PropertyId::BorderTopStyle,
                    PropertyId::BorderTopColor,
                )?;
                Ok(true)
            }
            "border-right" => {
                border::expand_border_shorthand(
                    properties,
                    value,
                    PropertyId::BorderRightWidth,
                    PropertyId::BorderRightStyle,
                    PropertyId::BorderRightColor,
                )?;
                Ok(true)
            }
            "border-bottom" => {
                border::expand_border_shorthand(
                    properties,
                    value,
                    PropertyId::BorderBottomWidth,
                    PropertyId::BorderBottomStyle,
                    PropertyId::BorderBottomColor,
                )?;
                Ok(true)
            }
            "border-left" => {
                border::expand_border_shorthand(
                    properties,
                    value,
                    PropertyId::BorderLeftWidth,
                    PropertyId::BorderLeftStyle,
                    PropertyId::BorderLeftColor,
                )?;
                Ok(true)
            }
            "font" => {
                font::expand_font_shorthand(properties, value)?;
                Ok(true)
            }
            "background" => {
                background::expand_background_shorthand(properties, value)?;
                Ok(true)
            }
            _ => Ok(false),
        }
    }
}
