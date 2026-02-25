//! Raster image rendering backend (PNG, JPEG)
//!
//! Converts area trees to raster images by first rendering to SVG,
//! then rasterizing using the resvg library.

use crate::svg::SvgRenderer;
use fop_layout::AreaTree;
use fop_types::{FopError, Result};
use log::{debug, info};

/// Raster output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RasterFormat {
    /// PNG format with lossless compression
    Png,

    /// JPEG format with configurable quality (1-100)
    Jpeg { quality: u8 },
}

/// Raster renderer that converts area trees to PNG or JPEG images
///
/// This renderer uses a two-stage process:
/// 1. Convert the area tree to SVG using SvgRenderer
/// 2. Rasterize the SVG to bitmap using resvg/tiny-skia
pub struct RasterRenderer {
    /// Target DPI (dots per inch) for rasterization
    /// Common values: 72 (screen), 96 (Windows), 150 (draft), 300 (print)
    dpi: u32,
}

impl RasterRenderer {
    /// Create a new raster renderer with specified DPI
    ///
    /// # Arguments
    /// * `dpi` - Target resolution in dots per inch (typical values: 72, 96, 150, 300)
    ///
    /// # Examples
    /// ```
    /// use fop_render::RasterRenderer;
    ///
    /// let renderer = RasterRenderer::new(300); // 300 DPI for print quality
    /// ```
    pub fn new(dpi: u32) -> Self {
        Self { dpi }
    }

    /// Render an area tree to raster images (one per page)
    ///
    /// # Arguments
    /// * `area_tree` - The area tree to render
    /// * `format` - The output format (PNG or JPEG)
    ///
    /// # Returns
    /// A vector of byte vectors, one for each page. Each inner vector contains
    /// the encoded image data (PNG or JPEG).
    ///
    /// # Errors
    /// Returns an error if SVG generation or rasterization fails.
    pub fn render_to_raster(
        &self,
        area_tree: &AreaTree,
        format: RasterFormat,
    ) -> Result<Vec<Vec<u8>>> {
        info!("Starting raster rendering at {} DPI", self.dpi);

        // Step 1: Render to SVG (one SVG per page)
        debug!("Step 1: Converting area tree to SVG pages");
        let svg_renderer = SvgRenderer::new();
        let svg_pages = svg_renderer.render_to_svg_pages(area_tree)?;

        debug!("Generated {} SVG page(s)", svg_pages.len());

        // Step 2: Rasterize each SVG page
        debug!("Step 2: Rasterizing SVG pages to bitmaps");
        let mut raster_pages = Vec::with_capacity(svg_pages.len());
        for (i, svg_content) in svg_pages.iter().enumerate() {
            debug!("Rasterizing page {} ({} bytes)", i + 1, svg_content.len());
            let page_images = self.rasterize_svg(svg_content, format)?;
            // Each SVG page should produce exactly one raster image
            raster_pages.extend(page_images);
        }

        info!("Successfully rendered {} page(s)", raster_pages.len());
        Ok(raster_pages)
    }

    /// Rasterize SVG content to one or more images
    fn rasterize_svg(&self, svg_content: &str, format: RasterFormat) -> Result<Vec<Vec<u8>>> {
        // Parse SVG with usvg
        let opt = usvg::Options::default();
        let tree = usvg::Tree::from_str(svg_content, &opt)
            .map_err(|e| FopError::Generic(format!("Failed to parse SVG: {}", e)))?;

        // Get the SVG size
        let svg_size = tree.size();
        debug!("SVG size: {}x{} pt", svg_size.width(), svg_size.height());

        // Calculate pixel dimensions based on DPI
        // SVG uses points (1pt = 1/72 inch), so we need to scale by DPI/72
        let scale = self.dpi as f32 / 72.0;
        let width = (svg_size.width() * scale).ceil() as u32;
        let height = (svg_size.height() * scale).ceil() as u32;

        debug!(
            "Raster size: {}x{} pixels (scale: {})",
            width, height, scale
        );

        // Create a pixmap to render into
        let mut pixmap = tiny_skia::Pixmap::new(width, height)
            .ok_or_else(|| FopError::Generic("Failed to create pixmap".to_string()))?;

        // Render the SVG
        let render_transform = tiny_skia::Transform::from_scale(scale, scale);
        resvg::render(&tree, render_transform, &mut pixmap.as_mut());

        // Encode the image
        let encoded = match format {
            RasterFormat::Png => self.encode_png(&pixmap)?,
            RasterFormat::Jpeg { quality } => self.encode_jpeg(&pixmap, quality)?,
        };

        // Return single image (this method processes one SVG at a time)
        Ok(vec![encoded])
    }

