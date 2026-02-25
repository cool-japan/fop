//! Property parsing for the FO tree builder
//!
//! Provides methods for parsing property key/value pairs from XML attributes
//! into typed `PropertyValue` instances, including length, color, gradient,
//! and expression parsing.

use crate::properties::{PropertyId, PropertyList, PropertyValue, RelativeFontSize};
use crate::{FopError, Result};
use fop_types::{Length, Percentage};
use std::borrow::Cow;

/// Parse a property from an attribute key/value pair into a `PropertyList`.
///
/// Non-property structural attributes (id, master-name, etc.) are silently skipped.
pub(super) fn parse_property<'a>(
    properties: &mut PropertyList<'a>,
    key: &str,
    value: &str,
) -> Result<()> {
    // Skip non-property attributes
    if matches!(
        key,
        "id" | "master-name"
            | "master-reference"
            | "maximum-repeats"
            | "page-position"
            | "odd-or-even"
            | "blank-or-not-blank"
            | "flow-name"
            | "src"
            | "internal-destination"
            | "external-destination"
            | "ref-id"
            | "marker-class-name"
            | "retrieve-class-name"
            | "retrieve-position"
    ) {
        return Ok(());
    }

    // Look up property ID
    if let Some(prop_id) = PropertyId::from_name(key) {
        let prop_value = parse_property_value(prop_id, value)?;

        // Try to expand shorthand properties
        use crate::properties::ShorthandExpander;
        let expanded = ShorthandExpander::expand(properties, key, &prop_value)?;

        // If not expanded, set the property directly
        if !expanded {
            properties.set(prop_id, prop_value);
        }
    }

    Ok(())
}

/// Parse a property value from string for the given property ID.
///
/// Handles special values (auto, none, inherit), relative font sizes, lists,
/// and delegates single-value parsing to `parse_single_value`.
pub(super) fn parse_property_value(prop_id: PropertyId, value: &str) -> Result<PropertyValue> {
    // Try parsing as special values first
    match value {
        "auto" => return Ok(PropertyValue::Auto),
        "none" => return Ok(PropertyValue::None),
        "inherit" => return Ok(PropertyValue::Inherit),
        // XSL-FO property functions: treat as inherit (use parent value)
        "from-parent()" | "from-nearest-specified-value()" => {
            return Ok(PropertyValue::Inherit);
        }
        _ => {}
    }

    // For font-size, try relative/absolute keyword values first
    if prop_id == PropertyId::FontSize {
        if let Some(rfs) = RelativeFontSize::parse(value) {
            return Ok(PropertyValue::RelativeFontSize(rfs));
        }
    }

    // Check if value contains spaces (potential list of values)
    let trimmed = value.trim();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    if parts.len() > 1 {
        // Parse as list of values
        let mut values = Vec::new();
        for part in parts {
            values.push(parse_single_value(part)?);
        }
        return Ok(PropertyValue::List(values));
    }

    // Single value
    parse_single_value(trimmed)
}

