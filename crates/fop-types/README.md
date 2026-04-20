# fop-types

[![Crates.io](https://img.shields.io/crates/v/fop-types.svg)](https://crates.io/crates/fop-types)
[![docs.rs](https://img.shields.io/docsrs/fop-types)](https://docs.rs/fop-types)
[![License](https://img.shields.io/crates/l/fop-types.svg)](LICENSE)

Foundational types for the FOP (Formatting Objects Processor) Rust implementation.
This crate has no internal dependencies and is used by all other FOP crates as the
shared type foundation for lengths, colors, geometry, font metrics, and error handling.

## Installation

Add `fop-types` to your `Cargo.toml`:

```toml
[dependencies]
fop-types = "0.1.1"
```

## Types

### `Length`

Dimensional measurements stored as integer millipoints (1/1000 of a typographic point)
for precision without floating-point errors.

```rust
use fop_types::Length;

fn example() -> Result<(), Box<dyn std::error::Error>> {
    let a = Length::from_pt(72.0);    // 72 points
    let b = Length::from_inch(1.0);   // 1 inch = 72 points
    assert_eq!(a, b);

    let c = Length::from_mm(210.0);   // A4 width
    let d = Length::from_cm(21.0);    // Same thing
    assert_eq!(c, d);

    // Arithmetic
    let sum = a + b;
    let half = a / 2;

    // Convert back to physical units
    println!("sum = {} pt", sum.to_pt());
    println!("half = {} mm", half.to_mm());
    Ok(())
}
```

### `Color`

RGBA color values with named color constants and hex parsing.

```rust
use fop_types::Color;

fn example() -> Result<(), Box<dyn std::error::Error>> {
    let red = Color::RED;
    let custom = Color::from_hex("#FF8800")?;
    let transparent = Color::new(255, 0, 0, 128);

    assert_eq!(red.r(), 255);
    assert_eq!(custom.g(), 0x88);
    assert_eq!(transparent.a(), 128);
    Ok(())
}
```

### `Point`, `Size`, `Rect`

2D geometry types using `Length` for coordinates.

```rust
use fop_types::{Length, Point, Size, Rect};

fn example() -> Result<(), Box<dyn std::error::Error>> {
    let origin = Point::new(Length::from_pt(10.0), Length::from_pt(20.0));
    let size = Size::new(Length::from_pt(100.0), Length::from_pt(50.0));
    let rect = Rect::from_point_size(origin, size);

    assert!(rect.contains(Point::new(
        Length::from_pt(50.0),
        Length::from_pt(30.0),
    )));
    Ok(())
}
```

### `FontMetrics` / `FontRegistry`

Font measurement data for layout calculations. Provides character widths, ascent,
descent, and line height for built-in fonts (Helvetica, Times-Roman).

```rust
use fop_types::FontRegistry;

fn example() -> Result<(), Box<dyn std::error::Error>> {
    let registry = FontRegistry::with_defaults();
    let helvetica = registry
        .get("Helvetica")
        .ok_or("font not found")?;
    let width = helvetica.string_width("Hello", 12.0);

    println!("\"Hello\" in 12pt Helvetica = {} pt", width.to_pt());
    Ok(())
}
```

### `FopError` / `Result`

Base error type used across all FOP crates, built on `thiserror`.

```rust
use fop_types::{FopError, FopResult};

fn parse_something(input: &str) -> FopResult<i32> {
    input.parse::<i32>().map_err(|e| {
        FopError::Parse(format!("invalid integer: {e}"))
    })
}
```

## Modules

| Module | Lines | Description |
|--------|------:|-------------|
| `length.rs` | 268 | Length type with unit conversions (pt, mm, cm, inch, millipoints) |
| `color.rs` | 194 | RGBA color with hex parsing and named constants |
| `geometry.rs` | 263 | Point, Size, Rect — 2D geometry primitives |
| `font_metrics.rs` | 392 | FontMetrics, FontRegistry — font measurement for layout |
| `error.rs` | 42 | FopError enum with thiserror derive |
| `lib.rs` | 19 | Public re-exports |
| **Total** | **1,178** | |

## Tests

25 unit tests covering:

- Length unit conversions (pt, mm, cm, inch) and arithmetic
- Color hex parsing, named colors, edge cases
- Geometry containment, intersection, size calculations
- Font metrics string width computation
- Error type Display/Debug

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `thiserror` | 2.0 | Derive macro for error types |

## Related Crates

| Crate | Description |
|-------|-------------|
| [`fop-core`](https://crates.io/crates/fop-core) | XSL-FO parser and document model |
| [`fop-layout`](https://crates.io/crates/fop-layout) | Page layout engine |
| [`fop-render`](https://crates.io/crates/fop-render) | Rendering abstraction layer |
| [`fop-pdf-renderer`](https://crates.io/crates/fop-pdf-renderer) | PDF output backend |
| [`fop-cli`](https://crates.io/crates/fop-cli) | Command-line interface |
| [`fop`](https://crates.io/crates/fop) | Umbrella crate (all-in-one) |

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE) for details.

## Author

Copyright 2024-2026 COOLJAPAN OU (Team Kitasan)

Repository: <https://github.com/cool-japan/fop>
