# fop-layout

[![Crates.io](https://img.shields.io/crates/v/fop-layout.svg)](https://crates.io/crates/fop-layout)
[![docs.rs](https://img.shields.io/docsrs/fop-layout)](https://docs.rs/fop-layout)
[![License](https://img.shields.io/crates/l/fop-layout.svg)](https://github.com/cool-japan/fop/blob/main/LICENSE)

Layout engine for the Apache FOP Rust implementation. Transforms the FO tree (from `fop-core`) into an area tree that can be rendered to PDF or other formats.

**Version:** 0.1.0 | **Release Date:** 2026-02-25

## Pipeline

```
FO Tree (fop-core)
    |
    v
LayoutEngine        -- coordinates the process
    |
    +-- BlockLayout  -- stacks blocks vertically
    +-- InlineLayout -- positions text horizontally, breaks lines
    +-- TableLayout  -- computes column widths, positions cells
    +-- ListLayout   -- positions labels and bodies side-by-side
    +-- PageBreaker  -- splits content across pages
    |
    v
Area Tree            -- positioned rectangles ready for rendering
```

## Installation

Add `fop-layout` to your `Cargo.toml`:

```toml
[dependencies]
fop-layout = "0.1"
```

Or with `cargo add`:

```sh
cargo add fop-layout
```

## Usage

```rust
use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let xml = r#"<?xml version="1.0"?>
    <fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
        <fo:layout-master-set>
            <fo:simple-page-master master-name="A4"
                page-width="210mm" page-height="297mm">
                <fo:region-body margin="1in"/>
            </fo:simple-page-master>
        </fo:layout-master-set>
        <fo:page-sequence master-reference="A4">
            <fo:flow flow-name="xsl-region-body">
                <fo:block>Hello, Layout!</fo:block>
            </fo:flow>
        </fo:page-sequence>
    </fo:root>"#;

    let fo_tree = FoTreeBuilder::new().parse(Cursor::new(xml))?;
    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&fo_tree)?;

    println!("Generated {} areas", area_tree.len());
    Ok(())
}
```

## Architecture

### `area/` — Area Tree Data Structures

| File | Lines | Description |
|------|------:|-------------|
| `types.rs` | 205 | `Area`, `AreaType` (8 types), `TraitSet`, `FontStyle` |
| `area_tree.rs` | 241 | `AreaTree`, `AreaNode`, `AreaId` (arena-allocated) |

**Area Types:** Page, Region, Block, Inline, Line, Table, ListItem, Image

Each area has a `Rect` (position + size) and an optional `TraitSet` for styling (font, color, borders).

### `layout/` — Layout Algorithms

| File | Lines | Description |
|------|------:|-------------|
| `engine.rs` | 533 | `LayoutEngine` — main coordinator, wires all algorithms |
| `block.rs` | 100 | `BlockLayoutContext` — vertical block stacking |
| `inline.rs` | 238 | `InlineArea`, `InlineLayoutContext`, `LineBreaker` |
| `knuth_plass.rs` | 336 | `KnuthPlassBreaker` — optimal line breaking algorithm |
| `table.rs` | 421 | `TableLayout`, `ColumnWidth` — table column computation |
| `list.rs` | 362 | `ListLayout`, `ListMarkerStyle` — list positioning |
| `page_break.rs` | 283 | `PageBreaker` — multi-page content splitting |
| `properties.rs` | 210 | Property extraction (FO properties → area traits) |

**Total: 2,994 lines across 11 files**

### Key Algorithms

#### Knuth-Plass Line Breaking

Optimal line breaking using the Knuth-Plass algorithm (same as TeX). Minimizes total "badness" across all lines rather than greedily filling each line.

```rust
use fop_layout::KnuthPlassBreaker;
use fop_types::Length;

fn break_lines() -> Result<(), Box<dyn std::error::Error>> {
    let breaker = KnuthPlassBreaker::new(Length::from_pt(300.0));
    let items = breaker.text_to_items("Long paragraph text...", 12.0);
    let breaks = breaker.find_breaks(&items);
    println!("Break points: {:?}", breaks);
    Ok(())
}
```

#### Table Layout

Computes column widths using fixed, proportional, or automatic sizing.

```rust
use fop_layout::{TableLayout, ColumnWidth};
use fop_types::Length;

fn compute_table() -> Result<(), Box<dyn std::error::Error>> {
    let layout = TableLayout::new(Length::from_pt(500.0))
        .with_border_spacing(Length::from_pt(2.0));

    let widths = layout.compute_fixed_widths(&[
        ColumnWidth::Fixed(Length::from_pt(100.0)),
        ColumnWidth::Proportional(2.0),
        ColumnWidth::Proportional(1.0),
    ]);
    println!("Column widths: {:?}", widths);
    Ok(())
}
```

#### List Layout

Positions list labels (markers) and bodies side-by-side with 9 marker styles.

```rust
use fop_layout::{ListLayout, ListMarkerStyle};
use fop_types::Length;

fn layout_list() -> Result<(), Box<dyn std::error::Error>> {
    let layout = ListLayout::new(Length::from_pt(400.0))
        .with_label_width(Length::from_pt(30.0))
        .with_label_separation(Length::from_pt(10.0));

    let marker = layout.generate_marker(1, ListMarkerStyle::Decimal); // "1."
    let bullet = layout.generate_marker(1, ListMarkerStyle::Disc);    // "•"
    println!("{marker} {bullet}");
    Ok(())
}
```

**Marker Styles:** Disc, Circle, Square, Decimal, LowerAlpha, UpperAlpha, LowerRoman, UpperRoman, None

#### Page Breaking

Splits content across multiple pages when it overflows.

```rust
use fop_layout::PageBreaker;
use fop_types::Length;

fn page_break_demo() -> Result<(), Box<dyn std::error::Error>> {
    let breaker = PageBreaker::new(
        Length::from_mm(210.0),  // A4 width
        Length::from_mm(297.0),  // A4 height
        [Length::from_inch(1.0); 4],  // 1-inch margins
    );

    assert_eq!(breaker.content_height(), Length::from_pt(698.0));
    assert!(breaker.fits_on_page(Length::from_pt(100.0), Length::from_pt(200.0)));
    Ok(())
}
```

## Tests

52 unit tests covering:

- Block layout vertical stacking
- Inline layout horizontal positioning
- Knuth-Plass line breaking (various paragraph widths)
- Table layout (fixed, proportional, auto, mixed columns)
- List layout (all 9 marker styles, positioning)
- Page breaking (overflow detection, multi-page, area splitting)
- Area tree construction and traversal
- Property extraction from FO nodes

## Dependencies

| Crate | Version | Kind |
|-------|---------|------|
| [fop-types](https://crates.io/crates/fop-types) | workspace | Internal |
| [fop-core](https://crates.io/crates/fop-core) | workspace | Internal |
| [thiserror](https://crates.io/crates/thiserror) | 2.0 | Error handling |
| [log](https://crates.io/crates/log) | 0.4 | Logging |

## Related Crates

| Crate | Description |
|-------|-------------|
| [fop-types](https://crates.io/crates/fop-types) | Core XSL-FO types (lengths, colors, properties) |
| [fop-core](https://crates.io/crates/fop-core) | FO tree parser and builder |
| [fop-render](https://crates.io/crates/fop-render) | Rendering traits and SVG/PostScript output |
| [fop-pdf-renderer](https://crates.io/crates/fop-pdf-renderer) | PDF output renderer |
| [fop-cli](https://crates.io/crates/fop-cli) | Command-line interface |

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](../../LICENSE) for details.

## Author

Copyright 2024–2026 COOLJAPAN OU (Team Kitasan)
