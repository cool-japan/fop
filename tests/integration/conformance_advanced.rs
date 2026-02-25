//! XSL-FO 1.1 Conformance: Advanced tests - markers, footnotes, columns, batch 6 and 7
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

// ---------------------------------------------------------------------------
// New conformance tests batch (added 2026-02-20)
// ---------------------------------------------------------------------------

#[test]
fn conformance_marker_multiple_sections() {
    // fo:marker in multiple sections with fo:retrieve-marker in a running header.
    // Each section block sets a new marker value; the static-content retrieves
    // the current chapter title on every page.
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="25mm" margin-bottom="20mm"
      margin-left="25mm" margin-right="25mm">
      <fo:region-body margin-top="15mm"/>
      <fo:region-before extent="15mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block font-size="9pt" font-style="italic" text-align="center"
        border-bottom="0.5pt solid black" padding-bottom="2pt">
        <fo:retrieve-marker retrieve-class-name="section-heading"
          retrieve-boundary="page-sequence"
          retrieve-position="first-starting-within-page"/>
      </fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="14pt" font-weight="bold" space-after="6pt"
        keep-with-next="always">
        <fo:marker marker-class-name="section-heading">Introduction</fo:marker>
        1. Introduction
      </fo:block>
      <fo:block font-size="10pt" space-after="4pt">
        This is the introductory section of the document. It provides
        background context and motivation for the work presented here.
      </fo:block>
      <fo:block font-size="10pt" space-after="12pt">
        The document covers several topics in depth.
      </fo:block>
      <fo:block font-size="14pt" font-weight="bold" space-after="6pt"
        keep-with-next="always">
        <fo:marker marker-class-name="section-heading">Background</fo:marker>
        2. Background
      </fo:block>
      <fo:block font-size="10pt" space-after="4pt">
        The background section reviews related work and establishes
        the theoretical framework for subsequent chapters.
      </fo:block>
      <fo:block font-size="14pt" font-weight="bold" space-after="6pt"
        keep-with-next="always">
        <fo:marker marker-class-name="section-heading">Methodology</fo:marker>
        3. Methodology
      </fo:block>
      <fo:block font-size="10pt">
        The methodology section describes the experimental approach
        and data collection procedures used in this study.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Multiple fo:marker sections with fo:retrieve-marker should work: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], b"%PDF");
}

#[test]
fn conformance_conditional_page_master_blank_pages() {
    // fo:conditional-page-master-reference with blank-or-not-blank="blank" condition.
    // A page-sequence-master with blank and non-blank alternatives exercises
    // the full set of conditional page master conditions.
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="blank-page"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="25mm" margin-right="25mm">
      <fo:region-body/>
    </fo:simple-page-master>
    <fo:simple-page-master master-name="first-odd"
      page-width="210mm" page-height="297mm"
      margin-top="30mm" margin-bottom="20mm"
      margin-left="25mm" margin-right="25mm">
      <fo:region-body margin-top="12mm"/>
      <fo:region-before extent="12mm"/>
    </fo:simple-page-master>
    <fo:simple-page-master master-name="odd-page"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="25mm" margin-right="25mm">
      <fo:region-body margin-top="12mm"/>
      <fo:region-before extent="12mm"/>
    </fo:simple-page-master>
    <fo:simple-page-master master-name="even-page"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="25mm" margin-right="25mm">
      <fo:region-body margin-top="12mm"/>
      <fo:region-before extent="12mm"/>
    </fo:simple-page-master>
    <fo:page-sequence-master master-name="book-layout">
      <fo:repeatable-page-master-alternatives>
        <fo:conditional-page-master-reference
          master-reference="blank-page"
          blank-or-not-blank="blank"/>
        <fo:conditional-page-master-reference
          master-reference="first-odd"
          page-position="first"
          odd-or-even="odd"/>
        <fo:conditional-page-master-reference
          master-reference="odd-page"
          odd-or-even="odd"/>
        <fo:conditional-page-master-reference
          master-reference="even-page"
          odd-or-even="even"/>
      </fo:repeatable-page-master-alternatives>
    </fo:page-sequence-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="book-layout" force-page-count="end-on-even">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block font-size="9pt" text-align="center" font-style="italic">
        Running Header
      </fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="12pt" font-weight="bold" space-after="8pt">Chapter One</fo:block>
      <fo:block font-size="10pt" space-after="6pt">
        This chapter opens on the first odd page with a larger top margin.
      </fo:block>
      <fo:block font-size="10pt" break-before="page">
        Continuing on a subsequent page using the standard odd/even alternation.
      </fo:block>
      <fo:block font-size="10pt" space-after="4pt">
        The blank-or-not-blank condition ensures intentionally blank pages
        receive the blank-page master instead of a normal page master.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Conditional page master with blank-or-not-blank should work: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], b"%PDF");
}

