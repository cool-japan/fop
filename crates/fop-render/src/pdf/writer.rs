//! PDF renderer - converts area tree to PDF

use super::compliance::extract_dc_fields;
use super::document::{PdfDocument, PdfPage};
use super::font_config::FontConfig;
use super::image::ImageXObject;
use crate::image::ImageInfo;
use fop_layout::{AreaId, AreaTree, AreaType};
use fop_types::{Length, Result};
use std::collections::HashMap;

use super::document::LinkDestination;

/// PDF renderer
pub struct PdfRenderer {
    /// Default page width (A4)
    #[allow(dead_code)]
    page_width: Length,

    /// Default page height (A4)
    #[allow(dead_code)]
    page_height: Length,

    /// Font configuration: maps family names to TTF file paths
    font_config: FontConfig,
}

impl PdfRenderer {
    /// Create a new PDF renderer with no extra font configuration.
    ///
    /// Call [`PdfRenderer::with_font_config`] to supply a custom
    /// [`FontConfig`], or [`PdfRenderer::with_system_fonts`] to
    /// auto-discover system fonts.
    pub fn new() -> Self {
        Self {
            page_width: Length::from_mm(210.0),
            page_height: Length::from_mm(297.0),
            font_config: FontConfig::new(),
        }
    }

    /// Create a new PDF renderer and populate its font configuration from
    /// standard system font directories.
    pub fn with_system_fonts() -> Self {
        Self {
            page_width: Length::from_mm(210.0),
            page_height: Length::from_mm(297.0),
            font_config: FontConfig::with_system_fonts(),
        }
    }

    /// Attach a custom [`FontConfig`] to this renderer.
    ///
    /// Any existing configuration is replaced.
    pub fn with_font_config(mut self, font_config: FontConfig) -> Self {
        self.font_config = font_config;
        self
    }

    /// Render an area tree and FO tree to a PDF document (with bookmark support)
    pub fn render_with_fo(
        &self,
        area_tree: &AreaTree,
        fo_tree: &fop_core::FoArena,
    ) -> Result<PdfDocument> {
        let mut doc = self.render(area_tree)?;
        // Extract bookmarks from the FO tree and add to document
        if let Ok(Some(outline)) = super::outline::extract_outline_from_fo_tree(fo_tree) {
            doc.set_outline(outline);
        }
        // Set document language from fo:root xml:lang attribute
        if let Some(ref lang) = fo_tree.document_lang {
            doc.info.lang = Some(lang.clone());
        }
        // Bridge XMP metadata from fo:declarations <x:xmpmeta> blocks.
        // The first packet wins; additional packets are ignored for now.
        if let Some(xmp_packet) = fo_tree.xmp_packets.first() {
            // Extract Dublin Core fields and sync them to the /Info dictionary
            // (overwriting the default "FOP Generated PDF" title when a dc:title
            // is present in the XMP so both sources stay consistent).
            let dc = extract_dc_fields(xmp_packet);
            if let Some(title) = dc.title {
                doc.info.title = Some(title);
            }
            if let Some(creator) = dc.creator {
                doc.info.author = Some(creator);
            }
            if let Some(description) = dc.description {
                doc.info.subject = Some(description);
            }
            doc.set_xmp_metadata(xmp_packet.clone());
        }
        Ok(doc)
    }

    /// Render an area tree to a PDF document
    pub fn render(&self, area_tree: &AreaTree) -> Result<PdfDocument> {
        let mut doc = PdfDocument::new();
        doc.info.title = Some("FOP Generated PDF".to_string());

        // First pass: collect all images and add them to the document
        let mut image_map = HashMap::new();
        self.collect_images(area_tree, &mut doc, &mut image_map)?;

        // Second pass: collect all opacity values and create ExtGStates
        let mut opacity_map = HashMap::new();
        self.collect_opacity_states(area_tree, &mut doc, &mut opacity_map);

        // Third pass: build font cache – scan the area tree for font-family
        // values and embed each referenced font into the document.
        let font_cache = self.build_font_cache(area_tree, &mut doc)?;

        // Fourth pass: render pages with image, opacity, and font references
        for (id, node) in area_tree.iter() {
            if matches!(node.area.area_type, AreaType::Page) {
                let page =
                    self.render_page(area_tree, id, &image_map, &opacity_map, &font_cache)?;
                doc.add_page(page);
            }
        }

        Ok(doc)
    }

