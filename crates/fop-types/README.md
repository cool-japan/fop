# fop-types

Foundational types for the Apache FOP Rust implementation. This crate has no internal dependencies and is used by all other FOP crates.

## Types

### `Length`
Dimensional measurements stored as integer millipoints (1/1000 of a typographic point) for precision without floating-point errors.

```rust
use fop_types::Length;

let a = Length::from_pt(72.0);    // 72 points
let b = Length::from_inch(1.0);   // 1 inch = 72 points
assert_eq!(a, b);

let c = Length::from_mm(210.0);   // A4 width
let d = Length::from_cm(21.0);    // Same thing
assert_eq!(c, d);

// Arithmetic
let sum = a + b;
let half = a / 2;
```

### `Color`
RGBA color values with named color constants and hex parsing.

```rust
use fop_types::Color;

let red = Color::RED;
let custom = Color::from_hex("#FF8800").unwrap();
let transparent = Color::new(255, 0, 0, 128);
```

### `Point`, `Size`, `Rect`
2D geometry types using `Length` for coordinates.

```rust
use fop_types::{Length, Point, Size, Rect};

let origin = Point::new(Length::from_pt(10.0), Length::from_pt(20.0));
let size = Size::new(Length::from_pt(100.0), Length::from_pt(50.0));
let rect = Rect::from_point_size(origin, size);

assert!(rect.contains(Point::new(Length::from_pt(50.0), Length::from_pt(30.0))));
```

### `FontMetrics` / `FontRegistry`
Font measurement data for layout calculations. Provides character widths, ascent, descent, and line height for built-in fonts (Helvetica, Times-Roman).

```rust
use fop_types::FontRegistry;

let registry = FontRegistry::with_defaults();
let helvetica = registry.get("Helvetica").unwrap();
let width = helvetica.string_width("Hello", 12.0);
```

### `FopError` / `Result`
Base error type used across all FOP crates.

## Modules

| Module | Lines | Description |
|--------|------:|-------------|
| `length.rs` | 268 | Length type with unit conversions |
| `color.rs` | 194 | RGBA color with hex parsing and constants |
| `geometry.rs` | 263 | Point, Size, Rect |
| `font_metrics.rs` | 392 | FontMetrics, FontRegistry |
| `error.rs` | 42 | FopError enum |
| `lib.rs` | 19 | Re-exports |
| **Total** | **1,178** | |

## Tests

25 unit tests covering:
- Length unit conversions (pt, mm, cm, inch) and arithmetic
- Color hex parsing, named colors, edge cases
- Geometry containment, intersection, size calculations
- Font metrics string width computation
- Error type Display/Debug

## Dependencies

- `thiserror` 2.0
