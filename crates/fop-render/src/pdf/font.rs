//! TrueType font embedding support for PDF
//!
//! Provides functionality to embed TrueType/OpenType fonts in PDF documents.

use fop_types::{FopError, Result};
use std::collections::{BTreeSet, HashMap};

/// Embedded TrueType font information
#[derive(Debug, Clone)]
pub struct PdfFont {
    /// Font name (extracted from the TTF)
    pub font_name: String,

    /// Font data (complete TTF/OTF file)
    pub font_data: Vec<u8>,

    /// Font flags for the descriptor
    pub flags: u32,

    /// Font bounding box [llx lly urx ury]
    pub bbox: [i16; 4],

    /// Italic angle (0 for upright fonts)
    pub italic_angle: i16,

    /// Ascent (height above baseline)
    pub ascent: i16,

    /// Descent (depth below baseline, typically negative)
    pub descent: i16,

    /// Cap height (height of capital letters)
    pub cap_height: i16,

    /// Stem vertical width
    pub stem_v: i16,

    /// Character widths for all used characters
    pub widths: Vec<u16>,

    /// First character code in widths array
    pub first_char: u32,

    /// Last character code in widths array
    pub last_char: u32,

    /// Units per em (font scaling factor, typically 1000 or 2048)
    pub units_per_em: u16,

    /// Character to glyph ID mapping for Unicode support
    pub char_to_glyph: std::collections::HashMap<char, u16>,
}

impl PdfFont {
    /// Parse a TrueType font from raw bytes
    ///
    /// Extracts font metrics and prepares the font for embedding in PDF.
    /// Supports basic Latin character set (ASCII 32-126).
    pub fn from_ttf_data(font_data: Vec<u8>) -> Result<Self> {
        let face = ttf_parser::Face::parse(&font_data, 0)
            .map_err(|e| FopError::Generic(format!("Failed to parse TTF: {:?}", e)))?;

        // Extract font name
        let font_name = face
            .names()
            .into_iter()
            .find(|name| name.name_id == ttf_parser::name_id::POST_SCRIPT_NAME)
            .and_then(|name| name.to_string())
            .unwrap_or_else(|| "CustomFont".to_string());

        // Get font metrics
        let units_per_em = face.units_per_em();
        let ascent = face.ascender();
        let descent = face.descender();

        // Get bounding box
        let bbox = {
            let bb = face.global_bounding_box();
            [bb.x_min, bb.y_min, bb.x_max, bb.y_max]
        };

        // Cap height - try to get from OS/2 table, fallback to ascent
        let cap_height = face
            .capital_height()
            .unwrap_or((ascent as f32 * 0.7) as i16);

        // Stem V - approximate from font weight
        let stem_v = face
            .weight()
            .to_number()
            .clamp(400, 900)
            .saturating_sub(300)
            / 5;

        // Calculate italic angle
        let italic_angle = face.italic_angle() as i16;

        // Font flags
        // Bit 1: Fixed pitch (monospace)
        // Bit 2: Serif
        // Bit 3: Symbolic (non-standard encoding)
        // Bit 6: Italic
        // Bit 7: All cap
        // Bit 17: Bold
        let mut flags = 32; // Bit 6 = non-symbolic (standard encoding)

        if face.is_monospaced() {
            flags |= 1;
        }

        if italic_angle != 0 {
            flags |= 64; // Italic flag
        }

        if face.is_bold() {
            flags |= 0x40000; // Bold flag (bit 18, value 262144)
        }

        // Build full character to glyph mapping
        // We'll populate this as characters are used
        let char_to_glyph = std::collections::HashMap::new();

        // Start with ASCII range as default
        let first_char = 32u32;
        let last_char = 126u32;
        let mut widths = Vec::new();

        for char_code in first_char..=last_char {
            let c = char::from_u32(char_code).unwrap_or('\0');
            let glyph_id = face.glyph_index(c).unwrap_or(ttf_parser::GlyphId(0));

            let width = face.glyph_hor_advance(glyph_id).unwrap_or(units_per_em / 2);

            widths.push(width);
        }

        Ok(Self {
            font_name,
            font_data,
            flags,
            bbox,
            italic_angle,
            ascent,
            descent,
            cap_height,
            stem_v: stem_v as i16,
            widths,
            first_char,
            last_char,
            units_per_em,
            char_to_glyph,
        })
    }

