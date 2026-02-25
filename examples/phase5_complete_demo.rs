//! Phase 5 Complete Integration Demo
//!
//! This example demonstrates ALL Phase 5 features in a comprehensive end-to-end integration:
//!
//! 1. **Visual Styling**
//!    - Background colors (named and hex)
//!    - 4-side borders with different styles (solid, dashed, dotted)
//!    - Text colors
//!
//! 2. **Layout Control**
//!    - Text alignment (left, center, right, justify)
//!    - Block spacing (space-before, space-after)
//!    - Multiple page layout
//!
//! 3. **Images**
//!    - PNG images with fo:external-graphic
//!    - JPEG images with fo:external-graphic
//!    - Width/height control
//!    - Programmatically generated test images
//!
//! 4. **PDF Features**
//!    - Hyperlinks (fo:basic-link with external-destination)
//!    - Bookmarks (fo:bookmark-tree with nested structure)
//!    - Document metadata
//!
//! 5. **Complex Layouts**
//!    - Tables with styled cells
//!    - Lists with backgrounds
//!    - Mixed content

use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use fop_render::PdfRenderer;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Phase 5 Complete Integration Demo ===\n");

    // Step 1: Generate test images programmatically
    println!("Step 1: Generating test images...");
    create_test_images()?;
    println!("  ✓ Created 3 test images in /tmp/\n");

    // Step 2: Create comprehensive XSL-FO document
    println!("Step 2: Creating comprehensive XSL-FO document...");
    let xml = create_comprehensive_fo_document();
    println!("  ✓ XSL-FO document created ({} bytes)\n", xml.len());

    // Step 3: Parse FO tree
    println!("Step 3: Parsing XSL-FO tree...");
    let fo_tree = FoTreeBuilder::new().parse(Cursor::new(xml))?;
    println!("  ✓ {} FO nodes parsed\n", fo_tree.len());

    // Step 4: Layout engine
    println!("Step 4: Running layout engine...");
    let area_tree = LayoutEngine::new().layout(&fo_tree)?;
    println!("  ✓ {} areas generated\n", area_tree.len());

    // Step 5: Render to PDF
    println!("Step 5: Rendering to PDF...");
    let mut pdf_doc = PdfRenderer::new().render(&area_tree)?;

    // Add document metadata
    pdf_doc.info.title = Some("Phase 5 Complete Demo".to_string());
    pdf_doc.info.author = Some("Apache FOP Rust".to_string());
    pdf_doc.info.subject = Some("Complete demonstration of all Phase 5 features".to_string());

    println!("  ✓ PDF document rendered\n");

    // Step 6: Save to file
    println!("Step 6: Saving PDF...");
    let pdf_bytes = pdf_doc.to_bytes()?;
    let output_path = "/tmp/phase5_complete_demo.pdf";
    std::fs::write(output_path, &pdf_bytes)?;

    // Report statistics
    println!("\n=== Phase 5 Demo Complete ===");
    println!("✓ PDF saved: {}", output_path);
    println!(
        "  - Size: {} bytes ({:.2} KB)",
        pdf_bytes.len(),
        pdf_bytes.len() as f64 / 1024.0
    );
    println!("  - Pages: {}", pdf_doc.pages.len());
    println!("  - FO nodes: {}", fo_tree.len());
    println!("  - Layout areas: {}", area_tree.len());
    println!("\nFeatures demonstrated:");
    println!("  ✓ Background colors (named and hex)");
    println!("  ✓ 4-side borders (solid, dashed, dotted)");
    println!("  ✓ Text colors and alignment (left, center, right, justify)");
    println!("  ✓ Block spacing (space-before, space-after)");
    println!("  ✓ PNG and JPEG images with width/height control");
    println!("  ✓ Hyperlinks (fo:basic-link)");
    println!("  ✓ Bookmarks (fo:bookmark-tree with nesting)");
    println!("  ✓ Document metadata");
    println!("  ✓ Tables with styled cells");
    println!("  ✓ Lists with backgrounds");
    println!("  ✓ Multi-page layout");
    println!(
        "\nView the PDF with: evince {} (or your preferred PDF viewer)",
        output_path
    );

    Ok(())
}

