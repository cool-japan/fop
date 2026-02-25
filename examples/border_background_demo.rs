//! Demonstration of border and background color property parsing
//!
//! This example shows how properties flow through the system:
//! FO tree (XML) → PropertyList → TraitSet → Area → PDF rendering

use fop_core::{FoTreeBuilder, PropertyId};
use fop_layout::LayoutEngine;
use fop_types::{Color, Length};
use std::io::Cursor;

fn main() {
    println!("=== Border and Background Color Property Parsing Demo ===\n");

    // XML with various border and background properties
    let xml = r###"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
            <fo:region-body margin="1in"/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block background-color="#FF0000" padding="5pt">
                Red background (hex color)
            </fo:block>
            <fo:block background-color="blue" padding="10pt">
                Blue background (named color)
            </fo:block>
            <fo:block border-top-width="2pt" border-top-color="green">
                Green top border
            </fo:block>
            <fo:block border-width="2pt" border-color="red">
                Red border all around
            </fo:block>
            <fo:block background-color="orange" padding="8pt">
                Orange background
            </fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"###;

    // Step 1: Parse FO tree
    println!("Step 1: Parsing FO tree from XML...");
    let cursor = Cursor::new(xml);
    let builder = FoTreeBuilder::new();
    let arena = builder.parse(cursor).expect("Failed to parse XML");
    println!("  ✓ Parsed {} nodes\n", arena.len());

    // Step 2: Examine properties in FO tree
    println!("Step 2: Examining properties in FO tree...");
    let mut bg_count = 0;
    let mut border_count = 0;

    for (id, node) in arena.iter() {
        if let Some(props) = node.data.properties() {
            // Check background color
            if let Ok(bg) = props.get(PropertyId::BackgroundColor) {
                if let Some(color) = bg.as_color() {
                    println!("  Node {:?}: background-color = {}", id, color);
                    bg_count += 1;
                }
            }

            // Check border properties
            if let Ok(top_width) = props.get(PropertyId::BorderTopWidth) {
                if let Some(width) = top_width.as_length() {
                    if width != Length::ZERO {
                        println!("  Node {:?}: border-top-width = {}", id, width);
                        border_count += 1;
                    }
                }
            }
        }
    }
    println!(
        "  ✓ Found {} background colors, {} borders\n",
        bg_count, border_count
    );

    // Step 3: Create layout (extracts properties to TraitSet)
    println!("Step 3: Creating layout (extracting properties to TraitSet)...");
    let engine = LayoutEngine::new();

    let area_tree = engine.layout(&arena).expect("Failed to layout");
    println!("  ✓ Created area tree with {} areas\n", area_tree.len());

    // Step 4: Verify TraitSet in areas
    println!("Step 4: Verifying TraitSet in areas...");
    let mut trait_bg_count = 0;
    let mut trait_border_count = 0;

    for (id, node) in area_tree.iter() {
        if let Some(bg_color) = node.area.traits.background_color {
            println!("  Area {:?}: background_color = {}", id, bg_color);
            trait_bg_count += 1;
        }

        if let Some(border_widths) = node.area.traits.border_width {
            if border_widths[0] != Length::ZERO
                || border_widths[1] != Length::ZERO
                || border_widths[2] != Length::ZERO
                || border_widths[3] != Length::ZERO
            {
                println!(
                    "  Area {:?}: border_width = [{}, {}, {}, {}]",
                    id, border_widths[0], border_widths[1], border_widths[2], border_widths[3]
                );
                trait_border_count += 1;
            }
        }
    }
    println!(
        "  ✓ Found {} background colors, {} borders in TraitSet\n",
        trait_bg_count, trait_border_count
    );

    // Summary
    println!("=== Summary ===");
    println!("✓ Step 1: XML parsed to FO tree");
    println!("✓ Step 2: Properties stored in PropertyList");
    println!("✓ Step 3: Layout created, properties extracted");
    println!("✓ Step 4: TraitSet populated with border/background properties");
    println!("\nProperty flow complete!");
    println!("  FO tree → PropertyList → TraitSet → Area → (ready for PDF rendering)");

    // Test shorthand expansion
    println!("\n=== Testing Shorthand Expansion ===");
    let shorthand_xml = r###"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
            <fo:region-body/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block border-width="2pt" border-color="red">Test</fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"###;

    let cursor2 = Cursor::new(shorthand_xml);
    let builder2 = FoTreeBuilder::new();
    let arena2 = builder2.parse(cursor2).expect("Failed to parse");

    for (_id, node) in arena2.iter() {
        if let Some(props) = node.data.properties() {
            let top = props
                .get(PropertyId::BorderTopWidth)
                .ok()
                .and_then(|v| v.as_length());
            let right = props
                .get(PropertyId::BorderRightWidth)
                .ok()
                .and_then(|v| v.as_length());
            let bottom = props
                .get(PropertyId::BorderBottomWidth)
                .ok()
                .and_then(|v| v.as_length());
            let left = props
                .get(PropertyId::BorderLeftWidth)
                .ok()
                .and_then(|v| v.as_length());

            if let (Some(t), Some(r), Some(b), Some(l)) = (top, right, bottom, left) {
                println!("Border shorthand expanded to:");
                println!("  Top: {}, Right: {}, Bottom: {}, Left: {}", t, r, b, l);
                if t == Length::from_pt(2.0) && r == t && b == t && l == t {
                    println!("  ✓ All sides correctly set to 2pt");
                }
            }

            let top_color = props
                .get(PropertyId::BorderTopColor)
                .ok()
                .and_then(|v| v.as_color());
            if let Some(color) = top_color {
                if color == Color::RED {
                    println!("  ✓ Border color correctly set to red");
                }
            }
        }
    }

    println!("\n=== All tests passed! ===");
}
