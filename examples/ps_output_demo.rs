//! PostScript output demonstration
//!
//! This example demonstrates rendering FO documents to PostScript format.

use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use fop_render::PsRenderer;
use std::fs;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // XSL-FO document with various features to demonstrate PostScript rendering
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
        PostScript Output Demo
      </fo:block>

      <!-- Subtitle -->
      <fo:block font-size="14pt" color="#666666" space-after="18pt">
        Demonstrating PostScript Level 2 rendering capabilities
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

      <!-- Multi-page content -->
      <fo:block font-size="12pt" space-before="24pt">
        This is a simple example demonstrating PostScript generation
        from XSL-FO using the Apache FOP Rust implementation.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>
"##;

    println!("PostScript Output Demo");
    println!("======================\n");

    // Step 1: Parse XSL-FO document
    println!("1. Parsing XSL-FO document...");
    let builder = FoTreeBuilder::new();
    let cursor = Cursor::new(fo_xml.as_bytes());
    let arena = builder.parse(cursor)?;
    println!("   ✓ Parsed {} FO nodes", arena.len());

    // Step 2: Layout processing
    println!("\n2. Processing layout...");
    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&arena)?;
    println!("   ✓ Created {} areas", area_tree.len());

    // Count pages
    let page_count = area_tree
        .iter()
        .filter(|(_, node)| matches!(node.area.area_type, fop_layout::AreaType::Page))
        .count();
    println!("   ✓ Generated {} page(s)", page_count);

    // Step 3: Render to PostScript
    println!("\n3. Rendering to PostScript...");
    let renderer = PsRenderer::new();
    let ps_content = renderer.render_to_ps(&area_tree)?;
    println!("   ✓ PostScript generated ({} bytes)", ps_content.len());

    // Step 4: Save to file
    let output_path = "/tmp/fop_demo_output.ps";
    fs::write(output_path, &ps_content)?;
    println!("\n4. Output saved to: {}", output_path);

    // Show preview of PostScript content
    println!("\n5. PostScript preview (first 50 lines):");
    println!("   {}", "=".repeat(70));
    for (i, line) in ps_content.lines().enumerate() {
        if i >= 50 {
            println!("   ... ({} more lines)", ps_content.lines().count() - 50);
            break;
        }
        println!("   {}", line);
    }
    println!("   {}", "=".repeat(70));

    println!("\n✓ Success! PostScript file generated.");
    println!("\nTo view the PostScript file:");
    println!("  - Linux:   evince {}", output_path);
    println!("  - macOS:   open {}", output_path);
    println!("  - Convert to PDF: ps2pdf {} /tmp/output.pdf", output_path);

    Ok(())
}
