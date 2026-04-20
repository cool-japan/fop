# fop-pdf-renderer

[![crates.io](https://img.shields.io/crates/v/fop-pdf-renderer.svg)](https://crates.io/crates/fop-pdf-renderer)
[![docs.rs](https://docs.rs/fop-pdf-renderer/badge.svg)](https://docs.rs/fop-pdf-renderer)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](https://github.com/cool-japan/fop/blob/main/LICENSE)
[![Pure Rust](https://img.shields.io/badge/Pure-Rust-orange.svg)](https://www.rust-lang.org/)

A pure Rust PDF-to-image renderer. Parses PDF documents and rasterizes pages to RGBA/PNG images without any C or native dependencies. Designed to complement the `fop-render` crate by enabling self-contained verification of generated PDF output.

## Features

- **Pure Rust** — zero C/C++ or native library dependencies; runs everywhere Rust compiles
- **PDF parsing** — xref table and xref-stream (PDF 1.5+) cross-reference parsing, indirect object resolution, full page-tree traversal
- **Content stream interpretation** — tokenizes and interprets PDF content operators (path construction, fills, strokes, text, images, graphics state)
- **Stream filter decoding** — FlateDecode (zlib/deflate), DCTDecode (JPEG pass-through), ASCIIHexDecode, and chained filter pipelines
- **Rasterization** — DPI-scalable rendering via [tiny-skia](https://crates.io/crates/tiny-skia); anti-aliased fill and stroke paths, image compositing
- **Font handling** — embedded TrueType font extraction via [ttf-parser](https://crates.io/crates/ttf-parser); ToUnicode CMap decoding; composite (Type0) font support
- **PNG output** — lossless RGBA PNG encoding via the [png](https://crates.io/crates/png) crate
- **CLI tool** — `fop-render-pdf` binary for batch and single-page rendering from the command line
- **Graceful error handling** — structured `PdfRenderError` enum with descriptive variants; no panics on malformed input

## Installation

Add `fop-pdf-renderer` to your project:

```bash
cargo add fop-pdf-renderer
```

Or add it manually to your `Cargo.toml`:

```toml
[dependencies]
fop-pdf-renderer = "0.1"
```

## Usage

### Render a single page to a PNG file

```rust
use fop_pdf_renderer::PdfRenderer;

fn main() -> fop_pdf_renderer::Result<()> {
    let pdf_data = std::fs::read("output.pdf")?;
    let renderer = PdfRenderer::from_bytes(&pdf_data)?;

    println!("Pages: {}", renderer.page_count());

    // Render page 0 at 150 DPI and save as PNG
    renderer.save_as_png(0, "page-0.png", 150.0)?;

    Ok(())
}
```

### Render a page to in-memory RGBA pixels

```rust
use fop_pdf_renderer::PdfRenderer;

fn main() -> fop_pdf_renderer::Result<()> {
    let pdf_data = std::fs::read("document.pdf")?;
    let renderer = PdfRenderer::from_bytes(&pdf_data)?;

    // RasterPage holds width, height, and raw RGBA pixel data
    let page = renderer.render_page(0, 300.0)?;

    println!(
        "Rendered page 0: {}x{} pixels ({} bytes RGBA)",
        page.width,
        page.height,
        page.pixels.len()
    );

    // Encode to PNG bytes in memory
    let png_bytes = page.to_png()?;
    std::fs::write("page-0-300dpi.png", &png_bytes)?;

    Ok(())
}
```

### Render all pages

```rust
use fop_pdf_renderer::PdfRenderer;

fn main() -> fop_pdf_renderer::Result<()> {
    let pdf_data = std::fs::read("multi-page.pdf")?;
    let renderer = PdfRenderer::from_bytes(&pdf_data)?;

    // Returns Vec<Vec<u8>> — one PNG byte vector per page
    let pages = renderer.render_all_pages(150.0)?;

    for (i, png) in pages.iter().enumerate() {
        let path = format!("page-{}.png", i);
        std::fs::write(&path, png)?;
        println!("Saved {}", path);
    }

    Ok(())
}
```

### Working with RasterPage directly

```rust
use fop_pdf_renderer::{PdfRenderer, RasterPage};

fn main() -> fop_pdf_renderer::Result<()> {
    let pdf_data = std::fs::read("input.pdf")?;
    let renderer = PdfRenderer::from_bytes(&pdf_data)?;

    let page: RasterPage = renderer.render_page(0, 72.0)?;

    // Access raw RGBA pixels (row-major, top-to-bottom)
    let (w, h) = (page.width, page.height);
    let rgba: &[u8] = &page.pixels; // length = w * h * 4

    println!("{}x{} image, {} bytes", w, h, rgba.len());

    // Save to disk
    page.save_png("/tmp/rendered.png")?;

    Ok(())
}
```

### Error handling

```rust
use fop_pdf_renderer::{PdfRenderer, PdfRenderError};

fn render(path: &str) -> fop_pdf_renderer::Result<()> {
    let data = std::fs::read(path)?;

    match PdfRenderer::from_bytes(&data) {
        Err(PdfRenderError::Parse(msg)) => eprintln!("Bad PDF structure: {}", msg),
        Err(PdfRenderError::PageNotFound(idx, total)) => {
            eprintln!("Page {} requested but document has {} pages", idx, total)
        }
        Err(e) => eprintln!("Rendering error: {}", e),
        Ok(renderer) => {
            println!("Loaded {} pages", renderer.page_count());
        }
    }

    Ok(())
}
```

## CLI Usage — `fop-render-pdf`

The crate ships a command-line binary named `fop-render-pdf`.

### Installation

```bash
cargo install fop-pdf-renderer --bin fop-render-pdf
```

Or build from source:

```bash
cargo build --release -p fop-pdf-renderer
# Binary is at: target/release/fop-render-pdf
```

### Synopsis

```
fop-render-pdf <input.pdf> <output.png> [OPTIONS]

Options:
  --page N     Page to render (0-indexed, default: 0)
  --dpi N      Output resolution in DPI (default: 150)
  --all        Render all pages (output must contain %d, e.g. page-%d.png)
```

### Examples

```bash
# Render the first page at the default 150 DPI
fop-render-pdf document.pdf page-0.png

# Render page 2 (0-indexed) at 300 DPI
fop-render-pdf document.pdf page-2.png --page 2 --dpi 300

# Render all pages to page-0.png, page-1.png, …
fop-render-pdf document.pdf page-%d.png --all

# Render all pages at 72 DPI
fop-render-pdf document.pdf page-%d.png --all --dpi 72
```

The tool prints progress to stderr and exits with a non-zero status on error, making it safe to use in shell pipelines and CI scripts.

## Public API Summary

| Item | Description |
|---|---|
| `PdfRenderer` | High-level entry point — parse and render PDF documents |
| `PdfRenderer::from_bytes(data)` | Parse a PDF from a `&[u8]` slice |
| `PdfRenderer::page_count()` | Return the number of pages |
| `PdfRenderer::render_page(idx, dpi)` | Render one page to a `RasterPage` |
| `PdfRenderer::save_as_png(idx, path, dpi)` | Render and write a PNG file |
| `PdfRenderer::render_all_pages(dpi)` | Render every page, return `Vec<Vec<u8>>` (PNG bytes) |
| `RasterPage` | Rasterized page: `width`, `height`, `pixels` (RGBA) |
| `RasterPage::to_png()` | Encode pixels as PNG, returning `Vec<u8>` |
| `RasterPage::save_png(path)` | Encode and write PNG to a file path |
| `PdfRenderError` | Structured error enum (`Parse`, `Unsupported`, `PageNotFound`, `Font`, `Image`, `Io`, `Decompress`) |
| `Result<T>` | Type alias for `std::result::Result<T, PdfRenderError>` |

## Feature Flags

This crate currently has no optional Cargo feature flags. All capabilities are enabled by default.

## Architecture

The `fop-pdf-renderer` crate is a standalone component within the FOP workspace, with no internal dependencies on `fop-types`, `fop-core`, or `fop-layout`. It can be used independently for any PDF rendering task.

The crate comprises approximately **6,200 lines** across 10 Rust source files (5,040 lines of code), contributing to the FOP workspace total of **15,357 lines**.

Key internal modules:

| Module | Responsibility |
|---|---|
| `parser` | PDF cross-reference table/stream parsing, object resolution |
| `content` | Content stream tokenization and operator interpretation |
| `filters` | Stream decompression (FlateDecode, DCTDecode, ASCIIHexDecode) |
| `fonts` | TrueType font extraction, CMap decoding, composite font support |
| `rasterizer` | Page rasterization via tiny-skia (paths, fills, strokes, images) |
| `renderer` | High-level `PdfRenderer` API orchestrating parse → render → encode |

## Integration with fop-render

`fop-pdf-renderer` is designed to work alongside `fop-render`, the XSL-FO to PDF backend in the [fop](https://github.com/cool-japan/fop) workspace. A typical verification workflow is:

```rust
fn verify_pdf(pdf_bytes: &[u8]) -> fop_pdf_renderer::Result<()> {
    // Render the generated PDF back to an image for visual inspection
    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(pdf_bytes)?;
    let image = renderer.render_page(0, 150.0)?;
    image.save_png("verification.png")?;

    Ok(())
}
```

## Testing

The crate includes a comprehensive test suite with **233 tests**:

```bash
# Run all tests
cargo nextest run -p fop-pdf-renderer --all-features

# Or with standard cargo test
cargo test -p fop-pdf-renderer --all-features
```

Tests cover PDF parsing, cross-reference resolution, content stream interpretation, stream filter decoding, font handling, rasterization correctness, and end-to-end render-to-PNG pipelines.

## Dependencies

| Crate | Purpose |
|---|---|
| `thiserror` 2.0 | Derive-based error type definitions |
| `log` 0.4 | Structured logging facade |
| `flate2` 1.1 | FlateDecode (zlib/deflate) stream decompression |
| `ttf-parser` 0.25 | TrueType/OpenType font outline parsing |
| `png` 0.18 | PNG encoding |
| `jpeg-decoder` 0.3 | DCT/JPEG image decoding |
| `tiny-skia` 0.12 | Software 2D rasterizer (paths, fills, strokes, anti-aliasing) |

All dependencies are pure Rust with no C or Fortran build dependencies.

## Related Crates

| Crate | Description |
|---|---|
| [`fop`](https://crates.io/crates/fop) | Top-level FOP workspace re-export |
| [`fop-types`](https://crates.io/crates/fop-types) | XSL-FO type definitions and property value types |
| [`fop-core`](https://crates.io/crates/fop-core) | XSL-FO XML parser and document model |
| [`fop-layout`](https://crates.io/crates/fop-layout) | Page layout engine (area tree, pagination, breaking) |
| [`fop-render`](https://crates.io/crates/fop-render) | PDF/PostScript/SVG output backends |
| [`fop-cli`](https://crates.io/crates/fop-cli) | Command-line interface for the FOP pipeline |
| [`fop-wasm`](https://crates.io/crates/fop-wasm) | WebAssembly bindings for browser-based rendering |
| [`fop-python`](https://crates.io/crates/fop-python) | Python bindings via PyO3/Maturin |

## License

Copyright 2024-2026 COOLJAPAN OU (Team Kitasan)

Licensed under the [Apache License, Version 2.0](https://github.com/cool-japan/fop/blob/main/LICENSE).

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.
