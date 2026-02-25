//! Production deployment example for FOP CLI
//!
//! This example demonstrates how to use the FOP CLI in a production environment
//! with proper error handling, logging, and monitoring.

use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== FOP CLI Production Deployment Example ===\n");

    // Example 1: Basic batch processing
    batch_conversion_example()?;

    // Example 2: Error handling and recovery
    error_handling_example()?;

    // Example 3: Monitoring and metrics
    monitoring_example()?;

    // Example 4: CI/CD integration
    ci_cd_example()?;

    println!("\n=== All Examples Complete ===");
    Ok(())
}

/// Example 1: Batch processing multiple documents
fn batch_conversion_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("Example 1: Batch Processing");
    println!("----------------------------");

    // Create sample documents
    let docs = vec![
        ("report1.fo", create_sample_fo("Report 1")),
        ("report2.fo", create_sample_fo("Report 2")),
        ("report3.fo", create_sample_fo("Report 3")),
    ];

    for (name, content) in &docs {
        let input_path = format!("/tmp/{}", name);
        fs::write(&input_path, content)?;
    }

    // Process batch
    println!("Processing {} documents...", docs.len());
    let mut success_count = 0;
    let mut error_count = 0;

    for (name, _) in &docs {
        let input_path = format!("/tmp/{}", name);
        let output_path = format!("/tmp/{}.pdf", name.trim_end_matches(".fo"));

        // Run fop CLI
        let status = Command::new("cargo")
            .args([
                "run",
                "--package",
                "fop-cli",
                "--release",
                "--",
                &input_path,
                &output_path,
                "--quiet",
            ])
            .stdout(Stdio::null())
            .status()?;

        if status.success() {
            success_count += 1;
            println!("  ✓ {}: {} → {}", name, input_path, output_path);
        } else {
            error_count += 1;
            eprintln!("  ✗ {}: Failed", name);
        }
    }

    println!(
        "\nBatch complete: {} success, {} errors\n",
        success_count, error_count
    );

    Ok(())
}

/// Example 2: Error handling and recovery
fn error_handling_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("Example 2: Error Handling");
    println!("-------------------------");

    // Create a valid document
    let valid_doc = "/tmp/valid_doc.fo";
    fs::write(valid_doc, create_sample_fo("Valid Document"))?;

    // Create an invalid document
    let invalid_doc = "/tmp/invalid_doc.fo";
    fs::write(invalid_doc, "<?xml version=\"1.0\"?><invalid>")?;

    // Try to process invalid document
    println!("Processing invalid document...");
    let output = Command::new("cargo")
        .args([
            "run",
            "--package",
            "fop-cli",
            "--release",
            "--",
            invalid_doc,
            "/tmp/invalid_output.pdf",
        ])
        .output()?;

    if !output.status.success() {
        println!("  ✓ Error correctly detected");
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("  Error message: {}", stderr.lines().next().unwrap_or(""));
    }

    // Process valid document with validation first
    println!("\nValidating document before conversion...");
    let status = Command::new("cargo")
        .args([
            "run",
            "--package",
            "fop-cli",
            "--release",
            "--",
            valid_doc,
            "--validate-only",
        ])
        .stdout(Stdio::null())
        .status()?;

    if status.success() {
        println!("  ✓ Validation passed");

        // Now convert
        let status = Command::new("cargo")
            .args([
                "run",
                "--package",
                "fop-cli",
                "--release",
                "--",
                valid_doc,
                "/tmp/valid_output.pdf",
                "--quiet",
            ])
            .status()?;

        if status.success() {
            println!("  ✓ Conversion successful\n");
        }
    }

    Ok(())
}

/// Example 3: Monitoring and metrics collection
fn monitoring_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("Example 3: Monitoring and Metrics");
    println!("----------------------------------");

    let input_doc = "/tmp/monitoring_doc.fo";
    fs::write(input_doc, create_sample_fo("Monitoring Test"))?;

    // Get metrics in JSON format
    println!("Collecting metrics...");
    let output = Command::new("cargo")
        .args([
            "run",
            "--package",
            "fop-cli",
            "--release",
            "--",
            input_doc,
            "/tmp/monitoring_output.pdf",
            "--stats",
            "--output-format",
            "json",
            "--quiet",
        ])
        .output()?;

    if output.status.success() {
        let metrics = String::from_utf8_lossy(&output.stdout);
        println!("  Metrics (JSON):");

        // Parse and display key metrics
        for line in metrics.lines() {
            if line.contains("duration_ms") || line.contains("nodes") || line.contains("pages") {
                println!("  {}", line.trim());
            }
        }
    }

    println!();
    Ok(())
}

