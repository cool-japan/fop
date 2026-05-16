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
    assert!(
        serialized.contains("Page"),
        "Area tree must contain a Page area: {}",
        serialized
    );

    // Should have at least one Block area
    assert!(
        serialized.contains("Block"),
        "Area tree must contain Block areas: {}",
        serialized
    );

    // Should have text content
    assert!(
        serialized.contains("Visual regression"),
        "Area tree must contain the first text block"
    );

    // Page geometry: A4 = 595.28 x 841.89 pt (within 1pt tolerance)
    assert!(
        serialized.contains("595.") || serialized.contains("594."),
        "A4 page width should be ~595pt: {}",
        serialized
    );
}

#[test]
fn regression_pdf_output_size() {
    let pdf_bytes = process_fo_document(BASELINE_FO).expect("PDF generation should succeed");

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
    assert_eq!(
        page_count, 2,
        "Document with two page-sequences should have exactly 2 pages, got: {}",
        page_count
    );
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

    let pdf_bytes = process_fo_document(&fo_input).expect("PDF generation should succeed");

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
fn regression_single_page_count() {
    let fo = super::load_fixture("simple_single_page.fo");
    let pdf =
        super::process_fo_document(&fo).expect("simple_single_page.fo should generate a valid PDF");

    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF should be parseable by fop-pdf-renderer");
    assert_eq!(
        renderer.page_count(),
        1,
        "simple_single_page.fo should produce exactly 1 page"
    );

    super::validate_pdf_bytes(&pdf);
}

#[test]
fn regression_two_page_count() {
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

    let pdf =
        process_fo_document(fo_input).expect("Two-page FO document should generate a valid PDF");

    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF should be parseable by fop-pdf-renderer");
    assert_eq!(
        renderer.page_count(),
        2,
        "Two-page FO should produce exactly 2 pages"
    );
}

#[test]
fn regression_all_pages_rasterize() {
    let fo = super::load_fixture("simple_single_page.fo");
    let pdf =
        super::process_fo_document(&fo).expect("simple_single_page.fo should generate a valid PDF");

    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF should be parseable by fop-pdf-renderer");

    let pages = renderer
        .render_all_pages(72.0)
        .expect("All pages should rasterize successfully");

    assert!(
        !pages.is_empty(),
        "PDF should have at least one rasterized page"
    );
    for (i, page_bytes) in pages.iter().enumerate() {
        // Verify each page is a valid PNG (starts with PNG magic bytes)
        assert!(
            page_bytes.len() >= 8,
            "Page {} PNG should have at least 8 bytes",
            i
        );
        assert_eq!(
            &page_bytes[0..8],
            b"\x89PNG\r\n\x1a\n",
            "Page {} should start with PNG magic bytes",
            i
        );
    }

    // Additional assertion the old external tool didn't check: render dimensions
    let page = renderer
        .render_page(0, 72.0)
        .expect("First page should rasterize");
    assert!(
        page.width > 0 && page.height > 0,
        "Rasterized page should have non-zero dimensions: {}x{}",
        page.width,
        page.height
    );
}

#[test]
fn regression_extracted_text_roundtrips() {
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

    let pdf = super::process_fo_document(fo_input)
        .expect("FO with Helvetica text should generate a valid PDF");

    let renderer =
        fop_pdf_renderer::PdfRenderer::from_bytes(&pdf).expect("PDF should be parseable");

    let text = renderer
        .extract_text(0)
        .expect("Text extraction should succeed");

    assert!(
        !text.is_empty(),
        "Extracted text should not be empty — got empty string. \
         Check if Helvetica text is being emitted with ToUnicode CMap."
    );
    assert!(
        text.contains("EXTRACTABLE TEXT CONTENT"),
        "Extracted text should contain 'EXTRACTABLE TEXT CONTENT' but got: {:?}",
        text
    );
}

/// Regression test: XMP metadata from `<fo:declarations>` round-trips through
/// the PDF pipeline.  The XMP packet captured in `fop-core` must appear in the
/// PDF's `/Metadata` stream, and Dublin Core fields must sync to the /Info dict.
#[test]
fn test_issue_1_xmp_metadata_roundtrip() {
    let fo_input = r##"<?xml version="1.0" encoding="utf-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
      <fo:region-body margin="2cm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:declarations>
    <x:xmpmeta xmlns:x="adobe:ns:meta/">
      <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description xmlns:dc="http://purl.org/dc/elements/1.1/" rdf:about="">
          <dc:title>
            <rdf:Alt><rdf:li xml:lang="x-default">Test Invoice</rdf:li></rdf:Alt>
          </dc:title>
        </rdf:Description>
      </rdf:RDF>
    </x:xmpmeta>
  </fo:declarations>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-family="Helvetica" font-size="12pt">Hello.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    let pdf = super::process_fo_document(fo_input)
        .expect("FO with fo:declarations + XMP metadata should generate a valid PDF");

    // Verify the PDF is well-formed
    super::validate_pdf_bytes(&pdf);

    // Verify page count via the PDF renderer
    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF should be parseable by fop-pdf-renderer");
    assert_eq!(
        renderer.page_count(),
        1,
        "fo:declarations with XMP metadata must produce exactly 1 page, got {}",
        renderer.page_count()
    );

    // Verify page has renderable content
    let page = renderer
        .render_page(0, 72.0)
        .expect("First page should rasterize successfully");
    assert!(
        page.width > 0 && page.height > 0,
        "Rasterized page should have non-zero dimensions"
    );

    // Verify that text extraction still works (XMP capture must not silently break content)
    let text = renderer
        .extract_text(0)
        .expect("Text extraction should succeed on page 0");
    assert!(
        text.contains("Hello."),
        "Page text should contain 'Hello.' but got: {:?}",
        text
    );

    // Verify the XMP metadata packet was embedded in the PDF
    let xmp = renderer
        .extract_xmp_metadata()
        .expect("PDF should have an embedded XMP metadata stream");

    assert!(
        xmp.contains("xmpmeta"),
        "XMP stream should contain the xmpmeta element: {}",
        xmp
    );
    assert!(
        xmp.contains("Test Invoice"),
        "XMP stream should contain the dc:title value 'Test Invoice' but got: {}",
        xmp
    );
}

/// Regression test for GitHub issue #1:
/// `<fo:declarations>` with an `<x:xmpmeta>` XMP block silently produced a
/// zero-page PDF because the non-FO close tags were incorrectly calling
/// `end_element()`, walking `current_node` up the parent chain before
/// `<fo:page-sequence>` was processed, leaving it orphaned in the arena.
#[test]
fn test_issue_1_declarations_xmpmeta_zero_pages() {
    let fo_input = r##"<?xml version="1.0" encoding="utf-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
      <fo:region-body margin="2cm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:declarations>
    <x:xmpmeta xmlns:x="adobe:ns:meta/">
      <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description xmlns:dc="http://purl.org/dc/elements/1.1/" rdf:about="">
          <dc:title>
            <rdf:Alt><rdf:li xml:lang="x-default">Test Invoice</rdf:li></rdf:Alt>
          </dc:title>
        </rdf:Description>
      </rdf:RDF>
    </x:xmpmeta>
  </fo:declarations>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Hello.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    let pdf = super::process_fo_document(fo_input)
        .expect("FO with fo:declarations + XMP metadata should generate a valid PDF");

    // Verify the PDF is a real document, not an empty placeholder
    super::validate_pdf_bytes(&pdf);

    // Verify the page count via the PDF renderer
    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("PDF should be parseable by fop-pdf-renderer");
    assert_eq!(
        renderer.page_count(),
        1,
        "fo:declarations with XMP metadata must produce exactly 1 page, got {}",
        renderer.page_count()
    );
}

/// Regression test for GitHub issue #1 (cool-japan/fop):
/// `<fo:declarations>` containing an `<x:xmpmeta>` XMP block must still
/// produce a PDF with **at least one page**.
///
/// Before the fix the non-FO end-tags inside `<fo:declarations>` incorrectly
/// called `end_element()`, which walked `current_node` up the arena parent
/// chain before `<fo:page-sequence>` was encountered.  The page-sequence was
/// then orphaned in the arena and never reached the layout engine, yielding a
/// zero-page PDF.
///
/// The fix introduces `non_fo_depth` to consume non-FO close-tags without
/// touching `current_node`, so the page-sequence is correctly attached.
#[test]
fn test_issue_1_declarations_xmp_produces_pages() {
    let fo_input = r##"<?xml version="1.0" encoding="utf-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
      <fo:region-body margin="2cm"/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:declarations>
    <x:xmpmeta xmlns:x="adobe:ns:meta/">
      <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
        <rdf:Description xmlns:dc="http://purl.org/dc/elements/1.1/" rdf:about="">
          <dc:title>
            <rdf:Alt><rdf:li xml:lang="x-default">Issue 1 Regression</rdf:li></rdf:Alt>
          </dc:title>
        </rdf:Description>
      </rdf:RDF>
    </x:xmpmeta>
  </fo:declarations>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Issue 1 regression: declarations + XMP must not suppress pages.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;

    let pdf = super::process_fo_document(fo_input).expect(
        "FO document with fo:declarations + x:xmpmeta must compile to a valid PDF",
    );

    super::validate_pdf_bytes(&pdf);

    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf)
        .expect("generated PDF must be parseable");
    assert!(
        renderer.page_count() >= 1,
        "fo:declarations with XMP metadata must produce at least 1 page, got {}",
        renderer.page_count()
    );
}