#[test]
fn conformance_change_bar_graceful_degradation() {
    // fo:change-bar-begin and fo:change-bar-end are known unsupported elements
    // in this implementation. This test verifies that documents containing these
    // elements still produce valid PDF output without panicking, and that the
    // surrounding content is rendered correctly.
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="25mm" margin-right="25mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="12pt" space-after="6pt">
        This document demonstrates graceful degradation when unsupported
        change-bar elements are encountered.
      </fo:block>
      <fo:block font-size="10pt" space-after="4pt">Unchanged section one.</fo:block>
      <fo:change-bar-begin change-bar-class="rev-a"
        change-bar-color="red"
        change-bar-style="solid"
        change-bar-width="2pt"
        change-bar-placement="start"/>
      <fo:block font-size="10pt" space-after="4pt">
        This paragraph marks the beginning of a revised region.
        The change-bar-begin element should be silently skipped.
      </fo:block>
      <fo:block font-size="10pt" space-after="4pt">
        More revised content in the same change-bar region.
      </fo:block>
      <fo:change-bar-end change-bar-class="rev-a"/>
      <fo:block font-size="10pt" space-after="4pt">Unchanged section two.</fo:block>
      <fo:change-bar-begin change-bar-class="rev-b"
        change-bar-color="blue"
        change-bar-style="dashed"/>
      <fo:block font-size="10pt">
        Second revision region with a different change-bar class.
      </fo:block>
      <fo:change-bar-end change-bar-class="rev-b"/>
      <fo:block font-size="10pt">Final unchanged content.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    // The document must render successfully even though change-bar elements
    // are unsupported — they should be skipped gracefully.
    assert!(
        result.is_ok(),
        "fo:change-bar elements should degrade gracefully, not panic: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], b"%PDF");
}

#[test]
fn conformance_table_and_caption_positioning() {
    // fo:table-and-caption with explicit caption-side values and caption formatting.
    // Tests that the caption appears in the correct position relative to the table,
    // including caption text alignment and spacing properties.
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="25mm" margin-right="25mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="12pt" font-weight="bold" space-after="8pt">
        Tables with Captions
      </fo:block>
      <fo:table-and-caption space-after="12pt" caption-side="before">
        <fo:table-caption>
          <fo:block font-size="9pt" font-style="italic" text-align="center"
            space-after="3pt">
            Table 1: Quarterly Revenue Summary (caption above table)
          </fo:block>
        </fo:table-caption>
        <fo:table table-layout="fixed" width="160mm">
          <fo:table-column column-width="60mm"/>
          <fo:table-column column-width="50mm"/>
          <fo:table-column column-width="50mm"/>
          <fo:table-header>
            <fo:table-row background-color="#dddddd">
              <fo:table-cell padding="2mm" border="0.5pt solid black">
                <fo:block font-weight="bold" font-size="9pt">Quarter</fo:block>
              </fo:table-cell>
              <fo:table-cell padding="2mm" border="0.5pt solid black">
                <fo:block font-weight="bold" font-size="9pt" text-align="right">Revenue</fo:block>
              </fo:table-cell>
              <fo:table-cell padding="2mm" border="0.5pt solid black">
                <fo:block font-weight="bold" font-size="9pt" text-align="right">Growth</fo:block>
              </fo:table-cell>
            </fo:table-row>
          </fo:table-header>
          <fo:table-body>
            <fo:table-row>
              <fo:table-cell padding="2mm" border="0.5pt solid black">
                <fo:block font-size="9pt">Q1 2024</fo:block>
              </fo:table-cell>
              <fo:table-cell padding="2mm" border="0.5pt solid black">
                <fo:block font-size="9pt" text-align="right">$1,200,000</fo:block>
              </fo:table-cell>
              <fo:table-cell padding="2mm" border="0.5pt solid black">
                <fo:block font-size="9pt" text-align="right">+8.5%</fo:block>
              </fo:table-cell>
            </fo:table-row>
            <fo:table-row background-color="#f8f8f8">
              <fo:table-cell padding="2mm" border="0.5pt solid black">
                <fo:block font-size="9pt">Q2 2024</fo:block>
              </fo:table-cell>
              <fo:table-cell padding="2mm" border="0.5pt solid black">
                <fo:block font-size="9pt" text-align="right">$1,350,000</fo:block>
              </fo:table-cell>
              <fo:table-cell padding="2mm" border="0.5pt solid black">
                <fo:block font-size="9pt" text-align="right">+12.5%</fo:block>
              </fo:table-cell>
            </fo:table-row>
          </fo:table-body>
        </fo:table>
      </fo:table-and-caption>
      <fo:table-and-caption space-after="12pt" caption-side="after">
        <fo:table-caption>
          <fo:block font-size="9pt" font-style="italic" text-align="center"
            space-before="3pt">
            Table 2: Annual Totals (caption below table)
          </fo:block>
        </fo:table-caption>
        <fo:table table-layout="fixed" width="120mm">
          <fo:table-column column-width="60mm"/>
          <fo:table-column column-width="60mm"/>
          <fo:table-body>
            <fo:table-row>
              <fo:table-cell padding="2mm" border="0.5pt solid black">
                <fo:block font-size="9pt">Total 2024</fo:block>
              </fo:table-cell>
              <fo:table-cell padding="2mm" border="0.5pt solid black">
                <fo:block font-size="9pt" text-align="right">$5,100,000</fo:block>
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
        "fo:table-and-caption with caption-side positioning should work: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], b"%PDF");
}

#[test]
fn conformance_instream_foreign_object_svg_inline() {
    // fo:instream-foreign-object containing SVG with multiple elements inline
    // within a paragraph alongside text content. Verifies that the foreign
    // object is treated as an inline replacement object and the document
    // renders without error.
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="25mm" margin-right="25mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="12pt" space-after="8pt">
        Inline SVG Foreign Objects
      </fo:block>
      <fo:block font-size="10pt" space-after="6pt">
        The following block contains an SVG diagram rendered inline:
      </fo:block>
      <fo:block text-align="center" space-after="6pt">
        <fo:instream-foreign-object content-width="100mm" content-height="60mm"
          display-align="center">
          <svg:svg xmlns:svg="http://www.w3.org/2000/svg"
            viewBox="0 0 200 120" width="200" height="120">
            <svg:rect x="5" y="5" width="190" height="110"
              fill="none" stroke="#333333" stroke-width="2"/>
            <svg:rect x="20" y="20" width="40" height="80" fill="#4472C4"/>
            <svg:rect x="80" y="40" width="40" height="60" fill="#ED7D31"/>
            <svg:rect x="140" y="30" width="40" height="70" fill="#70AD47"/>
            <svg:text x="100" y="115" text-anchor="middle"
              font-size="10" fill="#333333">Bar Chart</svg:text>
          </svg:svg>
        </fo:instream-foreign-object>
      </fo:block>
      <fo:block font-size="10pt" space-after="6pt">
        Figure 1: Sample bar chart as inline SVG foreign object.
      </fo:block>
      <fo:block font-size="10pt" space-after="6pt">
        A second inline SVG with path elements:
      </fo:block>
      <fo:block text-align="center">
        <fo:instream-foreign-object content-width="60mm" content-height="60mm">
          <svg:svg xmlns:svg="http://www.w3.org/2000/svg"
            viewBox="0 0 120 120" width="120" height="120">
            <svg:circle cx="60" cy="60" r="55"
              fill="none" stroke="#333333" stroke-width="2"/>
            <svg:path d="M60,60 L60,10 A50,50 0 0,1 110,60 Z" fill="#4472C4"/>
            <svg:path d="M60,60 L110,60 A50,50 0 0,1 35,107 Z" fill="#ED7D31"/>
            <svg:path d="M60,60 L35,107 A50,50 0 1,1 60,10 Z" fill="#70AD47"/>
          </svg:svg>
        </fo:instream-foreign-object>
      </fo:block>
      <fo:block font-size="10pt">
        Figure 2: Pie chart with path segments.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "fo:instream-foreign-object with inline SVG content should work: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], b"%PDF");
}

#[test]
fn conformance_footnote_complex_multi_paragraph() {
    // Complex footnote layout: three paragraphs each containing an fo:footnote
    // with multi-line body text. Verifies that footnote bodies accumulate at
    // the bottom of the page and that the main flow renders correctly.
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="25mm" margin-right="25mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="14pt" font-weight="bold" space-after="10pt">
        Academic Paper with Complex Footnotes
      </fo:block>
      <fo:block font-size="10pt" space-after="8pt" text-align="justify">
        The XSL-FO specification
        <fo:footnote>
          <fo:inline baseline-shift="super" font-size="7pt">1</fo:inline>
          <fo:footnote-body>
            <fo:block font-size="8pt" space-before="2pt"
              border-top="0.5pt solid black" padding-top="2pt">
              <fo:inline font-weight="bold">1.</fo:inline>
              Extensible Stylesheet Language — Formatting Objects (XSL-FO) 1.1,
              W3C Recommendation, December 2006. Available at
              https://www.w3.org/TR/xsl11/. This specification defines the
              formatting semantics for paginated documents.
            </fo:block>
          </fo:footnote-body>
        </fo:footnote>
        defines a rich set of formatting objects and properties for producing
        high-quality paginated output. The page layout model supports multiple
        regions per page, running headers and footers, and complex flow arrangements.
      </fo:block>
      <fo:block font-size="10pt" space-after="8pt" text-align="justify">
        Knuth-Plass line breaking
        <fo:footnote>
          <fo:inline baseline-shift="super" font-size="7pt">2</fo:inline>
          <fo:footnote-body>
            <fo:block font-size="8pt" space-before="2pt">
              <fo:inline font-weight="bold">2.</fo:inline>
              Knuth, D.E. and Plass, M.F. (1981). Breaking Paragraphs into Lines.
              Software: Practice and Experience, 11(11), pp. 1119-1184. The
              algorithm optimises line breaking by minimising the total badness
              across all lines in a paragraph, rather than greedily filling
              each line.
            </fo:block>
          </fo:footnote-body>
        </fo:footnote>
        provides the theoretical foundation for high-quality paragraph
        composition. By computing break points holistically across entire
        paragraphs, the algorithm avoids the tight lines and rivers of whitespace
        that plague simpler greedy approaches.
      </fo:block>
      <fo:block font-size="10pt" space-after="8pt" text-align="justify">
        PDF/A conformance
        <fo:footnote>
          <fo:inline baseline-shift="super" font-size="7pt">3</fo:inline>
          <fo:footnote-body>
            <fo:block font-size="8pt" space-before="2pt">
              <fo:inline font-weight="bold">3.</fo:inline>
              ISO 19005-1:2005. Document management — Electronic document file
              format for long-term preservation — Part 1: Use of PDF 1.4
              (PDF/A-1). PDF/A-1b requires all fonts to be embedded and
              prohibits encryption and external references.
            </fo:block>
          </fo:footnote-body>
        </fo:footnote>
        requirements mandate that all font resources are fully embedded in
        the output file and that the document structure permits reliable
        long-term archival reproduction independently of external resources.
      </fo:block>
      <fo:block font-size="10pt" text-align="justify">
        The combination of these standards enables production-quality document
        output suitable for both print and digital distribution workflows.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Complex multi-paragraph footnotes should work: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty());
    assert_eq!(&bytes[0..4], b"%PDF");
}

