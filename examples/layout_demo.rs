//! Example demonstrating the layout engine

use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use std::io::Cursor;

fn main() {
    println!("=== FOP Layout Engine Demo ===\n");

    // Create a simple FO document
    let xml = r##"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
            <fo:region-body margin="1in"/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block font-size="16pt" color="blue">
                First block - larger blue text
            </fo:block>
            <fo:block font-size="12pt" color="black">
                Second block - normal text
            </fo:block>
            <fo:block font-size="10pt" color="#808080">
                Third block - smaller gray text
            </fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"##;

    // Parse FO document
    println!("Step 1: Parsing XSL-FO document...");
    let cursor = Cursor::new(xml);
    let builder = FoTreeBuilder::new();
    let fo_tree = match builder.parse(cursor) {
        Ok(tree) => {
            println!("  ✓ Parsed {} FO nodes", tree.len());
            tree
        }
        Err(e) => {
            eprintln!("  ✗ Parse error: {}", e);
            return;
        }
    };

    // Perform layout
    println!("\nStep 2: Performing layout...");
    let engine = LayoutEngine::new();
    let area_tree = match engine.layout(&fo_tree) {
        Ok(tree) => {
            println!("  ✓ Created {} areas", tree.len());
            tree
        }
        Err(e) => {
            eprintln!("  ✗ Layout error: {}", e);
            return;
        }
    };

    // Display area tree structure
    println!("\nStep 3: Area Tree Structure:");
    println!(
        "  {:<15} {:<20} {:<20} {:<15}",
        "Type", "Position", "Size", "Content"
    );
    println!("  {}", "-".repeat(70));

    for (_id, node) in area_tree.iter() {
        let area = &node.area;
        let area_type = format!("{:?}", area.area_type);
        let position = format!(
            "({}, {})",
            area.geometry.x.to_pt() as i32,
            area.geometry.y.to_pt() as i32
        );
        let size = format!(
            "{}x{} pt",
            area.geometry.width.to_pt() as i32,
            area.geometry.height.to_pt() as i32
        );
        let content = if let Some(text) = area.text_content() {
            let truncated = if text.len() > 30 {
                format!("{}...", &text[..27])
            } else {
                text.to_string()
            };
            truncated.trim().to_string()
        } else if area.has_image_data() {
            "[IMAGE]".to_string()
        } else {
            "-".to_string()
        };

        println!(
            "  {:<15} {:<20} {:<20} {}",
            area_type, position, size, content
        );

        // Show traits if present
        if let Some(color) = area.traits.color {
            println!("    └─ color: {}", color);
        }
        if let Some(font_size) = area.traits.font_size {
            println!("    └─ font-size: {}", font_size);
        }
    }

    println!("\n✓ Layout complete!");
    println!("\nSummary:");
    println!("  FO nodes: {}", fo_tree.len());
    println!("  Areas created: {}", area_tree.len());

    // Count area types
    let mut page_count = 0;
    let mut block_count = 0;
    let mut text_count = 0;

    for (_, node) in area_tree.iter() {
        match node.area.area_type {
            fop_layout::AreaType::Page => page_count += 1,
            fop_layout::AreaType::Block => block_count += 1,
            fop_layout::AreaType::Text => text_count += 1,
            _ => {}
        }
    }

    println!("  Pages: {}", page_count);
    println!("  Blocks: {}", block_count);
    println!("  Text areas: {}", text_count);
}
