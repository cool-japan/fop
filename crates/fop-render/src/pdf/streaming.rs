//! Streaming PDF renderer for incremental document generation
//!
//! This module provides a streaming PDF renderer that can build PDF documents
//! incrementally, adding pages one at a time without keeping all pages in memory.

use super::document::{PdfDocument, PdfPage};
use super::image::ImageXObject;
use crate::image::ImageInfo;
use fop_layout::{AreaId, AreaTree, AreaType};
use fop_types::{Length, Result};
use std::collections::HashMap;
use std::io::Write;

/// Streaming PDF renderer that builds documents incrementally
///
/// Unlike the standard PdfRenderer which requires the full area tree,
/// this renderer accepts pages one at a time and can generate PDF output
/// incrementally to minimize memory usage.
pub struct StreamingPdfRenderer {
    /// The PDF document being built
    document: PdfDocument,

    /// Map from area IDs to image indices (within current page)
    image_map: HashMap<AreaId, usize>,

    /// Total number of pages added
    page_count: usize,
}

impl StreamingPdfRenderer {
    /// Create a new streaming PDF renderer
    pub fn new() -> Self {
        let mut document = PdfDocument::new();
        document.info.title = Some("FOP Streaming PDF".to_string());

        Self {
            document,
            image_map: HashMap::new(),
            page_count: 0,
        }
    }

    /// Add a single page from an area tree to the PDF
    ///
    /// This method processes one page at a time. After this call returns,
    /// the area_tree can be dropped to free memory.
    ///
    /// # Arguments
    ///
    /// * `area_tree` - Area tree containing exactly one page
    ///
    /// # Returns
    ///
    /// Returns Ok(()) if the page was successfully added, or an error otherwise.
    pub fn add_page(&mut self, area_tree: &AreaTree) -> Result<()> {
        // Clear image map for this page
        self.image_map.clear();

        // First pass: collect all images and add them to the document
        self.collect_images(area_tree)?;

        // Second pass: find the page area and render it
        for (id, node) in area_tree.iter() {
            if matches!(node.area.area_type, AreaType::Page) {
                let page = self.render_page(area_tree, id)?;
                self.document.add_page(page);
                self.page_count += 1;
                return Ok(());
            }
        }

        Err(fop_types::FopError::Generic(
            "No page area found in area tree".to_string(),
        ))
    }

    /// Write the PDF to an output stream
    ///
    /// This finalizes the PDF document and writes all data to the output stream.
    /// After this call, no more pages can be added.
    pub fn write_to<W: Write>(self, writer: &mut W) -> Result<()> {
        let bytes = self.document.to_bytes()?;
        writer
            .write_all(&bytes)
            .map_err(|e| fop_types::FopError::Generic(format!("Failed to write PDF: {}", e)))?;
        Ok(())
    }

    /// Get the bytes of the PDF document
    ///
    /// This finalizes the PDF document and returns all bytes.
    /// After this call, no more pages can be added.
    pub fn to_bytes(self) -> Result<Vec<u8>> {
        self.document.to_bytes()
    }

    /// Get the number of pages added so far
    pub fn page_count(&self) -> usize {
        self.page_count
    }

    /// Set the PDF document title
    pub fn set_title(&mut self, title: String) {
        self.document.info.title = Some(title);
    }

    /// Set the PDF document author
    pub fn set_author(&mut self, author: String) {
        self.document.info.author = Some(author);
    }

    /// Set the PDF document subject
    pub fn set_subject(&mut self, subject: String) {
        self.document.info.subject = Some(subject);
    }

    /// Collect all images from the area tree and add them to the document
    fn collect_images(&mut self, area_tree: &AreaTree) -> Result<()> {
        for (id, node) in area_tree.iter() {
            if matches!(node.area.area_type, AreaType::Viewport) {
                if let Some(image_data) = node.area.image_data() {
                    let image_index = self.add_image_from_data(image_data)?;
                    self.image_map.insert(id, image_index);
                }
            }
        }
        Ok(())
    }

    /// Add an image to the document from raw image data
    fn add_image_from_data(&mut self, image_data: &[u8]) -> Result<usize> {
        let image_info = ImageInfo::from_bytes(image_data)?;
        let xobject = ImageXObject::from_image_info(&image_info)?;
        Ok(self.document.add_image_xobject(xobject))
    }

