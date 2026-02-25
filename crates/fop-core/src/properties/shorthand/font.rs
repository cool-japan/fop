//! Font shorthand property expansion
//!
//! Handles the `font` shorthand which expands to font-style, font-variant,
//! font-weight, font-size, line-height, and font-family.

use crate::properties::{PropertyId, PropertyList, PropertyValue};
use crate::Result;
use fop_types::Length;

/// Expand font shorthand property
///
/// CSS syntax: [style] [variant] [weight] size[/line-height] family
/// Example: "italic small-caps bold 12pt/14pt Times, serif"
///
/// Components:
/// - style: optional (normal, italic, oblique)
/// - variant: optional (normal, small-caps)
/// - weight: optional (normal, bold, bolder, lighter, 100-900)
/// - size: required (with units: pt, px, em, %)
/// - line-height: optional (follows size with /)
/// - family: required (comma-separated list)
pub(super) fn expand_font_shorthand(
    properties: &mut PropertyList,
    value: &PropertyValue,
) -> Result<()> {
    // Extract the string representation
    let value_str = match value {
        PropertyValue::String(s) => s.as_ref(),
        PropertyValue::List(values) => {
            // If it's already a list, we need to reconstruct the string
            let parts: Vec<String> = values
                .iter()
                .map(|v| match v {
                    PropertyValue::String(s) => s.to_string(),
                    PropertyValue::Length(l) => format!("{}pt", l.to_pt()),
                    _ => String::new(),
                })
                .collect();
            return parse_font_shorthand(properties, &parts.join(" "));
        }
        _ => return Ok(()),
    };

    parse_font_shorthand(properties, value_str)
}

/// Parse font shorthand string
pub(super) fn parse_font_shorthand(properties: &mut PropertyList, value_str: &str) -> Result<()> {
    let trimmed = value_str.trim();
    if trimmed.is_empty() {
        return Ok(());
    }

    // Handle special keywords
    match trimmed {
        "inherit" => {
            properties.set(PropertyId::FontStyle, PropertyValue::Inherit);
            properties.set(PropertyId::FontVariant, PropertyValue::Inherit);
            properties.set(PropertyId::FontWeight, PropertyValue::Inherit);
            properties.set(PropertyId::FontSize, PropertyValue::Inherit);
            properties.set(PropertyId::LineHeight, PropertyValue::Inherit);
            properties.set(PropertyId::FontFamily, PropertyValue::Inherit);
            return Ok(());
        }
        "initial" => {
            properties.set(
                PropertyId::FontStyle,
                PropertyValue::String(std::borrow::Cow::Borrowed("normal")),
            );
            properties.set(
                PropertyId::FontVariant,
                PropertyValue::String(std::borrow::Cow::Borrowed("normal")),
            );
            properties.set(
                PropertyId::FontWeight,
                PropertyValue::String(std::borrow::Cow::Borrowed("normal")),
            );
            properties.set(
                PropertyId::FontSize,
                PropertyValue::String(std::borrow::Cow::Borrowed("medium")),
            );
            properties.set(
                PropertyId::LineHeight,
                PropertyValue::String(std::borrow::Cow::Borrowed("normal")),
            );
            properties.set(
                PropertyId::FontFamily,
                PropertyValue::String(std::borrow::Cow::Borrowed("serif")),
            );
            return Ok(());
        }
        _ => {}
    }

    // Split by whitespace
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.is_empty() {
        return Ok(());
    }

    let mut idx = 0;
    let mut font_style: Option<PropertyValue> = None;
    let mut font_variant: Option<PropertyValue> = None;
    let mut font_weight: Option<PropertyValue> = None;

    // Parse optional style, variant, weight (order-independent)
    while idx < parts.len() {
        let part = parts[idx];

        // Check for font-style keywords
        if matches!(part, "normal" | "italic" | "oblique") && font_style.is_none() {
            font_style = Some(PropertyValue::String(std::borrow::Cow::Owned(
                part.to_string(),
            )));
            idx += 1;
            continue;
        }

        // Check for font-variant keywords
        if matches!(part, "small-caps") && font_variant.is_none() {
            font_variant = Some(PropertyValue::String(std::borrow::Cow::Owned(
                part.to_string(),
            )));
            idx += 1;
            continue;
        }

        // Check for font-weight keywords
        if matches!(
            part,
            "bold"
                | "bolder"
                | "lighter"
                | "100"
                | "200"
                | "300"
                | "400"
                | "500"
                | "600"
                | "700"
                | "800"
                | "900"
        ) && font_weight.is_none()
        {
            font_weight = Some(PropertyValue::String(std::borrow::Cow::Owned(
                part.to_string(),
            )));
            idx += 1;
            continue;
        }

        // If we get here, we've found the font-size
        break;
    }

    // Parse required font-size and optional line-height
    if idx >= parts.len() {
        // No font-size found, invalid
        return Ok(());
    }

    let size_part = parts[idx];
    idx += 1;

    // Check if size contains line-height (size/line-height)
    if let Some(slash_pos) = size_part.find('/') {
        let (size_str, height_str) = size_part.split_at(slash_pos);
        let height_str = &height_str[1..]; // Skip the '/'

        // Parse font-size
        if let Some(size_value) = parse_font_size(size_str) {
            properties.set(PropertyId::FontSize, size_value);
        }

        // Parse line-height
        if let Some(height_value) = parse_line_height(height_str) {
            properties.set(PropertyId::LineHeight, height_value);
        }
    } else {
        // Just font-size, no line-height
        if let Some(size_value) = parse_font_size(size_part) {
            properties.set(PropertyId::FontSize, size_value);
        }
    }

    // Parse required font-family (rest of the string)
    if idx < parts.len() {
        let family_parts: Vec<&str> = parts[idx..].to_vec();
        let family_str = family_parts.join(" ");

        // Parse comma-separated font families
        let families: Vec<String> = family_str
            .split(',')
            .map(|f| f.trim().to_string())
            .filter(|f| !f.is_empty())
            .collect();

        if !families.is_empty() {
            let family_value = if families.len() == 1 {
                PropertyValue::String(std::borrow::Cow::Owned(families[0].clone()))
            } else {
                PropertyValue::List(
                    families
                        .into_iter()
                        .map(|f| PropertyValue::String(std::borrow::Cow::Owned(f)))
                        .collect(),
                )
            };
            properties.set(PropertyId::FontFamily, family_value);
        }
    }

    // Set optional properties (or defaults if not specified)
    if let Some(style) = font_style {
        properties.set(PropertyId::FontStyle, style);
    } else {
        properties.set(
            PropertyId::FontStyle,
            PropertyValue::String(std::borrow::Cow::Borrowed("normal")),
        );
    }

    if let Some(variant) = font_variant {
        properties.set(PropertyId::FontVariant, variant);
    } else {
        properties.set(
            PropertyId::FontVariant,
            PropertyValue::String(std::borrow::Cow::Borrowed("normal")),
        );
    }

    if let Some(weight) = font_weight {
        properties.set(PropertyId::FontWeight, weight);
    } else {
        properties.set(
            PropertyId::FontWeight,
            PropertyValue::String(std::borrow::Cow::Borrowed("normal")),
        );
    }

    Ok(())
}

