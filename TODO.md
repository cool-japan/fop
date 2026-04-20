# TODO - Apache FOP Rust

## Phase 5: Advanced Features

### Image Rendering
- [x] Add `png` crate dependency for PNG decoding
- [x] Add `jpeg-decoder` crate dependency for JPEG decoding
- [x] Implement PDF XObject image embedding
- [x] Wire `fo:external-graphic` through rendering pipeline
- [x] Support width/height/content-width/content-height properties
- [x] Test with various image sizes and formats

### Background Colors and Borders
- [x] Render background-color on `fo:block` areas
- [x] Render background-color on `fo:table-cell` areas
- [x] Implement 4-side border rendering (top/right/bottom/left)
- [x] Support border styles: solid, dashed, dotted
- [x] Wire `PdfGraphics.draw_borders()` into area rendering

### Links and Bookmarks
- [x] Implement internal links (`fo:basic-link` with `internal-destination`)
- [x] Implement external links (`fo:basic-link` with `external-destination`)
- [x] Generate PDF annotations for links
- [x] Implement PDF outline (bookmarks) from `fo:bookmark-tree`
- [x] Support `fo:page-number-citation`

### Font Embedding
- [x] Add `ttf-parser` crate for TrueType/OpenType parsing
- [x] Implement font metrics extraction from TTF files
- [x] Implement font subsetting (embed only used glyphs)
- [x] Add Courier, Symbol, ZapfDingbats to built-in fonts
- [x] Wire custom fonts through the layout pipeline
- [x] **Unicode font support** - Extended from ASCII (32-126) to full Unicode (u32)
- [x] **ToUnicode CMap generation** - Proper PDF Unicode character mapping
- [x] **Smart text encoding** - Hex strings for Unicode, literal for ASCII
- [x] **Font selection from FO properties** - Read font-family and use embedded fonts
- [x] **Font configuration system** - Map font names to font files (font_config.rs)
- [x] **System font discovery** - Automatic font loading from system

## Phase 6: Optimization and Polish

### Performance
- [x] Profile hot paths with `criterion` benchmarks
- [x] Implement streaming mode for large documents (>1000 pages)
- [x] Optimize memory usage in area tree
- [x] Consider parallel rendering for multi-page documents (ParallelRenderer in parallel.rs)

### Standards Compliance
- [x] PDF/A compliance for archival output (compliance.rs, --pdfa CLI flag)
- [x] Validate against XSL-FO 1.1 conformance test suite (60+ conformance tests covering all major XSL-FO 1.1 features)
- [x] Handle all shorthand property expansions (margin, padding, border, font, background - all implemented with 80+ tests)
- [x] Complete property initial value defaults (all 295 properties have initial values verified)

### Additional Output Formats
- [x] SVG renderer backend (fop-render/src/svg/)
- [x] PNG/JPEG raster output (fop-render/src/raster/ via resvg)
- [x] Plain text output (fop-render/src/text/)

### PDF Renderer (Pure Rust - Self-Verification)
**Goal:** Replace `pdftoppm` dependency with pure Rust PDF-to-image renderer

**Motivation:**
- Enable self-verification without C++ dependencies (poppler)
- Complete the cycle: Generate PDF → Render PDF → Verify output
- Pure Rust stack (no external tools needed)
- WASM-compatible for browser-based testing
- Better integration testing capabilities

**New Crate:** `fop-pdf-renderer`
```
fop/
└── crates/
    └── fop-pdf-renderer/
        ├── src/
        │   ├── lib.rs         - Public API
        │   ├── parser.rs      - PDF structure parser
        │   ├── content.rs     - Content stream interpreter
        │   ├── graphics.rs    - Graphics state machine
        │   ├── text.rs        - Text rendering
        │   ├── font.rs        - Font handling (TrueType, Type 0, CID)
        │   ├── image.rs       - Image XObject handling
        │   └── rasterizer.rs  - Pixel output (via tiny-skia or similar)
        └── Cargo.toml
```

**Implementation Tasks:**
- [x] **Phase 1: PDF Parser**
  - [x] Parse PDF structure (objects, xref table, trailer)
  - [x] Extract page tree
  - [x] Parse content streams
  - [x] Handle compressed streams (FlateDecode)

- [x] **Phase 2: Graphics Engine**
  - [x] Implement PDF graphics state machine
  - [x] Path construction and stroking
  - [x] Color spaces (DeviceRGB, DeviceGray)
  - [x] Transformation matrices (CTM)

- [x] **Phase 3: Text Rendering**
  - [x] Text state operators (Tf, Tm, Td, Tj, TJ)
  - [x] Font loading (TrueType from embedded streams)
  - [x] Glyph rasterization
  - [x] UTF-16BE text decoding
  - [x] CID font support (Type 0, CIDFontType2)
  - [x] ToUnicode CMap parsing

- [x] **Phase 4: Rasterization**
  - [x] Render to RGBA bitmap (tiny-skia)
  - [x] PNG encoding (png crate)
  - [x] DPI/scale support
  - [x] Anti-aliasing

