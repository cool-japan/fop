//! Internationalization (i18n) and Unicode support tests
//!
//! These tests verify that the FOP processor correctly handles:
//! - Japanese (Hiragana, Katakana, Kanji)
//! - Chinese (Simplified and Traditional)
//! - Korean (Hangul)
//! - Arabic (RTL text)
//! - Various Unicode symbols and emoji
//! - Mixed-language documents

use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use fop_render::PdfRenderer;
use std::io::Cursor;

/// Test Japanese text (Hiragana, Katakana, Kanji) in PDF
#[test]
fn test_japanese_hiragana_katakana_kanji() {
    let fo_doc = r###"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm">
      <fo:region-body margin="20mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="16pt" font-weight="bold" space-after="10pt">
        日本語テスト (Japanese Test)
      </fo:block>

      <fo:block font-size="12pt" space-after="6pt">
        ひらがな: あいうえお かきくけこ さしすせそ
      </fo:block>

      <fo:block font-size="12pt" space-after="6pt">
        カタカナ: アイウエオ カキクケコ サシスセソ
      </fo:block>

      <fo:block font-size="12pt" space-after="6pt">
        漢字: 日本語 東京 京都 大阪 北海道 沖縄
      </fo:block>

      <fo:block font-size="12pt" space-after="6pt">
        混合文: こんにちは、世界！ Hello, World!
      </fo:block>

      <fo:block font-size="12pt">
        長文: 吾輩は猫である。名前はまだ無い。どこで生れたかとんと見当がつかぬ。
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"###;

    let builder = FoTreeBuilder::new();
    let fo_tree = builder
        .parse(Cursor::new(fo_doc))
        .expect("Failed to parse Japanese FO document");

    let engine = LayoutEngine::new();
    let area_tree = engine
        .layout(&fo_tree)
        .expect("Failed to layout Japanese document");

    let renderer = PdfRenderer::new();
    let pdf = renderer
        .render(&area_tree)
        .expect("Failed to render Japanese PDF");

    assert!(
        !pdf.pages.is_empty(),
        "Should generate PDF with Japanese text"
    );
}

/// Test Chinese text (Simplified and Traditional)
#[test]
fn test_chinese_simplified_traditional() {
    let fo_doc = r###"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm">
      <fo:region-body margin="20mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="16pt" font-weight="bold" space-after="10pt">
        中文测试 (Chinese Test)
      </fo:block>

      <fo:block font-size="12pt" space-after="6pt">
        简体中文: 你好世界 北京 上海 广州 深圳
      </fo:block>

      <fo:block font-size="12pt" space-after="6pt">
        繁體中文: 你好世界 台北 香港 澳門
      </fo:block>

      <fo:block font-size="12pt">
        成语: 一帆风顺 马到成功 心想事成
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"###;

    let builder = FoTreeBuilder::new();
    let fo_tree = builder
        .parse(Cursor::new(fo_doc))
        .expect("Failed to parse Chinese FO document");

    let engine = LayoutEngine::new();
    let area_tree = engine
        .layout(&fo_tree)
        .expect("Failed to layout Chinese document");

    let renderer = PdfRenderer::new();
    let pdf = renderer
        .render(&area_tree)
        .expect("Failed to render Chinese PDF");

    assert!(
        !pdf.pages.is_empty(),
        "Should generate PDF with Chinese text"
    );
}

/// Test Korean Hangul
#[test]
fn test_korean_hangul() {
    let fo_doc = r###"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm">
      <fo:region-body margin="20mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="16pt" font-weight="bold" space-after="10pt">
        한국어 테스트 (Korean Test)
      </fo:block>

      <fo:block font-size="12pt" space-after="6pt">
        안녕하세요 여러분
      </fo:block>

      <fo:block font-size="12pt" space-after="6pt">
        서울 부산 대구 인천 광주
      </fo:block>

      <fo:block font-size="12pt">
        한글: 가나다라마바사아자차카타파하
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"###;

    let builder = FoTreeBuilder::new();
    let fo_tree = builder
        .parse(Cursor::new(fo_doc))
        .expect("Failed to parse Korean FO document");

    let engine = LayoutEngine::new();
    let area_tree = engine
        .layout(&fo_tree)
        .expect("Failed to layout Korean document");

    let renderer = PdfRenderer::new();
    let pdf = renderer
        .render(&area_tree)
        .expect("Failed to render Korean PDF");

    assert!(
        !pdf.pages.is_empty(),
        "Should generate PDF with Korean text"
    );
}