    /// Get the width of a character in font units
    pub fn char_width(&self, c: char) -> u16 {
        let char_code = c as u32;
        if char_code >= self.first_char && char_code <= self.last_char {
            let index = (char_code - self.first_char) as usize;
            self.widths
                .get(index)
                .copied()
                .unwrap_or(self.units_per_em / 2)
        } else {
            // For characters outside our range, use average width
            self.units_per_em / 2
        }
    }

    /// Measure text width at a given font size
    pub fn measure_text(&self, text: &str, font_size_pt: f64) -> f64 {
        let mut total_width = 0u32;
        for c in text.chars() {
            total_width += self.char_width(c) as u32;
        }

        // Convert from font units to points: (total_width / units_per_em) * font_size
        (total_width as f64 / self.units_per_em as f64) * font_size_pt
    }
}

/// Font object tuple: (descriptor_id, stream_id, cidfont_id, type0_dict_id, to_unicode_id, cidtogidmap_id, font)
pub type FontObjectTuple = (usize, usize, usize, usize, usize, usize, PdfFont);

/// Tracks character usage for font subsetting
#[derive(Debug, Clone, Default)]
pub struct FontSubsetter {
    /// Set of character codes used in the document
    used_chars: BTreeSet<char>,
}

impl FontSubsetter {
    /// Create a new font subsetter
    pub fn new() -> Self {
        Self {
            used_chars: BTreeSet::new(),
        }
    }

    /// Record characters used in text
    pub fn record_text(&mut self, text: &str) {
        for c in text.chars() {
            self.used_chars.insert(c);
        }
    }

    /// Get all used characters
    pub fn used_chars(&self) -> &BTreeSet<char> {
        &self.used_chars
    }

    /// Check if any characters have been used
    pub fn is_empty(&self) -> bool {
        self.used_chars.is_empty()
    }
}

/// Manages embedded fonts in a PDF document
#[derive(Debug, Default)]
pub struct FontManager {
    /// List of embedded fonts
    fonts: Vec<PdfFont>,

    /// Character usage tracking for each font
    subsetters: Vec<FontSubsetter>,
}

impl FontManager {
    /// Create a new font manager
    pub fn new() -> Self {
        Self {
            fonts: Vec::new(),
            subsetters: Vec::new(),
        }
    }

    /// Embed a font and return its index
    pub fn embed_font(&mut self, font_data: Vec<u8>) -> Result<usize> {
        let font = PdfFont::from_ttf_data(font_data)?;
        self.fonts.push(font);
        self.subsetters.push(FontSubsetter::new());
        Ok(self.fonts.len() - 1)
    }

    /// Record text usage for a specific font
    pub fn record_text(&mut self, font_index: usize, text: &str) {
        if let Some(subsetter) = self.subsetters.get_mut(font_index) {
            subsetter.record_text(text);
        }
    }

    /// Get an embedded font by index
    pub fn get_font(&self, index: usize) -> Option<&PdfFont> {
        self.fonts.get(index)
    }

    /// Get all embedded fonts
    pub fn fonts(&self) -> &[PdfFont] {
        &self.fonts
    }

    /// Number of embedded fonts
    pub fn font_count(&self) -> usize {
        self.fonts.len()
    }