- [x] **Phase 5: Integration**
  - [x] Public API: `PdfRenderer::from_bytes(pdf_data)`
  - [x] Public API: `renderer.render_page(page_num, dpi) -> RasterPage`
  - [x] Public API: `renderer.save_as_png(page, path)`
  - [x] CLI tool: `fop-render-pdf input.pdf output.png` (src/bin/fop_render_pdf.rs)
  - [x] Integration with auto-verify workflow
    - [x] Replace external-tool tests in `regression_tests.rs` with pure-Rust verifications
    - [x] Expand `verify_tests.rs` fixture coverage from 4 → 17 fixtures
    - [x] Add `--render-verify` CLI flag wired to `fop-pdf-renderer`
    - [x] Add `.github/workflows/ci.yml` running `fmt`/`clippy`/`nextest`/`doc` on Linux+macOS+Windows without poppler/gs
    - [x] Add `PdfRenderer::extract_text(page_index) -> Result<String>` to `fop-pdf-renderer` (prerequisite for regression tests)
- [x] **Phase 6: Glyph Outline Rendering** (completed 2026-04-20)
  - [x] Create `glyph.rs` with `OutlinePathBuilder` (ttf-parser → tiny-skia bridge) and standard-14 substitute font discovery with TTC index support
  - [x] Parse `/CIDToGIDMap` binary stream in `font.rs` (replaces `HashMap::new()` placeholder); add `SimpleEncoding` enum + WinAnsi/Standard tables
  - [x] Wire `PageRasterizer.font_cache` with `LoadedFontExt` struct; add `base_font` field to `LoadedFont`; remove `#[allow(dead_code)]`
  - [x] Replace filled-rectangle placeholder at `rasterizer.rs:160-178` with real ttf-parser outline fill via tiny-skia

**Dependencies (Pure Rust):**
- `pdf` or `lopdf` - PDF parsing (evaluate both)
- `tiny-skia` or `raqote` - 2D graphics rasterization
- `ttf-parser` - TrueType font parsing (already used)
- `png` - PNG encoding (already used)
- `image` - Image manipulation (already used)

**Benefits:**
- ✅ Auto-verification without `pdftoppm` (no C++ dependency)
- ✅ Cross-platform (Windows, macOS, Linux, WASM)
- ✅ Consistent rendering across platforms
- ✅ Better error messages (know our own format)
- ✅ Faster CI/CD (no external tool installation)
- ✅ Self-contained testing infrastructure

**Target Usage:**
```rust
use fop_pdf_renderer::PdfRenderer;

// Load PDF
let pdf_data = std::fs::read("output.pdf")?;
let renderer = PdfRenderer::from_bytes(&pdf_data)?;

// Render to image
let image = renderer.render_page(0, 150)?; // page 0, 150 DPI

// Save as PNG
renderer.save_as_png(0, "output.png")?;

// Auto-verification
assert!(image.contains_text("請求書"));
assert!(!image.contains_glyph(".notdef"));
```

**Estimated Effort:** 2-3 weeks for basic rendering, 4-6 weeks for production quality

**Priority:** Medium (useful but not blocking current PDF generation work)

### Advanced Layout
- [x] Headers and footers via `fo:static-content`
- [x] Page masters with multiple regions (SidebarStart/SidebarEnd AreaType)
- [x] Table column/row spanning (`number-columns-spanned`, `number-rows-spanned`)
- [x] Widow/orphan control (fully implemented with page breaking constraints, 8 comprehensive tests)
- [x] Keep-together / keep-with-next / keep-with-previous
- [x] Float support (`fo:float`) (FloatManager in layout engine)
- [x] Footnotes (`fo:footnote`) (inline reference marks + page-bottom body)
- [x] Leaders (`fo:leader`)

## Technical Debt
- [x] Remove `#![allow(dead_code)]` from `fop-core` when APIs stabilize (removed from lib.rs, targeted annotations added where needed)
- [x] Add `#[must_use]` annotations where appropriate (Result types are already #[must_use])
- [x] Increase test coverage for edge cases in property inheritance (added 14 comprehensive edge case tests covering: explicit inherit keyword, percentages, enums, lists, deep chains, border properties, caching, 4-level inheritance, partial overrides)
- [x] Add integration tests with real-world XSL-FO documents (4 new tests: invoice, report, form, Unicode)
- [x] Add fuzzing targets for XML parsing (fuzz_xml_parser and fuzz_property_parser)
- [x] Update `README.md` with accurate test count, file/LoC statistics, and new feature documentation (`--render-verify` CLI flag, `PdfRenderer::extract_text` API, `fop-pdf-renderer` crate, `fop-render-pdf` binary, `.github/workflows/ci.yml` CI workflow) (completed 2026-04-20)

## Version 0.1.1 Fixes and Enhancements (completed 2026-04-20)
- [x] **PyO3 0.28 migration** — updated `fop-python` to PyO3 0.28 and ported to the new `Python::attach` API (replaces `Python::with_gil`)
- [x] **macOS build.rs linker fix** — added `build.rs` to `fop-python` to resolve PyO3 ABI3 linker issues on macOS (`-undefined dynamic_lookup` flag)
- [x] **fop-wasm invalid XML test fixes** — corrected WASM binding error-handling tests to match updated error message format for malformed XML inputs
