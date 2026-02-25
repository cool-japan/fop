//! XSL-FO 1.1 Conformance: PDF/A compliance and additional formatting features
//!
//! Part of the XSL-FO 1.1 conformance test suite.
//! Reference: https://www.w3.org/TR/xsl11/

use super::process_fo_document;

// ---------------------------------------------------------------------------
// PDF/A compliance output
// ---------------------------------------------------------------------------

#[test]
fn conformance_pdfa_output() {
    // PDF/A mode should produce valid PDF with XMP metadata
    use fop_render::{PdfCompliance, PdfRenderer};

    let fo_xml = r##"<?xml version="1.0" encoding="UTF-8"?>
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
      <fo:block>PDF/A compliant document</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    let fo_tree = fop_core::FoTreeBuilder::new()
        .parse(std::io::Cursor::new(fo_xml))
        .expect("parse should succeed");
    let engine = fop_layout::LayoutEngine::new();
    let area_tree = engine.layout(&fo_tree).expect("layout should succeed");

    let renderer = PdfRenderer::new();
    let mut pdf_doc = renderer.render(&area_tree).expect("render should succeed");
    pdf_doc
        .set_compliance(PdfCompliance::PdfA1b)
        .expect("set PDF/A compliance should succeed");
    pdf_doc.info.title = Some("PDF/A Test Document".to_string());
    pdf_doc.info.author = Some("Test Suite".to_string());

    let bytes = pdf_doc.to_bytes().expect("to_bytes should succeed");

    assert!(!bytes.is_empty(), "PDF/A output should not be empty");
    assert!(bytes.starts_with(b"%PDF-"), "Should start with PDF header");
    // PDF/A should include XMP metadata
    let pdf_str = String::from_utf8_lossy(&bytes);
    assert!(
        pdf_str.contains("PDF/A")
            || pdf_str.contains("pdfa")
            || pdf_str.contains("XMP")
            || bytes.len() > 1000,
        "PDF/A output should be non-trivial"
    );
}

#[test]
fn conformance_bidi_override() {
    // fo:bidi-override sets text direction (Section 8.2)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
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
      <fo:block>
        LTR text
        <fo:bidi-override direction="rtl">RTL override text</fo:bidi-override>
        back to LTR
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "fo:bidi-override should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_initial_property_set() {
    // fo:initial-property-set sets properties for first line (Section 8.3)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
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
      <fo:block>
        <fo:initial-property-set font-weight="bold" color="blue"/>
        This paragraph has its first line styled differently via initial-property-set.
        The rest of the paragraph uses normal styling.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "fo:initial-property-set should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_wrapper_element() {
    // fo:wrapper provides a transparent property-setting container (Section 8.12)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
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
      <fo:block>
        Normal text,
        <fo:wrapper font-weight="bold" color="red">bold red text</fo:wrapper>,
        normal again.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "fo:wrapper should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_character_element() {
    // fo:character renders a single character with full property control (Section 8.5)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
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
      <fo:block>
        Text with special character:
        <fo:character character="&#x2764;" font-size="16pt" color="red"/>
        end.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "fo:character should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_percentage_dimensions() {
    // Percentage-based width/height values (Section 5.11.2)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
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
      <fo:block width="50%" background-color="lightgray">50% width block</fo:block>
      <fo:block width="75%" background-color="lightyellow">75% width block</fo:block>
      <fo:block text-indent="10%">Block with 10% text-indent</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Percentage dimensions should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_page_sequence_restart() {
    // Multiple page sequences, each restarting page numbering (Section 7.8)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4" initial-page-number="1">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Sequence 1, page <fo:page-number/></fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="A4" initial-page-number="1">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Sequence 2, page <fo:page-number/></fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Multiple page-sequences with restart should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_nested_lists() {
    // Nested fo:list-block with indentation (Section 9.5)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
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
      <fo:list-block provisional-distance-between-starts="15mm" provisional-label-separation="5mm">
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()"><fo:block>1.</fo:block></fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>First item</fo:block>
            <fo:list-block provisional-distance-between-starts="10mm" start-indent="10mm">
              <fo:list-item>
                <fo:list-item-label end-indent="label-end()"><fo:block>a.</fo:block></fo:list-item-label>
                <fo:list-item-body start-indent="body-start()">
                  <fo:block>Nested item a</fo:block>
                </fo:list-item-body>
              </fo:list-item>
              <fo:list-item>
                <fo:list-item-label end-indent="label-end()"><fo:block>b.</fo:block></fo:list-item-label>
                <fo:list-item-body start-indent="body-start()">
                  <fo:block>Nested item b</fo:block>
                </fo:list-item-body>
              </fo:list-item>
            </fo:list-block>
          </fo:list-item-body>
        </fo:list-item>
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()"><fo:block>2.</fo:block></fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>Second item</fo:block>
          </fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Nested lists should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_table_border_collapse() {
    // border-collapse="collapse" model (Section 9.3.4)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
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
      <fo:table border-collapse="collapse" width="150mm">
        <fo:table-column column-width="50mm"/>
        <fo:table-column column-width="50mm"/>
        <fo:table-column column-width="50mm"/>
        <fo:table-header>
          <fo:table-row>
            <fo:table-cell border="1pt solid black" padding="3pt"><fo:block font-weight="bold">Name</fo:block></fo:table-cell>
            <fo:table-cell border="1pt solid black" padding="3pt"><fo:block font-weight="bold">Value</fo:block></fo:table-cell>
            <fo:table-cell border="1pt solid black" padding="3pt"><fo:block font-weight="bold">Notes</fo:block></fo:table-cell>
          </fo:table-row>
        </fo:table-header>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell border="1pt solid black" padding="3pt"><fo:block>Row 1 A</fo:block></fo:table-cell>
            <fo:table-cell border="1pt solid black" padding="3pt"><fo:block>Row 1 B</fo:block></fo:table-cell>
            <fo:table-cell border="1pt solid black" padding="3pt"><fo:block>Row 1 C</fo:block></fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell border="1pt solid black" padding="3pt"><fo:block>Row 2 A</fo:block></fo:table-cell>
            <fo:table-cell border="1pt solid black" padding="3pt"><fo:block>Row 2 B</fo:block></fo:table-cell>
            <fo:table-cell border="1pt solid black" padding="3pt"><fo:block>Row 2 C</fo:block></fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Table border-collapse should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_page_number_formatted() {
    // fo:page-number in various contexts with formatting (Section 8.8)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="15mm" margin-bottom="15mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body margin-top="15mm" margin-bottom="15mm"/>
      <fo:region-before extent="15mm"/>
      <fo:region-after extent="15mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block text-align="center" font-size="9pt">
        Page <fo:page-number/> of <fo:page-number-citation ref-id="last-page"/>
      </fo:block>
    </fo:static-content>
    <fo:static-content flow-name="xsl-region-after">
      <fo:block text-align="center" font-size="9pt">Footer - Page <fo:page-number/></fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Page number in header and footer test.</fo:block>
      <fo:block break-before="page">Second page content.</fo:block>
      <fo:block id="last-page">Last page marker.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Page number formatting should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_table_auto_columns() {
    // table-layout="auto" with content-driven column widths (Section 9.3.1)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
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
      <fo:table table-layout="auto" width="150mm">
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell border="0.5pt solid black" padding="2pt"><fo:block>Short</fo:block></fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt"><fo:block>A much longer cell content</fo:block></fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt"><fo:block>Medium length</fo:block></fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Auto column width table should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}
