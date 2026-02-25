//! Comprehensive demo showcasing all Phase 3 features
//!
//! Demonstrates:
//! - Font metrics (accurate text measurement)
//! - Multiple font sizes and colors
//! - Border rendering
//! - List markers (automatic numbering)
//! - Table layout structure
//! - Multi-level nested content

use fop_core::FoTreeBuilder;
use fop_layout::{KnuthPlassBreaker, LayoutEngine, ListLayout, ListMarkerStyle};
use fop_render::{PdfGraphics, PdfRenderer};
use fop_types::{Color, FontRegistry, Length};
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Apache FOP Rust - Comprehensive Feature Demo ===\n");

    // Demonstrate font metrics
    demo_font_metrics();

    // Demonstrate Knuth-Plass line breaking
    demo_knuth_plass();

    // Demonstrate list markers
    demo_list_markers();

    // Generate full PDF with all features
    demo_full_pdf()?;

    Ok(())
}

/// Demonstrate accurate font metrics
fn demo_font_metrics() {
    println!("1. Font Metrics Demonstration");
    println!("   Comparing approximation vs real widths:\n");

    let registry = FontRegistry::new();
    let helvetica = registry
        .get("Helvetica")
        .expect("bench/example: should succeed");
    let times = registry
        .get("Times-Roman")
        .expect("bench/example: should succeed");

    let font_size = Length::from_pt(12.0);
    let test_strings = ["Hello", "WWW", "iii", "The quick brown fox"];

    for text in &test_strings {
        let helv_width = helvetica.measure_text(text, font_size);
        let times_width = times.measure_text(text, font_size);
        let approx = font_size.to_pt() * 0.6 * text.len() as f64;

        println!("   \"{}\":", text);
        println!("     Helvetica: {:.2}pt", helv_width.to_pt());
        println!("     Times:     {:.2}pt", times_width.to_pt());
        println!(
            "     Approx:    {:.2}pt (error: {:.1}%)\n",
            approx,
            ((approx - helv_width.to_pt()) / helv_width.to_pt() * 100.0).abs()
        );
    }
}

/// Demonstrate Knuth-Plass line breaking
fn demo_knuth_plass() {
    println!("\n2. Knuth-Plass Line Breaking");
    println!("   Optimal breaks for professional typography:\n");

    let breaker = KnuthPlassBreaker::new(Length::from_pt(200.0)).with_tolerance(2);

    let text = "This is a demonstration of the Knuth-Plass line breaking algorithm, \
                which produces optimal line breaks by considering the entire paragraph \
                rather than breaking greedily line by line.";

    let traits = fop_layout::area::TraitSet {
        font_size: Some(Length::from_pt(12.0)),
        font_family: Some("Helvetica".to_string()),
        ..Default::default()
    };

    let lines = breaker.break_text(text, &traits);

    println!("   Text broken into {} lines:", lines.len());
    for (i, line) in lines.iter().enumerate() {
        println!("   [{}] {}", i + 1, line);
    }
}

/// Demonstrate list markers
fn demo_list_markers() {
    println!("\n3. List Marker Generation");
    println!("   Automatic numbering in 9 styles:\n");

    let list = ListLayout::new(Length::from_pt(400.0));

    let styles = vec![
        (ListMarkerStyle::Disc, "Disc"),
        (ListMarkerStyle::Circle, "Circle"),
        (ListMarkerStyle::Square, "Square"),
        (ListMarkerStyle::Decimal, "Decimal"),
        (ListMarkerStyle::LowerAlpha, "Lower Alpha"),
        (ListMarkerStyle::UpperAlpha, "Upper Alpha"),
        (ListMarkerStyle::LowerRoman, "Lower Roman"),
        (ListMarkerStyle::UpperRoman, "Upper Roman"),
    ];

    for (style, name) in styles {
        print!("   {}: ", name);
        for i in 1..=5 {
            print!("{} ", list.generate_marker(i, style));
        }
        println!();
    }

    // Special cases
    println!("\n   Special cases:");
    println!(
        "     Alpha overflow: 26={}, 27={}, 52={}",
        list.generate_marker(26, ListMarkerStyle::LowerAlpha),
        list.generate_marker(27, ListMarkerStyle::LowerAlpha),
        list.generate_marker(52, ListMarkerStyle::LowerAlpha)
    );
    println!(
        "     Roman: 1994={}, 2023={}",
        list.generate_marker(1994, ListMarkerStyle::LowerRoman),
        list.generate_marker(2023, ListMarkerStyle::UpperRoman)
    );
}

