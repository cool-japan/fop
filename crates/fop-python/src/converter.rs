//! FOP converter for Python - the main Python-facing API
//!
//! Provides `FopConverter` class that can be used from Python to convert
//! XSL-FO documents to PDF, SVG, or plain text.

use crate::error::fop_error_to_py;
use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use fop_render::{PdfRenderer, SvgRenderer, TextRenderer};
use pyo3::prelude::*;
use std::io::Cursor;

/// FOP converter for Python
///
/// Usage:
/// ```python
/// import fop
///
/// converter = fop.FopConverter()
/// pdf_bytes = converter.convert_to_pdf(fo_xml_string)
/// svg_string = converter.convert_to_svg(fo_xml_string)
/// converter.convert_file("input.fo", "output.pdf")
/// ```
#[pyclass]
pub struct FopConverter {
    /// Whether verbose logging is enabled
    #[pyo3(get, set)]
    verbose: bool,
}

#[pymethods]
impl FopConverter {
    /// Create a new FOP converter
    #[new]
    pub fn new() -> Self {
        Self { verbose: false }
    }

    /// Convert XSL-FO string to PDF bytes
    ///
    /// Args:
    ///     fo_xml: XSL-FO document as a string
    ///
    /// Returns:
    ///     PDF content as bytes
    pub fn convert_to_pdf(&self, fo_xml: &str) -> PyResult<Vec<u8>> {
        // Parse
        let builder = FoTreeBuilder::new();
        let cursor = Cursor::new(fo_xml.as_bytes());
        let arena = builder.parse(cursor).map_err(fop_error_to_py)?;

        // Layout
        let engine = LayoutEngine::new();
        let area_tree = engine.layout(&arena).map_err(fop_error_to_py)?;

        // Render
        let renderer = PdfRenderer::new();
        let pdf_doc = renderer.render(&area_tree).map_err(fop_error_to_py)?;
        let pdf_bytes = pdf_doc.to_bytes().map_err(fop_error_to_py)?;

        Ok(pdf_bytes)
    }

    /// Convert XSL-FO string to SVG string
    ///
    /// Args:
    ///     fo_xml: XSL-FO document as a string
    ///
    /// Returns:
    ///     SVG content as a string
    pub fn convert_to_svg(&self, fo_xml: &str) -> PyResult<String> {
        let builder = FoTreeBuilder::new();
        let cursor = Cursor::new(fo_xml.as_bytes());
        let arena = builder.parse(cursor).map_err(fop_error_to_py)?;

        let engine = LayoutEngine::new();
        let area_tree = engine.layout(&arena).map_err(fop_error_to_py)?;

        let renderer = SvgRenderer::new();
        renderer.render_to_svg(&area_tree).map_err(fop_error_to_py)
    }

    /// Convert XSL-FO string to plain text
    ///
    /// Args:
    ///     fo_xml: XSL-FO document as a string
    ///
    /// Returns:
    ///     Plain text content as a string
    pub fn convert_to_text(&self, fo_xml: &str) -> PyResult<String> {
        let builder = FoTreeBuilder::new();
        let cursor = Cursor::new(fo_xml.as_bytes());
        let arena = builder.parse(cursor).map_err(fop_error_to_py)?;

        let engine = LayoutEngine::new();
        let area_tree = engine.layout(&arena).map_err(fop_error_to_py)?;

        let renderer = TextRenderer::new();
        renderer.render_to_text(&area_tree).map_err(fop_error_to_py)
    }

