//! PDF Image XObject support
//!
//! Handles embedding images (PNG, JPEG) as XObjects in PDF documents.

use crate::image::{ImageFormat, ImageInfo};
use fop_types::Result;
use jpeg_decoder::Decoder;
use oxiarc_deflate::zlib_compress;
use std::io::Cursor;

/// PDF Image XObject representation
#[derive(Debug, Clone)]
pub struct ImageXObject {
    /// Image width in pixels
    pub width: u32,

    /// Image height in pixels
    pub height: u32,

    /// Color space (DeviceRGB, DeviceGray, etc.)
    pub color_space: String,

    /// Bits per component (typically 8)
    pub bits_per_component: u8,

    /// Filter name (DCTDecode for JPEG, FlateDecode for PNG)
    pub filter: String,

    /// Raw image data (JPEG data for DCTDecode, compressed for FlateDecode)
    pub data: Vec<u8>,
}

impl ImageXObject {
    /// Create an XObject from ImageInfo
    pub fn from_image_info(info: &ImageInfo) -> Result<Self> {
        match info.format {
            ImageFormat::JPEG => Self::from_jpeg(&info.data),
            ImageFormat::PNG => Self::from_png(&info.data),
            ImageFormat::Unknown => Err(fop_types::FopError::Generic(
                "Cannot create XObject from unknown image format".to_string(),
            )),
        }
    }

    /// Create an XObject from JPEG data
    ///
    /// JPEG images can be embedded directly in PDF using the DCTDecode filter,
    /// which allows the raw JPEG data to be used without decompression.
    pub fn from_jpeg(jpeg_data: &[u8]) -> Result<Self> {
        // Decode JPEG to extract metadata
        let mut decoder = Decoder::new(Cursor::new(jpeg_data));
        decoder.read_info().map_err(|e| {
            fop_types::FopError::Generic(format!("Failed to read JPEG info: {}", e))
        })?;

        let metadata = decoder.info().ok_or_else(|| {
            fop_types::FopError::Generic("JPEG decoder info not available".to_string())
        })?;

        let width = metadata.width;
        let height = metadata.height;

        // Determine color space from pixel format
        let color_space = match metadata.pixel_format {
            jpeg_decoder::PixelFormat::L8 => "DeviceGray",
            jpeg_decoder::PixelFormat::L16 => "DeviceGray",
            jpeg_decoder::PixelFormat::RGB24 => "DeviceRGB",
            jpeg_decoder::PixelFormat::CMYK32 => "DeviceCMYK",
        };

        Ok(Self {
            width: width as u32,
            height: height as u32,
            color_space: color_space.to_string(),
            bits_per_component: 8,
            filter: "DCTDecode".to_string(),
            data: jpeg_data.to_vec(),
        })
    }