/// Generate full PDF with all features
fn demo_full_pdf() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n4. Generating Comprehensive PDF");
    println!("   Features: fonts, colors, borders, lists, tables\n");

    let fo_xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
                          page-width="210mm"
                          page-height="297mm"
                          margin="15mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="24pt" color="#0066CC" space-after="8pt">
        Apache FOP in Rust
      </fo:block>

      <fo:block font-size="14pt" color="#333333" space-after="6pt">
        Phase 3 Feature Showcase
      </fo:block>

      <fo:block font-size="12pt" color="#000000" space-after="4pt">
        This document demonstrates the advanced features implemented in Phase 3:
      </fo:block>

      <fo:block font-size="11pt" color="#000000" space-after="3pt">
        ✓ Accurate font metrics (Helvetica and Times-Roman)
      </fo:block>

      <fo:block font-size="11pt" color="#000000" space-after="3pt">
        ✓ Knuth-Plass line breaking (optimal paragraph layout)
      </fo:block>

      <fo:block font-size="11pt" color="#000000" space-after="3pt">
        ✓ Table layout algorithms (fixed, proportional, auto widths)
      </fo:block>

      <fo:block font-size="11pt" color="#000000" space-after="3pt">
        ✓ List layout with 9 marker styles (bullets, numbers, letters, roman)
      </fo:block>

      <fo:block font-size="11pt" color="#000000" space-after="3pt">
        ✓ Border rendering (PDF graphics operators)
      </fo:block>

      <fo:block font-size="11pt" color="#000000" space-after="8pt">
        ✓ Image support structure (PNG/JPEG with aspect ratio)
      </fo:block>

      <fo:block font-size="14pt" color="#CC0000" space-after="4pt">
        Text Measurement Accuracy
      </fo:block>

      <fo:block font-size="10pt" color="#666666" space-after="6pt">
        Real font metrics enable precise text positioning. The word "Hello"
        measures 27.3pt in Helvetica at 12pt, not the 36pt approximation.
        Wide characters like "WWW" correctly measure 33.9pt, while narrow
        characters like "iii" measure just 8.0pt.
      </fo:block>

      <fo:block font-size="14pt" color="#CC0000" space-after="4pt">
        Professional Typography
      </fo:block>

      <fo:block font-size="10pt" color="#666666" space-after="6pt">
        The Knuth-Plass algorithm optimizes line breaks across entire paragraphs,
        avoiding rivers of whitespace and producing balanced, visually pleasing
        text blocks. This is the same algorithm used in TeX and professional
        typesetting systems.
      </fo:block>

      <fo:block font-size="12pt" color="#006600" space-after="3pt">
        Phase 3: Complete ✓
      </fo:block>

      <fo:block font-size="10pt" color="#000000">
        All core layout algorithms implemented. Ready for Phase 4 integration.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    // Parse FO
    println!("   Parsing XSL-FO...");
    let builder = FoTreeBuilder::new();
    let cursor = Cursor::new(fo_xml.as_bytes());
    let arena = builder.parse(cursor)?;
    println!("     ✓ {} FO nodes", arena.len());

    // Layout
    println!("   Running layout engine...");
    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&arena)?;
    println!("     ✓ {} areas generated", area_tree.len());

    // Render to PDF
    println!("   Rendering to PDF...");
    let renderer = PdfRenderer::new();
    let mut pdf_doc = renderer.render(&area_tree)?;

    // Add metadata
    pdf_doc.info.title = Some("Apache FOP Rust - Phase 3 Demo".to_string());
    pdf_doc.info.author = Some("FOP Rust Team".to_string());
    pdf_doc.info.subject = Some("Comprehensive feature demonstration".to_string());

    // Write PDF
    let pdf_bytes = pdf_doc.to_bytes()?;
    let output_path = "/tmp/comprehensive_fop.pdf";
    std::fs::write(output_path, pdf_bytes)?;

    println!("\n✓ PDF generated: {}", output_path);
    println!("  Size: {} bytes", std::fs::metadata(output_path)?.len());
    println!("  Pages: {}", pdf_doc.pages.len());

    // Demonstrate graphics operations
    println!("\n5. Graphics Operations (borders, rectangles, lines)");
    demo_graphics();

    println!("\n=== Demo Complete ===");
    println!("Try: pdftotext {} -", output_path);
    println!("Or open in a PDF viewer to see all features.");

    Ok(())
}

/// Demonstrate graphics operations
fn demo_graphics() {
    let mut graphics = PdfGraphics::new();

    // Set colors and draw
    graphics
        .set_stroke_color(Color::RED)
        .expect("bench/example: should succeed");
    graphics
        .set_line_width(Length::from_pt(2.0))
        .expect("bench/example: should succeed");
    graphics
        .draw_rectangle(
            Length::from_pt(10.0),
            Length::from_pt(10.0),
            Length::from_pt(100.0),
            Length::from_pt(50.0),
        )
        .expect("bench/example: should succeed");

    graphics
        .set_fill_color(Color::BLUE)
        .expect("bench/example: should succeed");
    graphics
        .fill_rectangle(
            Length::from_pt(120.0),
            Length::from_pt(10.0),
            Length::from_pt(50.0),
            Length::from_pt(50.0),
        )
        .expect("bench/example: should succeed");

    // Border rendering
    use fop_layout::area::BorderStyle;
    graphics
        .draw_borders(
            Length::from_pt(200.0),
            Length::from_pt(10.0),
            Length::from_pt(100.0),
            Length::from_pt(50.0),
            [
                Length::from_pt(1.0),
                Length::from_pt(2.0),
                Length::from_pt(3.0),
                Length::from_pt(4.0),
            ],
            [Color::RED, Color::GREEN, Color::BLUE, Color::BLACK],
            [
                BorderStyle::Solid,
                BorderStyle::Dashed,
                BorderStyle::Dotted,
                BorderStyle::Solid,
            ],
        )
        .expect("bench/example: should succeed");

    println!(
        "   Generated {} bytes of graphics operators",
        graphics.content().len()
    );
    println!("   Operations: rectangles, fills, borders with varying widths");
}
