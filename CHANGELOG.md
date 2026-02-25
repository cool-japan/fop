# Changelog

All notable changes to the Apache FOP Rust implementation will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
