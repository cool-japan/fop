//! XSL-FO 1.1 Conformance: Writing modes, RTL/LTR/vertical, advanced list scenarios
//!
//! Part of the XSL-FO 1.1 conformance test suite.
//! Reference: https://www.w3.org/TR/xsl11/

use super::process_fo_document;

// ---------------------------------------------------------------------------
// Section 7.27: Writing mode and direction (RTL/LTR/vertical)
// ---------------------------------------------------------------------------

#[test]
fn conformance_rtl_writing_mode() {
    // fo:block with writing-mode="rl-tb" for Arabic/Hebrew (Section 7.27.7)
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
      <fo:block writing-mode="lr-tb">Left to right (default): Hello World</fo:block>
      <fo:block writing-mode="rl-tb" xml:lang="ar">&#x645;&#x631;&#x62D;&#x628;&#x627; &#x628;&#x627;&#x644;&#x639;&#x627;&#x644;&#x645;</fo:block>
      <fo:block writing-mode="rl-tb" xml:lang="he">&#x5E9;&#x5DC;&#x5D5;&#x5DD; &#x5E2;&#x5D5;&#x5DC;&#x5DD;</fo:block>
      <fo:block writing-mode="lr-tb">Back to LTR text.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "RTL writing-mode should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_direction_rtl() {
    // fo:block with direction="rtl" property (Section 7.27.1)
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
      <fo:block direction="ltr">English text direction left-to-right.</fo:block>
      <fo:block direction="rtl" text-align="right">RTL block direction applied here.</fo:block>
      <fo:block>
        Mixed: <fo:inline direction="rtl">inline RTL</fo:inline> back to LTR.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "direction property should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_vertical_writing_mode() {
    // fo:block with writing-mode="tb-rl" for CJK vertical text (Section 7.27.7)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="297mm" page-height="210mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block writing-mode="tb-rl" xml:lang="ja">&#x7E26;&#x66F8;&#x304D;&#x306E;&#x65E5;&#x672C;&#x8A9E;&#x30C6;&#x30AD;&#x30B9;&#x30C8;</fo:block>
      <fo:block writing-mode="tb-rl" xml:lang="zh">&#x4E2D;&#x6587;&#x5782;&#x76F4;&#x6392;&#x7248;</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Vertical writing-mode should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_bidi_mixed_content() {
    // Mixed LTR/RTL content with bidi-override (Section 7.27)
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
      <fo:block>English with embedded <fo:bidi-override direction="rtl">&#x645;&#x631;&#x62D;&#x628;&#x627;</fo:bidi-override> Arabic.</fo:block>
      <fo:block writing-mode="rl-tb" xml:lang="ar">
        &#x639;&#x631;&#x628;&#x64A; &#x645;&#x639; <fo:bidi-override direction="ltr">English</fo:bidi-override> &#x645;&#x636;&#x645;&#x646;.
      </fo:block>
      <fo:block>Normal LTR paragraph continues.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Bidi mixed content should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_external_graphic_scaling() {
    // fo:external-graphic with scaling attributes (Section 7.13.10)
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
      <fo:block>Scale to fit within box:</fo:block>
      <fo:block>
        <fo:external-graphic src="url(placeholder.png)"
          content-width="80mm" content-height="60mm"
          scaling="uniform"/>
      </fo:block>
      <fo:block>Non-uniform scaling:</fo:block>
      <fo:block>
        <fo:external-graphic src="url(placeholder.png)"
          content-width="60mm" content-height="80mm"
          scaling="non-uniform"/>
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "external-graphic scaling should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_external_graphic_explicit_dimensions() {
    // fo:external-graphic with explicit width and height (Section 7.13)
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
      <fo:block>Image with explicit dimensions:</fo:block>
      <fo:block>
        <fo:external-graphic src="url(test.jpg)"
          width="100mm" height="75mm"/>
      </fo:block>
      <fo:block>Scaled down to fit:</fo:block>
      <fo:block>
        <fo:external-graphic src="url(test.jpg)"
          content-width="scale-down-to-fit"
          content-height="scale-down-to-fit"
          width="50mm" height="50mm"/>
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "external-graphic explicit dimensions should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_external_graphic_alignment() {
    // fo:external-graphic with text-align and inline placement (Section 7.13)
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
      <fo:block text-align="center" space-after="5mm">
        <fo:external-graphic src="url(centered.png)"
          content-width="60mm" content-height="45mm"/>
      </fo:block>
      <fo:block text-align="right" space-after="5mm">
        <fo:external-graphic src="url(right.png)"
          content-width="40mm" content-height="30mm"/>
      </fo:block>
      <fo:block>Image followed by caption text.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "external-graphic alignment should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

// ---------------------------------------------------------------------------
// Section 8.4: Advanced list scenarios
// ---------------------------------------------------------------------------

#[test]
fn conformance_list_provisional_properties() {
    // fo:list-block with provisional-distance-between-starts (Section 8.4.2)
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
      <fo:list-block provisional-distance-between-starts="15mm"
        provisional-label-separation="2mm"
        space-after="6pt">
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()">
            <fo:block>1.</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>First item with custom provisional spacing</fo:block>
          </fo:list-item-body>
        </fo:list-item>
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()">
            <fo:block>2.</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>Second item in the list</fo:block>
          </fo:list-item-body>
        </fo:list-item>
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()">
            <fo:block>3.</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>Third item completes the list</fo:block>
          </fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "List with provisional properties should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_list_multiline_items() {
    // fo:list-item with multi-line body content (Section 8.4)
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
      <fo:list-block provisional-distance-between-starts="12mm">
        <fo:list-item space-after="6pt">
          <fo:list-item-label end-indent="label-end()">
            <fo:block>&#x2022;</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>This is the first item with a long description that wraps to multiple lines. The label should stay aligned at the top.</fo:block>
            <fo:block space-before="4pt">A second paragraph in the same list item.</fo:block>
          </fo:list-item-body>
        </fo:list-item>
        <fo:list-item space-after="6pt">
          <fo:list-item-label end-indent="label-end()">
            <fo:block>&#x2022;</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>Second item is shorter.</fo:block>
          </fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Multi-line list items should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_definition_list_pattern() {
    // Definition list pattern using fo:list-block (Section 8.4)
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
      <fo:block font-size="14pt" font-weight="bold" space-after="8pt">Glossary</fo:block>
      <fo:list-block provisional-distance-between-starts="40mm"
        provisional-label-separation="5mm">
        <fo:list-item space-after="8pt">
          <fo:list-item-label end-indent="label-end()">
            <fo:block font-weight="bold">XSL-FO</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>Extensible Stylesheet Language Formatting Objects. A markup language for XML document formatting.</fo:block>
          </fo:list-item-body>
        </fo:list-item>
        <fo:list-item space-after="8pt">
          <fo:list-item-label end-indent="label-end()">
            <fo:block font-weight="bold">PDF</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>Portable Document Format. A file format for capturing and sending electronic documents.</fo:block>
          </fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Definition list pattern should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_mixed_ordered_unordered_lists() {
    // Mixed ordered and unordered list styles (Section 8.4)
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
      <fo:block font-weight="bold">Steps:</fo:block>
      <fo:list-block provisional-distance-between-starts="10mm">
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()">
            <fo:block>1.</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>First step in the process</fo:block>
            <fo:block font-weight="bold" space-before="4pt">Notes:</fo:block>
            <fo:list-block provisional-distance-between-starts="8mm">
              <fo:list-item>
                <fo:list-item-label end-indent="label-end()">
                  <fo:block>&#x2013;</fo:block>
                </fo:list-item-label>
                <fo:list-item-body start-indent="body-start()">
                  <fo:block>Nested note one</fo:block>
                </fo:list-item-body>
              </fo:list-item>
              <fo:list-item>
                <fo:list-item-label end-indent="label-end()">
                  <fo:block>&#x2013;</fo:block>
                </fo:list-item-label>
                <fo:list-item-body start-indent="body-start()">
                  <fo:block>Nested note two</fo:block>
                </fo:list-item-body>
              </fo:list-item>
            </fo:list-block>
          </fo:list-item-body>
        </fo:list-item>
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()">
            <fo:block>2.</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>Second step completes the process</fo:block>
          </fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Mixed ordered/unordered lists should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_table_complex_spanning() {
    // Table with complex colspan and rowspan combinations (Section 9.3.5)
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
        <fo:table-column column-width="50mm"/>
        <fo:table-column column-width="50mm"/>
        <fo:table-column column-width="50mm"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell number-columns-spanned="2" number-rows-spanned="2"
              padding="2mm" border="1pt solid black" background-color="#e0e0e0">
              <fo:block font-weight="bold">Spans 2x2</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block>Top right</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block>Middle right</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block>Bottom left</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block>Bottom mid</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block>Bottom right</fo:block>
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
        "Complex spanning should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_table_nested_block_content() {
    // Table cells with nested blocks, lists, and inline formatting (Section 9.3)
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
        <fo:table-column column-width="60mm"/>
        <fo:table-column column-width="100mm"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell padding="3mm" border="1pt solid #999999">
              <fo:block font-weight="bold">Feature</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="3mm" border="1pt solid #999999">
              <fo:block font-weight="bold">Description</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell padding="3mm" border="1pt solid #999999" display-align="before">
              <fo:block>Nested Lists</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="3mm" border="1pt solid #999999">
              <fo:block>Cell with a list:</fo:block>
              <fo:list-block provisional-distance-between-starts="8mm">
                <fo:list-item>
                  <fo:list-item-label end-indent="label-end()"><fo:block>&#x2022;</fo:block></fo:list-item-label>
                  <fo:list-item-body start-indent="body-start()"><fo:block>Item one</fo:block></fo:list-item-body>
                </fo:list-item>
                <fo:list-item>
                  <fo:list-item-label end-indent="label-end()"><fo:block>&#x2022;</fo:block></fo:list-item-label>
                  <fo:list-item-body start-indent="body-start()"><fo:block>Item two</fo:block></fo:list-item-body>
                </fo:list-item>
              </fo:list-block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell padding="3mm" border="1pt solid #999999">
              <fo:block>Formatting</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="3mm" border="1pt solid #999999">
              <fo:block>
                <fo:inline font-weight="bold">Bold</fo:inline>,
                <fo:inline font-style="italic">italic</fo:inline>,
                <fo:inline text-decoration="underline">underline</fo:inline>
              </fo:block>
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
        "Table with nested content should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_table_text_wrap_cells() {
    // Table cells with long text that wraps to multiple lines (Section 9.3)
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
        <fo:table-column column-width="40mm"/>
        <fo:table-column column-width="120mm"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell padding="2mm" border="1pt solid black" font-weight="bold">
              <fo:block>Short</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block>This is a much longer piece of text that should wrap to multiple lines within the narrow column, demonstrating proper text flow within table cells.</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell padding="2mm" border="1pt solid black" font-weight="bold">
              <fo:block>Also wraps</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block>Another long sentence that demonstrates text wrapping. The table row height should be determined by the tallest cell content.</fo:block>
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
        "Table text wrap should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_table_mixed_column_widths() {
    // Table with fixed and proportional column widths mixed (Section 9.3.2)
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
        <fo:table-column column-width="30mm"/>
        <fo:table-column column-width="proportional-column-width(2)"/>
        <fo:table-column column-width="proportional-column-width(1)"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block font-weight="bold">Fixed 30mm</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block>Proportional 2x takes 2/3 of remaining</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block>Proportional 1x</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block>Row 2</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block>Wide cell content here</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block>Narrow</fo:block>
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
        "Mixed column widths should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_table_complete_structure() {
    // Complete table structure: header + body + footer with all features (Section 9.3)
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
      <fo:table table-layout="fixed" width="160mm" border-collapse="collapse">
        <fo:table-column column-width="20mm"/>
        <fo:table-column column-width="60mm"/>
        <fo:table-column column-width="40mm"/>
        <fo:table-column column-width="40mm"/>
        <fo:table-header>
          <fo:table-row background-color="#336699">
            <fo:table-cell padding="2mm" border="1pt solid white">
              <fo:block color="white" font-weight="bold">#</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid white">
              <fo:block color="white" font-weight="bold">Product</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid white">
              <fo:block color="white" font-weight="bold">Qty</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid white">
              <fo:block color="white" font-weight="bold">Price</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-header>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell padding="2mm" border="1pt solid #cccccc"><fo:block>1</fo:block></fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #cccccc"><fo:block>Widget A</fo:block></fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #cccccc"><fo:block>10</fo:block></fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #cccccc"><fo:block>$5.00</fo:block></fo:table-cell>
          </fo:table-row>
          <fo:table-row background-color="#f5f5f5">
            <fo:table-cell padding="2mm" border="1pt solid #cccccc"><fo:block>2</fo:block></fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #cccccc"><fo:block>Gadget B</fo:block></fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #cccccc"><fo:block>5</fo:block></fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #cccccc"><fo:block>$12.50</fo:block></fo:table-cell>
          </fo:table-row>
        </fo:table-body>
        <fo:table-footer>
          <fo:table-row background-color="#e8e8e8">
            <fo:table-cell number-columns-spanned="2" padding="2mm" border="1pt solid #cccccc">
              <fo:block font-weight="bold">Total</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #cccccc"><fo:block>15</fo:block></fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #cccccc"><fo:block>$112.50</fo:block></fo:table-cell>
          </fo:table-row>
        </fo:table-footer>
      </fo:table>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Complete table structure should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_letter_word_spacing() {
    // Letter-spacing and word-spacing properties (Section 7.15.2)
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
      <fo:block letter-spacing="0pt" space-after="6pt">Normal letter spacing: Hello World</fo:block>
      <fo:block letter-spacing="2pt" space-after="6pt">Wide letter spacing: Hello World</fo:block>
      <fo:block letter-spacing="-1pt" space-after="6pt">Narrow letter spacing: Hello World</fo:block>
      <fo:block word-spacing="10pt" space-after="6pt">Wide word spacing: Hello World</fo:block>
      <fo:block word-spacing="-2pt">Narrow word spacing: Hello World</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Letter and word spacing should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_text_rendering_modes() {
    // Various text rendering scenarios for quality (Section 7.15)
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
      <fo:block font-size="8pt" space-after="4pt">8pt: The quick brown fox jumps over the lazy dog.</fo:block>
      <fo:block font-size="10pt" space-after="4pt">10pt: The quick brown fox jumps over the lazy dog.</fo:block>
      <fo:block font-size="12pt" space-after="4pt">12pt: The quick brown fox jumps over the lazy dog.</fo:block>
      <fo:block font-size="14pt" space-after="4pt">14pt: The quick brown fox jumps.</fo:block>
      <fo:block font-size="18pt" space-after="4pt">18pt: Quick brown fox.</fo:block>
      <fo:block font-size="24pt">24pt: Big text!</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Text rendering modes should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_inline_text_mixed_sizes() {
    // Mixed font sizes in inline content on same line (Section 7.15)
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
      <fo:block line-height="20pt" space-after="8pt">
        Normal text with
        <fo:inline font-size="8pt"> small 8pt</fo:inline>
        and
        <fo:inline font-size="16pt"> large 16pt</fo:inline>
        and back to normal size.
      </fo:block>
      <fo:block>
        Chemical formula: H
        <fo:inline baseline-shift="sub" font-size="8pt">2</fo:inline>
        O + CO
        <fo:inline baseline-shift="sub" font-size="8pt">2</fo:inline>
      </fo:block>
      <fo:block space-before="6pt">
        Math: x
        <fo:inline baseline-shift="super" font-size="8pt">2</fo:inline>
        + y
        <fo:inline baseline-shift="super" font-size="8pt">2</fo:inline>
        = z
        <fo:inline baseline-shift="super" font-size="8pt">2</fo:inline>
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Mixed inline text sizes should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_multiple_footnotes() {
    // Multiple fo:footnote elements on the same page (Section 8.5)
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
      <fo:block>First footnote
        <fo:footnote>
          <fo:inline baseline-shift="super" font-size="8pt">1</fo:inline>
          <fo:footnote-body>
            <fo:block font-size="9pt">1. First footnote text.</fo:block>
          </fo:footnote-body>
        </fo:footnote>
        and second footnote
        <fo:footnote>
          <fo:inline baseline-shift="super" font-size="8pt">2</fo:inline>
          <fo:footnote-body>
            <fo:block font-size="9pt">2. Second footnote text.</fo:block>
          </fo:footnote-body>
        </fo:footnote>
        in the same paragraph.
      </fo:block>
      <fo:block space-before="6pt">Another paragraph with a third footnote
        <fo:footnote>
          <fo:inline baseline-shift="super" font-size="8pt">3</fo:inline>
          <fo:footnote-body>
            <fo:block font-size="9pt">3. Third footnote in a different block.</fo:block>
          </fo:footnote-body>
        </fo:footnote>
        reference.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Multiple footnotes should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_footnote_in_table() {
    // fo:footnote inside a table cell (Section 8.5)
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
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block>Cell with footnote
                <fo:footnote>
                  <fo:inline baseline-shift="super" font-size="8pt">*</fo:inline>
                  <fo:footnote-body>
                    <fo:block font-size="9pt">* Table cell footnote.</fo:block>
                  </fo:footnote-body>
                </fo:footnote>
              </fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block>Regular cell content</fo:block>
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
        "Footnote in table cell should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_keep_with_next_heading() {
    // fo:block keep-with-next to prevent orphaned headings (Section 7.20.3)
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
      <fo:block font-size="16pt" font-weight="bold" keep-with-next="always" space-after="6pt">
        Section 1: Introduction
      </fo:block>
      <fo:block>This paragraph follows the section heading. The heading should never appear alone at the bottom of a page without at least one following paragraph.</fo:block>
      <fo:block space-before="8pt">Another paragraph in section 1 continues here.</fo:block>
      <fo:block font-size="16pt" font-weight="bold" keep-with-next="always" space-before="12pt" space-after="6pt">
        Section 2: Details
      </fo:block>
      <fo:block>Section 2 content follows immediately after the heading.</fo:block>
      <fo:block space-before="6pt">More section 2 content here.</fo:block>
      <fo:block font-size="14pt" font-weight="bold" keep-with-next="always" space-before="8pt" space-after="4pt">
        Subsection 2.1
      </fo:block>
      <fo:block>Subsection content.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "keep-with-next heading should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_break_before_odd_even() {
    // break-before="page" for explicit page breaks (Section 7.20.2)
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
      <fo:block>Chapter 1 content on page 1.</fo:block>
      <fo:block break-before="page">This starts on a new page.</fo:block>
      <fo:block space-before="6pt">Continued content on page 2.</fo:block>
      <fo:block break-before="page">Chapter 3 starts on page 3.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "break-before page should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_keep_together_table() {
    // fo:table keep-together to prevent breaking mid-table (Section 7.20.1)
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
      <fo:block>Some introductory content before the table.</fo:block>
      <fo:table table-layout="fixed" width="160mm" keep-together="always" space-before="8pt">
        <fo:table-column column-width="80mm"/>
        <fo:table-column column-width="80mm"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block font-weight="bold">Column A</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block font-weight="bold">Column B</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block>Row 1 data</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block>Row 1 value</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block>Row 2 data</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block>Row 2 value</fo:block>
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
        "keep-together table should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_widow_orphan_paragraphs() {
    // Widow/orphan control for multi-paragraph content (Section 7.19.6)
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
      <fo:block widows="2" orphans="2">
        The first paragraph has widow and orphan control set to 2. This means at least 2 lines must appear at the top of a new page and at least 2 lines must remain at the bottom of the previous page. This prevents isolated single lines.
      </fo:block>
      <fo:block space-before="6pt" widows="3" orphans="3">
        This paragraph sets both widows and orphans to 3. That means at least 3 lines must stay together at the bottom and at least 3 lines must appear together at the top of a page. This is a more conservative setting for important text.
      </fo:block>
      <fo:block space-before="6pt">
        This paragraph uses default widow/orphan values and contains enough text to test normal text flow across page boundaries in the document layout.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Widow/orphan paragraphs should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_complex_multisequence_document() {
    // Complex document with multiple page sequences (Section 7.8)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="cover"
      page-width="210mm" page-height="297mm"
      margin-top="50mm" margin-bottom="50mm"
      margin-left="30mm" margin-right="30mm">
      <fo:region-body/>
    </fo:simple-page-master>
    <fo:simple-page-master master-name="toc"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body margin-top="12mm"/>
      <fo:region-before extent="12mm"/>
    </fo:simple-page-master>
    <fo:simple-page-master master-name="body"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body margin-top="12mm" margin-bottom="12mm"/>
      <fo:region-before extent="12mm"/>
      <fo:region-after extent="12mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="cover">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="24pt" font-weight="bold" text-align="center" space-before="30mm">
        Document Title
      </fo:block>
      <fo:block font-size="14pt" text-align="center" space-before="10mm">
        Author Name
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="toc" format="i" initial-page-number="1">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block text-align="center" font-size="9pt">Table of Contents</fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="14pt" font-weight="bold" space-after="8pt">Contents</fo:block>
      <fo:block>Chapter 1 .............. 1</fo:block>
      <fo:block>Chapter 2 .............. 5</fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="body" format="1" initial-page-number="1">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block text-align="right" font-size="9pt">Chapter 1</fo:block>
    </fo:static-content>
    <fo:static-content flow-name="xsl-region-after">
      <fo:block text-align="center" font-size="9pt">Page <fo:page-number/></fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="16pt" font-weight="bold" id="ch1">Chapter 1: Introduction</fo:block>
      <fo:block space-before="6pt">First chapter content begins here with body text that flows across pages.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Complex multi-sequence document should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}
