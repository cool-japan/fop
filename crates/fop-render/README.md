# fop-render

[![Crates.io](https://img.shields.io/crates/v/fop-render.svg)](https://crates.io/crates/fop-render)
[![Documentation](https://docs.rs/fop-render/badge.svg)](https://docs.rs/fop-render)
[![License](https://img.shields.io/crates/l/fop-render.svg)](LICENSE)

Multi-format rendering backends for the COOLJAPAN FOP ecosystem. Transforms the area tree produced by `fop-layout` into **7 output formats** across 6 renderer structs — PDF, SVG, PostScript, plain text, PNG, and JPEG — plus streaming and parallel PDF renderers for large-scale workloads.

**Version:** 0.1.1 — **Released:** 2026-04-20

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
fop-render = "0.1"
```

To include PNG/JPEG raster output (enabled by default):

```toml
[dependencies]
fop-render = { version = "0.1", features = ["raster"] }
```

To disable raster and keep only vector/text formats:

```toml
[dependencies]
fop-render = { version = "0.1", default-features = false }
```

## Supported Output Formats

| Format | Renderer | Feature | Description |
|--------|----------|---------|-------------|
| **PDF 1.4** | `PdfRenderer` | always | Full PDF with fonts, images, encryption, bookmarks |
| **PDF streaming** | `StreamingPdfRenderer` | always | Incremental PDF generation for large documents |
| **Parallel PDF** | `ParallelRenderer` | always | Multi-threaded PDF rendering |
| **SVG** | `SvgRenderer` | always | Scalable Vector Graphics output |
| **PostScript** | `PsRenderer` | always | PS-Adobe-3.0 compliant output |
| **Plain Text** | `TextRenderer` | always | Extracted text with layout preservation |
| **PNG** (raster) | `RasterRenderer` + `RasterFormat::Png` | `raster` | Rasterized page images via SVG→resvg pipeline |
| **JPEG** (raster) | `RasterRenderer` + `RasterFormat::Jpeg` | `raster` | Rasterized page images via SVG→resvg pipeline |

## Usage Examples

### PDF Output

```rust
use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use fop_render::PdfRenderer;
use std::io::Cursor;

fn render_pdf() -> Result<(), Box<dyn std::error::Error>> {
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
            <fo:block font-size="14pt">Hello, PDF!</fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;

    let fo_tree = FoTreeBuilder::new().parse(Cursor::new(xml))?;
    let area_tree = LayoutEngine::new().layout(&fo_tree)?;

    let renderer = PdfRenderer::new();
    let pdf_doc = renderer.render(&area_tree)?;
    let bytes = pdf_doc.to_bytes()?;

    std::fs::write("output.pdf", &bytes)?;
    Ok(())
}
```

### SVG Output

```rust
use fop_render::SvgRenderer;

fn render_svg(area_tree: &fop_layout::AreaTree) -> Result<(), Box<dyn std::error::Error>> {
    let renderer = SvgRenderer::new();
    let svg_doc = renderer.render(area_tree)?;
    let svg_string = svg_doc.to_string();

    std::fs::write("output.svg", svg_string)?;
    Ok(())
}
```

### PostScript Output

```rust
use fop_render::PsRenderer;

fn render_ps(area_tree: &fop_layout::AreaTree) -> Result<(), Box<dyn std::error::Error>> {
    let renderer = PsRenderer::new();
    let ps_doc = renderer.render(area_tree)?;
    let ps_bytes = ps_doc.to_bytes()?;

    std::fs::write("output.ps", &ps_bytes)?;
    Ok(())
}
```

### Plain Text Output

```rust
use fop_render::TextRenderer;

fn render_text(area_tree: &fop_layout::AreaTree) -> Result<(), Box<dyn std::error::Error>> {
    let renderer = TextRenderer::new();
    let text_output = renderer.render(area_tree)?;

    std::fs::write("output.txt", text_output)?;
    Ok(())
}
```

### Streaming PDF (Large Documents)

```rust
use fop_render::StreamingPdfRenderer;
use std::fs::File;

fn render_streaming(area_tree: &fop_layout::AreaTree) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create("large_output.pdf")?;
    let mut renderer = StreamingPdfRenderer::new(file);
    renderer.render(area_tree)?;
    renderer.finish()?;
    Ok(())
}
```

### Raster Output (PNG / JPEG)

Requires the `raster` feature (enabled by default).

```rust
use fop_render::{RasterRenderer, RasterFormat};

fn render_png(area_tree: &fop_layout::AreaTree) -> Result<(), Box<dyn std::error::Error>> {
    let renderer = RasterRenderer::new(RasterFormat::Png);
    let pages = renderer.render(area_tree)?;

    for (i, page_bytes) in pages.iter().enumerate() {
        std::fs::write(format!("page_{}.png", i + 1), page_bytes)?;
    }
    Ok(())
}
```

## Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `raster` | **yes** | PNG/JPEG raster output — enables `resvg`, `usvg`, `tiny-skia`, `jpeg-encoder` |

## Architecture

**Total: 15,357 lines across 24 files**

| File | Lines | Description |
|------|------:|-------------|
| `pdf/document/mod.rs` | 1,484 | `PdfDocument` core object model |
| `svg/mod.rs` | 1,361 | SVG renderer |
| `pdf/graphics.rs` | 1,239 | PDF content stream operations |
| `ps/mod.rs` | 1,206 | PostScript renderer |
| `pdf/security.rs` | 1,071 | RC4/AES encryption |
| `pdf/validator.rs` | 1,059 | PDF structural validation |
| `pdf/writer.rs` | 1,013 | `PdfRenderer` — area tree → PDF conversion |
| `pdf/font.rs` | 869 | Font embedding & subsetting |
| `pdf/image.rs` | 699 | PDF image XObject |
| `pdf/streaming.rs` | 698 | Streaming PDF renderer |
| `text/mod.rs` | 593 | Plain text renderer |
| `pdf/document/page.rs` | 577 | PDF page helpers |
| `pdf/outline.rs` | 531 | Bookmarks / outlines |
| `raster/mod.rs` | 503 | PNG/JPEG raster via SVG→resvg |
| `pdf/compliance.rs` | 369 | PDF/A compliance, XMP metadata |
| `image.rs` | 343 | Image format detection |
| `pdf/font_config.rs` | 324 | System font discovery |
| `pdf/cidfont.rs` | 305 | CIDFont for CJK/Unicode |
| `lib.rs` | 288 | Public API surface |
| `parallel.rs` | 254 | Multi-threaded PDF rendering |
| `pdf/document/gradient.rs` | 205 | Gradient support |
| `pdf/document/types.rs` | 176 | Type definitions |
| `pdf/document/outline.rs` | 162 | Outline serialization |
| `pdf/mod.rs` | 28 | Module re-exports |

## Public API

### Renderers

| Struct | Purpose |
|--------|---------|
| `PdfRenderer` | Full-featured PDF 1.4 generation |
| `SvgRenderer` | SVG vector output |
| `PsRenderer` | PostScript (PS-Adobe-3.0) output |
| `TextRenderer` | Plain text extraction |
| `RasterRenderer` | PNG/JPEG raster output (behind `raster` feature) |
| `StreamingPdfRenderer` | Incremental PDF for large documents |
| `ParallelRenderer` | Multi-threaded PDF rendering |

### Document Types

`PdfDocument`, `SvgDocument`, `SvgGraphics`, `PsDocument`, `PdfGraphics`, `RasterFormat`

### PDF Features

`PdfSecurity`, `PdfPermissions`, `EncryptionAlgorithm`, `EncryptionDict`, `PdfCompliance`, `PdfValidator`, `ValidationResult`, `FontConfig`

### Image Support

`ImageFormat`, `ImageInfo`, `ImagePlacement`

## Generated PDF Structure

```
%PDF-1.4
1 0 obj  << /Type /Catalog /Pages 2 0 R >>
2 0 obj  << /Type /Pages /Kids [...] /Count N >>
3 0 obj  << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>
4 0 obj  << /Type /Page /Parent 2 0 R /Contents 5 0 R >>
5 0 obj  << /Length ... >> stream ... endstream
...
xref
trailer
%%EOF
```

## Tests

23+ unit tests covering:

- PDF document structure (catalog, page tree)
- PDF serialization and cross-reference table
- Text rendering with font selection
- Graphics operations (lines, rectangles, colors)
- Image format detection (PNG, JPEG, GIF, TIFF, SVG, BMP)
- Image dimension extraction from file headers
- Aspect ratio calculation and placement
- SVG, PostScript, plain text output
- Encryption (RC4, AES)
- PDF structural validation

## Dependencies

### Required

| Crate | Version | Purpose |
|-------|---------|---------|
| `fop-types` | workspace | FO type definitions |
| `fop-layout` | workspace | Area tree input |
| `fop-core` | workspace | Core parsing / tree building |
| `thiserror` | 2.0 | Error derive macro |
| `log` | 0.4 | Logging facade |
| `png` | 0.18 | PNG decoding |
| `jpeg-decoder` | 0.3 | JPEG decoding |
| `flate2` | 1.1 | Deflate compression |
| `ttf-parser` | 0.25 | TrueType/OpenType font parsing |
| `aes` | 0.8 | AES encryption |
| `cbc` | 0.1 | CBC block cipher mode |
| `sha2` | 0.10 | SHA-256 hashing |
| `md-5` | 0.10 | MD5 hashing (PDF encryption) |

### Optional (raster feature)

| Crate | Version | Purpose |
|-------|---------|---------|
| `resvg` | 0.47 | SVG rendering to raster |
| `usvg` | 0.47 | SVG tree simplification |
| `tiny-skia` | 0.12 | 2D rasterization backend |
| `jpeg-encoder` | 0.7 | JPEG encoding |

## Related Crates

| Crate | Description |
|-------|-------------|
| [`fop-types`](https://crates.io/crates/fop-types) | XSL-FO type definitions and property values |
| [`fop-core`](https://crates.io/crates/fop-core) | XML parsing and FO tree construction |
| [`fop-layout`](https://crates.io/crates/fop-layout) | Layout engine — FO tree → area tree |
| [`fop-cli`](https://crates.io/crates/fop-cli) | Command-line interface |
| [`fop-pdf-renderer`](https://crates.io/crates/fop-pdf-renderer) | Standalone PDF rendering binary |
| [`fop`](https://crates.io/crates/fop) | Umbrella crate |

## License

Apache-2.0

## Author

COOLJAPAN OU (Team Kitasan)

Copyright 2024-2026 COOLJAPAN OU