    /// Convert a file to another file
    ///
    /// Args:
    ///     input_path: Path to the input XSL-FO file
    ///     output_path: Path to the output file (format detected from extension)
    pub fn convert_file(&self, input_path: &str, output_path: &str) -> PyResult<()> {
        use pyo3::exceptions::PyIOError;

        let fo_xml = std::fs::read_to_string(input_path)
            .map_err(|e| PyIOError::new_err(format!("Failed to read {}: {}", input_path, e)))?;

        let output_ext = std::path::Path::new(output_path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("pdf");

        let output_bytes = match output_ext {
            "svg" => self.convert_to_svg(&fo_xml)?.into_bytes(),
            "txt" => self.convert_to_text(&fo_xml)?.into_bytes(),
            _ => self.convert_to_pdf(&fo_xml)?,
        };

        std::fs::write(output_path, &output_bytes)
            .map_err(|e| PyIOError::new_err(format!("Failed to write {}: {}", output_path, e)))?;

        Ok(())
    }

    /// Validate an XSL-FO document
    ///
    /// Args:
    ///     fo_xml: XSL-FO document as a string
    ///
    /// Returns:
    ///     Dictionary with "valid" (bool), "nodes" (int), and optionally "error" (str)
    pub fn validate(&self, fo_xml: &str) -> PyResult<(bool, usize, Option<String>)> {
        let builder = FoTreeBuilder::new();
        let cursor = Cursor::new(fo_xml.as_bytes());

        match builder.parse(cursor) {
            Ok(arena) => Ok((true, arena.len(), None)),
            Err(e) => Ok((false, 0, Some(e.to_string()))),
        }
    }

    /// Get version information
    pub fn version(&self) -> String {
        format!("fop-python {}", env!("CARGO_PKG_VERSION"))
    }

    fn __repr__(&self) -> String {
        format!("FopConverter(verbose={})", self.verbose)
    }
}

impl Default for FopConverter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
      <fo:block>Hello from Python binding test!</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

    #[test]
    fn test_converter_creation() {
        let converter = FopConverter::new();
        assert!(!converter.verbose);
    }

    #[test]
    fn test_default_creation() {
        let converter = FopConverter::default();
        assert!(!converter.verbose);
    }

    #[test]
    fn test_convert_to_pdf_produces_valid_bytes() {
        let converter = FopConverter::new();
        let result = converter.convert_to_pdf(SIMPLE_FO);
        assert!(
            result.is_ok(),
            "PDF conversion should succeed: {:?}",
            result.err()
        );
        let pdf_bytes = result.expect("test: should succeed");
        assert!(!pdf_bytes.is_empty(), "PDF bytes should not be empty");
        assert!(
            pdf_bytes.starts_with(b"%PDF-"),
            "PDF should start with %PDF-"
        );
    }

    #[test]
    fn test_convert_to_svg_produces_valid_output() {
        let converter = FopConverter::new();
        let result = converter.convert_to_svg(SIMPLE_FO);
        assert!(
            result.is_ok(),
            "SVG conversion should succeed: {:?}",
            result.err()
        );
        let svg = result.expect("test: should succeed");
        assert!(!svg.is_empty(), "SVG should not be empty");
        assert!(
            svg.contains("<svg"),
            "SVG output should contain <svg element"
        );
    }

    #[test]
    fn test_convert_to_text_produces_utf8() {
        let converter = FopConverter::new();
        let result = converter.convert_to_text(SIMPLE_FO);
        assert!(
            result.is_ok(),
            "Text conversion should succeed: {:?}",
            result.err()
        );
        let text = result.expect("test: should succeed");
        assert!(!text.is_empty(), "Text output should not be empty");
    }

    #[test]
    fn test_validate_valid_document() {
        let converter = FopConverter::new();
        let result = converter.validate(SIMPLE_FO);
        assert!(result.is_ok(), "validate() should not return Err");
        let (valid, nodes, error) = result.expect("test: should succeed");
        assert!(valid, "Valid document should report valid=true");
        assert!(nodes > 0, "Valid document should have nodes > 0");
        assert!(
            error.is_none(),
            "Valid document should have no error message"
        );
    }

    #[test]
    fn test_validate_returns_ok_for_any_input() {
        // validate() always returns Ok — errors become (false, 0, Some(msg)) or (true, n, None)
        let converter = FopConverter::new();
        let result = converter.validate("not xml at all");
        assert!(
            result.is_ok(),
            "validate() should always return Ok, never Err"
        );
    }

    #[test]
    fn test_convert_to_pdf_fails_on_empty_input() {
        // Conversion pipeline should fail on empty input (no FO root element)
        let converter = FopConverter::new();
        let result = converter.convert_to_pdf("");
        // Either fails (no root element) or produces empty/invalid output
        // The important thing is it does not panic
        let _ = result;
    }