    /// Look up an embedded font index by its family name (case-insensitive).
    ///
    /// The comparison is done against `PdfFont::font_name` (the PostScript name)
    /// as well as any alias registered via `embed_font_with_alias`.
    /// Returns `None` if no font with that name is embedded.
    pub fn find_by_name(&self, family: &str) -> Option<usize> {
        let needle = family.to_lowercase();
        self.fonts.iter().position(|f| {
            f.font_name.to_lowercase() == needle
                // Also try matching the base name without style suffixes
                // e.g. "NotoSans-Regular" should match "noto sans"
                || f.font_name
                    .to_lowercase()
                    .replace('-', " ")
                    .starts_with(&needle)
        })
    }

    /// Get subsetter for a font by index
    pub fn get_subsetter(&self, index: usize) -> Option<&FontSubsetter> {
        self.subsetters.get(index)
    }

    /// Generate PDF font objects with subsetting
    ///
    /// Returns the font descriptor object ID, font stream object ID, CIDFont dictionary object ID,
    /// Type 0 font dictionary object ID, ToUnicode CMap object ID, CIDToGIDMap object ID,
    /// and the subset font for each embedded font.
    pub fn generate_font_objects(&self, start_obj_id: usize) -> Result<Vec<FontObjectTuple>> {
        let mut result = Vec::new();
        let mut obj_id = start_obj_id;

        for (font_idx, font) in self.fonts.iter().enumerate() {
            let descriptor_id = obj_id;
            let stream_id = obj_id + 1;
            let cidfont_id = obj_id + 2;
            let type0_dict_id = obj_id + 3;
            let to_unicode_id = obj_id + 4;
            let cidtogidmap_id = obj_id + 5;
            obj_id += 6; // 6 objects per font: descriptor, stream, CIDFont, Type0, ToUnicode, CIDToGIDMap

            // Create subset font if characters were used
            let subset_font = if let Some(subsetter) = self.subsetters.get(font_idx) {
                if !subsetter.is_empty() {
                    create_subset_font(font, subsetter)?
                } else {
                    // No characters used, use full font
                    font.clone()
                }
            } else {
                // No subsetter, use full font
                font.clone()
            };

            result.push((
                descriptor_id,
                stream_id,
                cidfont_id,
                type0_dict_id,
                to_unicode_id,
                cidtogidmap_id,
                subset_font,
            ));
        }

        Ok(result)
    }
}

/// Generate PDF font descriptor object content
pub fn generate_font_descriptor(font: &PdfFont, font_stream_obj_id: usize) -> String {
    format!(
        "<<\n\
         /Type /FontDescriptor\n\
         /FontName /{}\n\
         /Flags {}\n\
         /FontBBox [{} {} {} {}]\n\
         /ItalicAngle {}\n\
         /Ascent {}\n\
         /Descent {}\n\
         /CapHeight {}\n\
         /StemV {}\n\
         /FontFile2 {} 0 R\n\
         >>",
        font.font_name,
        font.flags,
        font.bbox[0],
        font.bbox[1],
        font.bbox[2],
        font.bbox[3],
        font.italic_angle,
        font.ascent,
        font.descent,
        font.cap_height,
        font.stem_v,
        font_stream_obj_id
    )
}

/// Generate PDF font stream object header
pub fn generate_font_stream_header(font: &PdfFont) -> String {
    format!(
        "<<\n\
         /Length {}\n\
         /Length1 {}\n\
         >>",
        font.font_data.len(),
        font.font_data.len()
    )
}

/// Generate PDF font dictionary object content (Type 0 composite font for Unicode support)
pub fn generate_font_dictionary(
    font: &PdfFont,
    descriptor_obj_id: usize,
    to_unicode_obj_id: Option<usize>,
) -> String {
    // For Unicode fonts, we need Type 0 composite font structure
    generate_type0_font_dict(font, descriptor_obj_id, to_unicode_obj_id)
}

