//! XSL-FO 1.1 Conformance: Page layout, overflow, gradients, output formats
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
// Additional XSL-FO 1.1 conformance tests: page layout and special features
// ---------------------------------------------------------------------------

#[test]
fn conformance_overflow_hidden() {
    // overflow="hidden" clips content to the area box (Section 7.22.4)
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
      <fo:block-container overflow="hidden" height="30mm" border="1pt solid black">
        <fo:block>This content is clipped to 30mm height.</fo:block>
        <fo:block>This line may be clipped if content overflows.</fo:block>
        <fo:block>Another line.</fo:block>
        <fo:block>Yet another line that overflows the container.</fo:block>
      </fo:block-container>
      <fo:block>Content after the clipped container.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "overflow=hidden should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_background_gradient() {
    // Linear gradient background (CSS extension)
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
      <fo:block background-color="#4a90d9" color="white" padding="10pt">
        Block with solid background
      </fo:block>
      <fo:block background-color="#f0f0f0" padding="8pt" margin-top="5pt">
        Block with gray background
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Background colors should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_multipage_complex_document() {
    // Complex multi-page document with headers, footers, and various content
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="content"
      page-width="210mm" page-height="297mm"
      margin-top="15mm" margin-bottom="15mm"
      margin-left="25mm" margin-right="25mm">
      <fo:region-body margin-top="20mm" margin-bottom="20mm"/>
      <fo:region-before extent="20mm"/>
      <fo:region-after extent="20mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="content">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block text-align="center" font-size="10pt" font-weight="bold">
        Technical Report - Page <fo:page-number/>
      </fo:block>
    </fo:static-content>
    <fo:static-content flow-name="xsl-region-after">
      <fo:block text-align="center" font-size="8pt" color="#666666">
        Confidential - Do Not Distribute
      </fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="18pt" font-weight="bold" space-after="10pt">Executive Summary</fo:block>
      <fo:block font-size="10pt" space-after="8pt">
        This document presents findings from the quarterly analysis.
      </fo:block>
      <fo:block font-size="14pt" font-weight="bold" space-before="12pt" space-after="6pt">
        Key Findings
      </fo:block>
      <fo:list-block provisional-distance-between-starts="10mm" space-after="8pt">
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()"><fo:block>&#x2022;</fo:block></fo:list-item-label>
          <fo:list-item-body start-indent="body-start()"><fo:block>Revenue increased 15% year-over-year</fo:block></fo:list-item-body>
        </fo:list-item>
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()"><fo:block>&#x2022;</fo:block></fo:list-item-label>
          <fo:list-item-body start-indent="body-start()"><fo:block>Customer satisfaction scores improved</fo:block></fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
      <fo:block font-size="14pt" font-weight="bold" space-before="12pt" space-after="6pt" break-before="page">
        Financial Summary
      </fo:block>
      <fo:table width="150mm" border-collapse="collapse">
        <fo:table-column column-width="75mm"/>
        <fo:table-column column-width="75mm"/>
        <fo:table-header>
          <fo:table-row background-color="#4a4a4a" color="white">
            <fo:table-cell padding="4pt" border="0.5pt solid white"><fo:block font-weight="bold">Category</fo:block></fo:table-cell>
            <fo:table-cell padding="4pt" border="0.5pt solid white"><fo:block font-weight="bold">Amount</fo:block></fo:table-cell>
          </fo:table-row>
        </fo:table-header>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell padding="3pt" border="0.5pt solid #cccccc"><fo:block>Total Revenue</fo:block></fo:table-cell>
            <fo:table-cell padding="3pt" border="0.5pt solid #cccccc"><fo:block>$2,450,000</fo:block></fo:table-cell>
          </fo:table-row>
          <fo:table-row background-color="#f9f9f9">
            <fo:table-cell padding="3pt" border="0.5pt solid #cccccc"><fo:block>Operating Costs</fo:block></fo:table-cell>
            <fo:table-cell padding="3pt" border="0.5pt solid #cccccc"><fo:block>$1,890,000</fo:block></fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell padding="3pt" border="0.5pt solid #cccccc"><fo:block font-weight="bold">Net Profit</fo:block></fo:table-cell>
            <fo:table-cell padding="3pt" border="0.5pt solid #cccccc"><fo:block font-weight="bold">$560,000</fo:block></fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Complex multi-page document should work: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty());
    // PDF should be a reasonable size for a 2-page document with tables
    assert!(bytes.len() > 1000, "PDF should have substantial content");
}

