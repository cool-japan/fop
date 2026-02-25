//! Example demonstrating shorthand property expansion

use fop_core::{Length, PropertyId, PropertyList, PropertyValue, ShorthandExpander};

fn main() {
    println!("=== Shorthand Property Expansion ===\n");

    // Single value margin
    println!("1. margin=\"10pt\" expands to:");
    let mut props = PropertyList::new();
    let value = PropertyValue::Length(Length::from_pt(10.0));
    ShorthandExpander::expand(&mut props, "margin", &value).expect("bench/example: should succeed");

    print_margin(&props);

    // Two value margin
    println!("\n2. margin=\"10pt 20pt\" expands to:");
    let mut props = PropertyList::new();
    let value = PropertyValue::List(vec![
        PropertyValue::Length(Length::from_pt(10.0)),
        PropertyValue::Length(Length::from_pt(20.0)),
    ]);
    ShorthandExpander::expand(&mut props, "margin", &value).expect("bench/example: should succeed");

    print_margin(&props);

    // Four value margin
    println!("\n3. margin=\"10pt 20pt 30pt 40pt\" expands to:");
    let mut props = PropertyList::new();
    let value = PropertyValue::List(vec![
        PropertyValue::Length(Length::from_pt(10.0)),
        PropertyValue::Length(Length::from_pt(20.0)),
        PropertyValue::Length(Length::from_pt(30.0)),
        PropertyValue::Length(Length::from_pt(40.0)),
    ]);
    ShorthandExpander::expand(&mut props, "margin", &value).expect("bench/example: should succeed");

    print_margin(&props);

    // Padding expansion
    println!("\n4. padding=\"5pt\" expands to:");
    let mut props = PropertyList::new();
    let value = PropertyValue::Length(Length::from_pt(5.0));
    ShorthandExpander::expand(&mut props, "padding", &value)
        .expect("bench/example: should succeed");

    println!(
        "  padding-top: {}",
        props
            .get(PropertyId::PaddingTop)
            .expect("bench/example: should succeed")
            .as_length()
            .expect("bench/example: should succeed")
    );
    println!(
        "  padding-right: {}",
        props
            .get(PropertyId::PaddingRight)
            .expect("bench/example: should succeed")
            .as_length()
            .expect("bench/example: should succeed")
    );
    println!(
        "  padding-bottom: {}",
        props
            .get(PropertyId::PaddingBottom)
            .expect("bench/example: should succeed")
            .as_length()
            .expect("bench/example: should succeed")
    );
    println!(
        "  padding-left: {}",
        props
            .get(PropertyId::PaddingLeft)
            .expect("bench/example: should succeed")
            .as_length()
            .expect("bench/example: should succeed")
    );
}

fn print_margin(props: &PropertyList) {
    println!(
        "  margin-top: {}",
        props
            .get(PropertyId::MarginTop)
            .expect("bench/example: should succeed")
            .as_length()
            .expect("bench/example: should succeed")
    );
    println!(
        "  margin-right: {}",
        props
            .get(PropertyId::MarginRight)
            .expect("bench/example: should succeed")
            .as_length()
            .expect("bench/example: should succeed")
    );
    println!(
        "  margin-bottom: {}",
        props
            .get(PropertyId::MarginBottom)
            .expect("bench/example: should succeed")
            .as_length()
            .expect("bench/example: should succeed")
    );
    println!(
        "  margin-left: {}",
        props
            .get(PropertyId::MarginLeft)
            .expect("bench/example: should succeed")
            .as_length()
            .expect("bench/example: should succeed")
    );
}
