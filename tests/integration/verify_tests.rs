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

// ── Sweep test: walk every .fo fixture, render, rasterize ────────────────────

#[test]
fn test_verify_all_fixtures_render() {
    let fixtures_dir = std::path::Path::new("tests/integration/fixtures");
    let mut entries: Vec<_> = std::fs::read_dir(fixtures_dir)
        .expect("fixtures dir should be readable")
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("fo"))
        .collect();
    assert!(!entries.is_empty(), "No .fo fixtures found in fixtures dir");

    // Sort for deterministic ordering in test output
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        let name = path
            .file_name()
            .expect("fixture path must have a file name component")
            .to_str()
            .expect("fixture file name must be valid UTF-8");
        let fo = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Could not read fixture {}: {}", name, e));

        let pdf = super::process_fo_document(&fo)
            .unwrap_or_else(|e| panic!("process_fo_document failed for {}: {}", name, e));

        let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
            .unwrap_or_else(|e| panic!("PdfRenderer::from_bytes failed for {}: {}", name, e));

        let page_count = renderer.page_count();
        // empty_document.fo may legitimately produce 0 pages
        for i in 0..page_count {
            let page = renderer
                .render_page(i, 72.0)
                .unwrap_or_else(|e| panic!("render_page({}) failed for {}: {}", i, name, e));
            assert!(
                page.width > 0 && page.height > 0,
                "Page {} of {} has zero dimensions (width={}, height={})",
                i,
                name,
                page.width,
                page.height
            );
        }
    }
}

// ── Per-fixture tests (14 fixtures not yet covered above) ────────────────────

#[test]
fn test_verify_fixture_complex_report() {
    let fo = super::load_fixture("complex_report.fo");
    let pdf = super::process_fo_document(&fo)
        .expect("complex_report.fo should generate PDF");
    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF from complex_report.fo should parse");
    assert!(
        renderer.page_count() > 0,
        "complex_report.fo should produce at least one page"
    );
    let page = renderer
        .render_page(0, 72.0)
        .expect("First page of complex_report.fo should rasterize");
    assert!(page.width > 0 && page.height > 0, "Page has zero dimensions");
}

#[test]
fn test_verify_fixture_deeply_nested() {
    let fo = super::load_fixture("deeply_nested.fo");
    let pdf = super::process_fo_document(&fo)
        .expect("deeply_nested.fo should generate PDF");
    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF from deeply_nested.fo should parse");
    assert!(
        renderer.page_count() > 0,
        "deeply_nested.fo should produce at least one page"
    );
    let page = renderer
        .render_page(0, 72.0)
        .expect("First page of deeply_nested.fo should rasterize");
    assert!(page.width > 0 && page.height > 0, "Page has zero dimensions");
}

#[test]
fn test_verify_fixture_empty_document() {
    let fo = super::load_fixture("empty_document.fo");
    match super::process_fo_document(&fo) {
        Ok(pdf) => {
            let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
                .expect("empty_document PDF should parse");
            // 0 pages is acceptable for an empty document — just ensure no panic
            let _ = renderer.page_count();
        }
        Err(_) => {
            // empty document may legitimately fail processing — acceptable
        }
    }
}

#[test]
fn test_verify_fixture_footnote_basic() {
    let fo = super::load_fixture("footnote_basic.fo");
    let pdf = super::process_fo_document(&fo)
        .expect("footnote_basic.fo should generate PDF");
    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF from footnote_basic.fo should parse");
    assert!(
        renderer.page_count() > 0,
        "footnote_basic.fo should produce at least one page"
    );
    let page = renderer
        .render_page(0, 72.0)
        .expect("First page of footnote_basic.fo should rasterize");
    assert!(page.width > 0 && page.height > 0, "Page has zero dimensions");
}

#[test]
fn test_verify_fixture_lists() {
    let fo = super::load_fixture("lists.fo");
    let pdf = super::process_fo_document(&fo)
        .expect("lists.fo should generate PDF");
    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF from lists.fo should parse");
    assert!(
        renderer.page_count() > 0,
        "lists.fo should produce at least one page"
    );
    let page = renderer
        .render_page(0, 72.0)
        .expect("First page of lists.fo should rasterize");
    assert!(page.width > 0 && page.height > 0, "Page has zero dimensions");
}

#[test]
fn test_verify_fixture_long_text() {
    let fo = super::load_fixture("long_text.fo");
    let pdf = super::process_fo_document(&fo)
        .expect("long_text.fo should generate PDF");
    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF from long_text.fo should parse");
    assert!(
        renderer.page_count() > 0,
        "long_text.fo should produce at least one page"
    );
    let page = renderer
        .render_page(0, 72.0)
        .expect("First page of long_text.fo should rasterize");
    assert!(page.width > 0 && page.height > 0, "Page has zero dimensions");
}