/// Example 4: CI/CD integration
fn ci_cd_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("Example 4: CI/CD Integration");
    println!("----------------------------");

    // Create a shell script for CI/CD
    let script = r#"#!/bin/bash
# CI/CD script for FOP document generation

set -e  # Exit on error

INPUT_DIR="${INPUT_DIR:-./documents}"
OUTPUT_DIR="${OUTPUT_DIR:-./output}"
BUILD_ID="${BUILD_ID:-local}"

echo "=== FOP Document Generation ==="
echo "Build ID: $BUILD_ID"
echo "Input:    $INPUT_DIR"
echo "Output:   $OUTPUT_DIR"
echo ""

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Process all .fo files
SUCCESS=0
FAILED=0

for fo_file in "$INPUT_DIR"/*.fo; do
    if [ -f "$fo_file" ]; then
        filename=$(basename "$fo_file" .fo)
        output_file="$OUTPUT_DIR/${filename}.pdf"

        echo "Processing: $filename"

        # Run FOP with quiet mode for CI
        if fop "$fo_file" "$output_file" --quiet --compress --fail-fast; then
            SUCCESS=$((SUCCESS + 1))
            echo "  ✓ Success: $output_file"
        else
            FAILED=$((FAILED + 1))
            echo "  ✗ Failed: $filename" >&2
        fi
    fi
done

echo ""
echo "=== Summary ==="
echo "Success: $SUCCESS"
echo "Failed:  $FAILED"
echo ""

# Exit with error if any failed
if [ $FAILED -gt 0 ]; then
    echo "ERROR: $FAILED documents failed to convert" >&2
    exit 1
fi

echo "All documents converted successfully"
exit 0
"#;

    // Write CI script
    let script_path = "/tmp/fop_ci.sh";
    let mut file = fs::File::create(script_path)?;
    file.write_all(script.as_bytes())?;

    println!("Created CI/CD script: {}", script_path);
    println!("\nUsage:");
    println!("  export INPUT_DIR=./my-documents");
    println!("  export OUTPUT_DIR=./pdfs");
    println!("  export BUILD_ID=build-123");
    println!("  bash {}", script_path);

    // Also create a GitHub Actions example
    let github_workflow = r#"
name: Generate PDFs

on:
  push:
    paths:
      - 'documents/**/*.fo'

jobs:
  generate-pdfs:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Build FOP CLI
        run: cargo build --release --package fop-cli

      - name: Generate PDFs
        run: |
          mkdir -p output
          for fo_file in documents/*.fo; do
            filename=$(basename "$fo_file" .fo)
            ./target/release/fop "$fo_file" "output/${filename}.pdf" \
              --quiet --compress --stats --output-format json > "output/${filename}.json"
          done

      - name: Upload PDFs
        uses: actions/upload-artifact@v3
        with:
          name: generated-pdfs
          path: output/*.pdf

      - name: Upload Metrics
        uses: actions/upload-artifact@v3
        with:
          name: conversion-metrics
          path: output/*.json
"#;

    let workflow_path = "/tmp/fop_github_workflow.yml";
    fs::write(workflow_path, github_workflow)?;

    println!("\nCreated GitHub Actions workflow: {}", workflow_path);
    println!("\nCI/CD examples created successfully\n");

    Ok(())
}

/// Create a sample XSL-FO document
fn create_sample_fo(title: &str) -> String {
    format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
                          page-width="210mm"
                          page-height="297mm"
                          margin="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="18pt" font-weight="bold" space-after="10pt">
        {}
      </fo:block>
      <fo:block font-size="12pt">
        This is a sample document for testing the FOP CLI.
      </fo:block>
      <fo:block font-size="10pt" color="#666666" space-before="10pt">
        Generated by Apache FOP Rust Implementation
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        title
    )
}
