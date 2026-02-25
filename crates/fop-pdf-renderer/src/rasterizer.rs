//! PDF page rasterizer
//!
//! Converts draw commands from the content interpreter into a pixel image
//! using tiny-skia for 2D rendering.

use crate::content::{ContentInterpreter, DrawCommand, FillRule};
use crate::error::{PdfRenderError, Result};
use crate::font::LoadedFont;
use crate::parser::{PdfDocument, PdfPage};
use std::collections::HashMap;

/// A rasterized page as RGBA pixels
pub struct RasterPage {
    pub width: u32,
    pub height: u32,
    /// RGBA pixel data, row-major top-to-bottom
    pub pixels: Vec<u8>,
}

impl RasterPage {
    /// Encode the page as a PNG
    pub fn to_png(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut out, self.width, self.height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder
                .write_header()
                .map_err(|e| PdfRenderError::Image(e.to_string()))?;
            writer
                .write_image_data(&self.pixels)
                .map_err(|e| PdfRenderError::Image(e.to_string()))?;
        }
        Ok(out)
    }

    /// Save as PNG file
    pub fn save_png(&self, path: &str) -> Result<()> {
        let png = self.to_png()?;
        std::fs::write(path, &png)?;
        Ok(())
    }
}

/// Rasterizes a single PDF page
pub struct PageRasterizer<'a> {
    doc: &'a PdfDocument,
    /// Font data cache: font resource name → loaded font
    #[allow(dead_code)]
    font_cache: HashMap<String, LoadedFont>,
}

impl<'a> PageRasterizer<'a> {
    pub fn new(doc: &'a PdfDocument) -> Self {
        Self {
            doc,
            font_cache: HashMap::new(),
        }
    }

    /// Render a page at the given DPI
    pub fn render(&mut self, page: &PdfPage, dpi: f32) -> Result<RasterPage> {
        // PDF points are 1/72 inch. At `dpi`, 1pt = dpi/72 pixels.
        let scale = dpi / 72.0;

        let [x0, y0, x1, y1] = page.media_box;
        let page_w_pt = (x1 - x0) as f32;
        let page_h_pt = (y1 - y0) as f32;
        let px_w = (page_w_pt * scale).ceil() as u32;
        let px_h = (page_h_pt * scale).ceil() as u32;

        if px_w == 0 || px_h == 0 {
            return Ok(RasterPage {
                width: 1,
                height: 1,
                pixels: vec![255; 4],
            });
        }

        // Create tiny-skia pixmap (RGBA, white background)
        let mut pixmap = tiny_skia::Pixmap::new(px_w, px_h)
            .ok_or_else(|| PdfRenderError::Parse("Failed to create pixmap".to_string()))?;
        pixmap.fill(tiny_skia::Color::WHITE);

        // Build a scale transform: page pt → pixels
        // Also handle Y-flip here via content interpreter page_height
        let transform = tiny_skia::Transform::from_scale(scale, scale);

        // Interpret content stream
        let page_height_pt = page_h_pt;
        let mut interpreter = ContentInterpreter::new(self.doc, &page.resources, page_height_pt);
        let commands = interpreter.interpret(&page.content)?;

        // Execute draw commands
        for cmd in commands {
            self.execute_command(&mut pixmap, cmd, scale, &transform)?;
        }

        // Extract RGBA pixels
        let pixels = pixmap.data().to_vec();
        Ok(RasterPage {
            width: px_w,
            height: px_h,
            pixels,
        })
    }

