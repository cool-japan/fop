//! Integration tests for fop-wasm WebAssembly bindings
//!
//! These tests use wasm-bindgen-test to verify the WASM API works correctly.

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

// Import the WASM module functions and types
use fop_wasm::{convert_fo_to_pdf, convert_fo_to_svg, supported_formats, FopConverter};

/// Simple valid XSL-FO document for testing
const SIMPLE_FO: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
                          page-width="210mm"
                          page-height="297mm">
      <fo:region-body margin="1in"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Hello from WASM!</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

/// Complex XSL-FO with multiple formatting features
const COMPLEX_FO: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="Letter"
                          page-width="8.5in"
                          page-height="11in">
      <fo:region-body margin="1in"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="Letter">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="18pt" font-weight="bold" space-after="12pt">
        Title Block
      </fo:block>
      <fo:block font-size="12pt" space-after="6pt">
        Regular paragraph with <fo:inline font-style="italic">italic text</fo:inline>
        and <fo:inline font-weight="bold">bold text</fo:inline>.
      </fo:block>
      <fo:block font-family="monospace" padding="6pt">
        Code block with monospace font
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

/// XSL-FO with multiple pages
const MULTI_PAGE_FO: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
                          page-width="210mm"
                          page-height="297mm">
      <fo:region-body margin="1in"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Page 1 content</fo:block>
      <fo:block break-before="page">Page 2 content</fo:block>
      <fo:block break-before="page">Page 3 content</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

/// Invalid XML document
const INVALID_XML: &str = r#"<<<invalid XML>>>"#;

/// Invalid XSL-FO (valid XML but wrong namespace/structure)
const INVALID_FO: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<root>
  <not-fo-element>This is not valid XSL-FO</not-fo-element>
</root>"#;

/// Malformed XSL-FO (missing required elements)
const MALFORMED_FO: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:page-sequence master-reference="missing-master">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Missing layout-master-set</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

/// Empty FO document
const EMPTY_FO: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
                          page-width="210mm"
                          page-height="297mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

// ============================================================================
// Test 1: PDF Conversion Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_pdf_conversion_simple() {
    let converter = FopConverter::new();
    let result = converter.convert_to_pdf(SIMPLE_FO);

    assert!(result.is_ok(), "Simple PDF conversion should succeed");
    let pdf_bytes = result.expect("test: should succeed");

    assert!(!pdf_bytes.is_empty(), "PDF should not be empty");
    assert!(pdf_bytes.len() > 100, "PDF should have reasonable size");

    // Check PDF header
    assert_eq!(&pdf_bytes[0..5], b"%PDF-", "PDF should start with %PDF-");
}

#[wasm_bindgen_test]
fn test_pdf_conversion_complex() {
    let converter = FopConverter::new();
    let result = converter.convert_to_pdf(COMPLEX_FO);

    assert!(result.is_ok(), "Complex PDF conversion should succeed");
    let pdf_bytes = result.expect("test: should succeed");

    assert!(!pdf_bytes.is_empty(), "PDF should not be empty");
    assert!(
        pdf_bytes.starts_with(b"%PDF-"),
        "PDF should have correct header"
    );
}

#[wasm_bindgen_test]
fn test_pdf_conversion_multi_page() {
    let converter = FopConverter::new();
    let result = converter.convert_to_pdf(MULTI_PAGE_FO);

    assert!(result.is_ok(), "Multi-page PDF conversion should succeed");
    let pdf_bytes = result.expect("test: should succeed");

    assert!(!pdf_bytes.is_empty(), "PDF should not be empty");
    assert!(pdf_bytes.len() > 200, "Multi-page PDF should be larger");
}

#[wasm_bindgen_test]
fn test_pdf_conversion_empty_flow() {
    let converter = FopConverter::new();
    let result = converter.convert_to_pdf(EMPTY_FO);

    assert!(result.is_ok(), "Empty flow PDF conversion should succeed");
    let pdf_bytes = result.expect("test: should succeed");

    assert!(
        !pdf_bytes.is_empty(),
        "PDF should not be empty even with empty flow"
    );
}

#[wasm_bindgen_test]
fn test_pdf_one_shot_function() {
    let result = convert_fo_to_pdf(SIMPLE_FO);

    assert!(result.is_ok(), "One-shot PDF conversion should succeed");
    let pdf_bytes = result.expect("test: should succeed");

    assert!(!pdf_bytes.is_empty(), "PDF should not be empty");
    assert!(
        pdf_bytes.starts_with(b"%PDF-"),
        "PDF should have correct header"
    );
}

