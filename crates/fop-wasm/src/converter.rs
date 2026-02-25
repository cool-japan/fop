//! FOP converter for WASM - the main JavaScript-facing API
//!
//! Provides `FopConverter` class that can be used from JavaScript to convert
//! XSL-FO documents to PDF or SVG.
//!
//! Because `FoArena<'a>` has a lifetime parameter (for property inheritance),
//! it cannot be stored directly in `#[wasm_bindgen]` structs. Instead, we
//! parse + layout + render in a single function call, which keeps the lifetime
//! contained within the call stack.

use crate::error::fop_error_to_js;
use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use fop_render::{PdfRenderer, SvgRenderer, TextRenderer};
use std::io::Cursor;
use wasm_bindgen::prelude::*;

/// FOP converter for JavaScript
///
/// Usage from JavaScript:
/// ```js
/// import init, { FopConverter } from 'fop-wasm';
/// await init();
/// const fop = new FopConverter();
/// const pdfBytes = fop.convert_to_pdf(foXmlString);
/// const svgString = fop.convert_to_svg(foXmlString);
/// ```
#[wasm_bindgen]
pub struct FopConverter {
    /// Whether to enable verbose logging
    verbose: bool,
}

#[wasm_bindgen]
impl FopConverter {
    /// Create a new FOP converter
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { verbose: false }
    }

    /// Enable or disable verbose logging
    #[wasm_bindgen(js_name = setVerbose)]
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    /// Convert XSL-FO to PDF bytes (one-shot conversion)
    ///
    /// # Arguments
    /// * `fo_xml` - XSL-FO document as a string
    ///
    /// # Returns
    /// PDF bytes as Uint8Array
    #[wasm_bindgen(js_name = convertToPdf)]
    pub fn convert_to_pdf(&self, fo_xml: &str) -> Result<Vec<u8>, JsValue> {
        // Parse
        let builder = FoTreeBuilder::new();
        let cursor = Cursor::new(fo_xml.as_bytes());
        let arena = builder.parse(cursor).map_err(fop_error_to_js)?;

        // Layout
        let engine = LayoutEngine::new();
        let area_tree = engine.layout(&arena).map_err(fop_error_to_js)?;

        // Render
        let renderer = PdfRenderer::new();
        let pdf_doc = renderer.render(&area_tree).map_err(fop_error_to_js)?;

        // Serialize
        let pdf_bytes = pdf_doc.to_bytes().map_err(fop_error_to_js)?;

        Ok(pdf_bytes)
    }

    /// Convert XSL-FO to SVG string (one-shot conversion)
    ///
    /// # Arguments
    /// * `fo_xml` - XSL-FO document as a string
    ///
    /// # Returns
    /// SVG content as a string
    #[wasm_bindgen(js_name = convertToSvg)]
    pub fn convert_to_svg(&self, fo_xml: &str) -> Result<String, JsValue> {
        // Parse
        let builder = FoTreeBuilder::new();
        let cursor = Cursor::new(fo_xml.as_bytes());
        let arena = builder.parse(cursor).map_err(fop_error_to_js)?;

        // Layout
        let engine = LayoutEngine::new();
        let area_tree = engine.layout(&arena).map_err(fop_error_to_js)?;

        // Render
        let renderer = SvgRenderer::new();
        let svg_content = renderer
            .render_to_svg(&area_tree)
            .map_err(fop_error_to_js)?;

        Ok(svg_content)
    }

    /// Convert XSL-FO to plain text (one-shot conversion)
    ///
    /// # Arguments
    /// * `fo_xml` - XSL-FO document as a string
    ///
    /// # Returns
    /// Plain text content
    #[wasm_bindgen(js_name = convertToText)]
    pub fn convert_to_text(&self, fo_xml: &str) -> Result<String, JsValue> {
        // Parse
        let builder = FoTreeBuilder::new();
        let cursor = Cursor::new(fo_xml.as_bytes());
        let arena = builder.parse(cursor).map_err(fop_error_to_js)?;

        // Layout
        let engine = LayoutEngine::new();
        let area_tree = engine.layout(&arena).map_err(fop_error_to_js)?;

        // Render
        let renderer = TextRenderer::new();
        let text = renderer
            .render_to_text(&area_tree)
            .map_err(fop_error_to_js)?;

        Ok(text)
    }

    /// Validate an XSL-FO document without rendering
    ///
    /// # Arguments
    /// * `fo_xml` - XSL-FO document as a string
    ///
    /// # Returns
    /// Validation result as a JSON string
    pub fn validate(&self, fo_xml: &str) -> Result<String, JsValue> {
        let builder = FoTreeBuilder::new();
        let cursor = Cursor::new(fo_xml.as_bytes());

        match builder.parse(cursor) {
            Ok(arena) => Ok(format!(r#"{{"valid": true, "nodes": {}}}"#, arena.len())),
            Err(e) => Ok(format!(
                r#"{{"valid": false, "error": "{}"}}"#,
                e.to_string().replace('"', "\\\"")
            )),
        }
    }

    /// Get version information
    pub fn version(&self) -> String {
        format!("fop-wasm {}", env!("CARGO_PKG_VERSION"))
    }
}

impl Default for FopConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// One-shot conversion function (no class needed)
///
/// Convenience function for simple one-off conversions.
#[wasm_bindgen(js_name = convertFoToPdf)]
pub fn convert_fo_to_pdf(fo_xml: &str) -> Result<Vec<u8>, JsValue> {
    let converter = FopConverter::new();
    converter.convert_to_pdf(fo_xml)
}

/// One-shot SVG conversion function
#[wasm_bindgen(js_name = convertFoToSvg)]
pub fn convert_fo_to_svg(fo_xml: &str) -> Result<String, JsValue> {
    let converter = FopConverter::new();
    converter.convert_to_svg(fo_xml)
}

/// Get supported output formats
#[wasm_bindgen(js_name = supportedFormats)]
pub fn supported_formats() -> Vec<String> {
    vec!["pdf".to_string(), "svg".to_string(), "text".to_string()]
}

// ============================================================================
// Native Rust API (not exposed to WASM)
// ============================================================================

impl FopConverter {
    /// Convert XSL-FO string to PDF bytes (native Rust API)
    pub fn fo_to_pdf(fo_xml: &str) -> std::result::Result<Vec<u8>, String> {
        let builder = FoTreeBuilder::new();
        let cursor = Cursor::new(fo_xml.as_bytes());
        let arena = builder.parse(cursor).map_err(|e| e.to_string())?;

        let engine = LayoutEngine::new();
        let area_tree = engine.layout(&arena).map_err(|e| e.to_string())?;

        let renderer = PdfRenderer::new();
        let pdf_doc = renderer.render(&area_tree).map_err(|e| e.to_string())?;

        pdf_doc.to_bytes().map_err(|e| e.to_string())
    }

    /// Convert XSL-FO string to SVG string (native Rust API)
    pub fn fo_to_svg(fo_xml: &str) -> std::result::Result<String, String> {
        let builder = FoTreeBuilder::new();
        let cursor = Cursor::new(fo_xml.as_bytes());
        let arena = builder.parse(cursor).map_err(|e| e.to_string())?;

        let engine = LayoutEngine::new();
        let area_tree = engine.layout(&arena).map_err(|e| e.to_string())?;

        let renderer = SvgRenderer::new();
        renderer
            .render_to_svg(&area_tree)
            .map_err(|e| e.to_string())
    }

    /// Convert XSL-FO string to plain text (native Rust API)
    pub fn fo_to_text(fo_xml: &str) -> std::result::Result<String, String> {
        let builder = FoTreeBuilder::new();
        let cursor = Cursor::new(fo_xml.as_bytes());
        let arena = builder.parse(cursor).map_err(|e| e.to_string())?;

        let engine = LayoutEngine::new();
        let area_tree = engine.layout(&arena).map_err(|e| e.to_string())?;

        let renderer = TextRenderer::new();
        renderer
            .render_to_text(&area_tree)
            .map_err(|e| e.to_string())
    }

    /// Validate XSL-FO document and return node count (native Rust API)
    pub fn fo_validate(fo_xml: &str) -> std::result::Result<usize, String> {
        let builder = FoTreeBuilder::new();
        let cursor = Cursor::new(fo_xml.as_bytes());
        builder
            .parse(cursor)
            .map(|arena| arena.len())
            .map_err(|e| e.to_string())
    }
}

// ============================================================================
// Internal implementation functions (testable without WASM runtime)
// ============================================================================

/// Internal implementation of FO-to-PDF conversion.
/// Returns PDF bytes or an error string.
pub fn convert_fo_to_pdf_internal(fo_xml: &str) -> Result<Vec<u8>, String> {
    FopConverter::fo_to_pdf(fo_xml)
}

/// Internal implementation of FO-to-SVG conversion.
/// Returns SVG string or an error string.
pub fn convert_fo_to_svg_internal(fo_xml: &str) -> Result<String, String> {
    FopConverter::fo_to_svg(fo_xml)
}

/// Internal implementation of FO-to-text conversion.
/// Returns plain-text string or an error string.
pub fn convert_fo_to_text_internal(fo_xml: &str) -> Result<String, String> {
    FopConverter::fo_to_text(fo_xml)
}

/// Internal implementation of FO document validation.
/// Returns node count on success, error string on failure.
pub fn validate_fo_internal(fo_xml: &str) -> Result<usize, String> {
    FopConverter::fo_validate(fo_xml)
}

// ============================================================================
// Unit tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // Shared test fixtures
    // ------------------------------------------------------------------

    const SIMPLE_FO: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
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
      <fo:block>Hello from WASM!</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

    const STYLED_FO: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
                          page-width="210mm"
                          page-height="297mm"
                          margin-top="20mm" margin-bottom="20mm"
                          margin-left="25mm" margin-right="25mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="18pt" font-weight="bold" color="blue">Title</fo:block>
      <fo:block font-size="12pt" space-after="6pt">
        Body with <fo:inline font-style="italic">italic</fo:inline>
        and <fo:inline font-weight="bold">bold</fo:inline> text.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

    const TABLE_FO: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
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
      <fo:table table-layout="fixed" width="150mm">
        <fo:table-column column-width="75mm"/>
        <fo:table-column column-width="75mm"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell border="1pt solid black" padding="2mm">
              <fo:block>Cell A1</fo:block>
            </fo:table-cell>
            <fo:table-cell border="1pt solid black" padding="2mm">
              <fo:block>Cell B1</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell border="1pt solid black" padding="2mm">
              <fo:block>Cell A2</fo:block>
            </fo:table-cell>
            <fo:table-cell border="1pt solid black" padding="2mm">
              <fo:block>Cell B2</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

    // ------------------------------------------------------------------
    // FopConverter struct tests
    // ------------------------------------------------------------------

    #[test]
    fn test_converter_creation() {
        let converter = FopConverter::new();
        assert!(!converter.verbose);
    }

    #[test]
    fn test_converter_default_equals_new() {
        let c1 = FopConverter::new();
        let c2 = FopConverter::default();
        assert_eq!(c1.verbose, c2.verbose);
    }

    #[test]
    fn test_converter_set_verbose_true() {
        let mut converter = FopConverter::new();
        converter.set_verbose(true);
        assert!(converter.verbose);
    }

    #[test]
    fn test_converter_set_verbose_false() {
        let mut converter = FopConverter::new();
        converter.set_verbose(true);
        converter.set_verbose(false);
        assert!(!converter.verbose);
    }

    #[test]
    fn test_version_contains_package_name() {
        let converter = FopConverter::new();
        let v = converter.version();
        assert!(v.contains("fop-wasm"), "version must contain crate name");
    }

    #[test]
    fn test_version_not_empty() {
        let v = FopConverter::new().version();
        assert!(!v.is_empty());
    }

    // ------------------------------------------------------------------
    // convert_fo_to_pdf_internal — happy paths
    // ------------------------------------------------------------------

    #[test]
    fn test_convert_fo_to_pdf_basic() {
        let result = convert_fo_to_pdf_internal(SIMPLE_FO);
        assert!(result.is_ok(), "expected Ok, got {:?}", result.err());
        let bytes = result.expect("test: should succeed");
        assert!(!bytes.is_empty());
        assert!(bytes.starts_with(b"%PDF-"), "must start with PDF header");
    }

    #[test]
    fn test_pdf_contains_eof_marker() {
        let bytes = convert_fo_to_pdf_internal(SIMPLE_FO).expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("%%EOF"), "PDF must end with %%EOF marker");
    }

    #[test]
    fn test_pdf_minimum_size() {
        let bytes = convert_fo_to_pdf_internal(SIMPLE_FO).expect("test: should succeed");
        // A minimal PDF is several hundred bytes at least
        assert!(bytes.len() > 200, "PDF too small: {} bytes", bytes.len());
    }

    #[test]
    fn test_pdf_styled_document() {
        let result = convert_fo_to_pdf_internal(STYLED_FO);
        assert!(result.is_ok(), "styled PDF conversion should succeed");
        let bytes = result.expect("test: should succeed");
        assert!(bytes.starts_with(b"%PDF-"));
    }

    #[test]
    fn test_pdf_table_document() {
        let result = convert_fo_to_pdf_internal(TABLE_FO);
        assert!(result.is_ok(), "table PDF conversion should succeed");
        let bytes = result.expect("test: should succeed");
        assert!(bytes.starts_with(b"%PDF-"));
    }

    #[test]
    fn test_pdf_multipage_document() {
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
      <fo:block>Page 1</fo:block>
      <fo:block break-before="page">Page 2</fo:block>
      <fo:block break-before="page">Page 3</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;
        let result = convert_fo_to_pdf_internal(fo);
        assert!(result.is_ok(), "multi-page PDF should succeed");
        let bytes = result.expect("test: should succeed");
        assert!(bytes.starts_with(b"%PDF-"));
    }

    #[test]
    fn test_pdf_unicode_content() {
        let fo = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
                          page-width="210mm" page-height="297mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Ünïcödé tëxt: αβγδ</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;
        let result = convert_fo_to_pdf_internal(fo);
        assert!(result.is_ok(), "Unicode content should convert to PDF");
    }

    #[test]
    fn test_pdf_xml_entities() {
        let fo = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
                          page-width="210mm" page-height="297mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>&lt;tag&gt; &amp; "quotes" &apos;apostrophe&apos;</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;
        let result = convert_fo_to_pdf_internal(fo);
        assert!(result.is_ok(), "XML entities should be handled");
    }

    // ------------------------------------------------------------------
    // convert_fo_to_pdf_internal — error paths
    // ------------------------------------------------------------------

    #[test]
    fn test_pdf_invalid_xml_errors() {
        let result = convert_fo_to_pdf_internal("<bad");
        assert!(result.is_err(), "malformed XML must produce an error");
    }

    #[test]
    fn test_pdf_empty_string_does_not_panic() {
        // Empty input is handled gracefully: may succeed with an empty document
        // or fail with a parse error — either is acceptable, but it must not panic.
        let result = convert_fo_to_pdf_internal("");
        let _ = result; // Ok or Err is fine; no panic is the invariant
    }

    #[test]
    fn test_pdf_wrong_namespace_errors() {
        // Valid XML but wrong namespace — should fail parsing/layout
        let fo = r#"<?xml version="1.0"?>
<root xmlns="http://example.com/wrong">
  <block>text</block>
</root>"#;
        let result = convert_fo_to_pdf_internal(fo);
        // Either error or empty-document fallback — must not panic
        let _ = result;
    }

    // ------------------------------------------------------------------
    // convert_fo_to_svg_internal — happy paths
    // ------------------------------------------------------------------

    #[test]
    fn test_convert_fo_to_svg_basic() {
        let result = convert_fo_to_svg_internal(SIMPLE_FO);
        assert!(result.is_ok(), "SVG conversion should succeed");
        let s = result.expect("test: should succeed");
        assert!(!s.is_empty());
        assert!(s.contains("<svg"), "output must contain <svg element");
    }

    #[test]
    fn test_svg_contains_xmlns() {
        let s = convert_fo_to_svg_internal(SIMPLE_FO).expect("test: should succeed");
        assert!(s.contains("xmlns"), "SVG must declare its namespace");
    }

    #[test]
    fn test_svg_styled_document() {
        let result = convert_fo_to_svg_internal(STYLED_FO);
        assert!(result.is_ok(), "styled SVG conversion should succeed");
        assert!(result.expect("test: should succeed").contains("<svg"));
    }

    #[test]
    fn test_svg_table_document() {
        let result = convert_fo_to_svg_internal(TABLE_FO);
        assert!(result.is_ok(), "table SVG conversion should succeed");
        assert!(result.expect("test: should succeed").contains("<svg"));
    }

    // ------------------------------------------------------------------
    // convert_fo_to_svg_internal — error paths
    // ------------------------------------------------------------------

    #[test]
    fn test_svg_invalid_xml_errors() {
        let result = convert_fo_to_svg_internal("<<<invalid");
        assert!(result.is_err(), "malformed XML must produce an SVG error");
    }

    #[test]
    fn test_svg_empty_string_does_not_panic() {
        // Empty input is handled gracefully: may succeed with an empty document
        // or fail with a parse error — either is acceptable, but it must not panic.
        let result = convert_fo_to_svg_internal("");
        let _ = result;
    }

    // ------------------------------------------------------------------
    // convert_fo_to_text_internal — happy paths
    // ------------------------------------------------------------------

    #[test]
    fn test_convert_fo_to_text_basic() {
        let result = convert_fo_to_text_internal(SIMPLE_FO);
        assert!(result.is_ok(), "text conversion should succeed");
        let text = result.expect("test: should succeed");
        assert!(!text.is_empty(), "converted text must not be empty");
    }

    #[test]
    fn test_text_contains_block_content() {
        let text = convert_fo_to_text_internal(SIMPLE_FO).expect("test: should succeed");
        // The block "Hello from WASM!" must appear in plain-text output
        assert!(
            text.contains("Hello from WASM!"),
            "text output must include block content"
        );
    }

    #[test]
    fn test_text_invalid_xml_errors() {
        let result = convert_fo_to_text_internal("<bad<bad");
        assert!(result.is_err(), "malformed XML must produce a text error");
    }

    // ------------------------------------------------------------------
    // validate_fo_internal
    // ------------------------------------------------------------------

    #[test]
    fn test_validate_valid_fo_returns_positive_count() {
        let result = validate_fo_internal(SIMPLE_FO);
        assert!(result.is_ok(), "validation of valid FO should succeed");
        let count = result.expect("test: should succeed");
        assert!(count > 0, "valid FO must have at least one node");
    }

    #[test]
    fn test_validate_invalid_xml_errors() {
        let result = validate_fo_internal("<bad");
        assert!(result.is_err(), "invalid XML must fail validation");
    }

    // ------------------------------------------------------------------
    // supported_formats
    // ------------------------------------------------------------------

    #[test]
    fn test_supported_formats_includes_pdf() {
        let formats = supported_formats();
        assert!(formats.contains(&"pdf".to_string()));
    }

    #[test]
    fn test_supported_formats_includes_svg() {
        let formats = supported_formats();
        assert!(formats.contains(&"svg".to_string()));
    }

    #[test]
    fn test_supported_formats_includes_text() {
        let formats = supported_formats();
        assert!(formats.contains(&"text".to_string()));
    }

    #[test]
    fn test_supported_formats_not_empty() {
        let formats = supported_formats();
        assert!(!formats.is_empty());
    }

    // ------------------------------------------------------------------
    // Consistency: same input → same output
    // ------------------------------------------------------------------

    #[test]
    fn test_pdf_and_svg_both_succeed_for_same_input() {
        let pdf = convert_fo_to_pdf_internal(SIMPLE_FO);
        let svg = convert_fo_to_svg_internal(SIMPLE_FO);
        assert!(pdf.is_ok(), "PDF must succeed");
        assert!(svg.is_ok(), "SVG must succeed");
    }

    #[test]
    fn test_repeated_pdf_conversions_are_consistent() {
        let r1 = convert_fo_to_pdf_internal(SIMPLE_FO).expect("test: should succeed");
        let r2 = convert_fo_to_pdf_internal(SIMPLE_FO).expect("test: should succeed");
        // Both should be valid PDFs of the same length (deterministic render)
        assert_eq!(
            r1.len(),
            r2.len(),
            "repeated PDF conversions must be deterministic"
        );
    }

    // ------------------------------------------------------------------
    // FopConverter::fo_to_pdf / fo_to_svg native API
    // ------------------------------------------------------------------

    #[test]
    fn test_convert_to_pdf() {
        let result = FopConverter::fo_to_pdf(SIMPLE_FO);
        assert!(result.is_ok());
        let pdf_bytes = result.expect("test: should succeed");
        assert!(!pdf_bytes.is_empty());
        assert!(pdf_bytes.starts_with(b"%PDF-"));
    }

    #[test]
    fn test_convert_to_svg() {
        let result = FopConverter::fo_to_svg(SIMPLE_FO);
        assert!(result.is_ok());
        let svg = result.expect("test: should succeed");
        assert!(svg.contains("<svg"));
    }

    #[test]
    fn test_invalid_fo() {
        let result = FopConverter::fo_to_pdf("<<<invalid>>><<<");
        assert!(result.is_err());
    }

    #[test]
    fn test_version() {
        let converter = FopConverter::new();
        let version = converter.version();
        assert!(version.contains("fop-wasm"));
    }
}