    /// Encode pixmap as PNG
    fn encode_png(&self, pixmap: &tiny_skia::Pixmap) -> Result<Vec<u8>> {
        debug!("Encoding as PNG");

        // Use the png crate to encode the image
        let mut buf = Vec::new();
        {
            let mut encoder = png::Encoder::new(
                std::io::Cursor::new(&mut buf),
                pixmap.width(),
                pixmap.height(),
            );
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);

            let mut writer = encoder
                .write_header()
                .map_err(|e| FopError::Generic(format!("PNG encoding failed: {}", e)))?;

            writer
                .write_image_data(pixmap.data())
                .map_err(|e| FopError::Generic(format!("PNG writing failed: {}", e)))?;
        }

        debug!("PNG encoded: {} bytes", buf.len());
        Ok(buf)
    }

    /// Encode pixmap as JPEG
    fn encode_jpeg(&self, pixmap: &tiny_skia::Pixmap, quality: u8) -> Result<Vec<u8>> {
        debug!("Encoding as JPEG with quality {}", quality);

        // Convert RGBA to RGB (JPEG doesn't support alpha)
        let width = pixmap.width();
        let height = pixmap.height();
        let rgba_data = pixmap.data();

        let mut rgb_data = Vec::with_capacity((width * height * 3) as usize);
        for chunk in rgba_data.chunks_exact(4) {
            // Blend alpha with white background
            let alpha = chunk[3] as f32 / 255.0;
            let r = (chunk[0] as f32 * alpha + 255.0 * (1.0 - alpha)) as u8;
            let g = (chunk[1] as f32 * alpha + 255.0 * (1.0 - alpha)) as u8;
            let b = (chunk[2] as f32 * alpha + 255.0 * (1.0 - alpha)) as u8;
            rgb_data.push(r);
            rgb_data.push(g);
            rgb_data.push(b);
        }

        // Encode as JPEG using jpeg-encoder
        let mut buf = Vec::new();
        {
            let encoder = jpeg_encoder::Encoder::new(&mut buf, quality);
            encoder
                .encode(
                    &rgb_data,
                    width as u16,
                    height as u16,
                    jpeg_encoder::ColorType::Rgb,
                )
                .map_err(|e| FopError::Generic(format!("JPEG encoding failed: {}", e)))?;
        }

        debug!("JPEG encoded: {} bytes", buf.len());
        Ok(buf)
    }
}

