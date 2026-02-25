# FOP Integration Tests

Comprehensive end-to-end integration tests for the XSL-FO processor.

## Overview

This test suite validates the complete FOP pipeline: **Parse → Layout → Render**.

- **32 integration tests** covering all major features
- **16 realistic XSL-FO fixture files** representing real-world use cases
- **4 test categories**: Basic, Tables, Advanced, Error Handling

## Test Structure

```
tests/integration/
├── mod.rs                    # Test runner and helper functions
├── basic_tests.rs            # 10 basic document tests
├── table_tests.rs            # 4 table layout tests
├── advanced_tests.rs         # 8 advanced feature tests
├── error_tests.rs            # 10 error handling tests
└── fixtures/                 # 16 XSL-FO test documents
    ├── simple_single_page.fo
    ├── multi_page.fo
    ├── text_formatting.fo
    ├── nested_blocks.fo
    ├── simple_invoice.fo
    ├── complex_report.fo
    ├── table_simple.fo
    ├── table_spanning.fo
    ├── table_header_footer.fo
    ├── lists.fo
    ├── unicode.fo
    ├── long_text.fo
    ├── empty_document.fo
    ├── deeply_nested.fo
    ├── multi_column.fo
    └── with_headers_footers.fo
```

## Test Categories

### Basic Documents (10 tests)

Tests fundamental document processing:

- ✅ Simple single-page document
- ✅ Multi-page document with page breaks
- ✅ Document with headers and footers
- ✅ Text formatting (bold, italic, colors)
- ✅ Nested blocks with property inheritance
- ✅ Empty document edge case
- ✅ Long text with line breaking
- ✅ Deeply nested elements (10 levels)
- ✅ Unicode and multilingual text
- ✅ Simple invoice with table

### Tables (4 tests)

Tests table layout algorithms:

- ✅ Simple table with borders
- ✅ Table with column spanning
- ✅ Table with header and footer
- ✅ Complex invoice table

### Advanced Features (8 tests)

Tests advanced layout capabilities:

- ✅ Lists with markers
- ✅ Multi-column layout
- ✅ Complex report with multiple features
- ✅ Property inheritance
- ✅ Page breaks
- ✅ Complex formatting
- ✅ Unicode multilingual support
- ✅ Minimal document

### Error Handling (10 tests)

Tests robustness and error recovery:

- ✅ Invalid XML
- ✅ Missing required elements
- ✅ Unknown properties (graceful handling)
- ✅ Invalid property values
- ✅ Missing namespace
- ✅ Empty page sequence
- ✅ Malformed table
- ✅ Very large values
- ✅ Negative dimensions
- ✅ Unknown elements

## Running Tests

```bash
# Run all integration tests
cargo test --test integration

# Run specific test category
cargo test --test integration basic_tests
cargo test --test integration table_tests
cargo test --test integration advanced_tests
cargo test --test integration error_tests

# Run specific test
cargo test --test integration test_simple_invoice

# Run with verbose output
cargo test --test integration --verbose

# Run with backtrace for debugging
RUST_BACKTRACE=1 cargo test --test integration
```

## Test Fixtures

All fixture files are realistic XSL-FO documents that represent common use cases:

### Simple Documents
- `simple_single_page.fo` - Basic document structure
- `multi_page.fo` - Multiple pages with explicit breaks
- `empty_document.fo` - Edge case: empty content

### Text Processing
- `text_formatting.fo` - Bold, italic, underline, colors
- `nested_blocks.fo` - Deeply nested with inheritance
- `long_text.fo` - Long paragraphs testing line breaking
- `unicode.fo` - Multilingual text and symbols

### Tables
- `table_simple.fo` - Basic table with borders
- `table_spanning.fo` - Column/row spanning
- `table_header_footer.fo` - Table with header and footer rows
- `simple_invoice.fo` - Real-world invoice table

### Advanced
- `complex_report.fo` - Comprehensive report with tables and lists
- `multi_column.fo` - Multi-column layout
- `with_headers_footers.fo` - Document with static headers/footers
- `lists.fo` - Ordered and unordered lists

### Edge Cases
- `deeply_nested.fo` - 10 levels of nesting
- `empty_document.fo` - Empty flow

## Helper Functions

The test suite provides helper functions in `mod.rs`:

### `process_fo_document(fo_content: &str) -> Result<Vec<u8>>`

Runs the complete FOP pipeline on an XSL-FO string:

```rust
let fo_content = load_fixture("simple_invoice.fo");
let pdf_bytes = process_fo_document(&fo_content)?;
```

### `load_fixture(filename: &str) -> String`

Loads a fixture file from `fixtures/` directory:

```rust
let fo_content = load_fixture("simple_invoice.fo");
```

### `validate_pdf_bytes(bytes: &[u8])`

Validates PDF output:
- Checks minimum size
- Validates PDF header
- Runs PdfValidator for structural checks

## Success Criteria

All tests must:

1. ✅ Parse XSL-FO without errors
2. ✅ Generate valid area tree
3. ✅ Produce valid PDF output
4. ✅ Pass PDF validation checks
5. ✅ Complete in < 50ms total

## Coverage

The integration tests cover:

- **29 FO element types** (root, page-sequence, flow, block, table, list, etc.)
- **100+ properties** (fonts, colors, spacing, borders, alignment, etc.)
- **Multiple renderers** (PDF output validation)
- **Error handling** (invalid XML, missing elements, bad values)
- **Edge cases** (empty documents, deep nesting, unicode)

## Performance

All 32 tests complete in approximately **10-20ms**, demonstrating:
- Efficient parsing
- Fast layout algorithms
- Quick PDF generation

## Future Enhancements

Potential additions:

- [ ] Image embedding tests (PNG, JPEG)
- [ ] Link and bookmark tests
- [ ] Footnote tests
- [ ] Float tests
- [ ] Large document stress tests (1000+ pages)
- [ ] SVG and PostScript output tests
- [ ] Snapshot testing with reference PDFs

## Maintenance

When adding new features:

1. Create realistic fixture file(s) in `fixtures/`
2. Add test(s) to appropriate category file
3. Ensure tests pass: `cargo test --test integration`
4. Verify no warnings: Check output for compiler warnings
5. Update this README with new test count

## Notes

- All tests follow the **NO WARNINGS POLICY**
- Tests use realistic XSL-FO documents, not minimal examples
- Error tests validate graceful degradation, not just failures
- Each test verifies both success and output quality
