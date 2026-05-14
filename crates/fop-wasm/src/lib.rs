//! WebAssembly bindings for Apache FOP
//!
//! Provides XSL-FO to PDF/SVG conversion in the browser via wasm-bindgen.
//!
//! # JavaScript Usage
//!
//! ```javascript
//! import init, { FopConverter, convertFoToPdf } from 'fop-wasm';
//! await init();
//!
//! // Class-based API
//! const fop = new FopConverter();
//! const pdfBytes = fop.convertToPdf(foXmlString);
//! const svgString = fop.convertToSvg(foXmlString);
//!
//! // One-shot function API
//! const pdfBytes = convertFoToPdf(foXmlString);
//! ```

pub mod converter;
pub mod error;

pub use converter::{convert_fo_to_pdf, convert_fo_to_svg, supported_formats, FopConverter};

// ============================================================================
// Unit tests for lib.rs (module-level re-exports and public surface)
// ============================================================================
//
// Note: `JsValue::from_str` panics on non-wasm32 native targets, so we avoid
// calling `fop_error_to_js` / `error_to_js` at test runtime. Those helpers are
// exercised indirectly through Display output checks.

#[cfg(test)]
#[cfg(not(target_arch = "wasm32"))]
mod tests {
    use crate::converter::{
        convert_fo_to_pdf_internal, convert_fo_to_svg_internal, convert_fo_to_text_internal,
        supported_formats, validate_fo_internal, FopConverter,
    };

    // ------------------------------------------------------------------
    // Shared minimal FO fixture
    // ------------------------------------------------------------------

    const MINIMAL_FO: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
                           page-width="210mm" page-height="297mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>lib.rs test content</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

    // ------------------------------------------------------------------
    // supported_formats (re-exported via lib.rs)
    // ------------------------------------------------------------------

    #[test]
    fn test_supported_formats_re_export_available() {
        let fmts = supported_formats();
        assert!(!fmts.is_empty());
    }

    #[test]
    fn test_supported_formats_contains_pdf() {
        assert!(supported_formats().contains(&"pdf".to_string()));
    }

    #[test]
    fn test_supported_formats_contains_svg() {
        assert!(supported_formats().contains(&"svg".to_string()));
    }

    #[test]
    fn test_supported_formats_contains_text() {
        assert!(supported_formats().contains(&"text".to_string()));
    }

    #[test]
    fn test_supported_formats_has_three_entries() {
        assert_eq!(supported_formats().len(), 3);
    }

    // ------------------------------------------------------------------
    // FopConverter constructable in non-wasm context
    // ------------------------------------------------------------------

    #[test]
    fn test_fop_converter_can_be_constructed() {
        let _c = FopConverter::new();
    }

    #[test]
    fn test_fop_converter_default() {
        let _c = FopConverter::default();
    }

    #[test]
    fn test_fop_converter_version_string() {
        let v = FopConverter::new().version();
        assert!(
            v.starts_with("fop-wasm"),
            "Version must start with crate name, got: {}",
            v
        );
    }

    #[test]
    fn test_fop_converter_version_not_empty() {
        assert!(!FopConverter::new().version().is_empty());
    }

    // ------------------------------------------------------------------
    // convert_fo_to_pdf_internal
    // ------------------------------------------------------------------

