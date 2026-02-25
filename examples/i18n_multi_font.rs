//! Multi-Font International Language Support
//!
//! Uses appropriate fonts for each script family to ensure proper rendering.
//! This demonstrates the CORRECT way to handle multilingual PDFs with automatic font fallback.

use fop_render::PdfDocument;
use fop_render::pdf::document::PdfPage;
use fop_types::Length;
use std::fs;
use std::path::Path;

fn find_font(paths: &[&str]) -> Option<(Vec<u8>, String)> {
    for path in paths {
        if let Ok(data) = fs::read(path) {
            return Some((data, path.to_string()));
        }
    }
    None
}

/// A run of text that should be rendered with a specific font
#[derive(Debug, Clone)]
struct TextRun {
    font_index: usize,
    text: String,
    #[allow(dead_code)]
    x_offset: f64,
}

/// Automatic font fallback helper
///
/// This follows the approach from Apache FOP's FontSelector.java:
/// - Iterates through available fonts in priority order
/// - For each character, finds the first font that can render it
/// - Groups consecutive characters using the same font into "runs"
/// - Returns runs WITHOUT cumulative x_offset (caller handles positioning)
#[allow(dead_code)]
fn select_fonts_for_text(
    text: &str,
    font_priorities: &[usize],
    pdf: &PdfDocument,
    _font_size_pt: f64,
) -> Vec<TextRun> {
    let mut runs = Vec::new();
    let mut current_font_idx: Option<usize> = None;
    let mut current_text = String::new();

    for c in text.chars() {
        // Find the first font that can render this character
        let mut selected_font_idx = font_priorities[0]; // Default to first font

        for &font_idx in font_priorities {
            if let Some(font) = pdf.font_manager.get_font(font_idx) {
                // Parse the font data to check if it can render this character
                if let Ok(face) = ttf_parser::Face::parse(&font.font_data, 0) {
                    if face.glyph_index(c).is_some() {
                        selected_font_idx = font_idx;
                        break;
                    }
                }
            }
        }

        // If font changes, finalize the current run and start a new one
        if Some(selected_font_idx) != current_font_idx {
            if !current_text.is_empty() {
                if let Some(font_idx) = current_font_idx {
                    runs.push(TextRun {
                        font_index: font_idx,
                        text: current_text.clone(),
                        x_offset: 0.0, // Not used - caller tracks position
                    });
                }
                current_text.clear();
            }
            current_font_idx = Some(selected_font_idx);
        }

        current_text.push(c);
    }

    // Finalize the last run
    if !current_text.is_empty() {
        if let Some(font_idx) = current_font_idx {
            runs.push(TextRun {
                font_index: font_idx,
                text: current_text,
                x_offset: 0.0, // Not used - caller tracks position
            });
        }
    }

    runs
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating Multilingual PDF with Multiple Fonts...\n");

    // Try to find fonts for different script families
    let fonts = vec![
        ("CJK", vec![
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttf",
        ]),
        ("Latin", vec![
            "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
        ]),
        ("Thai", vec![
            "/usr/share/fonts/truetype/noto/NotoSansThai-Regular.ttf",
            "/usr/share/fonts/truetype/tlwg/Garuda.ttf",
        ]),
        ("Arabic", vec![
            "/usr/share/fonts/truetype/noto/NotoSansArabic-Regular.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ]),
        ("Hebrew", vec![
            "/usr/share/fonts/truetype/noto/NotoSansHebrew-Regular.ttf",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        ]),
    ];

    let mut pdf = PdfDocument::new();
    pdf.info.title = Some("Multilingual PDF - Multiple Fonts".to_string());
    pdf.info.author = Some("Apache FOP Rust".to_string());

    let mut font_indices = std::collections::HashMap::new();

    println!("Loading fonts...");
    for (family, paths) in &fonts {
        if let Some((data, path)) = find_font(paths) {
            let index = pdf.embed_font(data)?;
            font_indices.insert(family.to_string(), index);
            println!("  ✓ {}: {} (index: {})", family,
                     Path::new(&path).file_name().expect("bench/example: should succeed").to_str().expect("bench/example: should succeed"), index);
        } else {
            println!("  ⚠ {}: Not found (some languages may not render)", family);
        }
    }

    if font_indices.is_empty() {
        return Err("No fonts found! Please install fonts:\n\
                    sudo apt install fonts-noto-cjk fonts-noto fonts-noto-extra".into());
    }

    println!();

    // Create page
    let mut page = PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0));
    let y_start = Length::from_mm(280.0);
    let x_margin = Length::from_mm(15.0);
    let line_height = Length::from_pt(16.0);
    let mut y = y_start;

    // Helper to add text with automatic font fallback
    let add_lang = |page: &mut PdfPage,
                    pdf: &mut PdfDocument,
                    y: &mut Length,
                    lang: &str,
                    text: &str,
                    translation: &str,
                    font_family: &str| {
        if let Some(&script_font_idx) = font_indices.get(font_family) {
            // Build font priority list: primary script font, then Latin, then others
            let latin_font_idx = font_indices.get("Latin")
                .or_else(|| font_indices.get("CJK"))
                .copied()
                .unwrap_or(script_font_idx);

            let mut font_priorities = vec![script_font_idx];
            if latin_font_idx != script_font_idx {
                font_priorities.push(latin_font_idx);
            }
            // Add all other fonts as fallbacks
            for &idx in font_indices.values() {
                if idx != script_font_idx && idx != latin_font_idx {
                    font_priorities.push(idx);
                }
            }

            // Build complete line of text to render
            let full_line = format!("{}: {}", lang, text);

            // Use automatic font fallback to select fonts for each character
            let runs = select_fonts_for_text(&full_line, &font_priorities, pdf, 11.0);

            // Render each run with its appropriate font
            let mut current_x = x_margin;
            for run in runs {
                pdf.font_manager.record_text(run.font_index, &run.text);
                page.add_text_with_font(
                    &run.text,
                    current_x,
                    *y,
                    Length::from_pt(11.0),
                    run.font_index
                );

                // Calculate width for next run
                if let Some(font) = pdf.font_manager.get_font(run.font_index) {
                    if let Ok(face) = ttf_parser::Face::parse(&font.font_data, 0) {
                        let mut width = 0u32;
                        for c in run.text.chars() {
                            if let Some(glyph_id) = face.glyph_index(c) {
                                width += face.glyph_hor_advance(glyph_id).unwrap_or(font.units_per_em / 2) as u32;
                            }
                        }
                        current_x += Length::from_pt((width as f64 / font.units_per_em as f64) * 11.0);
                    }
                }
            }

            *y -= Length::from_pt(13.0);

            // Translation uses automatic font fallback too
            if !translation.is_empty() {
                let trans = format!("  → {}", translation);
                let trans_runs = select_fonts_for_text(&trans, &font_priorities, pdf, 9.0);

                let mut trans_x = x_margin + Length::from_mm(5.0);
                for run in trans_runs {
                    pdf.font_manager.record_text(run.font_index, &run.text);
                    page.add_text_with_font(
                        &run.text,
                        trans_x,
                        *y,
                        Length::from_pt(9.0),
                        run.font_index
                    );

                    // Calculate width for next run
                    if let Some(font) = pdf.font_manager.get_font(run.font_index) {
                        if let Ok(face) = ttf_parser::Face::parse(&font.font_data, 0) {
                            let mut width = 0u32;
                            for c in run.text.chars() {
                                if let Some(glyph_id) = face.glyph_index(c) {
                                    width += face.glyph_hor_advance(glyph_id).unwrap_or(font.units_per_em / 2) as u32;
                                }
                            }
                            trans_x += Length::from_pt((width as f64 / font.units_per_em as f64) * 9.0);
                        }
                    }
                }

                *y -= Length::from_pt(15.0);
            }
        } else {
            let warning = format!("{}: [Font not available]", lang);
            if let Some(&font_idx) = font_indices.values().next() {
                pdf.font_manager.record_text(font_idx, &warning);
                page.add_text_with_font(&warning, x_margin, *y, Length::from_pt(11.0), font_idx);
                *y -= Length::from_pt(15.0);
            }
        }
    };

    // Title
    if let Some(&font_idx) = font_indices.get("Latin").or_else(|| font_indices.values().next()) {
        let title = "Apache FOP Rust - Multilingual Support (Multiple Fonts)";
        pdf.font_manager.record_text(font_idx, title);
        page.add_text_with_font(title, x_margin, y, Length::from_pt(14.0), font_idx);
        y -= line_height * 2;
    }

    // CJK Languages
    add_lang(&mut page, &mut pdf, &mut y,
             "Japanese (日本語)",
             "こんにちは、世界！桜が咲く春の日。",
             "Hello, world! Spring day when cherry blossoms bloom.",
             "CJK");

    add_lang(&mut page, &mut pdf, &mut y,
             "Traditional Chinese (繁體中文)",
             "您好，世界！繁體字是傳統漢字的書寫系統。",
             "Hello, world! Traditional Chinese is the traditional character system.",
             "CJK");

    add_lang(&mut page, &mut pdf, &mut y,
             "Simplified Chinese (简体中文)",
             "你好，世界！简体字是简化后的汉字系统。",
             "Hello, world! Simplified Chinese is the simplified character system.",
             "CJK");

    add_lang(&mut page, &mut pdf, &mut y,
             "Korean (한국어)",
             "안녕하세요, 세계! 한글은 한국의 문자입니다.",
             "Hello, world! Hangul is the Korean alphabet.",
             "CJK");

    // Thai
    add_lang(&mut page, &mut pdf, &mut y,
             "Thai (ภาษาไทย)",
             "สวัสดีชาวโลก! ภาษาไทยเป็นภาษาราชการของประเทศไทย",
             "Hello, world! Thai is the official language of Thailand.",
             "Thai");

    // European languages with Latin font
    add_lang(&mut page, &mut pdf, &mut y,
             "German (Deutsch)",
             "Guten Tag! Äpfel, Öl, Übung, ß (Eszett).",
             "Good day! Apples, oil, practice, sharp S.",
             "Latin");

    add_lang(&mut page, &mut pdf, &mut y,
             "French (Français)",
             "Bonjour le monde! Café, naïve, château, œuvre.",
             "Hello world! Coffee, naive, castle, work.",
             "Latin");

    add_lang(&mut page, &mut pdf, &mut y,
             "Spanish (Español)",
             "¡Hola mundo! Mañana, señor, niño, ¿Cómo estás?",
             "Hello world! Tomorrow, sir, boy, How are you?",
             "Latin");

    add_lang(&mut page, &mut pdf, &mut y,
             "Polish (Polski)",
             "Cześć świecie! Łódź, ąę, ćń, źż.",
             "Hello world! Lodz (city), various Polish letters.",
             "Latin");

    add_lang(&mut page, &mut pdf, &mut y,
             "Czech (Čeština)",
             "Ahoj světe! Počítač, čeština, řeč, žába.",
             "Hello world! Computer, Czech, speech, frog.",
             "Latin");

    add_lang(&mut page, &mut pdf, &mut y,
             "Turkish (Türkçe)",
             "Merhaba dünya! İstanbul, ğ, ş, ı (dotless i).",
             "Hello world! Istanbul, special Turkish letters.",
             "Latin");

    // Cyrillic - use Latin font (NotoSans has full Cyrillic support)
    add_lang(&mut page, &mut pdf, &mut y,
             "Russian (Русский)",
             "Привет, мир! Москва, Санкт-Петербург.",
             "Hello, world! Moscow, Saint Petersburg.",
             "Latin");

    add_lang(&mut page, &mut pdf, &mut y,
             "Ukrainian (Українська)",
             "Привіт, світ! Київ, Україна.",
             "Hello, world! Kyiv, Ukraine.",
             "Latin");

    // Greek - use Latin font
    add_lang(&mut page, &mut pdf, &mut y,
             "Greek (Ελληνικά)",
             "Γειά σου κόσμε! Αθήνα, αλφάβητο.",
             "Hello world! Athens, alphabet.",
             "Latin");

    // Hebrew
    add_lang(&mut page, &mut pdf, &mut y,
             "Hebrew (עברית)",
             "שלום עולם! ירושלים, ישראל.",
             "Hello world! Jerusalem, Israel.",
             "Hebrew");

    // Arabic
    add_lang(&mut page, &mut pdf, &mut y,
             "Arabic (العربية)",
             "مرحبا بالعالم! القاهرة، مصر.",
             "Hello world! Cairo, Egypt.",
             "Arabic");

    // Add footer
    y = Length::from_mm(10.0);
    if let Some(&font_idx) = font_indices.get("Latin").or_else(|| font_indices.values().next()) {
        let footer = format!("Generated by Apache FOP Rust - {} fonts embedded - Date: 2026-02-16",
                           font_indices.len());
        pdf.font_manager.record_text(font_idx, &footer);
        page.add_text_with_font(&footer, x_margin, y, Length::from_pt(8.0), font_idx);
    }

    pdf.add_page(page);

    // Generate PDF
    println!("Generating PDF with {} embedded fonts...", font_indices.len());
    let pdf_bytes = pdf.to_bytes()?;

    // Save
    let output_path = "/tmp/i18n_multi_font.pdf";
    fs::write(output_path, &pdf_bytes)?;

    println!("✓ PDF created: {} ({} KB)", output_path, pdf_bytes.len() / 1024);
    println!("\n✅ SUCCESS! Multilingual PDF with proper fonts created.");
    println!("   Open the PDF to verify: {}", output_path);

    println!("\n📊 Fonts Used:");
    for (family, &index) in &font_indices {
        println!("   • {}: Font index {}", family, index);
    }

    Ok(())
}
