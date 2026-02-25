//! XSL-FO 1.1 Conformance: Extended table tests, page number formatting, table-and-caption
//!
//! Part of the XSL-FO 1.1 conformance test suite.
//! Reference: https://www.w3.org/TR/xsl11/

use super::{process_fo_document, validate_pdf_bytes};

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

fn check_fo(fo: &str) -> Vec<u8> {
    process_fo_document(fo).unwrap_or_else(|e| panic!("XSL-FO processing failed: {}", e))
}

fn wrap_fo(body: &str) -> String {
    format!(
        r##"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
        page-width="210mm" page-height="297mm"
        margin-top="20mm" margin-bottom="20mm"
        margin-left="25mm" margin-right="25mm">
      <fo:region-body region-name="xsl-region-body"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      {}
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        body
    )
}

// ---------------------------------------------------------------------------
// Section 9: Table formatting objects - extended tests
// ---------------------------------------------------------------------------

#[test]
fn conformance_table_row_explicit_height() {
    // fo:table-row with explicit block-progression-dimension (Section 9.3.3)
    let body = r##"
      <fo:table table-layout="fixed" width="160mm">
        <fo:table-column column-width="80mm"/>
        <fo:table-column column-width="80mm"/>
        <fo:table-body>
          <fo:table-row block-progression-dimension="20mm">
            <fo:table-cell border="1pt solid black" padding="2mm">
              <fo:block>Tall row cell 1</fo:block>
            </fo:table-cell>
            <fo:table-cell border="1pt solid black" padding="2mm">
              <fo:block>Tall row cell 2</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell border="1pt solid black" padding="2mm">
              <fo:block>Normal height row</fo:block>
            </fo:table-cell>
            <fo:table-cell border="1pt solid black" padding="2mm">
              <fo:block>Normal row 2</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_table_cell_display_align() {
    // fo:table-cell display-align for vertical alignment (Section 9.3.4)
    let body = r##"
      <fo:table table-layout="fixed" width="150mm">
        <fo:table-column column-width="50mm"/>
        <fo:table-column column-width="50mm"/>
        <fo:table-column column-width="50mm"/>
        <fo:table-body>
          <fo:table-row block-progression-dimension="30mm">
            <fo:table-cell display-align="before" border="1pt solid black" padding="2mm">
              <fo:block>Top aligned</fo:block>
            </fo:table-cell>
            <fo:table-cell display-align="center" border="1pt solid black" padding="2mm">
              <fo:block>Center aligned</fo:block>
            </fo:table-cell>
            <fo:table-cell display-align="after" border="1pt solid black" padding="2mm">
              <fo:block>Bottom aligned</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_table_border_spacing() {
    // fo:table border-spacing with separate border model (Section 9.3.1)
    let body = r##"
      <fo:table table-layout="fixed" width="160mm" border-collapse="separate" border-spacing="3mm">
        <fo:table-column column-width="80mm"/>
        <fo:table-column column-width="80mm"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell border="1pt solid black" padding="2mm">
              <fo:block>Cell with spacing 1</fo:block>
            </fo:table-cell>
            <fo:table-cell border="1pt solid black" padding="2mm">
              <fo:block>Cell with spacing 2</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell border="1pt solid black" padding="2mm">
              <fo:block>Row 2 cell 1</fo:block>
            </fo:table-cell>
            <fo:table-cell border="1pt solid black" padding="2mm">
              <fo:block>Row 2 cell 2</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

// ---------------------------------------------------------------------------
// Section 7.8: Page number formatting
// ---------------------------------------------------------------------------

#[test]
fn conformance_page_number_roman_numerals() {
    // fo:page-sequence format="i" for Roman numeral page numbering (Section 7.8)
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
  <fo:page-sequence master-reference="A4" format="i" initial-page-number="1">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Introduction - page <fo:page-number/></fo:block>
      <fo:block break-before="page">Page ii content</fo:block>
      <fo:block break-before="page">Page iii content</fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="A4" format="1" initial-page-number="1">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Chapter 1 - page <fo:page-number/></fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Roman numeral page numbers should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_page_number_alphabetic() {
    // fo:page-sequence format="A" for alphabetic page numbering
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
  <fo:page-sequence master-reference="A4" format="A">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Appendix <fo:page-number/></fo:block>
      <fo:block break-before="page">Appendix B content</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Alphabetic page numbers should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_format_page_number_helper() {
    // Unit test for format_page_number via public API: upper Roman numerals
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
  <fo:page-sequence master-reference="A4" format="I">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Roman upper page <fo:page-number/></fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Upper Roman page numbers should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_block_span_all() {
    // fo:block span="all" spanning multiple columns (Section 7.6.1)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body column-count="2" column-gap="5mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block span="all" font-size="16pt" font-weight="bold" space-after="10pt">
        This heading spans all columns
      </fo:block>
      <fo:block>Column 1 content paragraph 1. This text fills the first column of the two-column layout.</fo:block>
      <fo:block>Column 1 content paragraph 2. More content in columns.</fo:block>
      <fo:block span="all" font-size="12pt" font-weight="bold" space-before="10pt" space-after="5pt">
        Another spanning section heading
      </fo:block>
      <fo:block>More column content after the second spanning heading.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "block span=all should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_block_span_three_columns() {
    // fo:block span="all" in a 3-column layout
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="15mm" margin-right="15mm">
      <fo:region-body column-count="3" column-gap="4mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block span="all" font-size="18pt" text-align="center" space-after="8pt">
        Three Column Document
      </fo:block>
      <fo:block>First column text. This content appears in the first of three columns.</fo:block>
      <fo:block>More first column text with additional content to fill space.</fo:block>
      <fo:block>Second paragraph in column flow.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "block span=all in 3-column should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_block_span_none_default() {
    // fo:block span="none" (default) in multi-column stays in column (Section 7.6.1)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body column-count="2" column-gap="5mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block span="none">Normal column block 1</fo:block>
      <fo:block span="none">Normal column block 2</fo:block>
      <fo:block>Block without span attribute (default none)</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "block span=none should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_complex_nested_structure() {
    // Complex nesting: list inside table cell
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
      <fo:table table-layout="fixed" width="160mm">
        <fo:table-column column-width="80mm"/>
        <fo:table-column column-width="80mm"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell padding="3mm" border="1pt solid gray">
              <fo:block font-weight="bold">Nested content</fo:block>
              <fo:list-block>
                <fo:list-item>
                  <fo:list-item-label end-indent="label-end()">
                    <fo:block>&#x2022;</fo:block>
                  </fo:list-item-label>
                  <fo:list-item-body start-indent="body-start()">
                    <fo:block>Item in table cell</fo:block>
                  </fo:list-item-body>
                </fo:list-item>
              </fo:list-block>
            </fo:table-cell>
            <fo:table-cell padding="3mm" border="1pt solid gray">
              <fo:block>Simple cell with <fo:inline font-weight="bold">bold text</fo:inline></fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Complex nested structure should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_force_page_count_even_odd() {
    // fo:page-sequence force-page-count attribute (Section 7.8)
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
  <fo:page-sequence master-reference="A4" force-page-count="even">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Chapter 1 content. This sequence must end on an even page.</fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="A4" force-page-count="no-force">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Chapter 2 content. No forced page count.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "force-page-count should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_multiple_flows_static() {
    // Multiple static-content flows on same page (header + footer)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="25mm" margin-bottom="25mm"
      margin-left="25mm" margin-right="25mm">
      <fo:region-body margin-top="15mm" margin-bottom="15mm"/>
      <fo:region-before extent="15mm"/>
      <fo:region-after extent="15mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block text-align="center" font-size="9pt" font-style="italic">
        Document Title - <fo:page-number/>
      </fo:block>
    </fo:static-content>
    <fo:static-content flow-name="xsl-region-after">
      <fo:block text-align="center" font-size="9pt">
        Page <fo:page-number/>
      </fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="16pt" font-weight="bold" space-after="12pt">
        Main Document Content
      </fo:block>
      <fo:block>Paragraph one of the document body text content.</fo:block>
      <fo:block space-before="6pt">Paragraph two continues the document.</fo:block>
      <fo:block id="doc-end">End of document.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Multiple static-content flows should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_linefeed_treatment() {
    // fo:block linefeed-treatment attribute (Section 7.15.9)
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
      <fo:block linefeed-treatment="treat-as-space">
        Line one content
        Line two content
        Line three content
      </fo:block>
      <fo:block linefeed-treatment="preserve" white-space-treatment="preserve">
        Preserved line one
        Preserved line two
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "linefeed-treatment should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_instream_foreign_object() {
    // fo:instream-foreign-object with SVG content (Section 7.19)
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
      <fo:block>Document with embedded SVG graphic:</fo:block>
      <fo:block>
        <fo:instream-foreign-object content-width="80mm" content-height="60mm">
          <svg:svg xmlns:svg="http://www.w3.org/2000/svg"
            width="80mm" height="60mm">
            <svg:rect x="10" y="10" width="60" height="40" fill="blue" stroke="black"/>
            <svg:text x="20" y="30" fill="white">SVG in FO</svg:text>
          </svg:svg>
        </fo:instream-foreign-object>
      </fo:block>
      <fo:block>Content continues after SVG.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "instream-foreign-object should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_instream_foreign_object_scaling() {
    // fo:instream-foreign-object with scaling="uniform" (Section 7.19)
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
        <fo:instream-foreign-object content-width="40mm" content-height="40mm"
          scaling="uniform">
          <svg:svg xmlns:svg="http://www.w3.org/2000/svg" width="100" height="100">
            <svg:circle cx="50" cy="50" r="45" fill="red"/>
          </svg:svg>
        </fo:instream-foreign-object>
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "instream-foreign-object scaling should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_instream_foreign_object_mathml() {
    // fo:instream-foreign-object with MathML-like content
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
      <fo:block>Formula:</fo:block>
      <fo:block>
        <fo:instream-foreign-object content-width="50mm" content-height="20mm">
          <m:math xmlns:m="http://www.w3.org/1998/Math/MathML">
            <m:mrow>
              <m:mi>E</m:mi><m:mo>=</m:mo>
              <m:mi>m</m:mi><m:msup><m:mi>c</m:mi><m:mn>2</m:mn></m:msup>
            </m:mrow>
          </m:math>
        </fo:instream-foreign-object>
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "instream-foreign-object MathML should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

// ---------------------------------------------------------------------------
// Section 9: fo:table-and-caption with caption-side
// ---------------------------------------------------------------------------

#[test]
fn conformance_table_with_caption_before() {
    // fo:table-and-caption with caption before table (caption-side="before", which is the default)
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
      <fo:table-and-caption>
        <fo:table-caption>
          <fo:block font-style="italic" text-align="center" space-after="4pt">
            Table 1: Sample Data with Caption
          </fo:block>
        </fo:table-caption>
        <fo:table table-layout="fixed" width="160mm">
          <fo:table-column column-width="80mm"/>
          <fo:table-column column-width="80mm"/>
          <fo:table-body>
            <fo:table-row>
              <fo:table-cell padding="2mm" border="1pt solid black">
                <fo:block>Data A</fo:block>
              </fo:table-cell>
              <fo:table-cell padding="2mm" border="1pt solid black">
                <fo:block>Data B</fo:block>
              </fo:table-cell>
            </fo:table-row>
          </fo:table-body>
        </fo:table>
      </fo:table-and-caption>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "table-with-caption-before should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_table_with_caption_after() {
    // fo:table-and-caption with caption after table (caption-side="after")
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
      <fo:block>Introduction text before table.</fo:block>
      <fo:table-and-caption caption-side="after">
        <fo:table table-layout="fixed" width="160mm">
          <fo:table-column column-width="53mm"/>
          <fo:table-column column-width="53mm"/>
          <fo:table-column column-width="54mm"/>
          <fo:table-header>
            <fo:table-row>
              <fo:table-cell padding="2mm" border="1pt solid black" background-color="#cccccc">
                <fo:block font-weight="bold">Name</fo:block>
              </fo:table-cell>
              <fo:table-cell padding="2mm" border="1pt solid black" background-color="#cccccc">
                <fo:block font-weight="bold">Value</fo:block>
              </fo:table-cell>
              <fo:table-cell padding="2mm" border="1pt solid black" background-color="#cccccc">
                <fo:block font-weight="bold">Notes</fo:block>
              </fo:table-cell>
            </fo:table-row>
          </fo:table-header>
          <fo:table-body>
            <fo:table-row>
              <fo:table-cell padding="2mm" border="1pt solid black">
                <fo:block>Alpha</fo:block>
              </fo:table-cell>
              <fo:table-cell padding="2mm" border="1pt solid black">
                <fo:block>100</fo:block>
              </fo:table-cell>
              <fo:table-cell padding="2mm" border="1pt solid black">
                <fo:block>First item</fo:block>
              </fo:table-cell>
            </fo:table-row>
          </fo:table-body>
        </fo:table>
        <fo:table-caption>
          <fo:block font-size="9pt" text-align="center" space-before="4pt">
            Table 2: Results Summary (caption below table)
          </fo:block>
        </fo:table-caption>
      </fo:table-and-caption>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "table-with-caption-after should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_multiple_captioned_tables() {
    // Multiple fo:table-and-caption elements on a page
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
      <fo:table-and-caption space-after="12pt">
        <fo:table-caption>
          <fo:block font-weight="bold">Table A</fo:block>
        </fo:table-caption>
        <fo:table table-layout="fixed" width="100mm">
          <fo:table-column column-width="50mm"/>
          <fo:table-column column-width="50mm"/>
          <fo:table-body>
            <fo:table-row>
              <fo:table-cell padding="2mm" border="1pt solid black"><fo:block>A1</fo:block></fo:table-cell>
              <fo:table-cell padding="2mm" border="1pt solid black"><fo:block>A2</fo:block></fo:table-cell>
            </fo:table-row>
          </fo:table-body>
        </fo:table>
      </fo:table-and-caption>
      <fo:table-and-caption>
        <fo:table-caption>
          <fo:block font-weight="bold">Table B</fo:block>
        </fo:table-caption>
        <fo:table table-layout="fixed" width="100mm">
          <fo:table-column column-width="50mm"/>
          <fo:table-column column-width="50mm"/>
          <fo:table-body>
            <fo:table-row>
              <fo:table-cell padding="2mm" border="1pt solid black"><fo:block>B1</fo:block></fo:table-cell>
              <fo:table-cell padding="2mm" border="1pt solid black"><fo:block>B2</fo:block></fo:table-cell>
            </fo:table-row>
          </fo:table-body>
        </fo:table>
      </fo:table-and-caption>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "multiple captioned tables should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_page_number_grouping() {
    // fo:page-sequence grouping-separator and grouping-size
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
  <fo:page-sequence master-reference="A4" format="1"
    grouping-separator="," grouping-size="3" initial-page-number="1">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Page <fo:page-number/></fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Page number grouping should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_page_number_citation_format() {
    // fo:page-number-citation preserves format of target sequence
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
  <fo:page-sequence master-reference="A4" format="i" initial-page-number="1">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Introduction page <fo:page-number/></fo:block>
      <fo:block id="intro-ref" break-before="page">Referenced page: <fo:page-number/></fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="A4" format="1" initial-page-number="1">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Reference to intro: <fo:page-number-citation ref-id="intro-ref"/></fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "page-number-citation with mixed formats: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_page_number_initial_values() {
    // fo:page-sequence initial-page-number="auto" vs explicit value
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
  <fo:page-sequence master-reference="A4" initial-page-number="5">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Starting at page 5: <fo:page-number/></fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="A4" initial-page-number="auto">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Auto continues from previous: <fo:page-number/></fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "initial-page-number variants should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_document_with_all_page_features() {
    // Complete document using multiple page number format features
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body margin-top="12mm"/>
      <fo:region-before extent="12mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4" format="i" initial-page-number="1">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block text-align="center" font-size="9pt">- <fo:page-number/> -</fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="14pt" font-weight="bold">Preface</fo:block>
      <fo:block>Preface content with roman numeral page numbers.</fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="A4" format="1" initial-page-number="1">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block text-align="center" font-size="9pt">Page <fo:page-number/></fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="14pt" font-weight="bold">Chapter 1</fo:block>
      <fo:block>Main body with decimal page numbers starting from 1.</fo:block>
      <fo:block break-before="page">Chapter 1 continues on page <fo:page-number/>.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Complete page format document should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_pdf_document_metadata() {
    // PDF metadata from FO properties
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format" xml:lang="en">
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
      <fo:block xml:lang="en">English language document content with xml:lang specified.</fo:block>
      <fo:block xml:lang="ja">日本語テキスト内容</fo:block>
      <fo:block xml:lang="fr">Contenu en français</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "PDF metadata with xml:lang should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_role_attribute_accessibility() {
    // fo:block/inline role attribute for PDF accessibility (Section 5.3)
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
      <fo:block role="H1" font-size="18pt" font-weight="bold">Main Heading</fo:block>
      <fo:block role="P">Paragraph with role attribute for accessibility.</fo:block>
      <fo:block role="H2" font-size="14pt" font-weight="bold">Subheading</fo:block>
      <fo:block role="P">
        Text with <fo:inline role="Strong" font-weight="bold">strong emphasis</fo:inline>
        and <fo:inline role="Em" font-style="italic">emphasized text</fo:inline>.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "role attribute should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_source_document_info() {
    // fo:page-sequence source-document attribute (Section 7.3)
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
  <fo:page-sequence master-reference="A4" source-document="doc1.xml" role="Document">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Document with source-document and role attributes.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "source-document attribute should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_declarations_color_profile() {
    // fo:declarations with fo:color-profile (Section 7.1.2)
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
  <fo:declarations>
    <fo:color-profile src="sRGB.icm" color-profile-name="sRGB"/>
  </fo:declarations>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block color="green">Text with color (ICC profile reference simulated).</fo:block>
      <fo:block>Normal text content.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "declarations with color-profile should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}