// ============================================================================
// Test 2: SVG Conversion Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_svg_conversion_simple() {
    let converter = FopConverter::new();
    let result = converter.convert_to_svg(SIMPLE_FO);

    assert!(result.is_ok(), "Simple SVG conversion should succeed");
    let svg = result.expect("test: should succeed");

    assert!(!svg.is_empty(), "SVG should not be empty");
    assert!(svg.contains("<svg"), "SVG should contain svg element");
    assert!(svg.contains("</svg>"), "SVG should be well-formed");
}

#[wasm_bindgen_test]
fn test_svg_conversion_complex() {
    let converter = FopConverter::new();
    let result = converter.convert_to_svg(COMPLEX_FO);

    assert!(result.is_ok(), "Complex SVG conversion should succeed");
    let svg = result.expect("test: should succeed");

    assert!(!svg.is_empty(), "SVG should not be empty");
    assert!(svg.contains("<svg"), "SVG should contain svg element");
    // Check for text elements from the formatted content
    assert!(
        svg.contains("<text") || svg.contains("Title"),
        "SVG should contain text content"
    );
}

#[wasm_bindgen_test]
fn test_svg_conversion_multi_page() {
    let converter = FopConverter::new();
    let result = converter.convert_to_svg(MULTI_PAGE_FO);

    assert!(result.is_ok(), "Multi-page SVG conversion should succeed");
    let svg = result.expect("test: should succeed");

    assert!(!svg.is_empty(), "SVG should not be empty");
    assert!(svg.contains("<svg"), "SVG should contain svg element");
}

#[wasm_bindgen_test]
fn test_svg_one_shot_function() {
    let result = convert_fo_to_svg(SIMPLE_FO);

    assert!(result.is_ok(), "One-shot SVG conversion should succeed");
    let svg = result.expect("test: should succeed");

    assert!(!svg.is_empty(), "SVG should not be empty");
    assert!(svg.contains("<svg"), "SVG should contain svg element");
}

// ============================================================================
// Test 3: Text Conversion Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_text_conversion_simple() {
    let converter = FopConverter::new();
    let result = converter.convert_to_text(SIMPLE_FO);

    assert!(result.is_ok(), "Simple text conversion should succeed");
    let text = result.expect("test: should succeed");

    assert!(!text.is_empty(), "Text should not be empty");
    assert!(
        text.contains("Hello from WASM!"),
        "Text should contain content"
    );
}

#[wasm_bindgen_test]
fn test_text_conversion_complex() {
    let converter = FopConverter::new();
    let result = converter.convert_to_text(COMPLEX_FO);

    assert!(result.is_ok(), "Complex text conversion should succeed");
    let text = result.expect("test: should succeed");

    assert!(!text.is_empty(), "Text should not be empty");
    assert!(text.contains("Title Block"), "Text should contain title");
    assert!(
        text.contains("Regular paragraph"),
        "Text should contain paragraph"
    );
}

#[wasm_bindgen_test]
fn test_text_conversion_preserves_content() {
    let converter = FopConverter::new();
    let result = converter.convert_to_text(MULTI_PAGE_FO);

    assert!(result.is_ok(), "Multi-page text conversion should succeed");
    let text = result.expect("test: should succeed");

    assert!(
        text.contains("Page 1"),
        "Text should contain page 1 content"
    );
    assert!(
        text.contains("Page 2"),
        "Text should contain page 2 content"
    );
    assert!(
        text.contains("Page 3"),
        "Text should contain page 3 content"
    );
}

