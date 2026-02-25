//! PDF Encryption Example
//!
//! Demonstrates how to create encrypted PDFs with various security settings.
//!
//! This example shows:
//! - Owner and user passwords
//! - Permission flags (print, copy, edit, annotations)
//! - RC4-128 encryption
//! - Different security levels

use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use fop_render::{PdfPermissions, PdfRenderer, PdfSecurity};
use fop_types::Result;
use std::io::Cursor;

fn main() -> Result<()> {
    println!("PDF Encryption Example\n");

    // Create a simple XSL-FO document
    let fo_content = r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm" margin="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="18pt" font-weight="bold" space-after="10pt">
        Confidential Document
      </fo:block>
      <fo:block font-size="12pt" space-after="10pt">
        This document contains sensitive information and is encrypted
        with the following security settings:
      </fo:block>
      <fo:block font-size="12pt" margin-left="20pt" space-after="5pt">
        - Printing: Restricted
      </fo:block>
      <fo:block font-size="12pt" margin-left="20pt" space-after="5pt">
        - Copying: Restricted
      </fo:block>
      <fo:block font-size="12pt" margin-left="20pt" space-after="5pt">
        - Editing: Restricted
      </fo:block>
      <fo:block font-size="12pt" margin-left="20pt" space-after="10pt">
        - Annotations: Restricted
      </fo:block>
      <fo:block font-size="10pt" font-style="italic" color="#666666">
        Owner password required to change security settings.
      </fo:block>
      <fo:block font-size="10pt" font-style="italic" color="#666666">
        User password required to open the document.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    // Parse and layout
    println!("1. Parsing XSL-FO document...");
    let builder = FoTreeBuilder::new();
    let arena = builder.parse(Cursor::new(fo_content.as_bytes()))?;

    println!("2. Processing layout...");
    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&arena)?;

    println!("3. Generating encrypted PDFs...\n");

    // Example 1: Maximum Security - No permissions allowed
    println!("Example 1: Maximum Security");
    let renderer = PdfRenderer::new();
    let mut pdf_doc = renderer.render(&area_tree)?;

    let max_security = PdfPermissions {
        allow_print: false,
        allow_copy: false,
        allow_modify: false,
        allow_annotations: false,
        allow_fill_forms: false,
        allow_accessibility: false,
        allow_assemble: false,
        allow_print_high_quality: false,
    };

    apply_encryption(
        &mut pdf_doc,
        "owner123",
        "user123",
        max_security,
        "max_security",
    )?;
    std::fs::write("/tmp/encrypted_max_security.pdf", pdf_doc.to_bytes()?)?;
    println!("   → Saved to: /tmp/encrypted_max_security.pdf");
    println!("   → Owner password: owner123");
    println!("   → User password: user123");
    println!("   → Permissions: All restricted\n");

    // Example 2: View-Only - Print allowed, no editing
    println!("Example 2: View-Only (Print Allowed)");
    let renderer = PdfRenderer::new();
    let mut pdf_doc = renderer.render(&area_tree)?;

    let view_only = PdfPermissions {
        allow_print: true,
        allow_copy: false,
        allow_modify: false,
        allow_annotations: false,
        allow_fill_forms: false,
        allow_accessibility: true, // For screen readers
        allow_assemble: false,
        allow_print_high_quality: true,
    };

    apply_encryption(
        &mut pdf_doc,
        "ownerview",
        "userview",
        view_only,
        "view_only",
    )?;
    std::fs::write("/tmp/encrypted_view_only.pdf", pdf_doc.to_bytes()?)?;
    println!("   → Saved to: /tmp/encrypted_view_only.pdf");
    println!("   → Owner password: ownerview");
    println!("   → User password: userview");
    println!("   → Permissions: Print + Accessibility allowed\n");

    // Example 3: Collaboration Mode - Allow annotations and forms
    println!("Example 3: Collaboration Mode");
    let renderer = PdfRenderer::new();
    let mut pdf_doc = renderer.render(&area_tree)?;

    let collab = PdfPermissions {
        allow_print: true,
        allow_copy: false,
        allow_modify: false,
        allow_annotations: true, // Can add comments
        allow_fill_forms: true,  // Can fill forms
        allow_accessibility: true,
        allow_assemble: false,
        allow_print_high_quality: true,
    };

    apply_encryption(
        &mut pdf_doc,
        "ownercollab",
        "usercollab",
        collab,
        "collaboration",
    )?;
    std::fs::write("/tmp/encrypted_collaboration.pdf", pdf_doc.to_bytes()?)?;
    println!("   → Saved to: /tmp/encrypted_collaboration.pdf");
    println!("   → Owner password: ownercollab");
    println!("   → User password: usercollab");
    println!("   → Permissions: Print, Annotations, Forms allowed\n");

    // Example 4: No user password - Document opens without password but restricted
    println!("Example 4: No User Password (Restricted Operations)");
    let renderer = PdfRenderer::new();
    let mut pdf_doc = renderer.render(&area_tree)?;

    let restricted = PdfPermissions {
        allow_print: true,
        allow_copy: true,
        allow_modify: false,
        allow_annotations: false,
        ..Default::default()
    };

    apply_encryption(&mut pdf_doc, "owneronly", "", restricted, "no_user_pwd")?;
    std::fs::write("/tmp/encrypted_no_user_password.pdf", pdf_doc.to_bytes()?)?;
    println!("   → Saved to: /tmp/encrypted_no_user_password.pdf");
    println!("   → Owner password: owneronly");
    println!("   → User password: (none - opens without password)");
    println!("   → Permissions: View and print, no editing\n");

    println!("✓ All encrypted PDFs generated successfully!");
    println!("\nNotes:");
    println!("- RC4-128 bit encryption is used");
    println!("- Owner password controls security settings");
    println!("- User password is required to open the document");
    println!("- Permissions control what users can do after opening");

    Ok(())
}

/// Helper function to apply encryption to a PDF document
fn apply_encryption(
    pdf_doc: &mut fop_render::PdfDocument,
    owner_pwd: &str,
    user_pwd: &str,
    permissions: PdfPermissions,
    label: &str,
) -> Result<()> {
    let security = PdfSecurity::new(owner_pwd, user_pwd, permissions);
    let file_id = fop_render::pdf::security::generate_file_id(&format!("example-{}", label));
    let encryption_dict = security.compute_encryption_dict(&file_id);

    println!("   → Encryption key length: {} bits", encryption_dict.key_length);
    println!("   → P value: {}", encryption_dict.p_value);

    pdf_doc.set_encryption(encryption_dict, file_id)?;
    Ok(())
}
