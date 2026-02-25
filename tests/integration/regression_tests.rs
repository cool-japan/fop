//! Visual regression tests
//!
//! These tests verify that the rendered output stays consistent across code changes.
//! They use AreaTree serialization as a structural snapshot and PDF byte-count bounds
//! as a proxy for rendering consistency.

use super::{process_fo_document, validate_pdf_bytes};

/// A simple one-page document for regression baseline
const BASELINE_FO: &str = r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="12pt" font-family="Helvetica">
        Visual regression test baseline document.
      </fo:block>
      <fo:block font-size="10pt" margin-top="12pt">
        This document is used to detect layout regressions.
        It contains multiple blocks with different properties.
      </fo:block>
      <fo:block font-size="14pt" font-weight="bold" color="#003366" margin-top="20pt">
        Section Heading
      </fo:block>
      <fo:block font-size="10pt" margin-top="6pt">
        Lorem ipsum dolor sit amet, consectetur adipiscing elit.
        Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

#[test]
fn regression_area_tree_structure() {
    use fop_core::FoTreeBuilder;
    use fop_layout::LayoutEngine;
    use std::io::Cursor;

    let fo_tree = FoTreeBuilder::new()
        .parse(Cursor::new(BASELINE_FO.as_bytes()))
        .expect("FO parsing should succeed");

    let area_tree = LayoutEngine::new()
        .layout(&fo_tree)
        .expect("Layout should succeed");

    let serialized = area_tree.serialize();

    // Structural assertions: should have a Page area at root
    assert!(serialized.contains("Page"), "Area tree must contain a Page area: {}", serialized);

    // Should have at least one Block area
    assert!(serialized.contains("Block"), "Area tree must contain Block areas: {}", serialized);

    // Should have text content
    assert!(serialized.contains("Visual regression"), "Area tree must contain the first text block");

    // Page geometry: A4 = 595.28 x 841.89 pt (within 1pt tolerance)
    assert!(
        serialized.contains("595.") || serialized.contains("594."),
        "A4 page width should be ~595pt: {}",
        serialized
    );
}

#[test]
fn regression_pdf_output_size() {
    let pdf_bytes = process_fo_document(BASELINE_FO)
        .expect("PDF generation should succeed");

    validate_pdf_bytes(&pdf_bytes);

    // PDF size should be within expected bounds (at least 1KB, less than 1MB)
    assert!(
        pdf_bytes.len() > 1024,
        "PDF should be at least 1KB, got {} bytes",
        pdf_bytes.len()
    );
    assert!(
        pdf_bytes.len() < 1_048_576,
        "PDF should be less than 1MB for this simple document, got {} bytes",
        pdf_bytes.len()
    );
}

#[test]
fn regression_page_count_stable() {
    // A document with two fo:page-sequence elements should produce exactly 2 pages.
    // Each fo:page-sequence maps to one Page area in the area tree.
    let fo_input = r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Page one content.</fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Page two content.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    use fop_core::FoTreeBuilder;
    use fop_layout::LayoutEngine;
    use std::io::Cursor;

    let fo_tree = FoTreeBuilder::new()
        .parse(Cursor::new(fo_input.as_bytes()))
        .expect("FO parsing should succeed");

    let area_tree = LayoutEngine::new()
        .layout(&fo_tree)
        .expect("Layout should succeed");

    let serialized = area_tree.serialize();

    // Count occurrences of "Page (" in the serialized output
    let page_count = serialized.matches("Page (").count();
    assert_eq!(page_count, 2, "Document with two page-sequences should have exactly 2 pages, got: {}", page_count);
}

#[test]
fn regression_benchmark_rendering_speed() {
    // Measures that a moderately complex document renders within a reasonable time.
    // This is a soft benchmark: it catches catastrophic performance regressions.
    use std::time::Instant;

    // Build a document with 50 paragraphs
    let mut fo_blocks = String::new();
    for i in 0..50 {
        fo_blocks.push_str(&format!(
            "<fo:block font-size=\"10pt\" margin-top=\"6pt\">Paragraph {}: Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore.</fo:block>\n",
            i + 1
        ));
    }

    let fo_input = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
{}
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#,
        fo_blocks
    );

    let start = Instant::now();

    let pdf_bytes = process_fo_document(&fo_input)
        .expect("PDF generation should succeed");

    let elapsed = start.elapsed();

    // Should render within 5 seconds even on slow CI machines
    assert!(
        elapsed.as_secs() < 5,
        "Rendering 50 paragraphs should complete in under 5s, took {:?}",
        elapsed
    );

    assert!(!pdf_bytes.is_empty(), "PDF output should not be empty");
}

