//! Python bindings for Apache FOP
//!
//! Provides XSL-FO to PDF/SVG/text conversion from Python.
//!
//! # Python Usage
//!
//! ```python
//! import fop
//!
//! converter = fop.FopConverter()
//! pdf_bytes = converter.convert_to_pdf(fo_xml_string)
//! converter.convert_file("input.fo", "output.pdf")
//! ```

pub mod converter;
pub mod error;

use converter::FopConverter;
use pyo3::prelude::*;

/// Python module definition
#[pymodule]
fn fop(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FopConverter>()?;

    // Module-level convenience functions
    m.add_function(wrap_pyfunction!(convert_to_pdf, m)?)?;
    m.add_function(wrap_pyfunction!(convert_to_svg, m)?)?;
    m.add_function(wrap_pyfunction!(version, m)?)?;

    Ok(())
}

/// One-shot conversion: XSL-FO string → PDF bytes
///
/// Args:
///     fo_xml: XSL-FO document as a string
///
/// Returns:
///     PDF content as bytes
#[pyfunction]
fn convert_to_pdf(fo_xml: &str) -> PyResult<Vec<u8>> {
    let converter = FopConverter::new();
    converter.convert_to_pdf(fo_xml)
}

/// One-shot conversion: XSL-FO string → SVG string
///
/// Args:
///     fo_xml: XSL-FO document as a string
///
/// Returns:
///     SVG content as a string
#[pyfunction]
fn convert_to_svg(fo_xml: &str) -> PyResult<String> {
    let converter = FopConverter::new();
    converter.convert_to_svg(fo_xml)
}

/// Get version information
#[pyfunction]
fn version() -> String {
    format!("fop-python {}", env!("CARGO_PKG_VERSION"))
}

// ============================================================================
// Unit tests for lib.rs (module-level functions and re-exports)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::converter::FopConverter;

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
      <fo:block>lib.rs test content here</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

    // ------------------------------------------------------------------
    // version() function
    // ------------------------------------------------------------------

    #[test]
    fn test_version_is_not_empty() {
        let v = version();
        assert!(!v.is_empty(), "version() must not return an empty string");
    }

    #[test]
    fn test_version_contains_crate_name() {
        let v = version();
        assert!(
            v.contains("fop-python"),
            "version() must contain crate name 'fop-python', got: {}",
            v
        );
    }

    #[test]
    fn test_version_contains_semver_dot() {
        let v = version();
        assert!(
            v.contains('.'),
            "version() must contain a dot (semver), got: {}",
            v
        );
    }

    // ------------------------------------------------------------------
    // convert_to_pdf (module-level one-shot function)
    // ------------------------------------------------------------------

    #[test]
    fn test_convert_to_pdf_valid_fo_returns_ok() {
        let result = convert_to_pdf(MINIMAL_FO);
        assert!(
            result.is_ok(),
            "convert_to_pdf must succeed for valid FO: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_convert_to_pdf_produces_pdf_header() {
        let bytes = convert_to_pdf(MINIMAL_FO).expect("test: should succeed");
        assert!(
            bytes.starts_with(b"%PDF-"),
            "Output must start with PDF header"
        );
    }

    #[test]
    fn test_convert_to_pdf_output_not_empty() {
        let bytes = convert_to_pdf(MINIMAL_FO).expect("test: should succeed");
        assert!(!bytes.is_empty(), "PDF output must not be empty");
    }

    #[test]
    fn test_convert_to_pdf_invalid_xml_returns_err() {
        let result = convert_to_pdf("<invalid-xml></not-closed>");
        assert!(
            result.is_err(),
            "convert_to_pdf must return Err for invalid XML"
        );
    }

    #[test]
    fn test_convert_to_pdf_empty_input_no_panic() {
        // Empty input may succeed or fail, but must not panic
        let _ = convert_to_pdf("");
    }

    // ------------------------------------------------------------------
    // convert_to_svg (module-level one-shot function)
    // ------------------------------------------------------------------

    #[test]
    fn test_convert_to_svg_valid_fo_returns_ok() {
        let result = convert_to_svg(MINIMAL_FO);
        assert!(
            result.is_ok(),
            "convert_to_svg must succeed for valid FO: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_convert_to_svg_output_contains_svg_element() {
        let svg = convert_to_svg(MINIMAL_FO).expect("test: should succeed");
        assert!(svg.contains("<svg"), "SVG output must contain <svg element");
    }

    #[test]
    fn test_convert_to_svg_output_not_empty() {
        let svg = convert_to_svg(MINIMAL_FO).expect("test: should succeed");
        assert!(!svg.is_empty(), "SVG output must not be empty");
    }

    #[test]
    fn test_convert_to_svg_invalid_xml_returns_err() {
        let result = convert_to_svg("<invalid-xml></not-closed>");
        assert!(
            result.is_err(),
            "convert_to_svg must return Err for invalid XML"
        );
    }

    #[test]
    fn test_convert_to_svg_empty_input_no_panic() {
        let _ = convert_to_svg("");
    }

    // ------------------------------------------------------------------
    // Module-level functions are consistent with FopConverter
    // ------------------------------------------------------------------

    #[test]
    fn test_module_pdf_and_converter_pdf_agree() {
        let module_result = convert_to_pdf(MINIMAL_FO);
        let converter = FopConverter::new();
        let converter_result = converter.convert_to_pdf(MINIMAL_FO);
        // Both should succeed or both should fail
        assert_eq!(
            module_result.is_ok(),
            converter_result.is_ok(),
            "Module-level and FopConverter must agree on success/failure"
        );
    }

    #[test]
    fn test_module_svg_and_converter_svg_agree() {
        let module_result = convert_to_svg(MINIMAL_FO);
        let converter = FopConverter::new();
        let converter_result = converter.convert_to_svg(MINIMAL_FO);
        assert_eq!(
            module_result.is_ok(),
            converter_result.is_ok(),
            "Module-level and FopConverter must agree on success/failure"
        );
    }

    #[test]
    fn test_pdf_output_ends_with_eof_marker() {
        let bytes = convert_to_pdf(MINIMAL_FO).expect("test: should succeed");
        let tail = &bytes[bytes.len().saturating_sub(10)..];
        let tail_str = std::str::from_utf8(tail).unwrap_or("");
        assert!(
            tail_str.contains("%%EOF"),
            "PDF must end with %%EOF, got tail: {:?}",
            tail_str
        );
    }

    #[test]
    fn test_svg_output_ends_with_closing_tag() {
        let svg = convert_to_svg(MINIMAL_FO).expect("test: should succeed");
        assert!(
            svg.contains("</svg>"),
            "SVG must end with closing </svg> tag"
        );
    }

    #[test]
    fn test_convert_to_pdf_consistent_between_calls() {
        let r1 = convert_to_pdf(MINIMAL_FO).expect("test: should succeed");
        let r2 = convert_to_pdf(MINIMAL_FO).expect("test: should succeed");
        assert_eq!(
            r1.len(),
            r2.len(),
            "Repeated PDF conversions must be deterministic (same byte count)"
        );
    }
}