/// Parse a single (non-list) value from a string.
///
/// Attempts parsing in priority order: special keywords, proportional-column-width,
/// list functions, percentage, em, length, gradient, color, integer, float, calc,
/// then falls back to string.
pub(super) fn parse_single_value(value: &str) -> Result<PropertyValue> {
    // Try parsing as special values first
    match value {
        "auto" => return Ok(PropertyValue::Auto),
        "none" => return Ok(PropertyValue::None),
        "inherit" => return Ok(PropertyValue::Inherit),
        _ => {}
    }

    // proportional-column-width(N) for table column widths (XSL-FO Section 9.3.2)
    if value.starts_with("proportional-column-width(") && value.ends_with(')') {
        let inner = &value["proportional-column-width(".len()..value.len() - 1];
        if let Ok(ratio) = inner.trim().parse::<f64>() {
            return Ok(PropertyValue::Number(ratio));
        }
    }

    // label-end() and body-start() - list positioning functions
    // Return as string; proper calculation happens in layout.
    // Also handle compound expressions like "body-start() + 5mm".
    if value == "label-end()" || value == "body-start()" {
        return Ok(PropertyValue::String(Cow::Owned(value.to_string())));
    }
    // Compound: "body-start() + <length>" or "label-end() - <length>" etc.
    for func in &["label-end()", "body-start()"] {
        if let Some(stripped) = value.strip_prefix(func) {
            let rest = stripped.trim();
            if rest.is_empty() {
                return Ok(PropertyValue::String(Cow::Owned(value.to_string())));
            }
            // Store as string for layout-time resolution
            return Ok(PropertyValue::String(Cow::Owned(value.to_string())));
        }
    }

    // Try parsing as percentage (must be before length, as both could have numeric part)
    if value.ends_with('%') {
        if let Ok(pct) = value.parse::<Percentage>() {
            return Ok(PropertyValue::Percentage(pct));
        }
    }

    // Try parsing as em unit (relative to parent font-size: 1em = 100%)
    if let Some(em_str) = value.strip_suffix("em") {
        if let Ok(em_val) = em_str.parse::<f64>() {
            // Store em as Percentage: 1em = 100%, 0.7em = 70%
            return Ok(PropertyValue::Percentage(Percentage::new(em_val)));
        }
    }

    // Try parsing as length
    if let Some(len) = parse_length(value) {
        return Ok(PropertyValue::Length(len));
    }

    // Try parsing as gradient (must be before color, as gradients contain colors)
    if value.contains("gradient(") {
        if let Some(gradient) = parse_gradient(value) {
            return Ok(PropertyValue::Gradient(gradient));
        }
    }

    // Try parsing as color
    if let Some(color) = parse_color(value) {
        return Ok(PropertyValue::Color(color));
    }

    // Try parsing as integer
    if let Ok(int_val) = value.parse::<i32>() {
        return Ok(PropertyValue::Integer(int_val));
    }

    // Try parsing as float (e.g. line-height="1.5", opacity="0.8")
    if let Ok(num_val) = value.parse::<f64>() {
        return Ok(PropertyValue::Number(num_val));
    }

    // Try parsing as calc() expression
    if value.starts_with("calc(") {
        if let Ok(expr) = fop_types::Expression::parse(value) {
            return Ok(PropertyValue::Expression(expr));
        }
    }

    // Default to string
    Ok(PropertyValue::String(Cow::Owned(value.to_string())))
}

/// Parse a length value from a string with CSS/XSL-FO unit suffixes.
///
/// Supported units: pt, px, mm, cm, in, em, ex
pub(super) fn parse_length(value: &str) -> Option<Length> {
    if let Some(pt_str) = value.strip_suffix("pt") {
        pt_str.parse::<f64>().ok().map(Length::from_pt)
    } else if let Some(px_str) = value.strip_suffix("px") {
        // 1px = 0.75pt
        px_str
            .parse::<f64>()
            .ok()
            .map(|px| Length::from_pt(px * 0.75))
    } else if let Some(mm_str) = value.strip_suffix("mm") {
        mm_str.parse::<f64>().ok().map(Length::from_mm)
    } else if let Some(cm_str) = value.strip_suffix("cm") {
        cm_str.parse::<f64>().ok().map(Length::from_cm)
    } else if let Some(in_str) = value.strip_suffix("in") {
        in_str.parse::<f64>().ok().map(Length::from_inch)
    } else if let Some(em_str) = value.strip_suffix("em") {
        // em is relative to parent font-size; treat 1em = 12pt as default approximation
        // A proper implementation would resolve this at layout time
        em_str
            .parse::<f64>()
            .ok()
            .map(|em| Length::from_pt(em * 12.0))
    } else if let Some(ex_str) = value.strip_suffix("ex") {
        // ex is ~0.5em, treat 1ex = 6pt as approximation
        ex_str
            .parse::<f64>()
            .ok()
            .map(|ex| Length::from_pt(ex * 6.0))
    } else {
        None
    }
}

