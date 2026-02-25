//! Text extraction demonstration
//!
//! This example demonstrates extracting plain text from FO documents for
//! accessibility and text analysis purposes.

use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use fop_render::TextRenderer;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // XSL-FO document with various text content
    let fo_xml = r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
                          page-width="210mm"
                          page-height="297mm"
                          margin-top="20mm"
                          margin-bottom="20mm"
                          margin-left="25mm"
                          margin-right="25mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <!-- Title -->
      <fo:block font-size="24pt" color="#0066cc" space-after="12pt">
        Text Extraction Demo
      </fo:block>

      <!-- Subtitle -->
      <fo:block font-size="14pt" color="#666666" space-after="18pt">
        Extracting plain text for accessibility
      </fo:block>

      <!-- Introduction -->
      <fo:block font-size="12pt" space-after="12pt">
        This example demonstrates how the text renderer extracts plain text
        content from XSL-FO documents while preserving document structure.
      </fo:block>

      <!-- Section header -->
      <fo:block font-size="14pt" font-weight="bold" space-before="18pt" space-after="12pt">
        Features
      </fo:block>

      <!-- List items (simulated with blocks) -->
      <fo:block font-size="12pt" space-after="6pt">
        1. Preserves paragraph structure with line breaks
      </fo:block>

      <fo:block font-size="12pt" space-after="6pt">
        2. Handles multiple pages with page separators
      </fo:block>

      <fo:block font-size="12pt" space-after="6pt">
        3. Extracts text in reading order
      </fo:block>

      <fo:block font-size="12pt" space-after="12pt">
        4. Provides image placeholders for non-text content
      </fo:block>

      <!-- Table of contents example -->
      <fo:block font-size="14pt" font-weight="bold" space-before="18pt" space-after="12pt">
        Table of Contents
      </fo:block>

      <fo:block font-size="10pt" space-after="3pt">
        Chapter 1: Introduction
      </fo:block>

      <fo:block font-size="10pt" space-after="3pt">
        Chapter 2: Getting Started
      </fo:block>

      <fo:block font-size="10pt" space-after="12pt">
        Chapter 3: Advanced Topics
      </fo:block>

      <!-- Quote block -->
      <fo:block font-size="11pt" font-style="italic"
                border-left-width="3pt" border-left-style="solid" border-left-color="#0066cc"
                padding-left="12pt" space-before="18pt" space-after="18pt">
        "Text extraction is essential for accessibility tools, search engines,
        and content analysis applications."
      </fo:block>

      <!-- Conclusion -->
      <fo:block font-size="12pt" space-after="12pt">
        The text renderer strips away all formatting and styling information,
        leaving only the raw text content in a readable format suitable for
        screen readers and text processing tools.
      </fo:block>

      <!-- Final note -->
      <fo:block font-size="10pt" color="#999999" space-before="18pt">
        Output format: Plain text with line breaks and page separators
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    println!("=== Text Extraction Demo ===");
    println!("Demonstrating FO → Layout → Text Pipeline\n");

    // Step 1: Parse XSL-FO
    println!("Step 1: Parsing XSL-FO document...");
    let builder = FoTreeBuilder::new();
    let cursor = Cursor::new(fo_xml.as_bytes());
    let arena = builder.parse(cursor)?;
    println!("  ✓ Parsed {} FO nodes", arena.len());

    // Step 2: Run layout engine
    println!("\nStep 2: Running layout engine...");
    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&arena)?;
    println!("  ✓ Generated {} areas", area_tree.len());

    // Step 3: Render to plain text
    println!("\nStep 3: Extracting text content...");
    let renderer = TextRenderer::new();
    let text_content = renderer.render_to_text(&area_tree)?;

    // Count pages
    let page_count = area_tree.iter()
        .filter(|(_, node)| matches!(node.area.area_type, fop_layout::AreaType::Page))
        .count();

    println!("  ✓ Extracted text content");
    println!("  ✓ Pages: {}", page_count);
    println!("  ✓ Text size: {} bytes", text_content.len());
    println!("  ✓ Lines: {}", text_content.lines().count());

    // Write to file
    let output_path = "/tmp/text_extraction_demo.txt";
    std::fs::write(output_path, &text_content)?;

    println!("\n=== SUCCESS ===");
    println!("Text written to: {}", output_path);
    println!("\n--- Extracted Text Preview ---");
    println!("{}", text_content);
    println!("--- End of Preview ---\n");

    println!("Use cases for text extraction:");
    println!("  - Screen readers for accessibility");
    println!("  - Content indexing for search engines");
    println!("  - Text analysis and data mining");
    println!("  - Quick content preview without rendering");
    println!("  - Copy-paste friendly output");

    Ok(())
}
