# Changelog

All notable changes to the Apache FOP Rust implementation will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.1] - 2026-04-20

### Added

#### PDF Rendering
- **Glyph Outline Rendering** - Real TrueType/OpenType glyph outlines via `ttf-parser` and `tiny-skia`
  - `OutlinePathBuilder` converts ttf-parser contours to tiny-skia paths
  - Standard-14 substitute font discovery with OS-specific directory search and TTC index support
  - `OnceLock`-based font cache for efficient repeated rendering
  - Correct Y-flip transform chain for proper glyph orientation
- **PDF Text Extraction** - `PdfRenderer::extract_text(page_index)` and `extract_all_text()` with ToUnicode CMap decoding
- **`SimpleDocumentBuilder`** - Lightweight programmatic PDF builder (no FO pipeline required); supports all 14 standard PDF Type 1 fonts
- **`--render-verify` CLI flag** - Re-parses and rasterizes the generated PDF as a self-verification step; exits non-zero on failure

#### CI / Testing
- Matrix GitHub Actions CI (Linux / macOS / Windows) with a separate bindings job (maturin + wasm-pack on Ubuntu)
- Integration test modules `verify_tests` and `regression_tests` now fully wired in; all 17 `.fo` fixtures covered by the auto-verify pipeline

#### Dependencies
- Replaced `flate2` with `oxiarc-deflate` in `fop-pdf-renderer` and `fop-render` (Pure Rust policy)
- Added npm publishing workflow for WASM bindings

### Changed
- **pyo3 upgraded to 0.28** - `Python::with_gil` → `Python::attach` across all `fop-python` tests
- `fop-python` dev-dependencies now include `pyo3` with `auto-initialize` feature for native test support
- Refactored formatting and method-chaining style across `fop-cli`, `fop-pdf-renderer`, `fop-render`, and integration tests for improved readability

### Fixed
- **macOS build fix** - `crates/fop-python/build.rs` emits `-undefined dynamic_lookup` and explicit Python library link args for `cargo test` (without maturin) on macOS
- `content.rs` correctness fix: `show_string` now falls back to `simple_byte_to_char(encoding, cid as u8)` for non-composite fonts when `cid_to_char(cid)` returns `None`, preventing wrong glyph IDs for standard-14 Helvetica text

## [0.1.0] - 2026-02-17

### Added - Phase 1 Complete ✅

#### Security
- **PDF Encryption Support** - Full RC4-128 encryption implementation
  - Owner and user password support
  - Permission flags: print, copy, edit, annotations
  - Integrated directly into `PdfDocument::to_bytes()`
  - Content streams properly encrypted (not visible in plaintext)
  - CLI flags: `-o`/`--owner-password`, `-u`/`--user-password`, `--noprint`, `--nocopy`, `--noedit`, `--noannotations`

#### Performance
- **Parallel Rendering Infrastructure** - `--jobs N` support
  - `ParallelRenderer` implementation with thread auto-detection
  - Falls back to sequential rendering (safe baseline)
  - Ready for future multi-threaded optimization

#### Core Features
- XSL-FO 1.1 parsing (29 elements, 294 properties)
- Knuth-Plass line breaking algorithm
- Multi-page document generation
- Table and list layout
- Font embedding (Type 0 composite fonts, CIDFontType2)
- Multi-format output: PDF, SVG, PostScript, PNG, JPEG, Text
- i18n support for 16+ languages with automatic font fallback
- Apache FOP CLI compatibility
- Stdin/stdout support (`-` filename)
- `--version` flag with detailed info

#### Language Bindings
- WASM bindings for browser usage (complete)
- Python bindings via PyO3 (complete)

#### Testing
- 616 comprehensive tests (all passing)
- 3 new encryption tests
- 3 new parallel rendering tests
- Zero compiler warnings
- Zero clippy warnings

#### Documentation
- 23 working examples
- Comprehensive README
- API documentation (rustdoc)
- Implementation blueprint (fop.md)

### Changed
- Encryption now applied during PDF serialization (not post-processing)
- Improved CLI help text and usage examples
- Enhanced error messages with context

### Fixed
- Clippy warnings in security.rs (borrowed expression)

## [Unreleased] - Future Work

### Planned Features

#### Phase 2: Advanced Security
- AES-256 encryption (PDF 2.0)
- Digital signatures
- Certificate-based encryption

#### Phase 3: Accessibility
- Tagged PDF (PDF/UA)
- Screen reader support
- Alternative text for images

#### Phase 4: Optimization
- True parallel page rendering (requires thread-safe font manager)
- Streaming mode for very large documents
- Font cache implementation (`--cache`, `--flush` backends)

#### Phase 5: Extended Features
- XSLT transformation (complete processor)
- PDF forms support
- Advanced annotations
- Incremental updates
- Pure Rust PDF renderer for testing

---

[0.1.0]: https://github.com/apache-fop-rust/fop/releases/tag/v0.1.0