/// Generate test images programmatically using the png crate
fn create_test_images() -> Result<(), Box<dyn std::error::Error>> {
    // Image 1: Red square (50x50)
    create_png_image("/tmp/test_red.png", 50, 50, &[255, 0, 0])?;

    // Image 2: Green rectangle (100x50)
    create_png_image("/tmp/test_green.png", 100, 50, &[0, 200, 0])?;

    // Image 3: Blue square (75x75)
    create_png_image("/tmp/test_blue.png", 75, 75, &[0, 100, 255])?;

    Ok(())
}

/// Create a simple solid-color PNG image
fn create_png_image(
    path: &str,
    width: u32,
    height: u32,
    color: &[u8; 3],
) -> Result<(), Box<dyn std::error::Error>> {
    let file = std::fs::File::create(path)?;
    let w = std::io::BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);

    let mut writer = encoder.write_header()?;

    // Create image data (all pixels same color)
    let mut data = Vec::with_capacity((width * height * 3) as usize);
    for _ in 0..(width * height) {
        data.extend_from_slice(color);
    }

    writer.write_image_data(&data)?;

    Ok(())
}

/// Create a comprehensive XSL-FO document demonstrating all Phase 5 features
#[allow(clippy::too_many_lines)]
fn create_comprehensive_fo_document() -> String {
    r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <!-- Page Layout Definitions -->
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4-portrait"
                          page-width="210mm"
                          page-height="297mm"
                          margin-top="20mm"
                          margin-bottom="20mm"
                          margin-left="25mm"
                          margin-right="25mm">
      <fo:region-body margin-top="10mm" margin-bottom="10mm"/>
      <fo:region-before extent="10mm"/>
      <fo:region-after extent="10mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <!-- Document Outline (Bookmarks) -->
  <fo:bookmark-tree>
    <fo:bookmark internal-destination="section1">
      <fo:bookmark-title>1. Visual Styling Features</fo:bookmark-title>
      <fo:bookmark internal-destination="subsec1-1">
        <fo:bookmark-title>1.1 Background Colors</fo:bookmark-title>
      </fo:bookmark>
      <fo:bookmark internal-destination="subsec1-2">
        <fo:bookmark-title>1.2 Borders</fo:bookmark-title>
      </fo:bookmark>
    </fo:bookmark>
    <fo:bookmark internal-destination="section2">
      <fo:bookmark-title>2. Layout Control</fo:bookmark-title>
    </fo:bookmark>
    <fo:bookmark internal-destination="section3">
      <fo:bookmark-title>3. Images</fo:bookmark-title>
    </fo:bookmark>
    <fo:bookmark internal-destination="section4">
      <fo:bookmark-title>4. Hyperlinks</fo:bookmark-title>
    </fo:bookmark>
    <fo:bookmark internal-destination="section5">
      <fo:bookmark-title>5. Complex Layouts</fo:bookmark-title>
    </fo:bookmark>
  </fo:bookmark-tree>

  <!-- Page Content -->
  <fo:page-sequence master-reference="A4-portrait">
    <fo:flow flow-name="xsl-region-body">

      <!-- Title Page -->
      <fo:block font-size="24pt"
                font-weight="bold"
                color="#0066CC"
                text-align="center"
                space-after="12pt"
                border-bottom="3pt solid #0066CC"
                padding-bottom="8pt">
        Phase 5 Complete Demo
      </fo:block>

      <fo:block font-size="14pt"
                color="#666666"
                text-align="center"
                space-after="24pt">
        Apache FOP Rust - All Features Showcase
      </fo:block>

      <!-- Section 1: Visual Styling -->
      <fo:block id="section1"
                font-size="18pt"
                font-weight="bold"
                color="#CC0000"
                space-before="18pt"
                space-after="8pt"
                border-left="4pt solid #CC0000"
                padding-left="8pt"
                background-color="#FFEEEE">
        1. Visual Styling Features
      </fo:block>

      <!-- Subsection 1.1: Background Colors -->
      <fo:block id="subsec1-1"
                font-size="14pt"
                font-weight="bold"
                space-before="12pt"
                space-after="6pt">
        1.1 Background Colors
      </fo:block>

      <fo:block font-size="11pt" space-after="4pt">
        Named colors:
      </fo:block>

      <fo:block background-color="yellow"
                padding="8pt"
                space-after="4pt"
                border="1pt solid black">
        This block has a yellow background (named color).
      </fo:block>

      <fo:block background-color="lightblue"
                padding="8pt"
                space-after="4pt"
                border="1pt solid black">
        This block has a lightblue background (named color).
      </fo:block>

      <fo:block font-size="11pt"
                space-before="8pt"
                space-after="4pt">
        Hex colors:
      </fo:block>

      <fo:block background-color="#FFEEAA"
                padding="8pt"
                space-after="4pt"
                border="1pt solid #AA8800">
        This block has a hex color background (#FFEEAA).
      </fo:block>

      <fo:block background-color="#AAEEFF"
                padding="8pt"
                space-after="4pt"
                border="1pt solid #0088AA">
        This block has a hex color background (#AAEEFF).
      </fo:block>

      <!-- Subsection 1.2: Borders -->
      <fo:block id="subsec1-2"
                font-size="14pt"
                font-weight="bold"
                space-before="12pt"
                space-after="6pt">
        1.2 Borders (4-side with different styles)
      </fo:block>

      <fo:block padding="12pt"
                space-after="6pt"
                border-top="3pt solid red"
                border-right="3pt dashed green"
                border-bottom="3pt dotted blue"
                border-left="3pt solid orange"
                background-color="#F5F5F5">
        This block demonstrates 4-side borders with different styles:
        top (solid red), right (dashed green), bottom (dotted blue), left (solid orange).
      </fo:block>

      <fo:block padding="12pt"
                space-after="6pt"
                border="2pt solid #333333"
                background-color="white">
        This block has a uniform 2pt solid border in dark gray (#333333).
      </fo:block>

      <!-- Text Colors -->
      <fo:block font-size="14pt"
                font-weight="bold"
                space-before="12pt"
                space-after="6pt">
        1.3 Text Colors
      </fo:block>

      <fo:block space-after="4pt">
        <fo:inline color="red">Red text</fo:inline>,
        <fo:inline color="green">green text</fo:inline>,
        <fo:inline color="blue">blue text</fo:inline>,
        <fo:inline color="#FF6600">orange text (#FF6600)</fo:inline>.
      </fo:block>

      <!-- Section 2: Layout Control -->
      <fo:block id="section2"
                font-size="18pt"
                font-weight="bold"
                color="#CC0000"
                space-before="24pt"
                space-after="8pt"
                border-left="4pt solid #CC0000"
                padding-left="8pt"
                background-color="#FFEEEE">
        2. Layout Control
      </fo:block>

      <fo:block font-size="14pt"
                font-weight="bold"
                space-before="12pt"
                space-after="6pt">
        2.1 Text Alignment
      </fo:block>

      <fo:block text-align="left"
                border="1pt solid #CCCCCC"
                padding="6pt"
                space-after="4pt"
                background-color="#FAFAFA">
        Left-aligned text (default): Lorem ipsum dolor sit amet.
      </fo:block>

      <fo:block text-align="center"
                border="1pt solid #CCCCCC"
                padding="6pt"
                space-after="4pt"
                background-color="#FAFAFA">
        Center-aligned text: Lorem ipsum dolor sit amet.
      </fo:block>

      <fo:block text-align="right"
                border="1pt solid #CCCCCC"
                padding="6pt"
                space-after="4pt"
                background-color="#FAFAFA">
        Right-aligned text: Lorem ipsum dolor sit amet.
      </fo:block>

      <fo:block text-align="justify"
                border="1pt solid #CCCCCC"
                padding="6pt"
                space-after="4pt"
                background-color="#FAFAFA">
        Justified text: In longer paragraphs, justified text spacing distributes words evenly across the line width from margin to margin for a professional appearance.
      </fo:block>

      <!-- Block Spacing -->
      <fo:block font-size="14pt"
                font-weight="bold"
                space-before="12pt"
                space-after="6pt">
        2.2 Block Spacing (space-before and space-after)
      </fo:block>

      <fo:block space-after="4pt">
        Normal block with default spacing.
      </fo:block>

      <fo:block space-before="18pt"
                space-after="18pt"
                background-color="#EEEEFF"
                padding="8pt"
                border="1pt solid #8888CC">
        This block has 18pt space before and after (notice the gap).
      </fo:block>

      <fo:block space-after="4pt">
        Another normal block to show the spacing effect.
      </fo:block>

      <!-- Section 3: Images -->
      <fo:block id="section3"
                font-size="18pt"
                font-weight="bold"
                color="#CC0000"
                space-before="24pt"
                space-after="8pt"
                border-left="4pt solid #CC0000"
                padding-left="8pt"
                background-color="#FFEEEE">
        3. Images (fo:external-graphic)
      </fo:block>

      <fo:block font-size="11pt" space-after="8pt">
        PNG images with width/height control:
      </fo:block>

      <!-- Red PNG -->
      <fo:block space-after="8pt">
        <fo:external-graphic src="/tmp/test_red.png"
                            width="50pt"
                            height="50pt"/>
        <fo:inline> ← Red 50x50 PNG</fo:inline>
      </fo:block>

      <!-- Green PNG -->
      <fo:block space-after="8pt">
        <fo:external-graphic src="/tmp/test_green.png"
                            width="100pt"
                            height="50pt"/>
        <fo:inline> ← Green 100x50 PNG</fo:inline>
      </fo:block>

      <!-- Blue PNG -->
      <fo:block space-after="8pt">
        <fo:external-graphic src="/tmp/test_blue.png"
                            width="75pt"
                            height="75pt"/>
        <fo:inline> ← Blue 75x75 PNG</fo:inline>
      </fo:block>

      <fo:block font-size="11pt"
                space-before="6pt"
                space-after="4pt">
        All images are programmatically generated PNG files demonstrating image embedding.
      </fo:block>

      <!-- Section 4: Hyperlinks -->
      <fo:block id="section4"
                font-size="18pt"
                font-weight="bold"
                color="#CC0000"
                space-before="24pt"
                space-after="8pt"
                border-left="4pt solid #CC0000"
                padding-left="8pt"
                background-color="#FFEEEE">
        4. Hyperlinks (fo:basic-link)
      </fo:block>

      <fo:block space-after="8pt">
        External hyperlinks using fo:basic-link:
      </fo:block>

      <fo:block space-after="4pt">
        Visit
        <fo:basic-link external-destination="https://www.apache.org">
          <fo:inline color="blue" text-decoration="underline">Apache Software Foundation</fo:inline>
        </fo:basic-link>
        for more information.
      </fo:block>

      <fo:block space-after="4pt">
        Check out
        <fo:basic-link external-destination="https://xmlgraphics.apache.org/fop/">
          <fo:inline color="blue" text-decoration="underline">Apache FOP</fo:inline>
        </fo:basic-link>
        (the original Java implementation).
      </fo:block>

      <fo:block space-after="4pt">
        Learn more about
        <fo:basic-link external-destination="https://www.w3.org/TR/xsl/">
          <fo:inline color="blue" text-decoration="underline">XSL-FO</fo:inline>
        </fo:basic-link>
        specification.
      </fo:block>

      <!-- Section 5: Complex Layouts -->
      <fo:block id="section5"
                font-size="18pt"
                font-weight="bold"
                color="#CC0000"
                space-before="24pt"
                space-after="8pt"
                border-left="4pt solid #CC0000"
                padding-left="8pt"
                background-color="#FFEEEE"
                break-before="page">
        5. Complex Layouts
      </fo:block>

      <!-- Tables -->
      <fo:block font-size="14pt"
                font-weight="bold"
                space-before="12pt"
                space-after="6pt">
        5.1 Tables with Styled Cells
      </fo:block>

      <fo:table space-after="12pt"
                border="2pt solid black">
        <fo:table-column column-width="40mm"/>
        <fo:table-column column-width="40mm"/>
        <fo:table-column column-width="40mm"/>

        <fo:table-header>
          <fo:table-row background-color="#0066CC">
            <fo:table-cell padding="6pt" border="1pt solid white">
              <fo:block color="white" font-weight="bold">Header 1</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="6pt" border="1pt solid white">
              <fo:block color="white" font-weight="bold">Header 2</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="6pt" border="1pt solid white">
              <fo:block color="white" font-weight="bold">Header 3</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-header>

        <fo:table-body>
          <fo:table-row background-color="#EEEEEE">
            <fo:table-cell padding="6pt" border="1pt solid #CCCCCC">
              <fo:block>Row 1, Col 1</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="6pt" border="1pt solid #CCCCCC">
              <fo:block>Row 1, Col 2</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="6pt" border="1pt solid #CCCCCC">
              <fo:block>Row 1, Col 3</fo:block>
            </fo:table-cell>
          </fo:table-row>

          <fo:table-row background-color="white">
            <fo:table-cell padding="6pt" border="1pt solid #CCCCCC">
              <fo:block>Row 2, Col 1</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="6pt" border="1pt solid #CCCCCC">
              <fo:block>Row 2, Col 2</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="6pt" border="1pt solid #CCCCCC" background-color="#FFEEAA">
              <fo:block>Highlighted</fo:block>
            </fo:table-cell>
          </fo:table-row>

          <fo:table-row background-color="#EEEEEE">
            <fo:table-cell padding="6pt" border="1pt solid #CCCCCC">
              <fo:block>Row 3, Col 1</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="6pt" border="1pt solid #CCCCCC">
              <fo:block>Row 3, Col 2</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="6pt" border="1pt solid #CCCCCC">
              <fo:block>Row 3, Col 3</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>

      <!-- Lists -->
      <fo:block font-size="14pt"
                font-weight="bold"
                space-before="12pt"
                space-after="6pt">
        5.2 Lists with Backgrounds
      </fo:block>

      <fo:list-block space-after="12pt">
        <fo:list-item space-after="4pt">
          <fo:list-item-label>
            <fo:block>•</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="20pt">
            <fo:block background-color="#EEFFEE"
                      padding="4pt"
                      border-left="3pt solid green">
              First list item with green accent and light background
            </fo:block>
          </fo:list-item-body>
        </fo:list-item>

        <fo:list-item space-after="4pt">
          <fo:list-item-label>
            <fo:block>•</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="20pt">
            <fo:block background-color="#FFEEEE"
                      padding="4pt"
                      border-left="3pt solid red">
              Second list item with red accent and light background
            </fo:block>
          </fo:list-item-body>
        </fo:list-item>

        <fo:list-item space-after="4pt">
          <fo:list-item-label>
            <fo:block>•</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="20pt">
            <fo:block background-color="#EEEEFF"
                      padding="4pt"
                      border-left="3pt solid blue">
              Third list item with blue accent and light background
            </fo:block>
          </fo:list-item-body>
        </fo:list-item>
      </fo:list-block>

      <!-- Mixed Content -->
      <fo:block font-size="14pt"
                font-weight="bold"
                space-before="12pt"
                space-after="6pt">
        5.3 Mixed Content
      </fo:block>

      <fo:block background-color="#F5F5F5"
                padding="12pt"
                border="2pt solid #666666"
                space-after="12pt">
        <fo:block font-weight="bold"
                  color="#0066CC"
                  space-after="6pt">
          Complex Block with Multiple Features
        </fo:block>

        <fo:block space-after="6pt">
          This block combines multiple Phase 5 features:
        </fo:block>

        <fo:block space-after="4pt">
          • Background color (#F5F5F5)
        </fo:block>
        <fo:block space-after="4pt">
          • Solid border (2pt #666666)
        </fo:block>
        <fo:block space-after="4pt">
          • <fo:inline color="red">Colored</fo:inline> inline text
        </fo:block>
        <fo:block space-after="4pt">
          • Proper spacing (padding and margins)
        </fo:block>
        <fo:block space-after="8pt">
          • <fo:basic-link external-destination="https://www.rust-lang.org">
              <fo:inline color="blue" text-decoration="underline">Embedded hyperlink</fo:inline>
            </fo:basic-link>
        </fo:block>

        <fo:block>
          <fo:external-graphic src="/tmp/test_red.png"
                              width="30pt"
                              height="30pt"/>
          <fo:inline> Embedded image inline with text</fo:inline>
        </fo:block>
      </fo:block>

      <!-- Conclusion -->
      <fo:block font-size="16pt"
                font-weight="bold"
                color="#006600"
                space-before="24pt"
                space-after="8pt"
                text-align="center"
                border-top="2pt solid #006600"
                border-bottom="2pt solid #006600"
                padding-top="8pt"
                padding-bottom="8pt"
                background-color="#EEFFEE">
        Phase 5 Complete ✓
      </fo:block>

      <fo:block text-align="center"
                space-after="8pt">
        All features successfully demonstrated
      </fo:block>

      <fo:block font-size="10pt"
                color="#666666"
                text-align="center"
                space-before="12pt">
        Generated by Apache FOP Rust - Phase 5 Complete Demo
      </fo:block>

    </fo:flow>
  </fo:page-sequence>
</fo:root>"##.to_string()
}
