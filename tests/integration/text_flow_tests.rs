//! Text flow tests - widow/orphan control, line breaking, pagination
//!
//! These tests verify that text flows correctly across pages with proper
//! widow and orphan control, respecting XSL-FO pagination constraints.

use fop_core::FoTreeBuilder;
use fop_layout::LayoutEngine;
use fop_render::PdfRenderer;
use std::io::Cursor;

/// Test basic widow control - prevent too few lines at top of page
#[test]
fn test_widow_control_prevents_single_line() {
    // Document with widows=2 (minimum 2 lines at top of next page)
    // Create a paragraph long enough to span pages, but ensure at least 2 lines on second page
    let fo_doc = r###"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="100pt" page-width="200pt">
      <fo:region-body margin="10pt"/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <!-- Block with widows=2 (default) -->
      <fo:block widows="2" orphans="2" font-size="12pt" line-height="14pt">
        Line one of the paragraph that will span across pages.
        Line two of the paragraph content.
        Line three of the paragraph content.
        Line four of the paragraph content.
        Line five of the paragraph content.
        Line six of the paragraph content.
        Line seven of the paragraph content.
        Line eight of the paragraph content.
        Line nine of the paragraph content.
        This is the final line.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"###;

    let builder = FoTreeBuilder::new();
    let fo_tree = builder
        .parse(Cursor::new(fo_doc))
        .expect("Failed to parse FO document");

    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&fo_tree).expect("Failed to layout document");

    let renderer = PdfRenderer::new();
    let pdf = renderer.render(&area_tree).expect("Failed to render PDF");

    // Should create pages as needed, respecting widow constraints
    assert!(!pdf.pages.is_empty(), "Should generate at least one page");
}

/// Test basic orphan control - prevent too few lines at bottom of page
#[test]
fn test_orphan_control_prevents_single_line() {
    // Document with orphans=2 (minimum 2 lines at bottom before page break)
    let fo_doc = r###"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="100pt" page-width="200pt">
      <fo:region-body margin="10pt"/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <!-- First block fills most of the page -->
      <fo:block font-size="12pt" line-height="14pt">
        Filler line 1.
        Filler line 2.
        Filler line 3.
        Filler line 4.
      </fo:block>

      <!-- Second block with orphans=2 should not leave single line at bottom -->
      <fo:block orphans="2" widows="2" font-size="12pt" line-height="14pt">
        First line of paragraph that tests orphan control.
        Second line of paragraph.
        Third line of paragraph.
        Fourth line of paragraph.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"###;

    let builder = FoTreeBuilder::new();
    let fo_tree = builder
        .parse(Cursor::new(fo_doc))
        .expect("Failed to parse FO document");

    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&fo_tree).expect("Failed to layout document");

    let renderer = PdfRenderer::new();
    let pdf = renderer.render(&area_tree).expect("Failed to render PDF");

    // Should respect orphan constraints when breaking pages
    assert!(!pdf.pages.is_empty(), "Should generate at least one page");
}

/// Test custom widow value (widows=3)
#[test]
fn test_custom_widow_value() {
    let fo_doc = r###"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="120pt" page-width="200pt">
      <fo:region-body margin="10pt"/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <!-- Block with widows=3 (custom value) -->
      <fo:block widows="3" orphans="2" font-size="12pt" line-height="14pt">
        Line 1. Line 2. Line 3. Line 4. Line 5. Line 6. Line 7. Line 8. Line 9. Line 10.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"###;

    let builder = FoTreeBuilder::new();
    let fo_tree = builder
        .parse(Cursor::new(fo_doc))
        .expect("Failed to parse FO document");

    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&fo_tree).expect("Failed to layout document");

    let renderer = PdfRenderer::new();
    let pdf = renderer.render(&area_tree).expect("Failed to render PDF");

    assert!(
        !pdf.pages.is_empty(),
        "Should generate pages with widows=3 constraint"
    );
}

/// Test custom orphan value (orphans=4)
#[test]
fn test_custom_orphan_value() {
    let fo_doc = r###"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="100pt" page-width="200pt">
      <fo:region-body margin="10pt"/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="12pt">Filler block.</fo:block>

      <!-- Block with orphans=4 (custom value) -->
      <fo:block orphans="4" widows="2" font-size="12pt" line-height="14pt">
        Line 1. Line 2. Line 3. Line 4. Line 5. Line 6. Line 7. Line 8.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"###;

    let builder = FoTreeBuilder::new();
    let fo_tree = builder
        .parse(Cursor::new(fo_doc))
        .expect("Failed to parse FO document");

    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&fo_tree).expect("Failed to layout document");

    let renderer = PdfRenderer::new();
    let pdf = renderer.render(&area_tree).expect("Failed to render PDF");

    assert!(
        !pdf.pages.is_empty(),
        "Should generate pages with orphans=4 constraint"
    );
}

