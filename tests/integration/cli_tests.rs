//! CLI integration tests - test the fop command-line interface

use std::process::Command;

fn fop_binary() -> std::path::PathBuf {
    // Use the debug build binary
    let mut p = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.push("target/debug/fop");
    p
}

fn simple_fo_doc() -> &'static str {
    r##"<?xml version="1.0" encoding="UTF-8"?>
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
      <fo:block>Hello from CLI test</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##
}

#[test]
fn cli_converts_fo_to_pdf() {
    let binary = fop_binary();
    if !binary.exists() {
        // Skip if not built yet
        return;
    }

    // Write FO to temp file
    let fo_path = std::env::temp_dir().join("cli_test_input.fo");
    let pdf_path = std::env::temp_dir().join("cli_test_output.pdf");

    std::fs::write(&fo_path, simple_fo_doc()).expect("write FO file");

    let output = Command::new(&binary)
        .arg(fo_path.to_str().expect("test: should succeed"))
        .arg(pdf_path.to_str().expect("test: should succeed"))
        .output()
        .expect("run fop binary");

    assert!(
        output.status.success(),
        "fop CLI should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let pdf_bytes = std::fs::read(&pdf_path).expect("read output PDF");
    assert!(!pdf_bytes.is_empty(), "PDF should not be empty");
    assert!(pdf_bytes.starts_with(b"%PDF-"), "Should be valid PDF");

    // Cleanup
    let _ = std::fs::remove_file(&fo_path);
    let _ = std::fs::remove_file(&pdf_path);
}

#[test]
fn cli_converts_to_svg() {
    let binary = fop_binary();
    if !binary.exists() {
        return;
    }

    let fo_path = std::env::temp_dir().join("cli_test_svg_input.fo");
    let svg_path = std::env::temp_dir().join("cli_test_svg_output.svg");

    std::fs::write(&fo_path, simple_fo_doc()).expect("write FO file");

    let output = Command::new(&binary)
        .arg(fo_path.to_str().expect("test: should succeed"))
        .arg(svg_path.to_str().expect("test: should succeed"))
        .output()
        .expect("run fop binary");

    assert!(
        output.status.success(),
        "fop CLI SVG conversion should succeed. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let svg_bytes = std::fs::read(&svg_path).expect("read output SVG");
    assert!(!svg_bytes.is_empty(), "SVG should not be empty");

    let _ = std::fs::remove_file(&fo_path);
    let _ = std::fs::remove_file(&svg_path);
}

#[test]
fn cli_fop_style_flags() {
    // Test --fo/--pdf long flags (double-dash style)
    let binary = fop_binary();
    if !binary.exists() {
        return;
    }

    let fo_path = std::env::temp_dir().join("cli_test_compat_input.fo");
    let pdf_path = std::env::temp_dir().join("cli_test_compat_output.pdf");

    std::fs::write(&fo_path, simple_fo_doc()).expect("write FO file");

    let output = Command::new(&binary)
        .arg("--fo")
        .arg(fo_path.to_str().expect("test: should succeed"))
        .arg("--pdf")
        .arg(pdf_path.to_str().expect("test: should succeed"))
        .output()
        .expect("run fop binary");

    assert!(
        output.status.success(),
        "--fo/--pdf flags should work. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let _ = std::fs::remove_file(&fo_path);
    let _ = std::fs::remove_file(&pdf_path);
}

#[test]
fn cli_validate_only() {
    // --validate-only should not produce output file
    let binary = fop_binary();
    if !binary.exists() {
        return;
    }

    let fo_path = std::env::temp_dir().join("cli_test_validate.fo");
    std::fs::write(&fo_path, simple_fo_doc()).expect("write FO file");

    let output = Command::new(&binary)
        .arg(fo_path.to_str().expect("test: should succeed"))
        .arg("--validate-only")
        .output()
        .expect("run fop binary");

    // Validate-only should succeed (exit 0) for valid FO
    assert!(
        output.status.success(),
        "--validate-only should succeed for valid FO. stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let _ = std::fs::remove_file(&fo_path);
}