/// Test mixed CJK (Chinese, Japanese, Korean) in same document
#[test]
fn test_mixed_cjk_document() {
    let fo_doc = r###"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm">
      <fo:region-body margin="20mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="16pt" font-weight="bold" space-after="10pt">
        Multilingual CJK Document
      </fo:block>

      <fo:block font-size="12pt" space-after="6pt">
        日本語: こんにちは世界
      </fo:block>

      <fo:block font-size="12pt" space-after="6pt">
        中文: 你好世界
      </fo:block>

      <fo:block font-size="12pt" space-after="6pt">
        한국어: 안녕하세요
      </fo:block>

      <fo:block font-size="12pt" space-after="6pt">
        English: Hello, World!
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"###;

    let builder = FoTreeBuilder::new();
    let fo_tree = builder
        .parse(Cursor::new(fo_doc))
        .expect("Failed to parse mixed CJK document");

    let engine = LayoutEngine::new();
    let area_tree = engine
        .layout(&fo_tree)
        .expect("Failed to layout mixed CJK document");

    let renderer = PdfRenderer::new();
    let pdf = renderer
        .render(&area_tree)
        .expect("Failed to render mixed CJK PDF");

    assert!(
        !pdf.pages.is_empty(),
        "Should generate PDF with mixed CJK text"
    );
}

/// Test Arabic (RTL - Right-to-Left) text
#[test]
fn test_arabic_rtl_text() {
    let fo_doc = r###"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm">
      <fo:region-body margin="20mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="16pt" font-weight="bold" space-after="10pt">
        Arabic Text Test
      </fo:block>

      <fo:block font-size="12pt" writing-mode="rl-tb" space-after="6pt">
        مرحبا بالعالم
      </fo:block>

      <fo:block font-size="12pt" writing-mode="rl-tb">
        السلام عليكم
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"###;

    let builder = FoTreeBuilder::new();
    let fo_tree = builder
        .parse(Cursor::new(fo_doc))
        .expect("Failed to parse Arabic FO document");

    let engine = LayoutEngine::new();
    let area_tree = engine
        .layout(&fo_tree)
        .expect("Failed to layout Arabic document");

    let renderer = PdfRenderer::new();
    let pdf = renderer
        .render(&area_tree)
        .expect("Failed to render Arabic PDF");

    assert!(
        !pdf.pages.is_empty(),
        "Should generate PDF with Arabic text"
    );
}

/// Test various Unicode symbols and special characters
#[test]
fn test_unicode_symbols_and_emoji() {
    let fo_doc = r###"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm">
      <fo:region-body margin="20mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="14pt" font-weight="bold" space-after="8pt">
        Unicode Symbols and Characters
      </fo:block>

      <fo:block font-size="11pt" space-after="6pt">
        Currency: € £ ¥ ₹ ₽ ₩ ₪
      </fo:block>

      <fo:block font-size="11pt" space-after="6pt">
        Math: ∑ ∏ √ ∞ ≈ ≠ ≤ ≥ ± × ÷
      </fo:block>

      <fo:block font-size="11pt" space-after="6pt">
        Arrows: ← → ↑ ↓ ↔ ↕ ⇐ ⇒ ⇔
      </fo:block>

      <fo:block font-size="11pt" space-after="6pt">
        Symbols: © ® ™ § ¶ † ‡ • … ‰ ′ ″
      </fo:block>

      <fo:block font-size="11pt" space-after="6pt">
        Greek: α β γ δ ε ζ η θ ι κ λ μ
      </fo:block>

      <fo:block font-size="11pt">
        Emoji: 😀 😃 😄 😁 🎉 🎊 ❤️ ⭐ ✨
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"###;

    let builder = FoTreeBuilder::new();
    let fo_tree = builder
        .parse(Cursor::new(fo_doc))
        .expect("Failed to parse symbols document");

    let engine = LayoutEngine::new();
    let area_tree = engine
        .layout(&fo_tree)
        .expect("Failed to layout symbols document");

    let renderer = PdfRenderer::new();
    let pdf = renderer
        .render(&area_tree)
        .expect("Failed to render symbols PDF");

    assert!(
        !pdf.pages.is_empty(),
        "Should generate PDF with Unicode symbols"
    );
}

