//! Image support for PDF rendering
//!
//! Handles image insertion (PNG, JPEG) with dimension and format detection.

use fop_types::{Length, Result};

/// Image format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// PNG format
    PNG,

    /// JPEG format
    JPEG,

    /// Unknown format
    Unknown,
}

/// Image information
#[derive(Debug, Clone)]
pub struct ImageInfo {
    /// Image format
    pub format: ImageFormat,

    /// Image width in pixels
    pub width_px: u32,

    /// Image height in pixels
    pub height_px: u32,

    /// Bits per component (typically 8)
    pub bits_per_component: u8,

    /// Color space (DeviceRGB, DeviceGray, etc.)
    pub color_space: String,

    /// Raw image data
    pub data: Vec<u8>,
}

impl ImageInfo {
    /// Create image info from raw data
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        // Detect format from magic bytes
        let format = Self::detect_format(data)?;

        match format {
            ImageFormat::PNG => Self::parse_png(data),
            ImageFormat::JPEG => Self::parse_jpeg(data),
            ImageFormat::Unknown => Err(fop_types::FopError::Generic(
                "Unknown image format".to_string(),
            )),
        }
    }

    /// Detect image format from magic bytes
    fn detect_format(data: &[u8]) -> Result<ImageFormat> {
        if data.len() < 8 {
            return Ok(ImageFormat::Unknown);
        }

        // PNG: 89 50 4E 47 0D 0A 1A 0A
        if data[0..8] == [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A] {
            return Ok(ImageFormat::PNG);
        }

        // JPEG: FF D8 FF
        if data.len() >= 3 && data[0..3] == [0xFF, 0xD8, 0xFF] {
            return Ok(ImageFormat::JPEG);
        }

        Ok(ImageFormat::Unknown)
    }

    /// Parse PNG image
    fn parse_png(data: &[u8]) -> Result<Self> {
        use std::io::Cursor;
        // Decode PNG using png crate to extract metadata
        let decoder = png::Decoder::new(Cursor::new(data));
        let reader = decoder
            .read_info()
            .map_err(|e| fop_types::FopError::Generic(format!("PNG decode error: {}", e)))?;

        let info = reader.info();
        let width_px = info.width;
        let height_px = info.height;
        let color_type = info.color_type;

        // Determine color space
        let color_space = match color_type {
            png::ColorType::Rgb | png::ColorType::Rgba => "DeviceRGB",
            png::ColorType::Grayscale | png::ColorType::GrayscaleAlpha => "DeviceGray",
            png::ColorType::Indexed => {
                return Err(fop_types::FopError::Generic(
                    "Indexed PNG images are not supported".to_string(),
                ))
            }
        };

        Ok(Self {
            format: ImageFormat::PNG,
            width_px,
            height_px,
            bits_per_component: 8,
            color_space: color_space.to_string(),
            data: data.to_vec(),
        })
    }

    /// Parse JPEG image using jpeg-decoder
    fn parse_jpeg(data: &[u8]) -> Result<Self> {
        use jpeg_decoder::Decoder;
        use std::io::Cursor;

        let mut decoder = Decoder::new(Cursor::new(data));
        decoder.read_info().map_err(|e| {
            fop_types::FopError::Generic(format!("Failed to read JPEG info: {}", e))
        })?;

        let metadata = decoder.info().ok_or_else(|| {
            fop_types::FopError::Generic("JPEG decoder info not available".to_string())
        })?;

        let color_space = match metadata.pixel_format {
            jpeg_decoder::PixelFormat::L8 => "DeviceGray",
            jpeg_decoder::PixelFormat::L16 => "DeviceGray",
            jpeg_decoder::PixelFormat::RGB24 => "DeviceRGB",
            jpeg_decoder::PixelFormat::CMYK32 => "DeviceCMYK",
        };

        Ok(Self {
            format: ImageFormat::JPEG,
            width_px: metadata.width as u32,
            height_px: metadata.height as u32,
            bits_per_component: 8,
            color_space: color_space.to_string(),
            data: data.to_vec(),
        })
    }

    /// Calculate display dimensions maintaining aspect ratio
    pub fn calculate_display_size(
        &self,
        max_width: Length,
        max_height: Length,
    ) -> (Length, Length) {
        let aspect_ratio = self.width_px as f64 / self.height_px as f64;

        // Try fitting by width
        let width_fit = max_width;
        let height_for_width = Length::from_pt(width_fit.to_pt() / aspect_ratio);

        if height_for_width <= max_height {
            return (width_fit, height_for_width);
        }

        // Fit by height
        let height_fit = max_height;
        let width_for_height = Length::from_pt(height_fit.to_pt() * aspect_ratio);

        (width_for_height, height_fit)
    }

    /// Get DPI assuming standard 72 DPI display
    pub fn dpi_at_size(&self, display_width: Length) -> f64 {
        72.0 * self.width_px as f64 / display_width.to_pt()
    }
}