#[test]
fn conformance_xml_entities_and_special_chars() {
    // XML entities and special characters in content (Section 5.7)
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
      <fo:block>Standard XML entities: &amp; &lt; &gt; &quot; &apos;</fo:block>
      <fo:block>Numeric decimal: &#65; &#66; &#67;</fo:block>
      <fo:block>Numeric hex: &#x41; &#x42; &#x43;</fo:block>
      <fo:block>Unicode: &#x00A9; &#x2122; &#x00AE;</fo:block>
      <fo:block>Japanese: &#x65E5;&#x672C;&#x8A9E;</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "XML entities should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_mixed_writing_directions() {
    // Mixed writing-mode in a document (Section 7.28)
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
      <fo:block writing-mode="lr-tb">Left-to-right text direction.</fo:block>
      <fo:block writing-mode="lr-tb" text-align="right">Right-aligned LTR text.</fo:block>
      <fo:block>Mixed: English and &#x0639;&#x0631;&#x0628;&#x064A; text</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Mixed writing directions should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_svg_output() {
    // SVG renderer should produce valid SVG output
    use std::io::Cursor;

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
      <fo:block font-size="14pt" font-weight="bold">SVG Output Test</fo:block>
      <fo:block>This document is rendered to SVG format.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    let fo_tree = fop_core::FoTreeBuilder::new()
        .parse(Cursor::new(fo_xml))
        .expect("parse should succeed");
    let engine = fop_layout::LayoutEngine::new();
    let area_tree = engine.layout(&fo_tree).expect("layout should succeed");
    let renderer = fop_render::SvgRenderer::new();
    let svg = renderer
        .render_to_svg(&area_tree)
        .expect("SVG render should succeed");
    assert!(!svg.is_empty(), "SVG output should not be empty");
    assert!(
        svg.contains("<svg") || svg.contains("<?xml"),
        "Should be valid SVG/XML"
    );
}

#[test]
fn conformance_postscript_output() {
    // PostScript renderer should produce valid PS output
    use std::io::Cursor;

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
      <fo:block>PostScript Output Test</fo:block>
      <fo:block>Testing PS renderer output format.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    let fo_tree = fop_core::FoTreeBuilder::new()
        .parse(Cursor::new(fo_xml))
        .expect("parse should succeed");
    let engine = fop_layout::LayoutEngine::new();
    let area_tree = engine.layout(&fo_tree).expect("layout should succeed");

    let renderer = fop_render::PsRenderer::new();
    let ps_output = renderer
        .render_to_ps(&area_tree)
        .expect("PS render should succeed");
    assert!(!ps_output.is_empty(), "PS output should not be empty");
    assert!(
        ps_output.starts_with("%!PS") || ps_output.starts_with("%!"),
        "Should be valid PostScript, got: {}",
        &ps_output[..ps_output.len().min(50)]
    );
}

#[test]
fn conformance_text_output() {
    // Plain text renderer should extract readable text
    use std::io::Cursor;

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
      <fo:block>Hello from text renderer</fo:block>
      <fo:block>Second paragraph</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    let fo_tree = fop_core::FoTreeBuilder::new()
        .parse(Cursor::new(fo_xml))
        .expect("parse should succeed");
    let engine = fop_layout::LayoutEngine::new();
    let area_tree = engine.layout(&fo_tree).expect("layout should succeed");

    let renderer = fop_render::TextRenderer::new();
    let text = renderer
        .render_to_text(&area_tree)
        .expect("Text render should succeed");
    assert!(!text.is_empty(), "Text output should not be empty");
    assert!(
        text.contains("Hello") || text.contains("text"),
        "Should contain rendered text content, got: {}",
        &text[..text.len().min(200)]
    );
}