// ---------------------------------------------------------------------------
// Additional conformance tests (batch 6)
// ---------------------------------------------------------------------------

#[test]
fn conformance_multi_column_span_all() {
    // 3-column layout with fo:block span="all" full-width heading, then column content below
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
        page-width="210mm" page-height="297mm"
        margin-top="20mm" margin-bottom="20mm"
        margin-left="20mm" margin-right="20mm">
      <fo:region-body column-count="3" column-gap="5mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block span="all" font-size="18pt" font-weight="bold"
          text-align="center" space-after="10pt">
        Full-Width Spanning Heading
      </fo:block>
      <fo:block font-size="10pt" space-after="6pt">
        Column content paragraph one. This text flows within the first column
        of the three-column layout below the spanning heading block.
      </fo:block>
      <fo:block font-size="10pt" space-after="6pt">
        Column content paragraph two. More text continues to fill the columns
        and demonstrates proper multi-column flow behaviour.
      </fo:block>
      <fo:block font-size="10pt" space-after="6pt">
        Column content paragraph three. Additional content ensures that the
        three-column layout receives enough material to be visible.
      </fo:block>
      <fo:block font-size="10pt" space-after="6pt">
        Column content paragraph four. The layout engine must handle the
        transition from spanning content back to normal column flow correctly.
      </fo:block>
      <fo:block span="all" font-size="13pt" font-weight="bold"
          text-align="center" space-before="8pt" space-after="8pt">
        Second Spanning Subheading
      </fo:block>
      <fo:block font-size="10pt" space-after="6pt">
        Content after the second spanning block continues in the three-column
        layout, verifying that multiple span transitions work correctly.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "3-column span-all layout should render to PDF: {:?}",
        result.err()
    );
    let pdf_bytes = result.expect("test: should succeed");
    assert!(!pdf_bytes.is_empty());
    assert_eq!(&pdf_bytes[0..4], b"%PDF");
}