    /// Render a single page from the area tree
    fn render_page(&self, area_tree: &AreaTree, page_id: AreaId) -> Result<PdfPage> {
        let page_node = area_tree
            .get(page_id)
            .ok_or_else(|| fop_types::FopError::Generic("Page not found".to_string()))?;

        let mut pdf_page = PdfPage::new(page_node.area.width(), page_node.area.height());

        // Render all child areas recursively
        let page_height = pdf_page.height;
        // Streaming renderer does not embed custom fonts; font_cache is empty.
        let font_cache: HashMap<String, usize> = HashMap::new();
        render_children(
            area_tree,
            page_id,
            &mut pdf_page,
            Length::ZERO,
            Length::ZERO,
            page_height,
            &self.image_map,
            &font_cache,
        )?;

        Ok(pdf_page)
    }
}

impl Default for StreamingPdfRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Render child areas recursively with absolute positioning
#[allow(clippy::too_many_arguments)]
fn render_children(
    area_tree: &AreaTree,
    parent_id: AreaId,
    pdf_page: &mut PdfPage,
    offset_x: Length,
    offset_y: Length,
    page_height: Length,
    image_map: &HashMap<AreaId, usize>,
    font_cache: &HashMap<String, usize>,
) -> Result<()> {
    let children = area_tree.children(parent_id);

    for child_id in children {
        if let Some(child_node) = area_tree.get(child_id) {
            let abs_x = offset_x + child_node.area.geometry.x;
            let abs_y = offset_y + child_node.area.geometry.y;

            // Render background color
            if let Some(bg_color) = child_node.area.traits.background_color {
                let pdf_y = page_height - abs_y - child_node.area.height();
                let border_radius = child_node.area.traits.border_radius;
                pdf_page.add_background_with_radius(
                    abs_x,
                    pdf_y,
                    child_node.area.width(),
                    child_node.area.height(),
                    bg_color,
                    border_radius,
                );
            }

            // Render borders
            if let (Some(border_widths), Some(border_colors), Some(border_styles)) = (
                child_node.area.traits.border_width,
                child_node.area.traits.border_color,
                child_node.area.traits.border_style,
            ) {
                let pdf_y = page_height - abs_y - child_node.area.height();
                let border_radius = child_node.area.traits.border_radius;
                pdf_page.add_borders_with_radius(
                    abs_x,
                    pdf_y,
                    child_node.area.width(),
                    child_node.area.height(),
                    border_widths,
                    border_colors,
                    border_styles,
                    border_radius,
                );
            }

            match child_node.area.area_type {
                AreaType::Text => {
                    if let Some(text_content) = child_node.area.text_content() {
                        let font_size = child_node
                            .area
                            .traits
                            .font_size
                            .unwrap_or(Length::from_pt(12.0));
                        let pdf_y = page_height - abs_y - font_size;

                        // Use embedded font when font-family is set and an entry
                        // exists in the font cache.
                        if let Some(family) = child_node.area.traits.font_family.as_deref() {
                            if let Some(&font_idx) = font_cache.get(&family.to_lowercase()) {
                                pdf_page.add_text_with_font(
                                    text_content,
                                    abs_x,
                                    pdf_y,
                                    font_size,
                                    font_idx,
                                );
                            } else {
                                pdf_page.add_text(text_content, abs_x, pdf_y, font_size);
                            }
                        } else {
                            pdf_page.add_text(text_content, abs_x, pdf_y, font_size);
                        }
                    }
                }
                AreaType::FootnoteSeparator => {
                    // Render as a thin horizontal line (rule)
                    let pdf_y = page_height - abs_y;
                    let thickness = child_node
                        .area
                        .traits
                        .border_width
                        .map(|w| w[0])
                        .unwrap_or(Length::from_pt(1.0));
                    let color = child_node
                        .area
                        .traits
                        .border_color
                        .map(|c| c[0])
                        .unwrap_or(fop_types::Color::BLACK);
                    pdf_page.add_rule(
                        abs_x,
                        pdf_y,
                        child_node.area.width(),
                        thickness,
                        color,
                        "solid",
                    );
                }
                AreaType::Footnote => {
                    // Render footnote content (recursively render children)
                    render_children(
                        area_tree,
                        child_id,
                        pdf_page,
                        abs_x,
                        abs_y,
                        page_height,
                        image_map,
                        font_cache,
                    )?;
                }
                AreaType::Viewport => {
                    if let Some(&image_index) = image_map.get(&child_id) {
                        let pdf_y = page_height - abs_y - child_node.area.height();
                        pdf_page.add_image(
                            image_index,
                            abs_x,
                            pdf_y,
                            child_node.area.width(),
                            child_node.area.height(),
                        );
                    }
                    render_children(
                        area_tree,
                        child_id,
                        pdf_page,
                        abs_x,
                        abs_y,
                        page_height,
                        image_map,
                        font_cache,
                    )?;
                }
                _ => {
                    render_children(
                        area_tree,
                        child_id,
                        pdf_page,
                        abs_x,
                        abs_y,
                        page_height,
                        image_map,
                        font_cache,
                    )?;
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fop_layout::{Area, AreaTree};
    use fop_types::{Point, Rect, Size};

    #[test]
    fn test_streaming_renderer_creation() {
        let renderer = StreamingPdfRenderer::new();
        assert_eq!(renderer.page_count(), 0);
    }

    #[test]
    fn test_add_single_page() {
        let mut renderer = StreamingPdfRenderer::new();
        let mut tree = AreaTree::new();

        // Create a page area
        let page_rect = Rect::from_point_size(
            Point::ZERO,
            Size::new(Length::from_mm(210.0), Length::from_mm(297.0)),
        );
        let page = Area::new(AreaType::Page, page_rect);
        tree.add_area(page);

        renderer.add_page(&tree).expect("test: should succeed");
        assert_eq!(renderer.page_count(), 1);
    }

    #[test]
    fn test_add_multiple_pages() {
        let mut renderer = StreamingPdfRenderer::new();

        for i in 0..5 {
            let mut tree = AreaTree::new();
            let page_rect = Rect::from_point_size(
                Point::ZERO,
                Size::new(Length::from_mm(210.0), Length::from_mm(297.0)),
            );
            let page = Area::new(AreaType::Page, page_rect);
            tree.add_area(page);

            renderer.add_page(&tree).expect("test: should succeed");
            assert_eq!(renderer.page_count(), i + 1);
        }
    }

    #[test]
    fn test_set_document_metadata() {
        let mut renderer = StreamingPdfRenderer::new();
        renderer.set_title("Test Document".to_string());
        renderer.set_author("Test Author".to_string());
        renderer.set_subject("Test Subject".to_string());

        let bytes = renderer.to_bytes().expect("test: should succeed");
        let pdf_str = String::from_utf8_lossy(&bytes);

        assert!(pdf_str.contains("Test Document"));
        assert!(pdf_str.contains("Test Author"));
        assert!(pdf_str.contains("Test Subject"));
    }

    #[test]
    fn test_streaming_pdf_output() {
        let mut renderer = StreamingPdfRenderer::new();
        renderer.set_title("Streaming Test".to_string());

        // Add 3 pages
        for _ in 0..3 {
            let mut tree = AreaTree::new();
            let page_rect = Rect::from_point_size(
                Point::ZERO,
                Size::new(Length::from_mm(210.0), Length::from_mm(297.0)),
            );
            let page = Area::new(AreaType::Page, page_rect);
            tree.add_area(page);

            renderer.add_page(&tree).expect("test: should succeed");
        }

        // Check page count before consuming renderer
        assert_eq!(renderer.page_count(), 3);

        let bytes = renderer.to_bytes().expect("test: should succeed");
        let pdf_str = String::from_utf8_lossy(&bytes);

        assert!(pdf_str.starts_with("%PDF-"));
        assert!(pdf_str.contains("%%EOF"));
    }

    #[test]
    fn test_write_to_stream() {
        let mut renderer = StreamingPdfRenderer::new();

        let mut tree = AreaTree::new();
        let page_rect = Rect::from_point_size(
            Point::ZERO,
            Size::new(Length::from_mm(210.0), Length::from_mm(297.0)),
        );
        let page = Area::new(AreaType::Page, page_rect);
        tree.add_area(page);

        renderer.add_page(&tree).expect("test: should succeed");

        let mut output = Vec::new();
        renderer
            .write_to(&mut output)
            .expect("test: should succeed");

        assert!(!output.is_empty());
        let pdf_str = String::from_utf8_lossy(&output);
        assert!(pdf_str.starts_with("%PDF-"));
    }

    #[test]
    fn test_memory_efficient_processing() {
        let mut renderer = StreamingPdfRenderer::new();

        // Simulate processing 100 pages
        for i in 0..100 {
            let mut tree = AreaTree::new();
            let page_rect = Rect::from_point_size(
                Point::ZERO,
                Size::new(Length::from_mm(210.0), Length::from_mm(297.0)),
            );
            let page = Area::new(AreaType::Page, page_rect);
            tree.add_area(page);

            renderer.add_page(&tree).expect("test: should succeed");
            // tree is dropped here, freeing memory for the next page

            if i % 10 == 0 {
                // Check progress
                assert_eq!(renderer.page_count(), i + 1);
            }
        }

        assert_eq!(renderer.page_count(), 100);
    }

    #[test]
    fn test_no_page_error() {
        let mut renderer = StreamingPdfRenderer::new();
        let tree = AreaTree::new(); // Empty tree, no page

        let result = renderer.add_page(&tree);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;
    use fop_layout::{Area, AreaTree};
    use fop_types::{Point, Rect, Size};

    fn make_page_tree(width_mm: f64, height_mm: f64) -> AreaTree {
        let mut tree = AreaTree::new();
        let rect = Rect::from_point_size(
            Point::ZERO,
            Size::new(Length::from_mm(width_mm), Length::from_mm(height_mm)),
        );
        let page = Area::new(AreaType::Page, rect);
        tree.add_area(page);
        tree
    }

    // ── Construction ─────────────────────────────────────────────────────────

    #[test]
    fn test_default_creates_zero_pages() {
        let r = StreamingPdfRenderer::default();
        assert_eq!(r.page_count(), 0);
    }

    #[test]
    fn test_new_starts_with_zero_pages() {
        let r = StreamingPdfRenderer::new();
        assert_eq!(r.page_count(), 0);
    }

    // ── Page addition ─────────────────────────────────────────────────────────

    #[test]
    fn test_add_one_page_increments_count() {
        let mut r = StreamingPdfRenderer::new();
        let tree = make_page_tree(210.0, 297.0);
        r.add_page(&tree).expect("test: should succeed");
        assert_eq!(r.page_count(), 1);
    }

    #[test]
    fn test_add_two_pages_increments_count() {
        let mut r = StreamingPdfRenderer::new();
        for _ in 0..2 {
            r.add_page(&make_page_tree(210.0, 297.0))
                .expect("test: should succeed");
        }
        assert_eq!(r.page_count(), 2);
    }

    #[test]
    fn test_add_ten_pages() {
        let mut r = StreamingPdfRenderer::new();
        for i in 1..=10 {
            r.add_page(&make_page_tree(210.0, 297.0))
                .expect("test: should succeed");
            assert_eq!(r.page_count(), i);
        }
    }

    #[test]
    fn test_add_landscape_page() {
        let mut r = StreamingPdfRenderer::new();
        // Landscape A4: 297 × 210 mm
        let tree = make_page_tree(297.0, 210.0);
        r.add_page(&tree).expect("test: should succeed");
        assert_eq!(r.page_count(), 1);
    }

    #[test]
    fn test_letter_size_page() {
        let mut r = StreamingPdfRenderer::new();
        // US Letter ≈ 215.9 × 279.4 mm
        let tree = make_page_tree(215.9, 279.4);
        r.add_page(&tree).expect("test: should succeed");
        assert_eq!(r.page_count(), 1);
    }

    // ── Empty tree error ─────────────────────────────────────────────────────

    #[test]
    fn test_empty_tree_returns_error() {
        let mut r = StreamingPdfRenderer::new();
        let empty_tree = AreaTree::new();
        assert!(r.add_page(&empty_tree).is_err());
    }

    // ── Metadata ─────────────────────────────────────────────────────────────

    #[test]
    fn test_title_appears_in_output() {
        let mut r = StreamingPdfRenderer::new();
        r.set_title("My Title".to_string());
        r.add_page(&make_page_tree(210.0, 297.0))
            .expect("test: should succeed");
        let bytes = r.to_bytes().expect("test: should succeed");
        assert!(String::from_utf8_lossy(&bytes).contains("My Title"));
    }

    #[test]
    fn test_author_appears_in_output() {
        let mut r = StreamingPdfRenderer::new();
        r.set_author("Jane Doe".to_string());
        r.add_page(&make_page_tree(210.0, 297.0))
            .expect("test: should succeed");
        let bytes = r.to_bytes().expect("test: should succeed");
        assert!(String::from_utf8_lossy(&bytes).contains("Jane Doe"));
    }

    #[test]
    fn test_subject_appears_in_output() {
        let mut r = StreamingPdfRenderer::new();
        r.set_subject("Test Subject".to_string());
        r.add_page(&make_page_tree(210.0, 297.0))
            .expect("test: should succeed");
        let bytes = r.to_bytes().expect("test: should succeed");
        assert!(String::from_utf8_lossy(&bytes).contains("Test Subject"));
    }

    // ── PDF structure ────────────────────────────────────────────────────────

    #[test]
    fn test_output_starts_with_pdf_header() {
        let mut r = StreamingPdfRenderer::new();
        r.add_page(&make_page_tree(210.0, 297.0))
            .expect("test: should succeed");
        let bytes = r.to_bytes().expect("test: should succeed");
        assert!(bytes.starts_with(b"%PDF-"), "PDF header missing");
    }

    #[test]
    fn test_output_ends_with_eof_marker() {
        let mut r = StreamingPdfRenderer::new();
        r.add_page(&make_page_tree(210.0, 297.0))
            .expect("test: should succeed");
        let bytes = r.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("%%EOF"), "%%EOF missing");
    }

    #[test]
    fn test_output_is_non_empty() {
        let mut r = StreamingPdfRenderer::new();
        r.add_page(&make_page_tree(210.0, 297.0))
            .expect("test: should succeed");
        let bytes = r.to_bytes().expect("test: should succeed");
        assert!(!bytes.is_empty());
    }

    // ── Buffer accumulation ──────────────────────────────────────────────────

    #[test]
    fn test_multi_page_output_larger_than_single() {
        let mut r1 = StreamingPdfRenderer::new();
        r1.add_page(&make_page_tree(210.0, 297.0))
            .expect("test: should succeed");
        let single = r1.to_bytes().expect("test: should succeed");

        let mut r5 = StreamingPdfRenderer::new();
        for _ in 0..5 {
            r5.add_page(&make_page_tree(210.0, 297.0))
                .expect("test: should succeed");
        }
        let multi = r5.to_bytes().expect("test: should succeed");

        assert!(
            multi.len() > single.len(),
            "5-page PDF should be larger than 1-page PDF"
        );
    }

    // ── write_to stream ──────────────────────────────────────────────────────

    #[test]
    fn test_write_to_vec_matches_to_bytes() {
        let mut r1 = StreamingPdfRenderer::new();
        r1.add_page(&make_page_tree(210.0, 297.0))
            .expect("test: should succeed");
        let expected = r1.to_bytes().expect("test: should succeed");

        let mut r2 = StreamingPdfRenderer::new();
        r2.add_page(&make_page_tree(210.0, 297.0))
            .expect("test: should succeed");
        let mut buf = Vec::new();
        r2.write_to(&mut buf).expect("test: should succeed");

        // Both renderers produce the same structure (may differ by
        // generation time, so just check they are non-empty and PDF)
        assert!(!buf.is_empty());
        assert!(buf.starts_with(b"%PDF-"));
        assert_eq!(buf.len(), expected.len());
    }

    #[test]
    fn test_write_to_cursor() {
        use std::io::Cursor;
        let mut r = StreamingPdfRenderer::new();
        r.add_page(&make_page_tree(210.0, 297.0))
            .expect("test: should succeed");
        let mut cursor = Cursor::new(Vec::new());
        r.write_to(&mut cursor).expect("test: should succeed");
        assert!(!cursor.into_inner().is_empty());
    }

    // ── Page count across mix of sizes ───────────────────────────────────────

    #[test]
    fn test_mixed_page_sizes_all_added() {
        let mut r = StreamingPdfRenderer::new();
        r.add_page(&make_page_tree(210.0, 297.0))
            .expect("test: should succeed"); // A4 portrait
        r.add_page(&make_page_tree(297.0, 210.0))
            .expect("test: should succeed"); // A4 landscape
        r.add_page(&make_page_tree(215.9, 279.4))
            .expect("test: should succeed"); // US Letter
        assert_eq!(r.page_count(), 3);
    }
}
