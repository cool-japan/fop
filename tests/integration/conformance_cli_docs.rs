//! XSL-FO 1.1 Conformance: CLI-specific tests and real-world document format tests
//!
//! Part of the XSL-FO 1.1 conformance test suite.
//! Reference: https://www.w3.org/TR/xsl11/

use super::{process_fo_document, validate_pdf_bytes};

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

fn check_fo(fo: &str) -> Vec<u8> {
    process_fo_document(fo).unwrap_or_else(|e| panic!("XSL-FO processing failed: {}", e))
}

fn wrap_fo(body: &str) -> String {
    format!(
        r##"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4"
        page-width="210mm" page-height="297mm"
        margin-top="20mm" margin-bottom="20mm"
        margin-left="25mm" margin-right="25mm">
      <fo:region-body region-name="xsl-region-body"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      {}
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        body
    )
}

// ---------------------------------------------------------------------------
// CLI-specific conformance tests
// Tests that verify the pipeline produces correct output for common CLI use
// cases: multi-format rendering, complex documents, and format fidelity.
// ---------------------------------------------------------------------------

#[test]
fn conformance_cli_multiformat_same_document() {
    // The same FO document should render successfully in all major output formats.
    // This mirrors the CLI's --svg, --ps, --txt, --pdf flags all processing
    // the same input document.
    let fo = wrap_fo(
        r#"<fo:block font-size="14pt" font-weight="bold">CLI Multi-Format Test</fo:block>
        <fo:block space-before="6pt">
          This document is rendered in PDF, SVG, PostScript and plain-text formats.
          All formats must succeed and produce non-empty output.
        </fo:block>"#,
    );

    for format in &["pdf", "svg", "ps", "text"] {
        let result = super::process_fo_document_format(&fo, format);
        assert!(
            result.is_ok(),
            "Format '{}' should succeed: {:?}",
            format,
            result.err()
        );
        let bytes = result.expect("test: should succeed");
        assert!(
            !bytes.is_empty(),
            "Format '{}' output must not be empty",
            format
        );
    }
}

#[test]
fn conformance_cli_pdf_header_integrity() {
    // PDF output produced via the normal pipeline must have a valid PDF header
    // and xref trailer — verifying that the full FOP pipeline produces a PDF
    // that a PDF reader would accept, not just an empty or truncated file.
    let fo = wrap_fo(
        r#"<fo:block font-size="12pt">PDF Integrity Test</fo:block>
        <fo:block>A document with standard body text to verify PDF structure.</fo:block>"#,
    );

    let bytes = check_fo(&fo);
    validate_pdf_bytes(&bytes);

    // Verify internal PDF structure markers
    let content = String::from_utf8_lossy(&bytes);
    assert!(content.contains("%%EOF"), "PDF must end with %%EOF marker");
    assert!(
        content.contains("/Pages"),
        "PDF must contain /Pages dictionary"
    );
    assert!(
        content.contains("/Type /Catalog"),
        "PDF must contain a catalog"
    );
}

#[test]
fn conformance_cli_text_extraction_content() {
    // Text-format output must contain the actual text from the FO document.
    // This is the primary conformance requirement for the CLI's --txt output
    // mode: the rendered text must be recoverable from the output.
    let marker = "UNIQUE_CLI_MARKER_12345";
    let fo = wrap_fo(&format!(
        r#"<fo:block font-size="12pt">{}</fo:block>
        <fo:block space-before="4pt">Supporting paragraph text.</fo:block>"#,
        marker
    ));

    let result = super::process_fo_document_format(&fo, "text");
    assert!(result.is_ok(), "Text render failed: {:?}", result.err());
    let bytes = result.expect("test: should succeed");
    let text = String::from_utf8_lossy(&bytes);
    assert!(
        text.contains(marker),
        "Text output must contain the original text content. Got: {}",
        &text[..text.len().min(200)]
    );
}

