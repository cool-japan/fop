//! XSL-FO 1.1 Conformance: Real-world document patterns (invoice, letter, newsletter, etc.)
//!
//! Part of the XSL-FO 1.1 conformance test suite.
//! Reference: https://www.w3.org/TR/xsl11/

use super::process_fo_document;

// ---------------------------------------------------------------------------
// Real-world document conformance tests
// ---------------------------------------------------------------------------

#[test]
fn conformance_invoice_document() {
    // Complete invoice document pattern (real-world use case)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="invoice"
      page-width="210mm" page-height="297mm"
      margin-top="15mm" margin-bottom="20mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body margin-top="35mm" margin-bottom="30mm"/>
      <fo:region-before extent="35mm"/>
      <fo:region-after extent="30mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="invoice">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block-container absolute-position="absolute" top="0mm" left="0mm" width="170mm">
        <fo:block font-size="24pt" font-weight="bold" color="#003366">INVOICE</fo:block>
        <fo:block font-size="10pt" color="#666666">Invoice #: INV-2024-001</fo:block>
        <fo:block font-size="10pt" color="#666666">Date: 2024-01-15</fo:block>
      </fo:block-container>
    </fo:static-content>
    <fo:static-content flow-name="xsl-region-after">
      <fo:block font-size="8pt" color="#999999" text-align="center">
        Page <fo:page-number/> of <fo:page-number-citation ref-id="last"/>
      </fo:block>
      <fo:block font-size="8pt" color="#999999" text-align="center">
        Company Name | 123 Business St | Tel: 555-1234
      </fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="11pt" font-weight="bold" space-after="2mm">Bill To:</fo:block>
      <fo:block font-size="10pt">Customer Company Ltd.</fo:block>
      <fo:block font-size="10pt">456 Customer Ave</fo:block>
      <fo:block font-size="10pt" space-after="8mm">customer@example.com</fo:block>
      <fo:table table-layout="fixed" width="170mm" border-collapse="collapse" space-after="5mm">
        <fo:table-column column-width="70mm"/>
        <fo:table-column column-width="20mm"/>
        <fo:table-column column-width="40mm"/>
        <fo:table-column column-width="40mm"/>
        <fo:table-header>
          <fo:table-row background-color="#003366">
            <fo:table-cell padding="2mm" border="1pt solid #003366">
              <fo:block color="white" font-weight="bold" font-size="9pt">Description</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #003366">
              <fo:block color="white" font-weight="bold" font-size="9pt">Qty</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #003366">
              <fo:block color="white" font-weight="bold" font-size="9pt">Unit Price</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #003366">
              <fo:block color="white" font-weight="bold" font-size="9pt">Total</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-header>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell padding="2mm" border="1pt solid #cccccc">
              <fo:block font-size="9pt">Web Development Services</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #cccccc">
              <fo:block font-size="9pt" text-align="center">10</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #cccccc">
              <fo:block font-size="9pt" text-align="right">$150.00</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #cccccc">
              <fo:block font-size="9pt" text-align="right">$1,500.00</fo:block>
            </fo:table-cell>
          </fo:table-row>
          <fo:table-row background-color="#f9f9f9">
            <fo:table-cell padding="2mm" border="1pt solid #cccccc">
              <fo:block font-size="9pt">Consulting Hours</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #cccccc">
              <fo:block font-size="9pt" text-align="center">5</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #cccccc">
              <fo:block font-size="9pt" text-align="right">$200.00</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #cccccc">
              <fo:block font-size="9pt" text-align="right">$1,000.00</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-body>
        <fo:table-footer>
          <fo:table-row>
            <fo:table-cell number-columns-spanned="3" padding="2mm" border="1pt solid #cccccc">
              <fo:block font-size="9pt" font-weight="bold" text-align="right">Total:</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid #003366">
              <fo:block font-size="9pt" font-weight="bold" text-align="right">$2,500.00</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-footer>
      </fo:table>
      <fo:block font-size="9pt" font-style="italic" space-before="5mm">
        Payment due within 30 days. Thank you for your business.
      </fo:block>
      <fo:block id="last"/>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Invoice document should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_academic_paper_format() {
    // Academic paper format with abstract and sections
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="paper"
      page-width="210mm" page-height="297mm"
      margin-top="25mm" margin-bottom="25mm"
      margin-left="25mm" margin-right="25mm">
      <fo:region-body margin-top="10mm" margin-bottom="10mm"/>
      <fo:region-before extent="10mm"/>
      <fo:region-after extent="10mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="paper">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block font-size="9pt" text-align="center" font-style="italic">
        Journal of Rust Computing, Vol. 1, 2024
      </fo:block>
    </fo:static-content>
    <fo:static-content flow-name="xsl-region-after">
      <fo:block font-size="9pt" text-align="center">
        <fo:page-number/>
      </fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="16pt" font-weight="bold" text-align="center" space-after="4pt">
        Efficient XSL-FO Processing in Rust
      </fo:block>
      <fo:block font-size="11pt" text-align="center" space-after="2pt">
        Author A, Author B
      </fo:block>
      <fo:block font-size="10pt" text-align="center" font-style="italic" space-after="10pt">
        Department of Computer Science
      </fo:block>
      <fo:block-container border="1pt solid #cccccc" padding="5mm" space-after="10pt"
        background-color="#f9f9f9">
        <fo:block font-weight="bold" font-size="10pt" space-after="4pt">Abstract</fo:block>
        <fo:block font-size="9pt" text-align="justify">
          This paper presents a high-performance implementation of XSL-FO document processing
          using the Rust programming language. Our approach achieves significant performance
          improvements over existing Java-based implementations while maintaining full
          specification compliance.
        </fo:block>
      </fo:block-container>
      <fo:block font-size="12pt" font-weight="bold" space-after="4pt" space-before="8pt">
        1. Introduction
      </fo:block>
      <fo:block font-size="10pt" text-align="justify" space-after="6pt">
        Document formatting remains a critical challenge in enterprise software development.
        The XSL-FO specification provides a rich vocabulary for precise document layout,
        but existing implementations have limitations in performance and memory usage.
        <fo:footnote>
          <fo:inline baseline-shift="super" font-size="7pt">1</fo:inline>
          <fo:footnote-body>
            <fo:block font-size="8pt">1. See W3C XSL-FO 1.1 specification for full details.</fo:block>
          </fo:footnote-body>
        </fo:footnote>
      </fo:block>
      <fo:block font-size="12pt" font-weight="bold" space-after="4pt" space-before="8pt">
        2. Implementation
      </fo:block>
      <fo:block font-size="10pt" text-align="justify">
        Our implementation leverages Rust's ownership system to provide memory-safe processing
        without garbage collection overhead. The arena allocator pattern enables efficient
        tree construction.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Academic paper format should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_letter_format() {
    // Business letter format (real-world use case)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="letter"
      page-width="210mm" page-height="297mm"
      margin-top="25mm" margin-bottom="25mm"
      margin-left="25mm" margin-right="25mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="letter">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="12pt" font-weight="bold" space-after="2mm">ACME Corporation</fo:block>
      <fo:block font-size="10pt">123 Business Street</fo:block>
      <fo:block font-size="10pt">Cityville, ST 12345</fo:block>
      <fo:block font-size="10pt" space-after="8mm">Tel: (555) 123-4567</fo:block>
      <fo:block font-size="10pt" space-after="8mm">January 15, 2024</fo:block>
      <fo:block font-size="10pt" font-weight="bold">John Smith</fo:block>
      <fo:block font-size="10pt">456 Customer Road</fo:block>
      <fo:block font-size="10pt" space-after="8mm">Townsburg, ST 67890</fo:block>
      <fo:block font-size="10pt" space-after="6mm">Dear Mr. Smith,</fo:block>
      <fo:block font-size="10pt" text-align="justify" space-after="6mm">
        I am writing to inform you about our new services that may be of interest to you and
        your organization. We have recently expanded our offerings to include comprehensive
        document processing solutions that can significantly improve your workflow efficiency.
      </fo:block>
      <fo:block font-size="10pt" text-align="justify" space-after="6mm">
        Our flagship product, the FOP Rust Processor, delivers exceptional performance and
        reliability for all your XSL-FO document generation needs. We would welcome the
        opportunity to discuss how we can assist your organization.
      </fo:block>
      <fo:block font-size="10pt" space-after="2mm">Sincerely yours,</fo:block>
      <fo:block font-size="10pt" space-before="10mm">Jane Doe</fo:block>
      <fo:block font-size="10pt">Sales Director</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Letter format should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_technical_manual_format() {
    // Technical manual with code-like blocks and callouts
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="manual"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="25mm" margin-right="25mm">
      <fo:region-body margin-top="10mm"/>
      <fo:region-before extent="10mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="manual">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block font-size="9pt" font-weight="bold">FOP Rust User Guide</fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="18pt" font-weight="bold" space-after="6pt">Chapter 3: Configuration</fo:block>
      <fo:block font-size="10pt" space-after="6pt">
        This chapter describes the configuration options available for FOP Rust.
      </fo:block>
      <fo:block font-size="13pt" font-weight="bold" space-after="4pt" keep-with-next="always">
        3.1 Basic Configuration
      </fo:block>
      <fo:block font-size="10pt" space-after="6pt">
        Configuration is specified using a TOML file. Here is a minimal example:
      </fo:block>
      <fo:block-container background-color="#f4f4f4" border="1pt solid #cccccc"
        padding="4mm" space-after="8pt" font-family="monospace">
        <fo:block font-size="9pt">[fop]</fo:block>
        <fo:block font-size="9pt">font_dir = "/usr/share/fonts"</fo:block>
        <fo:block font-size="9pt">default_font = "DejaVu Serif"</fo:block>
        <fo:block font-size="9pt">embed_fonts = true</fo:block>
      </fo:block-container>
      <fo:block-container background-color="#fff3cd" border="1pt solid #ffc107"
        padding="4mm" space-after="8pt">
        <fo:block font-size="10pt" font-weight="bold">Note</fo:block>
        <fo:block font-size="9pt">
          The font_dir path must contain valid TrueType or OpenType fonts.
          Invalid font files are silently skipped.
        </fo:block>
      </fo:block-container>
      <fo:block font-size="13pt" font-weight="bold" space-after="4pt" keep-with-next="always">
        3.2 Advanced Options
      </fo:block>
      <fo:block font-size="10pt" space-after="4pt">
        Additional configuration options:
      </fo:block>
      <fo:list-block provisional-distance-between-starts="50mm" space-after="8pt">
        <fo:list-item space-after="4pt">
          <fo:list-item-label end-indent="label-end()">
            <fo:block font-family="monospace" font-size="9pt">compress_pdf</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block font-size="9pt">Enable FlateDecode compression (default: true)</fo:block>
          </fo:list-item-body>
        </fo:list-item>
        <fo:list-item space-after="4pt">
          <fo:list-item-label end-indent="label-end()">
            <fo:block font-family="monospace" font-size="9pt">pdfa_mode</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block font-size="9pt">Output PDF/A-1b compliant documents (default: false)</fo:block>
          </fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Technical manual format should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_newsletter_multicolumn() {
    // Newsletter format with multi-column layout (real-world use case)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="newsletter"
      page-width="210mm" page-height="297mm"
      margin-top="15mm" margin-bottom="15mm"
      margin-left="15mm" margin-right="15mm">
      <fo:region-body column-count="2" column-gap="6mm" margin-top="20mm"/>
      <fo:region-before extent="20mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="newsletter">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block font-size="20pt" font-weight="bold" text-align="center"
        background-color="#003366" color="white" padding="4mm">
        THE RUST GAZETTE
      </fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block span="all" font-size="16pt" font-weight="bold" space-after="4pt"
        border-bottom="2pt solid #003366" padding-bottom="2pt">
        Main Story: Rust 2024 Edition Released
      </fo:block>
      <fo:block font-size="9pt" text-align="justify" space-after="4pt">
        The Rust programming language team has announced the release of the 2024 edition,
        bringing significant improvements to the language and ecosystem. This marks a major
        milestone in Rust's development journey.
      </fo:block>
      <fo:block font-size="9pt" text-align="justify" space-after="4pt">
        Key improvements include enhanced async/await syntax, better ergonomics for common
        patterns, and improved error messages that help developers diagnose issues faster.
      </fo:block>
      <fo:block font-size="11pt" font-weight="bold" space-after="3pt" space-before="6pt">
        Community Highlights
      </fo:block>
      <fo:block font-size="9pt" text-align="justify" space-after="4pt">
        The Rust community continues to grow rapidly with thousands of new contributors
        joining each month. The crates.io registry now hosts over 150,000 packages.
      </fo:block>
      <fo:block font-size="11pt" font-weight="bold" space-after="3pt" space-before="6pt">
        Upcoming Events
      </fo:block>
      <fo:list-block provisional-distance-between-starts="10mm">
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()">
            <fo:block font-size="8pt">&#x2022;</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block font-size="8pt">RustConf 2024 - March 15</fo:block>
          </fo:list-item-body>
        </fo:list-item>
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()">
            <fo:block font-size="8pt">&#x2022;</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block font-size="8pt">Rust Workshop - April 2</fo:block>
          </fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Newsletter format should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_empty_blocks_handling() {
    // Empty fo:block elements should not cause errors (Section 6.4.2)
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
      <fo:block/>
      <fo:block>After empty block</fo:block>
      <fo:block/>
      <fo:block/>
      <fo:block>After two empty blocks</fo:block>
      <fo:block space-before="5mm" space-after="5mm"/>
      <fo:block>After spaced empty block</fo:block>
      <fo:block border="1pt solid black"/>
      <fo:block>After bordered empty block</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Empty blocks should not cause errors: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_long_unbreakable_text() {
    // Very long word without spaces (should overflow or wrap) (Section 7.15)
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
      <fo:block overflow="hidden">
        ThisIsAVeryLongWordThatDoesNotContainAnySpacesAndShouldBeHandledGracefully
      </fo:block>
      <fo:block>Normal text follows the long word.</fo:block>
      <fo:block overflow="hidden">
        Short words are fine but
        AnExtremelyLongIdentifierNameInCamelCaseThatExceedsTheColumnWidth123456
        should be handled.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Long unbreakable text should not crash: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_unicode_special_characters() {
    // Unicode special characters including zero-width spaces (Section 6.6.4)
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
      <fo:block>Basic ASCII: Hello, World!</fo:block>
      <fo:block>Em-dash: before—after</fo:block>
      <fo:block>En-dash: 2020–2024</fo:block>
      <fo:block>Ellipsis: text…more text</fo:block>
      <fo:block>Quotes: &#x201C;curly quotes&#x201D; and &#x2018;single quotes&#x2019;</fo:block>
      <fo:block>Math: &#x03B1; + &#x03B2; = &#x03B3;, &#x2211; (sigma), &#x03C0; &#x2248; 3.14159</fo:block>
      <fo:block>Arrows: &#x2190; &#x2192; &#x2191; &#x2193; &#x27F9; &#x27FA;</fo:block>
      <fo:block>Box drawing: &#x250C;&#x2500;&#x2510; &#x2502; &#x2502; &#x2514;&#x2500;&#x2518;</fo:block>
      <fo:block>Symbols: &#x2713; &#x2717; &#x2605; &#x2606; &#x2666;</fo:block>
      <fo:block xml:lang="ja">Japanese: Tokyo, Osaka, Nagoya</fo:block>
      <fo:block xml:lang="zh">Chinese: Beijing, Shanghai, Guangzhou</fo:block>
      <fo:block xml:lang="ko">Korean: Seoul, Busan, Incheon</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Unicode special chars should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_deep_nesting() {
    // Deeply nested elements should not overflow the stack (Section 6.4)
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
        <fo:inline>
          <fo:inline>
            <fo:inline>
              <fo:inline>
                <fo:inline font-weight="bold">
                  <fo:inline font-style="italic">
                    Deeply nested inline content
                  </fo:inline>
                </fo:inline>
              </fo:inline>
            </fo:inline>
          </fo:inline>
        </fo:inline>
      </fo:block>
      <fo:block-container>
        <fo:block-container>
          <fo:block>Nested block-containers</fo:block>
        </fo:block-container>
      </fo:block-container>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Deep nesting should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_percentage_all_properties() {
    // Percentage values for various properties (Section 5.11.2)
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
      <fo:block font-size="12pt" line-height="150%" space-after="4pt">
        Line height 150% of font size
      </fo:block>
      <fo:block font-size="10pt" line-height="120%" space-after="4pt">
        Tighter 120% line height for this paragraph with some text content.
      </fo:block>
      <fo:table table-layout="fixed" width="100%">
        <fo:table-column column-width="proportional-column-width(1)"/>
        <fo:table-column column-width="proportional-column-width(1)"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block>50% width column</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid black">
              <fo:block>50% width column</fo:block>
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
        "Percentage dimensions should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_extreme_property_values() {
    // Extreme property values that should be handled gracefully (Section 5.11)
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
      <fo:block font-size="6pt">Very small 6pt text</fo:block>
      <fo:block font-size="8pt" space-after="2pt">Small 8pt text</fo:block>
      <fo:block font-size="36pt" space-after="4pt">Large 36pt</fo:block>
      <fo:block font-size="48pt">Very large 48pt</fo:block>
      <fo:block font-size="10pt" letter-spacing="0.5pt">0.5pt letter spacing</fo:block>
      <fo:block font-size="10pt" line-height="6pt" space-after="8pt">Very tight 6pt line height</fo:block>
      <fo:block font-size="10pt" line-height="40pt">Very loose 40pt line height</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Extreme property values should not crash: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}
