//! Example demonstrating the property system

use fop_core::{Color, Length, PropertyId, PropertyValue};
use std::borrow::Cow;

fn main() {
    println!("=== Property System ===");

    // Property IDs
    println!("\nProperty IDs:");
    let font_size = PropertyId::FontSize;
    let color_prop = PropertyId::Color;
    println!("  {} (id: {:?})", font_size.name(), font_size as u16);
    println!("  {} (id: {:?})", color_prop.name(), color_prop as u16);

    // Parse property names
    if let Some(prop) = PropertyId::from_name("margin-top") {
        println!("  Parsed 'margin-top': {}", prop.name());
    }

    // Property values
    println!("\nProperty Values:");
    let length_val = PropertyValue::Length(Length::from_pt(12.0));
    println!("  Font size: {:?}", length_val);

    let color_val = PropertyValue::Color(Color::rgb(255, 0, 0));
    println!("  Color: {:?}", color_val);

    let string_val = PropertyValue::String(Cow::Borrowed("Arial"));
    println!("  Font family: {:?}", string_val);

    let auto_val = PropertyValue::Auto;
    println!("  Width: {:?}", auto_val);

    // Extract values
    println!("\nExtracting Values:");
    if let Some(len) = length_val.as_length() {
        println!("  Length value: {}", len);
    }

    if let Some(s) = string_val.as_string() {
        println!("  String value: {}", s);
    }

    println!("  Is auto? {}", auto_val.is_auto());

    // List of values
    let list_val = PropertyValue::List(vec![
        PropertyValue::Length(Length::from_pt(10.0)),
        PropertyValue::Length(Length::from_pt(20.0)),
        PropertyValue::Length(Length::from_pt(10.0)),
        PropertyValue::Length(Length::from_pt(20.0)),
    ]);
    println!("\nMargin (top right bottom left): {:?}", list_val);
}
