# fop-render

Rendering backends for the Apache FOP Rust implementation. Transforms the area tree (from `fop-layout`) into output formats. Currently supports PDF output.

## Usage

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
            <fo:block font-size="14pt">Hello, PDF!</fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"#;

let fo_tree = FoTreeBuilder::new().parse(Cursor::new(xml)).unwrap();
let area_tree = LayoutEngine::new().layout(&fo_tree).unwrap();

let renderer = PdfRenderer::new();
let pdf_doc = renderer.render(&area_tree).unwrap();
let bytes = pdf_doc.to_bytes().unwrap();

std::fs::write("output.pdf", &bytes).unwrap();
```

## Architecture

### `pdf/` - PDF Backend

| File | Lines | Description |
|------|------:|-------------|
| `document.rs` | 286 | `PdfDocument` - PDF object model (pages, fonts, catalog) |
| `writer.rs` | 154 | `PdfRenderer` - area tree to PDF conversion |
| `graphics.rs` | 291 | `PdfGraphics` - PDF content stream operations |

#### PdfDocument
Manages the PDF object graph: catalog, page tree, pages, font dictionaries, and cross-reference table. Serializes to valid PDF 1.4 bytes.

#### PdfRenderer
Walks the area tree and emits PDF operations for each area. Handles text rendering with font selection, color setting, and coordinate transformation (PDF uses bottom-left origin vs FOP's top-left).

#### PdfGraphics
Low-level PDF content stream builder. Supports:
- Text operations: `BT`/`ET`, `Tf`, `Td`, `Tj`
- Graphics state: `q`/`Q`, `cm`
- Path operations: `m`, `l`, `re`, `S`, `f`
- Color: `rg`/`RG` (RGB), `g`/`G` (grayscale)
- Line width: `w`

### `image.rs` - Image Support

| File | Lines | Description |
|------|------:|-------------|
| `image.rs` | 304 | `ImageFormat`, `ImageInfo`, `ImagePlacement` |

Handles image format detection (PNG, JPEG, GIF, TIFF, SVG, BMP), dimension extraction from file headers, and aspect-ratio-preserving placement calculation.

**Total: 1,056 lines across 5 files**

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

23 unit tests covering:
- PDF document structure (catalog, page tree)
- PDF serialization and cross-reference table
- Text rendering with font selection
- Graphics operations (lines, rectangles, colors)
- Image format detection (PNG, JPEG, GIF, TIFF, SVG, BMP)
- Image dimension extraction from file headers
- Aspect ratio calculation and placement

## Dependencies

- `fop-types` (internal)
- `fop-layout` (internal)
- `thiserror` 2.0
- `log` 0.4