    /// Scan the area tree for all distinct `font-family` values and embed the
    /// corresponding fonts into `doc`.  Returns a map from family name
    /// (lowercase) to the embedded font index.
    fn build_font_cache(
        &self,
        area_tree: &AreaTree,
        doc: &mut PdfDocument,
    ) -> Result<HashMap<String, usize>> {
        let mut cache: HashMap<String, usize> = HashMap::new();

        for (_, node) in area_tree.iter() {
            if let Some(family) = node.area.traits.font_family.as_deref() {
                let key = family.to_lowercase();
                if cache.contains_key(&key) {
                    continue;
                }

                // Try to find the font file in our configuration
                if let Some(path) = self.font_config.find_font(family) {
                    match std::fs::read(path) {
                        Ok(data) => match doc.embed_font(data) {
                            Ok(idx) => {
                                cache.insert(key, idx);
                            }
                            Err(_) => {
                                // Font embedding failed – text will fall back to
                                // the default Helvetica font.
                            }
                        },
                        Err(_) => {
                            // File not readable – use default font.
                        }
                    }
                } else {
                    // No mapping in the config – check if it matches an already
                    // embedded font by name.
                    if let Some(idx) = doc.font_manager.find_by_name(family) {
                        cache.insert(key, idx);
                    }
                }
            }
        }

        Ok(cache)
    }

    /// Collect all images from the area tree and add them to the document
    fn collect_images(
        &self,
        area_tree: &AreaTree,
        doc: &mut PdfDocument,
        image_map: &mut HashMap<AreaId, usize>,
    ) -> Result<()> {
        for (id, node) in area_tree.iter() {
            // Check if this is a Viewport area (used for images)
            if matches!(node.area.area_type, AreaType::Viewport) {
                // Extract image data from the area
                if let Some(image_data) = node.area.image_data() {
                    // Add the image to the document
                    let image_index = self.add_image_from_data(doc, image_data)?;
                    image_map.insert(id, image_index);
                }
            }
        }
        Ok(())
    }

    /// Add an image to the document from raw image data
    pub fn add_image_from_data(&self, doc: &mut PdfDocument, image_data: &[u8]) -> Result<usize> {
        let image_info = ImageInfo::from_bytes(image_data)?;
        let xobject = ImageXObject::from_image_info(&image_info)?;
        Ok(doc.add_image_xobject(xobject))
    }

    /// Public wrapper for collect_images (for parallel rendering)
    pub fn collect_images_public(
        &self,
        area_tree: &AreaTree,
        doc: &mut PdfDocument,
        image_map: &mut HashMap<AreaId, usize>,
    ) -> Result<()> {
        self.collect_images(area_tree, doc, image_map)
    }

    /// Public wrapper for collect_opacity_states (for parallel rendering)
    pub fn collect_opacity_states_public(
        &self,
        area_tree: &AreaTree,
        doc: &mut PdfDocument,
        opacity_map: &mut HashMap<AreaId, usize>,
    ) {
        self.collect_opacity_states(area_tree, doc, opacity_map)
    }

    /// Public wrapper for render_page (for parallel rendering)
    pub fn render_page_public(
        &self,
        area_tree: &AreaTree,
        page_id: AreaId,
        image_map: &HashMap<AreaId, usize>,
        opacity_map: &HashMap<AreaId, usize>,
        font_cache: &HashMap<String, usize>,
    ) -> Result<PdfPage> {
        self.render_page(area_tree, page_id, image_map, opacity_map, font_cache)
    }

