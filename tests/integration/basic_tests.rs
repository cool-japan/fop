//! Basic document integration tests

use super::{load_fixture, process_fo_document, validate_pdf_bytes};

#[test]
fn test_simple_single_page() {
    let fo_content = load_fixture("simple_single_page.fo");
    let pdf_bytes = process_fo_document(&fo_content).expect("Processing failed");

    validate_pdf_bytes(&pdf_bytes);

    // Verify it's a reasonable size
    assert!(
        pdf_bytes.len() > 200,
        "PDF seems too small: {} bytes",
        pdf_bytes.len()
    );
}

#[test]
fn test_multi_page_document() {
    let fo_content = load_fixture("multi_page.fo");
    let pdf_bytes = process_fo_document(&fo_content).expect("Processing failed");

    validate_pdf_bytes(&pdf_bytes);

    // Multi-page document should be larger
    assert!(
        pdf_bytes.len() > 300,
        "PDF seems too small for multi-page: {} bytes",
        pdf_bytes.len()
    );
}

#[test]
fn test_text_formatting() {
    let fo_content = load_fixture("text_formatting.fo");
    let pdf_bytes = process_fo_document(&fo_content).expect("Processing failed");

    validate_pdf_bytes(&pdf_bytes);
}

#[test]
fn test_nested_blocks_with_inheritance() {
    let fo_content = load_fixture("nested_blocks.fo");
    let pdf_bytes = process_fo_document(&fo_content).expect("Processing failed");

    validate_pdf_bytes(&pdf_bytes);

    // Should handle nested blocks properly
    assert!(pdf_bytes.len() > 200);
}

#[test]
fn test_simple_invoice() {
    let fo_content = load_fixture("simple_invoice.fo");
    let pdf_bytes = process_fo_document(&fo_content).expect("Processing failed");

    validate_pdf_bytes(&pdf_bytes);

    // Invoice has table, should be substantial
    assert!(
        pdf_bytes.len() > 400,
        "Invoice PDF too small: {} bytes",
        pdf_bytes.len()
    );
}

#[test]
fn test_empty_document() {
    let fo_content = load_fixture("empty_document.fo");
    let pdf_bytes = process_fo_document(&fo_content).expect("Processing failed");

    validate_pdf_bytes(&pdf_bytes);

    // Even empty document should produce valid PDF structure
    assert!(pdf_bytes.len() > 100);
}

#[test]
fn test_long_text_line_breaking() {
    let fo_content = load_fixture("long_text.fo");
    let pdf_bytes = process_fo_document(&fo_content).expect("Processing failed");

    validate_pdf_bytes(&pdf_bytes);

    // Should handle long paragraphs with line breaking
    assert!(pdf_bytes.len() > 300);
}

#[test]
fn test_deeply_nested_elements() {
    let fo_content = load_fixture("deeply_nested.fo");
    let pdf_bytes = process_fo_document(&fo_content).expect("Processing failed");

    validate_pdf_bytes(&pdf_bytes);

    // Should handle 10 levels of nesting
    assert!(pdf_bytes.len() > 200);
}

#[test]
fn test_unicode_characters() {
    let fo_content = load_fixture("unicode.fo");
    let pdf_bytes = process_fo_document(&fo_content).expect("Processing failed");

    validate_pdf_bytes(&pdf_bytes);

    // Unicode content should be processed
    assert!(pdf_bytes.len() > 300);
}

#[test]
fn test_document_with_headers_and_footers() {
    let fo_content = load_fixture("with_headers_footers.fo");
    let pdf_bytes = process_fo_document(&fo_content).expect("Processing failed");

    validate_pdf_bytes(&pdf_bytes);

    // Headers and footers add structure
    assert!(pdf_bytes.len() > 300);
}

#[test]
fn test_footnotes() {
    let fo_content = load_fixture("footnote_basic.fo");
    let pdf_bytes = process_fo_document(&fo_content).expect("Footnote processing failed");

    validate_pdf_bytes(&pdf_bytes);

    // Document with footnotes should produce valid output
    assert!(
        pdf_bytes.len() > 200,
        "PDF seems too small: {} bytes",
        pdf_bytes.len()
    );
}