/// Regression test: xmlns:x / xmlns:rdf / xmlns:dc declared on `<fo:root>` (not on
/// `<x:xmpmeta>`) must produce a standalone-well-formed XMP packet in the PDF
/// `/Metadata` stream — no undefined namespace prefixes.
#[test]
fn test_issue_1_namespace_inheritance_pdf_roundtrip() {
    let fo_bytes = br#"<?xml version="1.0" encoding="UTF-8"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format"
         xmlns:x="adobe:ns:meta/"
         xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"
         xmlns:dc="http://purl.org/dc/elements/1.1/">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="297mm" page-width="210mm"
                           margin-top="20mm" margin-bottom="20mm"
                           margin-left="20mm" margin-right="20mm">
      <fo:region-body/>
    </fo:simple-page-master>
  </fo:layout-master-set>
  <fo:declarations>
    <x:xmpmeta>
      <rdf:RDF>
        <rdf:Description rdf:about="">
          <dc:title>
            <rdf:Alt><rdf:li xml:lang="x-default">Test Invoice</rdf:li></rdf:Alt>
          </dc:title>
          <dc:creator><rdf:Bag><rdf:li>Test Author</rdf:li></rdf:Bag></dc:creator>
        </rdf:Description>
      </rdf:RDF>
    </x:xmpmeta>
  </fo:declarations>
  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block>Hello.</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"#;

    let pdf_bytes = super::process_fo_document(
        std::str::from_utf8(fo_bytes)
            .expect("test FO is valid UTF-8"),
    )
    .expect("FO with inherited xmlns on fo:root should generate a valid PDF");

    super::validate_pdf_bytes(&pdf_bytes);

    // Extract XMP metadata via PdfRenderer
    let renderer = fop_pdf_renderer::PdfRenderer::from_bytes(&pdf_bytes)
        .expect("PDF should be parseable by fop-pdf-renderer");

    let xmp = renderer
        .extract_xmp_metadata()
        .expect("PDF should have /Metadata stream");

    // Verify the packet is standalone-parseable (no undefined prefixes)
    use quick_xml::name::ResolveResult;
    use quick_xml::NsReader;
    let mut ns_reader = NsReader::from_str(&xmp);
    ns_reader.config_mut().trim_text(true);
    let mut buf = Vec::new();
    loop {
        match ns_reader.read_resolved_event_into(&mut buf) {
            Ok((ResolveResult::Unknown(prefix), _)) => {
                panic!(
                    "undefined prefix in /Metadata XMP: {:?}",
                    std::str::from_utf8(&prefix)
                );
            }
            Ok((_, quick_xml::events::Event::Eof)) => break,
            Ok(_) => {}
            Err(e) => panic!("parse error in /Metadata XMP: {e}"),
        }
        buf.clear();
    }

    // Verify dc:title value is preserved
    assert!(
        xmp.contains("Test Invoice"),
        "XMP should contain dc:title value; got: {xmp}"
    );

    // Verify the page rendered correctly
    assert_eq!(renderer.page_count(), 1, "should be exactly 1 page");
    let text = renderer
        .extract_text(0)
        .expect("should extract text from page 0");
    assert!(
        text.contains("Hello"),
        "page text should still render; got: {text}"
    );
}