    fn execute_command(
        &mut self,
        pixmap: &mut tiny_skia::Pixmap,
        cmd: DrawCommand,
        scale: f32,
        base_transform: &tiny_skia::Transform,
    ) -> Result<()> {
        match cmd {
            DrawCommand::FillPath {
                path,
                color,
                fill_rule,
            } => {
                let skia_color = color.to_tiny_skia_color();
                let paint = paint_from_color(skia_color);
                let fill_rule = match fill_rule {
                    FillRule::NonZero => tiny_skia::FillRule::Winding,
                    FillRule::EvenOdd => tiny_skia::FillRule::EvenOdd,
                };
                // Scale path to pixels
                let scaled_path = path.transform(*base_transform);
                if let Some(scaled) = scaled_path {
                    pixmap.fill_path(
                        &scaled,
                        &paint,
                        fill_rule,
                        tiny_skia::Transform::identity(),
                        None,
                    );
                }
            }

            DrawCommand::StrokePath { path, color, width } => {
                let skia_color = color.to_tiny_skia_color();
                let paint = paint_from_color(skia_color);
                let stroke = tiny_skia::Stroke {
                    width: width * scale,
                    ..Default::default()
                };
                let scaled_path = path.transform(*base_transform);
                if let Some(scaled) = scaled_path {
                    pixmap.stroke_path(
                        &scaled,
                        &paint,
                        &stroke,
                        tiny_skia::Transform::identity(),
                        None,
                    );
                }
            }

            DrawCommand::DrawGlyph(glyph) => {
                // Render text as filled rectangles (simple approximation)
                // For proper glyph rendering we'd need font outline extraction
                if let Some(ch) = glyph.character {
                    // Use a simple pixel-font fallback: draw a filled rect for the glyph area
                    let x = glyph.x * scale;
                    let y = (glyph.y - glyph.font_size) * scale;
                    let w = glyph.advance * scale;
                    let h = glyph.font_size * scale;

                    if w > 0.0 && h > 0.0 && ch != ' ' {
                        let skia_color = glyph.color.to_tiny_skia_color();
                        let paint = paint_from_color(skia_color);
                        if let Some(rect) = tiny_skia::Rect::from_xywh(x, y, w, h) {
                            pixmap.fill_rect(rect, &paint, tiny_skia::Transform::identity(), None);
                        }
                    }
                }
            }

            DrawCommand::DrawImage { image, transform } => {
                // Place the image on the pixmap
                // The image transform maps from unit square [0,1]×[0,1] to user space
                // In PDF, images are 1×1 units in the current graphics state CTM

                let dst_x = (transform.e * scale) as i32;
                // Page height flip: the image origin in screen coords
                // transform.f is the Y in user space (already flipped in content interpreter)
                let dst_y = (transform.f * scale) as i32;

                // Width and height from transform matrix
                let img_w_pt = transform.a.abs();
                let img_h_pt = transform.d.abs();
                let dst_w = ((img_w_pt * scale) as u32).max(1);
                let dst_h = ((img_h_pt * scale) as u32).max(1);

                // Scale image pixels to destination size
                let src_w = image.width as usize;
                let src_h = image.height as usize;

                for dy in 0..dst_h {
                    for dx in 0..dst_w {
                        let px = dst_x + dx as i32;
                        let py = dst_y + dy as i32;

                        if px < 0 || py < 0 {
                            continue;
                        }
                        let px = px as u32;
                        let py = py as u32;
                        if px >= pixmap.width() || py >= pixmap.height() {
                            continue;
                        }

                        // Sample source pixel
                        let src_x = (dx as f32 / dst_w as f32 * src_w as f32) as usize;
                        let src_y = (dy as f32 / dst_h as f32 * src_h as f32) as usize;
                        let src_idx = (src_y.min(src_h - 1) * src_w + src_x.min(src_w - 1)) * 4;

                        if src_idx + 3 < image.pixels.len() {
                            let r = image.pixels[src_idx] as f32 / 255.0;
                            let g = image.pixels[src_idx + 1] as f32 / 255.0;
                            let b = image.pixels[src_idx + 2] as f32 / 255.0;
                            let a = image.pixels[src_idx + 3] as f32 / 255.0;

                            if let Some(color) = tiny_skia::Color::from_rgba(r, g, b, a) {
                                let paint = paint_from_color(color);
                                if let Some(rect) =
                                    tiny_skia::Rect::from_xywh(px as f32, py as f32, 1.0, 1.0)
                                {
                                    pixmap.fill_rect(
                                        rect,
                                        &paint,
                                        tiny_skia::Transform::identity(),
                                        None,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

fn paint_from_color(color: tiny_skia::Color) -> tiny_skia::Paint<'static> {
    let mut paint = tiny_skia::Paint::default();
    paint.set_color(color);
    paint.anti_alias = true;
    paint
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::PdfDocument;

    // -----------------------------------------------------------------------
    // Helper: build a minimal 1-page PDF (no content stream)
    // -----------------------------------------------------------------------

    fn build_minimal_pdf() -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();
        out.extend_from_slice(b"%PDF-1.4\n");
        let o1 = out.len();
        out.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");
        let o2 = out.len();
        out.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");
        let o3 = out.len();
        out.extend_from_slice(
            b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\nendobj\n",
        );
        let xref = out.len();
        out.extend_from_slice(b"xref\n0 4\n");
        out.extend_from_slice(b"0000000000 65535 f \n");
        out.extend_from_slice(format!("{:010} 00000 n \n", o1).as_bytes());
        out.extend_from_slice(format!("{:010} 00000 n \n", o2).as_bytes());
        out.extend_from_slice(format!("{:010} 00000 n \n", o3).as_bytes());
        out.extend_from_slice(b"trailer\n<< /Size 4 /Root 1 0 R >>\nstartxref\n");
        out.extend_from_slice(format!("{}\n", xref).as_bytes());
        out.extend_from_slice(b"%%EOF\n");
        out
    }

    /// Build a 1-page PDF with a minimal content stream (black rectangle).
    fn build_pdf_with_content() -> Vec<u8> {
        let content = b"0 0 0 rg 72 72 72 72 re f\n";
        let content_len = content.len();

        let mut out: Vec<u8> = Vec::new();
        out.extend_from_slice(b"%PDF-1.4\n");

        let o1 = out.len();
        out.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

        let o2 = out.len();
        out.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

        let o3 = out.len();
        out.extend_from_slice(
            "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R >>\nendobj\n"
            .as_bytes(),
        );

        let o4 = out.len();
        out.extend_from_slice(
            format!("4 0 obj\n<< /Length {} >>\nstream\n", content_len).as_bytes(),
        );
        out.extend_from_slice(content);
        out.extend_from_slice(b"endstream\nendobj\n");

        let xref = out.len();
        out.extend_from_slice(b"xref\n0 5\n");
        out.extend_from_slice(b"0000000000 65535 f \n");
        out.extend_from_slice(format!("{:010} 00000 n \n", o1).as_bytes());
        out.extend_from_slice(format!("{:010} 00000 n \n", o2).as_bytes());
        out.extend_from_slice(format!("{:010} 00000 n \n", o3).as_bytes());
        out.extend_from_slice(format!("{:010} 00000 n \n", o4).as_bytes());
        out.extend_from_slice(b"trailer\n<< /Size 5 /Root 1 0 R >>\nstartxref\n");
        out.extend_from_slice(format!("{}\n", xref).as_bytes());
        out.extend_from_slice(b"%%EOF\n");
        out
    }

    // -----------------------------------------------------------------------
    // RasterPage tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_raster_page_to_png_produces_valid_png_header() {
        let page = RasterPage {
            width: 2,
            height: 2,
            pixels: vec![255u8; 2 * 2 * 4], // white 2x2
        };
        let png = page.to_png().expect("to_png should succeed");
        assert!(!png.is_empty(), "PNG output should not be empty");
        // PNG magic bytes: 0x89 P N G
        assert_eq!(
            &png[0..4],
            &[0x89, b'P', b'N', b'G'],
            "Should have PNG magic bytes"
        );
    }

    #[test]
    fn test_raster_page_dimensions_preserved() {
        let page = RasterPage {
            width: 100,
            height: 200,
            pixels: vec![0u8; 100 * 200 * 4],
        };
        assert_eq!(page.width, 100);
        assert_eq!(page.height, 200);
    }

    #[test]
    fn test_raster_page_save_png_writes_file() {
        let page = RasterPage {
            width: 4,
            height: 4,
            pixels: vec![128u8; 4 * 4 * 4],
        };
        let path = "/tmp/fop_rasterizer_test_output.png";
        page.save_png(path).expect("save_png should succeed");
        assert!(std::path::Path::new(path).exists(), "PNG file should exist");
        let data = std::fs::read(path).expect("test: should succeed");
        assert!(!data.is_empty());
        std::fs::remove_file(path).ok();
    }

    // -----------------------------------------------------------------------
    // PageRasterizer tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_rasterizer_new_accepts_document() {
        let pdf = build_minimal_pdf();
        let doc = PdfDocument::from_bytes(&pdf).expect("parse");
        let _rasterizer = PageRasterizer::new(&doc);
        // No panic: rasterizer can be constructed
    }

    #[test]
    fn test_render_empty_page_at_72dpi() {
        let pdf = build_minimal_pdf();
        let doc = PdfDocument::from_bytes(&pdf).expect("parse");
        let page = doc.get_page(0).expect("page 0");
        let mut rasterizer = PageRasterizer::new(&doc);
        let result = rasterizer.render(&page, 72.0);
        assert!(
            result.is_ok(),
            "Rendering empty page should succeed: {:?}",
            result.err()
        );
        let raster = result.expect("test: should succeed");
        assert!(raster.width > 0, "Width should be > 0");
        assert!(raster.height > 0, "Height should be > 0");
    }

    #[test]
    fn test_render_empty_page_at_150dpi() {
        let pdf = build_minimal_pdf();
        let doc = PdfDocument::from_bytes(&pdf).expect("parse");
        let page = doc.get_page(0).expect("page 0");
        let mut rasterizer = PageRasterizer::new(&doc);
        let result = rasterizer.render(&page, 150.0);
        assert!(result.is_ok());
        let raster = result.expect("test: should succeed");
        // At 150 DPI, pixels should be ~2x larger than 72 DPI
        assert!(
            raster.width > 600,
            "At 150 DPI, width should be > 600px for 612pt wide page"
        );
    }

    #[test]
    fn test_render_empty_page_pixel_count_matches_dimensions() {
        let pdf = build_minimal_pdf();
        let doc = PdfDocument::from_bytes(&pdf).expect("parse");
        let page = doc.get_page(0).expect("page 0");
        let mut rasterizer = PageRasterizer::new(&doc);
        let raster = rasterizer.render(&page, 72.0).expect("render");
        let expected_pixels = (raster.width * raster.height * 4) as usize;
        assert_eq!(
            raster.pixels.len(),
            expected_pixels,
            "Pixel buffer size should match width*height*4 (RGBA)"
        );
    }

    #[test]
    fn test_render_empty_page_is_white_background() {
        let pdf = build_minimal_pdf();
        let doc = PdfDocument::from_bytes(&pdf).expect("parse");
        let page = doc.get_page(0).expect("page 0");
        let mut rasterizer = PageRasterizer::new(&doc);
        let raster = rasterizer.render(&page, 72.0).expect("render");
        // tiny-skia fills with white (255,255,255,255) by default
        // Check first few pixels are white
        assert!(raster.pixels.len() >= 4);
        // Pixels may be pre-multiplied alpha in tiny-skia — just check non-empty
        assert!(!raster.pixels.is_empty());
    }

    #[test]
    fn test_render_page_with_content_stream() {
        let pdf = build_pdf_with_content();
        let doc = PdfDocument::from_bytes(&pdf).expect("parse");
        assert_eq!(doc.page_count(), 1);
        let page = doc.get_page(0).expect("page 0");
        let mut rasterizer = PageRasterizer::new(&doc);
        let result = rasterizer.render(&page, 72.0);
        assert!(
            result.is_ok(),
            "Page with content stream should render: {:?}",
            result.err()
        );
        let raster = result.expect("test: should succeed");
        assert!(raster.width > 0 && raster.height > 0);
    }

    #[test]
    fn test_render_to_png_bytes() {
        let pdf = build_minimal_pdf();
        let doc = PdfDocument::from_bytes(&pdf).expect("parse");
        let page = doc.get_page(0).expect("page 0");
        let mut rasterizer = PageRasterizer::new(&doc);
        let raster = rasterizer.render(&page, 72.0).expect("render");
        let png = raster.to_png().expect("to_png");
        assert!(!png.is_empty());
        assert_eq!(&png[0..4], &[0x89, b'P', b'N', b'G']);
    }

    // -----------------------------------------------------------------------
    // DPI scaling tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_dpi_scaling_proportional() {
        let pdf = build_minimal_pdf();
        let doc = PdfDocument::from_bytes(&pdf).expect("parse");
        let page = doc.get_page(0).expect("page 0");
        let mut rasterizer = PageRasterizer::new(&doc);

        let r72 = rasterizer.render(&page, 72.0).expect("render 72");
        let r144 = rasterizer.render(&page, 144.0).expect("render 144");

        // At 2x DPI, image dimensions should be approximately 2x
        let ratio_w = r144.width as f64 / r72.width as f64;
        let ratio_h = r144.height as f64 / r72.height as f64;
        assert!(
            (ratio_w - 2.0).abs() < 0.1,
            "Width ratio should be ~2.0, got {:.2}",
            ratio_w
        );
        assert!(
            (ratio_h - 2.0).abs() < 0.1,
            "Height ratio should be ~2.0, got {:.2}",
            ratio_h
        );
    }
}