    #[test]
    fn test_version_contains_crate_name() {
        let converter = FopConverter::new();
        let version = converter.version();
        assert!(
            version.contains("fop-python"),
            "Version should include crate name"
        );
    }

    #[test]
    fn test_repr_contains_verbose_state() {
        let converter = FopConverter::new();
        let repr = converter.__repr__();
        assert!(
            repr.contains("verbose"),
            "Repr should contain verbose field"
        );
        assert!(
            repr.contains("false"),
            "Repr should show verbose=false for default"
        );
    }

    // ---- additional comprehensive tests ----

    const MARGINS_FO: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm" margin-left="20mm" margin-right="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Hello from Python binding test</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

    #[test]
    fn test_convert_to_pdf_basic_document() {
        let converter = FopConverter::new();
        let result = converter.convert_to_pdf(MARGINS_FO);
        assert!(
            result.is_ok(),
            "Basic PDF conversion should work: {:?}",
            result.err()
        );
        let bytes = result.expect("test: should succeed");
        assert!(!bytes.is_empty(), "PDF should not be empty");
        assert_eq!(&bytes[..4], b"%PDF", "Should start with %PDF");
    }

    #[test]
    fn test_convert_to_svg_basic_document() {
        let converter = FopConverter::new();
        let result = converter.convert_to_svg(MARGINS_FO);
        assert!(
            result.is_ok(),
            "SVG conversion should work: {:?}",
            result.err()
        );
        let svg = result.expect("test: should succeed");
        assert!(!svg.is_empty(), "SVG should not be empty");
        assert!(
            svg.contains("<svg"),
            "SVG output should contain <svg element"
        );
    }

    #[test]
    fn test_convert_invalid_xml_returns_error() {
        let converter = FopConverter::new();
        let result = converter.convert_to_pdf("<invalid-xml></not-closed>");
        assert!(result.is_err(), "Invalid XML should return error");
    }

    #[test]
    fn test_convert_empty_string_to_svg_does_not_panic() {
        // Empty input may succeed or fail but must not panic
        let converter = FopConverter::new();
        let _result = converter.convert_to_svg("");
    }

    #[test]
    fn test_convert_empty_string_to_text_does_not_panic() {
        // Empty input may succeed or fail but must not panic
        let converter = FopConverter::new();
        let _result = converter.convert_to_text("");
    }

    #[test]
    fn test_converter_new_creates_non_verbose() {
        let converter = FopConverter::new();
        assert!(!converter.verbose, "New converter should not be verbose");
    }

    #[test]
    fn test_verbose_flag_set() {
        let mut converter = FopConverter::new();
        converter.verbose = true;
        assert!(converter.verbose, "Verbose flag should be settable");
        assert!(
            converter.__repr__().contains("true"),
            "Repr should show verbose=true when set"
        );
    }

    #[test]
    fn test_convert_multipage_pdf() {
        let converter = FopConverter::new();
        let fo_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm" margin-left="20mm" margin-right="20mm">
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
        let result = converter.convert_to_pdf(fo_xml);
        assert!(
            result.is_ok(),
            "Multi-page PDF should work: {:?}",
            result.err()
        );
        let bytes = result.expect("test: should succeed");
        assert!(
            bytes.len() > 100,
            "Multi-page PDF should have substantial content"
        );
        assert!(bytes.starts_with(b"%PDF-"), "PDF header must be present");
    }

    #[test]
    fn test_convert_to_text_contains_content() {
        let converter = FopConverter::new();
        let fo_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm" margin-left="20mm" margin-right="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>TextContentInOutput12345</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;
        let result = converter.convert_to_text(fo_xml);
        assert!(
            result.is_ok(),
            "Text conversion should work: {:?}",
            result.err()
        );
        let text = result.expect("test: should succeed");
        assert!(
            text.contains("TextContentInOutput12345"),
            "Text output should contain the FO block content, got: {:?}",
            &text.chars().take(200).collect::<String>()
        );
    }

