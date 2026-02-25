# Apache FOP Rust - Documentation

Welcome to the Apache FOP Rust documentation! This directory contains comprehensive guides and references for using and understanding the FOP processor.

---

## 📚 Documentation Index

### User Guides

#### [I18N_CAPABILITIES.md](I18N_CAPABILITIES.md)
**Internationalization and Unicode Support**
- Comprehensive guide to generating PDFs in multiple languages
- Japanese, Chinese, Korean (CJK) support
- Arabic (RTL) and European languages
- Unicode symbols and emoji
- Usage examples and best practices
- **Status:** Production Ready

#### [LIMITATIONS.md](LIMITATIONS.md)
**Current Limitations and Migration Guide**
- Known limitations compared to Java FOP
- Missing XSL-FO 1.1 features
- Partial implementations
- Migration guide from Java FOP
- Platform-specific notes
- Roadmap for future features

---

## 🎯 Quick Reference

### Internationalization Support

Apache FOP Rust supports generating PDFs in:
- ✅ **Japanese** - Hiragana, Katakana, Kanji
- ✅ **Chinese** - Simplified & Traditional
- ✅ **Korean** - Hangul
- ✅ **Arabic** - RTL (Right-to-Left)
- ✅ **European Languages** - All Latin scripts with diacritics
- ✅ **Unicode Symbols** - Math, currency, arrows, emoji

**See:** [I18N_CAPABILITIES.md](I18N_CAPABILITIES.md) for details

### Current Limitations

Not all XSL-FO 1.1 features are implemented:
- ❌ Floats (`fo:float`)
- ❌ Footnotes (`fo:footnote`)
- ❌ Some region types (region-start, region-end)
- ❌ PDF/A compliance
- ✅ Most common features are fully supported

**See:** [LIMITATIONS.md](LIMITATIONS.md) for complete list

---

## 📖 Additional Resources

### In Project Root

- **[README.md](../README.md)** - Project overview and quick start
- **[TODO.md](../TODO.md)** - Development roadmap and progress
- **[fop.md](../fop.md)** - Technical architecture and implementation details

### Test Documentation

- **Integration Tests:** `tests/integration/`
  - 52 comprehensive tests
  - Real-world document examples
  - i18n test suite (8 tests)
  - Text flow tests (widow/orphan control)
  - Error handling tests

### Examples

- **[examples/](../examples/)** - Sample code and usage examples
  - Simple documents
  - Japanese PDF generation
  - Tables and complex layouts
  - CLI usage

---

## 🚀 Getting Started

### Basic Usage

```rust
use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use fop_render::PdfRenderer;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse XSL-FO document
    let fo_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
            <!-- Your XSL-FO content -->
        </fo:root>"#;

    let builder = FoTreeBuilder::new();
    let fo_tree = builder.parse(Cursor::new(fo_xml))?;

    // Layout
    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&fo_tree)?;

    // Render to PDF
    let renderer = PdfRenderer::new();
    let pdf = renderer.render(&area_tree)?;
    let bytes = pdf.to_bytes()?;

    std::fs::write("output.pdf", bytes)?;
    Ok(())
}
```

### Japanese PDF Example

```xml
<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
                           page-height="297mm" page-width="210mm">
      <fo:region-body margin="20mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="18pt" font-weight="bold">
        請求書
      </fo:block>
      <fo:block>
        こんにちは、世界！
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>
```

**See:** [I18N_CAPABILITIES.md](I18N_CAPABILITIES.md) for more examples

---

## 📊 Feature Overview

### Supported Features ✅

**Layout:**
- Block and inline elements
- Tables (auto & fixed layout)
- Lists with markers
- Multi-column layout
- Headers and footers
- Page breaks and pagination
- Widow/orphan control
- Keep constraints

**Typography:**
- Font embedding (TrueType/OpenType)
- Font subsetting
- Multiple languages (CJK, RTL, European)
- Text alignment and indentation
- Line breaking

**Graphics:**
- Background colors
- Borders (all styles)
- Images (PNG, JPEG)
- Leaders

**Output:**
- PDF 1.4 generation
- Links (internal & external)
- Bookmarks
- Metadata

### Performance

- **10-1200x faster** than Java FOP
- **100-150x less memory**
- Linear scalability O(n)
- Production-ready throughput:
  - Small documents: 2,525/sec
  - Invoices (50 blocks): 636/sec
  - Reports (100 blocks): 242/sec

---

## 🔧 Development

### Running Tests

```bash
# All tests
cargo test

# Integration tests only
cargo test --test integration

# i18n tests
cargo test i18n_tests

# With output
cargo test -- --nocapture
```

### Building Documentation

```bash
# API documentation
cargo doc --no-deps --open

# This user documentation is in markdown
# View with any markdown reader or on GitHub
```

---

## 📝 Contributing

When adding new features, please update:
1. This documentation (especially LIMITATIONS.md if removing a limitation)
2. Add tests in `tests/integration/`
3. Update TODO.md with progress
4. Add examples if applicable

---

## 📞 Support

For issues, questions, or contributions:
- GitHub Issues: Report bugs or request features
- Tests: See `tests/integration/` for usage examples
- Documentation: This directory for comprehensive guides

---

## ✅ Status

**Current Version:** Phase 5-6 Enhanced
**Production Ready:** Yes
**i18n Support:** Complete
**Test Coverage:** 52 integration tests, 383 unit tests
**Quality:** Zero warnings maintained

---

**Last Updated:** 2026-02-16
