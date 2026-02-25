//! Comprehensive integration demo for tables and lists
//!
//! Demonstrates end-to-end processing of XSL-FO documents with tables and lists.

use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use fop_render::PdfRenderer;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Apache FOP Rust - Tables and Lists Integration Demo ===\n");

    // Demo 1: Simple table
    demo_simple_table()?;

    // Demo 2: List with different marker styles
    demo_list_markers()?;

    // Demo 3: Complex document with both
    demo_complex_document()?;

    println!("\n=== Demo Complete ===");
    Ok(())
}

fn demo_simple_table() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Simple Table Demo");
    println!("   Creating a 2x2 table...");

    let xml = r##"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
            <fo:region-body margin="1in"/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block>Simple Table Example:</fo:block>
            <fo:table border="1pt solid black">
                <fo:table-body>
                    <fo:table-row>
                        <fo:table-cell>
                            <fo:block>Cell 1,1</fo:block>
                        </fo:table-cell>
                        <fo:table-cell>
                            <fo:block>Cell 1,2</fo:block>
                        </fo:table-cell>
                    </fo:table-row>
                    <fo:table-row>
                        <fo:table-cell>
                            <fo:block>Cell 2,1</fo:block>
                        </fo:table-cell>
                        <fo:table-cell>
                            <fo:block>Cell 2,2</fo:block>
                        </fo:table-cell>
                    </fo:table-row>
                </fo:table-body>
            </fo:table>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"##;

    // Parse FO tree
    let cursor = Cursor::new(xml);
    let builder = FoTreeBuilder::new();
    let fo_tree = builder.parse(cursor)?;
    println!("   ✓ Parsed {} FO nodes", fo_tree.len());

    // Layout to area tree
    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&fo_tree)?;
    println!("   ✓ Generated {} areas", area_tree.len());

    // Render to PDF
    let renderer = PdfRenderer::new();
    let pdf_doc = renderer.render(&area_tree)?;
    let pdf_bytes = pdf_doc.to_bytes()?;

    // Save
    std::fs::write("/tmp/fop_table_simple.pdf", &pdf_bytes)?;
    println!("   ✓ PDF generated: /tmp/fop_table_simple.pdf");
    println!(
        "   Size: {} bytes, Pages: {}\n",
        pdf_bytes.len(),
        pdf_doc.pages.len()
    );

    Ok(())
}

fn demo_list_markers() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. List Markers Demo");
    println!("   Creating lists with different marker styles...");

    let xml = r##"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
            <fo:region-body margin="1in"/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block font-size="14pt" font-weight="bold">Shopping List</fo:block>
            <fo:list-block>
                <fo:list-item>
                    <fo:list-item-label><fo:block>•</fo:block></fo:list-item-label>
                    <fo:list-item-body><fo:block>Milk</fo:block></fo:list-item-body>
                </fo:list-item>
                <fo:list-item>
                    <fo:list-item-label><fo:block>•</fo:block></fo:list-item-label>
                    <fo:list-item-body><fo:block>Bread</fo:block></fo:list-item-body>
                </fo:list-item>
                <fo:list-item>
                    <fo:list-item-label><fo:block>•</fo:block></fo:list-item-label>
                    <fo:list-item-body><fo:block>Eggs</fo:block></fo:list-item-body>
                </fo:list-item>
            </fo:list-block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"##;

    // Parse and process
    let cursor = Cursor::new(xml);
    let builder = FoTreeBuilder::new();
    let fo_tree = builder.parse(cursor)?;
    println!("   ✓ Parsed {} FO nodes", fo_tree.len());

    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&fo_tree)?;
    println!("   ✓ Generated {} areas", area_tree.len());

    let renderer = PdfRenderer::new();
    let pdf_doc = renderer.render(&area_tree)?;
    let pdf_bytes = pdf_doc.to_bytes()?;

    std::fs::write("/tmp/fop_list_simple.pdf", &pdf_bytes)?;
    println!("   ✓ PDF generated: /tmp/fop_list_simple.pdf");
    println!(
        "   Size: {} bytes, Pages: {}\n",
        pdf_bytes.len(),
        pdf_doc.pages.len()
    );

    Ok(())
}

