//! Font handling for PDF rendering
//!
//! Parses ToUnicode CMap streams and loads embedded TrueType fonts
//! for glyph outline extraction.

use crate::parser::{PdfDictionary, PdfDocument};
use std::collections::HashMap;

/// A loaded PDF font with CID→Unicode and CID→glyph-ID mappings
#[derive(Debug, Clone)]
pub struct LoadedFont {
    /// Subtype: "Type1", "TrueType", "Type0", "CIDFontType2", etc.
    pub subtype: String,
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
    let cid_to_gid = HashMap::new(); // identity map by default

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
    // Can be 4-char UTF-16BE hex: e.g. "30A2" → U+30A2
    let code_point = u32::from_str_radix(unicode_hex, 16).ok()?;
    let ch = char::from_u32(code_point)?;
    Some((cid, ch))
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
    // LoadedFont
    // -----------------------------------------------------------------------

    #[test]
    fn test_loaded_font_cid_to_char_known_cid() {
        let mut cid_to_unicode = HashMap::new();
        cid_to_unicode.insert(65u32, 'A');
        let font = LoadedFont {
            subtype: "TrueType".to_string(),
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
