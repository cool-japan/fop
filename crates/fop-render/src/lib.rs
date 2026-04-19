//! Rendering backends for FOP
//!
//! Transforms area trees into output formats (PDF, SVG, PostScript, images, text, etc.)

pub mod image;
pub mod parallel;
pub mod pdf;
pub mod ps;
#[cfg(feature = "raster")]
pub mod raster;
pub mod svg;
pub mod text;

pub use fop_types::{FopError, Result};
pub use image::{ImageFormat, ImageInfo, ImagePlacement};
pub use parallel::ParallelRenderer;
pub use pdf::{
    EncryptionAlgorithm, EncryptionDict, FontConfig, PdfBuiltinFont, PdfCompliance, PdfDocument,
    PdfGraphics, PdfPermissions, PdfRenderer, PdfSecurity, PdfValidator, SimpleDocumentBuilder,
    ValidationResult,
};
pub use ps::{PsDocument, PsRenderer};
#[cfg(feature = "raster")]
pub use raster::{RasterFormat, RasterRenderer};
pub use svg::{SvgDocument, SvgGraphics, SvgRenderer};
pub use text::TextRenderer;

#[cfg(test)]
mod tests {
    use fop_core::FoTreeBuilder;
    use fop_layout::{AreaTree, LayoutEngine};
    use std::io::Cursor;

    /// Build a minimal area tree from a simple XSL-FO document.
    fn minimal_area_tree() -> AreaTree {
        let fo_xml = r##"<?xml version="1.0"?>
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
      <fo:block>Hello, world!</fo:block>
    </fo:flow>
  </fo:page-sequence>
</fo:root>"##;
        let builder = FoTreeBuilder::new();
        let fo_tree = builder
            .parse(Cursor::new(fo_xml))
            .expect("test: should succeed");
        let engine = LayoutEngine::new();
        engine.layout(&fo_tree).expect("test: should succeed")
    }

    // ── PDF rendering ─────────────────────────────────────────────────────────

    #[test]
    fn test_pdf_render_produces_bytes() {
        let area_tree = minimal_area_tree();
        let renderer = crate::PdfRenderer::new();
        let doc = renderer.render(&area_tree).expect("test: should succeed");
        let bytes = doc.to_bytes().expect("test: should succeed");
        assert!(!bytes.is_empty(), "PDF output must not be empty");
        assert!(
            bytes.starts_with(b"%PDF-"),
            "PDF must start with %PDF- header"
        );
    }

    #[test]
    fn test_pdf_render_has_one_page() {
        let area_tree = minimal_area_tree();
        let renderer = crate::PdfRenderer::new();
        let doc = renderer.render(&area_tree).expect("test: should succeed");
        assert_eq!(
            doc.pages.len(),
            1,
            "single page-sequence must produce 1 PDF page"
        );
    }