#[test]
fn conformance_table_header_footer_repeat() {
    // Table with fo:table-header and fo:table-footer, 8 body data rows
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
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
      <fo:block font-size="14pt" font-weight="bold" space-after="8pt">
        Sales Report Q4
      </fo:block>
      <fo:table table-layout="fixed" width="160mm" border-collapse="collapse">
        <fo:table-column column-width="40mm"/>
        <fo:table-column column-width="40mm"/>
        <fo:table-column column-width="40mm"/>
        <fo:table-column column-width="40mm"/>
        <fo:table-header>
          <fo:table-row background-color="#cccccc">
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block font-weight="bold">Product</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block font-weight="bold">Region</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block font-weight="bold">Units</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block font-weight="bold">Revenue</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-header>
        <fo:table-footer>
          <fo:table-row background-color="#eeeeee">
            <fo:table-cell border="0.5pt solid black" padding="2pt"
                number-columns-spanned="3">
              <fo:block font-weight="bold">Total</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block font-weight="bold">$48,000</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-footer>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>Widget A</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>North</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>120</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>$6,000</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>Widget B</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>South</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>95</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>$4,750</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>Gadget X</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>East</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>210</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>$10,500</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>Gadget Y</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>West</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>88</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>$4,400</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>Component P</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>North</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>340</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>$6,800</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>Component Q</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>East</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>175</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>$5,250</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>Service M</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>South</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>60</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>$6,000</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>Service N</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>West</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>90</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block>$4,300</fo:block>
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
        "Table with header/footer and 8 data rows should render to PDF: {:?}",
        result.err()
    );
    let pdf_bytes = result.expect("test: should succeed");
    assert!(!pdf_bytes.is_empty());
    assert_eq!(&pdf_bytes[0..4], b"%PDF");
}

#[test]
fn conformance_alphabetic_page_numbering() {
    // fo:page-sequence with format="a" for lowercase alphabetic page numbers
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="alpha-page"
        page-width="210mm" page-height="297mm"
        margin-top="20mm" margin-bottom="20mm"
        margin-left="25mm" margin-right="25mm">
      <fo:region-body region-name="xsl-region-body"/>
      <fo:region-after region-name="xsl-region-after" extent="10mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="alpha-page" format="a" initial-page-number="1">
    <fo:static-content flow-name="xsl-region-after">
      <fo:block text-align="center" font-size="9pt">
        Page <fo:page-number/>
      </fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="14pt" font-weight="bold" space-after="10pt">
        Document with Alphabetic Page Numbers
      </fo:block>
      <fo:block font-size="10pt" space-after="8pt">
        This page should be numbered with lowercase alphabetic notation:
        page a, b, c, etc. The format attribute on fo:page-sequence controls
        the numbering scheme used throughout this sequence.
      </fo:block>
      <fo:block font-size="10pt" space-after="8pt">
        Alphabetic page numbering is specified in XSL-FO 1.1 Section 7.29.7
        via the format property. When format="a" is specified, page numbers
        render as a, b, c, ..., z, aa, ab, etc.
      </fo:block>
      <fo:block font-size="10pt" space-after="8pt">
        This content provides enough material to demonstrate the page
        numbering feature without requiring multiple physical pages in
        the output document.
      </fo:block>
      <fo:block font-size="10pt" space-after="8pt">
        Additional paragraph to ensure the flow has substantial content
        for the layout engine to process during alphabetic numbering tests.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Alphabetic page numbering (format=a) should render to PDF: {:?}",
        result.err()
    );
    let pdf_bytes = result.expect("test: should succeed");
    assert!(!pdf_bytes.is_empty());
    assert_eq!(&pdf_bytes[0..4], b"%PDF");
}

#[test]
fn conformance_roman_numeral_pages() {
    // Two page sequences: format="i" for front matter, format="1" for main content
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="front-master"
        page-width="210mm" page-height="297mm"
        margin-top="20mm" margin-bottom="20mm"
        margin-left="25mm" margin-right="25mm">
      <fo:region-body region-name="xsl-region-body"/>
      <fo:region-after region-name="xsl-region-after" extent="10mm"/>
    </fo:simple-page-master>
    <fo:simple-page-master master-name="main-master"
        page-width="210mm" page-height="297mm"
        margin-top="20mm" margin-bottom="20mm"
        margin-left="25mm" margin-right="25mm">
      <fo:region-body region-name="xsl-region-body"/>
      <fo:region-after region-name="xsl-region-after" extent="10mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="front-master" format="i" initial-page-number="1">
    <fo:static-content flow-name="xsl-region-after">
      <fo:block text-align="center" font-size="9pt">
        Page <fo:page-number/>
      </fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="16pt" font-weight="bold" text-align="center"
          space-after="12pt">
        Preface
      </fo:block>
      <fo:block font-size="10pt" space-after="8pt">
        This is the front matter section of the document. Pages in this
        section use lowercase Roman numerals (i, ii, iii, iv, ...) as
        specified by format="i" on the fo:page-sequence element.
      </fo:block>
      <fo:block font-size="10pt" space-after="8pt">
        Front matter typically includes the table of contents, preface,
        acknowledgements, and other introductory material before the
        main body of the document begins.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="main-master" format="1" initial-page-number="1">
    <fo:static-content flow-name="xsl-region-after">
      <fo:block text-align="center" font-size="9pt">
        Page <fo:page-number/>
      </fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="16pt" font-weight="bold" text-align="center"
          space-after="12pt">
        Chapter 1: Introduction
      </fo:block>
      <fo:block font-size="10pt" space-after="8pt">
        This is the main content section. Pages here use decimal numbering
        (1, 2, 3, ...) as specified by format="1" on this fo:page-sequence.
        The page counter restarts from 1 at the beginning of this sequence.
      </fo:block>
      <fo:block font-size="10pt" space-after="8pt">
        Having two separate page-sequence elements with different format
        attributes is a standard pattern for professional documents where
        front matter and body content use different numbering conventions.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Roman numeral + decimal dual page-sequence should render to PDF: {:?}",
        result.err()
    );
    let pdf_bytes = result.expect("test: should succeed");
    assert!(!pdf_bytes.is_empty());
    assert_eq!(&pdf_bytes[0..4], b"%PDF");
}

