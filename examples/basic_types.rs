//! Example demonstrating the basic types in fop-types

use fop_types::{Color, Length, Point, Rect, Size};

fn main() {
    // Length examples
    println!("=== Length Types ===");
    let pt = Length::from_pt(72.0);
    let inch = Length::from_inch(1.0);
    println!("72pt = {}", pt);
    println!("1in = {}", inch);
    println!("72pt == 1in: {}", pt == inch);

    // Length conversions
    let mm = Length::from_mm(25.4);
    println!("25.4mm = {:.2}in", mm.to_inch());

    // Length arithmetic
    let sum = pt + inch;
    println!("72pt + 1in = {}", sum);

    // Color examples
    println!("\n=== Color Types ===");
    let red = Color::RED;
    let custom = Color::rgb(128, 64, 192);
    println!("Red: {}", red);
    println!("Custom: {}", custom);

    // Hex parsing
    if let Some(color) = Color::from_hex("#FF00FF") {
        println!("Parsed #FF00FF: {}", color);
    }

    // Geometry examples
    println!("\n=== Geometry Types ===");
    let origin = Point::new(Length::from_pt(10.0), Length::from_pt(20.0));
    println!("Origin: {:?}", origin);

    let size = Size::new(Length::from_pt(100.0), Length::from_pt(200.0));
    println!("Size: {:?}", size);

    let rect = Rect::from_point_size(origin, size);
    println!("Rect: {:?}", rect);

    let test_point = Point::new(Length::from_pt(50.0), Length::from_pt(100.0));
    println!(
        "Point {:?} in rect: {}",
        test_point,
        rect.contains(test_point)
    );
}
