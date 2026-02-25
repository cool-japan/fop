# WASM Bindings Integration Tests

This directory contains comprehensive integration tests for the `fop-wasm` WebAssembly bindings.

## Test Files

### `web.rs` - WASM Browser Tests
These tests use `wasm-bindgen-test` and are designed to run in a WebAssembly environment (browser or Node.js).

**Running WASM tests:**
```bash
# Install wasm-pack if not already installed
cargo install wasm-pack

# Run tests in headless browser (requires Chrome/Firefox)
wasm-pack test --headless --chrome
wasm-pack test --headless --firefox

# Run tests in Node.js
wasm-pack test --node
```

**Test Coverage:**
1. **PDF Conversion Tests** (7 tests)
   - Simple document conversion
   - Complex formatted documents
   - Multi-page documents
   - Empty flow handling
   - One-shot function API

2. **SVG Conversion Tests** (5 tests)
   - Simple to complex SVG generation
   - Multi-page SVG output
   - One-shot function API

3. **Text Conversion Tests** (3 tests)
   - Plain text extraction
   - Content preservation across formats
   - Text from complex documents

4. **Validation Tests** (4 tests)
   - Valid document validation
   - Invalid XML detection
   - Error reporting in JSON format

5. **Error Handling Tests** (8 tests)
   - Invalid XML input for all formats
   - Malformed XSL-FO documents
   - Empty and whitespace-only input
   - One-shot function error handling

6. **Version Reporting Tests** (2 tests)
   - Version format validation
   - Version consistency

7. **Supported Formats Tests** (4 tests)
   - Format list validation
   - PDF, SVG, and text format support verification

8. **Integration Tests** (11+ tests)
   - Converter instance independence
   - Converter reusability
   - Verbose flag handling
   - Concurrent conversions
   - Large document processing
   - Unicode content support
   - Special character handling

**Total WASM tests: 44**

### `integration_tests.rs` - Native Rust Tests
These tests run using the native Rust API (not through WASM) and can be executed with `cargo test`.

**Running native tests:**
```bash
cargo test -p fop-wasm
```

**Test Coverage:**
1. **PDF Conversion Tests** (4 tests)
   - Simple and complex documents
   - Invalid input handling
   - Empty input handling

2. **SVG Conversion Tests** (3 tests)
   - Simple and complex documents
   - Invalid input error handling

3. **Multi-page Documents** (1 test)
   - Multi-page PDF generation

4. **Unicode and Special Characters** (2 tests)
   - Unicode content handling
   - XML entity escaping

5. **Converter Instance Tests** (3 tests)
   - Constructor validation
   - Default trait implementation
   - Verbose flag API

6. **Large Documents** (1 test)
   - Performance with many blocks

7. **Edge Cases** (2 tests)
   - Empty flow documents
   - Whitespace-only input

8. **Format Support** (3 tests)
   - Supported formats list
   - Multiple format conversions
   - Output format differences

**Total native tests: 19**

## Test Philosophy

### Comprehensive Coverage
- All public API methods are tested
- Both success and failure paths are validated
- Edge cases and error conditions are thoroughly tested

### Best Practices
- Each test has a clear, descriptive name
- Tests are organized by functionality
- Tests are independent and can run in any order
- No external dependencies or test data files required

### Documentation
- Test constants include example XSL-FO documents
- Each test includes assertion messages
- Test organization follows a logical structure

## Adding New Tests

When adding new features to `fop-wasm`, ensure:

1. Add corresponding tests to `web.rs` for WASM-specific functionality
2. Add tests to `integration_tests.rs` for native API validation
3. Test both success and error cases
4. Include edge cases (empty input, invalid data, etc.)
5. Update this README with new test categories

## Continuous Integration

These tests are designed to run in CI environments:

- Native tests run with standard `cargo test`
- WASM tests require `wasm-pack` and a browser/Node.js runtime
- All tests should pass with no warnings

## Performance Considerations

- Large document tests verify the system can handle reasonable workloads
- Tests avoid creating unnecessarily large outputs
- Concurrent conversion tests verify thread safety (in WASM single-threaded context)