/// Test European languages with diacritics
#[test]
fn test_european_languages_diacritics() {
    let fo_doc = r###"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm">
      <fo:region-body margin="20mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="14pt" font-weight="bold" space-after="8pt">
        European Languages
      </fo:block>

      <fo:block font-size="11pt" space-after="4pt">
        German: Grüß Gott, Äpfel, Öffnung, Übung
      </fo:block>

      <fo:block font-size="11pt" space-after="4pt">
        French: Bonjour, café, crème brûlée, Côte d'Azur
      </fo:block>

      <fo:block font-size="11pt" space-after="4pt">
        Spanish: ¡Hola! ¿Cómo estás? Niño, España, José
      </fo:block>

      <fo:block font-size="11pt" space-after="4pt">
        Portuguese: Olá, São Paulo, pão, ação
      </fo:block>

      <fo:block font-size="11pt" space-after="4pt">
        Italian: Ciao, città, perché, così
      </fo:block>

      <fo:block font-size="11pt" space-after="4pt">
        Czech: Dobrý den, Česká republika, Václav
      </fo:block>

      <fo:block font-size="11pt">
        Polish: Cześć, Kraków, Łódź, Gdańsk
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"###;

    let builder = FoTreeBuilder::new();
    let fo_tree = builder
        .parse(Cursor::new(fo_doc))
        .expect("Failed to parse European languages document");

    let engine = LayoutEngine::new();
    let area_tree = engine
        .layout(&fo_tree)
        .expect("Failed to layout European languages document");

    let renderer = PdfRenderer::new();
    let pdf = renderer
        .render(&area_tree)
        .expect("Failed to render European languages PDF");

    assert!(
        !pdf.pages.is_empty(),
        "Should generate PDF with European languages"
    );
}

/// Test realistic Japanese business document
#[test]
fn test_realistic_japanese_business_document() {
    let fo_doc = r###"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm">
      <fo:region-body margin="25mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="18pt" font-weight="bold" text-align="center" space-after="15pt">
        請求書
      </fo:block>

      <fo:block font-size="11pt" space-after="10pt">
        株式会社サンプル御中
      </fo:block>

      <fo:block font-size="11pt" space-after="6pt">
        下記の通りご請求申し上げます。
      </fo:block>

      <fo:block font-size="11pt" space-after="4pt">
        商品名: ソフトウェアライセンス
      </fo:block>

      <fo:block font-size="11pt" space-after="4pt">
        数量: 10
      </fo:block>

      <fo:block font-size="11pt" space-after="4pt">
        単価: ¥50,000
      </fo:block>

      <fo:block font-size="11pt" font-weight="bold" space-before="8pt">
        合計金額: ¥500,000
      </fo:block>

      <fo:block font-size="9pt" space-before="15pt" color="#666666">
        東京都千代田区丸の内1-1-1 | 電話: 03-1234-5678
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"###;

    let builder = FoTreeBuilder::new();
    let fo_tree = builder
        .parse(Cursor::new(fo_doc))
        .expect("Failed to parse Japanese invoice");

    let engine = LayoutEngine::new();
    let area_tree = engine
        .layout(&fo_tree)
        .expect("Failed to layout Japanese invoice");

    let renderer = PdfRenderer::new();
    let pdf = renderer
        .render(&area_tree)
        .expect("Failed to render Japanese invoice PDF");

    assert!(
        !pdf.pages.is_empty(),
        "Should generate realistic Japanese business document"
    );
}
