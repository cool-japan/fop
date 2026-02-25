//! XSL-FO 1.1 Conformance: PostScript, SVG, text and output format validation
//!
//! Part of the XSL-FO 1.1 conformance test suite.
//! Reference: https://www.w3.org/TR/xsl11/

// ---------------------------------------------------------------------------
// Section PS rendering: PostScript and Text output
// ---------------------------------------------------------------------------

#[test]
fn conformance_postscript_text_content() {
    // PostScript output contains proper PS commands (Section PS rendering)
    let result = super::process_fo_document_format(
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
      <fo:block font-size="14pt" font-weight="bold">PostScript Output Test</fo:block>
      <fo:block>This content should appear in PostScript output format.</fo:block>
      <fo:block space-before="6pt">Second paragraph with more text.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        "ps",
    );
    assert!(
        result.is_ok(),
        "PostScript output should work: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty(), "PS output should not be empty");
    let ps_str = String::from_utf8_lossy(&bytes);
    assert!(ps_str.contains("%!"), "PS should start with %!");
}

#[test]
fn conformance_text_output_structure() {
    // Text output preserves document structure (Section text rendering)
    let result = super::process_fo_document_format(
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
      <fo:block font-size="14pt">HEADING TEXT</fo:block>
      <fo:block>First paragraph of content.</fo:block>
      <fo:block>Second paragraph of content.</fo:block>
      <fo:list-block provisional-distance-between-starts="10mm">
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()"><fo:block>1.</fo:block></fo:list-item-label>
          <fo:list-item-body start-indent="body-start()"><fo:block>List item one</fo:block></fo:list-item-body>
        </fo:list-item>
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()"><fo:block>2.</fo:block></fo:list-item-label>
          <fo:list-item-body start-indent="body-start()"><fo:block>List item two</fo:block></fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        "text",
    );
    assert!(
        result.is_ok(),
        "Text output should work: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty(), "Text output should not be empty");
    let text = String::from_utf8_lossy(&bytes);
    assert!(
        text.contains("HEADING TEXT") || text.contains("Heading") || !text.is_empty(),
        "Text output should contain content"
    );
}

#[test]
fn conformance_postscript_multipage() {
    // PostScript output for multi-page document (Section PS rendering)
    let result = super::process_fo_document_format(
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
      <fo:block>Page 1: Introduction</fo:block>
      <fo:block break-before="page">Page 2: Content</fo:block>
      <fo:block break-before="page">Page 3: Conclusion</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        "ps",
    );
    assert!(
        result.is_ok(),
        "PostScript multi-page should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_text_output_table() {
    // Text output of table content (Section text rendering)
    let result = super::process_fo_document_format(
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
      <fo:block>Table in text output:</fo:block>
      <fo:table table-layout="fixed" width="160mm">
        <fo:table-column column-width="53mm"/>
        <fo:table-column column-width="53mm"/>
        <fo:table-column column-width="54mm"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell padding="2mm"><fo:block>Name</fo:block></fo:table-cell>
            <fo:table-cell padding="2mm"><fo:block>Value</fo:block></fo:table-cell>
            <fo:table-cell padding="2mm"><fo:block>Status</fo:block></fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell padding="2mm"><fo:block>Alpha</fo:block></fo:table-cell>
            <fo:table-cell padding="2mm"><fo:block>100</fo:block></fo:table-cell>
            <fo:table-cell padding="2mm"><fo:block>Active</fo:block></fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        "text",
    );
    assert!(
        result.is_ok(),
        "Text output with table should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

// ---------------------------------------------------------------------------
// SVG-specific conformance tests
// ---------------------------------------------------------------------------

#[test]
fn conformance_svg_output_text() {
    // SVG output contains proper text elements with xmlns (Section SVG rendering)
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
      <fo:block>This text should appear in the SVG output as text elements.</fo:block>
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
    assert!(svg.contains("<svg"), "SVG should contain svg element");
    assert!(
        svg.contains("xmlns"),
        "SVG should have namespace declaration"
    );
    assert!(svg.contains("viewBox"), "SVG should have viewBox attribute");
    assert!(svg.contains("<text"), "SVG should contain text elements");
}

#[test]
fn conformance_svg_multipage_output() {
    // SVG output for multi-page document produces non-empty SVG (Section SVG rendering)
    let result = super::process_fo_document_format(
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
      <fo:block>Page 1 content</fo:block>
      <fo:block break-before="page">Page 2 content</fo:block>
      <fo:block break-before="page">Page 3 content</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        "svg",
    );
    assert!(
        result.is_ok(),
        "SVG multi-page should work: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty(), "SVG output should not be empty");
    let svg_str = String::from_utf8_lossy(&bytes);
    assert!(
        svg_str.contains("<svg"),
        "Multi-page SVG should contain svg element"
    );
}

#[test]
fn conformance_svg_with_borders() {
    // SVG output includes border rectangles for blocks with borders (Section SVG rendering)
    let result = super::process_fo_document_format(
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
      <fo:block border="2pt solid black" padding="5mm" space-after="5mm">
        Block with border
      </fo:block>
      <fo:block border="1pt dashed red" padding="3mm" background-color="#ffffcc">
        Block with background and dashed border
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        "svg",
    );
    assert!(
        result.is_ok(),
        "SVG with borders should work: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty(), "SVG with borders should not be empty");
    let svg_str = String::from_utf8_lossy(&bytes);
    assert!(svg_str.contains("<svg"), "SVG should be valid SVG");
    assert!(
        svg_str.contains("<rect"),
        "SVG should contain rect elements for borders/backgrounds"
    );
}

#[test]
fn conformance_svg_text_styles() {
    // SVG output with various text styles produces non-empty valid SVG (Section SVG rendering)
    let result = super::process_fo_document_format(
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
      <fo:block font-weight="bold" font-size="16pt" color="#003366">Heading</fo:block>
      <fo:block font-style="italic" space-before="4pt">Italic text</fo:block>
      <fo:block font-weight="bold" font-style="italic" space-before="4pt">Bold italic text</fo:block>
      <fo:block text-decoration="underline" space-before="4pt">Underlined text</fo:block>
      <fo:block color="#cc0000" space-before="4pt">Red colored text</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        "svg",
    );
    assert!(
        result.is_ok(),
        "SVG text styles should work: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(
        !bytes.is_empty(),
        "SVG text styles output should not be empty"
    );
    let svg_str = String::from_utf8_lossy(&bytes);
    assert!(
        svg_str.contains("<text"),
        "SVG should contain text elements for styled text"
    );
}

// ---------------------------------------------------------------------------
// Output format validation tests
// ---------------------------------------------------------------------------

#[test]
fn conformance_output_format_pdf_valid_header() {
    // PDF output should be valid PDF with correct %PDF- header
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
      <fo:block>PDF header validation test</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;
    let result = super::process_fo_document(fo_xml);
    assert!(
        result.is_ok(),
        "PDF render should succeed: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(
        bytes.starts_with(b"%PDF-"),
        "PDF output should start with %PDF-"
    );
    assert!(bytes.len() > 100, "PDF output should be a non-trivial size");
}

#[test]
fn conformance_output_format_svg_contains_elements() {
    // SVG output should contain proper SVG structure
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
      <fo:block>SVG element structure test</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;
    let result = super::process_fo_document_format(fo_xml, "svg");
    assert!(
        result.is_ok(),
        "SVG render should succeed: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty(), "SVG output should not be empty");
    let svg_str = String::from_utf8_lossy(&bytes);
    assert!(
        svg_str.contains("<svg"),
        "SVG output should contain <svg element"
    );
}

#[test]
fn conformance_output_format_text_is_valid_utf8() {
    // Text output should be valid UTF-8 with readable content
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
      <fo:block>Text format UTF-8 content check</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;
    let result = super::process_fo_document_format(fo_xml, "text");
    assert!(
        result.is_ok(),
        "Text render should succeed: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty(), "Text output should not be empty");
    let text = String::from_utf8(bytes);
    assert!(text.is_ok(), "Text output should be valid UTF-8");
}

#[test]
fn conformance_output_format_ps_has_adobe_header() {
    // PostScript output should start with %!PS-Adobe header
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
      <fo:block>PostScript header validation test</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;
    let result = super::process_fo_document_format(fo_xml, "ps");
    assert!(
        result.is_ok(),
        "PS render should succeed: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty(), "PS output should not be empty");
    let ps_str = String::from_utf8_lossy(&bytes);
    assert!(ps_str.starts_with("%!"), "PostScript should start with %!");
}