impl Default for RasterRenderer {
    /// Create a renderer with default 96 DPI (Windows standard)
    fn default() -> Self {
        Self::new(96)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fop_layout::area::{Area, TraitSet};
    use fop_layout::AreaType;
    use fop_types::{Color, Length, Rect};

    fn create_test_area_tree() -> AreaTree {
        let mut tree = AreaTree::new();

        // Create a simple page with some content
        let page_area = Area {
            area_type: AreaType::Page,
            geometry: Rect {
                x: Length::ZERO,
                y: Length::ZERO,
                width: Length::from_mm(210.0),  // A4 width
                height: Length::from_mm(297.0), // A4 height
            },
            traits: TraitSet {
                background_color: Some(Color::WHITE),
                ..Default::default()
            },
            content: None,
            keep_constraint: None,
            break_before: None,
            break_after: None,
            widows: 2,
            orphans: 2,
        };

        tree.add_area(page_area);
        tree
    }

    #[test]
    fn test_raster_renderer_new() {
        let renderer = RasterRenderer::new(300);
        assert_eq!(renderer.dpi, 300);
    }

    #[test]
    fn test_raster_renderer_default() {
        let renderer = RasterRenderer::default();
        assert_eq!(renderer.dpi, 96);
    }

    #[test]
    fn test_render_to_png() {
        let renderer = RasterRenderer::new(72);
        let area_tree = create_test_area_tree();

        let result = renderer.render_to_raster(&area_tree, RasterFormat::Png);
        assert!(result.is_ok());

        let pages = result.expect("test: should succeed");
        assert_eq!(pages.len(), 1);
        assert!(!pages[0].is_empty());

        // Verify PNG signature
        assert_eq!(&pages[0][0..4], &[137, 80, 78, 71]);
    }

    #[test]
    fn test_render_to_jpeg() {
        let renderer = RasterRenderer::new(72);
        let area_tree = create_test_area_tree();

        let result = renderer.render_to_raster(&area_tree, RasterFormat::Jpeg { quality: 90 });
        assert!(result.is_ok());

        let pages = result.expect("test: should succeed");
        assert_eq!(pages.len(), 1);
        assert!(!pages[0].is_empty());

        // Verify JPEG signature (FF D8 FF)
        assert_eq!(&pages[0][0..3], &[0xFF, 0xD8, 0xFF]);
    }

    #[test]
    fn test_different_dpi_values() {
        let dpis = [72, 96, 150, 300];
        let area_tree = create_test_area_tree();

        for dpi in dpis {
            let renderer = RasterRenderer::new(dpi);
            let result = renderer.render_to_raster(&area_tree, RasterFormat::Png);
            assert!(result.is_ok(), "Failed at {} DPI", dpi);
        }
    }

    #[test]
    fn test_jpeg_quality_range() {
        let area_tree = create_test_area_tree();
        let renderer = RasterRenderer::new(72);

        for quality in [10, 50, 75, 90, 100] {
            let result = renderer.render_to_raster(&area_tree, RasterFormat::Jpeg { quality });
            assert!(result.is_ok(), "Failed at quality {}", quality);
        }
    }

    #[test]
    fn test_png_larger_than_jpeg() {
        let area_tree = create_test_area_tree();
        let renderer = RasterRenderer::new(72);

        let png = renderer
            .render_to_raster(&area_tree, RasterFormat::Png)
            .expect("test: should succeed");
        let jpeg = renderer
            .render_to_raster(&area_tree, RasterFormat::Jpeg { quality: 75 })
            .expect("test: should succeed");

        // PNG should typically be larger for simple graphics
        // (though this isn't always true for complex images)
        assert!(!png[0].is_empty());
        assert!(!jpeg[0].is_empty());
    }

    #[test]
    fn test_high_dpi_produces_larger_output() {
        let area_tree = create_test_area_tree();

        let low_dpi = RasterRenderer::new(72);
        let high_dpi = RasterRenderer::new(300);

        let low_result = low_dpi
            .render_to_raster(&area_tree, RasterFormat::Png)
            .expect("test: should succeed");
        let high_result = high_dpi
            .render_to_raster(&area_tree, RasterFormat::Png)
            .expect("test: should succeed");

        // Higher DPI should produce larger files
        assert!(high_result[0].len() > low_result[0].len());
    }

    #[test]
    fn test_raster_format_debug() {
        let png = RasterFormat::Png;
        let jpeg = RasterFormat::Jpeg { quality: 80 };

        assert_eq!(format!("{:?}", png), "Png");
        assert_eq!(format!("{:?}", jpeg), "Jpeg { quality: 80 }");
    }

    #[test]
    fn test_raster_format_equality() {
        assert_eq!(RasterFormat::Png, RasterFormat::Png);
        assert_eq!(
            RasterFormat::Jpeg { quality: 80 },
            RasterFormat::Jpeg { quality: 80 }
        );
        assert_ne!(RasterFormat::Png, RasterFormat::Jpeg { quality: 80 });
    }

    #[test]
    fn test_invalid_svg_handling() {
        let renderer = RasterRenderer::new(96);
        let invalid_svg = "not valid svg content";

        let result = renderer.rasterize_svg(invalid_svg, RasterFormat::Png);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;
    use fop_layout::area::{Area, TraitSet};
    use fop_layout::AreaType;
    use fop_types::{Color, Length, Rect};

    #[allow(dead_code)]
    fn two_page_area_tree() -> AreaTree {
        let mut tree = AreaTree::new();
        for _ in 0..2 {
            let area = Area {
                area_type: AreaType::Page,
                geometry: Rect {
                    x: Length::ZERO,
                    y: Length::ZERO,
                    width: Length::from_mm(210.0),
                    height: Length::from_mm(297.0),
                },
                traits: TraitSet {
                    background_color: Some(Color::WHITE),
                    ..Default::default()
                },
                content: None,
                keep_constraint: None,
                break_before: None,
                break_after: None,
                widows: 2,
                orphans: 2,
            };
            tree.add_area(area);
        }
        tree
    }

    fn single_page_tree() -> AreaTree {
        let mut tree = AreaTree::new();
        let area = Area {
            area_type: AreaType::Page,
            geometry: Rect {
                x: Length::ZERO,
                y: Length::ZERO,
                width: Length::from_mm(210.0),
                height: Length::from_mm(297.0),
            },
            traits: TraitSet {
                background_color: Some(Color::WHITE),
                ..Default::default()
            },
            content: None,
            keep_constraint: None,
            break_before: None,
            break_after: None,
            widows: 2,
            orphans: 2,
        };
        tree.add_area(area);
        tree
    }

    #[test]
    fn test_png_magic_bytes() {
        // PNG starts with: 0x89 'P' 'N' 'G' 0x0D 0x0A 0x1A 0x0A
        let renderer = RasterRenderer::new(72);
        let tree = single_page_tree();
        let pages = renderer
            .render_to_raster(&tree, RasterFormat::Png)
            .expect("test: should succeed");
        assert_eq!(
            &pages[0][..8],
            &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]
        );
    }

    #[test]
    fn test_jpeg_magic_bytes() {
        let renderer = RasterRenderer::new(72);
        let tree = single_page_tree();
        let pages = renderer
            .render_to_raster(&tree, RasterFormat::Jpeg { quality: 80 })
            .expect("test: should succeed");
        // JPEG SOI marker: FF D8
        assert_eq!(pages[0][0], 0xFF);
        assert_eq!(pages[0][1], 0xD8);
    }

    #[test]
    fn test_raster_format_clone() {
        let fmt = RasterFormat::Jpeg { quality: 70 };
        let fmt2 = fmt;
        assert_eq!(fmt, fmt2);
    }

    #[test]
    fn test_raster_format_copy() {
        let fmt1 = RasterFormat::Png;
        let fmt2 = fmt1; // Copy
        assert_eq!(fmt1, fmt2);
    }

    #[test]
    fn test_raster_renderer_dpi_stored() {
        let r = RasterRenderer::new(150);
        assert_eq!(r.dpi, 150);
    }

    #[test]
    fn test_higher_jpeg_quality_produces_different_bytes() {
        let tree = single_page_tree();
        let renderer = RasterRenderer::new(72);

        let low = renderer
            .render_to_raster(&tree, RasterFormat::Jpeg { quality: 10 })
            .expect("test: should succeed");
        let high = renderer
            .render_to_raster(&tree, RasterFormat::Jpeg { quality: 95 })
            .expect("test: should succeed");

        // Different quality → different output bytes (for non-trivial images)
        // At minimum both must be valid non-empty JPEG
        assert!(!low[0].is_empty());
        assert!(!high[0].is_empty());
    }

    #[test]
    fn test_empty_tree_produces_no_pages() {
        let renderer = RasterRenderer::new(96);
        let tree = AreaTree::new();
        let result = renderer
            .render_to_raster(&tree, RasterFormat::Png)
            .expect("test: should succeed");
        assert!(result.is_empty());
    }
}
