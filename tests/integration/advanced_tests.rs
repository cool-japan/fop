//! Advanced feature integration tests

use super::{load_fixture, process_fo_document, validate_pdf_bytes};

#[test]
fn test_lists_with_markers() {
    let fo_content = load_fixture("lists.fo");
    let pdf_bytes = process_fo_document(&fo_content).expect("Processing failed");

    validate_pdf_bytes(&pdf_bytes);

    // List markers should be rendered
    assert!(pdf_bytes.len() > 300);
}

#[test]
fn test_unicode_multilingual() {
    let fo_content = load_fixture("unicode.fo");
    let pdf_bytes = process_fo_document(&fo_content).expect("Processing failed");

    validate_pdf_bytes(&pdf_bytes);

    // Multiple languages and symbols
    assert!(pdf_bytes.len() > 400);
}

#[test]
fn test_complex_formatting() {
    // Test with text formatting fixture
    let fo_content = load_fixture("text_formatting.fo");
    let pdf_bytes = process_fo_document(&fo_content).expect("Processing failed");

    validate_pdf_bytes(&pdf_bytes);

    // Bold, italic, underline, colors
    assert!(pdf_bytes.len() > 300);
}

#[test]
fn test_property_inheritance() {
    // Nested blocks test property inheritance
    let fo_content = load_fixture("nested_blocks.fo");
    let pdf_bytes = process_fo_document(&fo_content).expect("Processing failed");

    validate_pdf_bytes(&pdf_bytes);

    // Nested blocks with inherited and overridden properties
    assert!(pdf_bytes.len() > 200);
}

#[test]
fn test_page_breaks() {
    // Multi-page document with explicit breaks
    let fo_content = load_fixture("multi_page.fo");
    let pdf_bytes = process_fo_document(&fo_content).expect("Processing failed");

    validate_pdf_bytes(&pdf_bytes);

    // Multiple pages means larger PDF
    assert!(pdf_bytes.len() > 300);
}

#[test]
fn test_minimal_document() {
    // Simplest possible valid document
    let fo_content = r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Test</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    let pdf_bytes = process_fo_document(fo_content).expect("Processing failed");
    validate_pdf_bytes(&pdf_bytes);
}

#[test]
fn test_complex_report() {
    let fo_content = load_fixture("complex_report.fo");
    let pdf_bytes = process_fo_document(&fo_content).expect("Processing failed");

    validate_pdf_bytes(&pdf_bytes);

    // Complex report with tables, lists, and formatting
    assert!(pdf_bytes.len() > 600);
}

#[test]
fn test_multi_column_layout() {
    let fo_content = load_fixture("multi_column.fo");
    let pdf_bytes = process_fo_document(&fo_content).expect("Processing failed");

    validate_pdf_bytes(&pdf_bytes);

    // Multi-column layout
    assert!(pdf_bytes.len() > 300);
}

#[test]
fn test_comprehensive_document() {
    // Create a document with multiple features
    let fo_content = r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
                          page-width="210mm"
                          page-height="297mm"
                          margin="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="18pt" font-weight="bold" color="#0000FF">
        Comprehensive Test Document
      </fo:block>

      <fo:block space-before="12pt">
        This document tests multiple features together.
      </fo:block>

      <fo:table space-before="12pt" border="1pt solid black">
        <fo:table-column column-width="50%"/>
        <fo:table-column column-width="50%"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell border="1pt solid black" padding="4pt">
              <fo:block>Table Cell 1</fo:block>
            </fo:table-cell>
            <fo:table-cell border="1pt solid black" padding="4pt">
              <fo:block>Table Cell 2</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>

      <fo:list-block provisional-distance-between-starts="18pt" space-before="12pt">
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()">
            <fo:block>•</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>List item with bullet</fo:block>
          </fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    let pdf_bytes = process_fo_document(fo_content).expect("Processing failed");
    validate_pdf_bytes(&pdf_bytes);

    // Comprehensive document should be substantial
    assert!(pdf_bytes.len() > 500);
}