#[test]
fn conformance_resume_cv_format() {
    // CV/Resume format with sections, lists, and formatting
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="cv"
      page-width="210mm" page-height="297mm"
      margin-top="15mm" margin-bottom="15mm"
      margin-left="15mm" margin-right="15mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="cv">
    <fo:flow flow-name="xsl-region-body">
      <fo:block-container>
        <fo:block font-size="22pt" font-weight="bold" color="#2c3e50">Jane Smith</fo:block>
        <fo:block font-size="11pt" color="#7f8c8d">Software Engineer | jane.smith@example.com | +1 (555) 123-4567</fo:block>
        <fo:block font-size="11pt" color="#7f8c8d" space-after="8pt">github.com/janesmith | linkedin.com/in/janesmith</fo:block>
      </fo:block-container>
      <fo:block border-bottom="2pt solid #2c3e50" space-after="2pt"/>
      <fo:block font-size="13pt" font-weight="bold" color="#2c3e50" space-before="8pt" space-after="4pt">
        EXPERIENCE
      </fo:block>
      <fo:block font-weight="bold" space-after="1pt">Senior Software Engineer — TechCorp Inc.</fo:block>
      <fo:block font-style="italic" color="#7f8c8d" space-after="3pt" font-size="9pt">January 2020 – Present</fo:block>
      <fo:list-block provisional-distance-between-starts="6mm" space-after="6pt">
        <fo:list-item space-after="1pt">
          <fo:list-item-label end-indent="label-end()"><fo:block>•</fo:block></fo:list-item-label>
          <fo:list-item-body start-indent="body-start()"><fo:block font-size="9pt">Led development of microservices architecture serving 10M+ users</fo:block></fo:list-item-body>
        </fo:list-item>
        <fo:list-item space-after="1pt">
          <fo:list-item-label end-indent="label-end()"><fo:block>•</fo:block></fo:list-item-label>
          <fo:list-item-body start-indent="body-start()"><fo:block font-size="9pt">Reduced API latency by 40% through caching and optimization</fo:block></fo:list-item-body>
        </fo:list-item>
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()"><fo:block>•</fo:block></fo:list-item-label>
          <fo:list-item-body start-indent="body-start()"><fo:block font-size="9pt">Mentored team of 5 junior engineers</fo:block></fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
      <fo:block font-weight="bold" space-after="1pt">Software Engineer — StartupXYZ</fo:block>
      <fo:block font-style="italic" color="#7f8c8d" space-after="3pt" font-size="9pt">June 2017 – December 2019</fo:block>
      <fo:list-block provisional-distance-between-starts="6mm" space-after="8pt">
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()"><fo:block>•</fo:block></fo:list-item-label>
          <fo:list-item-body start-indent="body-start()"><fo:block font-size="9pt">Built REST APIs using Python/Django and PostgreSQL</fo:block></fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
      <fo:block border-bottom="1pt solid #bdc3c7" space-after="2pt"/>
      <fo:block font-size="13pt" font-weight="bold" color="#2c3e50" space-before="6pt" space-after="4pt">SKILLS</fo:block>
      <fo:table table-layout="fixed" width="170mm">
        <fo:table-column column-width="35mm"/>
        <fo:table-column column-width="135mm"/>
        <fo:table-body>
          <fo:table-row space-after="2pt">
            <fo:table-cell padding="1mm"><fo:block font-weight="bold" font-size="9pt">Languages:</fo:block></fo:table-cell>
            <fo:table-cell padding="1mm"><fo:block font-size="9pt">Rust, Python, Go, TypeScript, SQL</fo:block></fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell padding="1mm"><fo:block font-weight="bold" font-size="9pt">Tools:</fo:block></fo:table-cell>
            <fo:table-cell padding="1mm"><fo:block font-size="9pt">Docker, Kubernetes, PostgreSQL, Redis, AWS, GCP</fo:block></fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Resume/CV format should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_product_catalog() {
    // Product catalog with images, prices, descriptions
    let mut products = String::new();
    for i in 1..=4 {
        products.push_str(&format!(r##"
          <fo:table-row>
            <fo:table-cell border="0.5pt solid #cccccc" padding="3mm" background-color="#fafafa">
              <fo:block-container height="20mm" background-color="#e8e8e8" border="0.5pt solid #cccccc">
                <fo:block font-size="8pt" text-align="center" padding-top="7mm" color="#999999">
                  [Product {} Image]
                </fo:block>
              </fo:block-container>
            </fo:table-cell>
            <fo:table-cell border="0.5pt solid #cccccc" padding="3mm">
              <fo:block font-weight="bold" font-size="10pt" space-after="2pt">Product Name {}</fo:block>
              <fo:block font-size="8pt" color="#666666" space-after="4pt">
                SKU: PROD-{:04} | Category: Electronics
              </fo:block>
              <fo:block font-size="9pt" space-after="4pt">
                High-quality product with excellent performance and durability. Ideal for professional use.
              </fo:block>
              <fo:block font-size="11pt" font-weight="bold" color="#e74c3c">
                ${}9.99
              </fo:block>
            </fo:table-cell>
          </fo:table-row>"##, i, i, i * 100, i * 10 + i));
    }
    let fo_xml = format!(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="catalog"
      page-width="210mm" page-height="297mm"
      margin-top="15mm" margin-bottom="15mm"
      margin-left="15mm" margin-right="15mm">
      <fo:region-body margin-top="20mm" margin-bottom="15mm"/>
      <fo:region-before extent="20mm"/>
      <fo:region-after extent="15mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="catalog">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block font-size="16pt" font-weight="bold" text-align="center" 
        background-color="#2c3e50" color="white" padding="4mm">
        PRODUCT CATALOG 2024
      </fo:block>
    </fo:static-content>
    <fo:static-content flow-name="xsl-region-after">
      <fo:block font-size="8pt" text-align="center" color="#999999">
        Page <fo:page-number/> | catalog@example.com | 1-800-PRODUCTS
      </fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="13pt" font-weight="bold" space-after="4pt">Electronics</fo:block>
      <fo:table table-layout="fixed" width="180mm">
        <fo:table-column column-width="40mm"/>
        <fo:table-column column-width="140mm"/>
        <fo:table-body>{}</fo:table-body>
      </fo:table>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
        products
    );
    let result = process_fo_document(&fo_xml);
    assert!(
        result.is_ok(),
        "Product catalog should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_meeting_minutes() {
    // Meeting minutes format
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="minutes"
      page-width="210mm" page-height="297mm"
      margin-top="20mm" margin-bottom="20mm"
      margin-left="25mm" margin-right="25mm">
      <fo:region-body margin-top="8mm"/>
      <fo:region-before extent="8mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="minutes">
    <fo:static-content flow-name="xsl-region-before">
      <fo:block font-size="9pt" text-align="right" color="#999999">CONFIDENTIAL</fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="16pt" font-weight="bold" text-align="center" space-after="2pt">MEETING MINUTES</fo:block>
      <fo:block font-size="11pt" text-align="center" space-after="8pt" color="#666666">Project Alpha Status Review</fo:block>
      <fo:table table-layout="fixed" width="160mm" space-after="8pt">
        <fo:table-column column-width="40mm"/>
        <fo:table-column column-width="120mm"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell padding="1mm"><fo:block font-weight="bold" font-size="9pt">Date:</fo:block></fo:table-cell>
            <fo:table-cell padding="1mm"><fo:block font-size="9pt">January 15, 2024</fo:block></fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell padding="1mm"><fo:block font-weight="bold" font-size="9pt">Time:</fo:block></fo:table-cell>
            <fo:table-cell padding="1mm"><fo:block font-size="9pt">10:00 AM – 11:30 AM</fo:block></fo:table-cell>
          </fo:table-row>
          <fo:table-row>
            <fo:table-cell padding="1mm"><fo:block font-weight="bold" font-size="9pt">Attendees:</fo:block></fo:table-cell>
            <fo:table-cell padding="1mm"><fo:block font-size="9pt">Alice (PM), Bob (Dev), Carol (QA), Dave (Design)</fo:block></fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
      <fo:block font-size="12pt" font-weight="bold" space-after="4pt" keep-with-next="always">1. Status Update</fo:block>
      <fo:block font-size="9pt" space-after="4pt">Bob reported that the backend API is 90% complete. Remaining work is focused on authentication module.</fo:block>
      <fo:block font-size="12pt" font-weight="bold" space-after="4pt" space-before="6pt" keep-with-next="always">2. Action Items</fo:block>
      <fo:list-block provisional-distance-between-starts="8mm">
        <fo:list-item space-after="2pt">
          <fo:list-item-label end-indent="label-end()"><fo:block font-size="9pt">1.</fo:block></fo:list-item-label>
          <fo:list-item-body start-indent="body-start()"><fo:block font-size="9pt"><fo:inline font-weight="bold">Bob</fo:inline>: Complete auth module by Jan 20</fo:block></fo:list-item-body>
        </fo:list-item>
        <fo:list-item space-after="2pt">
          <fo:list-item-label end-indent="label-end()"><fo:block font-size="9pt">2.</fo:block></fo:list-item-label>
          <fo:list-item-body start-indent="body-start()"><fo:block font-size="9pt"><fo:inline font-weight="bold">Carol</fo:inline>: Create test plan for auth feature</fo:block></fo:list-item-body>
        </fo:list-item>
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()"><fo:block font-size="9pt">3.</fo:block></fo:list-item-label>
          <fo:list-item-body start-indent="body-start()"><fo:block font-size="9pt"><fo:inline font-weight="bold">Alice</fo:inline>: Schedule next review for Jan 22</fo:block></fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
      <fo:block font-size="12pt" font-weight="bold" space-before="8pt" space-after="4pt">3. Next Meeting</fo:block>
      <fo:block font-size="9pt">January 22, 2024, 10:00 AM, Room 304</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Meeting minutes should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_certificate_document() {
    // Certificate/diploma format
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="cert"
      page-width="297mm" page-height="210mm"
      margin-top="15mm" margin-bottom="15mm"
      margin-left="20mm" margin-right="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="cert">
    <fo:flow flow-name="xsl-region-body">
      <fo:block-container border="3pt solid #c8a951" padding="10mm">
        <fo:block-container border="1pt solid #c8a951" padding="8mm">
          <fo:block font-size="10pt" text-align="center" letter-spacing="5pt" 
            color="#8b7355" space-after="4pt">
            ✦ CERTIFICATE OF ACHIEVEMENT ✦
          </fo:block>
          <fo:block font-size="9pt" text-align="center" color="#8b7355" space-after="12pt">
            This is to certify that
          </fo:block>
          <fo:block font-size="28pt" font-weight="bold" text-align="center" 
            color="#2c3e50" space-after="4pt">
            John Michael Doe
          </fo:block>
          <fo:block border-bottom="1pt solid #c8a951" width="120mm" space-after="12pt"/>
          <fo:block font-size="11pt" text-align="center" color="#555555" space-after="6pt">
            has successfully completed the course
          </fo:block>
          <fo:block font-size="18pt" font-weight="bold" text-align="center"
            color="#003366" space-after="4pt">
            Advanced Rust Programming
          </fo:block>
          <fo:block font-size="10pt" text-align="center" color="#555555" space-after="16pt">
            with distinction, having demonstrated exceptional proficiency in systems programming
          </fo:block>
          <fo:table table-layout="fixed" width="220mm">
            <fo:table-column column-width="110mm"/>
            <fo:table-column column-width="110mm"/>
            <fo:table-body>
              <fo:table-row>
                <fo:table-cell text-align="center" padding="4mm">
                  <fo:block border-top="1pt solid #2c3e50" padding-top="4pt" font-size="9pt" color="#555555">
                    Dr. Jane Smith, Course Director
                  </fo:block>
                </fo:table-cell>
                <fo:table-cell text-align="center" padding="4mm">
                  <fo:block border-top="1pt solid #2c3e50" padding-top="4pt" font-size="9pt" color="#555555">
                    Date: January 15, 2024
                  </fo:block>
                </fo:table-cell>
              </fo:table-row>
            </fo:table-body>
          </fo:table>
        </fo:block-container>
      </fo:block-container>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Certificate format should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}

#[test]
fn conformance_data_sheet_document() {
    // Technical data sheet format
    let result = process_fo_document(
        r##"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="datasheet"
      page-width="210mm" page-height="297mm"
      margin-top="15mm" margin-bottom="15mm"
      margin-left="12mm" margin-right="12mm">
      <fo:region-body column-count="2" column-gap="6mm" margin-top="22mm" margin-bottom="12mm"/>
      <fo:region-before extent="22mm"/>
      <fo:region-after extent="12mm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:page-sequence master-reference="datasheet">
    <fo:static-content flow-name="xsl-region-before">
      <fo:table table-layout="fixed" width="186mm">
        <fo:table-column column-width="100mm"/>
        <fo:table-column column-width="86mm"/>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell display-align="center">
              <fo:block font-size="18pt" font-weight="bold" color="#003366">MicroChip XR-2000</fo:block>
              <fo:block font-size="9pt" color="#666666">High-Performance Processing Unit</fo:block>
            </fo:table-cell>
            <fo:table-cell display-align="center">
              <fo:block font-size="9pt" text-align="right" color="#666666">Document: DS-XR2000-01</fo:block>
              <fo:block font-size="9pt" text-align="right" color="#666666">Rev: 1.2</fo:block>
            </fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    </fo:static-content>
    <fo:static-content flow-name="xsl-region-after">
      <fo:block font-size="8pt" color="#999999" border-top="0.5pt solid #cccccc" padding-top="2pt">
        <fo:inline font-weight="bold">© 2024 MicroChip Corp.</fo:inline>
        Page <fo:page-number/>
      </fo:block>
    </fo:static-content>
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="10pt" font-weight="bold" color="#003366" space-after="3pt"
        border-bottom="0.5pt solid #003366" keep-with-next="always">Features</fo:block>
      <fo:list-block provisional-distance-between-starts="5mm" space-after="6pt">
        <fo:list-item space-after="1pt">
          <fo:list-item-label end-indent="label-end()"><fo:block font-size="7pt">•</fo:block></fo:list-item-label>
          <fo:list-item-body start-indent="body-start()"><fo:block font-size="8pt">3.2 GHz quad-core processor</fo:block></fo:list-item-body>
        </fo:list-item>
        <fo:list-item space-after="1pt">
          <fo:list-item-label end-indent="label-end()"><fo:block font-size="7pt">•</fo:block></fo:list-item-label>
          <fo:list-item-body start-indent="body-start()"><fo:block font-size="8pt">16 KB L1 cache per core</fo:block></fo:list-item-body>
        </fo:list-item>
        <fo:list-item>
          <fo:list-item-label end-indent="label-end()"><fo:block font-size="7pt">•</fo:block></fo:list-item-label>
          <fo:list-item-body start-indent="body-start()"><fo:block font-size="8pt">3.3V CMOS process</fo:block></fo:list-item-body>
        </fo:list-item>
      </fo:list-block>
      <fo:block font-size="10pt" font-weight="bold" color="#003366" space-after="3pt"
        border-bottom="0.5pt solid #003366" keep-with-next="always">Electrical Characteristics</fo:block>
      <fo:table table-layout="fixed" width="84mm" space-after="6pt" font-size="7pt">
        <fo:table-column column-width="30mm"/>
        <fo:table-column column-width="18mm"/>
        <fo:table-column column-width="18mm"/>
        <fo:table-column column-width="18mm"/>
        <fo:table-header>
          <fo:table-row background-color="#e8edf2">
            <fo:table-cell padding="1mm" border="0.5pt solid #cccccc"><fo:block font-weight="bold">Parameter</fo:block></fo:table-cell>
            <fo:table-cell padding="1mm" border="0.5pt solid #cccccc"><fo:block font-weight="bold" text-align="center">Min</fo:block></fo:table-cell>
            <fo:table-cell padding="1mm" border="0.5pt solid #cccccc"><fo:block font-weight="bold" text-align="center">Typ</fo:block></fo:table-cell>
            <fo:table-cell padding="1mm" border="0.5pt solid #cccccc"><fo:block font-weight="bold" text-align="center">Max</fo:block></fo:table-cell>
          </fo:table-row>
        </fo:table-header>
        <fo:table-body>
          <fo:table-row>
            <fo:table-cell padding="1mm" border="0.5pt solid #cccccc"><fo:block>Vcc</fo:block></fo:table-cell>
            <fo:table-cell padding="1mm" border="0.5pt solid #cccccc"><fo:block text-align="center">3.0V</fo:block></fo:table-cell>
            <fo:table-cell padding="1mm" border="0.5pt solid #cccccc"><fo:block text-align="center">3.3V</fo:block></fo:table-cell>
            <fo:table-cell padding="1mm" border="0.5pt solid #cccccc"><fo:block text-align="center">3.6V</fo:block></fo:table-cell>
          </fo:table-row>
          <fo:table-row background-color="#f8f8f8">
            <fo:table-cell padding="1mm" border="0.5pt solid #cccccc"><fo:block>Icc (active)</fo:block></fo:table-cell>
            <fo:table-cell padding="1mm" border="0.5pt solid #cccccc"><fo:block text-align="center">—</fo:block></fo:table-cell>
            <fo:table-cell padding="1mm" border="0.5pt solid #cccccc"><fo:block text-align="center">125 mA</fo:block></fo:table-cell>
            <fo:table-cell padding="1mm" border="0.5pt solid #cccccc"><fo:block text-align="center">150 mA</fo:block></fo:table-cell>
          </fo:table-row>
        </fo:table-body>
      </fo:table>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##,
    );
    assert!(
        result.is_ok(),
        "Data sheet format should work: {:?}",
        result.err()
    );
    assert!(!result.expect("test: should succeed").is_empty());
}
