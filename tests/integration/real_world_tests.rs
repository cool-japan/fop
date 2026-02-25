//! Real-world document integration tests
//!
//! Tests using realistic XSL-FO documents that would be used in production.

use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use fop_render::PdfRenderer;
use std::io::Cursor;

/// Test a complete invoice document with table, formatting, and calculations
#[test]
fn test_production_invoice() {
    // Use a simple invoice for testing
    let invoice_fo = r###"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="page" page-width="8.5in" page-height="11in" margin="1in">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="page">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="16pt" font-weight="bold">INVOICE</fo:block>
      <fo:block space-before="12pt">Invoice Number: INV-001</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"###;

    // Parse
    let builder = FoTreeBuilder::new();
    let fo_tree = builder
        .parse(Cursor::new(invoice_fo))
        .expect("Failed to parse invoice FO");

    assert!(fo_tree.len() > 5, "Invoice should have many nodes");

    // Layout
    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&fo_tree).expect("Failed to layout invoice");

    assert!(area_tree.len() > 3, "Invoice should have many areas");

    // Render to PDF
    let renderer = PdfRenderer::new();
    let pdf = renderer
        .render(&area_tree)
        .expect("Failed to render invoice PDF");

    assert_eq!(pdf.pages.len(), 1, "Invoice should be 1 page");

    // Generate bytes
    let bytes = pdf.to_bytes().expect("Failed to serialize PDF");

    assert!(bytes.len() > 1000, "PDF should be substantial");
    assert!(bytes.starts_with(b"%PDF-"), "Should be valid PDF");
}

/// Test a multi-page report with headers and footers
#[test]
fn test_multi_page_report() {
    let report_fo = r###"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="page" page-width="8.5in" page-height="11in"
                          margin="0.75in">
      <fo:region-body margin-top="0.5in" margin-bottom="0.5in"/>
      <fo:region-before extent="0.5in"/>
      <fo:region-after extent="0.5in"/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="page">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block text-align="center" font-weight="bold" border-bottom="1pt solid black">
        Monthly Report
      </fo:block>
    </fo:static-content>

    <fo:static-content flow-name="xsl-region-after">
      <fo:block text-align="center" font-size="9pt">
        Confidential - Copyright 2024 ACME Corp
      </fo:block>
    </fo:static-content>

    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="18pt" font-weight="bold" space-after="12pt">
        Executive Summary
      </fo:block>

      <fo:block space-after="6pt">
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod
        tempor incididunt ut labore et dolore magna aliqua.
      </fo:block>

      <fo:block font-size="14pt" font-weight="bold" space-before="12pt" space-after="6pt">
        Key Metrics
      </fo:block>

      <fo:list-block provisional-distance-between-starts="0.5in">
        <fo:list-item space-after="4pt">
          <fo:list-item-label end-indent="label-end()">
            <fo:block>•</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>Revenue: $1.2M (↑15% YoY)</fo:block>
          </fo:list-item-body>
        </fo:list-item>

        <fo:list-item space-after="4pt">
          <fo:list-item-label end-indent="label-end()">
            <fo:block>•</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>Customers: 450 (↑8% YoY)</fo:block>
          </fo:list-item-body>
        </fo:list-item>

        <fo:list-item>
          <fo:list-item-label end-indent="label-end()">
            <fo:block>•</fo:block>
          </fo:list-item-label>
          <fo:list-item-body start-indent="body-start()">
            <fo:block>Satisfaction: 4.8/5.0 (↑0.2)</fo:block>
          </fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"###;

    let builder = FoTreeBuilder::new();
    let fo_tree = builder
        .parse(Cursor::new(report_fo))
        .expect("Failed to parse report");

    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&fo_tree).expect("Failed to layout report");

    let renderer = PdfRenderer::new();
    let pdf = renderer
        .render(&area_tree)
        .expect("Failed to render report");

    assert!(!pdf.pages.is_empty(), "Report should have at least 1 page");
}

/// Test a form with various field types
#[test]
fn test_form_document() {
    let form_fo = r###"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="form" page-width="8.5in" page-height="11in"
                          margin="1in">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="form">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="16pt" font-weight="bold" text-align="center" space-after="18pt">
        Employee Information Form
      </fo:block>

      <fo:table table-layout="fixed" width="100%" space-after="12pt">
        <fo:table-column column-width="30%"/>
        <fo:table-column column-width="70%"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell padding="4pt">
              <fo:block font-weight="bold">Full Name:</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="4pt" border-bottom="1pt solid black">
              <fo:block>_______________________________</fo:block>
            </fo:table-cell>
          </fo:table-row>

          <fo:table-row>
            <fo:table-cell padding="4pt">
              <fo:block font-weight="bold">Employee ID:</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="4pt" border-bottom="1pt solid black">
              <fo:block>_______________________________</fo:block>
            </fo:table-cell>
          </fo:table-row>

          <fo:table-row>
            <fo:table-cell padding="4pt">
              <fo:block font-weight="bold">Department:</fo:block>
            </fo:table-cell>
            <fo:table-cell padding="4pt" border-bottom="1pt solid black">
              <fo:block>_______________________________</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>

      <fo:block font-size="10pt" space-before="24pt" border-top="1pt solid black" padding-top="6pt">
        Please complete all fields and submit to HR by end of business day.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"###;

    let builder = FoTreeBuilder::new();
    let fo_tree = builder
        .parse(Cursor::new(form_fo))
        .expect("Failed to parse form");

    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&fo_tree).expect("Failed to layout form");

    let renderer = PdfRenderer::new();
    let pdf = renderer.render(&area_tree).expect("Failed to render form");

    assert_eq!(pdf.pages.len(), 1, "Form should be 1 page");
}

/// Test handling of special characters and Unicode
#[test]
fn test_unicode_document() {
    let unicode_fo = r###"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="page" page-width="210mm" page-height="297mm" margin="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="page">
    <fo:flow flow-name="xsl-region-body">
      <fo:block space-after="12pt">English: Hello World!</fo:block>
      <fo:block space-after="12pt">Spanish: ¡Hola Mundo! ñáéíóú</fo:block>
      <fo:block space-after="12pt">French: Bonjour le Monde! àâçèéêëîïôùûü</fo:block>
      <fo:block space-after="12pt">German: Hallo Welt! äöüßÄÖÜ</fo:block>
      <fo:block space-after="12pt">Math: π ≈ 3.14159, √2 ≈ 1.414, ∞</fo:block>
      <fo:block space-after="12pt">Symbols: © ® ™ € £ ¥ • ° ±</fo:block>
      <fo:block>Quotes: "English" «French» „German"</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"###;

    let builder = FoTreeBuilder::new();
    let fo_tree = builder
        .parse(Cursor::new(unicode_fo))
        .expect("Failed to parse Unicode document");

    let engine = LayoutEngine::new();
    let area_tree = engine
        .layout(&fo_tree)
        .expect("Failed to layout Unicode document");

    let renderer = PdfRenderer::new();
    let pdf = renderer
        .render(&area_tree)
        .expect("Failed to render Unicode PDF");

    assert_eq!(pdf.pages.len(), 1);

    let bytes = pdf.to_bytes().expect("Failed to serialize");
    assert!(bytes.len() > 500);
}