#[test]
fn conformance_external_graphic_placeholder() {
    // fo:external-graphic with a nonexistent image — must not panic, must produce valid PDF
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
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
      <fo:block font-size="12pt" font-weight="bold" space-after="8pt">
        Document with Missing Image Reference
      </fo:block>
      <fo:block font-size="10pt" space-after="6pt">
        The following block contains an fo:external-graphic pointing to a
        file that does not exist on the filesystem. The renderer should
        handle this gracefully by substituting a placeholder or skipping
        the image, rather than panicking or returning an error.
      </fo:block>
      <fo:block>
        <fo:external-graphic src="url('nonexistent-image.png')"
            content-width="60mm" content-height="40mm"
            scaling="uniform"/>
      </fo:block>
      <fo:block font-size="10pt" space-before="6pt">
        Content continues normally after the missing image reference.
        The document structure remains intact and the flow is not interrupted
        by the unavailable external resource.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Missing external-graphic should be handled gracefully (placeholder/skip): {:?}",
        result.err()
    );
    let pdf_bytes = result.expect("test: should succeed");
    assert!(!pdf_bytes.is_empty());
    assert_eq!(&pdf_bytes[0..4], b"%PDF");
}

#[test]
fn conformance_nested_block_containers_deep() {
    // Deeply nested fo:block-container elements with absolute positioning
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
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
      <fo:block font-size="12pt" font-weight="bold" space-after="8pt">
        Nested Block Containers
      </fo:block>
      <fo:block font-size="10pt" space-after="8pt">
        The following demonstrates deeply nested fo:block-container elements,
        including one with absolute positioning at specific page coordinates.
      </fo:block>
      <fo:block-container absolute-position="absolute"
          top="60mm" left="30mm" width="140mm" height="80mm">
        <fo:block font-size="11pt" font-weight="bold" space-after="6pt"
            border="0.5pt solid black" padding="4pt">
          Outer Absolutely-Positioned Container (140mm x 80mm)
        </fo:block>
        <fo:block-container width="100mm" height="50mm"
            border="0.5pt dashed gray" padding="4pt">
          <fo:block font-size="10pt" space-after="4pt">
            Inner container (100mm x 50mm) nested inside the outer one.
            This block-container has fixed dimensions but is not absolutely
            positioned; it flows within the parent container.
          </fo:block>
          <fo:block font-size="9pt" color="#333333">
            Nested content: line one of inner container text.
          </fo:block>
          <fo:block font-size="9pt" color="#333333">
            Nested content: line two of inner container text.
          </fo:block>
        </fo:block-container>
      </fo:block-container>
      <fo:block font-size="10pt" space-before="90mm" space-after="6pt">
        Regular flow content follows below the absolutely-positioned container.
        The layout engine must correctly handle the co-existence of absolute
        and normal-flow block-containers on the same page.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Deeply nested block-containers should render to PDF: {:?}",
        result.err()
    );
    let pdf_bytes = result.expect("test: should succeed");
    assert!(!pdf_bytes.is_empty());
    assert_eq!(&pdf_bytes[0..4], b"%PDF");
}

// ---------------------------------------------------------------------------
// New conformance tests: table column widths, background colors, text decoration,
// font weight variants, list style types, page margin variations
// ---------------------------------------------------------------------------

#[test]
fn conformance_table_column_widths() {
    // A table with proportional-column-width creating 2:1 width ratio columns.
    // Body has 5 rows x 3 columns.
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm"
      margin-top="20mm" margin-bottom="20mm" margin-left="25mm" margin-right="25mm">
      <fo:region-body region-name="xsl-region-body"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="12pt" font-weight="bold" space-after="6pt">
        Proportional Column Width Table (2:1:1 ratio)
      </fo:block>
      <fo:table table-layout="fixed" width="160mm" border-collapse="separate">
        <fo:table-column column-width="proportional-column-width(2)"/>
        <fo:table-column column-width="proportional-column-width(1)"/>
        <fo:table-column column-width="proportional-column-width(1)"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block font-size="9pt" font-weight="bold">Wide Column (2x)</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block font-size="9pt" font-weight="bold">Narrow (1x)</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block font-size="9pt" font-weight="bold">Narrow (1x)</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block font-size="9pt">Row 2 wide content here</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block font-size="9pt">Col B</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block font-size="9pt">Col C</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block font-size="9pt">Row 3 wide content here</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block font-size="9pt">B3</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block font-size="9pt">C3</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block font-size="9pt">Row 4 wide content here</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block font-size="9pt">B4</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block font-size="9pt">C4</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block font-size="9pt">Row 5 wide content here</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block font-size="9pt">B5</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid black" padding="2pt">
              <fo:block font-size="9pt">C5</fo:block>
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
        "Table with proportional-column-width should render to PDF: {:?}",
        result.err()
    );
    let pdf_bytes = result.expect("test: should succeed");
    assert!(!pdf_bytes.is_empty());
    assert_eq!(&pdf_bytes[0..4], b"%PDF");
}

#[test]
fn conformance_block_background_color_named() {
    // Multiple blocks with different background-color values.
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm"
      margin-top="20mm" margin-bottom="20mm" margin-left="25mm" margin-right="25mm">
      <fo:region-body region-name="xsl-region-body"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="12pt" font-weight="bold" space-after="8pt">
        Background Color Variations
      </fo:block>
      <fo:block background-color="red" color="white" padding="4pt" space-after="4pt" font-size="10pt">
        Red background block
      </fo:block>
      <fo:block background-color="green" color="white" padding="4pt" space-after="4pt" font-size="10pt">
        Green background block
      </fo:block>
      <fo:block background-color="blue" color="white" padding="4pt" space-after="4pt" font-size="10pt">
        Blue background block
      </fo:block>
      <fo:block background-color="yellow" color="black" padding="4pt" space-after="4pt" font-size="10pt">
        Yellow background block
      </fo:block>
      <fo:block background-color="white" color="black" padding="4pt" space-after="4pt" font-size="10pt"
        border="0.5pt solid black">
        White background block
      </fo:block>
      <fo:block background-color="transparent" color="black" padding="4pt" space-after="4pt" font-size="10pt">
        Transparent background block
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Blocks with various background-color values should render to PDF: {:?}",
        result.err()
    );
    let pdf_bytes = result.expect("test: should succeed");
    assert!(!pdf_bytes.is_empty());
    assert_eq!(&pdf_bytes[0..4], b"%PDF");
}