/// Generate Type 0 composite font dictionary
/// This is the top-level font object that references a CIDFont descendant
fn generate_type0_font_dict(
    font: &PdfFont,
    cidfont_obj_id: usize,
    to_unicode_obj_id: Option<usize>,
) -> String {
    let to_unicode_entry = if let Some(obj_id) = to_unicode_obj_id {
        format!("/ToUnicode {} 0 R\n         ", obj_id)
    } else {
        String::new()
    };

    format!(
        "<<\n\
         /Type /Font\n\
         /Subtype /Type0\n\
         /BaseFont /{}\n\
         /Encoding /Identity-H\n\
         /DescendantFonts [{} 0 R]\n\
         {}\
         >>",
        font.font_name, cidfont_obj_id, to_unicode_entry
    )
}

/// Generate CIDFont Type 2 dictionary (TrueType descendant font)
/// This is referenced by the Type 0 font and contains the actual font metrics
pub fn generate_cidfont_dict(
    font: &PdfFont,
    descriptor_obj_id: usize,
    cidtogidmap_obj_id: usize,
) -> String {
    // Build width array in CID format
    // For simplicity, we'll use default width (DW) and individual widths (W)
    let default_width = font.units_per_em / 2;

    // Build W array: [start_cid [widths...]]
    let mut w_array = String::new();
    if !font.widths.is_empty() {
        w_array.push_str(&format!("{} [", font.first_char));
        for (i, width) in font.widths.iter().enumerate() {
            if i > 0 {
                w_array.push(' ');
            }
            w_array.push_str(&width.to_string());
        }
        w_array.push(']');
    }

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
         {}\
         /CIDToGIDMap {} 0 R\n\
         >>",
        font.font_name,
        descriptor_obj_id,
        default_width,
        if w_array.is_empty() {
            String::new()
        } else {
            format!("/W [{}]\n         ", w_array)
        },
        cidtogidmap_obj_id
    )
}

/// Generate a ToUnicode CMap for CID fonts
/// Maps CID (character IDs) to Unicode code points
pub fn generate_to_unicode_cmap(font: &PdfFont) -> String {
    let mut cmap = String::from(
        "/CIDInit /ProcSet findresource begin\n\
         12 dict begin\n\
         begincmap\n\
         /CIDSystemInfo <<\n\
           /Registry (Adobe)\n\
           /Ordering (Identity)\n\
           /Supplement 0\n\
         >> def\n\
         /CMapName /Adobe-Identity-UCS def\n\
         /CMapType 2 def\n\
         1 begincodespacerange\n\
         <0000> <FFFF>\n\
         endcodespacerange\n",
    );

    // Build character mappings from char_to_glyph
    // For CID fonts, we map CID (glyph ID) to Unicode
    if !font.char_to_glyph.is_empty() {
        let mapping_count = font.char_to_glyph.len();
        cmap.push_str(&format!("{} beginbfchar\n", mapping_count));

        for (&c, _glyph_id) in font.char_to_glyph.iter() {
            // Map character code to Unicode
            let char_code = c as u32;
            cmap.push_str(&format!("<{:04X}> <{:04X}>\n", char_code, char_code));
        }

        cmap.push_str("endbfchar\n");
    } else {
        // If no explicit mapping, create identity mapping for the character range
        let range_size = (font.last_char - font.first_char + 1) as usize;
        if range_size > 0 && range_size <= 256 {
            cmap.push_str(&format!("{} beginbfchar\n", range_size));
            for char_code in font.first_char..=font.last_char {
                cmap.push_str(&format!("<{:04X}> <{:04X}>\n", char_code, char_code));
            }
            cmap.push_str("endbfchar\n");
        }
    }

    cmap.push_str(
        "endcmap\n\
         CMapName currentdict /CMap defineresource pop\n\
         end\n\
         end\n",
    );

    cmap
}