    /// Create an XObject from PNG data
    ///
    /// PNG images require decompression and recompression with FlateDecode.
    pub fn from_png(png_data: &[u8]) -> Result<Self> {
        use std::io::Cursor;
        // Decode PNG using png crate
        let decoder = png::Decoder::new(Cursor::new(png_data));
        let mut reader = decoder
            .read_info()
            .map_err(|e| fop_types::FopError::Generic(format!("PNG decode error: {}", e)))?;

        let info = reader.info();
        let width = info.width;
        let height = info.height;
        let color_type = info.color_type;
        let bit_depth = info.bit_depth;

        // Validate bit depth
        if bit_depth != png::BitDepth::Eight {
            return Err(fop_types::FopError::Generic(
                "Only 8-bit PNG images are supported".to_string(),
            ));
        }

        // Determine color space and expected components
        let (color_space, components) = match color_type {
            png::ColorType::Rgb => ("DeviceRGB", 3),
            png::ColorType::Rgba => ("DeviceRGB", 3), // We'll strip alpha
            png::ColorType::Grayscale => ("DeviceGray", 1),
            png::ColorType::GrayscaleAlpha => ("DeviceGray", 1), // We'll strip alpha
            png::ColorType::Indexed => {
                return Err(fop_types::FopError::Generic(
                    "Indexed PNG images are not supported".to_string(),
                ))
            }
        };

        // Allocate buffer for decoded image
        let buf_size = reader.output_buffer_size().ok_or_else(|| {
            fop_types::FopError::Generic("PNG: could not determine output buffer size".to_string())
        })?;
        let mut buf = vec![0; buf_size];
        let output_info = reader
            .next_frame(&mut buf)
            .map_err(|e| fop_types::FopError::Generic(format!("PNG frame error: {}", e)))?;

        // Get actual decoded data
        let decoded_data = &buf[..output_info.buffer_size()];

        // Strip alpha channel if present
        let rgb_data = if color_type == png::ColorType::Rgba {
            Self::strip_alpha_rgba(decoded_data, width, height)
        } else if color_type == png::ColorType::GrayscaleAlpha {
            Self::strip_alpha_grayscale(decoded_data, width, height)
        } else {
            decoded_data.to_vec()
        };

        // Validate data size
        let expected_size = (width * height) as usize * components;
        if rgb_data.len() != expected_size {
            return Err(fop_types::FopError::Generic(format!(
                "Unexpected PNG data size: got {}, expected {}",
                rgb_data.len(),
                expected_size
            )));
        }

        // Compress data using FlateDecode (zlib)
        let compressed_data = Self::compress_data(&rgb_data)?;

        Ok(Self {
            width,
            height,
            color_space: color_space.to_string(),
            bits_per_component: 8,
            filter: "FlateDecode".to_string(),
            data: compressed_data,
        })
    }

    /// Strip alpha channel from RGBA data
    fn strip_alpha_rgba(rgba_data: &[u8], width: u32, height: u32) -> Vec<u8> {
        let pixel_count = (width * height) as usize;
        let mut rgb_data = Vec::with_capacity(pixel_count * 3);

        for i in 0..pixel_count {
            let offset = i * 4;
            rgb_data.push(rgba_data[offset]); // R
            rgb_data.push(rgba_data[offset + 1]); // G
            rgb_data.push(rgba_data[offset + 2]); // B
                                                  // Skip alpha channel
        }

        rgb_data
    }

    /// Strip alpha channel from grayscale+alpha data
    fn strip_alpha_grayscale(ga_data: &[u8], width: u32, height: u32) -> Vec<u8> {
        let pixel_count = (width * height) as usize;
        let mut gray_data = Vec::with_capacity(pixel_count);

        for i in 0..pixel_count {
            let offset = i * 2;
            gray_data.push(ga_data[offset]); // Gray
                                             // Skip alpha channel
        }

        gray_data
    }

    /// Compress data using zlib (FlateDecode)
    fn compress_data(data: &[u8]) -> Result<Vec<u8>> {
        zlib_compress(data, 6)
            .map_err(|e| fop_types::FopError::Generic(format!("Compression error: {}", e)))
    }

    /// Generate the PDF XObject dictionary and stream
    pub fn to_pdf_stream(&self, object_id: u32) -> String {
        let mut result = String::new();

        // Object header
        result.push_str(&format!("{} 0 obj\n", object_id));
        result.push_str("<<\n");
        result.push_str("/Type /XObject\n");
        result.push_str("/Subtype /Image\n");
        result.push_str(&format!("/Width {}\n", self.width));
        result.push_str(&format!("/Height {}\n", self.height));
        result.push_str(&format!("/ColorSpace /{}\n", self.color_space));
        result.push_str(&format!("/BitsPerComponent {}\n", self.bits_per_component));
        result.push_str(&format!("/Filter /{}\n", self.filter));
        result.push_str(&format!("/Length {}\n", self.data.len()));
        result.push_str(">>\n");
        result.push_str("stream\n");

        // Binary data will be added separately
        result
    }

    /// Get the stream data
    pub fn stream_data(&self) -> &[u8] {
        &self.data
    }