#[test]
fn test_simple_builder_xmp_roundtrip_via_pdf_renderer_fast_path() {
    use fop_render::pdf::simple::{BuiltinFont, SimpleDocumentBuilder};
    use fop_pdf_renderer::PdfRenderer;

    let xmp_payload = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:dc="http://purl.org/dc/elements/1.1/"><rdf:RDF><rdf:Description rdf:about=""><dc:title><rdf:Alt><rdf:li xml:lang="x-default">Audit Log Q2</rdf:li></rdf:Alt></dc:title><dc:creator><rdf:Bag><rdf:li>Compliance</rdf:li></rdf:Bag></dc:creator></rdf:Description></rdf:RDF></x:xmpmeta>"#;

    let mut b = SimpleDocumentBuilder::new("Audit Log Q2");
    b.set_xmp_metadata(xmp_payload);
    // Helvetica only → fast path
    b.text("Report data", 12.0, 72.0, 720.0, BuiltinFont::Helvetica);
    let bytes = b.save();

    let renderer = PdfRenderer::from_bytes(&bytes)
        .expect("fast-path PDF should parse");
    assert!(renderer.page_count() > 0, "should have at least one page");

    let xmp = renderer
        .extract_xmp_metadata()
        .expect("fast-path PDF should have XMP metadata");
    assert!(
        xmp.contains("<x:xmpmeta"),
        "XMP should contain <x:xmpmeta; got: {xmp:?}"
    );
    assert!(
        xmp.contains("Audit Log Q2") || xmp.contains("Compliance"),
        "XMP should contain payload content; got: {xmp:?}"
    );
}