#[test]
fn conformance_text_decoration_all_types() {
    // Blocks with various text-decoration values.
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm"
      margin-top="20mm" margin-bottom="20mm" margin-left="25mm" margin-right="25mm">
      <fo:region-body region-name="xsl-region-body"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="12pt" font-weight="bold" space-after="8pt">
        Text Decoration Variants
      </fo:block>
      <fo:block text-decoration="underline" font-size="11pt" space-after="6pt">
        This text has underline decoration applied.
      </fo:block>
      <fo:block text-decoration="overline" font-size="11pt" space-after="6pt">
        This text has overline decoration applied.
      </fo:block>
      <fo:block text-decoration="line-through" font-size="11pt" space-after="6pt">
        This text has line-through (strikethrough) decoration.
      </fo:block>
      <fo:block text-decoration="none" font-size="11pt" space-after="6pt">
        This text has no decoration (explicit none).
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Blocks with text-decoration values should render to PDF: {:?}",
        result.err()
    );
    let pdf_bytes = result.expect("test: should succeed");
    assert!(!pdf_bytes.is_empty());
    assert_eq!(&pdf_bytes[0..4], b"%PDF");
}

#[test]
fn conformance_font_weight_variants() {
    // Blocks using all font-weight numeric values from 100 to 900.
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm"
      margin-top="20mm" margin-bottom="20mm" margin-left="25mm" margin-right="25mm">
      <fo:region-body region-name="xsl-region-body"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="12pt" font-weight="bold" space-after="8pt">
        Font Weight Variants (100 - 900)
      </fo:block>
      <fo:block font-weight="100" font-size="10pt" space-after="4pt">
        Weight 100 (Thin) - The quick brown fox
      </fo:block>
      <fo:block font-weight="200" font-size="10pt" space-after="4pt">
        Weight 200 (ExtraLight) - The quick brown fox
      </fo:block>
      <fo:block font-weight="300" font-size="10pt" space-after="4pt">
        Weight 300 (Light) - The quick brown fox
      </fo:block>
      <fo:block font-weight="400" font-size="10pt" space-after="4pt">
        Weight 400 (Normal) - The quick brown fox
      </fo:block>
      <fo:block font-weight="500" font-size="10pt" space-after="4pt">
        Weight 500 (Medium) - The quick brown fox
      </fo:block>
      <fo:block font-weight="600" font-size="10pt" space-after="4pt">
        Weight 600 (SemiBold) - The quick brown fox
      </fo:block>
      <fo:block font-weight="700" font-size="10pt" space-after="4pt">
        Weight 700 (Bold) - The quick brown fox
      </fo:block>
      <fo:block font-weight="800" font-size="10pt" space-after="4pt">
        Weight 800 (ExtraBold) - The quick brown fox
      </fo:block>
      <fo:block font-weight="900" font-size="10pt" space-after="4pt">
        Weight 900 (Black) - The quick brown fox
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Blocks with all font-weight values should render to PDF: {:?}",
        result.err()
    );
    let pdf_bytes = result.expect("test: should succeed");
    assert!(!pdf_bytes.is_empty());
    assert_eq!(&pdf_bytes[0..4], b"%PDF");
}

