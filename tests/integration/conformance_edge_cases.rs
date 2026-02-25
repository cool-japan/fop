//! XSL-FO 1.1 Conformance: Document structure edge cases, stress tests, RTL/CJK text
//!
//! Part of the XSL-FO 1.1 conformance test suite.
//! Reference: https://www.w3.org/TR/xsl11/

use super::process_fo_document;

// ---------------------------------------------------------------------------
// Section: Document structure edge cases
// ---------------------------------------------------------------------------

#[test]
fn conformance_long_text_line_breaking() {
    // Very long text in a single block must be properly broken across lines
    let long_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(10);
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
      <fo:block>{}</fo:block>
      <fo:block font-weight="bold">Short block after long text.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        long_text
    );

    let result = process_fo_document(&fo_xml);
    assert!(result.is_ok(), "Long text should work: {:?}", result.err());
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_empty_blocks() {
    // Empty fo:block elements should be handled gracefully
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
      <fo:block>Content before empty blocks</fo:block>
      <fo:block/>
      <fo:block> </fo:block>
      <fo:block space-before="6pt" space-after="6pt"/>
      <fo:block>Content after empty blocks</fo:block>
      <fo:block border="1pt solid black" height="10mm"/>
      <fo:block>Final content</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Empty blocks should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_deep_nesting_block_container() {
    // Deeply nested block-container and block elements (10+ levels)
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
        <fo:wrapper>
          <fo:block>
            <fo:wrapper>
              <fo:block>
                <fo:wrapper>
                  <fo:block>
                    <fo:wrapper>
                      <fo:block>
                        Deeply nested content at level 5
                      </fo:block>
                    </fo:wrapper>
                  </fo:block>
                </fo:wrapper>
              </fo:block>
            </fo:wrapper>
          </fo:block>
        </fo:wrapper>
      </fo:block>
      <fo:block-container margin="0mm 5mm" border="1pt solid #cccccc" padding="3mm">
        <fo:block-container margin="0mm 5mm" border="1pt solid #aaaaaa" padding="3mm">
          <fo:block-container margin="0mm 5mm" border="1pt solid #999999" padding="3mm">
            <fo:block>Nested block-containers (level 3)</fo:block>
          </fo:block-container>
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
fn conformance_force_page_count_multi_sequence() {
    // force-page-count attribute on fo:page-sequence (Section 7.3)
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
      <fo:block>Chapter 1 content (force-page-count=even adds blank page if needed)</fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="A4" force-page-count="odd">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Chapter 2 content (force-page-count=odd)</fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="A4" force-page-count="no-force">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Final section (force-page-count=no-force)</fo:block>
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
fn conformance_many_page_sequences() {
    // Document with many page-sequences (10+)
    let mut sequences = String::new();
    for i in 1..=10 {
        sequences.push_str(&format!(
            r##"
  <fo:page-sequence master-reference="A4" format="{}">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="14pt" font-weight="bold">Section {}</fo:block>
      <fo:block>Content for section {} on page <fo:page-number/>.</fo:block>
    </fo:flow>
  </fo:page-sequence>"##,
            if i <= 3 {
                "i"
            } else if i <= 6 {
                "1"
            } else {
                "A"
            },
            i,
            i
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
  {}
</fo:root>"##,
        sequences
    );

    let result = process_fo_document(&fo_xml);
    assert!(
        result.is_ok(),
        "Many page sequences should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_rtl_arabic_text() {
    // RTL text with Arabic characters (writing-mode rl-tb)
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
      <fo:block writing-mode="rl" direction="rtl" text-align="right">
        مرحبا بالعالم - Hello World in Arabic
      </fo:block>
      <fo:block writing-mode="rl" direction="rtl" text-align="right" space-before="4pt">
        هذا النص بالعربية. This text contains Arabic content with RTL direction.
      </fo:block>
      <fo:block space-before="4pt">LTR text after RTL block.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "RTL Arabic text should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_rtl_hebrew_text() {
    // RTL text with Hebrew characters
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
      <fo:block direction="rtl" text-align="right" font-size="14pt">
        שלום עולם - Hebrew text
      </fo:block>
      <fo:block direction="rtl" text-align="right" space-before="4pt">
        זהו טקסט בעברית. This is Hebrew text in RTL layout.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "RTL Hebrew text should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_mixed_bidi_content() {
    // Mixed bidirectional content (Arabic/Hebrew + English) (Section 7.13)
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
      <fo:block space-after="4pt">English text with embedded <fo:inline direction="rtl">عربي</fo:inline> Arabic.</fo:block>
      <fo:block direction="rtl" text-align="right" space-after="4pt">
        Arabic text with embedded
        <fo:bidi-override direction="ltr" unicode-bidi="embed">English</fo:bidi-override>
        words.
      </fo:block>
      <fo:block space-after="4pt">
        Citation: "<fo:bidi-override direction="rtl" unicode-bidi="embed">מקור עברי</fo:bidi-override>" (Hebrew source)
      </fo:block>
      <fo:block space-after="4pt">Numbers in RTL:
        <fo:inline direction="rtl">٤٥٦٧</fo:inline>
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Mixed bidi content should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_cjk_vertical_text() {
    // CJK content with various text properties (Section 7.13)
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
      <fo:block font-size="14pt" space-after="6pt">日本語テキスト Japanese Text</fo:block>
      <fo:block space-after="4pt">
        Japanese: 日本語のテキストコンテンツ。漢字とひらがなとカタカナが混在しています。
      </fo:block>
      <fo:block space-after="4pt">
        Chinese: 中文文字内容。简体中文和繁體中文字符测试。
      </fo:block>
      <fo:block space-after="4pt">
        Korean: 한국어 텍스트 내용. 한글 문자 렌더링 테스트.
      </fo:block>
      <fo:block>
        Mixed CJK and Latin: 日本語 (Japanese) + English + 中文 (Chinese).
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(result.is_ok(), "CJK text should work: {:?}", result.err());
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_multilingual_document() {
    // Complete multilingual document with many scripts
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
      <fo:block font-size="16pt" font-weight="bold" text-align="center" space-after="8pt">
        Multilingual Document / 多言語文書 / وثيقة متعددة اللغات
      </fo:block>
      <fo:block xml:lang="en" space-after="4pt">English: The quick brown fox jumps over the lazy dog.</fo:block>
      <fo:block xml:lang="de" space-after="4pt">German: Der schnelle braune Fuchs springt über den faulen Hund.</fo:block>
      <fo:block xml:lang="fr" space-after="4pt">French: Le renard brun rapide saute par-dessus le chien paresseux.</fo:block>
      <fo:block xml:lang="es" space-after="4pt">Spanish: El zorro marrón rápido salta sobre el perro perezoso.</fo:block>
      <fo:block xml:lang="pt" space-after="4pt">Portuguese: A raposa marrom rápida pula sobre o cachorro preguiçoso.</fo:block>
      <fo:block xml:lang="ru" space-after="4pt">Russian: Быстрая коричневая лиса прыгает через ленивую собаку.</fo:block>
      <fo:block xml:lang="ja" space-after="4pt">Japanese: 素早い茶色のキツネが怠け者の犬を飛び越えます。</fo:block>
      <fo:block xml:lang="zh" space-after="4pt">Chinese: 敏捷的棕色狐狸跳过了懒惰的狗。</fo:block>
      <fo:block xml:lang="ko" space-after="4pt">Korean: 빠른 갈색 여우가 게으른 개를 뛰어넘습니다.</fo:block>
      <fo:block xml:lang="ar" direction="rtl" text-align="right" space-after="4pt">Arabic: الثعلب البني السريع يقفز فوق الكلب الكسول.</fo:block>
      <fo:block xml:lang="he" direction="rtl" text-align="right">Hebrew: השועל החום המהיר קופץ מעל הכלב העצלן.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Multilingual document should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_portrait_landscape_mixed() {
    // Mixed portrait and landscape page masters (Section 7.1)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="portrait"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
    <fo:simple-page-master master-name="landscape"
      page-width="297mm" page-height="210mm"
      margin-top="15mm" margin-bottom="15mm"
      margin-left="15mm" margin-right="15mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="portrait">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="14pt" font-weight="bold">Portrait Section</fo:block>
      <fo:block>This is a portrait page (210mm x 297mm).</fo:block>
      <fo:block>Standard A4 portrait orientation document content.</fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="landscape">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="14pt" font-weight="bold">Landscape Section</fo:block>
      <fo:block>This is a landscape page (297mm x 210mm).</fo:block>
      <fo:table table-layout="fixed" width="260mm">
        <fo:table-column column-width="130mm"/>
        <fo:table-column column-width="130mm"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell border="1pt solid black" padding="2mm">
              <fo:block>Wider left column content</fo:block>
            </fo:table-cell>
            <fo:table-cell border="1pt solid black" padding="2mm">
              <fo:block>Wider right column content</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="portrait">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="14pt" font-weight="bold">Back to Portrait</fo:block>
      <fo:block>Returning to portrait orientation after landscape section.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Portrait+landscape mixed should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_all_four_regions() {
    // Document with all four page regions populated (Section 7.1)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="full-page"
      page-width="210mm" page-height="297mm"
      margin-top="25mm" margin-bottom="25mm"
      margin-left="30mm" margin-right="30mm">
      <fo:region-body region-name="xsl-region-body"/>
      <fo:region-before region-name="xsl-region-before" extent="25mm"/>
      <fo:region-after region-name="xsl-region-after" extent="25mm"/>
      <fo:region-start region-name="xsl-region-start" extent="30mm"/>
      <fo:region-end region-name="xsl-region-end" extent="30mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="full-page">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block font-size="10pt" font-weight="bold" text-align="center">
        Document Header - All Regions Test
      </fo:block>
      <fo:block font-size="8pt" text-align="center" color="#666666">
        Page <fo:page-number/> of the document
      </fo:block>
    </fo:static-content>
    <fo:static-content flow-name="xsl-region-after">
      <fo:block font-size="8pt" text-align="center" color="#666666">
        Footer: Confidential Document - Do Not Distribute
      </fo:block>
    </fo:static-content>
    <fo:static-content flow-name="xsl-region-start">
      <fo:block-container writing-mode="tb" reference-orientation="90">
        <fo:block font-size="8pt" text-align="center">Left Margin</fo:block>
      </fo:block-container>
    </fo:static-content>
    <fo:static-content flow-name="xsl-region-end">
      <fo:block-container writing-mode="tb" reference-orientation="-90">
        <fo:block font-size="8pt" text-align="center">Right Margin</fo:block>
      </fo:block-container>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="16pt" font-weight="bold" space-after="8pt">
        Main Document Content
      </fo:block>
      <fo:block>
        This document uses all four page regions: header, footer, left margin, and right margin.
        The body content appears in the central region.
      </fo:block>
      <fo:block space-before="6pt">
        This tests proper layout with all regions simultaneously active.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "All four regions should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_high_volume_pages() {
    // Document with many pages (50 paragraphs) verifying pagination
    let mut blocks = String::new();
    for i in 1..=50 {
        blocks.push_str(&format!(
            "<fo:block space-before=\"4pt\" space-after=\"4pt\">Paragraph {}: Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. This is page content for block number {}.</fo:block>\n",
            i, i
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
      <fo:region-body margin-bottom="15mm"/>
      <fo:region-after extent="15mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:static-content flow-name="xsl-region-after">
      <fo:block font-size="8pt" text-align="center">Page <fo:page-number/></fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="16pt" font-weight="bold" space-after="10pt">Long Document Test</fo:block>
      {}
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        blocks
    );

    let result = process_fo_document(&fo_xml);
    assert!(
        result.is_ok(),
        "High volume pages should work: {:?}",
        result.err()
    );
    let bytes = result.expect("test: should succeed");
    assert!(!bytes.is_empty(), "Output should not be empty");
}

#[test]
fn conformance_complex_gradient_document() {
    // Document with various background patterns and colors
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
      <fo:block background-color="#003366" color="white" padding="5mm"
        font-size="16pt" font-weight="bold" space-after="4mm">
        Dark blue header bar
      </fo:block>
      <fo:block-container background-color="#f0f4f8" border="1pt solid #d0d8e0"
        padding="5mm" space-after="4mm">
        <fo:block font-weight="bold" space-after="2mm">Info Box</fo:block>
        <fo:block font-size="9pt">Blue-tinted background information box content.</fo:block>
      </fo:block-container>
      <fo:block background-color="#fff3cd" border="2pt solid #ffc107" padding="4mm" space-after="4mm">
        Warning box with amber/yellow background
      </fo:block>
      <fo:block background-color="#d4edda" border="2pt solid #28a745" padding="4mm" space-after="4mm">
        Success box with green background
      </fo:block>
      <fo:block background-color="#f8d7da" border="2pt solid #dc3545" padding="4mm" space-after="4mm">
        Error box with red background
      </fo:block>
      <fo:table table-layout="fixed" width="160mm">
        <fo:table-column column-width="80mm"/>
        <fo:table-column column-width="80mm"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell background-color="#cce5ff" padding="3mm" border="1pt solid #004085">
              <fo:block font-size="9pt">Blue cell</fo:block>
            </fo:table-cell>
            <fo:table-cell background-color="#d1ecf1" padding="3mm" border="1pt solid #0c5460">
              <fo:block font-size="9pt">Cyan cell</fo:block>
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
        "Complex background colors should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_two_column_article() {
    // Two-column article with running header (common real-world pattern)
    let mut paragraphs = String::new();
    for i in 1..=8 {
        paragraphs.push_str(&format!(
            "<fo:block space-after=\"5pt\">Paragraph {}: Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation.</fo:block>\n",
            i
        ));
    }

    let fo_xml = format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="article"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="15mm" margin-right="15mm">
      <fo:region-body column-count="2" column-gap="8mm" margin-top="15mm" margin-bottom="15mm"/>
      <fo:region-before extent="15mm"/>
      <fo:region-after extent="15mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="article">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block font-size="9pt" font-style="italic" text-align="center" border-bottom="0.5pt solid #cccccc" padding-bottom="2pt">
        Journal of Technical Publishing - Volume 1
      </fo:block>
    </fo:static-content>
    <fo:static-content flow-name="xsl-region-after">
      <fo:block font-size="9pt" text-align="center" border-top="0.5pt solid #cccccc" padding-top="2pt">
        Page <fo:page-number/>
      </fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block span="all" font-size="18pt" font-weight="bold" text-align="center" space-after="4pt">
        Two-Column Article Format
      </fo:block>
      <fo:block span="all" font-size="11pt" font-style="italic" text-align="center" space-after="8pt">
        Testing Multi-Column Layout with Running Headers and Footers
      </fo:block>
      {}
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        paragraphs
    );

    let result = process_fo_document(&fo_xml);
    assert!(
        result.is_ok(),
        "Two-column article should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_stress_large_table() {
    // Large table with many rows and columns
    let mut rows = String::new();
    for row in 1..=25 {
        let mut cells = String::new();
        for col in 1..=6 {
            cells.push_str(&format!(
                r##"<fo:table-cell border="0.5pt solid #cccccc" padding="1mm">
                  <fo:block font-size="7pt">R{}C{}</fo:block>
                </fo:table-cell>"##,
                row, col
            ));
        }
        rows.push_str(&format!(
            r##"<fo:table-row background-color="{}">{}</fo:table-row>"##,
            if row % 2 == 0 { "#f8f8f8" } else { "white" },
            cells
        ));
    }

    let fo_xml = format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="15mm" margin-bottom="15mm"
      margin-left="10mm" margin-right="10mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:table table-layout="fixed" width="190mm">
        <fo:table-column column-width="proportional-column-width(1)"/>
        <fo:table-column column-width="proportional-column-width(2)"/>
        <fo:table-column column-width="proportional-column-width(2)"/>
        <fo:table-column column-width="proportional-column-width(1)"/>
        <fo:table-column column-width="proportional-column-width(2)"/>
        <fo:table-column column-width="proportional-column-width(1)"/>
        <fo:table-header>
          <fo:table-row background-color="#333333">
            <fo:table-cell padding="1.5mm" border="1pt solid #333333">
              <fo:block color="white" font-weight="bold" font-size="8pt">Col 1</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="1.5mm" border="1pt solid #333333">
              <fo:block color="white" font-weight="bold" font-size="8pt">Col 2</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="1.5mm" border="1pt solid #333333">
              <fo:block color="white" font-weight="bold" font-size="8pt">Col 3</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="1.5mm" border="1pt solid #333333">
              <fo:block color="white" font-weight="bold" font-size="8pt">Col 4</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="1.5mm" border="1pt solid #333333">
              <fo:block color="white" font-weight="bold" font-size="8pt">Col 5</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="1.5mm" border="1pt solid #333333">
              <fo:block color="white" font-weight="bold" font-size="8pt">Col 6</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-header>
        <fo:table-body>{}</fo:table-body>
      </fo:table>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        rows
    );

    let result = process_fo_document(&fo_xml);
    assert!(
        result.is_ok(),
        "Large table stress test should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_stress_complex_nested_structure() {
    // Complex nesting: lists inside table cells inside block-containers
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
      <fo:block-container border="2pt solid #003366" padding="5mm">
        <fo:block font-weight="bold" font-size="13pt" space-after="4pt">Complex Nested Structure</fo:block>
        <fo:table table-layout="fixed" width="150mm">
          <fo:table-column column-width="75mm"/>
          <fo:table-column column-width="75mm"/>
          <fo:table-body>
            <fo:table-row>
              <fo:table-cell border="1pt solid #cccccc" padding="3mm">
                <fo:block font-weight="bold" space-after="3pt">Features:</fo:block>
                <fo:list-block provisional-distance-between-starts="10mm">
                  <fo:list-item>
                    <fo:list-item-label end-indent="label-end()">
                      <fo:block>&#x2022;</fo:block>
                    </fo:list-item-label>
                    <fo:list-item-body start-indent="body-start()">
                      <fo:block font-size="9pt">Fast processing</fo:block>
                    </fo:list-item-body>
                  </fo:list-item>
                  <fo:list-item>
                    <fo:list-item-label end-indent="label-end()">
                      <fo:block>&#x2022;</fo:block>
                    </fo:list-item-label>
                    <fo:list-item-body start-indent="body-start()">
                      <fo:block font-size="9pt">Memory efficient</fo:block>
                    </fo:list-item-body>
                  </fo:list-item>
                  <fo:list-item>
                    <fo:list-item-label end-indent="label-end()">
                      <fo:block>&#x2022;</fo:block>
                    </fo:list-item-label>
                    <fo:list-item-body start-indent="body-start()">
                      <fo:block font-size="9pt">Zero-copy where possible</fo:block>
                    </fo:list-item-body>
                  </fo:list-item>
                </fo:list-block>
              </fo:table-cell>
              <fo:table-cell border="1pt solid #cccccc" padding="3mm">
                <fo:block font-weight="bold" space-after="3pt">Benefits:</fo:block>
                <fo:block-container background-color="#f9f9f9" border="0.5pt solid #dddddd" padding="3pt">
                  <fo:block font-size="9pt">High throughput document generation</fo:block>
                  <fo:block font-size="9pt">Low memory footprint</fo:block>
                  <fo:block font-size="9pt">Rust memory safety guarantees</fo:block>
                </fo:block-container>
                <fo:block font-size="8pt" font-style="italic" space-before="4pt">
                  All tested with comprehensive conformance suite.
                </fo:block>
              </fo:table-cell>
            </fo:table-row>
          </fo:table-body>
        </fo:table>
      </fo:block-container>
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
fn conformance_stress_many_cross_references() {
    // Document with many internal cross-references
    let mut content = String::new();
    // Create 20 sections with IDs
    for i in 1..=20 {
        let next = if i < 20 { i + 1 } else { 1 };
        content.push_str(&format!(r##"
      <fo:block id="sec-{i}" font-weight="bold" font-size="12pt" space-before="6pt" space-after="2pt">
        Section {i}: Topic {i}
      </fo:block>
      <fo:block font-size="9pt">
        This is section {i} content. See also
        <fo:basic-link internal-destination="sec-{next}" color="#0066cc">section {next}</fo:basic-link>
        and page <fo:page-number-citation ref-id="sec-{next}"/>.
      </fo:block>"##,
            i = i,
            next = next,
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
      <fo:block font-size="18pt" font-weight="bold" space-after="8pt">Document Index</fo:block>
      {}
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        content
    );

    let result = process_fo_document(&fo_xml);
    assert!(
        result.is_ok(),
        "Many cross-references should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_stress_multi_section_book() {
    // Multi-section book-like document with TOC, chapters, appendix
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="roman"
      page-width="210mm" page-height="297mm"
      margin-top="25mm" margin-bottom="25mm"
      margin-left="25mm" margin-right="25mm">
      <fo:region-body margin-bottom="15mm"/>
      <fo:region-after extent="15mm"/>
    </fo:simple-page-master>
    <fo:simple-page-master master-name="main"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="25mm"
      margin-left="25mm" margin-right="25mm">
      <fo:region-body margin-top="12mm" margin-bottom="15mm"/>
      <fo:region-before extent="12mm"/>
      <fo:region-after extent="15mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="roman" format="i">
    <fo:static-content flow-name="xsl-region-after">
      <fo:block font-size="9pt" text-align="center"><fo:page-number/></fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="18pt" font-weight="bold" text-align="center" space-after="12pt">Table of Contents</fo:block>
      <fo:block text-align-last="justify" space-after="3pt">
        Chapter 1: Introduction
        <fo:leader leader-pattern="dots"/>
        <fo:page-number-citation ref-id="ch1"/>
      </fo:block>
      <fo:block text-align-last="justify" space-after="3pt">
        Chapter 2: Methods
        <fo:leader leader-pattern="dots"/>
        <fo:page-number-citation ref-id="ch2"/>
      </fo:block>
      <fo:block text-align-last="justify" space-after="3pt">
        Chapter 3: Results
        <fo:leader leader-pattern="dots"/>
        <fo:page-number-citation ref-id="ch3"/>
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="main" format="1" initial-page-number="1">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block font-size="9pt" font-style="italic">Technical Report 2024</fo:block>
    </fo:static-content>
    <fo:static-content flow-name="xsl-region-after">
      <fo:block font-size="9pt" text-align="center"><fo:page-number/></fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block id="ch1" font-size="16pt" font-weight="bold" space-after="8pt" break-before="page">
        Chapter 1: Introduction
      </fo:block>
      <fo:block text-align="justify" space-after="6pt">
        This technical report presents findings from our research into high-performance
        document processing. Our approach leverages modern systems programming techniques
        to achieve significant improvements over existing solutions.
      </fo:block>
      <fo:block text-align="justify" space-after="6pt">
        The motivation for this work stems from the growing need for efficient PDF generation
        in enterprise environments, where throughput and latency are critical metrics.
      </fo:block>
      <fo:block id="ch2" font-size="16pt" font-weight="bold" space-after="8pt" break-before="page">
        Chapter 2: Methods
      </fo:block>
      <fo:block text-align="justify" space-after="6pt">
        We employed a multi-phase approach to document layout: first parsing the XSL-FO
        input into an intermediate tree representation, then performing layout calculations,
        and finally rendering to the target output format.
      </fo:block>
      <fo:block id="ch3" font-size="16pt" font-weight="bold" space-after="8pt" break-before="page">
        Chapter 3: Results
      </fo:block>
      <fo:block text-align="justify" space-after="6pt">
        Our implementation demonstrates significant performance improvements. Processing time
        for typical business documents was reduced by 75% compared to baseline implementations.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Multi-section book should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_stress_100_page_document() {
    // Document that generates 100+ pages
    let mut sections = String::new();
    for i in 1..=5 {
        let mut paras = String::new();
        for j in 1..=10 {
            paras.push_str(&format!(
                "<fo:block space-before=\"4pt\" space-after=\"4pt\" text-align=\"justify\">Section {} Paragraph {}: Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur.</fo:block>\n",
                i, j
            ));
        }
        sections.push_str(&format!(
            r##"<fo:block id="ch{i}" font-size="14pt" font-weight="bold" break-before="page" space-after="8pt">
              Part {i}: Advanced Topics
            </fo:block>
            {paras}"##,
            i = i,
            paras = paras,
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
      <fo:region-body margin-bottom="12mm"/>
      <fo:region-after extent="12mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:static-content flow-name="xsl-region-after">
      <fo:block font-size="8pt" text-align="center" border-top-style="solid" border-top-width="0.5pt" border-top-color="#999999">
        Page <fo:page-number/>
      </fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="24pt" font-weight="bold" text-align="center" space-after="12pt">
        Large Multi-Page Document
      </fo:block>
      {}
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        sections
    );

    let result = process_fo_document(&fo_xml);
    assert!(
        result.is_ok(),
        "100-page document should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}
#[test]
fn conformance_unicode_special_chars() {
    // Unicode special characters and emoji (Section 7.x)
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
      <fo:block>Latin: ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz</fo:block>
      <fo:block>Digits: 0123456789</fo:block>
      <fo:block>Punctuation: !@#$%^&amp;*()_+-=[]{}|;':",&lt;&gt;.?/</fo:block>
      <fo:block>Currency: $ &#x20AC; &#xa3; &#xa5; &#x20B9; &#x20BD; &#x20BF;</fo:block>
      <fo:block>Arrows: &#x2192; &#x2190; &#x2191; &#x2193; &#x21D2; &#x21D0; &#x21D1; &#x21D3; &#x2194; &#x21D4;</fo:block>
      <fo:block>Math: &#x2211; &#x220F; &#x222B; &#x221A; &#x221E; &#x2202; &#x394; &#x2207; &#x2248; &#x2260; &#x2264; &#x2265; &#xb1; &#xd7; &#xf7;</fo:block>
      <fo:block>Fractions: &#xBD; &#x2153; &#xBC; &#xBE; &#x215B; &#x215C; &#x215D; &#x215E;</fo:block>
      <fo:block>Box drawing: &#x2500; &#x2502; &#x250C; &#x2510; &#x2514; &#x2518; &#x251C; &#x2524; &#x252C; &#x2534; &#x253C;</fo:block>
      <fo:block>Emoji: &#x1F600; &#x1F389; &#x2705; &#x274C; &#x1F680; &#x1F4A1; &#x1F4DA; &#x1F527; &#x26A1; &#x1F30D;</fo:block>
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
fn conformance_long_unbreakable_words() {
    // Very long words without spaces (URL-like content)
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
      <fo:block>Short text before long URL:</fo:block>
      <fo:block font-family="monospace" font-size="9pt" space-before="3pt">
        https://www.example.com/very/long/path/that/does/not/break/automatically/and/goes/on/and/on/forever
      </fo:block>
      <fo:block space-before="6pt">Text after URL.</fo:block>
      <fo:block font-family="monospace" font-size="8pt" space-before="3pt" overflow="hidden">
        AVERYLONGWORDWITHNOSPACESTHATEXCEEDSTHETYPICALPAGEWIDTHBYSEVERALTIMESANDWILLREQUIRESPECIALHANDLING
      </fo:block>
      <fo:block space-before="6pt">Normal text continues here after very long words.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Long unbreakable words should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_xml_escaping() {
    // Properly escaped XML special characters in content (Section 5.x)
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
      <fo:block>Ampersand: A &amp; B</fo:block>
      <fo:block>Less-than: 5 &lt; 10</fo:block>
      <fo:block>Greater-than: 10 &gt; 5</fo:block>
      <fo:block>Quoted: &quot;Hello World&quot;</fo:block>
      <fo:block>Apostrophe: it&apos;s fine</fo:block>
      <fo:block>Numeric decimal: &#65;&#66;&#67; (= ABC)</fo:block>
      <fo:block>Numeric hex: &#x41;&#x42;&#x43; (= ABC)</fo:block>
      <fo:block>Combined: 1 &lt; 2 &amp; 3 &gt; 2 means &quot;1 &lt; 2 and 3 &gt; 2&quot;</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "XML escaping should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_zero_content_areas() {
    // Areas with zero content (empty pages, zero-height blocks)
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
      <fo:block>Content</fo:block>
      <fo:block height="0mm"/>
      <fo:block height="0mm" border="1pt solid black"/>
      <fo:block font-size="10pt">Minimal height text</fo:block>
      <fo:block>More content</fo:block>
      <fo:block space-before="0pt" space-after="0pt" margin="0pt"/>
      <fo:block>Final content</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Zero content areas should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_alternating_page_masters() {
    // Alternating simple-page-masters for odd/even pages (Section 7.1.3)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="odd-page"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="15mm" margin-right="25mm">
      <fo:region-body margin-bottom="12mm"/>
      <fo:region-after extent="12mm" region-name="odd-footer"/>
    </fo:simple-page-master>
    <fo:simple-page-master master-name="even-page"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="25mm" margin-right="15mm">
      <fo:region-body margin-bottom="12mm"/>
      <fo:region-after extent="12mm" region-name="even-footer"/>
    </fo:simple-page-master>
    <fo:page-sequence-master master-name="alternating">
      <fo:repeatable-page-master-alternatives>
        <fo:conditional-page-master-reference master-reference="odd-page"
          odd-or-even="odd"/>
        <fo:conditional-page-master-reference master-reference="even-page"
          odd-or-even="even"/>
      </fo:repeatable-page-master-alternatives>
    </fo:page-sequence-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="alternating">
    <fo:static-content flow-name="odd-footer">
      <fo:block text-align="right" font-size="9pt">Page <fo:page-number/></fo:block>
    </fo:static-content>
    <fo:static-content flow-name="even-footer">
      <fo:block text-align="left" font-size="9pt">Page <fo:page-number/></fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Page 1 (odd): right margin is wider</fo:block>
      <fo:block break-before="page">Page 2 (even): left margin is wider</fo:block>
      <fo:block break-before="page">Page 3 (odd): right margin is wider again</fo:block>
      <fo:block break-before="page">Page 4 (even): left margin is wider again</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Alternating page masters should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_table_omit_header_footer_at_break() {
    // fo:table table-omit-header-at-break and table-omit-footer-at-break (Section 8.3)
    let mut rows = String::new();
    for i in 1..=12 {
        rows.push_str(&format!(
            r##"
          <fo:table-row>
            <fo:table-cell border="0.5pt solid gray" padding="2mm">
              <fo:block font-size="9pt">Row {}</fo:block>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid gray" padding="2mm">
              <fo:block font-size="9pt">Data for row {}</fo:block>
            </fo:table-cell>
          </fo:table-row>"##,
            i, i
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
      <fo:table table-layout="fixed" width="160mm"
        table-omit-header-at-break="true"
        table-omit-footer-at-break="false">
        <fo:table-column column-width="60mm"/>
        <fo:table-column column-width="100mm"/>
        <fo:table-header>
          <fo:table-row background-color="navy">
            <fo:table-cell padding="2mm" border="1pt solid navy">
              <fo:block color="white" font-weight="bold" font-size="9pt">Name</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="2mm" border="1pt solid navy">
              <fo:block color="white" font-weight="bold" font-size="9pt">Value</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-header>
        <fo:table-footer>
          <fo:table-row>
            <fo:table-cell number-columns-spanned="2" padding="2mm" border="1pt solid silver">
              <fo:block font-size="8pt" font-style="italic">Table footer (omit=false)</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-footer>
        <fo:table-body>{}</fo:table-body>
      </fo:table>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        rows
    );
    let result = process_fo_document(&fo_xml);
    assert!(
        result.is_ok(),
        "Table omit header/footer at break should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_block_container_overflow_modes() {
    // fo:block-container with hidden, visible, and auto overflow values (Section 7.14.7)
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
      <fo:block font-weight="bold" space-after="4pt">Overflow hidden (fixed height):</fo:block>
      <fo:block-container height="20mm" overflow="hidden" border="1pt solid red"
        padding="2mm" space-after="8pt">
        <fo:block>Line 1 of content</fo:block>
        <fo:block>Line 2 of content</fo:block>
        <fo:block>Line 3 of content (may be clipped)</fo:block>
        <fo:block>Line 4 of content (should be hidden)</fo:block>
        <fo:block>Line 5 of content (definitely hidden)</fo:block>
      </fo:block-container>
      <fo:block font-weight="bold" space-after="4pt">Overflow visible (fixed height):</fo:block>
      <fo:block-container height="20mm" overflow="visible" border="1pt solid blue"
        padding="2mm" space-after="8pt">
        <fo:block>Line 1 of content</fo:block>
        <fo:block>Line 2 of content</fo:block>
        <fo:block>Line 3 of content (visible overflow)</fo:block>
        <fo:block>Line 4 of content</fo:block>
      </fo:block-container>
      <fo:block font-weight="bold" space-after="4pt">Auto height (expands):</fo:block>
      <fo:block-container border="1pt solid green" padding="2mm">
        <fo:block>Line 1</fo:block>
        <fo:block>Line 2</fo:block>
        <fo:block>Line 3 (container auto-sizes)</fo:block>
      </fo:block-container>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Block container overflow modes should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_page_number_grouping_large() {
    // fo:page-number with grouping-separator, grouping-size, and large initial-page-number (Section 7.8)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body margin-bottom="15mm"/>
      <fo:region-after extent="15mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4"
    initial-page-number="1234"
    format="1"
    grouping-separator=","
    grouping-size="3">
    <fo:static-content flow-name="xsl-region-after">
      <fo:block font-size="9pt" text-align="center">
        Page <fo:page-number/>
      </fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="14pt" font-weight="bold">Grouping Separator Test</fo:block>
      <fo:block>Page number with grouping separator (comma) and grouping size 3.</fo:block>
      <fo:block>This page starts at number 1234 (displayed as 1,234).</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Page number grouping with large numbers should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_alignment_properties() {
    // alignment-adjust and alignment-baseline on fo:inline (Section 7.8)
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
        Normal text with
        <fo:inline alignment-baseline="alphabetic">alphabetic baseline</fo:inline>
        alignment.
      </fo:block>
      <fo:block space-after="4pt">
        Text with
        <fo:inline alignment-baseline="middle" font-size="8pt">middle aligned small text</fo:inline>
        mixed with normal.
      </fo:block>
      <fo:block space-after="4pt">
        Text with
        <fo:inline alignment-adjust="2pt">adjusted up by 2pt</fo:inline>
        and
        <fo:inline alignment-adjust="-2pt">adjusted down by 2pt</fo:inline>
        content.
      </fo:block>
      <fo:block space-after="4pt">
        Chemical formula: H<fo:inline baseline-shift="sub" font-size="8pt">2</fo:inline>O
        and CO<fo:inline baseline-shift="sub" font-size="8pt">2</fo:inline>
        and E=mc<fo:inline baseline-shift="super" font-size="8pt">2</fo:inline>
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Alignment properties should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_first_rest_page_masters() {
    // Conditional page masters for first and rest pages (Section 7.1.3)
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="first-page"
      page-width="210mm" page-height="297mm"
      margin-top="40mm" margin-bottom="20mm"
      margin-left="25mm" margin-right="25mm">
      <fo:region-body margin-top="20mm"/>
      <fo:region-before extent="20mm" region-name="first-header"/>
    </fo:simple-page-master>
    <fo:simple-page-master master-name="rest-page"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body margin-top="15mm" margin-bottom="12mm"/>
      <fo:region-before extent="15mm" region-name="rest-header"/>
      <fo:region-after extent="12mm" region-name="rest-footer"/>
    </fo:simple-page-master>
    <fo:page-sequence-master master-name="book">
      <fo:repeatable-page-master-alternatives>
        <fo:conditional-page-master-reference master-reference="first-page"
          page-position="first"/>
        <fo:conditional-page-master-reference master-reference="rest-page"
          page-position="rest"/>
      </fo:repeatable-page-master-alternatives>
    </fo:page-sequence-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="book">
    <fo:static-content flow-name="first-header">
      <fo:block font-size="24pt" font-weight="bold" text-align="center">Book Title</fo:block>
    </fo:static-content>
    <fo:static-content flow-name="rest-header">
      <fo:block font-size="9pt" text-align="center" border-bottom="0.5pt solid gray">
        Chapter 1
      </fo:block>
    </fo:static-content>
    <fo:static-content flow-name="rest-footer">
      <fo:block font-size="9pt" text-align="center">Page <fo:page-number/></fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="16pt" font-weight="bold" space-after="8pt">
        Chapter 1: Introduction
      </fo:block>
      <fo:block space-after="6pt">
        This is the first page of the chapter, using a special first-page master
        with a prominent title header and larger top margin.
      </fo:block>
      <fo:block break-before="page" space-after="6pt">
        This is the second page using the rest-page master with smaller header.
      </fo:block>
      <fo:block>Content continues on subsequent pages with the same rest-page layout.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "First/rest page masters should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}
