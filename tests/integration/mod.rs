//! Integration tests for the FOP XSL-FO processor
//!
//! This module contains comprehensive end-to-end tests that verify
//! the complete pipeline: Parse → Layout → Render

mod advanced_tests;
mod basic_tests;
mod cli_tests;
// conformance_tests split into multiple modules (each <2000 lines per policy)
mod conformance_advanced;
mod conformance_basic;
mod conformance_cli_docs;
mod conformance_complex_scenarios;
mod conformance_direction;
mod conformance_edge_cases;
mod conformance_layout;
mod conformance_output_formats;
mod conformance_pdf_renderer;
mod conformance_pdfa;
mod conformance_properties;
mod conformance_realworld;
mod conformance_table_extended;
mod error_tests;
mod i18n_tests;
mod real_world_tests;
mod table_tests;
mod text_flow_tests;
mod regression_tests;
mod verify_tests;

use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use fop_render::{PdfRenderer, PdfValidator};
use std::fs;
use std::io::Cursor;

/// Helper function to run the complete FOP pipeline on an XSL-FO document
pub fn process_fo_document(fo_content: &str) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Parse
    let builder = FoTreeBuilder::new();
    let cursor = Cursor::new(fo_content.as_bytes());
    let arena = builder.parse(cursor)?;

    // Layout
    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&arena)?;

    // Render to PDF (with bookmark/outline support from FO tree)
    let renderer = PdfRenderer::new();
    let pdf_doc = renderer.render_with_fo(&area_tree, &arena)?;
    let bytes = pdf_doc.to_bytes()?;

    Ok(bytes)
}

/// Helper function to load a fixture file
pub fn load_fixture(filename: &str) -> String {
    let path = format!("tests/integration/fixtures/{}", filename);
    fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to load fixture {}: {}", path, e))
}

/// Validate that PDF bytes are well-formed
pub fn validate_pdf_bytes(bytes: &[u8]) {
    assert!(bytes.len() > 100, "PDF too small: {} bytes", bytes.len());
    assert!(
        bytes.starts_with(b"%PDF-"),
        "Invalid PDF header: {:?}",
        &bytes[..20]
    );

    // Optional: Use PdfValidator for more thorough validation
    let validator = PdfValidator::new();
    let validation_result = validator.validate_pdf(bytes);
    if validation_result.has_errors() {
        eprintln!("PDF validation errors: {:?}", validation_result.issues());
    }
}

/// Verify that a PDF renders correctly (non-blank) using fop-pdf-renderer
pub fn verify_pdf_rendering(pdf: &[u8], expected_pages: usize) {
    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(pdf).expect("PDF parse failed");
    assert_eq!(
        renderer.page_count(),
        expected_pages,
        "Expected {} pages, got {}",
        expected_pages,
        renderer.page_count()
    );
    for i in 0..renderer.page_count() {
        let page = renderer
            .render_page(i, 72.0)
            .unwrap_or_else(|e| panic!("Failed to render page {}: {}", i, e));
        assert!(page.width > 0, "Page {} has zero width", i);
        assert!(page.height > 0, "Page {} has zero height", i);
    }
}

/// Helper function to run the complete FOP pipeline with a specific output format
/// format: "pdf", "ps", "text", "svg"
pub fn process_fo_document_format(
    fo_content: &str,
    format: &str,
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    // Parse
    let builder = FoTreeBuilder::new();
    let cursor = Cursor::new(fo_content.as_bytes());
    let arena = builder.parse(cursor)?;

    // Layout
    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&arena)?;

    match format {
        "ps" => {
            use fop_render::PsRenderer;
            let renderer = PsRenderer::new();
            let ps_string = renderer.render_to_ps(&area_tree)?;
            Ok(ps_string.into_bytes())
        }
        "text" => {
            use fop_render::TextRenderer;
            let renderer = TextRenderer::new();
            let text_string = renderer.render_to_text(&area_tree)?;
            Ok(text_string.into_bytes())
        }
        "svg" => {
            use fop_render::SvgRenderer;
            let renderer = SvgRenderer::new();
            let svg_string = renderer.render_to_svg(&area_tree)?;
            Ok(svg_string.into_bytes())
        }
        _ => {
            // Default: PDF
            let renderer = fop_render::PdfRenderer::new();
            let pdf_doc = renderer.render_with_fo(&area_tree, &arena)?;
            let bytes = pdf_doc.to_bytes()?;
            Ok(bytes)
        }
    }
}