    #[test]
    fn test_convert_fo_to_pdf_internal_valid_fo() {
        let result = convert_fo_to_pdf_internal(MINIMAL_FO);
        assert!(
            result.is_ok(),
            "valid FO must produce Ok: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_convert_fo_to_pdf_internal_produces_pdf_header() {
        let bytes = convert_fo_to_pdf_internal(MINIMAL_FO).expect("test: should succeed");
        assert!(bytes.starts_with(b"%PDF-"), "output must start with %PDF-");
    }

    #[test]
    fn test_convert_fo_to_pdf_internal_invalid_fo_returns_err() {
        let result = convert_fo_to_pdf_internal("<invalid-xml></not-closed>");
        assert!(result.is_err(), "invalid FO must return Err");
    }

    #[test]
    fn test_convert_fo_to_pdf_internal_empty_input_no_panic() {
        let _ = convert_fo_to_pdf_internal("");
    }

    #[test]
    fn test_convert_fo_to_pdf_internal_result_not_empty() {
        let bytes = convert_fo_to_pdf_internal(MINIMAL_FO).expect("test: should succeed");
        assert!(!bytes.is_empty(), "PDF output must not be empty");
    }

    // ------------------------------------------------------------------
    // convert_fo_to_svg_internal
    // ------------------------------------------------------------------

    #[test]
    fn test_convert_fo_to_svg_internal_returns_svg() {
        let result = convert_fo_to_svg_internal(MINIMAL_FO);
        assert!(result.is_ok(), "SVG conversion of valid FO must succeed");
        assert!(
            result.expect("test: should succeed").contains("<svg"),
            "output must contain <svg"
        );
    }

    #[test]
    fn test_convert_fo_to_svg_internal_invalid_fo_returns_err() {
        let result = convert_fo_to_svg_internal("<invalid-xml></not-closed>");
        assert!(result.is_err(), "invalid FO must return SVG Err");
    }

    #[test]
    fn test_convert_fo_to_svg_internal_empty_input_no_panic() {
        let _ = convert_fo_to_svg_internal("");
    }

    #[test]
    fn test_convert_fo_to_svg_internal_svg_not_empty() {
        let svg = convert_fo_to_svg_internal(MINIMAL_FO).expect("test: should succeed");
        assert!(!svg.is_empty(), "SVG output must not be empty");
    }

    // ------------------------------------------------------------------
    // validate_fo_internal
    // ------------------------------------------------------------------

    #[test]
    fn test_validate_fo_internal_valid_returns_positive_count() {
        let result = validate_fo_internal(MINIMAL_FO);
        assert!(result.is_ok(), "valid FO must pass validation");
        assert!(
            result.expect("test: should succeed") > 0,
            "node count must be > 0"
        );
    }

    #[test]
    fn test_validate_fo_internal_invalid_xml_returns_err() {
        let result = validate_fo_internal("<bad");
        assert!(result.is_err(), "invalid XML must fail validation");
    }

    #[test]
    fn test_validate_fo_internal_empty_returns_err_or_zero() {
        let result = validate_fo_internal("");
        if let Ok(n) = result {
            assert_eq!(n, 0, "empty string should yield 0 nodes if Ok");
        } // expected: Err case is acceptable
    }

    // ------------------------------------------------------------------
    // convert_fo_to_text_internal
    // ------------------------------------------------------------------

    #[test]
    fn test_convert_fo_to_text_internal_contains_content() {
        let result = convert_fo_to_text_internal(MINIMAL_FO);
        assert!(result.is_ok(), "text conversion must succeed");
        let text = result.expect("test: should succeed");
        assert!(!text.is_empty(), "text output must not be empty");
    }

    #[test]
    fn test_convert_fo_to_text_internal_invalid_fo_returns_err() {
        let result = convert_fo_to_text_internal("<invalid-xml></not-closed>");
        assert!(result.is_err(), "invalid FO must return text Err");
    }

    // ------------------------------------------------------------------
    // FopError Display strings verify what JsValue would receive
    // (without calling JsValue::from_str which panics on native targets)
    // ------------------------------------------------------------------

    #[test]
    fn test_fop_error_display_for_js_parse_error() {
        use fop_types::FopError;
        let err = FopError::ParseError("js-bound test".to_string());
        // This is exactly what fop_error_to_js() passes to JsValue::from_str
        let would_be_js_str = err.to_string();
        assert!(
            would_be_js_str.contains("js-bound test"),
            "String fed to JsValue must contain message, got: {}",
            would_be_js_str
        );
    }

    #[test]
    fn test_fop_error_display_for_js_generic_error() {
        use fop_types::FopError;
        let err = FopError::Generic("lib generic test".to_string());
        let would_be_js_str = err.to_string();
        assert!(
            would_be_js_str.contains("lib generic test"),
            "Generic error string must pass through unchanged"
        );
    }
}
