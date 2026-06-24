//! Font handling for PDF rendering
//!
//! Parses ToUnicode CMap streams and loads embedded TrueType fonts
//! for glyph outline extraction.

use crate::parser::{PdfDictionary, PdfDocument};
use std::collections::HashMap;

/// Simple font encoding type parsed from /Encoding entry.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SimpleEncoding {
    #[default]
    WinAnsi,
    MacRoman,
    Standard,
    Identity,
}

/// WinAnsiEncoding byte-to-Unicode table (PDF Reference 1.7, Appendix D).
/// Index = byte value (0..=255), value = Unicode codepoint (0 = unmapped).
pub static WIN_ANSI_TABLE: [u16; 256] = [
    // 0x00–0x1F: control chars (unmapped)
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    // 0x20–0x7E: ASCII (direct mapping)
    0x0020, 0x0021, 0x0022, 0x0023, 0x0024, 0x0025, 0x0026, 0x0027, 0x0028, 0x0029, 0x002A, 0x002B,
    0x002C, 0x002D, 0x002E, 0x002F, 0x0030, 0x0031, 0x0032, 0x0033, 0x0034, 0x0035, 0x0036, 0x0037,
    0x0038, 0x0039, 0x003A, 0x003B, 0x003C, 0x003D, 0x003E, 0x003F, 0x0040, 0x0041, 0x0042, 0x0043,
    0x0044, 0x0045, 0x0046, 0x0047, 0x0048, 0x0049, 0x004A, 0x004B, 0x004C, 0x004D, 0x004E, 0x004F,
    0x0050, 0x0051, 0x0052, 0x0053, 0x0054, 0x0055, 0x0056, 0x0057, 0x0058, 0x0059, 0x005A, 0x005B,
    0x005C, 0x005D, 0x005E, 0x005F, 0x0060, 0x0061, 0x0062, 0x0063, 0x0064, 0x0065, 0x0066, 0x0067,
    0x0068, 0x0069, 0x006A, 0x006B, 0x006C, 0x006D, 0x006E, 0x006F, 0x0070, 0x0071, 0x0072, 0x0073,
    0x0074, 0x0075, 0x0076, 0x0077, 0x0078, 0x0079, 0x007A, 0x007B, 0x007C, 0x007D, 0x007E,
    // 0x7F: undefined
    0, // 0x80–0x9F: Windows-1252 extensions
    0x20AC, 0, 0x201A, 0x0192, 0x201E, 0x2026, 0x2020, 0x2021, 0x02C6, 0x2030, 0x0160, 0x2039,
    0x0152, 0, 0x017D, 0, 0, 0x2018, 0x2019, 0x201C, 0x201D, 0x2022, 0x2013, 0x2014, 0x02DC,
    0x2122, 0x0161, 0x203A, 0x0153, 0, 0x017E, 0x0178,
    // 0xA0–0xFF: Latin-1 supplement (direct mapping)
    0x00A0, 0x00A1, 0x00A2, 0x00A3, 0x00A4, 0x00A5, 0x00A6, 0x00A7, 0x00A8, 0x00A9, 0x00AA, 0x00AB,
    0x00AC, 0x00AD, 0x00AE, 0x00AF, 0x00B0, 0x00B1, 0x00B2, 0x00B3, 0x00B4, 0x00B5, 0x00B6, 0x00B7,
    0x00B8, 0x00B9, 0x00BA, 0x00BB, 0x00BC, 0x00BD, 0x00BE, 0x00BF, 0x00C0, 0x00C1, 0x00C2, 0x00C3,
    0x00C4, 0x00C5, 0x00C6, 0x00C7, 0x00C8, 0x00C9, 0x00CA, 0x00CB, 0x00CC, 0x00CD, 0x00CE, 0x00CF,
    0x00D0, 0x00D1, 0x00D2, 0x00D3, 0x00D4, 0x00D5, 0x00D6, 0x00D7, 0x00D8, 0x00D9, 0x00DA, 0x00DB,
    0x00DC, 0x00DD, 0x00DE, 0x00DF, 0x00E0, 0x00E1, 0x00E2, 0x00E3, 0x00E4, 0x00E5, 0x00E6, 0x00E7,
    0x00E8, 0x00E9, 0x00EA, 0x00EB, 0x00EC, 0x00ED, 0x00EE, 0x00EF, 0x00F0, 0x00F1, 0x00F2, 0x00F3,
    0x00F4, 0x00F5, 0x00F6, 0x00F7, 0x00F8, 0x00F9, 0x00FA, 0x00FB, 0x00FC, 0x00FD, 0x00FE, 0x00FF,
];

