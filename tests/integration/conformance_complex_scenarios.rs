//! XSL-FO 1.1 Conformance: Complex page layout scenarios and text output format tests
//!
//! Part of the XSL-FO 1.1 conformance test suite.
//! Reference: https://www.w3.org/TR/xsl11/

use super::process_fo_document;

// ---------------------------------------------------------------------------
// Section 7.25/7.7/7.10/7.27/6.6: Complex scenarios
// ---------------------------------------------------------------------------

#[test]
fn conformance_conditional_first_last_pages() {
    // Different layouts for first, last, and middle pages (Section 7.25.10)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="cover-page"
      page-width="210mm" page-height="297mm"
      margin-top="40mm" margin-bottom="40mm"
      margin-left="30mm" margin-right="30mm">
      <fo:region-body/>
    </fo:simple-page-master>
    <fo:simple-page-master master-name="first-body"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="25mm" margin-right="25mm">
      <fo:region-body margin-top="0mm" margin-bottom="12mm"/>
      <fo:region-after extent="12mm"/>
    </fo:simple-page-master>
    <fo:simple-page-master master-name="odd-body"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="25mm" margin-right="25mm">
      <fo:region-body margin-top="10mm" margin-bottom="12mm"/>
      <fo:region-before extent="10mm"/>
      <fo:region-after extent="12mm"/>
    </fo:simple-page-master>
    <fo:simple-page-master master-name="even-body"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="25mm" margin-right="25mm">
      <fo:region-body margin-top="10mm" margin-bottom="12mm"/>
      <fo:region-before extent="10mm"/>
      <fo:region-after extent="12mm"/>
    </fo:simple-page-master>
    <fo:page-sequence-master master-name="body-sequence">
      <fo:repeatable-page-master-alternatives>
        <fo:conditional-page-master-reference master-reference="first-body" page-position="first"/>
        <fo:conditional-page-master-reference master-reference="even-body" odd-or-even="even"/>
        <fo:conditional-page-master-reference master-reference="odd-body" odd-or-even="odd"/>
      </fo:repeatable-page-master-alternatives>
    </fo:page-sequence-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="cover-page">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="24pt" font-weight="bold" text-align="center" space-before="30mm">
        Document Cover Page
      </fo:block>
      <fo:block text-align="center" space-before="10mm">Subtitle Here</fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="body-sequence">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block font-size="9pt" text-align="center">Chapter Content</fo:block>
    </fo:static-content>
    <fo:static-content flow-name="xsl-region-after">
      <fo:block font-size="9pt" text-align="center">Page <fo:page-number/></fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="14pt" font-weight="bold">Chapter 1</fo:block>
      <fo:block space-before="6pt">First page content (no header).</fo:block>
      <fo:block break-before="page" font-size="14pt" font-weight="bold">Chapter 2</fo:block>
      <fo:block space-before="6pt">Even page content (header shows).</fo:block>
      <fo:block break-before="page">Odd page content.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Conditional first/last pages should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_nested_block_containers_positioned() {
    // Nested block-containers with absolute positioning (Section 7.7.6)
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
      <fo:block-container position="relative" height="60mm"
        border="1pt solid #cccccc" background-color="#f9f9f9">
        <fo:block-container absolute-position="absolute" top="5mm" left="5mm"
          width="70mm" height="20mm" background-color="#dee2f0" padding="2mm">
          <fo:block font-size="9pt" font-weight="bold">Positioned Box 1</fo:block>
          <fo:block font-size="8pt">Top-left position</fo:block>
        </fo:block-container>
        <fo:block-container absolute-position="absolute" top="5mm" right="5mm"
          width="70mm" height="20mm" background-color="#d0f0de" padding="2mm">
          <fo:block font-size="9pt" font-weight="bold">Positioned Box 2</fo:block>
          <fo:block font-size="8pt">Top-right position</fo:block>
        </fo:block-container>
        <fo:block-container absolute-position="absolute" top="30mm" left="5mm"
          width="160mm" height="20mm" background-color="#fff3cd" padding="2mm">
          <fo:block font-size="9pt">Bottom spanning box with absolute positioning</fo:block>
        </fo:block-container>
      </fo:block-container>
      <fo:block space-before="5mm">Normal content continues after positioned boxes.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Nested positioned block-containers should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_document_with_toc() {
    // Complete document with table of contents using fo:basic-link (Section 7.10)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="toc-page"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
    <fo:simple-page-master master-name="body-page"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body margin-bottom="12mm"/>
      <fo:region-after extent="12mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="toc-page" format="i" initial-page-number="1">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="16pt" font-weight="bold" space-after="12pt">Table of Contents</fo:block>
      <fo:block text-align-last="justify" space-after="4pt">
        <fo:basic-link internal-destination="chapter1" color="blue">
          1. Introduction
        </fo:basic-link>
        <fo:leader leader-pattern="dots"/>
        <fo:page-number-citation ref-id="chapter1"/>
      </fo:block>
      <fo:block text-align-last="justify" space-after="4pt">
        <fo:basic-link internal-destination="chapter2" color="blue">
          2. Methods
        </fo:basic-link>
        <fo:leader leader-pattern="dots"/>
        <fo:page-number-citation ref-id="chapter2"/>
      </fo:block>
      <fo:block text-align-last="justify">
        <fo:basic-link internal-destination="chapter3" color="blue">
          3. Conclusion
        </fo:basic-link>
        <fo:leader leader-pattern="dots"/>
        <fo:page-number-citation ref-id="chapter3"/>
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="body-page" format="1" initial-page-number="1">
    <fo:static-content flow-name="xsl-region-after">
      <fo:block text-align="center" font-size="9pt">— <fo:page-number/> —</fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block id="chapter1" font-size="16pt" font-weight="bold" space-after="6pt">
        1. Introduction
      </fo:block>
      <fo:block>Introduction content goes here with some body text.</fo:block>
      <fo:block id="chapter2" break-before="page" font-size="16pt" font-weight="bold" space-after="6pt">
        2. Methods
      </fo:block>
      <fo:block>Methods chapter content with description of approaches.</fo:block>
      <fo:block id="chapter3" break-before="page" font-size="16pt" font-weight="bold" space-after="6pt">
        3. Conclusion
      </fo:block>
      <fo:block>Final conclusions and recommendations.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Document with TOC should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_multilang_font_fallback() {
    // Multi-language document with various scripts (Section 7.27)
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
      <fo:block font-size="14pt" font-weight="bold" space-after="6pt">Multilingual Content</fo:block>
      <fo:block xml:lang="en" space-after="4pt">English: The quick brown fox</fo:block>
      <fo:block xml:lang="de" space-after="4pt">German: Ünterführung, Übung, Äpfel</fo:block>
      <fo:block xml:lang="fr" space-after="4pt">French: C'est très beau, élégant</fo:block>
      <fo:block xml:lang="es" space-after="4pt">Spanish: ¿Cómo estás? Bien, gracias</fo:block>
      <fo:block xml:lang="pt" space-after="4pt">Portuguese: Não é possível, não</fo:block>
      <fo:block xml:lang="ja" space-after="4pt">Japanese: 日本語テキスト、漢字を含む</fo:block>
      <fo:block xml:lang="zh" space-after="4pt">Chinese: 中文文字内容测试</fo:block>
      <fo:block xml:lang="ko" space-after="4pt">Korean: 한국어 텍스트 내용</fo:block>
      <fo:block xml:lang="ar" space-after="4pt">Arabic: مرحبا بالعالم العربي</fo:block>
      <fo:block xml:lang="ru" space-after="4pt">Russian: Привет мир, кириллица</fo:block>
      <fo:block xml:lang="el" space-after="4pt">Greek: Καλημέρα, αλφάβητο</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Multi-language document should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_inline_mixed_content() {
    // Complex inline content mixing text, images, and formatting (Section 6.6)
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
      <fo:block line-height="18pt">
        Regular text with
        <fo:inline font-weight="bold">bold</fo:inline>,
        <fo:inline font-style="italic">italic</fo:inline>,
        <fo:inline font-weight="bold" font-style="italic">bold italic</fo:inline>,
        <fo:inline text-decoration="underline">underline</fo:inline>,
        <fo:inline text-decoration="line-through">strikethrough</fo:inline>,
        <fo:inline color="red">red text</fo:inline>,
        <fo:inline color="#003366" font-weight="bold">dark blue bold</fo:inline>,
        superscript<fo:inline baseline-shift="super" font-size="8pt">sup</fo:inline>,
        subscript<fo:inline baseline-shift="sub" font-size="8pt">sub</fo:inline>,
        and normal text again.
      </fo:block>
      <fo:block space-before="6pt" line-height="20pt">
        Code-like content:
        <fo:inline font-family="monospace" background-color="#f4f4f4"
          border="0.5pt solid #cccccc" padding="1pt">let x = 42;</fo:inline>
        and
        <fo:inline font-family="monospace" background-color="#f4f4f4">println!("{}", x)</fo:inline>
        are Rust expressions.
      </fo:block>
      <fo:block space-before="6pt">
        Link:
        <fo:basic-link external-destination="https://www.rust-lang.org"
          color="blue" text-decoration="underline">
          The Rust Programming Language
        </fo:basic-link>
        — a systems language.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Mixed inline content should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_complex_page_number_scenarios() {
    // Various page number display scenarios in headers/footers
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="25mm" margin-bottom="25mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body margin-top="12mm" margin-bottom="12mm"/>
      <fo:region-before extent="12mm"/>
      <fo:region-after extent="12mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4" format="i" initial-page-number="1">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block text-align="center" font-size="9pt" font-style="italic">
        Preface — Page <fo:page-number/>
      </fo:block>
    </fo:static-content>
    <fo:static-content flow-name="xsl-region-after">
      <fo:block text-align="center" font-size="9pt">
        <fo:page-number/>
      </fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="14pt" font-weight="bold">Preface</fo:block>
      <fo:block space-before="6pt">Introduction content with roman page numbers.</fo:block>
      <fo:block break-before="page">Second preface page.</fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="A4" format="1" initial-page-number="1">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block text-align="right" font-size="9pt">
        Page <fo:page-number/> of
        <fo:page-number-citation ref-id="doc-end"/>
      </fo:block>
    </fo:static-content>
    <fo:static-content flow-name="xsl-region-after">
      <fo:block text-align="center" font-size="9pt" font-style="italic">
        The Rust Document Processing Guide
      </fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="16pt" font-weight="bold">Chapter 1</fo:block>
      <fo:block>Body content with decimal page numbers.</fo:block>
      <fo:block break-before="page">Chapter 2 on page 2.</fo:block>
      <fo:block break-before="page">Chapter 3 on page 3.</fo:block>
      <fo:block id="doc-end">End of document.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Complex page number scenarios should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

// ============================================================
// PostScript-specific conformance tests
// ============================================================

#[test]
fn conformance_ps_output_complete() {
    // PostScript output includes proper DSC structure
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
      <fo:block space-before="6pt">This should produce valid PostScript output.</fo:block>
      <fo:block space-before="4pt" border="1pt solid black" padding="3mm">
        Block with border in PostScript
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        "ps",
    );
    assert!(result.is_ok(), "PS output should work: {:?}", result.err());
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty(), "PS output should not be empty");

    let ps_str = String::from_utf8_lossy(&bytes);
    assert!(
        ps_str.starts_with("%!PS-Adobe"),
        "PS should start with %!PS-Adobe header, got: {}",
        &ps_str[..ps_str.len().min(80)]
    );
    assert!(
        ps_str.contains("%%BoundingBox:"),
        "PS should contain %%BoundingBox comment"
    );
    assert!(
        ps_str.contains("%%Pages:"),
        "PS should contain %%Pages comment"
    );
    assert!(
        ps_str.contains("%%BeginProlog"),
        "PS should contain %%BeginProlog section"
    );
    assert!(
        ps_str.contains("%%EndProlog"),
        "PS should contain %%EndProlog"
    );
    assert!(
        ps_str.contains("%%BeginSetup"),
        "PS should contain %%BeginSetup section"
    );
    assert!(
        ps_str.contains("%%Page:"),
        "PS should contain %%Page marker"
    );
    assert!(
        ps_str.contains("%%PageBoundingBox:"),
        "PS should contain %%PageBoundingBox per page"
    );
    assert!(
        ps_str.contains("showpage"),
        "PS should contain showpage command"
    );
    assert!(ps_str.contains("%%EOF"), "PS should end with %%EOF");
}

#[test]
fn conformance_ps_multipage() {
    // PostScript output for multi-page document.
    // Each fo:page-sequence produces one %%Page in the PS output.
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
      <fo:block>Page 1 content for PostScript</fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Page 2 content for PostScript</fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Page 3 content for PostScript</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        "ps",
    );
    assert!(
        result.is_ok(),
        "PS multi-page should work: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(
        !bytes.is_empty(),
        "PS multi-page output should not be empty"
    );

    let ps_str = String::from_utf8_lossy(&bytes);
    // Each fo:page-sequence produces one %%Page marker
    let page_count = ps_str.matches("%%Page:").count();
    assert!(
        page_count >= 3,
        "PS should contain at least 3 %%Page markers for 3 page-sequences, got {}",
        page_count
    );
    // Should have 3 showpage calls
    let showpage_count = ps_str.matches("showpage").count();
    assert!(
        showpage_count >= 3,
        "PS should have at least 3 showpage calls, got {}",
        showpage_count
    );
    assert!(
        ps_str.contains("%%Pages: 3"),
        "PS trailer should report 3 pages"
    );
}

#[test]
fn conformance_ps_with_colors() {
    // PostScript output with colored text and backgrounds
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
      <fo:block color="#cc0000" font-weight="bold">Red heading</fo:block>
      <fo:block color="#0066cc">Blue text</fo:block>
      <fo:block background-color="#ffff99" padding="2mm">Yellow background</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        "ps",
    );
    assert!(
        result.is_ok(),
        "PS with colors should work: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty(), "PS color output should not be empty");

    let ps_str = String::from_utf8_lossy(&bytes);
    // RGB color calls should appear
    assert!(
        ps_str.contains("RGB"),
        "PS output should use RGB color commands"
    );
    assert!(
        ps_str.starts_with("%!PS-Adobe"),
        "PS color output should have proper header"
    );
}

#[test]
fn conformance_ps_table_output() {
    // PostScript output for a document with a table
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
      <fo:block space-after="5mm">Table in PostScript:</fo:block>
      <fo:table table-layout="fixed" width="160mm">
        <fo:table-column column-width="80mm"/>
        <fo:table-column column-width="80mm"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell border="1pt solid black" padding="2mm">
              <fo:block>Cell A</fo:block>
            </fo:table-cell>
            <fo:table-cell border="1pt solid black" padding="2mm">
              <fo:block>Cell B</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell border="1pt solid black" padding="2mm">
              <fo:block>Cell C</fo:block>
            </fo:table-cell>
            <fo:table-cell border="1pt solid black" padding="2mm">
              <fo:block>Cell D</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        "ps",
    );
    assert!(
        result.is_ok(),
        "PS table output should work: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty(), "PS table output should not be empty");

    let ps_str = String::from_utf8_lossy(&bytes);
    assert!(
        ps_str.starts_with("%!PS-Adobe"),
        "PS table output should have proper header"
    );
    // Border rectangles should appear (frect commands)
    assert!(
        ps_str.contains("frect"),
        "PS table output should contain rectangle commands for borders"
    );
}

#[test]
fn conformance_ps_font_encoding() {
    // PostScript output should define font encoding in setup section
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
      <fo:block font-weight="bold">Bold text</fo:block>
      <fo:block font-style="italic">Italic text</fo:block>
      <fo:block font-weight="bold" font-style="italic">Bold italic text</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        "ps",
    );
    assert!(
        result.is_ok(),
        "PS font encoding test should work: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty());

    let ps_str = String::from_utf8_lossy(&bytes);
    // Setup section with font re-encoding
    assert!(
        ps_str.contains("ISOLatin1Encoding"),
        "PS should define ISOLatin1 font encoding in setup"
    );
    assert!(
        ps_str.contains("%%BeginSetup"),
        "PS should have %%BeginSetup section"
    );
    assert!(
        ps_str.contains("%%EndSetup"),
        "PS should have %%EndSetup section"
    );
    // Bold font should be referenced
    assert!(
        ps_str.contains("Helvetica-Bold") || ps_str.contains("Helvetica"),
        "PS should reference Helvetica font family"
    );
}

#[test]
fn conformance_table_large_data() {
    // Large data table with many rows (performance/correctness test)
    let mut rows = String::new();
    for i in 1..=20 {
        rows.push_str(&format!(
            r##"
          <fo:table-row background-color="{}">
            <fo:table-cell border="0.5pt solid #cccccc" padding="1mm">
              <fo:block font-size="8pt">{}</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid #cccccc" padding="1mm">
              <fo:block font-size="8pt">Item Name {}</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid #cccccc" padding="1mm">
              <fo:block font-size="8pt" text-align="right">{}.00</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid #cccccc" padding="1mm">
              <fo:block font-size="8pt" text-align="center">In Stock</fo:block>
            </fo:table-cell>
          </fo:table-row>"##,
            if i % 2 == 0 { "#f8f9fa" } else { "white" },
            i,
            i,
            i * 10
        ));
    }

    let fo_xml = format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="15mm" margin-right="15mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:table table-layout="fixed" width="180mm">
        <fo:table-column column-width="20mm"/>
        <fo:table-column column-width="80mm"/>
        <fo:table-column column-width="40mm"/>
        <fo:table-column column-width="40mm"/>
        <fo:table-header>
          <fo:table-row background-color="#343a40">
            <fo:table-cell padding="2mm" border="1pt solid #343a40">
              <fo:block color="white" font-weight="bold" font-size="9pt">#</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #343a40">
              <fo:block color="white" font-weight="bold" font-size="9pt">Name</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #343a40">
              <fo:block color="white" font-weight="bold" font-size="9pt">Price</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #343a40">
              <fo:block color="white" font-weight="bold" font-size="9pt">Status</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-header>
        <fo:table-body>
          {}
        </fo:table-body>
      </fo:table>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        rows
    );

    let result = process_fo_document(&fo_xml);
    assert!(
        result.is_ok(),
        "Large data table should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_nested_tables_complex() {
    // Nested tables with headers and spanning (Section 8.3)
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
        <fo:table-column column-width="80mm"/>
        <fo:table-column column-width="80mm"/>
        <fo:table-header>
          <fo:table-row>
            <fo:table-cell number-columns-spanned="2" border="2pt solid #003366"
              padding="2mm" background-color="#003366">
              <fo:block color="white" font-weight="bold" text-align="center">
                Outer Table Header
              </fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-header>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell border="1pt solid #cccccc" padding="2mm">
              <fo:block font-weight="bold" space-after="2mm">Left Section</fo:block>
              <fo:table table-layout="fixed" width="70mm" border-collapse="collapse">
                <fo:table-column column-width="35mm"/>
                <fo:table-column column-width="35mm"/>
                <fo:table-body>
                  <fo:table-row>
                    <fo:table-cell border-style="solid" border-width="0.5pt" border-color="gray" padding="1mm">
                      <fo:block font-size="8pt">Inner A1</fo:block>
                    </fo:table-cell>
                    <fo:table-cell border-style="solid" border-width="0.5pt" border-color="gray" padding="1mm">
                      <fo:block font-size="8pt">Inner A2</fo:block>
                    </fo:table-cell>
                  </fo:table-row>
                  <fo:table-row>
                    <fo:table-cell border-style="solid" border-width="0.5pt" border-color="gray" padding="1mm">
                      <fo:block font-size="8pt">Inner B1</fo:block>
                    </fo:table-cell>
                    <fo:table-cell border-style="solid" border-width="0.5pt" border-color="gray" padding="1mm">
                      <fo:block font-size="8pt">Inner B2</fo:block>
                    </fo:table-cell>
                  </fo:table-row>
                </fo:table-body>
              </fo:table>
            </fo:table-cell>
            <fo:table-cell border="1pt solid #cccccc" padding="2mm">
              <fo:block font-weight="bold">Right Section</fo:block>
              <fo:block font-size="9pt">Right column with plain text content spanning multiple lines for proper testing.</fo:block>
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
        "Nested complex tables should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_table_with_page_header_footer() {
    // Table with repeated header and footer across pages (Section 8.3)
    let mut rows = String::new();
    for i in 1..=15 {
        rows.push_str(&format!(
            r##"
          <fo:table-row>
            <fo:table-cell border="0.5pt solid #cccccc" padding="2mm">
              <fo:block font-size="9pt">Row {}</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid #cccccc" padding="2mm">
              <fo:block font-size="9pt">Content for row {} in this table</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid #cccccc" padding="2mm">
              <fo:block font-size="9pt" text-align="right">{}.00</fo:block>
            </fo:table-cell>
          </fo:table-row>"##,
            i,
            i,
            i * 100
        ));
    }

    let fo_xml = format!(
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
        <fo:table-column column-width="100mm"/>
        <fo:table-column column-width="40mm"/>
        <fo:table-header>
          <fo:table-row background-color="#003366">
            <fo:table-cell padding="2mm" border="1pt solid #003366">
              <fo:block color="white" font-weight="bold" font-size="9pt">#</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #003366">
              <fo:block color="white" font-weight="bold" font-size="9pt">Description</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #003366">
              <fo:block color="white" font-weight="bold" font-size="9pt" text-align="right">Amount</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-header>
        <fo:table-footer>
          <fo:table-row>
            <fo:table-cell number-columns-spanned="2" padding="2mm" border="2pt solid #003366">
              <fo:block font-weight="bold" font-size="9pt">Total</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="2pt solid #003366">
              <fo:block font-weight="bold" font-size="9pt" text-align="right">12,000.00</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-footer>
        <fo:table-body>
          {}
        </fo:table-body>
      </fo:table>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        rows
    );

    let result = process_fo_document(&fo_xml);
    assert!(
        result.is_ok(),
        "Table with header/footer should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_inline_mixed_formatting() {
    // Complex inline formatting with nested spans (Section 7.4)
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
        Normal text with
        <fo:inline font-weight="bold">bold</fo:inline>,
        <fo:inline font-style="italic">italic</fo:inline>,
        <fo:inline font-weight="bold" font-style="italic">bold-italic</fo:inline>,
        <fo:inline text-decoration="underline">underlined</fo:inline>,
        <fo:inline color="#cc0000">red</fo:inline>,
        and <fo:inline font-size="8pt">small</fo:inline>
        <fo:inline font-size="14pt">large</fo:inline> text.
      </fo:block>
      <fo:block space-before="6pt">
        Nested: <fo:inline font-weight="bold">Bold with
          <fo:inline font-style="italic">nested italic inside bold</fo:inline>
          and back to just bold</fo:inline> then normal.
      </fo:block>
      <fo:block space-before="6pt">
        Superscript: H<fo:inline baseline-shift="super" font-size="8pt">2</fo:inline>O
        and CO<fo:inline baseline-shift="sub" font-size="8pt">2</fo:inline>
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Mixed inline formatting should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_leader_patterns_advanced() {
    // fo:leader with different patterns in table of contents style (Section 8.6)
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
      <fo:block font-size="14pt" font-weight="bold" space-after="6pt">Table of Contents</fo:block>
      <fo:block text-align-last="justify" space-after="2pt">
        Chapter 1: Introduction
        <fo:leader leader-pattern="dots" leader-length.maximum="100%"/>
        1
      </fo:block>
      <fo:block text-align-last="justify" space-after="2pt">
        Chapter 2: Background
        <fo:leader leader-pattern="dots" leader-length.maximum="100%"/>
        15
      </fo:block>
      <fo:block text-align-last="justify" space-after="2pt">
        Chapter 3: Methodology
        <fo:leader leader-pattern="dots" leader-length.maximum="100%"/>
        42
      </fo:block>
      <fo:block text-align-last="justify" space-after="6pt">
        Chapter 4: Results
        <fo:leader leader-pattern="dots" leader-length.maximum="100%"/>
        78
      </fo:block>
      <fo:block text-align-last="justify">
        <fo:leader leader-pattern="rule" rule-thickness="0.5pt" leader-length.optimum="100%"/>
      </fo:block>
      <fo:block space-before="4pt">Appendix A: Data Tables
        <fo:leader leader-pattern="space" leader-length.minimum="10mm" leader-length.maximum="100%"/>
        95
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Advanced leader patterns should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_inline_box_model() {
    // fo:inline with box model properties (padding, border) (Section 7.4)
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
      <fo:block space-after="4pt">
        Text with
        <fo:inline background-color="#ffff00" padding="1pt 3pt">highlighted inline</fo:inline>
        content.
      </fo:block>
      <fo:block space-after="4pt">
        Text with
        <fo:inline border="1pt solid red" padding="1pt 2pt">bordered inline</fo:inline>
        content.
      </fo:block>
      <fo:block space-after="4pt">
        Text with
        <fo:inline font-family="monospace" background-color="#f0f0f0"
          padding="1pt 4pt" border="0.5pt solid #cccccc">code</fo:inline>
        inline snippet.
      </fo:block>
      <fo:block>
        Button-like:
        <fo:inline background-color="#0066cc" color="white"
          padding="2pt 6pt" font-weight="bold">Click Me</fo:inline>
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Inline box model should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_inline_container_usage() {
    // fo:inline-container for inline block context (Section 7.4)
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
        <fo:inline-container border="1pt solid #cccccc" padding="2pt"
          inline-progression-dimension="30mm">
          <fo:block font-size="8pt" text-align="center">Inline</fo:block>
          <fo:block font-size="8pt" text-align="center">Container</fo:block>
        </fo:inline-container>
        text after inline container.
      </fo:block>
      <fo:block space-before="6pt">
        Another paragraph with
        <fo:inline-container background-color="#e8f4f8" padding="3pt">
          <fo:block font-size="9pt">Side note</fo:block>
        </fo:inline-container>
        embedded in text flow.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Inline container should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_link_and_citation() {
    // fo:basic-link for internal and external links (Section 8.7)
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
      <fo:block space-after="6pt">
        See
        <fo:basic-link internal-destination="section-2" color="#0066cc"
          text-decoration="underline">Section 2</fo:basic-link>
        for more details about external links.
      </fo:block>
      <fo:block space-after="6pt">
        Visit
        <fo:basic-link external-destination="https://www.w3.org/TR/xsl11/"
          color="#0066cc" text-decoration="underline">W3C XSL-FO spec</fo:basic-link>
        for the full specification.
      </fo:block>
      <fo:block id="section-2" font-size="14pt" font-weight="bold" space-before="10pt">
        Section 2: Links
      </fo:block>
      <fo:block space-after="6pt">
        Internal link target with id="section-2".
      </fo:block>
      <fo:block>
        Page reference: see page
        <fo:page-number-citation ref-id="section-2"/>.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Links and citations should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

// ---------------------------------------------------------------------------
// Text output format tests
// ---------------------------------------------------------------------------

#[test]
fn conformance_text_output_content() {
    // Text output contains block text content
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
      <fo:block>Plain text heading</fo:block>
      <fo:block>First paragraph of content</fo:block>
      <fo:block>Second paragraph with more content</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        "text",
    );
    assert!(
        result.is_ok(),
        "Text output should succeed: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty(), "Text output should not be empty");
    let text = String::from_utf8_lossy(&bytes);
    assert!(
        text.contains("Plain text heading") || text.contains("paragraph"),
        "Text output should contain document content, got: {:?}",
        &text[..text.len().min(200)]
    );
}

#[test]
fn conformance_text_output_list() {
    // Text output with list items
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
      <fo:block>Shopping List:</fo:block>
      <fo:list-block provisional-distance-between-starts="10mm">
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()">
            <fo:block>1.</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>Apples</fo:block>
          </fo:list-item-body>
        </fo:list-item>
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()">
            <fo:block>2.</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>Bananas</fo:block>
          </fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        "text",
    );
    assert!(
        result.is_ok(),
        "Text list output should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_text_output_unicode() {
    // Text output preserves Unicode characters
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
      <fo:block>English: Hello World</fo:block>
      <fo:block>German: Äöü Straße</fo:block>
      <fo:block>French: café, naïve, résumé</fo:block>
      <fo:block>Math: α β γ δ ε ∑ ∞ √</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        "text",
    );
    assert!(
        result.is_ok(),
        "Text Unicode output should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}
