//! Example demonstrating full FO document parsing

use fop_core::{FoTreeBuilder, PropertyId};
use std::io::Cursor;

fn main() {
    // Sample XSL-FO document
    let xml = r##"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4"
                               page-width="210mm"
                               page-height="297mm"
                               margin="1in">
            <fo:region-body margin="0.5in"/>
        </fo:simple-page-master>
    </fo:layout-master-set>

    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block font-size="14pt"
                      font-family="Arial"
                      color="blue"
                      margin-bottom="12pt">
                Hello, FOP World!
            </fo:block>

            <fo:block font-size="12pt"
                      text-align="justify">
                This is a sample paragraph demonstrating the XSL-FO parsing
                capabilities of the Rust implementation.
            </fo:block>

            <fo:block font-size="10pt"
                      color="#808080"
                      margin-top="20pt">
                Text in gray with custom color.
            </fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"##;

    println!("=== Parsing XSL-FO Document ===\n");

    // Parse the document
    let cursor = Cursor::new(xml);
    let builder = FoTreeBuilder::new();

    match builder.parse(cursor) {
        Ok(arena) => {
            println!("✓ Document parsed successfully");
            println!("  Total nodes: {}", arena.len());
            println!();

            // Display tree structure
            println!("=== Document Structure ===\n");
            for (id, node) in arena.iter() {
                let depth = arena.depth(id);
                let indent = "  ".repeat(depth);
                let element_name = node.data.element_name();

                println!("{}{} (depth: {})", indent, element_name, depth);

                // Show properties for some nodes
                if let Some(props) = node.data.properties() {
                    if let Ok(font_size) = props.get(PropertyId::FontSize) {
                        println!("{}  └─ font-size: {:?}", indent, font_size);
                    }
                    if let Ok(color) = props.get(PropertyId::Color) {
                        println!("{}  └─ color: {:?}", indent, color);
                    }
                }
            }

            println!();

            // Display tree statistics
            println!("=== Statistics ===");

            let mut block_count = 0;
            let mut text_count = 0;

            for (_, node) in arena.iter() {
                match node.data.element_name() {
                    "block" => block_count += 1,
                    "#text" => text_count += 1,
                    _ => {}
                }
            }

            println!("  fo:block elements: {}", block_count);
            println!("  Text nodes: {}", text_count);

            // Get root and its children
            if let Some((root_id, _)) = arena.root() {
                let children = arena.children(root_id);
                println!("  Root children: {}", children.len());
            }
        }
        Err(e) => {
            eprintln!("✗ Parse error: {}", e);
        }
    }
}