    /// Public wrapper for build_font_cache (for parallel rendering)
    ///
    /// Scans `area_tree` for all distinct `font-family` trait values, embeds
    /// the corresponding fonts into `doc` using this renderer's [`FontConfig`],
    /// and returns a map from family name (lowercase) to embedded font index.
    ///
    /// Font embedding mutates `doc` and therefore cannot be performed
    /// concurrently; call this once sequentially before the parallel page loop.
    pub fn build_font_cache_public(
        &self,
        area_tree: &AreaTree,
        doc: &mut PdfDocument,
    ) -> Result<HashMap<String, usize>> {
        self.build_font_cache(area_tree, doc)
    }

    /// Collect all opacity values from the area tree and create ExtGStates
    fn collect_opacity_states(
        &self,
        area_tree: &AreaTree,
        doc: &mut PdfDocument,
        opacity_map: &mut HashMap<AreaId, usize>,
    ) {
        for (id, node) in area_tree.iter() {
            // Check if this area has opacity set
            if let Some(opacity) = node.area.traits.opacity {
                if (opacity - 1.0).abs() > f64::EPSILON {
                    // Add ExtGState for this opacity (both fill and stroke)
                    let gs_index = doc.add_ext_g_state(opacity, opacity);
                    opacity_map.insert(id, gs_index);
                }
            }
        }
    }

    /// Render a single page
    fn render_page(
        &self,
        area_tree: &AreaTree,
        page_id: AreaId,
        image_map: &HashMap<AreaId, usize>,
        opacity_map: &HashMap<AreaId, usize>,
        font_cache: &HashMap<String, usize>,
    ) -> Result<PdfPage> {
        let page_node = area_tree
            .get(page_id)
            .ok_or_else(|| fop_types::FopError::Generic("Page not found".to_string()))?;

        let mut pdf_page = PdfPage::new(page_node.area.width(), page_node.area.height());

        // Render all child areas recursively with absolute positioning
        let page_height = pdf_page.height;
        render_children(
            area_tree,
            page_id,
            &mut pdf_page,
            Length::ZERO,
            Length::ZERO,
            page_height,
            image_map,
            opacity_map,
            font_cache,
        )?;

        Ok(pdf_page)
    }
}