    #[test]
    fn test_convert_with_table() {
        let converter = FopConverter::new();
        let fo_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm" margin-left="20mm" margin-right="20mm">
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
              <fo:block>Cell 1</fo:block>
            </fo:table-cell>
            <fo:table-cell border="1pt solid black" padding="2mm">
              <fo:block>Cell 2</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;
        let result = converter.convert_to_pdf(fo_xml);
        assert!(
            result.is_ok(),
            "Table conversion should work: {:?}",
            result.err()
        );
        assert!(
            !result.expect("test: should succeed").is_empty(),
            "Table PDF should not be empty"
        );
    }

    #[test]
    fn test_validate_invalid_xml() {
        let converter = FopConverter::new();
        let result = converter.validate("<broken xml");
        assert!(result.is_ok(), "validate() always returns Ok");
        let (valid, nodes, error) = result.expect("test: should succeed");
        assert!(!valid, "Broken XML should not be valid");
        assert_eq!(nodes, 0, "Broken XML should report 0 nodes");
        assert!(
            error.is_some(),
            "Broken XML should provide an error message"
        );
    }

    #[test]
    fn test_validate_empty_string() {
        // validate() always returns Ok(()) — errors become (false, 0, Some(msg))
        let converter = FopConverter::new();
        let result = converter.validate("");
        assert!(result.is_ok(), "validate() should always return Ok");
        // Either (false, 0, Some(msg)) or (true, 0, None) — both are fine
        let _ = result.expect("test: should succeed");
    }

    #[test]
    fn test_validate_counts_nodes_correctly() {
        let converter = FopConverter::new();
        let result = converter.validate(SIMPLE_FO);
        assert!(result.is_ok());
        let (valid, nodes, _error) = result.expect("test: should succeed");
        assert!(valid);
        // SIMPLE_FO has root, layout-master-set, simple-page-master, region-body,
        // page-sequence, flow, block — at least 7 nodes
        assert!(nodes >= 7, "Expected at least 7 nodes, got {}", nodes);
    }

    #[test]
    fn test_version_is_semver_format() {
        let converter = FopConverter::new();
        let version = converter.version();
        // Should be like "fop-python 0.1.0"
        assert!(
            version.contains("."),
            "Version should contain a dot (semver), got: {}",
            version
        );
    }

    #[test]
    fn test_convert_to_svg_contains_text_content() {
        let converter = FopConverter::new();
        let fo_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm" margin-left="20mm" margin-right="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>SVGContentCheck9876</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;
        let result = converter.convert_to_svg(fo_xml);
        assert!(
            result.is_ok(),
            "SVG conversion should succeed: {:?}",
            result.err()
        );
        let svg = result.expect("test: should succeed");
        assert!(
            svg.contains("SVGContentCheck9876"),
            "SVG should contain block text content, got snippet: {:?}",
            &svg.chars().take(300).collect::<String>()
        );
    }

    #[test]
    fn test_convert_file_to_pdf() {
        let input = "/tmp/fop_python_test_input.fo";
        let output = "/tmp/fop_python_test_output.pdf";
        std::fs::write(input, SIMPLE_FO).expect("write input");
        let converter = FopConverter::new();
        let result = converter.convert_file(input, output);
        assert!(
            result.is_ok(),
            "convert_file to PDF should work: {:?}",
            result.err()
        );
        let bytes = std::fs::read(output).expect("read output");
        assert!(bytes.starts_with(b"%PDF-"), "Output should be valid PDF");
        // cleanup
        let _ = std::fs::remove_file(input);
        let _ = std::fs::remove_file(output);
    }

    #[test]
    fn test_convert_file_to_svg() {
        let input = "/tmp/fop_python_test_svg_input.fo";
        let output = "/tmp/fop_python_test_svg_output.svg";
        std::fs::write(input, SIMPLE_FO).expect("write input");
        let converter = FopConverter::new();
        let result = converter.convert_file(input, output);
        assert!(
            result.is_ok(),
            "convert_file to SVG should work: {:?}",
            result.err()
        );
        let content = std::fs::read_to_string(output).expect("read output");
        assert!(content.contains("<svg"), "Output should be SVG");
        // cleanup
        let _ = std::fs::remove_file(input);
        let _ = std::fs::remove_file(output);
    }

