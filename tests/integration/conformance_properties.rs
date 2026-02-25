//! XSL-FO 1.1 Conformance: Property inheritance, shorthands, special formatting objects
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
// Property inheritance tests (Section 5.1)
// ---------------------------------------------------------------------------

#[test]
fn conformance_property_inheritance() {
    // Properties inherit from parent to child (Section 5.1)
    let body = r##"
      <fo:block font-size="14pt" color="navy" font-family="serif">
        Outer block sets font-size and color.
        <fo:inline>Inline inherits these properties.</fo:inline>
        <fo:inline font-size="10pt">But this overrides font-size.</fo:inline>
      </fo:block>
      <fo:block color="gray">
        Gray text — inherits page margins but sets its own color.
        <fo:block>
          Nested block also inherits gray color.
          <fo:block color="black">Explicit black overrides.</fo:block>
        </fo:block>
      </fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_property_shorthand_expansion() {
    // Shorthands like margin, padding, border, font expand to individuals
    let body = r##"
      <fo:block margin="10pt" padding="5pt" border="1pt solid black">
        Shorthand properties (margin/padding/border)
      </fo:block>
      <fo:block font="bold 14pt/1.5 serif">
        Font shorthand: weight size/line-height family
      </fo:block>
      <fo:block background="#FFFF99" padding="3pt">
        Background color via shorthand
      </fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

// ---------------------------------------------------------------------------
// Unicode and internationalization (XSL-FO 1.1 Section 7.9.1, 7.17)
// ---------------------------------------------------------------------------

#[test]
fn conformance_unicode_text() {
    // XSL-FO must handle Unicode text (Section 7.17)
    let body = r##"
      <fo:block>English: The quick brown fox jumps over the lazy dog.</fo:block>
      <fo:block>Japanese: &#x65E5;&#x672C;&#x8A9E;&#x30C6;&#x30B9;&#x30C8;</fo:block>
      <fo:block>Chinese: &#x4E2D;&#x6587;&#x6D4B;&#x8BD5;</fo:block>
      <fo:block>Korean: &#xD55C;&#xAD6D;&#xC5B4; &#xD14C;&#xC2A4;&#xD2B8;</fo:block>
      <fo:block>Arabic: &#x0645;&#x0631;&#x062D;&#x0628;&#x0627;</fo:block>
      <fo:block>Greek: &#x03B1;&#x03B2;&#x03B3;&#x03B4;&#x03B5;</fo:block>
      <fo:block>Special: &#x00A9; &#x00AE; &#x2122; &#x2014; &#x2013;</fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_writing_mode() {
    // writing-mode property (Section 7.29.7) — LTR (default)
    let body = r##"
      <fo:block writing-mode="lr-tb">
        Left-to-right, top-to-bottom text direction.
      </fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

// ---------------------------------------------------------------------------
// Page layout features
// ---------------------------------------------------------------------------

#[test]
fn conformance_page_number_initial_value() {
    // initial-page-number on fo:page-sequence (Section 7.27.7)
    let fo = r##"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
        page-width="210mm" page-height="297mm" margin="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4" initial-page-number="5">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>This document starts at page 5. Current page: <fo:page-number/></fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;
    let pdf = check_fo(fo);
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_force_page_count() {
    // force-page-count on fo:page-sequence (Section 7.27.1)
    let fo = r##"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
        page-width="210mm" page-height="297mm" margin="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4" force-page-count="auto">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Content with auto page count control</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;
    let pdf = check_fo(fo);
    validate_pdf_bytes(&pdf);
}

// ---------------------------------------------------------------------------
// Block container (Section 7.3)
// ---------------------------------------------------------------------------

#[test]
fn conformance_block_container() {
    // fo:block-container for fixed-position or independent block areas
    let body = r##"
      <fo:block-container width="80mm" height="30mm"
          border="1pt solid black" padding="5pt">
        <fo:block>Content inside a block-container</fo:block>
        <fo:block>Second block inside container</fo:block>
      </fo:block-container>
      <fo:block>Normal block after the container</fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

// ---------------------------------------------------------------------------
// Widow and orphan control (Section 7.20.4, 7.20.5)
// ---------------------------------------------------------------------------

#[test]
fn conformance_widow_orphan_control() {
    // widows and orphans properties
    let body = r##"
      <fo:block widows="2" orphans="2">
        This paragraph has widow and orphan control set to 2 lines each.
        When this content is laid out, at least 2 lines must appear together
        at the top and bottom of pages. This prevents widows and orphans
        in the output document.
      </fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

// ---------------------------------------------------------------------------
// Color model (Section 7.18)
// ---------------------------------------------------------------------------

#[test]
fn conformance_color_values() {
    // Various color notations per CSS2 color model
    let body = r##"
      <fo:block color="#FF0000">Hex color #FF0000</fo:block>
      <fo:block color="#F00">Short hex #F00</fo:block>
      <fo:block color="rgb(0, 128, 0)">RGB function</fo:block>
      <fo:block color="blue">Named color</fo:block>
      <fo:block color="navy">Named color navy</fo:block>
      <fo:block color="transparent">Transparent (default inherited color shown)</fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

// ---------------------------------------------------------------------------
// Mixed content (inline + block)
// ---------------------------------------------------------------------------

#[test]
fn conformance_mixed_inline_block_content() {
    // Realistic document with mixed formatting
    let body = r##"
      <fo:block font-size="18pt" font-weight="bold" space-after="12pt">
        Document Title
      </fo:block>
      <fo:block font-size="14pt" font-style="italic" space-after="8pt" color="#555555">
        Subtitle or Abstract
      </fo:block>
      <fo:block space-after="6pt">
        This is a paragraph with <fo:inline font-weight="bold">bold</fo:inline>,
        <fo:inline font-style="italic">italic</fo:inline>, and
        <fo:inline color="navy" text-decoration="underline">underlined navy</fo:inline>
        inline text mixed in.
      </fo:block>
      <fo:block space-after="6pt">
        A second paragraph demonstrating that the layout engine correctly handles
        multiple paragraphs with space-after properties.
      </fo:block>
      <fo:block font-size="12pt" font-weight="bold" space-before="12pt" space-after="6pt">
        Section 1
      </fo:block>
      <fo:block>
        Section content. This tests that section headers keep with their following content.
      </fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

// ---------------------------------------------------------------------------
// Numeric formatting (percentages, em units)
// ---------------------------------------------------------------------------

#[test]
fn conformance_length_units() {
    // Various length units: pt, mm, cm, in, em, px, % (Section 5.11.1)
    let body = r##"
      <fo:block font-size="12pt" margin-left="10mm">Margin in mm</fo:block>
      <fo:block font-size="12pt" margin-left="1cm">Margin in cm</fo:block>
      <fo:block font-size="12pt" margin-left="0.5in">Margin in inches</fo:block>
      <fo:block font-size="12pt" margin-left="2em">Margin in em (2 * 12pt)</fo:block>
      <fo:block font-size="12pt" margin-left="20px">Margin in px</fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

// ---------------------------------------------------------------------------
// Page break control (Section 7.20.1)
// ---------------------------------------------------------------------------

#[test]
fn conformance_page_break_before_after() {
    // break-before and break-after
    let body = r##"
      <fo:block>First section content</fo:block>
      <fo:block break-before="page" font-weight="bold">
        New page heading (break-before=page)
      </fo:block>
      <fo:block>Content after the forced page break.</fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

// ---------------------------------------------------------------------------
// fo:block-container with absolute positioning (Section 7.3)
// ---------------------------------------------------------------------------

#[test]
fn conformance_nested_tables() {
    // Tables within tables (not explicitly in spec but commonly needed)
    let body = r##"
      <fo:table width="100%" table-layout="fixed">
        <fo:table-column column-width="50%"/>
        <fo:table-column column-width="50%"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell border="0.5pt solid black" padding="4pt">
              <fo:block font-weight="bold">Outer Cell 1</fo:block>
              <fo:table width="100%" table-layout="fixed">
                <fo:table-column column-width="50%"/>
                <fo:table-column column-width="50%"/>
                <fo:table-body>
                  <fo:table-row>
                    <fo:table-cell border="0.5pt solid gray" padding="2pt">
                      <fo:block font-size="8pt">Inner A</fo:block>
                    </fo:table-cell>
                    <fo:table-cell border="0.5pt solid gray" padding="2pt">
                      <fo:block font-size="8pt">Inner B</fo:block>
                    </fo:table-cell>
                  </fo:table-row>
                </fo:table-body>
              </fo:table>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="4pt">
              <fo:block>Outer Cell 2</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

// ---------------------------------------------------------------------------
// Absolute positioning (block-container with absolute-position)
// ---------------------------------------------------------------------------

#[test]
fn conformance_block_container_absolute() {
    // fo:block-container with absolute-position="absolute" uses top/left
    let body = r##"
      <fo:block>Normal block before absolute container</fo:block>
      <fo:block-container absolute-position="absolute" top="50mm" left="30mm"
          width="100mm" height="40mm" border="1pt solid navy" padding="5pt">
        <fo:block font-size="9pt" color="navy">This text is absolutely positioned</fo:block>
      </fo:block-container>
      <fo:block>Normal block after (absolute container does not advance flow)</fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

// ---------------------------------------------------------------------------
// Custom list markers from fo:list-item-label
// ---------------------------------------------------------------------------

#[test]
fn conformance_list_custom_markers() {
    // fo:list-item-label provides custom marker text
    let body = r##"
      <fo:list-block provisional-distance-between-starts="30mm"
                     provisional-label-separation="5mm">
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()">
            <fo:block>Step 1.</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>First item with a custom label</fo:block>
          </fo:list-item-body>
        </fo:list-item>
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()">
            <fo:block>Step 2.</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>Second item with a custom label and more content</fo:block>
            <fo:block>Additional block in the second item</fo:block>
          </fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

// ---------------------------------------------------------------------------
// Relative font-size keywords
// ---------------------------------------------------------------------------

#[test]
fn conformance_relative_font_size() {
    // font-size keyword values per XSL-FO spec Section 7.9.4
    let body = r##"
      <fo:block font-size="xx-small">xx-small text (9pt)</fo:block>
      <fo:block font-size="x-small">x-small text (10pt)</fo:block>
      <fo:block font-size="small">small text (13pt)</fo:block>
      <fo:block font-size="medium">medium text (16pt)</fo:block>
      <fo:block font-size="large">large text (18pt)</fo:block>
      <fo:block font-size="x-large">x-large text (24pt)</fo:block>
      <fo:block font-size="xx-large">xx-large text (32pt)</fo:block>
      <fo:block font-size="12pt">
        Normal 12pt block with
        <fo:inline font-size="larger">larger inline text</fo:inline>
        and
        <fo:inline font-size="smaller">smaller inline text</fo:inline>
        mixed in.
      </fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

// ---------------------------------------------------------------------------
// Area tree serialization
// ---------------------------------------------------------------------------

#[test]
fn conformance_area_tree_serialization() {
    // Verify area tree serialization produces useful output
    let fo = wrap_fo(r##"<fo:block font-size="12pt">Serialization test</fo:block>"##);
    let arena = fop_core::FoTreeBuilder::new()
        .parse(std::io::Cursor::new(fo.as_bytes()))
        .expect("test: should succeed");
    let engine = fop_layout::LayoutEngine::new();
    let area_tree = engine.layout(&arena).expect("test: should succeed");
    let serialized = area_tree.serialize();
    assert!(
        serialized.contains("Page"),
        "serialize() should contain 'Page'"
    );
    assert!(!serialized.is_empty(), "serialize() should not be empty");
}

// ---------------------------------------------------------------------------
// calc() expression support (Section 7 of XSL-FO spec)
// ---------------------------------------------------------------------------

#[test]
fn conformance_calc_expressions() {
    // CSS calc() in property values (Section 7 of XSL-FO spec)
    let body = r##"
      <fo:block margin-left="calc(10mm + 5pt)" font-size="12pt">
        Block with calc() margin
      </fo:block>
      <fo:block padding-top="calc(5% + 2pt)" font-size="12pt">
        Block with calc() padding
      </fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

// ---------------------------------------------------------------------------
// Area tree diffing for regression testing
// ---------------------------------------------------------------------------

#[test]
fn conformance_area_tree_diff() {
    // Verify area tree diffing detects changes correctly
    let fo1 = wrap_fo(r##"<fo:block font-size="12pt">Hello World</fo:block>"##);
    let fo2 = wrap_fo(r##"<fo:block font-size="12pt">Hello World</fo:block>"##);

    let arena1 = fop_core::FoTreeBuilder::new()
        .parse(std::io::Cursor::new(fo1.as_bytes()))
        .expect("test: should succeed");
    let arena2 = fop_core::FoTreeBuilder::new()
        .parse(std::io::Cursor::new(fo2.as_bytes()))
        .expect("test: should succeed");

    let engine = fop_layout::LayoutEngine::new();
    let tree1 = engine.layout(&arena1).expect("test: should succeed");
    let tree2 = engine.layout(&arena2).expect("test: should succeed");

    // Same document should produce no diffs
    let diffs = tree1.diff(&tree2);
    assert!(
        diffs.is_empty(),
        "Identical documents should produce no diffs: {:?}",
        diffs
    );
}

// ---------------------------------------------------------------------------
// fo:declarations support
// ---------------------------------------------------------------------------

#[test]
fn conformance_declarations_support() {
    // fo:declarations with fo:color-profile should be parsed without errors
    let fo_input = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:declarations>
    <fo:color-profile src="cmyk.icc" color-profile-name="cmyk"/>
  </fo:declarations>
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
      <fo:block>Color profile test</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

    let arena = fop_core::FoTreeBuilder::new()
        .parse(std::io::Cursor::new(fo_input.as_bytes()))
        .expect("fo:declarations should be parsed without error");
    let engine = fop_layout::LayoutEngine::new();
    let area_tree = engine
        .layout(&arena)
        .expect("layout should succeed with fo:declarations");
    assert!(!area_tree.is_empty(), "Area tree should not be empty");
}

// ---------------------------------------------------------------------------
// Known-but-unsupported element passthrough
// ---------------------------------------------------------------------------

#[test]
fn conformance_unsupported_element_passthrough() {
    // fo:wrapper is a known-but-unsupported element: should not return an error
    let fo_input = r#"<?xml version="1.0" encoding="UTF-8"?>
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
      <fo:wrapper color="red">
        <fo:block>Wrapped content</fo:block>
      </fo:wrapper>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

    let result = fop_core::FoTreeBuilder::new().parse(std::io::Cursor::new(fo_input.as_bytes()));
    assert!(
        result.is_ok(),
        "fo:wrapper should be parsed without error: {:?}",
        result.err()
    );
    let arena = result.expect("test: should succeed");
    let engine = fop_layout::LayoutEngine::new();
    let layout_result = engine.layout(&arena);
    assert!(
        layout_result.is_ok(),
        "fo:wrapper should be handled gracefully: {:?}",
        layout_result.err()
    );
}

// ---------------------------------------------------------------------------
// fo:float (Section 10.3)
// ---------------------------------------------------------------------------

#[test]
fn conformance_float_basic() {
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
        <fo:float float="start">
          <fo:block font-size="10pt" width="40mm">Float content here.</fo:block>
        </fo:float>
        Main text that flows around the float. Lorem ipsum dolor sit amet.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "fo:float should be supported: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty());
}

// ---------------------------------------------------------------------------
// fo:footnote (Section 10.4)
// ---------------------------------------------------------------------------

#[test]
fn conformance_footnote_basic() {
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
        Main text with footnote reference
        <fo:footnote>
          <fo:inline baseline-shift="super" font-size="7pt">1</fo:inline>
          <fo:footnote-body>
            <fo:block font-size="9pt">1. This is the footnote text.</fo:block>
          </fo:footnote-body>
        </fo:footnote>
        continuing after the footnote.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "fo:footnote should be supported: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty());
}

// ---------------------------------------------------------------------------
// fo:marker and fo:retrieve-marker (Section 9.8)
// ---------------------------------------------------------------------------

#[test]
fn conformance_marker_retrieve() {
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="15mm" margin-bottom="15mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body margin-top="15mm"/>
      <fo:region-before extent="15mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block font-size="10pt">
        Chapter: <fo:retrieve-marker retrieve-class-name="chapter-title"/>
      </fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block>
        <fo:marker marker-class-name="chapter-title">Introduction</fo:marker>
        Chapter 1 content.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "fo:marker + fo:retrieve-marker should be supported: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty());
}

// ---------------------------------------------------------------------------
// Multi-column layout (Section 7.24)
// ---------------------------------------------------------------------------

#[test]
fn conformance_multicolumn_layout() {
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body column-count="2" column-gap="10mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="10pt">First column text.</fo:block>
      <fo:block font-size="10pt">Second column continues here with more text to fill the second column.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Multi-column layout should be supported: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty());
}

// ---------------------------------------------------------------------------
// Conditional page masters (Section 7.27)
// ---------------------------------------------------------------------------

#[test]
fn conformance_conditional_page_masters() {
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="first-page"
      page-width="210mm" page-height="297mm"
      margin-top="30mm" margin-bottom="20mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
    <fo:simple-page-master master-name="rest-pages"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
    <fo:page-sequence-master master-name="doc">
      <fo:repeatable-page-master-alternatives>
        <fo:conditional-page-master-reference
          master-reference="first-page"
          page-position="first"/>
        <fo:conditional-page-master-reference
          master-reference="rest-pages"
          page-position="rest"/>
      </fo:repeatable-page-master-alternatives>
    </fo:page-sequence-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="doc">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>First page uses wider top margin.</fo:block>
      <fo:block break-before="page">Second page uses standard margins.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Conditional page masters should be supported: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty());
}

// ---------------------------------------------------------------------------
// fo:external-graphic (Section 8.6)
// ---------------------------------------------------------------------------

#[test]
fn conformance_external_graphic_missing_src() {
    // Verify that a missing graphic src is handled gracefully (not a panic)
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
      <fo:block>Image below:</fo:block>
      <fo:block>
        <fo:external-graphic src="nonexistent.png" content-width="50mm" content-height="30mm"/>
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    // Should not panic; may succeed (with placeholder) or return an error
    // The important thing is graceful handling
    if let Ok(bytes) = result {
        assert!(!bytes.is_empty(), "PDF should not be empty on success");
    }
    // Graceful error is acceptable
}

// ---------------------------------------------------------------------------
// PDF Bookmarks (fo:bookmark-tree)
// ---------------------------------------------------------------------------

#[test]
fn conformance_bookmark_tree() {
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
  <fo:bookmark-tree>
    <fo:bookmark internal-destination="intro">
      <fo:bookmark-title>Introduction</fo:bookmark-title>
    </fo:bookmark>
    <fo:bookmark internal-destination="chapter1">
      <fo:bookmark-title>Chapter 1</fo:bookmark-title>
      <fo:bookmark internal-destination="section1-1">
        <fo:bookmark-title>Section 1.1</fo:bookmark-title>
      </fo:bookmark>
    </fo:bookmark>
  </fo:bookmark-tree>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block id="intro" font-size="14pt" font-weight="bold">Introduction</fo:block>
      <fo:block>Introductory text.</fo:block>
      <fo:block id="chapter1" font-size="14pt" font-weight="bold" break-before="page">Chapter 1</fo:block>
      <fo:block id="section1-1" font-size="12pt" font-weight="bold">Section 1.1</fo:block>
      <fo:block>Section content.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "fo:bookmark-tree should be supported: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty());
    // Check that the PDF contains /Outlines (bookmark structure)
    let pdf_str = String::from_utf8_lossy(&bytes);
    assert!(
        pdf_str.contains("/Outlines") || pdf_str.contains("/Outline"),
        "PDF should contain bookmark outlines"
    );
}

// ---------------------------------------------------------------------------
// Text decoration properties (Section 7.17.4)
// ---------------------------------------------------------------------------

#[test]
fn conformance_text_decoration() {
    // text-decoration: underline, overline, line-through (Section 7.17.4)
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
      <fo:block text-decoration="underline">Underlined text</fo:block>
      <fo:block text-decoration="overline">Overlined text</fo:block>
      <fo:block text-decoration="line-through">Strikethrough text</fo:block>
      <fo:block>
        <fo:inline text-decoration="underline">inline underline</fo:inline>
        normal
        <fo:inline text-decoration="line-through">inline strike</fo:inline>
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "text-decoration should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

// ---------------------------------------------------------------------------
// baseline-shift: superscript/subscript (Section 7.14.3)
// ---------------------------------------------------------------------------

#[test]
fn conformance_baseline_shift() {
    // baseline-shift: super, sub, <length> (Section 7.14.3)
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
        H<fo:inline baseline-shift="sub" font-size="0.7em">2</fo:inline>O (water)
      </fo:block>
      <fo:block>
        E = mc<fo:inline baseline-shift="super" font-size="0.7em">2</fo:inline>
      </fo:block>
      <fo:block>
        Footnote<fo:inline baseline-shift="super" font-size="7pt">1</fo:inline>
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "baseline-shift should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

// ---------------------------------------------------------------------------
// Compound space properties (Section 5.11)
// ---------------------------------------------------------------------------

#[test]
fn conformance_compound_space_properties() {
    // Compound space properties with minimum/optimum/maximum (Section 5.11)
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
      <fo:block space-before.minimum="5pt" space-before.optimum="10pt" space-before.maximum="20pt">
        Block with compound space-before
      </fo:block>
      <fo:block space-after.minimum="5pt" space-after.optimum="10pt" space-after.maximum="20pt">
        Block with compound space-after
      </fo:block>
      <fo:block space-before="12pt" space-after="8pt">
        Block with simple spacing
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Compound space properties should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

// ---------------------------------------------------------------------------
// keep-together compound properties (Section 7.19.3)
// ---------------------------------------------------------------------------

#[test]
fn conformance_keep_together_compound() {
    // keep-together.within-page / within-column / within-line (Section 7.19.3)
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
      <fo:block keep-together.within-page="always">
        This block should be kept together within a page.
        All lines should appear on the same page.
      </fo:block>
      <fo:block keep-together.within-column="always">
        This block should be kept within a single column.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "keep-together compound properties should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

// ---------------------------------------------------------------------------
// opacity and transparency (Section 7.21)
// ---------------------------------------------------------------------------

#[test]
fn conformance_opacity_transparency() {
    // opacity property (Section 7.21)
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
      <fo:block opacity="1.0" background-color="blue" color="white">Fully opaque</fo:block>
      <fo:block opacity="0.5" background-color="red" color="white">50% transparent</fo:block>
      <fo:block opacity="0.25" background-color="green" color="white">75% transparent</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "opacity should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

// ---------------------------------------------------------------------------
// border-radius (rounded corners)
// ---------------------------------------------------------------------------

#[test]
fn conformance_border_radius() {
    // border-radius for rounded corners on blocks (CSS extension in XSL-FO processors)
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
      <fo:block border="2pt solid black" border-radius="5pt" padding="5pt">
        Block with rounded corners
      </fo:block>
      <fo:block border="1pt solid blue" border-radius="10pt" padding="8pt" margin-top="10pt">
        Another rounded block
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "border-radius should be supported: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}