fn demo_complex_document() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. Complex Document Demo");
    println!("   Creating document with tables AND lists...");

    let xml = r##"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
            <fo:region-body margin="1in"/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block font-size="18pt" font-weight="bold" color="blue">
                Product Catalog
            </fo:block>

            <fo:block font-size="14pt" space-before="12pt">
                Product Comparison Table
            </fo:block>

            <fo:table border="1pt solid black" space-before="6pt">
                <fo:table-body>
                    <fo:table-row background-color="#CCCCCC">
                        <fo:table-cell><fo:block>Product</fo:block></fo:table-cell>
                        <fo:table-cell><fo:block>Price</fo:block></fo:table-cell>
                        <fo:table-cell><fo:block>Stock</fo:block></fo:table-cell>
                    </fo:table-row>
                    <fo:table-row>
                        <fo:table-cell><fo:block>Laptop</fo:block></fo:table-cell>
                        <fo:table-cell><fo:block>$999</fo:block></fo:table-cell>
                        <fo:table-cell><fo:block>15</fo:block></fo:table-cell>
                    </fo:table-row>
                    <fo:table-row>
                        <fo:table-cell><fo:block>Mouse</fo:block></fo:table-cell>
                        <fo:table-cell><fo:block>$29</fo:block></fo:table-cell>
                        <fo:table-cell><fo:block>150</fo:block></fo:table-cell>
                    </fo:table-row>
                    <fo:table-row>
                        <fo:table-cell><fo:block>Keyboard</fo:block></fo:table-cell>
                        <fo:table-cell><fo:block>$79</fo:block></fo:table-cell>
                        <fo:table-cell><fo:block>80</fo:block></fo:table-cell>
                    </fo:table-row>
                </fo:table-body>
            </fo:table>

            <fo:block font-size="14pt" space-before="18pt">
                Product Features
            </fo:block>

            <fo:list-block space-before="6pt">
                <fo:list-item>
                    <fo:list-item-label><fo:block>✓</fo:block></fo:list-item-label>
                    <fo:list-item-body>
                        <fo:block>Free shipping on orders over $100</fo:block>
                    </fo:list-item-body>
                </fo:list-item>
                <fo:list-item>
                    <fo:list-item-label><fo:block>✓</fo:block></fo:list-item-label>
                    <fo:list-item-body>
                        <fo:block>30-day money-back guarantee</fo:block>
                    </fo:list-item-body>
                </fo:list-item>
                <fo:list-item>
                    <fo:list-item-label><fo:block>✓</fo:block></fo:list-item-label>
                    <fo:list-item-body>
                        <fo:block>1-year manufacturer warranty</fo:block>
                    </fo:list-item-body>
                </fo:list-item>
                <fo:list-item>
                    <fo:list-item-label><fo:block>✓</fo:block></fo:list-item-label>
                    <fo:list-item-body>
                        <fo:block>24/7 customer support</fo:block>
                    </fo:list-item-body>
                </fo:list-item>
            </fo:list-block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"##;

    // Full pipeline
    let cursor = Cursor::new(xml);
    let builder = FoTreeBuilder::new();
    let fo_tree = builder.parse(cursor)?;
    println!("   ✓ Parsed {} FO nodes", fo_tree.len());

    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&fo_tree)?;
    println!("   ✓ Generated {} areas", area_tree.len());

    let renderer = PdfRenderer::new();
    let pdf_doc = renderer.render(&area_tree)?;
    let pdf_bytes = pdf_doc.to_bytes()?;

    std::fs::write("/tmp/fop_complex.pdf", &pdf_bytes)?;
    println!("   ✓ PDF generated: /tmp/fop_complex.pdf");
    println!(
        "   Size: {} bytes, Pages: {}",
        pdf_bytes.len(),
        pdf_doc.pages.len()
    );

    println!("\n   Document structure:");
    println!("   - Title (18pt blue)");
    println!("   - Table with 4 rows × 3 columns");
    println!("   - List with 4 items");
    println!("\n   Try: pdftotext /tmp/fop_complex.pdf -");

    Ok(())
}
