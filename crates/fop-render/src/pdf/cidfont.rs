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

/// Generate CIDToGIDMap stream mapping original GIDs (CIDs) to new (subset) GIDs.
///
/// Per the module-level CID invariant in `fop_render::pdf::font`:
///
/// > CID == original TrueType glyph ID
///
/// This stream is therefore indexed by the original (pre-subset) GID and each
/// 2-byte entry contains the corresponding new (post-subset, renumbered) GID.
/// A PDF viewer reads `CIDToGIDMap[cid * 2 .. cid * 2 + 2]` as a big-endian
/// u16 to obtain the actual glyph index inside the embedded font program.
///
/// Because original GIDs are `u16` values (at most 65 535), the stream is at
/// most 131 072 bytes (64 Ki entries × 2 bytes). In practice it is much smaller
/// because we only allocate up to `max_orig_gid + 1` entries.
///
/// This function is correct for **all** Unicode scalars including astral
/// characters (U+10000..=U+10FFFF): their original GIDs are ordinary u16 values,
/// so no special surrogate-pair handling is needed here.
///
/// # Arguments
/// * `char_to_new_glyph` — mapping from characters to their **new** (post-subset) GIDs.
/// * `char_to_orig_glyph` — mapping from characters to their **original** (pre-subset) GIDs.
///   When non-empty this is authoritative; when empty the function falls back to
///   treating `char_to_new_glyph` as an identity map (CID == new GID).
///
/// # Returns
/// Binary data suitable for embedding as a PDF stream object.
pub fn generate_cidtogidmap_stream(
    char_to_new_glyph: &std::collections::HashMap<char, u16>,
    char_to_orig_glyph: &std::collections::HashMap<char, u16>,
) -> Vec<u8> {
    if char_to_orig_glyph.is_empty() && char_to_new_glyph.is_empty() {
        // No mappings at all — return a minimal 2-byte stream (CID 0 → GID 0).
        return vec![0u8; 2];
    }

    if !char_to_orig_glyph.is_empty() {
        // Preferred path: orig_gid is the CID, new_gid is the GID in the subset.
        let max_orig_gid = char_to_orig_glyph.values().copied().max().unwrap_or(0) as usize;

        // Allocate stream: 2 bytes per CID slot.  Slots with no mapping stay 0
        // (→ GID 0 = .notdef), which is the correct fallback.
        let mut stream = vec![0u8; (max_orig_gid + 1) * 2];

        for (&ch, &orig_gid) in char_to_orig_glyph.iter() {
            if let Some(&new_gid) = char_to_new_glyph.get(&ch) {
                let offset = (orig_gid as usize) * 2;
                // Big-endian u16
                stream[offset] = (new_gid >> 8) as u8;
                stream[offset + 1] = (new_gid & 0xFF) as u8;
            }
        }

        stream
    } else {
        // Fallback: no orig map — treat new GID as CID (identity subset, BMP only).
        // This path is hit for non-subsetted fonts; since orig == new in an
        // un-subsetted font this is still consistent.
        let max_cid = char_to_new_glyph.values().copied().max().unwrap_or(0) as usize;

        let mut stream = vec![0u8; (max_cid + 1) * 2];
        for (&_ch, &gid) in char_to_new_glyph.iter() {
            let offset = (gid as usize) * 2;
            stream[offset] = (gid >> 8) as u8;
            stream[offset + 1] = (gid & 0xFF) as u8;
        }
        stream
    }
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
        use std::collections::HashMap;
        let char_to_new_glyph: HashMap<char, u16> = HashMap::new();
        let char_to_orig_glyph: HashMap<char, u16> = HashMap::new();
        let map = generate_cidtogidmap_stream(&char_to_new_glyph, &char_to_orig_glyph);
        // Both maps empty → minimal 2-byte stream.
        assert_eq!(map, vec![0u8; 2]);
    }

    #[test]
    fn test_generate_cidtogidmap_stream_single_char() {
        use std::collections::HashMap;
        // 'A' has orig GID 36 (arbitrary in this test) and new GID 1.
        // CIDToGIDMap[36] = 1 → stream length = (36+1)*2 = 74 bytes.
        let mut char_to_new_glyph: HashMap<char, u16> = HashMap::new();
        char_to_new_glyph.insert('A', 1);
        let mut char_to_orig_glyph: HashMap<char, u16> = HashMap::new();
        char_to_orig_glyph.insert('A', 36);
        let map = generate_cidtogidmap_stream(&char_to_new_glyph, &char_to_orig_glyph);
        // Stream is (36+1)*2 = 74 bytes.
        assert_eq!(map.len(), 74);
        // At offset 36*2=72: big-endian 1 = 0x00, 0x01
        assert_eq!(map[72], 0x00);
        assert_eq!(map[73], 0x01);
    }

    /// Non-BMP character (U+1F600 😀) must have its orig GID (a u16) as the CID
    /// and the new (post-subset) GID as the value in the CIDToGIDMap stream.
    /// The CID must fit in u16 — no 0x1F600 (>u16) should appear as an index.
    #[test]
    fn test_generate_cidtogidmap_stream_astral_char() {
        use std::collections::HashMap;
        // Hypothetical: orig GID 3456 (a realistic u16), new GID 5.
        let emoji = '\u{1F600}';
        let mut char_to_new_glyph: HashMap<char, u16> = HashMap::new();
        char_to_new_glyph.insert(emoji, 5);
        let mut char_to_orig_glyph: HashMap<char, u16> = HashMap::new();
        char_to_orig_glyph.insert(emoji, 3456);
        let map = generate_cidtogidmap_stream(&char_to_new_glyph, &char_to_orig_glyph);
        // Stream size = (3456+1)*2 = 6914 bytes (not ~512KB as the old code did).
        assert_eq!(map.len(), (3456 + 1) * 2);
        // CIDToGIDMap[3456] = 5
        let offset = 3456 * 2;
        assert_eq!(map[offset], 0x00); // high byte of 5
        assert_eq!(map[offset + 1], 0x05); // low byte of 5
    }
}