#[test]
fn conformance_proportional_column_width() {
    // proportional-column-width() distributes remaining space proportionally (Section 9.3.2)
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
      <fo:table table-layout="fixed" width="150mm">
        <fo:table-column column-width="proportional-column-width(1)"/>
        <fo:table-column column-width="proportional-column-width(2)"/>
        <fo:table-column column-width="proportional-column-width(1)"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell border="0.5pt solid black" padding="3pt"><fo:block>1 part</fo:block></fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="3pt"><fo:block>2 parts (wider)</fo:block></fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="3pt"><fo:block>1 part</fo:block></fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "proportional-column-width() should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_mixed_column_widths() {
    // Table with mix of fixed and auto column widths
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
      <fo:table width="150mm">
        <fo:table-column column-width="30mm"/>
        <fo:table-column column-number="2"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell border="0.5pt solid black" padding="3pt">
              <fo:block>Fixed 30mm</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="3pt">
              <fo:block>Auto width fills remaining space</fo:block>
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
        "Mixed column widths should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_table_alternating_row_colors() {
    // Alternating row background colors (zebra striping)
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
      <fo:table width="150mm">
        <fo:table-column column-width="75mm"/>
        <fo:table-column column-width="75mm"/>
        <fo:table-body>
          <fo:table-row background-color="#f0f0f0">
            <fo:table-cell padding="3pt"><fo:block>Row 1, Col A</fo:block></fo:table-cell>
            <fo:table-cell padding="3pt"><fo:block>Row 1, Col B</fo:block></fo:table-cell>
          </fo:table-row>
          <fo:table-row background-color="white">
            <fo:table-cell padding="3pt"><fo:block>Row 2, Col A</fo:block></fo:table-cell>
            <fo:table-cell padding="3pt"><fo:block>Row 2, Col B</fo:block></fo:table-cell>
          </fo:table-row>
          <fo:table-row background-color="#f0f0f0">
            <fo:table-cell padding="3pt"><fo:block>Row 3, Col A</fo:block></fo:table-cell>
            <fo:table-cell padding="3pt"><fo:block>Row 3, Col B</fo:block></fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Alternating row colors should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_nested_block_containers() {
    // Nested block containers with different properties (Section 8.14)
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
      <fo:block-container border="1pt solid gray" padding="5pt" background-color="#fafafa">
        <fo:block font-weight="bold">Outer container</fo:block>
        <fo:block-container border="1pt solid silver" padding="4pt" background-color="#f0f0f0" margin-top="5pt">
          <fo:block>Inner container level 1</fo:block>
          <fo:block-container border="1pt solid black" padding="3pt" background-color="#e8e8e8" margin-top="4pt">
            <fo:block font-style="italic">Innermost container</fo:block>
          </fo:block-container>
        </fo:block-container>
      </fo:block-container>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Nested block-containers should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_inline_container() {
    // fo:inline-container creates a block viewport within inline context (Section 8.13)
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
        Text before
        <fo:inline-container width="40mm" border="0.5pt solid black">
          <fo:block font-size="8pt">Inline block content</fo:block>
        </fo:inline-container>
        text after inline container.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "fo:inline-container should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_table_and_caption() {
    // fo:table-and-caption wraps a table with its caption (Section 9.3.3)
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
          <fo:block font-weight="bold" text-align="center">Table 1: Sample Data</fo:block>
        </fo:table-caption>
        <fo:table width="150mm">
          <fo:table-column column-width="75mm"/>
          <fo:table-column column-width="75mm"/>
          <fo:table-body>
            <fo:table-row>
              <fo:table-cell border="0.5pt solid black" padding="3pt"><fo:block>Cell A</fo:block></fo:table-cell>
              <fo:table-cell border="0.5pt solid black" padding="3pt"><fo:block>Cell B</fo:block></fo:table-cell>
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
        "fo:table-and-caption should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_text_transform() {
    // text-transform: uppercase, lowercase, capitalize (Section 7.17.6)
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
      <fo:block text-transform="uppercase">this will be uppercase</fo:block>
      <fo:block text-transform="lowercase">THIS WILL BE LOWERCASE</fo:block>
      <fo:block text-transform="capitalize">each word capitalized</fo:block>
      <fo:block text-transform="none">No transformation applied</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "text-transform should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_font_variant_small_caps() {
    // font-variant: small-caps renders text in small capital letters (Section 7.9.8)
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
      <fo:block font-variant="small-caps">Small Caps Text Example</fo:block>
      <fo:block font-variant="normal">Normal text for comparison</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "font-variant: small-caps should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_display_align() {
    // display-align: vertical alignment within block containers (Section 7.14.1)
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
      <fo:table width="150mm">
        <fo:table-column column-width="50mm"/>
        <fo:table-column column-width="50mm"/>
        <fo:table-column column-width="50mm"/>
        <fo:table-body>
          <fo:table-row height="30mm">
            <fo:table-cell display-align="before" border="0.5pt solid black"><fo:block>Top aligned</fo:block></fo:table-cell>
            <fo:table-cell display-align="center" border="0.5pt solid black"><fo:block>Center aligned</fo:block></fo:table-cell>
            <fo:table-cell display-align="after" border="0.5pt solid black"><fo:block>Bottom aligned</fo:block></fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "display-align should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_baseline_shift_superscript() {
    // baseline-shift: super/sub correctly offsets inline text (Section 7.14.3)
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
        x<fo:inline baseline-shift="super" font-size="7pt">2</fo:inline>
        + y<fo:inline baseline-shift="super" font-size="7pt">2</fo:inline>
        = r<fo:inline baseline-shift="super" font-size="7pt">2</fo:inline>
      </fo:block>
      <fo:block>
        H<fo:inline baseline-shift="sub" font-size="7pt">2</fo:inline>O
        + CO<fo:inline baseline-shift="sub" font-size="7pt">2</fo:inline>
      </fo:block>
      <fo:block>
        Normal baseline text with <fo:inline baseline-shift="0pt">zero shift</fo:inline>.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "baseline-shift super/sub should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_multi_switch() {
    // fo:multi-switch with fo:multi-case - first visible case is rendered (Section 11)
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
      <fo:block>Content before switch</fo:block>
      <fo:multi-switch>
        <fo:multi-case starting-state="visible">
          <fo:block color="blue">This case is shown (visible)</fo:block>
        </fo:multi-case>
        <fo:multi-case starting-state="hidden">
          <fo:block color="red">This case is hidden</fo:block>
        </fo:multi-case>
      </fo:multi-switch>
      <fo:block>Content after switch</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "fo:multi-switch should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_block_container_overflow_scroll() {
    // overflow="scroll" on block-container (Section 7.22.4)
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
      <fo:block-container height="40mm" overflow="scroll" border="1pt solid gray">
        <fo:block>Line 1</fo:block>
        <fo:block>Line 2</fo:block>
        <fo:block>Line 3</fo:block>
        <fo:block>Line 4 - may overflow</fo:block>
        <fo:block>Line 5 - definitely overflow</fo:block>
      </fo:block-container>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "overflow=scroll should be handled: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_space_specifier_precedence() {
    // space-before/after with conditionality (Section 4.3)
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
      <fo:block space-before="10pt" space-after="5pt">Block 1 with spacing</fo:block>
      <fo:block space-before="10pt" space-after="5pt">Block 2 with spacing</fo:block>
      <fo:block space-before.minimum="5pt" space-before.optimum="10pt" space-before.maximum="20pt">
        Block 3 with compound space
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Space specifiers should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_hyphenation_properties() {
    // hyphenate="true" enables automatic hyphenation (Section 7.10.1)
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
      <fo:block hyphenate="true" language="en" xml:lang="en">
        This is a paragraph with hyphenation enabled. Long words like
        "incomprehensibility" and "counterrevolutionary" may be hyphenated
        when they appear at the end of a line in justified text.
      </fo:block>
      <fo:block hyphenate="false">
        Hyphenation is disabled for this paragraph.
        Words will not be broken at line boundaries.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "hyphenate property should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_text_measurement_cjk() {
    // CJK characters take full-width in text measurement
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
      <fo:block font-size="12pt">&#x65E5;&#x672C;&#x8A9E;&#x30C6;&#x30AD;&#x30B9;&#x30C8; (Japanese text)</fo:block>
      <fo:block font-size="12pt">&#x4E2D;&#x6587;&#x6587;&#x672C; (Chinese text)</fo:block>
      <fo:block font-size="12pt">&#xD55C;&#xAD6D;&#xC5B4; &#xD14D;&#xC2A4;&#xD2B8; (Korean text)</fo:block>
      <fo:block font-size="10pt">Mixed: Hello &#x4e16;&#x754c; World</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "CJK text measurement should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_font_stretch() {
    // font-stretch property (Section 7.9.9)
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
      <fo:block font-stretch="condensed">Condensed text style</fo:block>
      <fo:block font-stretch="normal">Normal width text</fo:block>
      <fo:block font-stretch="expanded">Expanded text style</fo:block>
      <fo:block font-stretch="ultra-condensed">Ultra condensed</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "font-stretch should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_text_align_last() {
    // text-align-last controls alignment of last line in a justified block
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
      <fo:block text-align="justify" text-align-last="center">
        This paragraph is justified with the last line centered.
        The last line of this block should be centered.
      </fo:block>
      <fo:block text-align="justify" text-align-last="left">
        This paragraph is justified with the last line left-aligned.
        The last line should be flush left.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "text-align-last should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_page_number_citation_last() {
    // fo:page-number-citation-last references last page of an element (Section 8.10)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="15mm" margin-bottom="15mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body margin-top="12mm"/>
      <fo:region-before extent="12mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block font-size="9pt" text-align="center">
        Pages <fo:page-number/> of <fo:page-number-citation-last ref-id="doc-end"/>
      </fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block id="doc-start">Start of document</fo:block>
      <fo:block break-before="page">Page 2 content</fo:block>
      <fo:block id="doc-end">End of document</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "fo:page-number-citation-last should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_change_bar() {
    // fo:change-bar-begin/end marks changed content regions (Section 12)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="25mm" margin-right="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Original unchanged content.</fo:block>
      <fo:change-bar-begin change-bar-class="rev1" change-bar-color="red" change-bar-style="solid"/>
      <fo:block>This content has been changed (marked with change bar).</fo:block>
      <fo:block>More changed content on the next line.</fo:block>
      <fo:change-bar-end change-bar-class="rev1"/>
      <fo:block>Unchanged content after the revision.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "fo:change-bar should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_conditional_keep() {
    // keep-with-next and keep-with-previous preventing orphan headings (Section 7.19)
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
      <fo:block font-size="14pt" font-weight="bold" keep-with-next.within-page="always">
        Section Heading (keeps with following paragraph)
      </fo:block>
      <fo:block>
        This paragraph follows the heading and should always appear on the same page
        as the heading above it due to keep-with-next.
      </fo:block>
      <fo:block font-size="14pt" font-weight="bold"
                keep-with-next.within-page="always"
                space-before="12pt">
        Another Heading
      </fo:block>
      <fo:block>Following paragraph.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "keep-with-next should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_leader_patterns() {
    // fo:leader with different patterns (dots, rule, space, use-content) (Section 8.9)
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
      <fo:block text-align-last="justify">
        Chapter 1<fo:leader leader-pattern="dots"/>1
      </fo:block>
      <fo:block text-align-last="justify">
        Chapter 2<fo:leader leader-pattern="rule" rule-thickness="0.5pt"/>15
      </fo:block>
      <fo:block text-align-last="justify">
        Chapter 3<fo:leader leader-pattern="space"/>42
      </fo:block>
      <fo:block text-align-last="justify">
        Appendix A<fo:leader leader-pattern="dots" leader-length.minimum="10mm"/>99
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "fo:leader patterns should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_basic_link_with_styling() {
    // fo:basic-link with color and text-decoration properties (Section 7.10.2)
    let body = r##"
      <fo:block>
        Click <fo:basic-link external-destination="https://www.example.com"
          color="blue" text-decoration="underline">here</fo:basic-link> to visit the site.
      </fo:block>
      <fo:block space-before="6pt">
        Or go <fo:basic-link external-destination="https://www.example.org"
          color="#003366">there</fo:basic-link>.
      </fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_basic_link_in_table_of_contents() {
    // fo:basic-link used in a table of contents pattern with leader and page-number-citation
    // (Section 7.10.2)
    let body = r##"
      <fo:block font-weight="bold" font-size="14pt" space-after="10pt">Table of Contents</fo:block>
      <fo:block text-align-last="justify">
        <fo:basic-link internal-destination="sec1">Section 1: Introduction</fo:basic-link>
        <fo:leader leader-pattern="dots"/>
        <fo:page-number-citation ref-id="sec1"/>
      </fo:block>
      <fo:block text-align-last="justify">
        <fo:basic-link internal-destination="sec2">Section 2: Details</fo:basic-link>
        <fo:leader leader-pattern="dots"/>
        <fo:page-number-citation ref-id="sec2"/>
      </fo:block>
      <fo:block id="sec1" break-before="page" font-size="13pt" font-weight="bold">Section 1: Introduction</fo:block>
      <fo:block>Introduction content goes here.</fo:block>
      <fo:block id="sec2" break-before="page" font-size="13pt" font-weight="bold">Section 2: Details</fo:block>
      <fo:block>Detail content goes here.</fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_basic_link_multilink_block() {
    // Multiple fo:basic-link elements within a single block (Section 7.10.2)
    let body = r##"
      <fo:block>
        See also:
        <fo:basic-link external-destination="https://www.w3.org">W3C</fo:basic-link>,
        <fo:basic-link external-destination="https://www.ietf.org">IETF</fo:basic-link>,
        and <fo:basic-link internal-destination="appendix">Appendix</fo:basic-link>.
      </fo:block>
      <fo:block id="appendix" space-before="24pt" font-weight="bold">Appendix</fo:block>
      <fo:block>Appendix content.</fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}