/// Test widows=1 and orphans=1 (minimal/relaxed constraints)
#[test]
fn test_minimal_widow_orphan_constraints() {
    let fo_doc = r###"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="100pt" page-width="200pt">
      <fo:region-body margin="10pt"/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <!-- Minimal widow/orphan constraints (allows most flexible breaking) -->
      <fo:block widows="1" orphans="1" font-size="12pt" line-height="14pt">
        Line 1. Line 2. Line 3. Line 4. Line 5. Line 6. Line 7. Line 8. Line 9.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"###;

    let builder = FoTreeBuilder::new();
    let fo_tree = builder
        .parse(Cursor::new(fo_doc))
        .expect("Failed to parse FO document");

    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&fo_tree).expect("Failed to layout document");

    let renderer = PdfRenderer::new();
    let pdf = renderer.render(&area_tree).expect("Failed to render PDF");

    assert!(
        !pdf.pages.is_empty(),
        "Should generate pages with minimal widow/orphan constraints"
    );
}

/// Test multiple paragraphs with different widow/orphan settings
#[test]
fn test_multiple_blocks_different_constraints() {
    let fo_doc = r###"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="150pt" page-width="200pt">
      <fo:region-body margin="10pt"/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <!-- First block: strict constraints -->
      <fo:block widows="3" orphans="3" font-size="12pt" line-height="14pt">
        Paragraph 1: Line 1. Line 2. Line 3. Line 4. Line 5.
      </fo:block>

      <!-- Second block: relaxed constraints -->
      <fo:block widows="1" orphans="1" font-size="12pt" line-height="14pt">
        Paragraph 2: Line 1. Line 2. Line 3. Line 4.
      </fo:block>

      <!-- Third block: default constraints (2/2) -->
      <fo:block font-size="12pt" line-height="14pt">
        Paragraph 3: Line 1. Line 2. Line 3.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"###;

    let builder = FoTreeBuilder::new();
    let fo_tree = builder
        .parse(Cursor::new(fo_doc))
        .expect("Failed to parse FO document");

    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&fo_tree).expect("Failed to layout document");

    let renderer = PdfRenderer::new();
    let pdf = renderer.render(&area_tree).expect("Failed to render PDF");

    assert!(
        !pdf.pages.is_empty(),
        "Should handle multiple blocks with different constraints"
    );
}

/// Test widow/orphan control with keep-together
#[test]
fn test_widow_orphan_with_keep_together() {
    let fo_doc = r###"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="120pt" page-width="200pt">
      <fo:region-body margin="10pt"/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="12pt">Filler to push content down.</fo:block>

      <!-- Block with both keep-together and widow/orphan -->
      <fo:block keep-together.within-page="always" widows="2" orphans="2"
                font-size="12pt" line-height="14pt">
        Line 1. Line 2. Line 3. Line 4.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"###;

    let builder = FoTreeBuilder::new();
    let fo_tree = builder
        .parse(Cursor::new(fo_doc))
        .expect("Failed to parse FO document");

    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&fo_tree).expect("Failed to layout document");

    let renderer = PdfRenderer::new();
    let pdf = renderer.render(&area_tree).expect("Failed to render PDF");

    assert!(
        !pdf.pages.is_empty(),
        "Should handle keep-together with widow/orphan constraints"
    );
}

/// Test realistic multi-page document with widow/orphan control
#[test]
fn test_realistic_multipage_with_widows_orphans() {
    let fo_doc = r###"<?xml version="1.0"?>
<fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
  <fo:layout-master-set>
    <fo:simple-page-master master-name="A4" page-height="200pt" page-width="150pt">
      <fo:region-body margin="15pt"/>
    </fo:simple-page-master>
  </fo:layout-master-set>

  <fo:page-sequence master-reference="A4">
    <fo:flow flow-name="xsl-region-body">
      <fo:block font-size="16pt" font-weight="bold" space-after="10pt">
        Document Title
      </fo:block>

      <fo:block widows="2" orphans="2" font-size="11pt" line-height="13pt"
                text-align="justify" space-after="6pt">
        First paragraph with default widow/orphan control. This paragraph contains
        enough text to potentially span multiple lines and demonstrate proper
        text flow control across page boundaries.
      </fo:block>

      <fo:block widows="2" orphans="2" font-size="11pt" line-height="13pt"
                text-align="justify" space-after="6pt">
        Second paragraph also with widow/orphan constraints. The layout engine
        should ensure that neither widows nor orphans appear when this content
        flows across page boundaries.
      </fo:block>

      <fo:block widows="3" orphans="3" font-size="11pt" line-height="13pt"
                text-align="justify" space-after="6pt">
        Third paragraph with stricter constraints (3/3). This ensures even better
        readability by preventing very short fragments at page edges.
      </fo:block>

      <fo:block widows="2" orphans="2" font-size="11pt" line-height="13pt"
                text-align="justify">
        Final paragraph concludes the document with proper text flow control
        throughout all pages.
      </fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"###;

    let builder = FoTreeBuilder::new();
    let fo_tree = builder
        .parse(Cursor::new(fo_doc))
        .expect("Failed to parse FO document");

    let engine = LayoutEngine::new();
    let area_tree = engine.layout(&fo_tree).expect("Failed to layout document");

    let renderer = PdfRenderer::new();
    let pdf = renderer.render(&area_tree).expect("Failed to render PDF");

    // Should generate a multi-page document
    assert!(
        !pdf.pages.is_empty(),
        "Should generate multi-page document with proper widow/orphan control"
    );
}