#[test]
fn test_simple_builder_xmp_roundtrip_via_pdf_renderer_slow_path() {
    use fop_render::pdf::simple::{BuiltinFont, SimpleDocumentBuilder};
    use fop_pdf_renderer::PdfRenderer;

    let xmp_payload = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:dc="http://purl.org/dc/elements/1.1/"><rdf:RDF><rdf:Description rdf:about=""><dc:title><rdf:Alt><rdf:li xml:lang="x-default">Audit Log Q2</rdf:li></rdf:Alt></dc:title><dc:creator><rdf:Bag><rdf:li>Compliance</rdf:li></rdf:Bag></dc:creator></rdf:Description></rdf:RDF></x:xmpmeta>"#;

    let mut b = SimpleDocumentBuilder::new("Audit Log Q2");
    b.set_xmp_metadata(xmp_payload);
    // HelveticaBold → slow path
    b.text("Report data", 12.0, 72.0, 720.0, BuiltinFont::HelveticaBold);
    let bytes = b.save();

    let renderer = PdfRenderer::from_bytes(&bytes)
        .expect("slow-path PDF should parse");
    assert!(renderer.page_count() > 0, "should have at least one page");

    let xmp = renderer
        .extract_xmp_metadata()
        .expect("slow-path PDF should have XMP metadata");
    assert!(
        xmp.contains("<x:xmpmeta"),
        "XMP should contain <x:xmpmeta; got: {xmp:?}"
    );
    assert!(
        xmp.contains("Audit Log Q2") || xmp.contains("Compliance"),
        "XMP should contain payload content; got: {xmp:?}"
    );
}
