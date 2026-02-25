//! SVG output demonstration
//!
//! This example demonstrates rendering FO documents to SVG format.

use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use fop_render::SvgRenderer;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // XSL-FO document with various features to demonstrate SVG rendering
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
      <!-- Title with large font -->
      <fo:block font-size="24pt" color="#0066cc" space-after="12pt">
        SVG Output Demo
      </fo:block>

      <!-- Subtitle -->
      <fo:block font-size="14pt" color="#666666" space-after="18pt">
        Demonstrating SVG rendering capabilities
      </fo:block>

      <!-- Text with background color -->
      <fo:block font-size="12pt" background-color="#f0f0f0"
                padding-top="6pt" padding-bottom="6pt"
                padding-left="6pt" padding-right="6pt"
                space-after="12pt">
        This block has a gray background color.
      </fo:block>

      <!-- Block with border -->
      <fo:block font-size="12pt"
                border-top-width="1pt" border-top-style="solid" border-top-color="#000000"
                border-right-width="1pt" border-right-style="solid" border-right-color="#000000"
                border-bottom-width="1pt" border-bottom-style="solid" border-bottom-color="#000000"
                border-left-width="1pt" border-left-style="solid" border-left-color="#000000"
                padding-top="6pt" padding-bottom="6pt"
                padding-left="6pt" padding-right="6pt"
                space-after="12pt">
        This block has a black border.
      </fo:block>

      <!-- Colored text -->
      <fo:block font-size="12pt" space-after="12pt">
        <fo:inline color="#ff0000">Red text</fo:inline>,
        <fo:inline color="#00ff00">green text</fo:inline>, and
        <fo:inline color="#0000ff">blue text</fo:inline>.
      </fo:block>

      <!-- Leader examples -->
      <fo:block font-size="12pt" space-after="6pt">
        Table of Contents
      </fo:block>

      <fo:block font-size="10pt" text-align-last="justify" space-after="3pt">
        Chapter 1<fo:leader leader-pattern="dots"/>10
      </fo:block>

      <fo:block font-size="10pt" text-align-last="justify" space-after="3pt">
        Chapter 2<fo:leader leader-pattern="dots"/>25
      </fo:block>

      <!-- Horizontal rule -->
      <fo:block space-before="12pt" space-after="12pt">
        <fo:leader leader-pattern="rule" rule-thickness="0.5pt"/>
      </fo:block>

      <!-- Different border styles -->
      <fo:block font-size="10pt" space-after="6pt">
        Border Styles:
      </fo:block>

      <fo:block font-size="10pt"
                border-top-width="2pt" border-top-style="solid" border-top-color="#cc0000"
                padding-top="6pt" padding-bottom="6pt"
                space-after="6pt">
        Solid border (red)
      </fo:block>

      <fo:block font-size="10pt"
                border-top-width="2pt" border-top-style="dashed" border-top-color="#0000cc"
                padding-top="6pt" padding-bottom="6pt"
                space-after="6pt">
        Dashed border (blue)
      </fo:block>

      <fo:block font-size="10pt"
                border-top-width="2pt" border-top-style="dotted" border-top-color="#00cc00"
                padding-top="6pt" padding-bottom="6pt"
                space-after="12pt">
        Dotted border (green)
      </fo:block>

      <!-- Final note -->
      <fo:block font-size="10pt" color="#999999"
                border-left-width="3pt" border-left-style="solid" border-left-color="#0066cc"
                padding-left="12pt" space-before="18pt">
        SVG output preserves all formatting from the XSL-FO source,
        including colors, borders, backgrounds, and text styling.
        The output can be viewed in any web browser or SVG-compatible viewer.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    println!("=== SVG Output Demo ===");
    println!("Demonstrating FO → Layout → SVG Pipeline\n");

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

    // Step 3: Render to SVG
    println!("\nStep 3: Rendering to SVG...");
    let renderer = SvgRenderer::new();
    let svg_content = renderer.render_to_svg(&area_tree)?;

    // Count pages
    let page_count = area_tree.iter()
        .filter(|(_, node)| matches!(node.area.area_type, fop_layout::AreaType::Page))
        .count();

    println!("  ✓ Generated SVG document");
    println!("  ✓ Pages: {}", page_count);
    println!("  ✓ SVG size: {} bytes", svg_content.len());

    // Write to file
    let output_path = "/tmp/svg_demo.svg";
    std::fs::write(output_path, svg_content)?;

    println!("\n=== SUCCESS ===");
    println!("SVG written to: {}", output_path);
    println!("\nYou can open this file in:");
    println!("  - Any web browser (Firefox, Chrome, Safari, etc.)");
    println!("  - Inkscape or other SVG editors");
    println!("  - Image viewers that support SVG");
    println!("\nThe SVG format preserves all formatting and can be scaled to any size.");

    Ok(())
}
