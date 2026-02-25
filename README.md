# FOP - Rust Implementation

A high-performance Rust reimplementation of [Apache FOP](https://xmlgraphics.apache.org/fop/) (Formatting Objects Processor), translating XSL-FO documents to PDF and other output formats.

## Project Status

**Phase 5-6 Enhanced** - Production ready with comprehensive features and testing.

| Phase | Status | Description |
|-------|--------|-------------|
| Phase 1: Foundation | ✅ Complete | Core types, property system, FO tree parsing |
| Phase 2: Basic Layout | ✅ Complete | Area tree, block/inline layout, PDF output |
| Phase 3: Advanced Layout | ✅ Complete | Knuth-Plass, tables, lists, graphics, images |
| Phase 4: Integration | ✅ Complete | Multi-page breaking, engine integration |
| Phase 5: Advanced Features | ✅ Complete | Image rendering, links, bookmarks, font embedding |
| Phase 6: Optimization | 🔄 85% Complete | Performance (✅), streaming (✅), testing (✅) |

**Current stats:** 52 integration tests + 383 unit tests, 0 warnings, **10-1200x faster than Java FOP**

## Documentation

📚 **[Comprehensive documentation available in `docs/`](docs/)**

- **[I18N_CAPABILITIES.md](docs/I18N_CAPABILITIES.md)** - Internationalization guide (Japanese, Chinese, Korean, Arabic, etc.)
- **[LIMITATIONS.md](docs/LIMITATIONS.md)** - Current limitations and migration guide from Java FOP
- **[docs/README.md](docs/README.md)** - Complete documentation index

## What Works

The complete pipeline transforms XSL-FO XML documents into valid PDF files:

```
XSL-FO XML --> FO Tree --> Area Tree --> PDF
  (parse)     (layout)    (render)
```

### Supported XSL-FO Elements
- `fo:root`, `fo:layout-master-set`, `fo:simple-page-master`
- `fo:region-body`, `fo:region-before`, `fo:region-after`
- `fo:page-sequence`, `fo:flow`, `fo:static-content`
- `fo:block`, `fo:inline`
- `fo:table`, `fo:table-body`, `fo:table-row`, `fo:table-cell`
- `fo:list-block`, `fo:list-item`, `fo:list-item-label`, `fo:list-item-body`
- `fo:external-graphic`, `fo:basic-link` (structure only)

### Supported Features
- 294 XSL-FO 1.1 properties with inheritance
- Font metrics (Helvetica, Times-Roman)
- Knuth-Plass optimal line breaking
- Table layout (fixed, proportional, auto column widths)
- List layout (9 marker styles: disc, circle, square, decimal, alpha, roman...)
- Multi-page breaking with overflow detection
- PDF generation (valid, viewable, text-extractable)
- Graphics operations (borders, rectangles, lines, colors)
- Image format detection and aspect ratio calculation

## Quick Start

```bash
# Build
cargo build --release

# Run tests (NO WARNINGS policy)
cargo test
cargo clippy --all-targets -- -D warnings

# Run examples
cargo run --example hello_pdf              # Basic PDF
cargo run --example styled_pdf             # Colors and fonts
cargo run --example comprehensive_demo     # Phase 3 features
cargo run --example tables_lists_demo      # Tables + lists end-to-end
```

### Generate a PDF from XSL-FO

```rust
use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use fop_render::PdfRenderer;
use std::io::Cursor;

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
let fo_tree = FoTreeBuilder::new().parse(Cursor::new(xml)).unwrap();
let area_tree = LayoutEngine::new().layout(&fo_tree).unwrap();
let pdf_doc = PdfRenderer::new().render(&area_tree).unwrap();
let pdf_bytes = pdf_doc.to_bytes().unwrap();

std::fs::write("output.pdf", &pdf_bytes).unwrap();
```

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
```

## Workspace Structure

```
fop/
├── Cargo.toml                  # Workspace root
├── crates/
│   ├── fop-types/              # Shared types (Length, Color, Rect, errors)
│   ├── fop-core/               # FO tree parsing & property system
│   ├── fop-layout/             # Layout engine (block, inline, table, list)
│   └── fop-render/             # Rendering backends (PDF)
├── examples/                   # 10 runnable examples
└── benches/                    # Performance benchmarks
```

### Dependency Graph

```
fop-types          (no internal deps)
    |
    +-- fop-core   (+ quick-xml)
    |       |
    |       +-- fop-layout
    |               |
    |               +-- fop-render
    +---------------+
```

## Examples

| Example | Description |
|---------|-------------|
| `basic_types` | Length, Color, Geometry type usage |
| `properties` | Property system and inheritance |
| `parse_fo_document` | Full XSL-FO document parsing |
| `validation` | Element nesting validation |
| `shorthand` | Shorthand property expansion (margin, padding) |
| `layout_demo` | FO tree to area tree transformation |
| `hello_pdf` | Minimal PDF generation |
| `styled_pdf` | PDF with colors and fonts |
| `comprehensive_demo` | All Phase 3 features combined |
| `tables_lists_demo` | Tables and lists end-to-end |

## Dependencies

**Production** (3 crates, minimal by design):
- `quick-xml` 0.39 - XML parsing
- `thiserror` 2.0 - Error handling
- `log` 0.4 - Logging facade

**Development:**
- `criterion` 0.5 - Benchmarks
- `env_logger` 0.11 - Test logging

## Performance Targets

| Metric | Java FOP | Rust FOP (Target) |
|--------|----------|-------------------|
| Parse 1000-page doc | ~500ms | <50ms |
| Memory per page | ~50KB | <5KB |
| Binary size | 15MB+ JAR | <10MB stripped |
| Startup time | ~2000ms (JVM) | <10ms |

## Development

### NO WARNINGS Policy

This project enforces zero compiler warnings and zero clippy warnings:

```bash
cargo clippy --all-targets -- -D warnings
cargo test | tail
```

### Architecture Principles

- **Zero-copy parsing**: `Cow<'static, str>` and arena allocation throughout
- **Arena allocation**: Index-based tree nodes (no `Rc<RefCell<>>`)
- **Static dispatch**: Enum dispatch over trait objects where possible
- **Minimal dependencies**: Only 3 production crates

## Reference

- Based on [Apache FOP](https://xmlgraphics.apache.org/fop/) (Java)
- Implements [XSL-FO 1.1 Specification](https://www.w3.org/TR/xsl11/)
- Java reference: 1566+ files across 8 Maven modules

## License

Apache License 2.0
