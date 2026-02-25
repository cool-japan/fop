//! Image XObject decoding for PDF rendering

use crate::error::{PdfRenderError, Result};
use crate::parser::PdfDictionary;

/// A decoded image ready for rasterization
#[derive(Debug, Clone)]
pub struct DecodedImage {
    pub width: u32,
    pub height: u32,
    /// RGBA pixel data (row-major, top-to-bottom)
    pub pixels: Vec<u8>,
}

impl DecodedImage {
    /// Decode an image XObject from its dictionary and raw (possibly still compressed) data
    pub fn decode(dict: &PdfDictionary, data: &[u8]) -> Result<Self> {
        let width = dict.get_integer("Width").unwrap_or(1) as u32;
        let height = dict.get_integer("Height").unwrap_or(1) as u32;
        let bits = dict.get_integer("BitsPerComponent").unwrap_or(8) as u32;
        let cs = dict.get_name("ColorSpace").unwrap_or("DeviceRGB");
        let filter = dict.get_name("Filter");

        // Determine image format from filter or color space
        let pixels = match filter {
            Some("DCTDecode") | Some("DCT") => decode_jpeg(data, width, height)?,
            Some("FlateDecode") | Some("Fl") => {
                // Already decoded (stream was already decoded in parser)
                decode_raw(data, width, height, bits, cs)
            }
            _ => {
                // Try JPEG magic bytes
                if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
                    decode_jpeg(data, width, height)?
                } else if data.starts_with(b"\x89PNG") {
                    decode_png(data)?
                } else {
                    decode_raw(data, width, height, bits, cs)
                }
            }
        };

        Ok(DecodedImage {
            width,
            height,
            pixels,
        })
    }
}

fn decode_jpeg(data: &[u8], _expected_w: u32, _expected_h: u32) -> Result<Vec<u8>> {
    let mut decoder = jpeg_decoder::Decoder::new(data);
    let pixels = decoder
        .decode()
        .map_err(|e| PdfRenderError::Image(e.to_string()))?;
    let info = decoder
        .info()
        .ok_or_else(|| PdfRenderError::Image("No JPEG info".to_string()))?;

    // Convert to RGBA
    let rgba = match info.pixel_format {
        jpeg_decoder::PixelFormat::RGB24 => pixels
            .chunks(3)
            .flat_map(|c| [c[0], c[1], c[2], 255])
            .collect(),
        jpeg_decoder::PixelFormat::L8 => pixels.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        jpeg_decoder::PixelFormat::CMYK32 => {
            // Simple CMYK → RGB
            pixels
                .chunks(4)
                .flat_map(|c| {
                    let r = (c[0] as f32 * c[3] as f32 / 255.0) as u8;
                    let g = (c[1] as f32 * c[3] as f32 / 255.0) as u8;
                    let b = (c[2] as f32 * c[3] as f32 / 255.0) as u8;
                    [r, g, b, 255u8]
                })
                .collect()
        }
        _ => {
            // Fallback: treat as gray
            pixels.iter().flat_map(|&g| [g, g, g, 255u8]).collect()
        }
    };
    Ok(rgba)
}

fn decode_png(data: &[u8]) -> Result<Vec<u8>> {
    let cursor = std::io::Cursor::new(data);
    let decoder = png::Decoder::new(cursor);
    let mut reader = decoder
        .read_info()
        .map_err(|e: png::DecodingError| PdfRenderError::Image(e.to_string()))?;
    let buf_size = reader.output_buffer_size().ok_or_else(|| {
        PdfRenderError::Image("PNG: could not determine output buffer size".to_string())
    })?;
    let mut buf = vec![0u8; buf_size];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e: png::DecodingError| PdfRenderError::Image(e.to_string()))?;
    let frame = &buf[..info.buffer_size()];
    let w = info.width;
    let h = info.height;

    let rgba = match info.color_type {
        png::ColorType::Rgb => frame
            .chunks(3)
            .flat_map(|c| [c[0], c[1], c[2], 255])
            .collect(),
        png::ColorType::Rgba => frame.to_vec(),
        png::ColorType::Grayscale => frame.iter().flat_map(|&g| [g, g, g, 255u8]).collect(),
        png::ColorType::GrayscaleAlpha => frame
            .chunks(2)
            .flat_map(|c| [c[0], c[0], c[0], c[1]])
            .collect(),
        png::ColorType::Indexed => {
            // Indexed color - return as is (simplified)
            frame.iter().flat_map(|&g| [g, g, g, 255u8]).collect()
        }
    };
    let _ = (w, h);
    Ok(rgba)
}

