//! XSL-FO 1.1 Conformance: Basic document structure, block/inline/table/list formatting
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
// Section 4: Basic XSL-FO document structure
// ---------------------------------------------------------------------------

#[test]
fn conformance_basic_document_structure() {
    // Minimal valid XSL-FO document per spec Section 4
    let fo = r##"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="simple"
        page-width="8.5in" page-height="11in"
        margin="1in">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="simple">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Hello XSL-FO 1.1</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;
    let pdf = check_fo(fo);
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_multiple_page_sequences() {
    // Multiple page sequences in one document (Section 4.2.3)
    let fo = r##"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
        page-width="210mm" page-height="297mm" margin="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Chapter 1</fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Chapter 2</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;
    let pdf = check_fo(fo);
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_static_content_header_footer() {
    // fo:static-content for headers and footers (Section 4.2.2)
    let fo = r##"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
        page-width="210mm" page-height="297mm"
        margin-top="30mm" margin-bottom="30mm"
        margin-left="25mm" margin-right="25mm">
      <fo:region-body region-name="xsl-region-body"
          margin-top="15mm" margin-bottom="15mm"/>
      <fo:region-before region-name="xsl-region-before" extent="15mm"/>
      <fo:region-after region-name="xsl-region-after" extent="15mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block font-size="9pt" text-align="center">Document Header</fo:block>
    </fo:static-content>
    <fo:static-content flow-name="xsl-region-after">
      <fo:block font-size="9pt" text-align="center">Page <fo:page-number/></fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Body content</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;
    let pdf = check_fo(fo);
    validate_pdf_bytes(&pdf);
}

// ---------------------------------------------------------------------------
// Section 6: Formatting Properties — block-level
// ---------------------------------------------------------------------------

#[test]
fn conformance_block_text_alignment() {
    // text-align: start/center/end/justify (Section 7.16.9)
    let body = r##"
      <fo:block text-align="start">Left aligned text block</fo:block>
      <fo:block text-align="center">Centered text block</fo:block>
      <fo:block text-align="end">Right aligned text block</fo:block>
      <fo:block text-align="justify">Justified text block with enough words to actually justify</fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_block_spacing_and_indents() {
    // space-before, space-after, start-indent, end-indent (Section 7.12)
    let body = r##"
      <fo:block space-before="10pt" space-after="10pt">Block with space before and after</fo:block>
      <fo:block start-indent="20pt" end-indent="20pt">Indented block</fo:block>
      <fo:block text-indent="36pt">Block with text-indent (hanging paragraph style)</fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_block_font_properties() {
    // font-family, font-size, font-weight, font-style (Section 7.9)
    let body = r##"
      <fo:block font-size="8pt">Eight point text</fo:block>
      <fo:block font-size="12pt">Twelve point text</fo:block>
      <fo:block font-size="18pt">Eighteen point text</fo:block>
      <fo:block font-size="24pt">Twenty-four point text</fo:block>
      <fo:block font-weight="bold">Bold text</fo:block>
      <fo:block font-style="italic">Italic text</fo:block>
      <fo:block font-weight="bold" font-style="italic">Bold italic text</fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_block_background_color() {
    // background-color on block (Section 7.8)
    let body = r##"
      <fo:block background-color="#FF0000" color="white" padding="5pt">Red background</fo:block>
      <fo:block background-color="#00AA00" color="white" padding="5pt">Green background</fo:block>
      <fo:block background-color="#0000FF" color="white" padding="5pt">Blue background</fo:block>
      <fo:block background-color="rgb(128,128,0)" padding="5pt">Olive background</fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_block_borders() {
    // border shorthand and individual sides (Section 7.7)
    let body = r##"
      <fo:block border="1pt solid black" padding="5pt" margin-bottom="5pt">All borders</fo:block>
      <fo:block border-top="2pt solid red" padding="5pt" margin-bottom="5pt">Top border only</fo:block>
      <fo:block border-left="3pt dashed blue" padding="5pt" margin-bottom="5pt">Left dashed</fo:block>
      <fo:block border="1pt dotted green" padding="5pt">Dotted border</fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_block_padding_and_margin() {
    // padding and margin shorthands (Section 7.7, 7.12)
    let body = r##"
      <fo:block padding="10pt" margin="5pt" background-color="#EEEEEE">Padded and margined</fo:block>
      <fo:block padding-top="5pt" padding-bottom="5pt" padding-left="10pt" padding-right="10pt">
        Individually padded block
      </fo:block>
      <fo:block margin-top="10pt" margin-bottom="10pt">Top and bottom margin</fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_block_line_height() {
    // line-height property (Section 7.16.4)
    let body = r##"
      <fo:block line-height="1.0">Single-spaced text. Line height equals font size.</fo:block>
      <fo:block line-height="1.5">One-and-a-half spaced text. More readable for body copy.</fo:block>
      <fo:block line-height="2.0">Double spaced text. Classic for academic papers.</fo:block>
      <fo:block line-height="24pt">Fixed 24pt line height.</fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_block_keep_properties() {
    // keep-together, keep-with-next, keep-with-previous (Section 7.20)
    let body = r##"
      <fo:block keep-with-next.within-page="always">Heading that keeps with next</fo:block>
      <fo:block>Following paragraph that must stay with the heading above.</fo:block>
      <fo:block keep-together.within-page="always">
        This block must stay together on one page.
        It should not be split across a page break.
      </fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

// ---------------------------------------------------------------------------
// Section 8: Inline-level formatting objects
// ---------------------------------------------------------------------------

#[test]
fn conformance_inline_formatting() {
    // fo:inline for inline character formatting (Section 8.3.1)
    let body = r##"
      <fo:block>
        Normal text with <fo:inline font-weight="bold">bold</fo:inline> and
        <fo:inline font-style="italic">italic</fo:inline> and
        <fo:inline color="red">red colored</fo:inline> inline elements.
      </fo:block>
      <fo:block>
        Text with <fo:inline font-size="8pt">smaller</fo:inline> and
        <fo:inline font-size="18pt">larger</fo:inline> inline text.
      </fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_page_number() {
    // fo:page-number (Section 8.3.8)
    let fo = r##"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
        page-width="210mm" page-height="297mm" margin="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>This is page <fo:page-number/> of the document.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;
    let pdf = check_fo(fo);
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_leader() {
    // fo:leader for TOC dots/lines (Section 8.3.4)
    let body = r##"
      <fo:block text-align-last="justify">
        Chapter 1<fo:leader leader-pattern="dots"/>1
      </fo:block>
      <fo:block text-align-last="justify">
        Chapter 2: A Longer Chapter Title<fo:leader leader-pattern="dots"/>12
      </fo:block>
      <fo:block text-align-last="justify">
        Chapter 3<fo:leader leader-pattern="rule"/>25
      </fo:block>
      <fo:block text-align-last="justify">
        Chapter 4<fo:leader leader-pattern="space"/>42
      </fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_basic_link_external() {
    // fo:basic-link external-destination (Section 8.4)
    let body = r##"
      <fo:block>
        Visit <fo:basic-link external-destination="https://www.w3.org/TR/xsl11/">
          W3C XSL-FO 1.1 Specification
        </fo:basic-link> for details.
      </fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_basic_link_internal() {
    // fo:basic-link internal-destination (Section 8.4)
    let body = r##"
      <fo:block>
        See <fo:basic-link internal-destination="section2">Section 2</fo:basic-link> below.
      </fo:block>
      <fo:block id="section2" space-before="20pt">
        This is Section 2 (the link target).
      </fo:block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

// ---------------------------------------------------------------------------
// Section 9: Table formatting objects
// ---------------------------------------------------------------------------

#[test]
fn conformance_table_basic() {
    // Basic table with explicit column widths (Section 9.3)
    let body = r##"
      <fo:table width="100%" table-layout="fixed">
        <fo:table-column column-width="33%"/>
        <fo:table-column column-width="33%"/>
        <fo:table-column column-width="34%"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell border="0.5pt solid black" padding="4pt">
              <fo:block>Cell A1</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="4pt">
              <fo:block>Cell B1</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="4pt">
              <fo:block>Cell C1</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell border="0.5pt solid black" padding="4pt">
              <fo:block>Cell A2</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="4pt">
              <fo:block>Cell B2</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="4pt">
              <fo:block>Cell C2</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_table_with_header() {
    // Table with header and footer (Section 9.3.1, 9.3.2)
    let body = r##"
      <fo:table width="100%" table-layout="fixed">
        <fo:table-column column-width="50%"/>
        <fo:table-column column-width="50%"/>
        <fo:table-header>
          <fo:table-row>
            <fo:table-cell background-color="#CCCCCC" border="0.5pt solid black" padding="4pt">
              <fo:block font-weight="bold">Name</fo:block>
            </fo:table-cell>
            <fo:table-cell background-color="#CCCCCC" border="0.5pt solid black" padding="4pt">
              <fo:block font-weight="bold">Value</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-header>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell border="0.5pt solid black" padding="4pt">
              <fo:block>Item A</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="4pt">
              <fo:block>100</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell border="0.5pt solid black" padding="4pt">
              <fo:block>Item B</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="4pt">
              <fo:block>200</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_table_column_spanning() {
    // number-columns-spanned (Section 9.5)
    let body = r##"
      <fo:table width="100%" table-layout="fixed">
        <fo:table-column column-width="25%"/>
        <fo:table-column column-width="25%"/>
        <fo:table-column column-width="25%"/>
        <fo:table-column column-width="25%"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell number-columns-spanned="4" border="0.5pt solid black" padding="4pt">
              <fo:block text-align="center" font-weight="bold">Full-width header (spans 4 columns)</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell number-columns-spanned="2" border="0.5pt solid black" padding="4pt">
              <fo:block>Spans 2 columns</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="4pt">
              <fo:block>Col 3</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="4pt">
              <fo:block>Col 4</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_table_row_spanning() {
    // number-rows-spanned (Section 9.5)
    let body = r##"
      <fo:table width="100%" table-layout="fixed">
        <fo:table-column column-width="33%"/>
        <fo:table-column column-width="33%"/>
        <fo:table-column column-width="34%"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell number-rows-spanned="2" border="0.5pt solid black" padding="4pt">
              <fo:block>Spans 2 rows</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="4pt">
              <fo:block>Row 1 Col 2</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="4pt">
              <fo:block>Row 1 Col 3</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell border="0.5pt solid black" padding="4pt">
              <fo:block>Row 2 Col 2</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="4pt">
              <fo:block>Row 2 Col 3</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

// ---------------------------------------------------------------------------
// Section 10: List formatting objects
// ---------------------------------------------------------------------------

#[test]
fn conformance_list_block_basic() {
    // fo:list-block with fo:list-item (Section 10.3)
    let body = r##"
      <fo:list-block provisional-distance-between-starts="15mm"
          provisional-label-separation="3mm">
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()">
            <fo:block>&#x2022;</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>First list item</fo:block>
          </fo:list-item-body>
        </fo:list-item>
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()">
            <fo:block>&#x2022;</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>Second list item with more text</fo:block>
          </fo:list-item-body>
        </fo:list-item>
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()">
            <fo:block>&#x2022;</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>Third list item</fo:block>
          </fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_ordered_list() {
    // Numbered list (Section 10.3)
    let body = r##"
      <fo:list-block provisional-distance-between-starts="15mm"
          provisional-label-separation="3mm">
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()">
            <fo:block>1.</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>First numbered item</fo:block>
          </fo:list-item-body>
        </fo:list-item>
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()">
            <fo:block>2.</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>Second numbered item</fo:block>
          </fo:list-item-body>
        </fo:list-item>
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()">
            <fo:block>3.</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>Third numbered item with longer text that wraps</fo:block>
          </fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
    "##;
    let pdf = check_fo(&wrap_fo(body));
    validate_pdf_bytes(&pdf);
}
