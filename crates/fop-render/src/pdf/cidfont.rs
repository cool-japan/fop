//! CID-keyed font support for Unicode (Japanese/CJK) text in PDF
//!
//! This module implements Type 0 composite fonts with CIDFont descendants,
//! which are required for proper Unicode text rendering in PDF.

use std::collections::HashMap;

/// Generate a Type 0 composite font dictionary for Unicode text
///
/// Type 0 fonts are required for multi-byte character sets like CJK.
/// They use CIDFont descendants with Identity-H encoding.
pub fn generate_type0_font_dict(
    base_font_name: &str,
    cid_font_obj_id: usize,
    to_unicode_obj_id: usize,
) -> String {
    format!(
        "<<\n\
         /Type /Font\n\
         /Subtype /Type0\n\
         /BaseFont /{}-Identity-H\n\
         /Encoding /Identity-H\n\
         /DescendantFonts [{} 0 R]\n\
         /ToUnicode {} 0 R\n\
         >>",
        base_font_name, cid_font_obj_id, to_unicode_obj_id
    )
}

/// Generate a CIDFont dictionary (descendant of Type 0 font)
pub fn generate_cidfont_dict(
    base_font_name: &str,
    descriptor_obj_id: usize,
    widths: &[u16],
    default_width: u16,
) -> String {
    // For CIDFont, we use a default width and specific widths for used characters
    // For simplicity, we'll use the default width for all characters initially

    format!(
        "<<\n\
         /Type /Font\n\
         /Subtype /CIDFontType2\n\
         /BaseFont /{}\n\
         /CIDSystemInfo <<\n\
           /Registry (Adobe)\n\
           /Ordering (Identity)\n\
           /Supplement 0\n\
         >>\n\
         /FontDescriptor {} 0 R\n\
         /DW {}\n\
         /W [0 [{} ]]\n\
         >>",
        base_font_name,
        descriptor_obj_id,
        default_width,
        widths
            .iter()
            .map(|w| w.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    )
}

/// Generate ToUnicode CMap for CID-keyed fonts
pub fn generate_cidfont_tounicode_cmap(char_map: &HashMap<u16, char>) -> String {
    let mut cmap = String::from(
        "/CIDInit /ProcSet findresource begin\n\
         12 dict begin\n\
         begincmap\n\
         /CIDSystemInfo <<\n\
           /Registry (Adobe)\n\
           /Ordering (UCS)\n\
           /Supplement 0\n\
         >> def\n\
         /CMapName /Adobe-Identity-UCS def\n\
         /CMapType 2 def\n\
         1 begincodespacerange\n\
         <0000> <FFFF>\n\
         endcodespacerange\n",
    );

    // Add character mappings
    if !char_map.is_empty() {
        cmap.push_str(&format!("{} beginbfchar\n", char_map.len()));

        for (&glyph_id, &ch) in char_map.iter() {
            // Map glyph ID to Unicode code point
            cmap.push_str(&format!("<{:04X}> <{:04X}>\n", glyph_id, ch as u32));
        }

        cmap.push_str("endbfchar\n");
    }

    cmap.push_str(
        "endcmap\n\
         CMapName currentdict /CMap defineresource pop\n\
         end\n\
         end\n",
    );

    cmap
}

/// Encode text as UTF-16BE for use with CID-keyed fonts
///
/// CID-keyed fonts expect text in UTF-16BE encoding.
/// Returns a hex string suitable for PDF (e.g., <FEFF...>)
pub fn encode_text_utf16be(text: &str) -> String {
    let mut result = String::from("<FEFF"); // BOM for UTF-16BE

    for ch in text.chars() {
        let code = ch as u32;
        if code <= 0xFFFF {
            // BMP character - single UTF-16 code unit
            result.push_str(&format!("{:04X}", code));
        } else {
            // Supplementary character - surrogate pair
            let code = code - 0x10000;
            let high = 0xD800 + (code >> 10);
            let low = 0xDC00 + (code & 0x3FF);
            result.push_str(&format!("{:04X}{:04X}", high, low));
        }
    }

    result.push('>');
    result
}

/// Generate CIDToGIDMap stream for mapping CIDs to actual glyph IDs
///
/// Creates a binary stream where each CID (Unicode codepoint) maps to its
/// actual glyph ID in the TrueType font. This is required when the font's
/// glyph IDs don't match Unicode codepoints (which is common for CJK fonts).
///
/// Format: Binary stream where offset = CID * 2, value = uint16 GID (big-endian)
///
/// # Arguments
/// * `char_to_glyph` - Mapping from characters to their glyph IDs in the font
/// * `used_chars` - Set of characters actually used in the document
///
/// # Returns
/// Binary data suitable for embedding as a PDF stream
pub fn generate_cidtogidmap_stream(
    char_to_glyph: &std::collections::HashMap<char, u16>,
    used_chars: &std::collections::BTreeSet<char>,
) -> Vec<u8> {
    // Find max CID (Unicode value) we'll use
    let max_cid = used_chars.iter().map(|&c| c as u32).max().unwrap_or(0);

    // Create byte array: 2 bytes per CID
    let mut map = vec![0u8; ((max_cid + 1) * 2) as usize];

    // Fill in mappings for used characters
    for &ch in used_chars.iter() {
        let cid = ch as u32;
        if let Some(&gid) = char_to_glyph.get(&ch) {
            let offset = (cid * 2) as usize;
            // Big-endian uint16
            map[offset] = (gid >> 8) as u8;
            map[offset + 1] = (gid & 0xFF) as u8;
        }
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_text_utf16be_ascii() {
        let encoded = encode_text_utf16be("Hello");
        assert!(encoded.starts_with("<FEFF"));
        assert!(encoded.ends_with('>'));
        // H=0048, e=0065, l=006C, o=006F
        assert!(encoded.contains("0048"));
        assert!(encoded.contains("0065"));
    }

    #[test]
    fn test_encode_text_utf16be_japanese() {
        let encoded = encode_text_utf16be("請求書");
        assert!(encoded.starts_with("<FEFF"));
        // 請=8ACB, 求=6C42, 書=66F8
        assert!(encoded.contains("8ACB"));
        assert!(encoded.contains("6C42"));
        assert!(encoded.contains("66F8"));
    }

    #[test]
    fn test_encode_text_utf16be_mixed() {
        let encoded = encode_text_utf16be("Hello世界");
        assert!(encoded.starts_with("<FEFF"));
        // Should contain both ASCII and Japanese
        assert!(encoded.contains("0048")); // H
        assert!(encoded.contains("4E16")); // 世
        assert!(encoded.contains("754C")); // 界
    }

    #[test]
    fn test_tounicode_cmap_generation() {
        let mut char_map = HashMap::new();
        char_map.insert(100, 'A');
        char_map.insert(200, '請');

        let cmap = generate_cidfont_tounicode_cmap(&char_map);

        assert!(cmap.contains("begincmap"));
        assert!(cmap.contains("endbfchar"));
        assert!(cmap.contains("<0064> <0041>")); // 100 -> 'A'
        assert!(cmap.contains("<00C8> <8ACB>")); // 200 -> '請'
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    #[test]
    fn test_type0_font_dict_structure() {
        let dict = generate_type0_font_dict("NotoSans", 5, 6);
        assert!(dict.contains("/Type /Font"));
        assert!(dict.contains("/Subtype /Type0"));
        assert!(dict.contains("/Encoding /Identity-H"));
        assert!(dict.contains("NotoSans"));
        assert!(dict.contains("5 0 R")); // cid_font_obj_id
        assert!(dict.contains("6 0 R")); // to_unicode_obj_id
    }

    #[test]
    fn test_type0_font_dict_base_font_name_format() {
        let dict = generate_type0_font_dict("MyFont", 10, 11);
        // BaseFont should contain font name with Identity-H suffix
        assert!(dict.contains("MyFont-Identity-H"));
    }

    #[test]
    fn test_cidfont_dict_structure() {
        let widths = vec![500u16; 10];
        let dict = generate_cidfont_dict("NotoSans", 3, &widths, 500);
        assert!(dict.contains("/Type /Font"));
        assert!(dict.contains("/Subtype /CIDFontType2"));
        assert!(dict.contains("/Registry (Adobe)"));
        assert!(dict.contains("/Ordering (Identity)"));
        assert!(dict.contains("NotoSans"));
    }

    #[test]
    fn test_cidfont_dict_contains_descriptor_ref() {
        let widths = vec![600u16; 5];
        let dict = generate_cidfont_dict("TestFont", 42, &widths, 600);
        assert!(dict.contains("42 0 R"));
    }

    #[test]
    fn test_cidfont_dict_default_width() {
        let widths: Vec<u16> = vec![];
        let dict = generate_cidfont_dict("Font", 1, &widths, 1000);
        assert!(dict.contains("/DW 1000"));
    }

    #[test]
    fn test_tounicode_cmap_empty_map() {
        let char_map = HashMap::new();
        let cmap = generate_cidfont_tounicode_cmap(&char_map);
        // Should still have valid CMap structure
        assert!(cmap.contains("begincmap"));
        assert!(cmap.contains("endcmap"));
    }

    #[test]
    fn test_encode_text_utf16be_empty() {
        let encoded = encode_text_utf16be("");
        // Should at least have the BOM prefix and closing angle bracket
        assert!(encoded.starts_with("<FEFF"));
        assert!(encoded.ends_with('>'));
    }

    #[test]
    fn test_generate_cidtogidmap_stream_empty() {
        use std::collections::{BTreeSet, HashMap};
        let char_to_glyph: HashMap<char, u16> = HashMap::new();
        let used_chars: BTreeSet<char> = BTreeSet::new();
        let map = generate_cidtogidmap_stream(&char_to_glyph, &used_chars);
        // Empty used_chars → max_cid = 0 → 2 bytes (map of size (0+1)*2=2)
        assert_eq!(map, vec![0u8; 2]);
    }

    #[test]
    fn test_generate_cidtogidmap_stream_single_char() {
        use std::collections::{BTreeSet, HashMap};
        let mut char_to_glyph: HashMap<char, u16> = HashMap::new();
        char_to_glyph.insert('A', 36);
        let mut used_chars: BTreeSet<char> = BTreeSet::new();
        used_chars.insert('A');
        let map = generate_cidtogidmap_stream(&char_to_glyph, &used_chars);
        // 'A' is U+0041 = 65; map has 2*(65+1)=132 bytes
        assert_eq!(map.len(), 132);
        // At offset 65*2=130: big-endian 36 = 0x00, 0x24
        assert_eq!(map[130], 0x00);
        assert_eq!(map[131], 0x24);
    }
}