/// A loaded PDF font with CID→Unicode and CID→glyph-ID mappings
#[derive(Debug, Clone)]
pub struct LoadedFont {
    /// Subtype: "Type1", "TrueType", "Type0", "CIDFontType2", etc.
    pub subtype: String,
    /// Base font name (PDF /BaseFont entry, e.g. "Helvetica")
    pub base_font: String,
    /// Simple font encoding (for Type1/TrueType without ToUnicode)
    pub encoding: SimpleEncoding,
    /// CID → Unicode character mapping (from ToUnicode CMap)
    pub cid_to_unicode: HashMap<u32, char>,
    /// CID → GID mapping (for embedded TrueType fonts)
    pub cid_to_gid: HashMap<u32, u16>,
    /// Embedded TrueType font data (if available)
    pub font_data: Option<Vec<u8>>,
    /// Width table: CID → advance width in glyph units (1000ths of a point)
    pub widths: HashMap<u32, f32>,
    /// Default width for CIDs not in widths table
    pub default_width: f32,
    /// Units per em for the embedded font
    pub units_per_em: u16,
}

impl LoadedFont {
    /// Load a font from a PDF font dictionary
    pub fn load(doc: &PdfDocument, font_dict: &PdfDictionary) -> Self {
        let subtype = font_dict.get_name("Subtype").unwrap_or("").to_string();

        // Parse /BaseFont name (strip leading slash if present in raw name strings)
        let base_font = font_dict
            .get_name("BaseFont")
            .unwrap_or("")
            .trim_start_matches('/')
            .to_string();

        // Parse /Encoding — only handle Name variants; dict (with /Differences) falls back to WinAnsi
        let encoding = match font_dict.get("Encoding") {
            Some(crate::parser::PdfObject::Name(s)) => match s.as_str() {
                "WinAnsiEncoding" => SimpleEncoding::WinAnsi,
                "MacRomanEncoding" => SimpleEncoding::MacRoman,
                "StandardEncoding" => SimpleEncoding::Standard,
                _ => SimpleEncoding::Identity,
            },
            _ => SimpleEncoding::WinAnsi,
        };

        // Parse ToUnicode CMap
        let cid_to_unicode = doc
            .get_to_unicode(font_dict)
            .map(|bytes| parse_to_unicode(&bytes))
            .unwrap_or_default();

        // For Type0 fonts, dig into descendant CIDFont
        let (cid_to_gid, font_data, widths, default_width, units_per_em) = if subtype == "Type0" {
            load_type0_info(doc, font_dict)
        } else {
            // Simple font
            let fd = doc.get_font_descriptor(font_dict);
            let font_data = fd.as_ref().and_then(|d| doc.get_font_file(d));
            let units_per_em = font_data
                .as_deref()
                .and_then(ttf_units_per_em)
                .unwrap_or(1000);
            (
                HashMap::new(),
                font_data,
                HashMap::new(),
                1000.0,
                units_per_em,
            )
        };

        LoadedFont {
            subtype,
            base_font,
            encoding,
            cid_to_unicode,
            cid_to_gid,
            font_data,
            widths,
            default_width,
            units_per_em,
        }
    }

    /// Get Unicode character for a CID (or glyph index for simple fonts)
    pub fn cid_to_char(&self, cid: u32) -> Option<char> {
        self.cid_to_unicode.get(&cid).copied()
    }

    /// Get advance width for a CID in glyph units
    pub fn advance_width(&self, cid: u32) -> f32 {
        self.widths.get(&cid).copied().unwrap_or(self.default_width)
    }

    /// Map CID to GID, falling back to identity (CID == GID) if not in table.
    pub fn cid_to_gid_or_identity(&self, cid: u32) -> u16 {
        self.cid_to_gid.get(&cid).copied().unwrap_or(cid as u16)
    }

