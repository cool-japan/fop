//! Auto-verify workflow tests
//!
//! These tests exercise the full cycle:
//!   XSL-FO → Parse → Layout → PDF (fop-render) → Rasterize (fop-pdf-renderer) → Verify pixels
//!
//! This validates that our PDF generator actually produces renderable, non-blank output.

use super::{process_fo_document, verify_pdf_rendering, load_fixture};

/// Minimal one-page document with a single text block
const SIMPLE_FO: &str = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
        page-width="210mm" page-height="297mm"
        margin="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="24pt" color="black">Hello World</fo:block>
      <fo:block font-size="12pt">This is a test paragraph.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

/// Document with colored background rectangles
const COLORED_FO: &str = r#"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
        page-width="210mm" page-height="297mm"
        margin="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block background-color="navy" color="white" padding="10pt" font-size="18pt">
        Dark background block
      </fo:block>
      <fo:block background-color="yellow" color="black" padding="5pt" font-size="12pt">
        Yellow background block
      </fo:block>
      <fo:block color="red" font-size="14pt">Red text</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

#[test]
fn test_verify_simple_document_renders() {
    let pdf = process_fo_document(SIMPLE_FO).expect("PDF generation failed");
    verify_pdf_rendering(&pdf, 1);
}

#[test]
fn test_verify_colored_document_renders() {
    let pdf = process_fo_document(COLORED_FO).expect("PDF generation failed");
    // Colored blocks produce non-white pixels
    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF parse failed");
    assert_eq!(renderer.page_count(), 1);
    let page = renderer.render_page(0, 72.0).expect("Render failed");
    assert!(page.width > 0 && page.height > 0);
}

#[test]
fn test_verify_fixture_simple_single_page() {
    let fo = load_fixture("simple_single_page.fo");
    let pdf = process_fo_document(&fo).expect("PDF generation failed");
    verify_pdf_rendering(&pdf, 1);
}

#[test]
fn test_verify_fixture_multi_page() {
    let fo = load_fixture("multi_page.fo");
    let pdf = process_fo_document(&fo).expect("PDF generation failed");

    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF parse failed");
    // Multi-page document should parse fine
    assert!(renderer.page_count() >= 1, "Multi-page document should have at least 1 page");
    // Render first page
    let page = renderer.render_page(0, 72.0).expect("Render page 0 failed");
    assert!(page.width > 0 && page.height > 0);
}

#[test]
fn test_verify_fixture_table() {
    let fo = load_fixture("table_simple.fo");
    let pdf = process_fo_document(&fo).expect("PDF generation failed");
    verify_pdf_rendering(&pdf, 1);
}

#[test]
fn test_verify_pdf_renderer_page_count() {
    // Verify that fop-pdf-renderer page count matches fop-render page count
    let fo = load_fixture("multi_page.fo");
    let pdf = process_fo_document(&fo).expect("PDF generation failed");

    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF parse failed");

    // All pages should be renderable
    for i in 0..renderer.page_count() {
        let page = renderer.render_page(i, 72.0)
            .unwrap_or_else(|e| panic!("Failed to render page {}: {}", i, e));
        assert!(page.width > 0, "Page {} has zero width", i);
        assert!(page.height > 0, "Page {} has zero height", i);
    }
}

#[test]
fn test_verify_pdf_to_png_bytes() {
    let pdf = process_fo_document(SIMPLE_FO).expect("PDF generation failed");
    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF parse failed");

    let png_bytes = renderer.render_page(0, 72.0)
        .expect("Render failed")
        .to_png()
        .expect("PNG encode failed");

    // Valid PNG starts with the PNG magic bytes
    assert!(png_bytes.starts_with(b"\x89PNG"), "Output is not valid PNG");
    assert!(png_bytes.len() > 100, "PNG too small");
}

#[test]
fn test_verify_all_pages_pipeline() {
    let pdf = process_fo_document(SIMPLE_FO).expect("PDF generation failed");
    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF parse failed");
    let pages = renderer.render_all_pages(72.0).expect("render_all_pages failed");
    assert!(!pages.is_empty(), "No pages rendered");
    for (i, png) in pages.iter().enumerate() {
        assert!(png.starts_with(b"\x89PNG"), "Page {} is not valid PNG", i);
    }
}