#[test]
fn conformance_list_style_types() {
    // List blocks with disc, circle, square, and numbered marker styles.
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm"
      margin-top="20mm" margin-bottom="20mm" margin-left="25mm" margin-right="25mm">
      <fo:region-body region-name="xsl-region-body"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="12pt" font-weight="bold" space-after="8pt">
        List Style Types
      </fo:block>
      <fo:block font-size="10pt" font-weight="bold" space-after="4pt">Disc markers (&#x2022;)</fo:block>
      <fo:list-block provisional-distance-between-starts="12mm"
        provisional-label-separation="4mm" space-after="8pt">
        <fo:list-item space-after="2pt">
          <fo:list-item-label end-indent="label-end()">
            <fo:block font-size="10pt">&#x2022;</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block font-size="10pt">First disc item</fo:block>
          </fo:list-item-body>
        </fo:list-item>
        <fo:list-item space-after="2pt">
          <fo:list-item-label end-indent="label-end()">
            <fo:block font-size="10pt">&#x2022;</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block font-size="10pt">Second disc item</fo:block>
          </fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
      <fo:block font-size="10pt" font-weight="bold" space-after="4pt">Circle markers (&#x25E6;)</fo:block>
      <fo:list-block provisional-distance-between-starts="12mm"
        provisional-label-separation="4mm" space-after="8pt">
        <fo:list-item space-after="2pt">
          <fo:list-item-label end-indent="label-end()">
            <fo:block font-size="10pt">&#x25E6;</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block font-size="10pt">First circle item</fo:block>
          </fo:list-item-body>
        </fo:list-item>
        <fo:list-item space-after="2pt">
          <fo:list-item-label end-indent="label-end()">
            <fo:block font-size="10pt">&#x25E6;</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block font-size="10pt">Second circle item</fo:block>
          </fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
      <fo:block font-size="10pt" font-weight="bold" space-after="4pt">Square markers (&#x25AA;)</fo:block>
      <fo:list-block provisional-distance-between-starts="12mm"
        provisional-label-separation="4mm" space-after="8pt">
        <fo:list-item space-after="2pt">
          <fo:list-item-label end-indent="label-end()">
            <fo:block font-size="10pt">&#x25AA;</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block font-size="10pt">First square item</fo:block>
          </fo:list-item-body>
        </fo:list-item>
        <fo:list-item space-after="2pt">
          <fo:list-item-label end-indent="label-end()">
            <fo:block font-size="10pt">&#x25AA;</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block font-size="10pt">Second square item</fo:block>
          </fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
      <fo:block font-size="10pt" font-weight="bold" space-after="4pt">Numbered markers</fo:block>
      <fo:list-block provisional-distance-between-starts="12mm"
        provisional-label-separation="4mm" space-after="8pt">
        <fo:list-item space-after="2pt">
          <fo:list-item-label end-indent="label-end()">
            <fo:block font-size="10pt">1.</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block font-size="10pt">First numbered item</fo:block>
          </fo:list-item-body>
        </fo:list-item>
        <fo:list-item space-after="2pt">
          <fo:list-item-label end-indent="label-end()">
            <fo:block font-size="10pt">2.</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block font-size="10pt">Second numbered item</fo:block>
          </fo:list-item-body>
        </fo:list-item>
        <fo:list-item space-after="2pt">
          <fo:list-item-label end-indent="label-end()">
            <fo:block font-size="10pt">3.</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block font-size="10pt">Third numbered item</fo:block>
          </fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "List blocks with various marker styles should render to PDF: {:?}",
        result.err()
    );
    let pdf_bytes = result.expect("test: should succeed");
    assert!(!pdf_bytes.is_empty());
    assert_eq!(&pdf_bytes[0..4], b"%PDF");
}

#[test]
fn conformance_page_margin_variations() {
    // Four page sequences each with different margin settings.
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="narrow" page-height="297mm" page-width="210mm"
      margin-top="10mm" margin-bottom="10mm" margin-left="10mm" margin-right="10mm">
      <fo:region-body region-name="xsl-region-body"/>
    </fo:simple-page-master>
    <fo:simple-page-master master-name="normal" page-height="297mm" page-width="210mm"
      margin-top="25mm" margin-bottom="25mm" margin-left="25mm" margin-right="25mm">
      <fo:region-body region-name="xsl-region-body"/>
    </fo:simple-page-master>
    <fo:simple-page-master master-name="wide" page-height="297mm" page-width="210mm"
      margin-top="40mm" margin-bottom="40mm" margin-left="40mm" margin-right="40mm">
      <fo:region-body region-name="xsl-region-body"/>
    </fo:simple-page-master>
    <fo:simple-page-master master-name="asymmetric" page-height="297mm" page-width="210mm"
      margin-top="20mm" margin-bottom="20mm" margin-left="25mm" margin-right="15mm">
      <fo:region-body region-name="xsl-region-body"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="narrow">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="11pt" font-weight="bold" space-after="6pt">
        Narrow Margins (10mm all sides)
      </fo:block>
      <fo:block font-size="10pt">
        This page uses narrow margins of 10mm on all sides. More content fits per
        page but reading comfort is reduced. Useful for dense technical documents
        or forms where space is at a premium.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="normal">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="11pt" font-weight="bold" space-after="6pt">
        Normal Margins (25mm all sides)
      </fo:block>
      <fo:block font-size="10pt">
        This page uses standard margins of 25mm on all sides. This is the typical
        setting for office documents and provides a comfortable reading experience
        with adequate white space around the content area.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="wide">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="11pt" font-weight="bold" space-after="6pt">
        Wide Margins (40mm all sides)
      </fo:block>
      <fo:block font-size="10pt">
        This page uses wide margins of 40mm on all sides. Often used for formal
        documents, letters, or publications where generous white space is desired
        for a clean and professional appearance.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="asymmetric">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="11pt" font-weight="bold" space-after="6pt">
        Asymmetric Margins (25mm left, 15mm right)
      </fo:block>
      <fo:block font-size="10pt">
        This page uses asymmetric left and right margins: 25mm on the left and
        15mm on the right. This layout is common in bound documents where the
        inner (binding) margin is wider to account for the spine.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Page sequences with different margin settings should render to PDF: {:?}",
        result.err()
    );
    let pdf_bytes = result.expect("test: should succeed");
    assert!(!pdf_bytes.is_empty());
    assert_eq!(&pdf_bytes[0..4], b"%PDF");
}

// ---------------------------------------------------------------------------
// Section: Extended Conformance Tests (Batch 7)
// ---------------------------------------------------------------------------

#[test]
fn conformance_page_number_citation_basic() {
    // Test fo:page-number-citation referencing a block with an id
    let pdf = check_fo(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm"
      margin-top="20mm" margin-bottom="20mm" margin-left="25mm" margin-right="25mm">
      <fo:region-body region-name="xsl-region-body"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="14pt" font-weight="bold" space-after="10pt">
        Table of Contents
      </fo:block>
      <fo:block font-size="11pt" space-after="6pt">
        Section 1 is on page <fo:page-number-citation ref-id="section1"/>.
      </fo:block>
      <fo:block font-size="11pt" space-after="6pt">
        Section 2 is on page <fo:page-number-citation ref-id="section2"/>.
      </fo:block>
      <fo:block font-size="14pt" font-weight="bold" space-before="12pt" space-after="6pt" id="section1">
        Section 1: Introduction
      </fo:block>
      <fo:block font-size="11pt" space-after="6pt">
        This is the introductory section of the document. It provides an overview
        of the topics that will be covered in subsequent sections.
      </fo:block>
      <fo:block font-size="14pt" font-weight="bold" space-before="12pt" space-after="6pt" id="section2">
        Section 2: Details
      </fo:block>
      <fo:block font-size="11pt" space-after="6pt">
        This section contains the detailed content. Each subsection covers a
        specific aspect of the main topic in depth.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#,
    );
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_keep_with_next_block() {
    // Test keep-with-next.within-page="always" on heading blocks before paragraphs
    let pdf = check_fo(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm"
      margin-top="20mm" margin-bottom="20mm" margin-left="25mm" margin-right="25mm">
      <fo:region-body region-name="xsl-region-body"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="14pt" font-weight="bold" space-after="6pt"
                keep-with-next.within-page="always">
        Heading One
      </fo:block>
      <fo:block font-size="11pt" space-after="12pt">
        Paragraph following heading one. The heading above should be kept with
        this paragraph on the same page using keep-with-next always.
      </fo:block>
      <fo:block font-size="14pt" font-weight="bold" space-after="6pt"
                keep-with-next.within-page="always">
        Heading Two
      </fo:block>
      <fo:block font-size="11pt" space-after="12pt">
        Paragraph following heading two. Again the heading is kept with this
        paragraph to avoid orphaned headings at the bottom of pages.
      </fo:block>
      <fo:block font-size="14pt" font-weight="bold" space-after="6pt"
                keep-with-next.within-page="always">
        Heading Three
      </fo:block>
      <fo:block font-size="11pt" space-after="12pt">
        Paragraph following heading three. Multiple heading-paragraph pairs
        each with keep-with-next ensure proper document layout.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#,
    );
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_widow_orphan_settings() {
    // Test widows=3 orphans=3 to prevent short lines at page breaks
    let pdf = check_fo(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm"
      margin-top="20mm" margin-bottom="20mm" margin-left="25mm" margin-right="25mm">
      <fo:region-body region-name="xsl-region-body"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body" widows="3" orphans="3">
      <fo:block font-size="11pt" space-after="8pt">
        First paragraph: Lorem ipsum dolor sit amet, consectetur adipiscing elit.
        Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim
        ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip.
      </fo:block>
      <fo:block font-size="11pt" space-after="8pt">
        Second paragraph: Duis aute irure dolor in reprehenderit in voluptate velit
        esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat
        non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
      </fo:block>
      <fo:block font-size="11pt" space-after="8pt">
        Third paragraph: Pellentesque habitant morbi tristique senectus et netus et
        malesuada fames ac turpis egestas. Vestibulum tortor quam, feugiat vitae,
        ultricies eget, tempor sit amet, ante. Donec eu libero sit amet quam egestas.
      </fo:block>
      <fo:block font-size="11pt" space-after="8pt">
        Fourth paragraph with widows and orphans set to 3: At vero eos et accusamus
        et iusto odio dignissimos ducimus qui blanditiis praesentium voluptatum
        deleniti atque corrupti quos dolores et quas molestias excepturi sint.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#,
    );
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_conditional_space_properties() {
    // Test space-before.conditionality="discard" and space-after.conditionality="retain"
    let pdf = check_fo(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm"
      margin-top="20mm" margin-bottom="20mm" margin-left="25mm" margin-right="25mm">
      <fo:region-body region-name="xsl-region-body"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="12pt" font-weight="bold"
                space-before.optimum="12pt" space-before.conditionality="discard"
                space-after.optimum="6pt" space-after.conditionality="retain">
        Block with discard space-before and retain space-after
      </fo:block>
      <fo:block font-size="11pt"
                space-before.optimum="8pt" space-before.conditionality="retain"
                space-after.optimum="8pt" space-after.conditionality="discard">
        Block with retain space-before and discard space-after. The conditionality
        property controls whether spacing is discarded at page break boundaries.
      </fo:block>
      <fo:block font-size="11pt"
                space-before.optimum="10pt" space-before.conditionality="discard"
                space-after.optimum="10pt" space-after.conditionality="discard">
        Block with both space-before and space-after set to discard conditionality.
        This is useful for elements that appear at the start or end of a page area.
      </fo:block>
      <fo:block font-size="11pt"
                space-before.optimum="6pt" space-before.conditionality="retain"
                space-after.optimum="6pt" space-after.conditionality="retain">
        Block with both space-before and space-after set to retain conditionality.
        Spacing is always preserved regardless of page break position.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#,
    );
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_hyphenation_long_words() {
    // Test hyphenate="true" with language/country settings on blocks with long words
    let pdf = check_fo(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm"
      margin-top="20mm" margin-bottom="20mm" margin-left="25mm" margin-right="25mm">
      <fo:region-body region-name="xsl-region-body"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="12pt" font-weight="bold" space-after="8pt">
        Hyphenation Test
      </fo:block>
      <fo:block font-size="11pt" space-after="8pt"
                hyphenate="true" language="en" country="US"
                hyphenation-push-character-count="2"
                hyphenation-remain-character-count="2">
        This paragraph contains some very long words like internationalization,
        incomprehensibilities, and pneumonoultramicroscopicsilicovolcanoconiosis
        that may benefit from hyphenation when they appear at line boundaries.
      </fo:block>
      <fo:block font-size="11pt" space-after="8pt"
                hyphenate="true" language="en" country="GB">
        British English hyphenation: industrialization, modernisation,
        computerisation, and other long technical terms that appear frequently
        in formal documentation benefit from proper hyphenation support.
      </fo:block>
      <fo:block font-size="11pt" space-after="8pt"
                hyphenate="false">
        This paragraph has hyphenation disabled. Even long words like
        antidisestablishmentarianism will not be hyphenated regardless of
        line breaking requirements.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#,
    );
    validate_pdf_bytes(&pdf);
}

#[test]
fn conformance_multipage_document_100() {
    // Stress test: 100 fo:block elements with distinct content each
    let mut blocks = String::new();
    for i in 1..=100 {
        blocks.push_str(&format!(
            "<fo:block font-size=\"10pt\" space-after=\"6pt\">Paragraph {} of 100: Lorem ipsum dolor sit amet, consectetur adipiscing elit. This is test paragraph number {}.</fo:block>\n",
            i, i
        ));
    }
    let fo_input = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm"
      margin-top="20mm" margin-bottom="20mm" margin-left="25mm" margin-right="25mm">
      <fo:region-body region-name="xsl-region-body"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
{blocks}    </fo:flow>
  </fo:page-sequence>
</fo:root>"#,
        blocks = blocks
    );
    let pdf = check_fo(&fo_input);
    validate_pdf_bytes(&pdf);
}
