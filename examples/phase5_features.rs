//! Phase 5 features demonstration
//!
//! Demonstrates background colors, borders, text alignment, and spacing features.

use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use fop_render::PdfRenderer;
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Phase 5 Features Demo ===\n");

    demo_backgrounds_and_borders()?;
    demo_text_alignment()?;
    demo_block_spacing()?;

    println!("\n=== All Phase 5 demos complete ===");
    Ok(())
}

fn demo_backgrounds_and_borders() -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Background Colors and Borders");

    let xml = r##"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4"
            page-width="210mm" page-height="297mm">
            <fo:region-body margin="1in"/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block font-size="18pt" font-weight="bold" color="blue">
                Backgrounds and Borders Demo
            </fo:block>

            <fo:block background-color="#FFEEAA"
                      border="2pt solid black"
                      padding="12pt">
                This block has a light yellow background and solid black border.
            </fo:block>

            <fo:block background-color="#AAEEFF"
                      border-top="3pt solid red"
                      border-bottom="3pt solid blue"
                      padding="8pt">
                This block has different top and bottom borders.
            </fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"##;

    let fo_tree = FoTreeBuilder::new().parse(Cursor::new(xml))?;
    let area_tree = LayoutEngine::new().layout(&fo_tree)?;
    let pdf_doc = PdfRenderer::new().render(&area_tree)?;
    let pdf_bytes = pdf_doc.to_bytes()?;

    std::fs::write("/tmp/fop_backgrounds_borders.pdf", &pdf_bytes)?;
    println!("   ✓ PDF generated: /tmp/fop_backgrounds_borders.pdf");
    println!("   Size: {} bytes\n", pdf_bytes.len());

    Ok(())
}

fn demo_text_alignment() -> Result<(), Box<dyn std::error::Error>> {
    println!("2. Text Alignment");

    let xml = r##"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4"
            page-width="210mm" page-height="297mm">
            <fo:region-body margin="1in"/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block font-size="18pt" font-weight="bold">
                Text Alignment Demo
            </fo:block>

            <fo:block text-align="left" border="1pt solid gray">
                This text is left-aligned (default).
            </fo:block>

            <fo:block text-align="center" border="1pt solid gray">
                This text is center-aligned.
            </fo:block>

            <fo:block text-align="right" border="1pt solid gray">
                This text is right-aligned.
            </fo:block>

            <fo:block text-align="justify" border="1pt solid gray">
                This text is justified. In a longer paragraph, the words would be spaced
                to fill the entire line width evenly from left to right margins.
            </fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"##;

    let fo_tree = FoTreeBuilder::new().parse(Cursor::new(xml))?;
    let area_tree = LayoutEngine::new().layout(&fo_tree)?;
    let pdf_doc = PdfRenderer::new().render(&area_tree)?;
    let pdf_bytes = pdf_doc.to_bytes()?;

    std::fs::write("/tmp/fop_text_alignment.pdf", &pdf_bytes)?;
    println!("   ✓ PDF generated: /tmp/fop_text_alignment.pdf");
    println!("   Size: {} bytes\n", pdf_bytes.len());

    Ok(())
}

fn demo_block_spacing() -> Result<(), Box<dyn std::error::Error>> {
    println!("3. Block Spacing (space-before/space-after)");

    let xml = r##"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
    <fo:layout-master-set>
        <fo:simple-page-master master-name="A4"
            page-width="210mm" page-height="297mm">
            <fo:region-body margin="1in"/>
        </fo:simple-page-master>
    </fo:layout-master-set>
    <fo:page-sequence master-reference="A4">
        <fo:flow flow-name="xsl-region-body">
            <fo:block font-size="18pt" font-weight="bold">
                Block Spacing Demo
            </fo:block>

            <fo:block>
                This is a normal block with default spacing.
            </fo:block>

            <fo:block space-before="24pt" space-after="24pt"
                      background-color="#EEEEFF">
                This block has 24pt space before and after.
            </fo:block>

            <fo:block>
                Another normal block to show the spacing effect.
            </fo:block>

            <fo:block space-before="12pt"
                      background-color="#FFEEEE">
                This block has only space-before.
            </fo:block>

            <fo:block space-after="12pt"
                      background-color="#EEFFEE">
                This block has only space-after.
            </fo:block>

            <fo:block>
                Final block showing spacing is applied correctly.
            </fo:block>
        </fo:flow>
    </fo:page-sequence>
</fo:root>"##;

    let fo_tree = FoTreeBuilder::new().parse(Cursor::new(xml))?;
    let area_tree = LayoutEngine::new().layout(&fo_tree)?;
    let pdf_doc = PdfRenderer::new().render(&area_tree)?;
    let pdf_bytes = pdf_doc.to_bytes()?;

    std::fs::write("/tmp/fop_block_spacing.pdf", &pdf_bytes)?;
    println!("   ✓ PDF generated: /tmp/fop_block_spacing.pdf");
    println!("   Size: {} bytes\n", pdf_bytes.len());

    Ok(())
}