    /// Map a simple-font byte to a Unicode codepoint using this font's encoding.
    pub fn simple_byte_to_char(encoding: SimpleEncoding, byte: u8) -> Option<char> {
        let cp = match encoding {
            SimpleEncoding::WinAnsi => {
                let v = WIN_ANSI_TABLE[byte as usize];
                if v == 0 {
                    return None;
                }
                v as u32
            }
            SimpleEncoding::MacRoman | SimpleEncoding::Standard | SimpleEncoding::Identity => {
                byte as u32 // Latin-1 fallback
            }
        };
        char::from_u32(cp)
    }
}

// ---------------------------------------------------------------------------
// Type0 / CID font loading
// ---------------------------------------------------------------------------

type Type0Info = (
    HashMap<u32, u16>,
    Option<Vec<u8>>,
    HashMap<u32, f32>,
    f32,
    u16,
);

fn load_type0_info(doc: &PdfDocument, font_dict: &PdfDictionary) -> Type0Info {
    let empty = (HashMap::new(), None, HashMap::new(), 1000.0, 1000u16);

    let descendant = match doc.get_descendant_font(font_dict) {
        Some(d) => d,
        None => return empty,
    };

    let fd = doc.get_font_descriptor(&descendant);
    let font_data = fd.as_ref().and_then(|d| doc.get_font_file(d));

    let units_per_em = font_data
        .as_deref()
        .and_then(ttf_units_per_em)
        .unwrap_or(1000);

    // Parse DW (default width)
    let default_width = descendant.get_integer("DW").unwrap_or(1000) as f32;

    // Parse W (widths array)
    let widths = descendant
        .get_array("W")
        .map(parse_widths_array)
        .unwrap_or_default();

    // Parse CIDToGIDMap
    // Format: binary stream where byte_offset = CID * 2, value = u16 GID big-endian.
    // /Identity (or absent) means identity mapping — empty HashMap signals identity fallback.
    let cid_to_gid: HashMap<u32, u16> = {
        let mut map = HashMap::new();
        if let Some(obj) = descendant.get("CIDToGIDMap") {
            match obj {
                crate::parser::PdfObject::Name(s) if s == "Identity" => {
                    // identity mapping — empty map means identity fallback in cid_to_gid_or_identity()
                }
                crate::parser::PdfObject::Reference(n, _) => {
                    let obj_num = *n;
                    if let Ok(bytes) = doc.decode_stream(obj_num) {
                        for (cid, chunk) in bytes.chunks_exact(2).enumerate() {
                            let gid = u16::from_be_bytes([chunk[0], chunk[1]]);
                            if gid != 0 {
                                map.insert(cid as u32, gid);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        map
    };

    (cid_to_gid, font_data, widths, default_width, units_per_em)
}

/// Parse PDF "W" widths array format:
///   [first [w0 w1 ... wN]] or [first last w]
fn parse_widths_array(arr: &[crate::parser::PdfObject]) -> HashMap<u32, f32> {
    use crate::parser::PdfObject;
    let mut map = HashMap::new();
    let mut i = 0;
    while i < arr.len() {
        let first = match arr[i].as_integer() {
            Some(n) => n as u32,
            None => {
                i += 1;
                continue;
            }
        };
        i += 1;
        if i >= arr.len() {
            break;
        }

        match &arr[i] {
            PdfObject::Array(widths) => {
                for (j, w) in widths.iter().enumerate() {
                    if let Some(wv) = w.as_real() {
                        map.insert(first + j as u32, wv as f32);
                    }
                }
                i += 1;
            }
            _ => {
                // Range form: first last w
                let last = arr[i].as_integer().unwrap_or(first as i64) as u32;
                i += 1;
                if i < arr.len() {
                    let w = arr[i].as_real().unwrap_or(1000.0) as f32;
                    for cid in first..=last {
                        map.insert(cid, w);
                    }
                    i += 1;
                }
            }
        }
    }
    map
}

// ---------------------------------------------------------------------------
// ToUnicode CMap parser
// ---------------------------------------------------------------------------

/// Parse a ToUnicode CMap stream into CID → char mapping
pub fn parse_to_unicode(data: &[u8]) -> HashMap<u32, char> {
    let text = String::from_utf8_lossy(data);
    let mut map = HashMap::new();

    let mut in_bf_char = false;
    let mut in_bf_range = false;

    for line in text.lines() {
        let line = line.trim();

        if line.ends_with("beginbfchar") {
            in_bf_char = true;
            in_bf_range = false;
            continue;
        }
        if line == "endbfchar" {
            in_bf_char = false;
            continue;
        }
        if line.ends_with("beginbfrange") {
            in_bf_range = true;
            in_bf_char = false;
            continue;
        }
        if line == "endbfrange" {
            in_bf_range = false;
            continue;
        }

        if in_bf_char {
            // Format: <CID> <Unicode>
            if let Some((cid, ch)) = parse_bf_char_line(line) {
                map.insert(cid, ch);
            }
        } else if in_bf_range {
            // Format: <start> <end> <Unicode_start>
            parse_bf_range_line(line, &mut map);
        }
    }

    map
}

fn parse_hex_u32(s: &str) -> Option<u32> {
    let s = s.trim().trim_matches('<').trim_matches('>');
    u32::from_str_radix(s.trim(), 16).ok()
}

fn parse_bf_char_line(line: &str) -> Option<(u32, char)> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }
    let cid = parse_hex_u32(parts[0])?;
    let unicode_hex = parts[1].trim().trim_matches('<').trim_matches('>');
    decode_utf16be_hex_to_char(unicode_hex).map(|ch| (cid, ch))
}

/// Decode a UTF-16BE hex string from a ToUnicode CMap destination to a `char`.
///
/// Handles both:
/// - 4-hex-digit BMP code points: `"0041"` → `'A'`
/// - 8-hex-digit surrogate pairs:  `"D83DDE00"` → `'😀'` (U+1F600)
///
/// Returns `None` for malformed input or isolated / mismatched surrogates.
fn decode_utf16be_hex_to_char(hex: &str) -> Option<char> {
    match hex.len() {
        4 => {
            // BMP: single UTF-16 code unit
            let cp = u32::from_str_radix(hex, 16).ok()?;
            char::from_u32(cp)
        }
        8 => {
            // Supplementary character encoded as a surrogate pair in UTF-16BE.
            // High surrogate: first 4 hex digits (0xD800..=0xDBFF)
            // Low surrogate:  last 4 hex digits (0xDC00..=0xDFFF)
            let high = u32::from_str_radix(&hex[..4], 16).ok()?;
            let low = u32::from_str_radix(&hex[4..], 16).ok()?;
            if (0xD800..=0xDBFF).contains(&high) && (0xDC00..=0xDFFF).contains(&low) {
                // Reconstruct the supplementary code point.
                let cp = 0x10000 + ((high - 0xD800) << 10) + (low - 0xDC00);
                char::from_u32(cp)
            } else {
                None
            }
        }
        _ => {
            // Attempt direct hex parse for other even lengths (rare but legal).
            let cp = u32::from_str_radix(hex, 16).ok()?;
            char::from_u32(cp)
        }
    }
}

fn parse_bf_range_line(line: &str, map: &mut HashMap<u32, char>) {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 3 {
        return;
    }
    let start = match parse_hex_u32(parts[0]) {
        Some(v) => v,
        None => return,
    };
    let end = match parse_hex_u32(parts[1]) {
        Some(v) => v,
        None => return,
    };
    let unicode_start_hex = parts[2].trim().trim_matches('<').trim_matches('>');
    let unicode_start = match u32::from_str_radix(unicode_start_hex, 16) {
        Ok(v) => v,
        Err(_) => return,
    };
    for offset in 0..=(end - start) {
        let cid = start + offset;
        let code_point = unicode_start + offset;
        if let Some(ch) = char::from_u32(code_point) {
            map.insert(cid, ch);
        }
    }
}

// ---------------------------------------------------------------------------
// TrueType helpers
// ---------------------------------------------------------------------------

fn ttf_units_per_em(data: &[u8]) -> Option<u16> {
    let face = ttf_parser::Face::parse(data, 0).ok()?;
    Some(face.units_per_em())
}

/// Get glyph advance width from TrueType font data
pub fn ttf_advance_width(font_data: &[u8], glyph_id: u16, units_per_em: u16) -> f32 {
    let face = match ttf_parser::Face::parse(font_data, 0) {
        Ok(f) => f,
        Err(_) => return 1000.0,
    };
    let gid = ttf_parser::GlyphId(glyph_id);
    let aw = face.glyph_hor_advance(gid).unwrap_or(units_per_em);
    // Convert to 1000-unit space
    (aw as f32 / units_per_em as f32) * 1000.0
}

/// Get glyph bounding box from TrueType
pub fn ttf_glyph_bbox(font_data: &[u8], glyph_id: u16) -> Option<[f32; 4]> {
    let face = ttf_parser::Face::parse(font_data, 0).ok()?;
    let gid = ttf_parser::GlyphId(glyph_id);
    let bbox = face.glyph_bounding_box(gid)?;
    Some([
        bbox.x_min as f32,
        bbox.y_min as f32,
        bbox.x_max as f32,
        bbox.y_max as f32,
    ])
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // parse_hex_u32
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_hex_u32_simple() {
        let v = parse_hex_u32("<0041>");
        assert_eq!(v, Some(0x0041));
    }

    #[test]
    fn test_parse_hex_u32_without_brackets() {
        let v = parse_hex_u32("0041");
        assert_eq!(v, Some(0x0041));
    }

    #[test]
    fn test_parse_hex_u32_four_digit() {
        let v = parse_hex_u32("<30A2>");
        assert_eq!(v, Some(0x30A2));
    }

    #[test]
    fn test_parse_hex_u32_zero() {
        let v = parse_hex_u32("<0000>");
        assert_eq!(v, Some(0));
    }

    #[test]
    fn test_parse_hex_u32_ff() {
        let v = parse_hex_u32("<FF>");
        assert_eq!(v, Some(0xFF));
    }

    #[test]
    fn test_parse_hex_u32_invalid_returns_none() {
        let v = parse_hex_u32("<GGGG>");
        assert!(v.is_none(), "Invalid hex should return None");
    }

    #[test]
    fn test_parse_hex_u32_empty_returns_none() {
        let v = parse_hex_u32("<>");
        assert!(v.is_none(), "Empty hex should return None");
    }

    // -----------------------------------------------------------------------
    // parse_bf_char_line
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_bf_char_line_basic() {
        let result = parse_bf_char_line("<0041> <0041>");
        assert_eq!(result, Some((0x0041u32, 'A')));
    }

    #[test]
    fn test_parse_bf_char_line_japanese() {
        // CID 1 → U+30A2 (カタカナ 'ア')
        let result = parse_bf_char_line("<0001> <30A2>");
        assert_eq!(result, Some((1u32, '\u{30A2}')));
    }

    #[test]
    fn test_parse_bf_char_line_missing_second_token() {
        let result = parse_bf_char_line("<0041>");
        assert!(result.is_none(), "Should return None with only one token");
    }

    #[test]
    fn test_parse_bf_char_line_space_char() {
        let result = parse_bf_char_line("<0020> <0020>");
        assert_eq!(result, Some((0x0020u32, ' ')));
    }

    #[test]
    fn test_parse_bf_char_line_digit() {
        let result = parse_bf_char_line("<0030> <0030>");
        assert_eq!(result, Some((0x30u32, '0')));
    }

    // -----------------------------------------------------------------------
    // parse_to_unicode — begincmap / endcmap
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_to_unicode_empty_cmap() {
        let data = b"/CIDInit /ProcSet findresource begin\nbegincmap\nendcmap\n";
        let map = parse_to_unicode(data);
        assert!(map.is_empty(), "Empty cmap should produce empty mapping");
    }

    #[test]
    fn test_parse_to_unicode_single_bfchar() {
        let cmap = b"begincmap\n1 beginbfchar\n<0001> <0041>\nendbfchar\nendcmap\n";
        let map = parse_to_unicode(cmap);
        assert_eq!(map.get(&1), Some(&'A'));
    }

    #[test]
    fn test_parse_to_unicode_multiple_bfchar() {
        let cmap = b"begincmap\n3 beginbfchar\n<0001> <0041>\n<0002> <0042>\n<0003> <0043>\nendbfchar\nendcmap\n";
        let map = parse_to_unicode(cmap);
        assert_eq!(map.get(&1), Some(&'A'));
        assert_eq!(map.get(&2), Some(&'B'));
        assert_eq!(map.get(&3), Some(&'C'));
    }

    #[test]
    fn test_parse_to_unicode_bfrange_simple() {
        // Range: CIDs 0x20..=0x22 → 'A', 'B', 'C' (U+0041..=0x0043)
        let cmap = b"begincmap\n1 beginbfrange\n<0020> <0022> <0041>\nendbfrange\nendcmap\n";
        let map = parse_to_unicode(cmap);
        assert_eq!(map.get(&0x20), Some(&'A'));
        assert_eq!(map.get(&0x21), Some(&'B'));
        assert_eq!(map.get(&0x22), Some(&'C'));
    }

    #[test]
    fn test_parse_to_unicode_bfrange_single_element() {
        let cmap = b"begincmap\n1 beginbfrange\n<0005> <0005> <0041>\nendbfrange\nendcmap\n";
        let map = parse_to_unicode(cmap);
        assert_eq!(map.get(&5), Some(&'A'));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_parse_to_unicode_bfchar_space() {
        let cmap = b"begincmap\n1 beginbfchar\n<0020> <0020>\nendbfchar\nendcmap\n";
        let map = parse_to_unicode(cmap);
        assert_eq!(map.get(&0x20), Some(&' '));
    }

    #[test]
    fn test_parse_to_unicode_bfrange_digits() {
        // CIDs 0x10..=0x19 → '0'..'9' (0x30..=0x39)
        let cmap = b"begincmap\n1 beginbfrange\n<0010> <0019> <0030>\nendbfrange\nendcmap\n";
        let map = parse_to_unicode(cmap);
        assert_eq!(map.get(&0x10), Some(&'0'));
        assert_eq!(map.get(&0x19), Some(&'9'));
        assert_eq!(map.len(), 10);
    }

    #[test]
    fn test_parse_to_unicode_bfchar_and_bfrange_combined() {
        let cmap = b"begincmap\n1 beginbfchar\n<0001> <0041>\nendbfchar\n1 beginbfrange\n<0010> <0011> <0042>\nendbfrange\nendcmap\n";
        let map = parse_to_unicode(cmap);
        assert_eq!(map.get(&1), Some(&'A'));
        assert_eq!(map.get(&0x10), Some(&'B'));
        assert_eq!(map.get(&0x11), Some(&'C'));
    }

    #[test]
    fn test_parse_to_unicode_ignores_malformed_lines() {
        // Malformed entries should not panic or cause errors
        let cmap =
            b"begincmap\n1 beginbfchar\nmalformed line here\n<0001> <0041>\nendbfchar\nendcmap\n";
        let map = parse_to_unicode(cmap);
        // At minimum CID 1 → 'A' should be present (or map could be empty if all fail)
        // The important thing is no panic
        let _ = map;
    }

    // -----------------------------------------------------------------------
    // decode_utf16be_hex_to_char — non-BMP surrogate-pair decoding
    // -----------------------------------------------------------------------

    /// BMP characters: 4-hex-digit destination → direct Unicode scalar.
    #[test]
    fn test_decode_utf16be_bmp_char() {
        assert_eq!(decode_utf16be_hex_to_char("0041"), Some('A'));
        assert_eq!(decode_utf16be_hex_to_char("30A2"), Some('\u{30A2}')); // ア
        assert_eq!(decode_utf16be_hex_to_char("0020"), Some(' '));
    }

    /// Astral characters: 8-hex-digit surrogate pair → supplementary scalar.
    /// U+1F600 😀: high=D83D, low=DE00 → "D83DDE00"
    #[test]
    fn test_decode_utf16be_astral_char_emoji() {
        let result = decode_utf16be_hex_to_char("D83DDE00");
        assert_eq!(
            result,
            Some('\u{1F600}'),
            "D83DDE00 should decode to U+1F600 😀"
        );
    }

    /// U+10300 𐌀 (Old Italic A): high=D800, low=DF00 → "D800DF00"
    #[test]
    fn test_decode_utf16be_astral_char_old_italic() {
        let result = decode_utf16be_hex_to_char("D800DF00");
        assert_eq!(
            result,
            Some('\u{10300}'),
            "D800DF00 should decode to U+10300 𐌀"
        );
    }

    /// An isolated high surrogate must not produce a char (invalid UTF-16).
    #[test]
    fn test_decode_utf16be_isolated_high_surrogate_returns_none() {
        // "D83D" is a valid high surrogate when paired, but invalid alone.
        let result = decode_utf16be_hex_to_char("D83D");
        // The 4-digit path tries to parse 0xD83D as a Unicode scalar,
        // which fails because surrogates are not valid scalars in Rust.
        assert!(
            result.is_none(),
            "Isolated high surrogate must not produce a char"
        );
    }

    /// Mismatched surrogates must not produce a char.
    #[test]
    fn test_decode_utf16be_mismatched_surrogates_returns_none() {
        // 0001 is not a valid low surrogate (0xDC00..=0xDFFF).
        let result = decode_utf16be_hex_to_char("D83D0001");
        assert!(
            result.is_none(),
            "Mismatched surrogates must not produce a char"
        );
    }

    /// parse_bf_char_line must correctly parse an 8-hex-digit surrogate-pair dest.
    #[test]
    fn test_parse_bf_char_line_astral_surrogate_pair() {
        // CID 0x14FD (GID 5373, U+10300 in DejaVu) → U+10300 encoded as D800DF00
        let result = parse_bf_char_line("<14FD> <D800DF00>");
        assert_eq!(
            result,
            Some((0x14FDu32, '\u{10300}')),
            "parse_bf_char_line must decode 8-digit surrogate-pair destination",
        );
    }

    /// parse_to_unicode must handle a CMap stream that uses surrogate-pair dests.
    #[test]
    fn test_parse_to_unicode_with_surrogate_pair_dest() {
        // A ToUnicode CMap as generated by generate_to_unicode_cmap for U+1F600:
        //   CID 0x0D80 → <D83DDE00> (UTF-16BE surrogate pair for U+1F600)
        let cmap =
            b"begincmap\n2 beginbfchar\n<0041> <0041>\n<0D80> <D83DDE00>\nendbfchar\nendcmap\n";
        let map = parse_to_unicode(cmap);
        assert_eq!(map.get(&0x0041), Some(&'A'), "BMP char must still parse");
        assert_eq!(
            map.get(&0x0D80),
            Some(&'\u{1F600}'),
            "CID 0x0D80 must map to U+1F600 😀 via surrogate pair",
        );
    }

    // -----------------------------------------------------------------------
    // LoadedFont
    // -----------------------------------------------------------------------

    #[test]
    fn test_loaded_font_cid_to_char_known_cid() {
        let mut cid_to_unicode = HashMap::new();
        cid_to_unicode.insert(65u32, 'A');
        let font = LoadedFont {
            subtype: "TrueType".to_string(),
            base_font: String::new(),
            encoding: SimpleEncoding::WinAnsi,
            cid_to_unicode,
            cid_to_gid: HashMap::new(),
            font_data: None,
            widths: HashMap::new(),
            default_width: 1000.0,
            units_per_em: 1000,
        };
        assert_eq!(font.cid_to_char(65), Some('A'));
    }

    #[test]
    fn test_loaded_font_cid_to_char_unknown_cid() {
        let font = LoadedFont {
            subtype: "TrueType".to_string(),
            base_font: String::new(),
            encoding: SimpleEncoding::WinAnsi,
            cid_to_unicode: HashMap::new(),
            cid_to_gid: HashMap::new(),
            font_data: None,
            widths: HashMap::new(),
            default_width: 1000.0,
            units_per_em: 1000,
        };
        assert_eq!(font.cid_to_char(99), None);
    }

    #[test]
    fn test_loaded_font_advance_width_from_widths_table() {
        let mut widths = HashMap::new();
        widths.insert(65u32, 750.0f32);
        let font = LoadedFont {
            subtype: "TrueType".to_string(),
            base_font: String::new(),
            encoding: SimpleEncoding::WinAnsi,
            cid_to_unicode: HashMap::new(),
            cid_to_gid: HashMap::new(),
            font_data: None,
            widths,
            default_width: 1000.0,
            units_per_em: 1000,
        };
        assert!((font.advance_width(65) - 750.0).abs() < 1e-3);
    }

    #[test]
    fn test_loaded_font_advance_width_default_for_unknown_cid() {
        let font = LoadedFont {
            subtype: "TrueType".to_string(),
            base_font: String::new(),
            encoding: SimpleEncoding::WinAnsi,
            cid_to_unicode: HashMap::new(),
            cid_to_gid: HashMap::new(),
            font_data: None,
            widths: HashMap::new(),
            default_width: 500.0,
            units_per_em: 1000,
        };
        assert!((font.advance_width(9999) - 500.0).abs() < 1e-3);
    }

    #[test]
    fn test_loaded_font_subtype_type0_detection() {
        let font = LoadedFont {
            subtype: "Type0".to_string(),
            base_font: String::new(),
            encoding: SimpleEncoding::WinAnsi,
            cid_to_unicode: HashMap::new(),
            cid_to_gid: HashMap::new(),
            font_data: None,
            widths: HashMap::new(),
            default_width: 1000.0,
            units_per_em: 1000,
        };
        assert_eq!(font.subtype, "Type0");
    }

    #[test]
    fn test_loaded_font_no_font_data() {
        let font = LoadedFont {
            subtype: "Type1".to_string(),
            base_font: String::new(),
            encoding: SimpleEncoding::WinAnsi,
            cid_to_unicode: HashMap::new(),
            cid_to_gid: HashMap::new(),
            font_data: None,
            widths: HashMap::new(),
            default_width: 1000.0,
            units_per_em: 1000,
        };
        assert!(font.font_data.is_none());
    }

    #[test]
    fn test_loaded_font_with_embedded_data() {
        let font = LoadedFont {
            subtype: "TrueType".to_string(),
            base_font: String::new(),
            encoding: SimpleEncoding::WinAnsi,
            cid_to_unicode: HashMap::new(),
            cid_to_gid: HashMap::new(),
            font_data: Some(vec![0u8; 100]),
            widths: HashMap::new(),
            default_width: 1000.0,
            units_per_em: 1000,
        };
        assert!(font.font_data.is_some());
        assert_eq!(
            font.font_data.as_ref().expect("test: should succeed").len(),
            100
        );
    }

    // -----------------------------------------------------------------------
    // parse_widths_array — PDF "W" format
    // -----------------------------------------------------------------------

    #[test]
    fn test_parse_widths_array_range_form() {
        use crate::parser::PdfObject;
        // [10 12 750] means CIDs 10, 11, 12 all have width 750
        let arr = vec![
            PdfObject::Integer(10),
            PdfObject::Integer(12),
            PdfObject::Real(750.0),
        ];
        let map = parse_widths_array(&arr);
        assert!((map[&10] - 750.0).abs() < 1e-3);
        assert!((map[&11] - 750.0).abs() < 1e-3);
        assert!((map[&12] - 750.0).abs() < 1e-3);
        assert_eq!(map.len(), 3);
    }

    #[test]
    fn test_parse_widths_array_individual_form() {
        use crate::parser::PdfObject;
        // [10 [600 700 800]] means CID 10→600, 11→700, 12→800
        let inner = vec![
            PdfObject::Real(600.0),
            PdfObject::Real(700.0),
            PdfObject::Real(800.0),
        ];
        let arr = vec![PdfObject::Integer(10), PdfObject::Array(inner)];
        let map = parse_widths_array(&arr);
        assert!((map[&10] - 600.0).abs() < 1e-3);
        assert!((map[&11] - 700.0).abs() < 1e-3);
        assert!((map[&12] - 800.0).abs() < 1e-3);
    }

    #[test]
    fn test_parse_widths_array_empty() {
        let map = parse_widths_array(&[]);
        assert!(map.is_empty());
    }

    #[test]
    fn test_loaded_font_cid_to_char_multiple_mappings() {
        let mut cid_to_unicode = HashMap::new();
        cid_to_unicode.insert(32u32, ' ');
        cid_to_unicode.insert(65u32, 'A');
        cid_to_unicode.insert(97u32, 'a');
        let font = LoadedFont {
            subtype: "TrueType".to_string(),
            base_font: String::new(),
            encoding: SimpleEncoding::WinAnsi,
            cid_to_unicode,
            cid_to_gid: HashMap::new(),
            font_data: None,
            widths: HashMap::new(),
            default_width: 1000.0,
            units_per_em: 1000,
        };
        assert_eq!(font.cid_to_char(32), Some(' '));
        assert_eq!(font.cid_to_char(65), Some('A'));
        assert_eq!(font.cid_to_char(97), Some('a'));
        assert_eq!(font.cid_to_char(0), None);
    }

    #[test]
    fn test_loaded_font_with_embedded_data_length() {
        let font = LoadedFont {
            subtype: "TrueType".to_string(),
            base_font: String::new(),
            encoding: SimpleEncoding::WinAnsi,
            cid_to_unicode: HashMap::new(),
            cid_to_gid: HashMap::new(),
            font_data: Some(vec![0u8; 100]),
            widths: HashMap::new(),
            default_width: 1000.0,
            units_per_em: 1000,
        };
        assert!(font.font_data.is_some());
        assert_eq!(
            font.font_data.as_ref().expect("test: should succeed").len(),
            100
        );
    }
}