/// Image placement in PDF
#[derive(Debug, Clone)]
pub struct ImagePlacement {
    /// X position
    pub x: Length,

    /// Y position
    pub y: Length,

    /// Display width
    pub width: Length,

    /// Display height
    pub height: Length,

    /// Image data
    pub image: ImageInfo,
}

impl ImagePlacement {
    /// Create a new image placement
    pub fn new(x: Length, y: Length, width: Length, height: Length, image: ImageInfo) -> Self {
        Self {
            x,
            y,
            width,
            height,
            image,
        }
    }

    /// Create with automatic sizing
    pub fn auto_size(
        x: Length,
        y: Length,
        max_width: Length,
        max_height: Length,
        image: ImageInfo,
    ) -> Self {
        let (width, height) = image.calculate_display_size(max_width, max_height);
        Self {
            x,
            y,
            width,
            height,
            image,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_png() {
        let png_header = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        let format = ImageInfo::detect_format(&png_header).expect("test: should succeed");
        assert_eq!(format, ImageFormat::PNG);
    }

    #[test]
    fn test_detect_jpeg() {
        let jpeg_header = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46]; // JFIF header
        let format = ImageInfo::detect_format(&jpeg_header).expect("test: should succeed");
        assert_eq!(format, ImageFormat::JPEG);
    }

    #[test]
    fn test_detect_unknown() {
        let unknown = vec![0x00, 0x00, 0x00, 0x00];
        let format = ImageInfo::detect_format(&unknown).expect("test: should succeed");
        assert_eq!(format, ImageFormat::Unknown);
    }

    #[test]
    fn test_aspect_ratio_fit_by_width() {
        let image = ImageInfo {
            format: ImageFormat::PNG,
            width_px: 200,
            height_px: 100, // 2:1 aspect ratio
            bits_per_component: 8,
            color_space: "DeviceRGB".to_string(),
            data: Vec::new(),
        };

        let (w, h) = image.calculate_display_size(Length::from_pt(100.0), Length::from_pt(100.0));

        // Should fit by width: 100pt wide, 50pt tall
        assert_eq!(w, Length::from_pt(100.0));
        assert_eq!(h, Length::from_pt(50.0));
    }

    #[test]
    fn test_aspect_ratio_fit_by_height() {
        let image = ImageInfo {
            format: ImageFormat::PNG,
            width_px: 100,
            height_px: 200, // 1:2 aspect ratio
            bits_per_component: 8,
            color_space: "DeviceRGB".to_string(),
            data: Vec::new(),
        };

        let (w, h) = image.calculate_display_size(Length::from_pt(100.0), Length::from_pt(100.0));

        // Should fit by height: 50pt wide, 100pt tall
        assert_eq!(w, Length::from_pt(50.0));
        assert_eq!(h, Length::from_pt(100.0));
    }

    #[test]
    fn test_dpi_calculation() {
        let image = ImageInfo {
            format: ImageFormat::PNG,
            width_px: 720,
            height_px: 720,
            bits_per_component: 8,
            color_space: "DeviceRGB".to_string(),
            data: Vec::new(),
        };

        // Display at 72pt -> 720px / 72pt * 72dpi = 720 DPI
        let dpi = image.dpi_at_size(Length::from_pt(72.0));
        assert_eq!(dpi, 720.0);
    }

    #[test]
    fn test_image_placement() {
        let image = ImageInfo {
            format: ImageFormat::JPEG,
            width_px: 100,
            height_px: 100,
            bits_per_component: 8,
            color_space: "DeviceRGB".to_string(),
            data: Vec::new(),
        };

        let placement = ImagePlacement::new(
            Length::from_pt(10.0),
            Length::from_pt(20.0),
            Length::from_pt(50.0),
            Length::from_pt(50.0),
            image,
        );

        assert_eq!(placement.x, Length::from_pt(10.0));
        assert_eq!(placement.width, Length::from_pt(50.0));
    }

    #[test]
    fn test_auto_size_placement() {
        let image = ImageInfo {
            format: ImageFormat::PNG,
            width_px: 200,
            height_px: 100,
            bits_per_component: 8,
            color_space: "DeviceRGB".to_string(),
            data: Vec::new(),
        };

        let placement = ImagePlacement::auto_size(
            Length::ZERO,
            Length::ZERO,
            Length::from_pt(100.0),
            Length::from_pt(100.0),
            image,
        );

        assert_eq!(placement.width, Length::from_pt(100.0));
        assert_eq!(placement.height, Length::from_pt(50.0));
    }
}
