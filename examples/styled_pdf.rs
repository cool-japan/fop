//! Advanced example: Styled PDF with colors and multiple font sizes
//!
//! Demonstrates property extraction (colors, font sizes) and PDF rendering.

use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use fop_render::PdfRenderer;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fo_xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
                          page-width="210mm"
                          page-height="297mm"
                          margin="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="28pt" color="#0066CC" space-after="10pt">
        Apache FOP in Rust
      </fo:block>

      <fo:block font-size="16pt" color="#333333" space-after="6pt">
        A high-performance XSL-FO processor
      </fo:block>

      <fo:block font-size="12pt" color="#000000" space-after="4pt">
        This document demonstrates the layout engine capabilities:
      </fo:block>

      <fo:block font-size="10pt" color="#666666" space-after="3pt">
        • Text rendering with positioning
      </fo:block>

      <fo:block font-size="10pt" color="#666666" space-after="3pt">
        • Multiple font sizes (10pt, 12pt, 16pt, 28pt)
      </fo:block>

      <fo:block font-size="10pt" color="#666666" space-after="3pt">
        • Color support (hex colors)
      </fo:block>

      <fo:block font-size="10pt" color="#666666" space-after="12pt">
        • Vertical block stacking with spacing
      </fo:block>

      <fo:block font-size="14pt" color="#CC0000" space-after="6pt">
        Phase 2 Complete!
      </fo:block>

      <fo:block font-size="11pt" color="#006600">
        ✓ FO tree parsing ✓ Layout engine ✓ PDF rendering
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    println!("=== Styled PDF Generation Demo ===\n");

    // Parse XSL-FO
    println!("Parsing XSL-FO...");
    let builder = FoTreeBuilder::new();
    let cursor = Cursor::new(fo_xml.as_bytes());
    let arena = builder.parse(cursor)?;
    println!("  ✓ {} FO nodes", arena.len());

    // Layout
    println!("Running layout engine...");
    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&arena)?;
    println!("  ✓ {} areas generated", area_tree.len());

    // Render to PDF
    println!("Rendering to PDF...");
    let renderer = PdfRenderer::new();
    let pdf_doc = renderer.render(&area_tree)?;

    // Write PDF
    let pdf_bytes = pdf_doc.to_bytes()?;
    let output_path = "/tmp/styled_fop.pdf";
    std::fs::write(output_path, pdf_bytes)?;

    println!("\n✓ PDF generated: {}", output_path);
    println!("  Size: {} bytes", std::fs::metadata(output_path)?.len());
    println!("  Pages: {}", pdf_doc.pages.len());

    println!("\nTry: pdftotext {} -", output_path);
    println!("Or open in a PDF viewer to see the styled output.");

    Ok(())
}