    #[test]
    fn test_pdf_render_has_eof() {
        let area_tree = minimal_area_tree();
        let renderer = crate::PdfRenderer::new();
        let bytes = renderer
            .render(&area_tree)
            .expect("test: should succeed")
            .to_bytes()
            .expect("test: should succeed");
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("%%EOF"), "PDF must end with %%EOF");
    }

    #[test]
    fn test_pdf_render_contains_text() {
        let area_tree = minimal_area_tree();
        let renderer = crate::PdfRenderer::new();
        let bytes = renderer
            .render(&area_tree)
            .expect("test: should succeed")
            .to_bytes()
            .expect("test: should succeed");
        let content = String::from_utf8_lossy(&bytes);
        assert!(
            content.contains("Hello, world!"),
            "PDF must contain the rendered text"
        );
    }

    // ── SVG rendering ─────────────────────────────────────────────────────────

    #[test]
    fn test_svg_render_produces_string() {
        let area_tree = minimal_area_tree();
        let renderer = crate::SvgRenderer::new();
        let result = renderer
            .render_to_svg(&area_tree)
            .expect("test: should succeed");
        assert!(!result.is_empty(), "SVG output must not be empty");
    }

    #[test]
    fn test_svg_render_has_xml_declaration() {
        let area_tree = minimal_area_tree();
        let renderer = crate::SvgRenderer::new();
        let result = renderer
            .render_to_svg(&area_tree)
            .expect("test: should succeed");
        assert!(
            result.starts_with("<?xml"),
            "SVG output must start with XML declaration"
        );
    }

    #[test]
    fn test_svg_render_has_svg_element() {
        let area_tree = minimal_area_tree();
        let renderer = crate::SvgRenderer::new();
        let result = renderer
            .render_to_svg(&area_tree)
            .expect("test: should succeed");
        assert!(
            result.contains("<svg"),
            "SVG output must contain <svg element"
        );
    }

    #[test]
    fn test_svg_render_contains_text() {
        let area_tree = minimal_area_tree();
        let renderer = crate::SvgRenderer::new();
        let result = renderer
            .render_to_svg(&area_tree)
            .expect("test: should succeed");
        assert!(
            result.contains("Hello, world!"),
            "SVG output must contain the rendered text"
        );
    }

    #[test]
    fn test_svg_render_pages_produces_one_page() {
        let area_tree = minimal_area_tree();
        let renderer = crate::SvgRenderer::new();
        let pages = renderer
            .render_to_svg_pages(&area_tree)
            .expect("test: should succeed");
        assert_eq!(
            pages.len(),
            1,
            "single page-sequence must produce 1 SVG page"
        );
    }

    #[test]
    fn test_svg_render_each_page_is_valid_svg() {
        let area_tree = minimal_area_tree();
        let renderer = crate::SvgRenderer::new();
        let pages = renderer
            .render_to_svg_pages(&area_tree)
            .expect("test: should succeed");
        for (i, page) in pages.iter().enumerate() {
            assert!(
                page.contains("<svg"),
                "SVG page {i} must contain <svg element"
            );
            assert!(page.contains("</svg>"), "SVG page {i} must close </svg>");
        }
    }

    // ── PostScript rendering ──────────────────────────────────────────────────

    #[test]
    fn test_ps_render_produces_string() {
        let area_tree = minimal_area_tree();
        let renderer = crate::PsRenderer::new();
        let result = renderer
            .render_to_ps(&area_tree)
            .expect("test: should succeed");
        assert!(!result.is_empty(), "PostScript output must not be empty");
    }

    #[test]
    fn test_ps_render_has_ps_header() {
        let area_tree = minimal_area_tree();
        let renderer = crate::PsRenderer::new();
        let result = renderer
            .render_to_ps(&area_tree)
            .expect("test: should succeed");
        assert!(
            result.starts_with("%!PS-Adobe-3.0"),
            "PostScript output must start with %!PS-Adobe-3.0"
        );
    }

    #[test]
    fn test_ps_render_has_eof() {
        let area_tree = minimal_area_tree();
        let renderer = crate::PsRenderer::new();
        let result = renderer
            .render_to_ps(&area_tree)
            .expect("test: should succeed");
        assert!(
            result.contains("%%EOF"),
            "PostScript output must contain %%EOF"
        );
    }

    #[test]
    fn test_ps_render_has_one_page() {
        let area_tree = minimal_area_tree();
        let renderer = crate::PsRenderer::new();
        let result = renderer
            .render_to_ps(&area_tree)
            .expect("test: should succeed");
        assert!(
            result.contains("%%Pages: 1"),
            "single page-sequence must produce 1 PostScript page"
        );
    }

    // ── Text rendering ────────────────────────────────────────────────────────

    #[test]
    fn test_text_render_produces_string() {
        let area_tree = minimal_area_tree();
        let renderer = crate::TextRenderer::new();
        let result = renderer
            .render_to_text(&area_tree)
            .expect("test: should succeed");
        assert!(!result.is_empty(), "text output must not be empty");
    }

    #[test]
    fn test_text_render_contains_content() {
        let area_tree = minimal_area_tree();
        let renderer = crate::TextRenderer::new();
        let result = renderer
            .render_to_text(&area_tree)
            .expect("test: should succeed");
        assert!(
            result.contains("Hello, world!"),
            "text output must contain the rendered text"
        );
    }

    // ── Renderer constructors ─────────────────────────────────────────────────

    #[test]
    fn test_all_renderers_constructible() {
        let _pdf = crate::PdfRenderer::new();
        let _svg = crate::SvgRenderer::new();
        let _ps = crate::PsRenderer::new();
        let _text = crate::TextRenderer::new();
        // all renderers must be constructible without panicking
    }

    #[test]
    fn test_pdf_renderer_with_system_fonts_constructible() {
        let renderer = crate::PdfRenderer::with_system_fonts();
        let _ = renderer;
    }
}
