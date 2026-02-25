//! Integration tests for PDF encryption
//!
//! Tests end-to-end encryption functionality through the CLI and library API.

use std::io::Cursor;
use std::process::Command;

#[test]
#[ignore = "slow: takes ~221s, run manually"]
fn test_cli_encryption_basic() {
    // Create a simple FO document
    let fo_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm" margin="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Encrypted Content Test</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

    // Write to temp file
    let temp_dir = std::env::temp_dir();
    let fo_path = temp_dir.join("test_encryption.fo");
    let pdf_path = temp_dir.join("test_encryption_out.pdf");

    std::fs::write(&fo_path, fo_content).expect("test: should succeed");

    // Run fop with encryption
    // Use cargo run instead of direct binary
    let output = Command::new("cargo")
        .args(["run", "--release", "-p", "fop-cli", "--"])
        .args([
            fo_path.to_str().expect("test: should succeed"),
            pdf_path.to_str().expect("test: should succeed"),
            "-o",
            "ownerpass",
            "-u",
            "userpass",
            "--noprint",
            "--nocopy",
        ])
        .output()
        .expect("Failed to execute fop");

    // Check it succeeded
    assert!(
        output.status.success(),
        "FOP command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    // Check PDF was created
    assert!(pdf_path.exists(), "PDF file was not created");

    // Read PDF bytes
    let pdf_bytes = std::fs::read(&pdf_path).expect("test: should succeed");

    // Verify PDF structure
    let pdf_str = String::from_utf8_lossy(&pdf_bytes);
    assert!(pdf_str.starts_with("%PDF-"), "Not a valid PDF");
    assert!(
        pdf_str.contains("/Filter /Standard"),
        "No encryption filter"
    );
    assert!(pdf_str.contains("/Length 128"), "Wrong encryption length");
    assert!(pdf_str.contains("/Encrypt"), "No encrypt reference");

    // Verify content is encrypted (not visible in plaintext)
    assert!(
        !pdf_str.contains("Encrypted Content Test"),
        "Content not encrypted!"
    );

    // Cleanup
    let _ = std::fs::remove_file(&fo_path);
    let _ = std::fs::remove_file(&pdf_path);
}

#[test]
fn test_encryption_permissions() {
    use fop_core::FoTreeBuilder;
    use fop_layout::LayoutEngine;
    use fop_render::{PdfPermissions, PdfRenderer, PdfSecurity};

    // Create FO document
    let fo_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm" margin="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Permission Test</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

    // Parse and layout
    let builder = FoTreeBuilder::new();
    let arena = builder
        .parse(Cursor::new(fo_content.as_bytes()))
        .expect("test: should succeed");
    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&arena).expect("test: should succeed");

    // Render with different permission sets
    let test_cases = vec![
        (
            "no_print_copy",
            PdfPermissions {
                allow_print: false,
                allow_copy: false,
                allow_modify: true,
                allow_annotations: true,
                ..Default::default()
            },
        ),
        (
            "no_edit",
            PdfPermissions {
                allow_print: true,
                allow_copy: true,
                allow_modify: false,
                allow_annotations: false,
                ..Default::default()
            },
        ),
        (
            "all_restricted",
            PdfPermissions {
                allow_print: false,
                allow_copy: false,
                allow_modify: false,
                allow_annotations: false,
                ..Default::default()
            },
        ),
    ];

    for (name, permissions) in test_cases {
        let renderer = PdfRenderer::new();
        let mut pdf_doc = renderer.render(&area_tree).expect("test: should succeed");

        // Apply encryption
        let security = PdfSecurity::new("owner", "user", permissions);
        let file_id = fop_render::pdf::security::generate_file_id(&format!("test-{}", name));
        let encryption_dict = security.compute_encryption_dict(&file_id);
        pdf_doc
            .set_encryption(encryption_dict, file_id)
            .expect("test: should succeed");

        // Generate PDF
        let pdf_bytes = pdf_doc.to_bytes().expect("test: should succeed");

        // Verify structure
        let pdf_str = String::from_utf8_lossy(&pdf_bytes);
        assert!(pdf_str.contains("/Encrypt"), "{}: Missing encryption", name);
        assert!(
            !pdf_str.contains("Permission Test"),
            "{}: Content not encrypted",
            name
        );
    }
}

#[test]
fn test_unencrypted_pdf_has_plaintext() {
    use fop_core::FoTreeBuilder;
    use fop_layout::LayoutEngine;
    use fop_render::PdfRenderer;

    let fo_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm" margin="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Plaintext Content</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

    let builder = FoTreeBuilder::new();
    let arena = builder
        .parse(Cursor::new(fo_content.as_bytes()))
        .expect("test: should succeed");
    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&arena).expect("test: should succeed");

    let renderer = PdfRenderer::new();
    let pdf_doc = renderer.render(&area_tree).expect("test: should succeed");

    // Generate PDF WITHOUT encryption
    let pdf_bytes = pdf_doc.to_bytes().expect("test: should succeed");
    let pdf_str = String::from_utf8_lossy(&pdf_bytes);

    // Should NOT have encryption
    assert!(!pdf_str.contains("/Encrypt"), "Should not be encrypted");
    assert!(
        !pdf_str.contains("/Filter /Standard"),
        "Should not have encryption filter"
    );

    // Should have plaintext content
    assert!(
        pdf_str.contains("Plaintext Content"),
        "Content should be visible"
    );
}