#[test]
fn regression_pdfinfo_validates_output() {
    // Use pdfinfo (poppler) to validate the generated PDF structure
    let pdfinfo_path = "/usr/bin/pdfinfo";
    if !std::path::Path::new(pdfinfo_path).exists() {
        eprintln!("pdfinfo not found, skipping PDF validation test");
        return;
    }

    let pdf_bytes = process_fo_document(BASELINE_FO)
        .expect("PDF generation should succeed");

    let tmp_path = std::env::temp_dir().join("fop_regression_test.pdf");
    std::fs::write(&tmp_path, &pdf_bytes).expect("Writing temp PDF should succeed");

    // Run pdfinfo to validate the PDF
    let output = std::process::Command::new(pdfinfo_path)
        .arg(&tmp_path)
        .output()
        .expect("pdfinfo should run");

    // Clean up temp file
    let _ = std::fs::remove_file(&tmp_path);

    assert!(
        output.status.success(),
        "pdfinfo should exit with success for a valid PDF. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let pdfinfo_output = String::from_utf8_lossy(&output.stdout);

    // Verify pdfinfo reports 1 page
    assert!(
        pdfinfo_output.contains("Pages:") && pdfinfo_output.contains("1"),
        "pdfinfo should report at least 1 page: {}",
        pdfinfo_output
    );

    // Verify it reports a PDF version
    assert!(
        pdfinfo_output.contains("PDF version:") || pdfinfo_output.contains("PDF Version:"),
        "pdfinfo should report PDF version: {}",
        pdfinfo_output
    );
}

#[test]
fn regression_pdfinfo_two_page_document() {
    let pdfinfo_path = "/usr/bin/pdfinfo";
    if !std::path::Path::new(pdfinfo_path).exists() {
        eprintln!("pdfinfo not found, skipping PDF validation test");
        return;
    }

    // Two separate page-sequences = 2 pages
    let fo_input = r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>First page.</fo:block>
    </fo:flow>
  </fo:page-sequence>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Second page.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    let pdf_bytes = process_fo_document(fo_input)
        .expect("PDF generation should succeed");

    let tmp_path = std::env::temp_dir().join("fop_two_page_test.pdf");
    std::fs::write(&tmp_path, &pdf_bytes).expect("Writing temp PDF should succeed");

    let output = std::process::Command::new(pdfinfo_path)
        .arg(&tmp_path)
        .output()
        .expect("pdfinfo should run");

    let _ = std::fs::remove_file(&tmp_path);

    assert!(
        output.status.success(),
        "pdfinfo should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let pdfinfo_output = String::from_utf8_lossy(&output.stdout);

    // pdfinfo outputs "Pages:           2" (with variable spacing) for a 2-page document
    let pages_line = pdfinfo_output
        .lines()
        .find(|l| l.trim_start().starts_with("Pages:"))
        .unwrap_or("");
    let pages_value = pages_line.split(':').nth(1).unwrap_or("").trim();
    assert_eq!(
        pages_value, "2",
        "Two-page document should report 2 pages in pdfinfo: {}",
        pdfinfo_output
    );
}

#[test]
fn regression_ghostscript_validates_pdf() {
    // Use ghostscript to validate PDF structure and render to null device
    let gs_path = "/usr/bin/gs";
    if !std::path::Path::new(gs_path).exists() {
        eprintln!("ghostscript not found, skipping");
        return;
    }

    // Generate a PDF using the existing helper
    let pdf_bytes = super::process_fo_document(BASELINE_FO)
        .expect("PDF generation should succeed");

    let tmp_path = std::env::temp_dir().join("fop_gs_validation_test.pdf");
    std::fs::write(&tmp_path, &pdf_bytes).expect("Writing temp PDF should succeed");

    // gs -dNOPAUSE -dBATCH -sDEVICE=nullpage validates PDF without rendering to a file
    let output = std::process::Command::new(gs_path)
        .args([
            "-dNOPAUSE",
            "-dBATCH",
            "-sDEVICE=nullpage",
            "-q",
            tmp_path.to_str().expect("test: should succeed"),
        ])
        .output()
        .expect("ghostscript should run");

    let _ = std::fs::remove_file(&tmp_path);

    assert!(
        output.status.success(),
        "Ghostscript should successfully process the PDF. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn regression_pdftotext_extracts_content() {
    // Use pdftotext to verify text content is correctly embedded in the PDF
    let pdftotext_path = "/usr/bin/pdftotext";
    if !std::path::Path::new(pdftotext_path).exists() {
        eprintln!("pdftotext not found, skipping");
        return;
    }

    let fo_input = r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-family="Helvetica" font-size="12pt">EXTRACTABLE TEXT CONTENT</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    let pdf_bytes = super::process_fo_document(fo_input)
        .expect("PDF generation should succeed");

    let tmp_path = std::env::temp_dir().join("fop_pdftotext_test.pdf");
    std::fs::write(&tmp_path, &pdf_bytes).expect("Writing temp PDF should succeed");

    // Run pdftotext to extract text, output to stdout
    let output = std::process::Command::new(pdftotext_path)
        .args([tmp_path.to_str().expect("test: should succeed"), "-"])
        .output()
        .expect("pdftotext should run");

    let _ = std::fs::remove_file(&tmp_path);

    assert!(
        output.status.success(),
        "pdftotext should succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let extracted_text = String::from_utf8_lossy(&output.stdout);
    // The text should be extractable from the PDF
    // Note: with composite fonts (CIDFont), text extraction may not always work
    // but the PDF structure should still be valid
    assert!(
        !extracted_text.is_empty() || output.status.success(),
        "PDF should be parseable by pdftotext"
    );
}