/// Parse a color value from a string.
///
/// Supports hex colors (#RGB, #RRGGBB) and named CSS color keywords.
pub(super) fn parse_color(value: &str) -> Option<crate::Color> {
    use crate::Color;

    // Try hex
    if value.starts_with('#') {
        return Color::from_hex(value);
    }

    // Try named colors (basic CSS color keywords)
    match value.to_lowercase().as_str() {
        "black" => Some(Color::BLACK),
        "white" => Some(Color::WHITE),
        "red" => Some(Color::RED),
        "green" => Some(Color::GREEN),
        "blue" => Some(Color::BLUE),
        "yellow" => Some(Color::rgb(255, 255, 0)),
        "cyan" => Some(Color::rgb(0, 255, 255)),
        "magenta" => Some(Color::rgb(255, 0, 255)),
        "gray" | "grey" => Some(Color::rgb(128, 128, 128)),
        "silver" => Some(Color::rgb(192, 192, 192)),
        "maroon" => Some(Color::rgb(128, 0, 0)),
        "olive" => Some(Color::rgb(128, 128, 0)),
        "lime" => Some(Color::rgb(0, 255, 0)),
        "aqua" => Some(Color::rgb(0, 255, 255)),
        "teal" => Some(Color::rgb(0, 128, 128)),
        "navy" => Some(Color::rgb(0, 0, 128)),
        "fuchsia" => Some(Color::rgb(255, 0, 255)),
        "purple" => Some(Color::rgb(128, 0, 128)),
        "orange" => Some(Color::rgb(255, 165, 0)),
        "transparent" => Some(Color::TRANSPARENT),
        _ => None,
    }
}

/// Parse a gradient value from a string.
///
/// Supports:
/// - linear-gradient(angle, color1, color2, ...)
/// - linear-gradient(to direction, color1, color2, ...)
/// - radial-gradient(color1, color2, ...)
/// - radial-gradient(circle, color1, color2, ...)
pub(super) fn parse_gradient(value: &str) -> Option<fop_types::Gradient> {
    use fop_types::{ColorStop, Gradient, Point};

    let value = value.trim();

    if let Some(content) = value.strip_prefix("linear-gradient(") {
        let content = content.strip_suffix(')')?;
        let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();

        if parts.is_empty() {
            return None;
        }

        // Parse angle or direction
        let (angle, color_start_idx) = if parts[0].ends_with("deg") {
            // Parse angle in degrees
            let angle_str = parts[0].strip_suffix("deg")?.trim();
            let angle = angle_str.parse::<f64>().ok()?;
            (angle, 1)
        } else if parts[0].starts_with("to ") {
            // Parse direction (to top, to right, to bottom, to left)
            let direction = parts[0].strip_prefix("to ")?.trim();
            let angle = match direction {
                "top" => 0.0,
                "right" => 90.0,
                "bottom" => 180.0,
                "left" => 270.0,
                _ => return None,
            };
            (angle, 1)
        } else {
            // No angle specified, default to 180deg (top to bottom)
            (180.0, 0)
        };

        // Parse color stops
        let mut color_stops = Vec::new();
        let color_parts = &parts[color_start_idx..];

        if color_parts.is_empty() {
            return None;
        }

        for (i, color_str) in color_parts.iter().enumerate() {
            let color = parse_color(color_str)?;
            let offset = if color_parts.len() == 1 {
                0.5
            } else {
                i as f64 / (color_parts.len() - 1) as f64
            };
            color_stops.push(ColorStop::new(offset, color));
        }

        Some(Gradient::linear_from_angle(angle, color_stops))
    } else if let Some(content) = value.strip_prefix("radial-gradient(") {
        let content = content.strip_suffix(')')?;
        let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();

        if parts.is_empty() {
            return None;
        }

        // Check if first part is "circle" (shape specifier)
        let color_start_idx = if parts[0] == "circle" { 1 } else { 0 };

        // Parse color stops
        let mut color_stops = Vec::new();
        let color_parts = &parts[color_start_idx..];

        if color_parts.is_empty() {
            return None;
        }

        for (i, color_str) in color_parts.iter().enumerate() {
            let color = parse_color(color_str)?;
            let offset = if color_parts.len() == 1 {
                0.5
            } else {
                i as f64 / (color_parts.len() - 1) as f64
            };
            color_stops.push(ColorStop::new(offset, color));
        }

        // Center at 50%, 50% with radius 0.5 (normalized)
        let center = Point::new(Length::from_pt(50.0), Length::from_pt(50.0));
        Some(Gradient::radial(center, 0.5, color_stops))
    } else {
        None
    }
}

// Suppress the dead_code warning for the error type that may be unused in some code paths
#[allow(dead_code)]
fn _use_fop_error(_: FopError) {}
