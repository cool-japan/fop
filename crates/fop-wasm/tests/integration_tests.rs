//! Native integration tests for fop-wasm (run without WASM environment)
//!
//! These tests verify the core functionality using the native Rust API.

use fop_wasm::FopConverter;

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

/// Invalid XML document (matches the unit test format)
const INVALID_XML: &str = "<<<invalid>>><<<";

// ============================================================================
// PDF Conversion Tests
// ============================================================================

#[test]
fn test_native_pdf_conversion_simple() {
    let result = FopConverter::fo_to_pdf(SIMPLE_FO);
    assert!(result.is_ok(), "PDF conversion should succeed");

    let pdf_bytes = result.expect("test: should succeed");
    assert!(!pdf_bytes.is_empty(), "PDF should not be empty");
    assert!(
        pdf_bytes.starts_with(b"%PDF-"),
        "PDF should have correct header"
    );
}

#[test]
fn test_native_pdf_conversion_complex() {
    let result = FopConverter::fo_to_pdf(COMPLEX_FO);
    assert!(result.is_ok(), "Complex PDF conversion should succeed");

    let pdf_bytes = result.expect("test: should succeed");
    assert!(!pdf_bytes.is_empty(), "PDF should not be empty");
    assert!(pdf_bytes.len() > 100, "PDF should have reasonable size");
}

#[test]
fn test_native_pdf_invalid_input() {
    let result = FopConverter::fo_to_pdf(INVALID_XML);
    assert!(result.is_err(), "Invalid XML should fail");
}

#[test]
fn test_native_pdf_empty_input() {
    let result = FopConverter::fo_to_pdf("");
    // Empty input will fail during XML parsing
    assert!(
        result.is_err() || result.is_ok(),
        "Empty input handling should not panic"
    );
}

// ============================================================================
// SVG Conversion Tests
// ============================================================================

#[test]
fn test_native_svg_conversion_simple() {
    let result = FopConverter::fo_to_svg(SIMPLE_FO);
    assert!(result.is_ok(), "SVG conversion should succeed");

    let svg = result.expect("test: should succeed");
    assert!(!svg.is_empty(), "SVG should not be empty");
    assert!(svg.contains("<svg"), "SVG should contain svg element");
}

#[test]
fn test_native_svg_conversion_complex() {
    let result = FopConverter::fo_to_svg(COMPLEX_FO);
    assert!(result.is_ok(), "Complex SVG conversion should succeed");

    let svg = result.expect("test: should succeed");
    assert!(!svg.is_empty(), "SVG should not be empty");
    assert!(svg.contains("<svg"), "SVG should contain svg element");
}

#[test]
fn test_native_svg_invalid_input() {
    let result = FopConverter::fo_to_svg(INVALID_XML);
    assert!(result.is_err(), "Invalid XML should fail");
}

// ============================================================================
// Multi-page Document Tests
// ============================================================================

#[test]
fn test_native_multi_page_pdf() {
    let multi_page_fo = r#"<?xml version="1.0" encoding="UTF-8"?>
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

    let result = FopConverter::fo_to_pdf(multi_page_fo);
    assert!(result.is_ok(), "Multi-page PDF should succeed");

    let pdf_bytes = result.expect("test: should succeed");
    assert!(pdf_bytes.len() > 200, "Multi-page PDF should be larger");
}

// ============================================================================
// Unicode and Special Characters Tests
// ============================================================================

#[test]
fn test_native_unicode_content() {
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
      <fo:block>Hello 世界</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

    let result = FopConverter::fo_to_pdf(unicode_fo);
    assert!(result.is_ok(), "Unicode content should be handled");
}

#[test]
fn test_native_xml_entities() {
    let entities_fo = r#"<?xml version="1.0" encoding="UTF-8"?>
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
      <fo:block>&lt;Tag&gt; &amp; "quotes"</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

    let result = FopConverter::fo_to_pdf(entities_fo);
    assert!(result.is_ok(), "XML entities should be handled");
}

// ============================================================================
// Converter Instance Tests
// ============================================================================