/// Parse font-size value
pub(super) fn parse_font_size(value: &str) -> Option<PropertyValue> {
    // Try parsing as absolute keywords
    match value {
        "xx-small" | "x-small" | "small" | "medium" | "large" | "x-large" | "xx-large" => {
            return Some(PropertyValue::String(std::borrow::Cow::Owned(
                value.to_string(),
            )));
        }
        "smaller" | "larger" => {
            return Some(PropertyValue::String(std::borrow::Cow::Owned(
                value.to_string(),
            )));
        }
        _ => {}
    }

    // Try parsing as percentage
    if value.ends_with('%') {
        if let Ok(pct) = value.parse::<fop_types::Percentage>() {
            return Some(PropertyValue::Percentage(pct));
        }
    }

    // Try parsing as length
    if let Some(pt_str) = value.strip_suffix("pt") {
        if let Ok(num) = pt_str.parse::<f64>() {
            return Some(PropertyValue::Length(Length::from_pt(num)));
        }
    } else if let Some(px_str) = value.strip_suffix("px") {
        if let Ok(num) = px_str.parse::<f64>() {
            return Some(PropertyValue::Length(Length::from_pt(num * 0.75)));
        }
    } else if let Some(em_str) = value.strip_suffix("em") {
        if em_str.parse::<f64>().is_ok() {
            // em is relative, store as string for now
            return Some(PropertyValue::String(std::borrow::Cow::Owned(
                value.to_string(),
            )));
        }
    } else if let Some(mm_str) = value.strip_suffix("mm") {
        if let Ok(num) = mm_str.parse::<f64>() {
            return Some(PropertyValue::Length(Length::from_mm(num)));
        }
    } else if let Some(cm_str) = value.strip_suffix("cm") {
        if let Ok(num) = cm_str.parse::<f64>() {
            return Some(PropertyValue::Length(Length::from_cm(num)));
        }
    } else if let Some(in_str) = value.strip_suffix("in") {
        if let Ok(num) = in_str.parse::<f64>() {
            return Some(PropertyValue::Length(Length::from_inch(num)));
        }
    }

    None
}

/// Parse line-height value
pub(super) fn parse_line_height(value: &str) -> Option<PropertyValue> {
    // Handle special keywords
    if value == "normal" {
        return Some(PropertyValue::String(std::borrow::Cow::Borrowed("normal")));
    }

    // Try parsing as percentage
    if value.ends_with('%') {
        if let Ok(pct) = value.parse::<fop_types::Percentage>() {
            return Some(PropertyValue::Percentage(pct));
        }
    }

    // Try parsing as number (unitless multiplier)
    if let Ok(num) = value.parse::<f64>() {
        return Some(PropertyValue::Number(num));
    }

    // Try parsing as length
    if let Some(pt_str) = value.strip_suffix("pt") {
        if let Ok(num) = pt_str.parse::<f64>() {
            return Some(PropertyValue::Length(Length::from_pt(num)));
        }
    } else if let Some(px_str) = value.strip_suffix("px") {
        if let Ok(num) = px_str.parse::<f64>() {
            return Some(PropertyValue::Length(Length::from_pt(num * 0.75)));
        }
    } else if let Some(em_str) = value.strip_suffix("em") {
        if em_str.parse::<f64>().is_ok() {
            return Some(PropertyValue::String(std::borrow::Cow::Owned(
                value.to_string(),
            )));
        }
    }

    None
}