impl Default for PdfRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Render child areas recursively with absolute positioning
#[allow(clippy::too_many_arguments)]
#[allow(clippy::only_used_in_recursion)]
fn render_children(
    area_tree: &AreaTree,
    parent_id: AreaId,
    pdf_page: &mut PdfPage,
    offset_x: Length,
    offset_y: Length,
    page_height: Length,
    image_map: &HashMap<AreaId, usize>,
    opacity_map: &HashMap<AreaId, usize>,
    font_cache: &HashMap<String, usize>,
) -> Result<()> {
    let children = area_tree.children(parent_id);

    for child_id in children {
        if let Some(child_node) = area_tree.get(child_id) {
            // Calculate absolute position
            let abs_x = offset_x + child_node.area.geometry.x;
            let abs_y = offset_y + child_node.area.geometry.y;

            // Check if clipping is needed for overflow control
            let needs_clipping = child_node
                .area
                .traits
                .overflow
                .map(|o| o.clips_content())
                .unwrap_or(false);

            // Save state and set clipping if overflow=hidden or scroll
            if needs_clipping {
                let pdf_y = page_height - abs_y - child_node.area.height();
                pdf_page.save_clip_state(
                    abs_x,
                    pdf_y,
                    child_node.area.width(),
                    child_node.area.height(),
                )?;
            }

            // Render background color if present (with optional opacity)
            if let Some(bg_color) = child_node.area.traits.background_color {
                let pdf_y = page_height - abs_y - child_node.area.height();
                let border_radius = child_node.area.traits.border_radius;

                // Check if this area has opacity set
                if let Some(&gs_index) = opacity_map.get(&child_id) {
                    // Use opacity-aware rendering
                    pdf_page.add_background_with_opacity(
                        abs_x,
                        pdf_y,
                        child_node.area.width(),
                        child_node.area.height(),
                        bg_color,
                        border_radius,
                        gs_index,
                    );
                } else {
                    // Normal rendering without opacity
                    pdf_page.add_background_with_radius(
                        abs_x,
                        pdf_y,
                        child_node.area.width(),
                        child_node.area.height(),
                        bg_color,
                        border_radius,
                    );
                }
            }

            // Render borders if present (with optional opacity)
            if let (Some(border_widths), Some(border_colors), Some(border_styles)) = (
                child_node.area.traits.border_width,
                child_node.area.traits.border_color,
                child_node.area.traits.border_style,
            ) {
                let pdf_y = page_height - abs_y - child_node.area.height();
                let border_radius = child_node.area.traits.border_radius;

                // Check if this area has opacity set
                if let Some(&gs_index) = opacity_map.get(&child_id) {
                    // Use opacity-aware rendering for borders
                    pdf_page.add_borders_with_opacity(
                        abs_x,
                        pdf_y,
                        child_node.area.width(),
                        child_node.area.height(),
                        border_widths,
                        border_colors,
                        border_styles,
                        border_radius,
                        gs_index,
                    );
                } else {
                    // Normal rendering without opacity
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
            }

            match child_node.area.area_type {
                AreaType::Text => {
                    // Check if this is a leader area
                    if let Some(leader_pattern) = &child_node.area.traits.is_leader {
                        // Render leader based on pattern
                        render_leader(
                            pdf_page,
                            leader_pattern,
                            abs_x,
                            abs_y,
                            child_node.area.width(),
                            child_node.area.height(),
                            page_height,
                            &child_node.area.traits,
                        );
                    } else if let Some(text_content) = child_node.area.text_content() {
                        // Render normal text
                        let font_size = child_node
                            .area
                            .traits
                            .font_size
                            .unwrap_or(Length::from_pt(12.0));

                        // Convert from top-left origin to bottom-left origin (PDF coordinate system)
                        let pdf_y = page_height - abs_y - font_size;

                        // Extract letter-spacing and word-spacing for PDF Tc/Tw operators
                        let letter_spacing = child_node.area.traits.letter_spacing;
                        let word_spacing = child_node.area.traits.word_spacing;

                        // Use embedded font if font-family is set and an embedded
                        // font with that name was found during the font-cache pass.
                        if let Some(family) = child_node.area.traits.font_family.as_deref() {
                            if let Some(&font_idx) = font_cache.get(&family.to_lowercase()) {
                                pdf_page.add_text_with_font_and_spacing(
                                    text_content,
                                    abs_x,
                                    pdf_y,
                                    font_size,
                                    font_idx,
                                    letter_spacing,
                                    word_spacing,
                                );
                            } else {
                                // Family specified but not embedded – fall back to default
                                pdf_page.add_text_with_spacing(
                                    text_content,
                                    abs_x,
                                    pdf_y,
                                    font_size,
                                    letter_spacing,
                                    word_spacing,
                                );
                            }
                        } else {
                            pdf_page.add_text_with_spacing(
                                text_content,
                                abs_x,
                                pdf_y,
                                font_size,
                                letter_spacing,
                                word_spacing,
                            );
                        }

                        // Add link annotation if this text has a link destination
                        if let Some(link_dest) = &child_node.area.traits.link_destination {
                            let destination = if link_dest.starts_with("http://")
                                || link_dest.starts_with("https://")
                                || link_dest.starts_with("mailto:")
                            {
                                LinkDestination::External(link_dest.clone())
                            } else {
                                LinkDestination::Internal(link_dest.clone())
                            };

                            // Add annotation for the text area
                            pdf_page.add_link_annotation(
                                abs_x,
                                pdf_y,
                                child_node.area.width(),
                                font_size,
                                destination,
                            );
                        }
                    }
                }
                AreaType::Inline => {
                    // Check if this is a leader area without text content
                    if let Some(leader_pattern) = &child_node.area.traits.is_leader {
                        render_leader(
                            pdf_page,
                            leader_pattern,
                            abs_x,
                            abs_y,
                            child_node.area.width(),
                            child_node.area.height(),
                            page_height,
                            &child_node.area.traits,
                        );
                    } else {
                        // Check if this inline area has a link destination
                        if let Some(link_dest) = &child_node.area.traits.link_destination {
                            let destination = if link_dest.starts_with("http://")
                                || link_dest.starts_with("https://")
                                || link_dest.starts_with("mailto:")
                            {
                                LinkDestination::External(link_dest.clone())
                            } else {
                                LinkDestination::Internal(link_dest.clone())
                            };

                            // Convert to PDF coordinates
                            let pdf_y = page_height - abs_y - child_node.area.height();

                            // Add annotation for the inline area
                            pdf_page.add_link_annotation(
                                abs_x,
                                pdf_y,
                                child_node.area.width(),
                                child_node.area.height(),
                                destination,
                            );
                        }

                        // Recursively render inline areas children
                        render_children(
                            area_tree,
                            child_id,
                            pdf_page,
                            abs_x,
                            abs_y,
                            page_height,
                            image_map,
                            opacity_map,
                            font_cache,
                        )?;
                    }
                }
                AreaType::Viewport => {
                    // Render image if this viewport has an associated image
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
                    // Still recurse in case there are child areas
                    render_children(
                        area_tree,
                        child_id,
                        pdf_page,
                        abs_x,
                        abs_y,
                        page_height,
                        image_map,
                        opacity_map,
                        font_cache,
                    )?;
                }
                _ => {
                    // Recursively render other areas with accumulated offsets
                    render_children(
                        area_tree,
                        child_id,
                        pdf_page,
                        abs_x,
                        abs_y,
                        page_height,
                        image_map,
                        opacity_map,
                        font_cache,
                    )?;
                }
            }

            // Restore graphics state if clipping was applied
            if needs_clipping {
                pdf_page.restore_clip_state()?;
            }
        }
    }

    Ok(())
}

/// Render a leader (dots, rule, space, etc.)
#[allow(clippy::too_many_arguments)]
fn render_leader(
    pdf_page: &mut PdfPage,
    leader_pattern: &str,
    x: Length,
    y: Length,
    width: Length,
    height: Length,
    page_height: Length,
    traits: &fop_layout::area::TraitSet,
) {
    match leader_pattern {
        "rule" => {
            // Render as a horizontal line
            let thickness = traits.rule_thickness.unwrap_or(Length::from_pt(0.5));

            let style = traits.rule_style.as_deref().unwrap_or("solid");

            // Center the rule vertically within the leader area
            let half_diff = Length::from_millipoints((height - thickness).millipoints() / 2);
            let rule_y = y + half_diff;

            // Convert to PDF coordinates (bottom-left origin)
            let pdf_y = page_height - rule_y - thickness;

            // Get color from traits or default to black
            let color = traits.color.unwrap_or(fop_types::Color::BLACK);

            pdf_page.add_rule(x, pdf_y, width, thickness, color, style);
        }
        "dots" => {
            // This is already handled by text rendering with the dot pattern
            // The layout engine generates the dot string
        }
        "space" => {
            // Space leaders render nothing (just occupy space)
        }
        _ => {
            // Unknown pattern, default to space behavior
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fop_layout::{Area, AreaTree};
    use fop_types::{Point, Rect, Size};

    #[test]
    fn test_renderer_creation() {
        let renderer = PdfRenderer::new();
        assert_eq!(renderer.page_width, Length::from_mm(210.0));
        assert_eq!(renderer.page_height, Length::from_mm(297.0));
    }

    #[test]
    fn test_render_empty_tree() {
        let renderer = PdfRenderer::new();
        let tree = AreaTree::new();

        let doc = renderer.render(&tree).expect("test: should succeed");
        assert_eq!(doc.pages.len(), 0);
    }

    #[test]
    fn test_render_single_page() {
        let renderer = PdfRenderer::new();
        let mut tree = AreaTree::new();

        // Create a page area
        let page_rect = Rect::from_point_size(
            Point::ZERO,
            Size::new(Length::from_mm(210.0), Length::from_mm(297.0)),
        );
        let page = Area::new(AreaType::Page, page_rect);
        tree.add_area(page);

        let doc = renderer.render(&tree).expect("test: should succeed");
        assert_eq!(doc.pages.len(), 1);
    }

    #[test]
    fn test_add_image_to_document() {
        let renderer = PdfRenderer::new();
        let mut doc = PdfDocument::new();

        // Create a minimal valid PNG image (1x1 red pixel)
        let mut png_data = Vec::new();
        let mut encoder = png::Encoder::new(&mut png_data, 1, 1);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header().expect("test: should succeed");
        let data = vec![255, 0, 0]; // Red pixel
        writer
            .write_image_data(&data)
            .expect("test: should succeed");
        drop(writer);

        // Add the image to the document
        let image_index = renderer
            .add_image_from_data(&mut doc, &png_data)
            .expect("test: should succeed");
        assert_eq!(image_index, 0);
        assert_eq!(doc.image_xobjects.len(), 1);

        // Verify the image XObject properties
        let xobject = &doc.image_xobjects[0];
        assert_eq!(xobject.width, 1);
        assert_eq!(xobject.height, 1);
        assert_eq!(xobject.color_space, "DeviceRGB");
        assert_eq!(xobject.filter, "FlateDecode");
    }

    #[test]
    fn test_pdf_with_image_generates_valid_bytes() {
        let renderer = PdfRenderer::new();
        let mut doc = PdfDocument::new();

        // Create a minimal PNG
        let mut png_data = Vec::new();
        let mut encoder = png::Encoder::new(&mut png_data, 2, 2);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);

        let mut writer = encoder.write_header().expect("test: should succeed");
        let data = vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255]; // 2x2 RGBW
        writer
            .write_image_data(&data)
            .expect("test: should succeed");
        drop(writer);

        // Add image and a page
        renderer
            .add_image_from_data(&mut doc, &png_data)
            .expect("test: should succeed");

        let mut page = super::PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0));
        page.add_image(
            0,
            Length::from_pt(100.0),
            Length::from_pt(100.0),
            Length::from_pt(50.0),
            Length::from_pt(50.0),
        );
        doc.add_page(page);

        // Generate PDF bytes
        let bytes = doc.to_bytes().expect("test: should succeed");

        // Verify PDF structure
        let pdf_str = String::from_utf8_lossy(&bytes);
        assert!(pdf_str.starts_with("%PDF-"));
        assert!(pdf_str.contains("/Type /XObject"));
        assert!(pdf_str.contains("/Subtype /Image"));
        assert!(pdf_str.contains("/Filter /FlateDecode"));
        assert!(pdf_str.contains("/Im0 Do")); // Image rendering command
        assert!(pdf_str.contains("%%EOF"));
    }
}

