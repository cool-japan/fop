//! Table layout integration tests

use super::{load_fixture, process_fo_document, validate_pdf_bytes};

#[test]
fn test_simple_table_with_borders() {
    let fo_content = load_fixture("table_simple.fo");
    let pdf_bytes = process_fo_document(&fo_content).expect("Processing failed");

    validate_pdf_bytes(&pdf_bytes);

    // Tables should produce substantial PDF content
    assert!(
        pdf_bytes.len() > 300,
        "Table PDF too small: {} bytes",
        pdf_bytes.len()
    );
}

#[test]
fn test_table_with_column_spanning() {
    let fo_content = load_fixture("table_spanning.fo");
    let pdf_bytes = process_fo_document(&fo_content).expect("Processing failed");

    validate_pdf_bytes(&pdf_bytes);

    // Column spanning adds complexity
    assert!(pdf_bytes.len() > 300);
}

#[test]
fn test_table_with_header_and_footer() {
    let fo_content = load_fixture("table_header_footer.fo");
    let pdf_bytes = process_fo_document(&fo_content).expect("Processing failed");

    validate_pdf_bytes(&pdf_bytes);

    // Header and footer should be rendered
    assert!(pdf_bytes.len() > 400);
}

#[test]
fn test_invoice_table_layout() {
    // Reuse invoice fixture which has a complex table
    let fo_content = load_fixture("simple_invoice.fo");
    let pdf_bytes = process_fo_document(&fo_content).expect("Processing failed");

    validate_pdf_bytes(&pdf_bytes);

    // Invoice table has header and multiple rows
    assert!(pdf_bytes.len() > 400);
}