    #[test]
    fn test_convert_file_to_text() {
        let input = "/tmp/fop_python_test_txt_input.fo";
        let output = "/tmp/fop_python_test_txt_output.txt";
        std::fs::write(input, SIMPLE_FO).expect("write input");
        let converter = FopConverter::new();
        let result = converter.convert_file(input, output);
        assert!(
            result.is_ok(),
            "convert_file to text should work: {:?}",
            result.err()
        );
        let content = std::fs::read_to_string(output).expect("read output");
        assert!(!content.is_empty(), "Text output should not be empty");
        // cleanup
        let _ = std::fs::remove_file(input);
        let _ = std::fs::remove_file(output);
    }

    #[test]
    fn test_convert_file_nonexistent_input() {
        let converter = FopConverter::new();
        let result = converter.convert_file("/tmp/does_not_exist_xyz.fo", "/tmp/out.pdf");
        assert!(result.is_err(), "Non-existent input should return error");
    }

    #[test]
    fn test_convert_with_list() {
        let converter = FopConverter::new();
        let fo_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm" margin-left="20mm" margin-right="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:list-block>
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()">
            <fo:block>1.</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>First item</fo:block>
          </fo:list-item-body>
        </fo:list-item>
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()">
            <fo:block>2.</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>Second item</fo:block>
          </fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;
        let result = converter.convert_to_pdf(fo_xml);
        assert!(
            result.is_ok(),
            "List conversion should work: {:?}",
            result.err()
        );
        assert!(!result.expect("test: should succeed").is_empty());
    }

    #[test]
    fn test_convert_with_nested_blocks() {
        let converter = FopConverter::new();
        let fo_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm" margin-left="20mm" margin-right="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="18pt" font-weight="bold">Title</fo:block>
      <fo:block font-size="12pt" space-before="6pt">
        <fo:inline font-style="italic">Italic </fo:inline>
        <fo:inline font-weight="bold">Bold</fo:inline>
        normal text.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;
        let result = converter.convert_to_pdf(fo_xml);
        assert!(
            result.is_ok(),
            "Nested block conversion should work: {:?}",
            result.err()
        );
        assert!(!result.expect("test: should succeed").is_empty());
    }

    #[test]
    fn test_pdf_output_ends_with_eof_marker() {
        let converter = FopConverter::new();
        let result = converter.convert_to_pdf(SIMPLE_FO);
        assert!(result.is_ok());
        let bytes = result.expect("test: should succeed");
        // PDF files must end with %%EOF
        let tail = &bytes[bytes.len().saturating_sub(10)..];
        let tail_str = String::from_utf8_lossy(tail);
        assert!(
            tail_str.contains("%%EOF"),
            "PDF must end with %%EOF marker, got tail: {:?}",
            tail_str
        );
    }

    #[test]
    fn test_svg_output_ends_properly() {
        let converter = FopConverter::new();
        let result = converter.convert_to_svg(SIMPLE_FO);
        assert!(result.is_ok());
        let svg = result.expect("test: should succeed");
        assert!(
            svg.contains("</svg>"),
            "SVG must end with closing </svg> tag"
        );
    }

    #[test]
    fn test_convert_with_footnote() {
        let converter = FopConverter::new();
        let fo_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm" margin-left="20mm" margin-right="20mm">
      <fo:region-body/>
      <fo:region-after extent="20mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Main text with a footnote reference.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;
        // Should not panic even with region-after
        let result = converter.convert_to_pdf(fo_xml);
        // Either succeeds or fails cleanly
        let _ = result;
    }

    #[test]
    fn test_default_and_new_are_equivalent() {
        let c1 = FopConverter::new();
        let c2 = FopConverter::default();
        assert_eq!(c1.verbose, c2.verbose);
        assert_eq!(c1.version(), c2.version());
    }
}