#[cfg(test)]
mod tests_writer_comprehensive {
    use super::*;
    use fop_layout::{Area, AreaTree, AreaType};
    use fop_types::{Length, Point, Rect, Size};

    fn make_page_area(w_mm: f64, h_mm: f64) -> Area {
        let rect = Rect::from_point_size(
            Point::ZERO,
            Size::new(Length::from_mm(w_mm), Length::from_mm(h_mm)),
        );
        Area::new(AreaType::Page, rect)
    }

    // ── PdfRenderer::new() ────────────────────────────────────────────────────

    #[test]
    fn test_renderer_new_default_page_width() {
        let r = PdfRenderer::new();
        assert_eq!(r.page_width, Length::from_mm(210.0));
    }

    #[test]
    fn test_renderer_new_default_page_height() {
        let r = PdfRenderer::new();
        assert_eq!(r.page_height, Length::from_mm(297.0));
    }

    #[test]
    fn test_renderer_default_equals_new() {
        let r1 = PdfRenderer::new();
        let r2 = PdfRenderer::default();
        assert_eq!(r1.page_width, r2.page_width);
        assert_eq!(r1.page_height, r2.page_height);
    }

    // ── render() on empty tree ────────────────────────────────────────────────

    #[test]
    fn test_render_empty_tree_no_pages() {
        let r = PdfRenderer::new();
        let tree = AreaTree::new();
        let doc = r.render(&tree).expect("test: should succeed");
        assert_eq!(doc.pages.len(), 0);
    }

