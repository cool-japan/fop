//! International Language Support Showcase
//!
//! Demonstrates Apache FOP Rust's i18n capabilities with multiple languages:
//! - 日本語 (Japanese: Hiragana, Katakana, Kanji)
//! - 繁體中文 (Traditional Chinese)
//! - 简体中文 (Simplified Chinese)
//! - 한국어 (Korean: Hangul)
//! - ภาษาไทย (Thai)
//! - Deutsch (German: Umlauts)
//! - Français (French: Accents)
//! - Español (Spanish: Tildes)
//! - Русский (Russian: Cyrillic)
//! - العربية (Arabic: RTL)
//! - Ελληνικά (Greek)

use fop_render::PdfDocument;
use fop_render::pdf::document::PdfPage;
use fop_types::Length;
use std::fs;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating Multilingual PDF Showcase...\n");

    // Try to load a comprehensive Unicode font
    let font_paths = vec![
        // Noto fonts (best coverage)
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
        // Fallback fonts
        "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        "/System/Library/Fonts/Supplemental/Arial Unicode.ttf", // macOS
    ];

    let mut font_data = None;
    let mut font_source = String::new();

    for path in &font_paths {
        if let Ok(data) = fs::read(path) {
            println!("✓ Found font: {}", path);
            font_source = path.to_string();
            font_data = Some(data);
            break;
        }
    }

    let font_data = font_data.ok_or(
        "No suitable font found. Please install:\n\
         - Linux: sudo apt install fonts-noto-cjk fonts-noto\n\
         - macOS: (fonts pre-installed)\n\
         - Windows: Install Noto fonts from Google Fonts"
    )?;

    // Create PDF document
    let mut pdf = PdfDocument::new();
    pdf.info.title = Some("International Language Support Showcase".to_string());
    pdf.info.author = Some("Apache FOP Rust".to_string());

    // Embed the font
    println!("Embedding font...");
    let font_index = pdf.embed_font(font_data)?;
    println!("✓ Font embedded (index: {})\n", font_index);

    // Create A4 page
    let mut page = PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0));

    let y_start = Length::from_mm(280.0);
    let x_margin = Length::from_mm(15.0);
    let line_height = Length::from_pt(18.0);
    let mut y = y_start;

    // Title
    let title = "🌍 Apache FOP Rust - International Language Support";
    pdf.font_manager.record_text(font_index, title);
    page.add_text_with_font(
        title,
        x_margin,
        y,
        Length::from_pt(16.0),
        font_index,
    );
    y -= line_height * 2;

    // Language samples
    let samples = vec![
        // Header
        ("Language", "Sample Text", "Translation"),
        ("--------", "-----------", "-----------"),

        // CJK Languages
        ("Japanese (日本語)",
         "こんにちは、世界！桜が咲く春の日。",
         "Hello, world! Spring day when cherry blossoms bloom."),

        ("Traditional Chinese (繁體中文)",
         "您好，世界！繁體字是傳統漢字的書寫系統。",
         "Hello, world! Traditional Chinese is the traditional character system."),

        ("Simplified Chinese (简体中文)",
         "你好，世界！简体字是简化后的汉字系统。",
         "Hello, world! Simplified Chinese is the simplified character system."),

        ("Korean (한국어)",
         "안녕하세요, 세계! 한글은 한국의 문자입니다.",
         "Hello, world! Hangul is the Korean alphabet."),

        // Southeast Asian
        ("Thai (ภาษาไทย)",
         "สวัสดีชาวโลก! ภาษาไทยเป็นภาษาราชการของประเทศไทย",
         "Hello, world! Thai is the official language of Thailand."),

        ("Vietnamese (Tiếng Việt)",
         "Xin chào thế giới! Tiếng Việt có dấu thanh.",
         "Hello, world! Vietnamese has tone marks."),

        // European Languages
        ("German (Deutsch)",
         "Guten Tag! Äpfel, Öl, Übung, ß (Eszett).",
         "Good day! Apples, oil, practice, sharp S."),

        ("French (Français)",
         "Bonjour le monde! Café, naïve, château, œuvre.",
         "Hello world! Coffee, naive, castle, work."),

        ("Spanish (Español)",
         "¡Hola mundo! Mañana, señor, niño, ¿Cómo estás?",
         "Hello world! Tomorrow, sir, boy, How are you?"),

        ("Portuguese (Português)",
         "Olá mundo! São Paulo, ação, coração, não.",
         "Hello world! Sao Paulo, action, heart, no."),

        ("Italian (Italiano)",
         "Ciao mondo! Città, perché, più, così.",
         "Hello world! City, because, more, so."),

        ("Polish (Polski)",
         "Cześć świecie! Łódź, ąę, ćń, źż.",
         "Hello world! Lodz (city), various Polish letters."),

        ("Czech (Čeština)",
         "Ahoj světe! Počítač, čeština, řeč, žába.",
         "Hello world! Computer, Czech, speech, frog."),

        ("Turkish (Türkçe)",
         "Merhaba dünya! İstanbul, ğ, ş, ı (dotless i).",
         "Hello world! Istanbul, special Turkish letters."),

        ("Swedish (Svenska)",
         "Hej världen! Åke, älg, öl - svenska bokstäver.",
         "Hello world! Swedish letters."),

        ("Norwegian (Norsk)",
         "Hei verden! Æ, ø, å - norske bokstaver.",
         "Hello world! Norwegian letters."),

        ("Danish (Dansk)",
         "Hej verden! Æble, øl, å - danske bogstaver.",
         "Hello world! Danish letters."),

        // Cyrillic
        ("Russian (Русский)",
         "Привет, мир! Москва, Санкт-Петербург.",
         "Hello, world! Moscow, Saint Petersburg."),

        ("Ukrainian (Українська)",
         "Привіт, світ! Київ, Україна, ґ, є, і, ї.",
         "Hello, world! Kyiv, Ukraine, special letters."),

        ("Bulgarian (Български)",
         "Здравей, свят! София, България.",
         "Hello, world! Sofia, Bulgaria."),

        ("Serbian (Српски)",
         "Здраво, свете! Београд, Србија.",
         "Hello, world! Belgrade, Serbia."),

        // Greek
        ("Greek (Ελληνικά)",
         "Γειά σου κόσμε! Αθήνα, αλφάβητο: α β γ δ ε.",
         "Hello world! Athens, alphabet: alpha beta gamma delta epsilon."),

        // Hebrew (RTL)
        ("Hebrew (עברית)",
         "שלום עולם! ירושלים, ישראל.",
         "Hello world! Jerusalem, Israel."),

        // Arabic (RTL)
        ("Arabic (العربية)",
         "مرحبا بالعالم! القاهرة، مصر.",
         "Hello world! Cairo, Egypt."),

        // Special Characters & Symbols
        ("Math & Symbols",
         "∑ ∏ √ ∞ ≈ ≠ ≤ ≥ ± × ÷ ∫ ∂ ∇",
         "Sum, product, root, infinity, approximately, not equal, etc."),

        ("Currency",
         "€ £ ¥ ₹ ₽ ₩ ₪ $ ¢ ₿ (Bitcoin)",
         "Euro, Pound, Yen, Rupee, Ruble, Won, Shekel, Dollar, Cent, Bitcoin"),

        ("Arrows",
         "← → ↑ ↓ ↔ ↕ ⇐ ⇒ ⇔ ⇄ ↺ ↻",
         "Left, right, up, down, bidirectional, circular"),

        ("Symbols",
         "© ® ™ § ¶ † ‡ • … ‰ ′ ″ ℃ ℉ №",
         "Copyright, registered, trademark, section, paragraph, etc."),
    ];

    println!("Adding {} language samples...\n", samples.len());

    for (language, sample, translation) in &samples {
        // Language name
        let line = format!("{}: {}", language, sample);
        pdf.font_manager.record_text(font_index, &line);
        page.add_text_with_font(
            &line,
            x_margin,
            y,
            Length::from_pt(10.0),
            font_index,
        );
        y -= Length::from_pt(12.0);

        // Translation (smaller, indented)
        if !translation.is_empty() && *language != "--------" {
            let trans_line = format!("  → {}", translation);
            pdf.font_manager.record_text(font_index, &trans_line);
            page.add_text_with_font(
                &trans_line,
                x_margin + Length::from_mm(5.0),
                y,
                Length::from_pt(8.0),
                font_index,
            );
            y -= Length::from_pt(14.0);
        } else {
            y -= Length::from_pt(4.0);
        }

        // Check if we need a new page
        if y.to_pt() < 40.0 {
            println!("  Adding new page...");
            pdf.add_page(page);
            page = PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0));
            y = y_start;
        }
    }

    // Footer
    y = Length::from_mm(10.0);
    let footer = format!(
        "Generated by Apache FOP Rust - Font: {} - Date: 2026-02-16",
        font_source.split('/').last().unwrap_or("Unknown")
    );
    pdf.font_manager.record_text(font_index, &footer);
    page.add_text_with_font(
        &footer,
        x_margin,
        y,
        Length::from_pt(8.0),
        font_index,
    );

    pdf.add_page(page);

    // Generate PDF
    println!("\nGenerating PDF...");
    let pdf_bytes = pdf.to_bytes()?;

    // Save to file
    let output_path = "/tmp/i18n_showcase.pdf";
    fs::write(output_path, &pdf_bytes)?;

    println!("✓ PDF created: {} ({} KB)", output_path, pdf_bytes.len() / 1024);
    println!("\n✅ SUCCESS! Multilingual PDF with {} languages created.", samples.len());
    println!("   Open the PDF to see: {}", output_path);

    // Print language categories
    println!("\n📊 Language Coverage:");
    println!("   • CJK: Japanese, Traditional Chinese, Simplified Chinese, Korean");
    println!("   • Southeast Asian: Thai, Vietnamese");
    println!("   • European: German, French, Spanish, Portuguese, Italian, Polish,");
    println!("              Czech, Turkish, Swedish, Norwegian, Danish");
    println!("   • Cyrillic: Russian, Ukrainian, Bulgarian, Serbian");
    println!("   • Greek: Modern Greek");
    println!("   • RTL: Hebrew, Arabic");
    println!("   • Symbols: Math, Currency, Arrows, Special characters");

    Ok(())
}