/// Create a subset font containing only the used characters
fn create_subset_font(original_font: &PdfFont, subsetter: &FontSubsetter) -> Result<PdfFont> {
    let face = ttf_parser::Face::parse(&original_font.font_data, 0)
        .map_err(|e| FopError::Generic(format!("Failed to parse TTF for subsetting: {:?}", e)))?;

    let used_chars = subsetter.used_chars();

    // If no characters used, return original font
    if used_chars.is_empty() {
        return Ok(original_font.clone());
    }

    // Build glyph mapping: char -> glyph_id
    let mut char_to_glyph = HashMap::new();
    let mut used_glyphs = BTreeSet::new();

    // Always include glyph 0 (notdef)
    used_glyphs.insert(ttf_parser::GlyphId(0));

    for &c in used_chars.iter() {
        if let Some(glyph_id) = face.glyph_index(c) {
            char_to_glyph.insert(c, glyph_id);
            used_glyphs.insert(glyph_id);
        }
    }

    // Create a simple subset by keeping only the used glyphs
    // For now, we'll use the full font but track which characters are used
    // A full subsetting implementation would rebuild the TTF tables

    // For CID fonts, we use CID-based width arrays
    // First and last char codes for the range
    let first_char = used_chars.iter().next().map(|&c| c as u32).unwrap_or(0);
    let last_char = used_chars
        .iter()
        .next_back()
        .map(|&c| c as u32)
        .unwrap_or(0xFFFF);

    // Build character to glyph mapping for all used characters
    let mut char_to_glyph_map = std::collections::HashMap::new();
    for &c in used_chars.iter() {
        if let Some(glyph_id) = face.glyph_index(c) {
            char_to_glyph_map.insert(c, glyph_id.0);
        }
    }

    // For CID fonts, extract widths only for used characters
    // We'll store them sparsely and use DW (default width) for others
    let mut widths = Vec::new();

    // Build width array for the character range
    // For efficiency with sparse ranges, we only include widths for actually used characters
    let range_size = (last_char - first_char + 1) as usize;
    if range_size > 0 && range_size <= 65536 {
        // Build continuous width array for the range
        for char_code in first_char..=last_char {
            if let Some(c) = char::from_u32(char_code) {
                if used_chars.contains(&c) {
                    let glyph_id = face.glyph_index(c).unwrap_or(ttf_parser::GlyphId(0));
                    let width = face
                        .glyph_hor_advance(glyph_id)
                        .unwrap_or(original_font.units_per_em / 2);
                    widths.push(width);
                } else {
                    // Use default width for unused characters in range
                    widths.push(original_font.units_per_em / 2);
                }
            } else {
                widths.push(original_font.units_per_em / 2);
            }
        }
    }

    // For a simple implementation, we'll embed the full font but only declare the used character range
    // A full implementation would rebuild the TTF with only used glyphs
    let subset_font_data = create_simple_subset(&original_font.font_data, &used_glyphs)?;

    Ok(PdfFont {
        font_name: original_font.font_name.clone(),
        font_data: subset_font_data,
        flags: original_font.flags,
        bbox: original_font.bbox,
        italic_angle: original_font.italic_angle,
        ascent: original_font.ascent,
        descent: original_font.descent,
        cap_height: original_font.cap_height,
        stem_v: original_font.stem_v,
        widths,
        first_char,
        last_char,
        units_per_em: original_font.units_per_em,
        char_to_glyph: char_to_glyph_map,
    })
}

