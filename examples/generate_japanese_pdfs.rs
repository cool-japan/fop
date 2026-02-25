//! Generate Japanese PDF samples

use std::fs;
use std::io::Cursor;
use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use fop_render::PdfRenderer;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Generating Japanese PDFs...\n");
    
    // Generate Invoice PDF
    let invoice_fo = fs::read_to_string("/tmp/japanese_invoice.fo")?;
    println!("Processing Japanese Invoice...");
    let pdf_bytes = process_fo(&invoice_fo)?;
    fs::write("/tmp/japanese_invoice.pdf", pdf_bytes)?;
    println!("✓ Created: /tmp/japanese_invoice.pdf");
    
    // Generate Letter PDF
    let letter_fo = fs::read_to_string("/tmp/japanese_letter.fo")?;
    println!("\nProcessing Japanese Business Letter...");
    let pdf_bytes = process_fo(&letter_fo)?;
    fs::write("/tmp/japanese_letter.pdf", pdf_bytes)?;
    println!("✓ Created: /tmp/japanese_letter.pdf");
    
    println!("\n✅ Successfully created 2 Japanese PDF files!");
    println!("\nFiles:");
    println!("  1. /tmp/japanese_invoice.pdf - 請求書 (Invoice)");
    println!("  2. /tmp/japanese_letter.pdf - 案内状 (Business Letter)");
    
    Ok(())
}

fn process_fo(fo_content: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let builder = FoTreeBuilder::new();
    let fo_tree = builder.parse(Cursor::new(fo_content))?;
    
    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&fo_tree)?;
    
    let renderer = PdfRenderer::new();
    let pdf_doc = renderer.render(&area_tree)?;
    let bytes = pdf_doc.to_bytes()?;
    
    Ok(bytes)
}