    /// Get the stream end marker
    pub fn stream_end() -> &'static str {
        "\nendstream\nendobj\n"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Minimal JPEG header (SOI + SOF0 + EOI)
    fn minimal_jpeg() -> Vec<u8> {
        vec![
            0xFF, 0xD8, // SOI (Start of Image)
            0xFF, 0xE0, // APP0
            0x00, 0x10, // Length
            0x4A, 0x46, 0x49, 0x46, 0x00, // "JFIF\0"
            0x01, 0x01, // Version 1.1
            0x00, // Density units
            0x00, 0x01, 0x00, 0x01, // X and Y density
            0x00, 0x00, // Thumbnail size
            0xFF, 0xC0, // SOF0 (Start of Frame, baseline DCT)
            0x00, 0x11, // Length
            0x08, // Precision (8 bits)
            0x00, 0x64, // Height (100)
            0x00, 0x64, // Width (100)
            0x03, // Number of components (RGB)
            0x01, 0x22, 0x00, // Component 1 (Y)
            0x02, 0x11, 0x01, // Component 2 (Cb)
            0x03, 0x11, 0x01, // Component 3 (Cr)
            0xFF, 0xD9, // EOI (End of Image)
        ]
    }

    #[test]
    fn test_jpeg_xobject_creation() {
        let jpeg_data = minimal_jpeg();
        let xobject = ImageXObject::from_jpeg(&jpeg_data).expect("test: should succeed");

        assert_eq!(xobject.width, 100);
        assert_eq!(xobject.height, 100);
        assert_eq!(xobject.color_space, "DeviceRGB");
        assert_eq!(xobject.bits_per_component, 8);
        assert_eq!(xobject.filter, "DCTDecode");
        assert_eq!(xobject.data.len(), jpeg_data.len());
    }

    #[test]
    fn test_jpeg_xobject_pdf_stream() {
        let jpeg_data = minimal_jpeg();
        let xobject = ImageXObject::from_jpeg(&jpeg_data).expect("test: should succeed");
        let pdf_stream = xobject.to_pdf_stream(5);

        assert!(pdf_stream.contains("5 0 obj"));
        assert!(pdf_stream.contains("/Type /XObject"));
        assert!(pdf_stream.contains("/Subtype /Image"));
        assert!(pdf_stream.contains("/Width 100"));
        assert!(pdf_stream.contains("/Height 100"));
        assert!(pdf_stream.contains("/ColorSpace /DeviceRGB"));
        assert!(pdf_stream.contains("/BitsPerComponent 8"));
        assert!(pdf_stream.contains("/Filter /DCTDecode"));
        assert!(pdf_stream.contains(&format!("/Length {}", jpeg_data.len())));
        assert!(pdf_stream.contains("stream"));
    }

    #[test]
    fn test_jpeg_xobject_stream_data() {
        let jpeg_data = minimal_jpeg();
        let xobject = ImageXObject::from_jpeg(&jpeg_data).expect("test: should succeed");
        let stream_data = xobject.stream_data();

        assert_eq!(stream_data, &jpeg_data[..]);
    }

    #[test]
    fn test_jpeg_xobject_stream_end() {
        let end = ImageXObject::stream_end();
        assert_eq!(end, "\nendstream\nendobj\n");
    }

    /// Create a minimal valid PNG image (1x1 red pixel)
    fn create_test_png() -> Vec<u8> {
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

        png_data
    }

    #[test]
    fn test_png_xobject_creation() {
        let png_data = create_test_png();
        let xobject = ImageXObject::from_png(&png_data).expect("test: should succeed");

        assert_eq!(xobject.width, 1);
        assert_eq!(xobject.height, 1);
        assert_eq!(xobject.color_space, "DeviceRGB");
        assert_eq!(xobject.bits_per_component, 8);
        assert_eq!(xobject.filter, "FlateDecode");
        assert!(!xobject.data.is_empty());
    }

    #[test]
    fn test_png_xobject_pdf_stream() {
        let png_data = create_test_png();
        let xobject = ImageXObject::from_png(&png_data).expect("test: should succeed");
        let pdf_stream = xobject.to_pdf_stream(6);

        assert!(pdf_stream.contains("6 0 obj"));
        assert!(pdf_stream.contains("/Type /XObject"));
        assert!(pdf_stream.contains("/Subtype /Image"));
        assert!(pdf_stream.contains("/Width 1"));
        assert!(pdf_stream.contains("/Height 1"));
        assert!(pdf_stream.contains("/ColorSpace /DeviceRGB"));
        assert!(pdf_stream.contains("/BitsPerComponent 8"));
        assert!(pdf_stream.contains("/Filter /FlateDecode"));
        assert!(pdf_stream.contains("stream"));
    }