/// Create a simple subset by including only used glyphs
fn create_simple_subset(
    font_data: &[u8],
    used_glyphs: &BTreeSet<ttf_parser::GlyphId>,
) -> Result<Vec<u8>> {
    let face = ttf_parser::Face::parse(font_data, 0)
        .map_err(|e| FopError::Generic(format!("Failed to parse TTF for subsetting: {:?}", e)))?;

    // For a basic implementation, we use a simplified approach:
    // If the subset is small enough (< 50% of glyphs), we create a minimal subset
    // Otherwise, we keep the full font

    let total_glyphs = face.number_of_glyphs();
    let used_glyph_count = used_glyphs.len();

    // If we're using more than 50% of glyphs, just use the full font
    if used_glyph_count as f32 / total_glyphs as f32 > 0.5 {
        return Ok(font_data.to_vec());
    }

    // For now, return the full font data
    // A full implementation would rebuild the TTF tables with only used glyphs
    // This requires:
    // 1. Remapping glyph IDs to be contiguous (0, 1, 2, ...)
    // 2. Rebuilding glyf, loca, cmap tables
    // 3. Updating head, hhea, hmtx, maxp tables
    // 4. Recalculating checksums

    Ok(font_data.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_manager_creation() {
        let manager = FontManager::new();
        assert_eq!(manager.font_count(), 0);
    }

    #[test]
    fn test_font_manager_default() {
        let manager = FontManager::default();
        assert_eq!(manager.font_count(), 0);
    }

    #[test]
    fn test_font_subsetter_creation() {
        let subsetter = FontSubsetter::new();
        assert!(subsetter.is_empty());
        assert_eq!(subsetter.used_chars().len(), 0);
    }

    #[test]
    fn test_font_subsetter_record_text() {
        let mut subsetter = FontSubsetter::new();
        subsetter.record_text("Hello");

        assert!(!subsetter.is_empty());
        assert_eq!(subsetter.used_chars().len(), 4); // H, e, l, o (l appears twice)

        assert!(subsetter.used_chars().contains(&'H'));
        assert!(subsetter.used_chars().contains(&'e'));
        assert!(subsetter.used_chars().contains(&'l'));
        assert!(subsetter.used_chars().contains(&'o'));
    }

    #[test]
    fn test_font_subsetter_multiple_texts() {
        let mut subsetter = FontSubsetter::new();
        subsetter.record_text("ABC");
        subsetter.record_text("BCD");

        assert_eq!(subsetter.used_chars().len(), 4); // A, B, C, D
        assert!(subsetter.used_chars().contains(&'A'));
        assert!(subsetter.used_chars().contains(&'B'));
        assert!(subsetter.used_chars().contains(&'C'));
        assert!(subsetter.used_chars().contains(&'D'));
    }

    #[test]
    fn test_font_manager_record_text() {
        let mut manager = FontManager::new();

        // Create a minimal TTF for testing (this would fail without a valid font)
        // In real usage, we'd load an actual font file
        // For now, just test that the API works

        // Verify we can call record_text even without fonts
        manager.record_text(0, "test");
        // Should not panic even if font doesn't exist
    }

    #[test]
    fn test_subsetter_unicode_support() {
        let mut subsetter = FontSubsetter::new();
        subsetter.record_text("Hello 世界");

        assert!(subsetter.used_chars().contains(&'H'));
        assert!(subsetter.used_chars().contains(&'世'));
        assert!(subsetter.used_chars().contains(&'界'));
    }

    #[test]
    fn test_subsetter_special_characters() {
        let mut subsetter = FontSubsetter::new();
        subsetter.record_text("!@#$%^&*()");

        assert!(subsetter.used_chars().contains(&'!'));
        assert!(subsetter.used_chars().contains(&'@'));
        assert!(subsetter.used_chars().contains(&'#'));
        assert!(subsetter.used_chars().contains(&'('));
        assert!(subsetter.used_chars().contains(&')'));
    }

    // Note: Testing actual TTF parsing requires a valid TTF file
    // In a real test environment, you would include a small test font
}

#[cfg(test)]
mod tests_extended {
    use super::*;

    fn minimal_pdf_font() -> PdfFont {
        PdfFont {
            font_name: "TestFont".to_string(),
            font_data: vec![0u8; 100],
            flags: 32, // non-symbolic
            bbox: [-100, -200, 900, 800],
            italic_angle: 0,
            ascent: 800,
            descent: -200,
            cap_height: 700,
            stem_v: 80,
            widths: vec![500; 95], // ASCII 32..=126
            first_char: 32,
            last_char: 126,
            units_per_em: 1000,
            char_to_glyph: HashMap::new(),
        }
    }

    #[test]
    fn test_font_subsetter_empty_initially() {
        let s = FontSubsetter::new();
        assert!(s.is_empty());
    }

    #[test]
    fn test_font_subsetter_deduplicates() {
        let mut s = FontSubsetter::new();
        s.record_text("aaa");
        // 'a' should appear only once in the set
        assert_eq!(s.used_chars().len(), 1);
        assert!(s.used_chars().contains(&'a'));
    }

    #[test]
    fn test_font_subsetter_is_not_empty_after_text() {
        let mut s = FontSubsetter::new();
        s.record_text("X");
        assert!(!s.is_empty());
    }

    #[test]
    fn test_font_manager_default_empty() {
        let m = FontManager::default();
        assert_eq!(m.font_count(), 0);
        assert!(m.get_font(0).is_none());
        assert!(m.get_subsetter(0).is_none());
    }

    #[test]
    fn test_font_manager_find_by_name_empty() {
        let m = FontManager::new();
        assert!(m.find_by_name("Arial").is_none());
    }

    #[test]
    fn test_generate_font_descriptor_contains_font_name() {
        let font = minimal_pdf_font();
        let descriptor = generate_font_descriptor(&font, 42);
        assert!(descriptor.contains("TestFont"));
        assert!(descriptor.contains("/FontDescriptor"));
    }

    #[test]
    fn test_generate_font_descriptor_references_stream_obj() {
        let font = minimal_pdf_font();
        let descriptor = generate_font_descriptor(&font, 99);
        assert!(descriptor.contains("99"));
    }

    #[test]
    fn test_generate_font_stream_header_contains_length() {
        let font = minimal_pdf_font();
        let header = generate_font_stream_header(&font);
        assert!(header.contains("/Length"));
        // font_data is 100 bytes
        assert!(header.contains("100"));
    }

    #[test]
    fn test_generate_font_dictionary_type0() {
        let font = minimal_pdf_font();
        let dict = generate_font_dictionary(&font, 10, Some(15));
        assert!(dict.contains("/Type /Font"));
        assert!(dict.contains("/Subtype /Type0"));
        assert!(dict.contains("TestFont"));
    }

    #[test]
    fn test_generate_font_dictionary_no_to_unicode() {
        let font = minimal_pdf_font();
        let dict = generate_font_dictionary(&font, 10, None);
        // Without ToUnicode, /ToUnicode should be absent
        assert!(!dict.contains("/ToUnicode"));
    }

    #[test]
    fn test_generate_to_unicode_cmap_identity_range() {
        let mut font = minimal_pdf_font();
        // With empty char_to_glyph and a small range it uses identity mapping
        font.first_char = 65; // 'A'
        font.last_char = 67; // 'C'
        font.widths = vec![500; 3];
        let cmap = generate_to_unicode_cmap(&font);
        assert!(cmap.contains("begincmap"));
        assert!(cmap.contains("endcmap"));
        assert!(cmap.contains("<0041> <0041>")); // 'A' -> 'A'
        assert!(cmap.contains("<0042> <0042>")); // 'B' -> 'B'
    }

    #[test]
    fn test_generate_to_unicode_cmap_with_char_map() {
        let mut font = minimal_pdf_font();
        font.char_to_glyph.insert('A', 100);
        font.char_to_glyph.insert('Z', 200);
        let cmap = generate_to_unicode_cmap(&font);
        assert!(cmap.contains("begincmap"));
        assert!(cmap.contains("beginbfchar"));
        // 'A' = 0x0041 → 0x0041
        assert!(cmap.contains("<0041> <0041>"));
    }

    #[test]
    fn test_generate_font_objects_empty_manager() {
        let manager = FontManager::new();
        let objects = manager
            .generate_font_objects(10)
            .expect("test: should succeed");
        assert!(objects.is_empty());
    }
}