#[test]
fn test_native_converter_creation() {
    let converter = FopConverter::new();
    let version = converter.version();
    assert!(!version.is_empty(), "Version should not be empty");
    assert!(
        version.contains("fop-wasm"),
        "Version should contain package name"
    );
}

#[test]
fn test_native_converter_default() {
    let converter = FopConverter::default();
    let version = converter.version();
    assert!(!version.is_empty(), "Default constructor should work");
}

#[test]
fn test_native_converter_verbose() {
    let mut converter = FopConverter::new();
    converter.set_verbose(true);
    // Verbose flag is set, but doesn't affect native conversion
    // This just verifies the API works
}

// ============================================================================
// Large Document Tests
// ============================================================================

#[test]
fn test_native_large_document() {
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

    // Add many blocks
    for i in 0..50 {
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

    let result = FopConverter::fo_to_pdf(&large_fo);
    assert!(result.is_ok(), "Large document should be processed");
}

// ============================================================================
// Edge Cases Tests
// ============================================================================

#[test]
fn test_native_empty_flow() {
    let empty_flow_fo = r#"<?xml version="1.0" encoding="UTF-8"?>
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

    let result = FopConverter::fo_to_pdf(empty_flow_fo);
    assert!(result.is_ok(), "Empty flow should be allowed");
}

#[test]
fn test_native_whitespace_only() {
    let result = FopConverter::fo_to_pdf("   \n\t  ");
    // Whitespace-only input will fail during XML parsing
    assert!(
        result.is_err() || result.is_ok(),
        "Whitespace-only input handling should not panic"
    );
}

// ============================================================================
// Format Support Tests
// ============================================================================

#[test]
fn test_native_supported_formats() {
    let formats = fop_wasm::supported_formats();
    assert!(!formats.is_empty(), "Should support at least one format");
    assert!(formats.contains(&"pdf".to_string()), "Should support PDF");
    assert!(formats.contains(&"svg".to_string()), "Should support SVG");
    assert!(formats.contains(&"text".to_string()), "Should support text");
}

// ============================================================================
// Format Support Tests (continued)
// ============================================================================

#[test]
fn test_native_pdf_and_svg_same_input() {
    let pdf_result = FopConverter::fo_to_pdf(SIMPLE_FO);
    let svg_result = FopConverter::fo_to_svg(SIMPLE_FO);

    assert!(pdf_result.is_ok(), "PDF conversion should work");
    assert!(svg_result.is_ok(), "SVG conversion should work");
}

#[test]
fn test_native_multiple_conversions() {
    // Test that multiple conversions work in sequence
    let result1 = FopConverter::fo_to_pdf(SIMPLE_FO);
    let result2 = FopConverter::fo_to_svg(SIMPLE_FO);
    let result3 = FopConverter::fo_to_pdf(COMPLEX_FO);

    assert!(
        result1.is_ok() && result2.is_ok() && result3.is_ok(),
        "Multiple conversions should all succeed"
    );
}

// ============================================================================
// WASM-Specific Conformance Tests
//
// These three tests verify WASM API behaviour for patterns that matter in a
// browser / Node.js runtime context:
//
//   1. Idempotent conversion  — the same XSL-FO input always yields identical
//      output (deterministic renderer, safe for caching in JS land).
//   2. Static-content headers — `fo:static-content` (page headers / footers)
//      must survive the full parse → layout → render pipeline and produce a
//      valid PDF, since this is the primary WASM use-case pattern.
//   3. Plain-text extraction  — `fo_to_text()` must extract all block text
//      from a structured document, which is the typical WASM preview path.
// ============================================================================

// ── Conformance test 1: deterministic / idempotent conversion ────────────────

#[test]
fn wasm_conformance_idempotent_pdf_conversion() {
    // The WASM JS wrapper calls the Rust function each time a user triggers
    // a "Download PDF" action.  Two calls with identical input must produce
    // byte-for-byte identical output so that browser caching / comparison
    // logic is reliable.
    let fo = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
                          page-width="210mm" page-height="297mm"
                          margin-top="20mm" margin-bottom="20mm"
                          margin-left="20mm" margin-right="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="14pt" font-weight="bold">Idempotency Report</fo:block>
      <fo:block space-before="6pt">
        This document is rendered twice.  Both renders must produce identical
        PDF bytes.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

    let first = FopConverter::fo_to_pdf(fo).expect("first render must succeed");
    let second = FopConverter::fo_to_pdf(fo).expect("second render must succeed");

    assert!(
        first.starts_with(b"%PDF-"),
        "first render must be a valid PDF"
    );
    assert_eq!(
        first.len(),
        second.len(),
        "idempotent renders must produce identical byte counts"
    );
    assert_eq!(
        first, second,
        "idempotent renders must be byte-for-byte equal"
    );
}

// ── Conformance test 2: static-content (headers / footers) ──────────────────

#[test]
fn wasm_conformance_static_content_header_footer() {
    // Documents served through the WASM API commonly include page headers and
    // footers via `fo:static-content`.  The pipeline must handle the full
    // before/after region rendering without error.
    let fo = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
                          page-width="210mm" page-height="297mm"
                          margin-top="25mm" margin-bottom="25mm"
                          margin-left="20mm" margin-right="20mm">
      <fo:region-before extent="15mm"/>
      <fo:region-body/>
      <fo:region-after extent="15mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block font-size="9pt" text-align="center" border-bottom="0.5pt solid gray">
        WASM PDF Header
      </fo:block>
    </fo:static-content>
    <fo:static-content flow-name="xsl-region-after">
      <fo:block font-size="9pt" text-align="center" border-top="0.5pt solid gray">
        Page 1
      </fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="12pt">
        Main body content rendered via the WASM API.
      </fo:block>
      <fo:block space-before="6pt">
        Second paragraph confirming static-content does not interfere with
        normal flow rendering.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

    let pdf_result = FopConverter::fo_to_pdf(fo);
    assert!(
        pdf_result.is_ok(),
        "static-content (header/footer) document must convert to PDF: {:?}",
        pdf_result.err()
    );
    let pdf = pdf_result.expect("test: should succeed");
    assert!(pdf.starts_with(b"%PDF-"), "result must be a valid PDF");

    // SVG path must also handle static-content without panic
    let svg_result = FopConverter::fo_to_svg(fo);
    assert!(
        svg_result.is_ok(),
        "static-content document must convert to SVG: {:?}",
        svg_result.err()
    );
    let svg = svg_result.expect("test: should succeed");
    assert!(
        svg.contains("<svg"),
        "SVG output must contain an <svg element"
    );
}

// ── Conformance test 3: plain-text extraction from structured document ───────

#[test]
fn wasm_conformance_text_extraction_structured_document() {
    // The WASM "preview" pattern extracts plain text from a richly structured
    // XSL-FO document (headings, body paragraphs, a footer note).  All
    // block-level text content must appear in the extracted text.
    let fo = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
                          page-width="210mm" page-height="297mm"
                          margin-top="20mm" margin-bottom="20mm"
                          margin-left="20mm" margin-right="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="16pt" font-weight="bold" space-after="8pt">
        WASM Preview Heading
      </fo:block>
      <fo:block font-size="11pt" space-after="4pt">
        First paragraph of the document body for text extraction.
      </fo:block>
      <fo:block font-size="11pt" space-after="4pt">
        Second paragraph containing additional content.
      </fo:block>
      <fo:block font-size="9pt" color="gray">
        Footer note at the bottom of the extracted text.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

    let text_result = FopConverter::fo_to_text(fo);
    assert!(
        text_result.is_ok(),
        "text extraction from structured document must succeed: {:?}",
        text_result.err()
    );

    let text = text_result.expect("test: should succeed");
    assert!(!text.is_empty(), "extracted text must not be empty");

    // All four blocks must be present in the extracted plain text
    assert!(
        text.contains("WASM Preview Heading"),
        "extracted text must contain the heading block"
    );
    assert!(
        text.contains("First paragraph"),
        "extracted text must contain the first body paragraph"
    );
    assert!(
        text.contains("Second paragraph"),
        "extracted text must contain the second body paragraph"
    );
    assert!(
        text.contains("Footer note"),
        "extracted text must contain the footer note block"
    );
}