    #[test]
    fn test_render_empty_tree_produces_valid_pdf() {
        let r = PdfRenderer::new();
        let tree = AreaTree::new();
        let doc = r.render(&tree).expect("test: should succeed");
        let bytes = doc.to_bytes().expect("test: should succeed");
        assert!(bytes.starts_with(b"%PDF-"));
    }

    // ── render() with pages ───────────────────────────────────────────────────

    #[test]
    fn test_render_one_page_produces_one_page_doc() {
        let r = PdfRenderer::new();
        let mut tree = AreaTree::new();
        tree.add_area(make_page_area(210.0, 297.0));
        let doc = r.render(&tree).expect("test: should succeed");
        assert_eq!(doc.pages.len(), 1);
    }

    #[test]
    fn test_render_two_pages_produces_two_page_doc() {
        let r = PdfRenderer::new();
        let mut tree = AreaTree::new();
        tree.add_area(make_page_area(210.0, 297.0));
        tree.add_area(make_page_area(210.0, 297.0));
        let doc = r.render(&tree).expect("test: should succeed");
        assert_eq!(doc.pages.len(), 2);
    }

    #[test]
    fn test_render_five_pages_produces_five_page_doc() {
        let r = PdfRenderer::new();
        let mut tree = AreaTree::new();
        for _ in 0..5 {
            tree.add_area(make_page_area(210.0, 297.0));
        }
        let doc = r.render(&tree).expect("test: should succeed");
        assert_eq!(doc.pages.len(), 5);
    }