fn decode_raw(data: &[u8], width: u32, height: u32, bits: u32, color_space: &str) -> Vec<u8> {
    let channels = match color_space {
        "DeviceGray" | "CalGray" => 1u32,
        "DeviceCMYK" => 4,
        _ => 3, // DeviceRGB, CalRGB, etc.
    };

    let bytes_per_component = 1u32;
    let row_bytes = (width * channels * bits).div_ceil(8) as usize;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);

    for row in 0..height as usize {
        let start = row * row_bytes;
        if start >= data.len() {
            // Pad with black
            for _ in 0..width {
                rgba.extend_from_slice(&[0, 0, 0, 255]);
            }
            continue;
        }
        let row_data = &data[start..(start + row_bytes).min(data.len())];

        for col in 0..width as usize {
            let pixel_start = col * (channels * bytes_per_component) as usize;
            match channels {
                1 => {
                    let g = row_data.get(pixel_start).copied().unwrap_or(0);
                    rgba.extend_from_slice(&[g, g, g, 255]);
                }
                3 => {
                    let r = row_data.get(pixel_start).copied().unwrap_or(0);
                    let g = row_data.get(pixel_start + 1).copied().unwrap_or(0);
                    let b = row_data.get(pixel_start + 2).copied().unwrap_or(0);
                    rgba.extend_from_slice(&[r, g, b, 255]);
                }
                4 => {
                    // CMYK → RGB
                    let c = row_data.get(pixel_start).copied().unwrap_or(0) as f32 / 255.0;
                    let m = row_data.get(pixel_start + 1).copied().unwrap_or(0) as f32 / 255.0;
                    let y = row_data.get(pixel_start + 2).copied().unwrap_or(0) as f32 / 255.0;
                    let k = row_data.get(pixel_start + 3).copied().unwrap_or(0) as f32 / 255.0;
                    let r = ((1.0 - c) * (1.0 - k) * 255.0) as u8;
                    let g = ((1.0 - m) * (1.0 - k) * 255.0) as u8;
                    let b = ((1.0 - y) * (1.0 - k) * 255.0) as u8;
                    rgba.extend_from_slice(&[r, g, b, 255]);
                }
                _ => {
                    rgba.extend_from_slice(&[0, 0, 0, 255]);
                }
            }
        }
    }
    rgba
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{PdfDictionary, PdfObject};
    use std::collections::HashMap;

    // -----------------------------------------------------------------------
    // Helper: build a minimal PdfDictionary for an image XObject
    // -----------------------------------------------------------------------

    fn make_image_dict(
        width: i64,
        height: i64,
        bits: i64,
        color_space: &str,
        filter: Option<&str>,
    ) -> PdfDictionary {
        let mut map = HashMap::new();
        map.insert("Subtype".to_string(), PdfObject::Name("Image".to_string()));
        map.insert("Width".to_string(), PdfObject::Integer(width));
        map.insert("Height".to_string(), PdfObject::Integer(height));
        map.insert("BitsPerComponent".to_string(), PdfObject::Integer(bits));
        map.insert(
            "ColorSpace".to_string(),
            PdfObject::Name(color_space.to_string()),
        );
        if let Some(f) = filter {
            map.insert("Filter".to_string(), PdfObject::Name(f.to_string()));
        }
        PdfDictionary(map)
    }

    // -----------------------------------------------------------------------
    // DecodedImage::decode — raw image data
    // -----------------------------------------------------------------------

    #[test]
    fn test_decode_raw_rgb_1x1() {
        let dict = make_image_dict(1, 1, 8, "DeviceRGB", None);
        let data = vec![255u8, 0, 0]; // red pixel
        let img = DecodedImage::decode(&dict, &data).expect("decode should succeed");
        assert_eq!(img.width, 1);
        assert_eq!(img.height, 1);
        assert_eq!(img.pixels.len(), 4); // RGBA
        assert_eq!(img.pixels[0], 255); // R
        assert_eq!(img.pixels[1], 0); // G
        assert_eq!(img.pixels[2], 0); // B
        assert_eq!(img.pixels[3], 255); // A (fully opaque)
    }

    #[test]
    fn test_decode_raw_rgb_2x2() {
        let dict = make_image_dict(2, 2, 8, "DeviceRGB", None);
        // 4 pixels, each RGB = [R, G, B]
        let data: Vec<u8> = vec![
            255, 0, 0, // pixel (0,0): red
            0, 255, 0, // pixel (0,1): green
            0, 0, 255, // pixel (1,0): blue
            255, 255, 255, // pixel (1,1): white
        ];
        let img = DecodedImage::decode(&dict, &data).expect("decode");
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        assert_eq!(img.pixels.len(), 16); // 4 pixels * 4 channels
    }

    #[test]
    fn test_decode_raw_gray_1x1() {
        let dict = make_image_dict(1, 1, 8, "DeviceGray", None);
        let data = vec![128u8]; // 50% gray
        let img = DecodedImage::decode(&dict, &data).expect("decode");
        assert_eq!(img.width, 1);
        assert_eq!(img.height, 1);
        assert_eq!(img.pixels.len(), 4);
        // Gray: R == G == B == 128
        assert_eq!(img.pixels[0], 128);
        assert_eq!(img.pixels[1], 128);
        assert_eq!(img.pixels[2], 128);
        assert_eq!(img.pixels[3], 255);
    }

    #[test]
    fn test_decode_raw_gray_white() {
        let dict = make_image_dict(1, 1, 8, "DeviceGray", None);
        let data = vec![255u8];
        let img = DecodedImage::decode(&dict, &data).expect("decode");
        assert_eq!(&img.pixels[0..3], &[255u8, 255, 255]);
    }

    #[test]
    fn test_decode_raw_gray_black() {
        let dict = make_image_dict(1, 1, 8, "DeviceGray", None);
        let data = vec![0u8];
        let img = DecodedImage::decode(&dict, &data).expect("decode");
        assert_eq!(&img.pixels[0..3], &[0u8, 0, 0]);
    }

    #[test]
    fn test_decode_raw_cmyk_1x1() {
        // CMYK pure black: C=0, M=0, Y=0, K=255
        let dict = make_image_dict(1, 1, 8, "DeviceCMYK", None);
        let data = vec![0u8, 0, 0, 255]; // K=255 = pure black
        let img = DecodedImage::decode(&dict, &data).expect("decode");
        assert_eq!(img.pixels.len(), 4);
        // (1-0)*(1-1)*255 = 0 for all channels
        assert_eq!(img.pixels[0], 0); // R
        assert_eq!(img.pixels[1], 0); // G
        assert_eq!(img.pixels[2], 0); // B
    }

    #[test]
    fn test_decode_raw_cmyk_white() {
        // CMYK white: C=0, M=0, Y=0, K=0
        let dict = make_image_dict(1, 1, 8, "DeviceCMYK", None);
        let data = vec![0u8, 0, 0, 0];
        let img = DecodedImage::decode(&dict, &data).expect("decode");
        // (1-0)*(1-0)*255 = 255 for all channels
        assert_eq!(img.pixels[0], 255); // R
        assert_eq!(img.pixels[1], 255); // G
        assert_eq!(img.pixels[2], 255); // B
    }

    #[test]
    fn test_decode_width_height_extraction() {
        let dict = make_image_dict(100, 200, 8, "DeviceRGB", None);
        let data = vec![0u8; 100 * 200 * 3];
        let img = DecodedImage::decode(&dict, &data).expect("decode");
        assert_eq!(img.width, 100);
        assert_eq!(img.height, 200);
    }

    #[test]
    fn test_decode_pixel_count_equals_width_times_height_times_4() {
        let w = 4u32;
        let h = 3u32;
        let dict = make_image_dict(w as i64, h as i64, 8, "DeviceRGB", None);
        let data = vec![0u8; (w * h * 3) as usize];
        let img = DecodedImage::decode(&dict, &data).expect("decode");
        assert_eq!(img.pixels.len(), (w * h * 4) as usize);
    }

    #[test]
    fn test_decode_short_data_pads_with_black() {
        // Provide too-short data: missing rows should be padded with black
        let dict = make_image_dict(2, 2, 8, "DeviceRGB", None);
        let data = vec![255u8, 255, 255, 255, 255, 255]; // only 2 of 4 pixels
        let img = DecodedImage::decode(&dict, &data).expect("decode");
        assert_eq!(img.width, 2);
        assert_eq!(img.height, 2);
        assert_eq!(img.pixels.len(), 16);
        // Second row should be padded with black (0,0,0,255)
        assert_eq!(img.pixels[8], 0); // row 1, col 0, R
        assert_eq!(img.pixels[9], 0); // row 1, col 0, G
        assert_eq!(img.pixels[10], 0); // row 1, col 0, B
    }

    #[test]
    fn test_decode_flatedecode_filter_treated_as_raw() {
        // FlateDecode filter means the stream is already decoded (in our parser)
        let dict = make_image_dict(1, 1, 8, "DeviceRGB", Some("FlateDecode"));
        let data = vec![0u8, 128, 255]; // one RGB pixel
        let img = DecodedImage::decode(&dict, &data).expect("decode with FlateDecode");
        assert_eq!(img.pixels[0], 0); // R
        assert_eq!(img.pixels[1], 128); // G
        assert_eq!(img.pixels[2], 255); // B
        assert_eq!(img.pixels[3], 255); // A
    }

    // -----------------------------------------------------------------------
    // decode_raw helper — internal function, tested via DecodedImage::decode
    // -----------------------------------------------------------------------

    #[test]
    fn test_decode_raw_all_channels_opaque() {
        let dict = make_image_dict(3, 1, 8, "DeviceRGB", None);
        let data: Vec<u8> = vec![
            10, 20, 30, // pixel 0
            40, 50, 60, // pixel 1
            70, 80, 90, // pixel 2
        ];
        let img = DecodedImage::decode(&dict, &data).expect("decode");
        // Check alpha is always 255
        for i in 0..3 {
            assert_eq!(
                img.pixels[i * 4 + 3],
                255,
                "Alpha at pixel {} should be 255",
                i
            );
        }
    }

    // -----------------------------------------------------------------------
    // Image XObject detection helpers
    // -----------------------------------------------------------------------

    #[test]
    fn test_image_dict_subtype_is_image() {
        let dict = make_image_dict(1, 1, 8, "DeviceRGB", None);
        assert_eq!(dict.get_name("Subtype"), Some("Image"));
    }

    #[test]
    fn test_image_dict_width_height_bits() {
        let dict = make_image_dict(640, 480, 8, "DeviceRGB", None);
        assert_eq!(dict.get_integer("Width"), Some(640));
        assert_eq!(dict.get_integer("Height"), Some(480));
        assert_eq!(dict.get_integer("BitsPerComponent"), Some(8));
    }

    #[test]
    fn test_image_dict_colorspace_device_gray() {
        let dict = make_image_dict(1, 1, 8, "DeviceGray", None);
        assert_eq!(dict.get_name("ColorSpace"), Some("DeviceGray"));
    }

    #[test]
    fn test_image_dict_colorspace_device_cmyk() {
        let dict = make_image_dict(1, 1, 8, "DeviceCMYK", None);
        assert_eq!(dict.get_name("ColorSpace"), Some("DeviceCMYK"));
    }

    #[test]
    fn test_image_dict_filter_dct_decode() {
        let dict = make_image_dict(1, 1, 8, "DeviceRGB", Some("DCTDecode"));
        assert_eq!(dict.get_name("Filter"), Some("DCTDecode"));
    }

    #[test]
    fn test_image_dict_filter_flatedecode() {
        let dict = make_image_dict(1, 1, 8, "DeviceRGB", Some("FlateDecode"));
        assert_eq!(dict.get_name("Filter"), Some("FlateDecode"));
    }

    #[test]
    fn test_image_dict_no_filter() {
        let dict = make_image_dict(1, 1, 8, "DeviceRGB", None);
        assert_eq!(dict.get_name("Filter"), None);
    }
}
