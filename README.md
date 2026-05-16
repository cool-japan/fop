# FOP — Pure Rust XSL-FO Processor

[![Crates.io](https://img.shields.io/crates/v/fop.svg)](https://crates.io/crates/fop)
[![docs.rs](https://img.shields.io/docsrs/fop)](https://docs.rs/fop)
[![License](https://img.shields.io/crates/l/fop.svg)](LICENSE)
[![CI](https://github.com/cool-japan/fop/actions/workflows/ci.yml/badge.svg)](https://github.com/cool-japan/fop/actions)

A high-performance, pure Rust reimplementation of [Apache FOP](https://xmlgraphics.apache.org/fop/) (Formatting Objects Processor), translating XSL-FO documents to PDF, SVG, PostScript, raster images, and plain text.

**171 Rust files · 74,516 lines of code · 3,063+ tests · Zero warnings · 10–1200× faster than Java FOP**
<!-- stats regenerated: 2026-05-16 -->

## Project Status

**Phase 5–6 Enhanced** — Production ready with comprehensive features and testing.

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 1: Foundation | ✅ Complete | Core types, property system, FO tree parsing |
| Phase 2: Basic Layout | ✅ Complete | Area tree, block/inline layout, PDF output |
| Phase 3: Advanced Layout | ✅ Complete | Knuth-Plass line breaking, tables, lists, graphics, images |
| Phase 4: Integration | ✅ Complete | Multi-page breaking, engine integration |
| Phase 5: Advanced Features | ✅ Complete | Image rendering, links, bookmarks, font embedding, i18n, encryption |
| Phase 6: Optimization | 🔄 85% Complete | Performance (✅), streaming (✅), testing (✅) |

**Current stats:** 3,063+ tests (all passing), 0 compiler warnings, 0 clippy warnings, **10–1200× faster than Java FOP**

## Key Features

### XSL-FO Processing
- **294 XSL-FO 1.1 properties** with full inheritance and shorthand expansion
- **29 FO element types** — blocks, inlines, tables, lists, graphics, links, page masters
- **Knuth-Plass optimal line breaking** — same algorithm as TeX for publication-quality typography
- **Multi-page breaking** with widow/orphan control and overflow detection

### Output Formats
- **PDF** — valid, viewable, text-extractable, with font embedding and subsetting
- **SVG** — scalable vector graphics output
- **PostScript** — for print workflows
- **PNG / JPEG** — raster image output
- **Plain text** — text extraction mode

### PDF Advanced Features
- **PDF encryption** — RC4-128 with owner/user passwords and permission control
- **PDF/A compliance mode** — for archival-quality output
- **Bookmarks and outlines** — document navigation
- **Internal and external links** — clickable hyperlinks
- **Font embedding and subsetting** — TrueType/OpenType with CIDFontType2 + Identity-H encoding
- **XMP metadata embedding** — `<fo:declarations>` block embeds XMP metadata into generated PDF output

### XML Parsing
- **Namespace scope management** — XmlParser correctly tracks namespace scope inheritance during parsing

### Internationalization (i18n)
- **16+ languages** supported — Japanese, Chinese, Korean, Arabic, Thai, Hindi, Hebrew, and more
- **CJK support** — Type 0 composite fonts with proper glyph rendering
- **Right-to-left (RTL)** — Arabic and Hebrew bidirectional text
- **Emoji and full Unicode** — complete Unicode coverage
- See [docs/I18N_CAPABILITIES.md](docs/I18N_CAPABILITIES.md) for the full guide

### Streaming & Performance
- **Streaming mode** for large documents (>1000 pages) with bounded memory
- **Parallel rendering infrastructure** for multi-core utilization
- **10–1200× faster** than Java FOP across all benchmarks
- **<10ms startup** vs ~2000ms JVM cold start

### Self-verification (Pure Rust)

The `fop-pdf-renderer` crate parses and rasterizes PDF output with no C dependencies (no poppler, no Ghostscript). Use it programmatically or from the CLI:

```rust
use fop_pdf_renderer::PdfRenderer;

let renderer = PdfRenderer::from_bytes(&pdf_bytes)?;
let page = renderer.render_page(0, 150.0)?;  // 150 DPI → RasterPage
let text = renderer.extract_text(0)?;        // user-visible text extraction
```

CLI equivalents:

```bash
# Round-trip self-verification (generates PDF then rasterizes it internally)
fop input.fo output.pdf --render-verify

# Standalone PDF → PNG conversion
fop-render-pdf output.pdf page.png 150
```

### Bindings
- **CLI** — Apache FOP-compatible command-line interface with progress bars and JSON stats
- **WASM** — browser and Node.js bindings via `wasm-bindgen`
- **Python** — bindings via PyO3 (published as `fop2` on PyPI)

## Performance

Real benchmark results against Java Apache FOP:

| Metric | Java FOP | Rust FOP | Speedup |
|--------|----------|----------|---------|
| Simple document render | ~50ms | ~0.04ms | **~1200×** |
| Parse 1000-page doc | ~500ms | <50ms | **~10×** |
| Memory per page | ~50KB | <5KB | **~10×** |
| Binary size | 15MB+ JAR | <5MB stripped | **~3×** |
| Startup time | ~2000ms (JVM) | <10ms | **~200×** |

## Quick Start

```bash
# Build
cargo build --release

# Run tests (zero warnings policy)
cargo nextest run --all-features
cargo clippy --all-targets -- -D warnings

# Run an example
cargo run --example hello_pdf
```

### Generate a PDF from XSL-FO

```rust
use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use fop_render::PdfRenderer;
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
            <fo:block font-size="14pt">Hello, FOP!</fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;

    // Parse -> Layout -> Render
    let fo_tree = FoTreeBuilder::new().parse(Cursor::new(xml))?;
    let area_tree = LayoutEngine::new().layout(&fo_tree)?;
    let pdf_doc = PdfRenderer::new().render(&area_tree)?;
    let pdf_bytes = pdf_doc.to_bytes()?;

    std::fs::write("output.pdf", &pdf_bytes)?;
    println!("Wrote output.pdf ({} bytes)", pdf_bytes.len());
    Ok(())
}
```

### CLI Usage

```bash
# Convert XSL-FO to PDF
fop -fo input.fo -pdf output.pdf

# Convert to SVG
fop -fo input.fo -svg output.svg

# Convert to PostScript
fop -fo input.fo -ps output.ps

# With progress bar and JSON stats
fop -fo input.fo -pdf output.pdf --progress --stats

# See all options
fop --help
```

See [crates/fop-cli/USAGE.md](crates/fop-cli/USAGE.md) for the full CLI reference.

## Python Bindings

The Python package is available on PyPI as `fop2`.

```bash
pip install fop2
```

```python
import fop

# One-shot conversion
pdf_bytes = fop.convert_to_pdf(fo_xml_string)

# Or use the converter class
converter = fop.FopConverter()
pdf = converter.convert_to_pdf(fo_xml)
svg = converter.convert_to_svg(fo_xml)
text = converter.convert_to_text(fo_xml)
```

## WASM Bindings

The WASM package is available on npm as `@cooljapan/fop`.

```bash
npm install @cooljapan/fop
```

```javascript
import init, { FopConverter } from '@cooljapan/fop';

await init();
const converter = new FopConverter();
const pdfBytes = converter.convertToPdf(foXml);
const svgString = converter.convertToSvg(foXml);
```

## Documentation

📚 **[Comprehensive documentation available in `docs/`](docs/)**

- [docs/README.md](docs/README.md) — Complete documentation index
- [docs/I18N_CAPABILITIES.md](docs/I18N_CAPABILITIES.md) — Internationalization guide (Japanese, CJK, Arabic RTL, etc.)
- [docs/LIMITATIONS.md](docs/LIMITATIONS.md) — Current limitations and migration guide from Java FOP

## Workspace Structure

```
fop/
├── Cargo.toml                  # Workspace root + top-level fop crate
├── crates/
│   ├── fop-types/              # Core types (Length, Color, Rect, FontMetrics, errors)
│   ├── fop-core/               # FO tree parsing & 294-property system
│   ├── fop-layout/             # Layout engine (block, inline, table, list, page breaking)
│   ├── fop-render/             # Rendering backends (PDF, SVG, PostScript, raster, text)
│   ├── fop-pdf-renderer/       # Pure Rust PDF-to-image renderer (glyph outlines, text extraction, self-verification)
│   ├── fop-cli/                # CLI tool (fop binary)
│   ├── fop-wasm/               # WebAssembly bindings (browser / Node.js)
│   └── fop-python/             # Python bindings via PyO3
├── examples/                   # 23 runnable examples
├── benches/                    # Performance & comparison benchmarks
├── tests/                      # Integration tests
├── fuzz/                       # Fuzz testing targets
└── docs/                       # Documentation
```

### Dependency Graph

```
fop-types              (no internal deps)
    │
    ├── fop-core       (+ quick-xml)
    │       │
    │       ├── fop-layout   (+ image)
    │       │       │
    │       │       ├── fop-render   (+ oxiarc-deflate, ttf-parser, png, aes, sha2, md-5,
    │       │       │                   resvg, usvg, jpeg-encoder, tiny-skia)
    │       │       │
    │       │       ├── fop-cli      (+ clap, anyhow, indicatif, console,
    │       │       │                   humantime, bytesize, serde_json)
    │       │       │
    │       │       ├── fop-wasm     (+ wasm-bindgen, js-sys, serde-wasm-bindgen)
    │       │       │
    │       │       └── fop-python   (+ pyo3)
    │       │
    └───────┘

fop-pdf-renderer       (standalone: thiserror, oxiarc-deflate, ttf-parser, png,
                         jpeg-decoder, tiny-skia)
```

## Supported XSL-FO Elements

| Category | Elements |
|----------|----------|
| Root & Structure | `fo:root`, `fo:layout-master-set`, `fo:simple-page-master` |
| Regions | `fo:region-body`, `fo:region-before`, `fo:region-after`, `fo:region-start`, `fo:region-end` |
| Page Sequences | `fo:page-sequence`, `fo:flow`, `fo:static-content` |
| Block-level | `fo:block`, `fo:block-container` |
| Inline-level | `fo:inline`, `fo:character`, `fo:page-number`, `fo:page-number-citation` |
| Tables | `fo:table`, `fo:table-column`, `fo:table-header`, `fo:table-body`, `fo:table-row`, `fo:table-cell` |
| Lists | `fo:list-block`, `fo:list-item`, `fo:list-item-label`, `fo:list-item-body` |
| Graphics & Links | `fo:external-graphic`, `fo:basic-link` |
| Leaders | `fo:leader` |

## Examples

All 23 examples can be run with `cargo run --example <name>`:

| Example | Description |
|---------|-------------|
| `basic_types` | Length, Color, Geometry type usage |
| `border_background_demo` | Borders and background styling |
| `cli_production_example` | Production CLI workflow demonstration |
| `comprehensive_demo` | All Phase 3 features combined |
| `generate_japanese_pdfs` | Japanese PDF generation with CJK fonts |
| `hello_pdf` | Minimal PDF generation |
| `i18n_multi_font` | Multi-font internationalization |
| `i18n_showcase` | Full i18n showcase (16+ languages) |
| `layout_demo` | FO tree to area tree transformation |
| `manual_japanese_font` | Manual Japanese font configuration |
| `parse_fo_document` | Full XSL-FO document parsing |
| `pdf_encryption` | PDF encryption with RC4-128 |
| `phase5_complete_demo` | Phase 5 complete feature demonstration |
| `phase5_features` | Phase 5 individual features |
| `properties` | Property system and inheritance |
| `ps_output_demo` | PostScript output generation |
| `shorthand` | Shorthand property expansion (margin, padding, border) |
| `streaming_demo` | Streaming mode for large documents |
| `styled_pdf` | PDF with colors and fonts |
| `svg_output_demo` | SVG output generation |
| `tables_lists_demo` | Tables and lists end-to-end |
| `text_extraction_demo` | Text extraction from PDF |
| `validation` | Element nesting validation |

## Dependencies

All dependencies are **pure Rust** — no C/Fortran/system libraries required.

### Production Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `quick-xml` | 0.39 | XML parsing (zero-copy) |
| `thiserror` | 2.0 | Error type derivation |
| `log` | 0.4 | Logging facade |
| `oxiarc-deflate` | 0.2.6 | PDF stream compression (deflate) — pure Rust |
| `ttf-parser` | 0.25 | TrueType/OpenType font parsing |
| `png` | 0.18 | PNG image encoding/decoding |
| `jpeg-decoder` | 0.3 | JPEG image decoding |
| `jpeg-encoder` | 0.7 | JPEG image encoding |
| `image` | 0.25 | Image format detection and loading |
| `tiny-skia` | 0.12 | 2D raster rendering |
| `resvg` | 0.47 | SVG rendering |
| `usvg` | 0.47 | SVG tree simplification |
| `aes` | 0.8 | AES encryption (PDF security) |
| `cbc` | 0.1 | CBC block cipher mode |
| `sha2` | 0.10 | SHA-256 hashing (PDF encryption) |
| `md-5` | 0.10 | MD5 hashing (PDF encryption) |

### CLI Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `clap` | 4.5 | Command-line argument parsing |
| `anyhow` | 1.0 | Error handling |
| `indicatif` | 0.18 | Progress bars |
| `console` | 0.16 | Terminal formatting |
| `humantime` | 2.3 | Human-readable durations |
| `bytesize` | 2.3 | Human-readable byte sizes |
| `serde_json` | 1.0 | JSON statistics output |

### Bindings Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `pyo3` | 0.28 | Python bindings (ABI3, Python 3.8+) |
| `wasm-bindgen` | 0.2 | WebAssembly bindings |
| `js-sys` | 0.3 | JavaScript interop |

## Architecture

### Pipeline

```
XSL-FO XML ──→ FO Tree ──→ Area Tree ──→ Output (PDF/SVG/PS/PNG/text)
  (parse)       (layout)     (render)
```

### Design Principles

- **Zero-copy parsing** — `Cow<'static, str>` and arena allocation throughout
- **Arena allocation** — index-based tree nodes (no `Rc<RefCell<>>` overhead)
- **Static dispatch** — enum dispatch over trait objects where possible
- **Millipoint arithmetic** — 1/1000 pt precision with integer math (no floating-point drift)
- **Release profile** — LTO + single codegen unit + opt-level 3 + stripped binaries

### Testing

- **3,063+ tests** — unit, integration, and fuzz targets
- **Zero warnings** — enforced via `cargo clippy --all-targets -- -D warnings`
- **Fuzz testing** — `fuzz_xml_parser`, `fuzz_property_parser`, `fuzz_layout`
- **PDF self-verification** — `fop-pdf-renderer` renders generated PDFs back to compare

### Continuous Integration

`.github/workflows/ci.yml` runs `fmt`, `clippy -D warnings`, `cargo nextest`, and `doc -D warnings` on every push and PR across **Linux, macOS, and Windows** (stable Rust). No poppler or Ghostscript is installed on any runner — the auto-verify path is exercised end-to-end through the pure-Rust `fop-pdf-renderer`. Python (PyO3) and WASM bindings are built in a dedicated job that installs the required toolchains.

## Development

### Zero Warnings Policy

This project enforces zero compiler warnings and zero clippy warnings:

```bash
cargo clippy --all-targets -- -D warnings
cargo nextest run --all-features
```

### Build Profiles

```bash
# Debug build
cargo build

# Optimized release build (LTO + stripped)
cargo build --release

# Build WASM bindings
cd crates/fop-wasm && wasm-pack build --target web

# Build Python bindings
cd crates/fop-python && maturin develop --release
```

### Benchmarks

```bash
cargo bench --bench fop_benchmarks
cargo bench --bench comparison_benchmarks
cargo bench --bench performance_benchmarks
```

See [benches/README.md](benches/README.md) for benchmark details.

## Reference

- Based on [Apache FOP](https://xmlgraphics.apache.org/fop/) (Java, 1566+ files across 8 Maven modules)
- Implements [XSL-FO 1.1 Specification](https://www.w3.org/TR/xsl11/)
- [CHANGELOG.md](CHANGELOG.md) — Release history

## Sponsorship

FOP is developed and maintained by **COOLJAPAN OU (Team Kitasan)**.

If you find FOP useful, please consider sponsoring the project to support continued development of the Pure Rust ecosystem.

[![Sponsor](https://img.shields.io/badge/Sponsor-%E2%9D%A4-red?logo=github)](https://github.com/sponsors/cool-japan)

**[https://github.com/sponsors/cool-japan](https://github.com/sponsors/cool-japan)**

Your sponsorship helps us:
- Maintain and improve the COOLJAPAN ecosystem
- Keep the entire ecosystem (OxiBLAS, OxiFFT, SciRS2, etc.) 100% Pure Rust
- Provide long-term support and security updates

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

Copyright 2024–2026 COOLJAPAN OU (Team Kitasan)