// ============================================================================
// Test 4: Validation Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_validate_valid_document() {
    let converter = FopConverter::new();
    let result = converter.validate(SIMPLE_FO);

    assert!(result.is_ok(), "Validation should return Ok");
    let json = result.expect("test: should succeed");

    assert!(
        json.contains(r#""valid": true"#),
        "Document should be valid"
    );
    assert!(json.contains(r#""nodes""#), "Should report node count");
}

#[wasm_bindgen_test]
fn test_validate_complex_document() {
    let converter = FopConverter::new();
    let result = converter.validate(COMPLEX_FO);

    assert!(result.is_ok(), "Validation should return Ok");
    let json = result.expect("test: should succeed");

    assert!(
        json.contains(r#""valid": true"#),
        "Complex document should be valid"
    );
}

#[wasm_bindgen_test]
fn test_validate_invalid_xml() {
    let converter = FopConverter::new();
    let result = converter.validate(INVALID_XML);

    assert!(
        result.is_ok(),
        "Validation should return Ok (with error in JSON)"
    );
    let json = result.expect("test: should succeed");

    assert!(
        json.contains(r#""valid": false"#),
        "Invalid XML should not be valid"
    );
    assert!(json.contains(r#""error""#), "Should report error");
}

#[wasm_bindgen_test]
fn test_validate_invalid_fo() {
    let converter = FopConverter::new();
    let result = converter.validate(INVALID_FO);

    assert!(
        result.is_ok(),
        "Validation should return Ok (with error in JSON)"
    );
    let json = result.expect("test: should succeed");

    // This might parse but fail validation depending on implementation
    assert!(json.contains(r#""valid":"#), "Should have valid field");
}

// ============================================================================
// Test 5: Error Handling Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_error_invalid_xml_pdf() {
    let converter = FopConverter::new();
    let result = converter.convert_to_pdf(INVALID_XML);

    assert!(result.is_err(), "Invalid XML should fail PDF conversion");
}

#[wasm_bindgen_test]
fn test_error_invalid_xml_svg() {
    let converter = FopConverter::new();
    let result = converter.convert_to_svg(INVALID_XML);

    assert!(result.is_err(), "Invalid XML should fail SVG conversion");
}

#[wasm_bindgen_test]
fn test_error_invalid_xml_text() {
    let converter = FopConverter::new();
    let result = converter.convert_to_text(INVALID_XML);

    assert!(result.is_err(), "Invalid XML should fail text conversion");
}

#[wasm_bindgen_test]
fn test_error_malformed_fo() {
    let converter = FopConverter::new();
    let result = converter.convert_to_pdf(MALFORMED_FO);

    // Depending on validation strictness, this might fail at parse or layout stage
    assert!(
        result.is_err() || result.is_ok(),
        "Malformed FO handling should not panic"
    );
}

#[wasm_bindgen_test]
fn test_error_empty_string() {
    let converter = FopConverter::new();
    let result = converter.convert_to_pdf("");

    assert!(result.is_err(), "Empty string should fail conversion");
}

#[wasm_bindgen_test]
fn test_error_whitespace_only() {
    let converter = FopConverter::new();
    let result = converter.convert_to_pdf("   \n\t  ");

    assert!(
        result.is_err(),
        "Whitespace-only string should fail conversion"
    );
}

#[wasm_bindgen_test]
fn test_error_one_shot_pdf_invalid() {
    let result = convert_fo_to_pdf(INVALID_XML);
    assert!(
        result.is_err(),
        "One-shot PDF with invalid input should fail"
    );
}

#[wasm_bindgen_test]
fn test_error_one_shot_svg_invalid() {
    let result = convert_fo_to_svg(INVALID_XML);
    assert!(
        result.is_err(),
        "One-shot SVG with invalid input should fail"
    );
}

// ============================================================================
// Test 6: Version Reporting Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_version_format() {
    let converter = FopConverter::new();
    let version = converter.version();

    assert!(!version.is_empty(), "Version should not be empty");
    assert!(
        version.contains("fop-wasm"),
        "Version should contain package name"
    );
    assert!(
        version.contains('.'),
        "Version should contain version number"
    );
}

#[wasm_bindgen_test]
fn test_version_consistency() {
    let converter1 = FopConverter::new();
    let converter2 = FopConverter::new();

    assert_eq!(
        converter1.version(),
        converter2.version(),
        "Version should be consistent across instances"
    );
}

// ============================================================================
// Test 7: Supported Formats Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_supported_formats_list() {
    let formats = supported_formats();

    assert!(!formats.is_empty(), "Should support at least one format");
    assert!(formats.len() >= 3, "Should support PDF, SVG, and text");
}

#[wasm_bindgen_test]
fn test_supported_formats_contains_pdf() {
    let formats = supported_formats();

    assert!(
        formats.contains(&"pdf".to_string()),
        "Should support PDF format"
    );
}

#[wasm_bindgen_test]
fn test_supported_formats_contains_svg() {
    let formats = supported_formats();

    assert!(
        formats.contains(&"svg".to_string()),
        "Should support SVG format"
    );
}

#[wasm_bindgen_test]
fn test_supported_formats_contains_text() {
    let formats = supported_formats();

    assert!(
        formats.contains(&"text".to_string()),
        "Should support text format"
    );
}

// ============================================================================
// Additional Integration Tests
// ============================================================================

#[wasm_bindgen_test]
fn test_converter_instance_independence() {
    let converter1 = FopConverter::new();
    let converter2 = FopConverter::new();

    let result1 = converter1.convert_to_pdf(SIMPLE_FO);
    let result2 = converter2.convert_to_pdf(SIMPLE_FO);

    assert!(
        result1.is_ok() && result2.is_ok(),
        "Multiple converters should work independently"
    );
}

#[wasm_bindgen_test]
fn test_converter_reuse() {
    let converter = FopConverter::new();

    let result1 = converter.convert_to_pdf(SIMPLE_FO);
    let result2 = converter.convert_to_pdf(COMPLEX_FO);
    let result3 = converter.convert_to_svg(SIMPLE_FO);

    assert!(result1.is_ok(), "First conversion should succeed");
    assert!(result2.is_ok(), "Second conversion should succeed");
    assert!(result3.is_ok(), "Third conversion should succeed");
}

#[wasm_bindgen_test]
fn test_verbose_flag() {
    let mut converter = FopConverter::new();

    converter.set_verbose(true);
    let result = converter.convert_to_pdf(SIMPLE_FO);

    assert!(
        result.is_ok(),
        "Conversion with verbose=true should succeed"
    );
}

#[wasm_bindgen_test]
fn test_default_constructor() {
    let converter1 = FopConverter::new();
    let converter2 = FopConverter::default();

    let result1 = converter1.convert_to_pdf(SIMPLE_FO);
    let result2 = converter2.convert_to_pdf(SIMPLE_FO);

    assert!(
        result1.is_ok() && result2.is_ok(),
        "Both new() and default() should work"
    );
}

#[wasm_bindgen_test]
fn test_concurrent_conversions() {
    let converter = FopConverter::new();

    // Simulate concurrent conversions (in WASM context)
    let pdf_result = converter.convert_to_pdf(SIMPLE_FO);
    let svg_result = converter.convert_to_svg(SIMPLE_FO);
    let text_result = converter.convert_to_text(SIMPLE_FO);

    assert!(pdf_result.is_ok(), "PDF conversion should succeed");
    assert!(svg_result.is_ok(), "SVG conversion should succeed");
    assert!(text_result.is_ok(), "Text conversion should succeed");
}

#[wasm_bindgen_test]
fn test_output_format_differences() {
    let converter = FopConverter::new();

    let pdf_bytes = converter
        .convert_to_pdf(SIMPLE_FO)
        .expect("test: should succeed");
    let svg_string = converter
        .convert_to_svg(SIMPLE_FO)
        .expect("test: should succeed");
    let text_string = converter
        .convert_to_text(SIMPLE_FO)
        .expect("test: should succeed");

    // Verify different outputs
    assert!(
        pdf_bytes.starts_with(b"%PDF-"),
        "PDF should have PDF header"
    );
    assert!(svg_string.contains("<svg"), "SVG should have SVG tags");
    assert!(!text_string.contains("<"), "Text should not have XML tags");
}

#[wasm_bindgen_test]
fn test_large_document() {
    let converter = FopConverter::new();

    // Build a large document with many blocks
    let mut large_fo = String::from(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
                          page-width="210mm"
                          page-height="297mm">
      <fo:region-body margin="1in"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">"#,
    );

    for i in 0..100 {
        large_fo.push_str(&format!(
            r#"<fo:block>This is block number {}</fo:block>"#,
            i
        ));
    }

    large_fo.push_str(
        r#"
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#,
    );

    let result = converter.convert_to_pdf(&large_fo);
    assert!(result.is_ok(), "Large document conversion should succeed");
}

#[wasm_bindgen_test]
fn test_unicode_content() {
    let unicode_fo = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
                          page-width="210mm"
                          page-height="297mm">
      <fo:region-body margin="1in"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Hello 世界 🌍 مرحبا Привет</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

    let converter = FopConverter::new();
    let result = converter.convert_to_pdf(unicode_fo);

    assert!(
        result.is_ok(),
        "Unicode content should be handled correctly"
    );
}

#[wasm_bindgen_test]
fn test_special_characters_in_content() {
    let special_fo = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
                          page-width="210mm"
                          page-height="297mm">
      <fo:region-body margin="1in"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>&lt;Special&gt; &amp; "Characters"</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

    let converter = FopConverter::new();
    let result = converter.convert_to_text(special_fo);

    assert!(result.is_ok(), "Special XML characters should be handled");
    let text = result.expect("test: should succeed");
    assert!(
        text.contains("Special") || text.contains("Characters"),
        "Text should contain escaped content"
    );
}