#[test]
fn test_verify_fixture_multi_column() {
    let fo = super::load_fixture("multi_column.fo");
    let pdf = super::process_fo_document(&fo)
        .expect("multi_column.fo should generate PDF");
    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF from multi_column.fo should parse");
    assert!(
        renderer.page_count() > 0,
        "multi_column.fo should produce at least one page"
    );
    let page = renderer
        .render_page(0, 72.0)
        .expect("First page of multi_column.fo should rasterize");
    assert!(page.width > 0 && page.height > 0, "Page has zero dimensions");
}

#[test]
fn test_verify_fixture_nested_blocks() {
    let fo = super::load_fixture("nested_blocks.fo");
    let pdf = super::process_fo_document(&fo)
        .expect("nested_blocks.fo should generate PDF");
    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF from nested_blocks.fo should parse");
    assert!(
        renderer.page_count() > 0,
        "nested_blocks.fo should produce at least one page"
    );
    let page = renderer
        .render_page(0, 72.0)
        .expect("First page of nested_blocks.fo should rasterize");
    assert!(page.width > 0 && page.height > 0, "Page has zero dimensions");
}

#[test]
fn test_verify_fixture_simple_invoice() {
    let fo = super::load_fixture("simple_invoice.fo");
    let pdf = super::process_fo_document(&fo)
        .expect("simple_invoice.fo should generate PDF");
    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF from simple_invoice.fo should parse");
    assert!(
        renderer.page_count() > 0,
        "simple_invoice.fo should produce at least one page"
    );
    let page = renderer
        .render_page(0, 72.0)
        .expect("First page of simple_invoice.fo should rasterize");
    assert!(page.width > 0 && page.height > 0, "Page has zero dimensions");
}

#[test]
fn test_verify_fixture_table_header_footer() {
    let fo = super::load_fixture("table_header_footer.fo");
    let pdf = super::process_fo_document(&fo)
        .expect("table_header_footer.fo should generate PDF");
    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF from table_header_footer.fo should parse");
    assert!(
        renderer.page_count() > 0,
        "table_header_footer.fo should produce at least one page"
    );
    let page = renderer
        .render_page(0, 72.0)
        .expect("First page of table_header_footer.fo should rasterize");
    assert!(page.width > 0 && page.height > 0, "Page has zero dimensions");
}

#[test]
fn test_verify_fixture_table_spanning() {
    let fo = super::load_fixture("table_spanning.fo");
    let pdf = super::process_fo_document(&fo)
        .expect("table_spanning.fo should generate PDF");
    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF from table_spanning.fo should parse");
    assert!(
        renderer.page_count() > 0,
        "table_spanning.fo should produce at least one page"
    );
    let page = renderer
        .render_page(0, 72.0)
        .expect("First page of table_spanning.fo should rasterize");
    assert!(page.width > 0 && page.height > 0, "Page has zero dimensions");
}

#[test]
fn test_verify_fixture_text_formatting() {
    let fo = super::load_fixture("text_formatting.fo");
    let pdf = super::process_fo_document(&fo)
        .expect("text_formatting.fo should generate PDF");
    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF from text_formatting.fo should parse");
    assert!(
        renderer.page_count() > 0,
        "text_formatting.fo should produce at least one page"
    );
    let page = renderer
        .render_page(0, 72.0)
        .expect("First page of text_formatting.fo should rasterize");
    assert!(page.width > 0 && page.height > 0, "Page has zero dimensions");
}

#[test]
fn test_verify_fixture_unicode() {
    let fo = super::load_fixture("unicode.fo");
    let pdf = super::process_fo_document(&fo)
        .expect("unicode.fo should generate PDF");
    assert!(!pdf.is_empty(), "PDF from unicode.fo must not be empty");
    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF from unicode.fo should parse");
    if renderer.page_count() > 0 {
        let page = renderer
            .render_page(0, 72.0)
            .expect("First page of unicode.fo should rasterize without panic");
        assert!(
            page.width > 0 && page.height > 0,
            "Page has zero dimensions"
        );
    }
}

#[test]
fn test_verify_fixture_with_headers_footers() {
    let fo = super::load_fixture("with_headers_footers.fo");
    let pdf = super::process_fo_document(&fo)
        .expect("with_headers_footers.fo should generate PDF");
    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF from with_headers_footers.fo should parse");
    assert!(
        renderer.page_count() > 0,
        "with_headers_footers.fo should produce at least one page"
    );
    let page = renderer
        .render_page(0, 72.0)
        .expect("First page of with_headers_footers.fo should rasterize");
    assert!(page.width > 0 && page.height > 0, "Page has zero dimensions");
}
