//! Integration tests for FOP CLI

#![allow(deprecated)]

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

const SIMPLE_FO: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
                          page-width="210mm"
                          page-height="297mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Test</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

#[test]
fn test_help_command() {
    let mut cmd = Command::cargo_bin("fop").expect("test: should succeed");
    cmd.arg("--help");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Apache FOP"))
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn test_version_command() {
    let mut cmd = Command::cargo_bin("fop").expect("test: should succeed");
    cmd.arg("--version");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Apache FOP"))
        .stdout(predicate::str::contains("Version:"));
}

#[test]
fn test_basic_conversion() {
    let temp_dir = TempDir::new().expect("test: should succeed");
    let input_path = temp_dir.path().join("input.fo");
    let output_path = temp_dir.path().join("output.pdf");

    // Write test FO file
    fs::write(&input_path, SIMPLE_FO).expect("test: should succeed");

    // Run conversion
    let mut cmd = Command::cargo_bin("fop").expect("test: should succeed");
    cmd.arg(&input_path).arg(&output_path);

    cmd.assert().success();

    // Check output file exists
    assert!(output_path.exists());

    // Check output file has content
    let metadata = fs::metadata(&output_path).expect("test: should succeed");
    assert!(metadata.len() > 0);
}

#[test]
fn test_validation_only() {
    let temp_dir = TempDir::new().expect("test: should succeed");
    let input_path = temp_dir.path().join("input.fo");

    // Write test FO file
    fs::write(&input_path, SIMPLE_FO).expect("test: should succeed");

    // Run validation
    let mut cmd = Command::cargo_bin("fop").expect("test: should succeed");
    cmd.arg(&input_path).arg("--validate-only");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("valid"));
}

#[test]
fn test_stats_option() {
    let temp_dir = TempDir::new().expect("test: should succeed");
    let input_path = temp_dir.path().join("input.fo");
    let output_path = temp_dir.path().join("output.pdf");

    // Write test FO file
    fs::write(&input_path, SIMPLE_FO).expect("test: should succeed");

    // Run conversion with stats
    let mut cmd = Command::cargo_bin("fop").expect("test: should succeed");
    cmd.arg(&input_path).arg(&output_path).arg("--stats");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Statistics"))
        .stdout(predicate::str::contains("Parsing"))
        .stdout(predicate::str::contains("Layout"))
        .stdout(predicate::str::contains("Rendering"));
}

#[test]
fn test_missing_input_file() {
    let temp_dir = TempDir::new().expect("test: should succeed");
    let output_path = temp_dir.path().join("output.pdf");

    // Try to convert non-existent file
    let mut cmd = Command::cargo_bin("fop").expect("test: should succeed");
    cmd.arg("nonexistent.fo").arg(&output_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("does not exist"));
}

#[test]
fn test_missing_output_file() {
    let temp_dir = TempDir::new().expect("test: should succeed");
    let input_path = temp_dir.path().join("input.fo");

    // Write test FO file
    fs::write(&input_path, SIMPLE_FO).expect("test: should succeed");

    // Try to convert without output file (and not validation-only)
    let mut cmd = Command::cargo_bin("fop").expect("test: should succeed");
    cmd.arg(&input_path);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_quiet_mode() {
    let temp_dir = TempDir::new().expect("test: should succeed");
    let input_path = temp_dir.path().join("input.fo");
    let output_path = temp_dir.path().join("output.pdf");

    // Write test FO file
    fs::write(&input_path, SIMPLE_FO).expect("test: should succeed");

    // Run conversion in quiet mode
    let mut cmd = Command::cargo_bin("fop").expect("test: should succeed");
    cmd.arg(&input_path).arg(&output_path).arg("--quiet");

    cmd.assert().success();

    // Output should be minimal in quiet mode
    let output = cmd.output().expect("test: should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("Apache FOP"));
}

#[test]
fn test_invalid_pdf_version() {
    let temp_dir = TempDir::new().expect("test: should succeed");
    let input_path = temp_dir.path().join("input.fo");
    let output_path = temp_dir.path().join("output.pdf");

    // Write test FO file
    fs::write(&input_path, SIMPLE_FO).expect("test: should succeed");

    // Try invalid PDF version
    let mut cmd = Command::cargo_bin("fop").expect("test: should succeed");
    cmd.arg(&input_path)
        .arg(&output_path)
        .arg("--pdf-version")
        .arg("99.9");

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid PDF version"));
}

#[test]
fn test_json_output_format() {
    let temp_dir = TempDir::new().expect("test: should succeed");
    let input_path = temp_dir.path().join("input.fo");
    let output_path = temp_dir.path().join("output.pdf");

    // Write test FO file
    fs::write(&input_path, SIMPLE_FO).expect("test: should succeed");

    // Run conversion with JSON stats
    let mut cmd = Command::cargo_bin("fop").expect("test: should succeed");
    cmd.arg(&input_path)
        .arg(&output_path)
        .arg("--stats")
        .arg("--output-format")
        .arg("json");

    cmd.assert()
        .success()
        .stdout(predicate::str::contains(r#""parsing""#))
        .stdout(predicate::str::contains(r#""layout""#))
        .stdout(predicate::str::contains(r#""rendering""#));
}

#[test]
fn test_render_verify_flag_succeeds_for_valid_pdf() {
    let temp_dir = TempDir::new().expect("test: should succeed");
    let input_path = temp_dir.path().join("input.fo");
    let output_path = temp_dir.path().join("test_render_verify.pdf");

    fs::write(&input_path, SIMPLE_FO).expect("test: should succeed");

    Command::cargo_bin("fop")
        .expect("test: should succeed")
        .arg(&input_path)
        .arg(&output_path)
        .arg("--render-verify")
        .assert()
        .success();

    // Clean up output file
    let _ = fs::remove_file(&output_path);
}

#[test]
fn test_render_verify_flag_no_op_for_non_pdf_output() {
    let temp_dir = TempDir::new().expect("test: should succeed");
    let input_path = temp_dir.path().join("input.fo");
    let output_path = temp_dir.path().join("test_render_verify_noop.svg");

    fs::write(&input_path, SIMPLE_FO).expect("test: should succeed");

    // --render-verify with SVG output should be a silent no-op (success, no error)
    Command::cargo_bin("fop")
        .expect("test: should succeed")
        .arg(&input_path)
        .arg(&output_path)
        .arg("--render-verify")
        .assert()
        .success();

    // Clean up output file
    let _ = fs::remove_file(&output_path);
}