    #[test]
    fn test_strip_alpha_rgba() {
        let rgba = vec![
            255, 0, 0, 255, // Red with full alpha
            0, 255, 0, 128, // Green with half alpha
        ];

        let rgb = ImageXObject::strip_alpha_rgba(&rgba, 2, 1);

        assert_eq!(rgb, vec![255, 0, 0, 0, 255, 0]);
    }

    #[test]
    fn test_strip_alpha_grayscale() {
        let ga = vec![
            128, 255, // Gray 128 with full alpha
            64, 128, // Gray 64 with half alpha
        ];

        let gray = ImageXObject::strip_alpha_grayscale(&ga, 2, 1);

        assert_eq!(gray, vec![128, 64]);
    }

    #[test]
    fn test_from_image_info_jpeg() {
        let jpeg_data = minimal_jpeg();
        let image_info = ImageInfo {
            format: ImageFormat::JPEG,
            width_px: 100,
            height_px: 100,
            bits_per_component: 8,
            color_space: "DeviceRGB".to_string(),
            data: jpeg_data,
        };

        let xobject = ImageXObject::from_image_info(&image_info).expect("test: should succeed");
        assert_eq!(xobject.filter, "DCTDecode");
    }

    #[test]
    fn test_from_image_info_unknown() {
        let image_info = ImageInfo {
            format: ImageFormat::Unknown,
            width_px: 100,
            height_px: 100,
            bits_per_component: 8,
            color_space: "DeviceRGB".to_string(),
            data: vec![],
        };

        let result = ImageXObject::from_image_info(&image_info);
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    // ── JPEG magic bytes and header ──────────────────────────────────────────

    /// Minimal valid JPEG with real scan data (enough for jpeg_decoder)
    fn minimal_jpeg() -> Vec<u8> {
        vec![
            0xFF, 0xD8, // SOI
            0xFF, 0xE0, // APP0
            0x00, 0x10, // Length 16
            0x4A, 0x46, 0x49, 0x46, 0x00, // "JFIF\0"
            0x01, 0x01, // Version 1.1
            0x00, // Density units
            0x00, 0x01, 0x00, 0x01, // X and Y density
            0x00, 0x00, // Thumbnail size
            0xFF, 0xC0, // SOF0
            0x00, 0x11, // Length 17
            0x08, // Precision 8
            0x00, 0x64, // Height 100
            0x00, 0x64, // Width 100
            0x03, // 3 components
            0x01, 0x22, 0x00, 0x02, 0x11, 0x01, 0x03, 0x11, 0x01, 0xFF, 0xD9, // EOI
        ]
    }

    fn create_test_png_1x1() -> Vec<u8> {
        let mut png_data = Vec::new();
        let mut encoder = png::Encoder::new(&mut png_data, 1, 1);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("test: should succeed");
        writer
            .write_image_data(&[255u8, 0, 0])
            .expect("test: should succeed");
        drop(writer);
        png_data
    }

    fn create_test_png_gray() -> Vec<u8> {
        let mut png_data = Vec::new();
        let mut encoder = png::Encoder::new(&mut png_data, 2, 2);
        encoder.set_color(png::ColorType::Grayscale);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("test: should succeed");
        writer
            .write_image_data(&[100u8, 150, 200, 250])
            .expect("test: should succeed");
        drop(writer);
        png_data
    }

    fn create_test_png_rgba() -> Vec<u8> {
        let mut png_data = Vec::new();
        let mut encoder = png::Encoder::new(&mut png_data, 1, 1);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().expect("test: should succeed");
        writer
            .write_image_data(&[255u8, 128, 0, 200])
            .expect("test: should succeed");
        drop(writer);
        png_data
    }

    // ── JPEG magic bytes ─────────────────────────────────────────────────────

    #[test]
    fn test_jpeg_soi_marker() {
        let data = minimal_jpeg();
        // JPEG files start with SOI marker 0xFF 0xD8
        assert_eq!(data[0], 0xFF);
        assert_eq!(data[1], 0xD8);
    }

    #[test]
    fn test_jpeg_xobject_stores_original_data() {
        let jpeg_data = minimal_jpeg();
        let xobj = ImageXObject::from_jpeg(&jpeg_data).expect("test: should succeed");
        // DCT decode stores the raw JPEG bytes
        assert_eq!(xobj.data, jpeg_data);
        assert_eq!(xobj.stream_data(), &jpeg_data[..]);
    }

    #[test]
    fn test_jpeg_xobject_filter_is_dctdecode() {
        let jpeg_data = minimal_jpeg();
        let xobj = ImageXObject::from_jpeg(&jpeg_data).expect("test: should succeed");
        assert_eq!(xobj.filter, "DCTDecode");
    }

    #[test]
    fn test_jpeg_color_space_is_device_rgb() {
        let jpeg_data = minimal_jpeg();
        let xobj = ImageXObject::from_jpeg(&jpeg_data).expect("test: should succeed");
        assert_eq!(xobj.color_space, "DeviceRGB");
    }

    #[test]
    fn test_jpeg_bits_per_component_is_8() {
        let jpeg_data = minimal_jpeg();
        let xobj = ImageXObject::from_jpeg(&jpeg_data).expect("test: should succeed");
        assert_eq!(xobj.bits_per_component, 8);
    }

    // ── Image dictionary entries ─────────────────────────────────────────────

    #[test]
    fn test_pdf_stream_type_xobject() {
        let jpeg_data = minimal_jpeg();
        let xobj = ImageXObject::from_jpeg(&jpeg_data).expect("test: should succeed");
        let stream = xobj.to_pdf_stream(10);
        assert!(stream.contains("/Type /XObject"), "/Type /XObject missing");
    }

    #[test]
    fn test_pdf_stream_subtype_image() {
        let jpeg_data = minimal_jpeg();
        let xobj = ImageXObject::from_jpeg(&jpeg_data).expect("test: should succeed");
        let stream = xobj.to_pdf_stream(10);
        assert!(
            stream.contains("/Subtype /Image"),
            "/Subtype /Image missing"
        );
    }

    #[test]
    fn test_pdf_stream_width_entry() {
        let png_data = create_test_png_1x1();
        let xobj = ImageXObject::from_png(&png_data).expect("test: should succeed");
        let stream = xobj.to_pdf_stream(7);
        assert!(stream.contains("/Width 1"), "/Width entry wrong");
    }

    #[test]
    fn test_pdf_stream_height_entry() {
        let png_data = create_test_png_1x1();
        let xobj = ImageXObject::from_png(&png_data).expect("test: should succeed");
        let stream = xobj.to_pdf_stream(7);
        assert!(stream.contains("/Height 1"), "/Height entry wrong");
    }

    #[test]
    fn test_pdf_stream_colorspace_device_rgb() {
        let png_data = create_test_png_1x1();
        let xobj = ImageXObject::from_png(&png_data).expect("test: should succeed");
        let stream = xobj.to_pdf_stream(7);
        assert!(
            stream.contains("/ColorSpace /DeviceRGB"),
            "ColorSpace entry wrong: {}",
            stream
        );
    }

    #[test]
    fn test_pdf_stream_bits_per_component_8() {
        let png_data = create_test_png_1x1();
        let xobj = ImageXObject::from_png(&png_data).expect("test: should succeed");
        let stream = xobj.to_pdf_stream(7);
        assert!(
            stream.contains("/BitsPerComponent 8"),
            "BitsPerComponent missing"
        );
    }

    #[test]
    fn test_pdf_stream_has_stream_keyword() {
        let png_data = create_test_png_1x1();
        let xobj = ImageXObject::from_png(&png_data).expect("test: should succeed");
        let stream = xobj.to_pdf_stream(7);
        assert!(stream.contains("stream\n"), "stream keyword missing");
    }

    #[test]
    fn test_pdf_stream_length_matches_data() {
        let jpeg_data = minimal_jpeg();
        let xobj = ImageXObject::from_jpeg(&jpeg_data).expect("test: should succeed");
        let stream = xobj.to_pdf_stream(5);
        let expected = format!("/Length {}", jpeg_data.len());
        assert!(stream.contains(&expected), "Length entry wrong: {}", stream);
    }

    #[test]
    fn test_stream_end_marker() {
        let end = ImageXObject::stream_end();
        assert!(end.contains("endstream"), "endstream missing");
        assert!(end.contains("endobj"), "endobj missing");
    }

    // ── PNG-specific tests ───────────────────────────────────────────────────

    #[test]
    fn test_png_xobject_filter_is_flatedecode() {
        let png_data = create_test_png_1x1();
        let xobj = ImageXObject::from_png(&png_data).expect("test: should succeed");
        assert_eq!(xobj.filter, "FlateDecode");
    }

    #[test]
    fn test_png_grayscale_color_space() {
        let png_data = create_test_png_gray();
        let xobj = ImageXObject::from_png(&png_data).expect("test: should succeed");
        assert_eq!(xobj.color_space, "DeviceGray");
    }

    #[test]
    fn test_png_rgba_strips_alpha_to_rgb() {
        let png_data = create_test_png_rgba();
        let xobj = ImageXObject::from_png(&png_data).expect("test: should succeed");
        // Alpha stripped → DeviceRGB
        assert_eq!(xobj.color_space, "DeviceRGB");
    }

    #[test]
    fn test_png_data_is_compressed() {
        let png_data = create_test_png_1x1();
        let xobj = ImageXObject::from_png(&png_data).expect("test: should succeed");
        // Compressed data should be non-empty
        assert!(!xobj.data.is_empty());
        // The raw pixel data (3 bytes for 1x1 RGB) gets compressed; the result
        // is not equal to the raw pixel bytes.
        assert_ne!(xobj.data, vec![255u8, 0, 0]);
    }

    #[test]
    fn test_png_2x2_dimensions() {
        let png_data = create_test_png_gray();
        let xobj = ImageXObject::from_png(&png_data).expect("test: should succeed");
        assert_eq!(xobj.width, 2);
        assert_eq!(xobj.height, 2);
    }

    // ── Strip alpha helpers ──────────────────────────────────────────────────

    #[test]
    fn test_strip_alpha_rgba_pixel_order() {
        // 2 pixels: RGBA RGBA → RGB RGB
        let rgba = vec![10u8, 20, 30, 255, 40, 50, 60, 128];
        let rgb = ImageXObject::strip_alpha_rgba(&rgba, 2, 1);
        assert_eq!(rgb, vec![10, 20, 30, 40, 50, 60]);
    }

    #[test]
    fn test_strip_alpha_grayscale_alpha_removed() {
        // 3 pixels: GA GA GA → G G G
        let ga = vec![50u8, 255, 100, 128, 200, 64];
        let gray = ImageXObject::strip_alpha_grayscale(&ga, 3, 1);
        assert_eq!(gray, vec![50, 100, 200]);
    }

    // ── Object ID in PDF stream ──────────────────────────────────────────────

    #[test]
    fn test_pdf_stream_uses_provided_object_id() {
        let jpeg_data = minimal_jpeg();
        let xobj = ImageXObject::from_jpeg(&jpeg_data).expect("test: should succeed");

        for id in [1u32, 42, 100, 999] {
            let stream = xobj.to_pdf_stream(id);
            assert!(
                stream.starts_with(&format!("{} 0 obj\n", id)),
                "object id {} not at start: {}",
                id,
                &stream[..20]
            );
        }
    }

    // ── from_image_info dispatch ─────────────────────────────────────────────

    #[test]
    fn test_from_image_info_png_dispatch() {
        let png_data = create_test_png_1x1();
        let info = ImageInfo {
            format: crate::image::ImageFormat::PNG,
            width_px: 1,
            height_px: 1,
            bits_per_component: 8,
            color_space: "DeviceRGB".to_string(),
            data: png_data,
        };
        let xobj = ImageXObject::from_image_info(&info).expect("test: should succeed");
        assert_eq!(xobj.filter, "FlateDecode");
    }
}