    #[test]
    fn test_render_page_count_in_catalog_bytes() {
        let r = PdfRenderer::new();
        let mut tree = AreaTree::new();
        tree.add_area(make_page_area(210.0, 297.0));
        tree.add_area(make_page_area(210.0, 297.0));
        let doc = r.render(&tree).expect("test: should succeed");
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/Count 2"));
    }

    // ── render() sets title ───────────────────────────────────────────────────

    #[test]
    fn test_render_sets_default_title() {
        let r = PdfRenderer::new();
        let tree = AreaTree::new();
        let doc = r.render(&tree).expect("test: should succeed");
        assert_eq!(doc.info.title.as_deref(), Some("FOP Generated PDF"));
    }

    // ── render() output is valid PDF ──────────────────────────────────────────

    #[test]
    fn test_render_output_has_eof_marker() {
        let r = PdfRenderer::new();
        let tree = AreaTree::new();
        let doc = r.render(&tree).expect("test: should succeed");
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("%%EOF"));
    }

    #[test]
    fn test_render_output_has_catalog() {
        let r = PdfRenderer::new();
        let tree = AreaTree::new();
        let doc = r.render(&tree).expect("test: should succeed");
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/Type /Catalog"));
    }

    #[test]
    fn test_render_output_has_pages_dict() {
        let r = PdfRenderer::new();
        let tree = AreaTree::new();
        let doc = r.render(&tree).expect("test: should succeed");
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/Type /Pages"));
    }

    #[test]
    fn test_render_output_has_font_resource() {
        let r = PdfRenderer::new();
        let tree = AreaTree::new();
        let doc = r.render(&tree).expect("test: should succeed");
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        // Default Helvetica font should be present
        assert!(s.contains("/BaseFont /Helvetica"));
    }

    // ── add_image_from_data() ─────────────────────────────────────────────────

    fn make_png_1x1_red() -> Vec<u8> {
        let mut buf = Vec::new();
        let mut enc = png::Encoder::new(&mut buf, 1, 1);
        enc.set_color(png::ColorType::Rgb);
        enc.set_depth(png::BitDepth::Eight);
        let mut w = enc.write_header().expect("test: should succeed");
        w.write_image_data(&[255, 0, 0])
            .expect("test: should succeed");
        drop(w);
        buf
    }

    #[test]
    fn test_add_image_returns_index_zero_for_first() {
        let r = PdfRenderer::new();
        let mut doc = super::PdfDocument::new();
        let idx = r
            .add_image_from_data(&mut doc, &make_png_1x1_red())
            .expect("test: should succeed");
        assert_eq!(idx, 0);
    }

    #[test]
    fn test_add_image_increments_index_for_second() {
        let r = PdfRenderer::new();
        let mut doc = super::PdfDocument::new();
        r.add_image_from_data(&mut doc, &make_png_1x1_red())
            .expect("test: should succeed");
        let idx2 = r
            .add_image_from_data(&mut doc, &make_png_1x1_red())
            .expect("test: should succeed");
        assert_eq!(idx2, 1);
    }

    #[test]
    fn test_add_image_grows_image_xobjects() {
        let r = PdfRenderer::new();
        let mut doc = super::PdfDocument::new();
        r.add_image_from_data(&mut doc, &make_png_1x1_red())
            .expect("test: should succeed");
        r.add_image_from_data(&mut doc, &make_png_1x1_red())
            .expect("test: should succeed");
        assert_eq!(doc.image_xobjects.len(), 2);
    }

    // ── collect_images_public() ───────────────────────────────────────────────

    #[test]
    fn test_collect_images_public_empty_tree_no_images() {
        let r = PdfRenderer::new();
        let tree = AreaTree::new();
        let mut doc = super::PdfDocument::new();
        let mut map = HashMap::new();
        r.collect_images_public(&tree, &mut doc, &mut map)
            .expect("test: should succeed");
        assert!(doc.image_xobjects.is_empty());
        assert!(map.is_empty());
    }

    // ── collect_opacity_states_public() ──────────────────────────────────────

    #[test]
    fn test_collect_opacity_states_empty_tree_no_states() {
        let r = PdfRenderer::new();
        let tree = AreaTree::new();
        let mut doc = super::PdfDocument::new();
        let mut map = HashMap::new();
        r.collect_opacity_states_public(&tree, &mut doc, &mut map);
        assert!(doc.ext_g_states.is_empty());
        assert!(map.is_empty());
    }

    // ── render_page_public() ─────────────────────────────────────────────────

    #[test]
    fn test_render_page_public_produces_correct_dimensions() {
        let r = PdfRenderer::new();
        let mut tree = AreaTree::new();
        let page_id = tree.add_area(make_page_area(210.0, 297.0));
        let doc = super::PdfDocument::new();
        let _ = doc; // doc not needed for page render directly
        let img_map = HashMap::new();
        let op_map = HashMap::new();
        let font_cache = HashMap::new();
        let page = r
            .render_page_public(&tree, page_id, &img_map, &op_map, &font_cache)
            .expect("test: should succeed");
        assert_eq!(page.width, Length::from_mm(210.0));
        assert_eq!(page.height, Length::from_mm(297.0));
    }

    // ── with_system_fonts() ───────────────────────────────────────────────────

    #[test]
    fn test_with_system_fonts_can_render_empty_tree() {
        let r = PdfRenderer::with_system_fonts();
        let tree = AreaTree::new();
        let doc = r.render(&tree).expect("test: should succeed");
        assert_eq!(doc.pages.len(), 0);
    }

    // ── Full round-trip: render + to_bytes ────────────────────────────────────

    #[test]
    fn test_full_round_trip_single_page_pdf_is_valid() {
        let r = PdfRenderer::new();
        let mut tree = AreaTree::new();
        tree.add_area(make_page_area(210.0, 297.0));
        let doc = r.render(&tree).expect("test: should succeed");
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.starts_with("%PDF-"));
        assert!(s.contains("%%EOF"));
        assert!(s.contains("/Count 1"));
    }

    #[test]
    fn test_a5_page_dimensions_in_media_box() {
        let r = PdfRenderer::new();
        let mut tree = AreaTree::new();
        // A5 is 148x210mm
        tree.add_area(make_page_area(148.0, 210.0));
        let doc = r.render(&tree).expect("test: should succeed");
        let bytes = doc.to_bytes().expect("test: should succeed");
        let s = String::from_utf8_lossy(&bytes);
        assert!(s.contains("/MediaBox"));
    }
}
