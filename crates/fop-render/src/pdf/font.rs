//! TrueType font embedding support for PDF
//!
//! Provides functionality to embed TrueType/OpenType fonts in PDF documents.
//!
//! ## CID-assignment invariant
//!
//! All CID-keyed composite fonts produced by this module follow a single, strict
//! invariant that covers every character including astral (non-BMP) code points:
//!
//! > **CID == original TrueType glyph ID** (the GID as returned by
//! > `ttf_parser::Face::glyph_index`, before any subsetting renumbering).
//!
//! Consequences of the invariant:
//! - The 2-byte code written into the PDF content stream for a character is
//!   always its original GID.  Because original GIDs are `u16` values, every
//!   code fits in two bytes — no surrogate pairs appear in the content stream.
//! - `CIDToGIDMap[original_gid]` is the *new* (post-subset, renumbered) glyph
//!   ID, so the PDF renderer follows `CID → original GID → new GID → outline`.
//! - The `/W` widths array is keyed by original GID.
//! - The `ToUnicode` CMap maps each original GID to its Unicode scalar.  For
//!   astral characters (U+10000…U+10FFFF) the destination string is the
//!   4-byte UTF-16BE surrogate pair, e.g. `<D83DDE00>` for U+1F600 (😀).
//!
//! This invariant makes BMP and non-BMP characters uniformly consistent: there
//! is no special-casing in any of the five places (content stream, CIDToGIDMap,
//! /W, ToUnicode, CIDToGIDMap stream) that name a CID.

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

    /// Character to *new* (post-subset) glyph ID.
    ///
    /// Populated by `create_subset_font` from the subsetter's GID remapper.
    /// Used to build the `CIDToGIDMap` stream (see module-level CID invariant).
    pub char_to_glyph: std::collections::HashMap<char, u16>,

    /// Character to *original* (pre-subset) glyph ID.
    ///
    /// This is the CID used in the PDF content stream and as the index into
    /// the `CIDToGIDMap` stream and the `/W` widths array.  Original GIDs are
    /// always ≤ 65 535 (u16), so they always fit in the 2-byte CID space.
    /// Populated by `create_subset_font`; empty for non-subsetted fonts.
    pub char_to_orig_glyph: std::collections::HashMap<char, u16>,
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
        let char_to_orig_glyph = std::collections::HashMap::new();

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
            char_to_orig_glyph,
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

    /// Encode `text` as a PDF hex string using original glyph IDs as CIDs.
    ///
    /// Each character's 2-byte CID is its original (pre-subset) TrueType glyph
    /// ID, looked up from `char_to_orig_glyph`.  This is consistent with the
    /// module-level CID invariant.
    ///
    /// If a character is not found in `char_to_orig_glyph` (which can happen for
    /// non-subsetted fonts or characters added after subsetting), the method
    /// falls back to parsing the embedded font data on the spot to look up the
    /// original GID.  Characters not present in the font at all are encoded as
    /// CID 0 (`.notdef`).
    ///
    /// The returned string is a PDF hex string: `<XXXX...>` where each pair of
    /// hex digits is one byte (2 hex digits per byte, 4 hex digits per CID).
    pub fn encode_text_with_glyph_ids(&self, text: &str) -> String {
        // Parse the face once per call (cheap on modern hardware for short text,
        // and we avoid storing the non-Send ttf_parser::Face on PdfFont itself).
        let face_opt = ttf_parser::Face::parse(&self.font_data, 0).ok();

        let mut result = String::from("<");
        for c in text.chars() {
            // First try the pre-built map, then fall back to the live font parse.
            let orig_gid: u16 = if let Some(&gid) = self.char_to_orig_glyph.get(&c) {
                gid
            } else if let Some(ref face) = face_opt {
                face.glyph_index(c).map(|gid| gid.0).unwrap_or(0)
            } else {
                0
            };
            // Write the 2-byte CID as 4 hex digits (big-endian).
            result.push_str(&format!("{:04X}", orig_gid));
        }
        result.push('>');
        result
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

/// Generate CIDFont Type 2 dictionary (TrueType descendant font).
///
/// The `/W` widths array is keyed by **original GID** (the CID per the
/// module-level invariant).  When `char_to_orig_glyph` is populated (subsetted
/// font), the array is built sparsely from that map so it stays compact even if
/// used characters are spread across a wide GID range or include non-BMP glyphs.
/// For un-subsetted fonts the existing dense array starting at `first_char` is
/// kept for backward compatibility.
pub fn generate_cidfont_dict(
    font: &PdfFont,
    descriptor_obj_id: usize,
    cidtogidmap_obj_id: usize,
) -> String {
    let default_width = font.units_per_em / 2;

    // Build a sparse W array keyed by original GID.  Each entry is written in
    // the PDF form  `<gid> [<width>]`  (single-glyph sub-array), which is
    // always valid and avoids large contiguous ranges for sparse mappings.
    let w_array = if !font.char_to_orig_glyph.is_empty() {
        // Parse the face to read advance widths by original GID.
        let face_opt = ttf_parser::Face::parse(&font.font_data, 0).ok();

        let mut entries: Vec<(u16, u16)> = font
            .char_to_orig_glyph
            .iter()
            .map(|(&c, &orig_gid)| {
                let width = if let Some(ref face) = face_opt {
                    let glyph_id = face.glyph_index(c).unwrap_or(ttf_parser::GlyphId(0));
                    face.glyph_hor_advance(glyph_id)
                        .unwrap_or(font.units_per_em / 2)
                } else {
                    font.units_per_em / 2
                };
                (orig_gid, width)
            })
            .collect();

        // Sort by GID for deterministic, verifiable output.
        entries.sort_by_key(|&(gid, _)| gid);

        let mut s = String::new();
        for (gid, width) in entries {
            if !s.is_empty() {
                s.push(' ');
            }
            s.push_str(&format!("{} [{}]", gid, width));
        }
        s
    } else if !font.widths.is_empty() {
        // Fallback: dense array starting at first_char (BMP-only, non-subsetted).
        let mut s = format!("{} [", font.first_char);
        for (i, width) in font.widths.iter().enumerate() {
            if i > 0 {
                s.push(' ');
            }
            s.push_str(&width.to_string());
        }
        s.push(']');
        s
    } else {
        String::new()
    };

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

/// Encode a Unicode scalar value as a UTF-16BE hex string for use in a ToUnicode
/// CMap destination.
///
/// BMP characters (U+0000..=U+FFFF) produce a 4-hex-digit string (`XXXX`).
/// Astral characters (U+10000..=U+10FFFF) produce the 8-hex-digit surrogate pair
/// string (`HHHHLLLL`), as required by PDF 1.7 §9.10.3 and the UTF-16BE
/// encoding of supplementary characters.
fn unicode_to_utf16be_hex(c: char) -> String {
    let cp = c as u32;
    if cp <= 0xFFFF {
        format!("{:04X}", cp)
    } else {
        // Surrogate-pair encoding (UTF-16BE).
        let cp_shifted = cp - 0x10000;
        let high: u32 = 0xD800 + (cp_shifted >> 10);
        let low: u32 = 0xDC00 + (cp_shifted & 0x3FF);
        format!("{:04X}{:04X}", high, low)
    }
}

/// Generate a ToUnicode CMap for CID fonts.
///
/// Implements the module-level CID invariant: for each used character the CID
/// is the character's *original* TrueType glyph ID (`char_to_orig_glyph`).
/// The mapping is therefore `<orig_gid> <utf16be_unicode>`.
///
/// For astral characters (U+10000..=U+10FFFF) the destination value is an
/// 8-hex-digit UTF-16BE surrogate pair (e.g. `<D83DDE00>` for U+1F600 😀),
/// as specified in PDF 1.7 §9.10.3 — conforming viewers decode this back to the
/// full scalar.
///
/// When no `char_to_orig_glyph` mapping is available (un-subsetted font) the
/// function falls back to a compact BMP identity mapping so the CMap is never
/// empty.
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

    // Use char_to_orig_glyph when available (subsetted font path).
    // The CID is the original GID (a u16 ≤ 0xFFFF); the destination is the
    // UTF-16BE encoding of the Unicode scalar.
    if !font.char_to_orig_glyph.is_empty() {
        let mapping_count = font.char_to_orig_glyph.len();
        cmap.push_str(&format!("{} beginbfchar\n", mapping_count));

        // Sort by original GID for deterministic output.
        let mut entries: Vec<(u16, char)> = font
            .char_to_orig_glyph
            .iter()
            .map(|(&c, &orig_gid)| (orig_gid, c))
            .collect();
        entries.sort_by_key(|&(gid, _)| gid);

        for (orig_gid, c) in entries {
            let dest = unicode_to_utf16be_hex(c);
            cmap.push_str(&format!("<{:04X}> <{}>\n", orig_gid, dest));
        }

        cmap.push_str("endbfchar\n");
    } else if !font.char_to_glyph.is_empty() {
        // Intermediate path: char_to_glyph holds new GIDs but no orig map.
        // Treat new GID as CID (identity post-subset, always BMP-safe).
        let mapping_count = font.char_to_glyph.len();
        cmap.push_str(&format!("{} beginbfchar\n", mapping_count));

        let mut entries: Vec<(u16, char)> = font
            .char_to_glyph
            .iter()
            .map(|(&c, &new_gid)| (new_gid, c))
            .collect();
        entries.sort_by_key(|&(gid, _)| gid);

        for (new_gid, c) in entries {
            let dest = unicode_to_utf16be_hex(c);
            cmap.push_str(&format!("<{:04X}> <{}>\n", new_gid, dest));
        }

        cmap.push_str("endbfchar\n");
    } else {
        // Fallback: BMP identity mapping for the font's character range.
        // Only applies to non-subsetted fonts with a compact BMP range.
        let range_size = font
            .last_char
            .saturating_sub(font.first_char)
            .saturating_add(1) as usize;
        if range_size > 0 && range_size <= 256 && font.last_char <= 0xFFFF {
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

    // Resolve every used character to its glyph in the *original* font. This set
    // drives the subsetter; `.notdef` (glyph 0) is always retained so that any
    // unmapped CID still renders the font's own missing-glyph box.
    let mut used_glyphs = BTreeSet::new();
    used_glyphs.insert(ttf_parser::GlyphId(0));
    for &c in used_chars.iter() {
        if let Some(glyph_id) = face.glyph_index(c) {
            used_glyphs.insert(glyph_id);
        }
    }

    // Produce the real, strictly-smaller subset. `subsetter` renumbers the
    // retained glyphs into a new contiguous space and hands back an
    // original-GID -> new-GID map (see `font_subset` for the full rationale).
    // Everything the PDF says about glyph identity — the `CIDToGIDMap` stream
    // (generated downstream from `char_to_glyph`) and the embedded `cmap` — must
    // therefore speak this *new* glyph space.
    let subset = crate::pdf::font_subset::subset_font(&original_font.font_data, &used_glyphs)?;

    // CID first/last range (kept for metadata, but the actual CID space now
    // uses original GIDs — see module-level invariant).
    let first_char = used_chars.iter().next().map(|&c| c as u32).unwrap_or(0);
    let last_char = used_chars
        .iter()
        .next_back()
        .map(|&c| c as u32)
        .unwrap_or(0xFFFF);

    // char → NEW glyph ID (post-subset).
    // This feeds the `CIDToGIDMap` stream: `CIDToGIDMap[orig_gid] = new_gid`.
    let mut char_to_glyph_map: HashMap<char, u16> = HashMap::new();

    // char → ORIGINAL glyph ID.
    // This is the CID used in the content stream, `/W`, and the `ToUnicode` CMap.
    // Original GIDs are always u16 (≤ 65 535), so they always fit in the 2-byte
    // CID space — including for astral characters (U+10000..=U+10FFFF).
    let mut char_to_orig_glyph_map: HashMap<char, u16> = HashMap::new();

    for &c in used_chars.iter() {
        if let Some(glyph_id) = face.glyph_index(c) {
            let orig_gid = glyph_id.0; // Always a u16
            char_to_orig_glyph_map.insert(c, orig_gid);
            if let Some(&new_gid) = subset.gid_map.get(&orig_gid) {
                char_to_glyph_map.insert(c, new_gid);
            }
        }
    }

    // The widths field is kept for backward compatibility with the BMP dense-array
    // path, but for subsetted fonts the sparse W array in `generate_cidfont_dict`
    // (keyed by `char_to_orig_glyph`) takes precedence and this field is unused.
    let widths: Vec<u16> = Vec::new();

    Ok(PdfFont {
        font_name: original_font.font_name.clone(),
        font_data: subset.data,
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
        char_to_orig_glyph: char_to_orig_glyph_map,
    })
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
            char_to_orig_glyph: HashMap::new(),
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
    fn test_generate_to_unicode_cmap_with_orig_glyph_map() {
        let mut font = minimal_pdf_font();
        // char_to_orig_glyph: 'A' has orig GID 65, 'Z' has orig GID 90.
        font.char_to_orig_glyph.insert('A', 65);
        font.char_to_orig_glyph.insert('Z', 90);
        let cmap = generate_to_unicode_cmap(&font);
        assert!(cmap.contains("begincmap"));
        assert!(cmap.contains("beginbfchar"));
        // CID 65 (orig GID of 'A') → Unicode 'A' (U+0041)
        assert!(cmap.contains("<0041> <0041>"), "CID 0x41 → Unicode 'A'");
        // CID 90 (orig GID of 'Z') → Unicode 'Z' (U+005A)
        assert!(cmap.contains("<005A> <005A>"), "CID 0x5A → Unicode 'Z'");
    }

    #[test]
    fn test_generate_to_unicode_cmap_astral_char() {
        let mut font = minimal_pdf_font();
        // U+1F600 😀 has a hypothetical orig GID 3456.
        // CID should be 3456 (0x0D80); destination should be the surrogate pair
        // D83D DE00 (8 hex digits).
        font.char_to_orig_glyph.insert('\u{1F600}', 3456);
        let cmap = generate_to_unicode_cmap(&font);
        assert!(cmap.contains("begincmap"));
        assert!(cmap.contains("beginbfchar"));
        // CID 3456 = 0x0D80; dest = D83DDE00 (UTF-16BE surrogate pair for U+1F600)
        assert!(
            cmap.contains("<0D80> <D83DDE00>"),
            "astral char must use orig GID as CID and surrogate pair as destination; \
             cmap = {cmap}",
        );
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

/// End-to-end font-subset round-trip tests.
///
/// These build a real PDF that embeds a subsetted font, then load it back with
/// the workspace's own pure-Rust PDF renderer (`fop-pdf-renderer`). Passing them
/// proves three things at once:
///   * the embedded font really was subsetted (the whole PDF is far smaller than
///     the full font), and
///   * the `ToUnicode` CMap stayed consistent (`extract_text` recovers the
///     original string with no `?` / replacement characters), and
///   * the `CIDToGIDMap` stayed consistent with the *remapped* glyph space
///     (rasterising the page actually paints glyph ink rather than empty
///     `.notdef` boxes).
#[cfg(test)]
mod subset_roundtrip_tests {
    use crate::pdf::document::{PdfDocument, PdfPage};
    use fop_pdf_renderer::PdfRenderer;
    use fop_types::Length;

    const DEJAVU_SANS: &str = "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf";

    fn dejavu_bytes() -> Vec<u8> {
        assert!(
            std::path::Path::new(DEJAVU_SANS).exists(),
            "DejaVu Sans not found at {DEJAVU_SANS:?}; install fonts-dejavu-core",
        );
        std::fs::read(DEJAVU_SANS).expect("test: read DejaVu Sans")
    }

    /// Build a single-page PDF that draws `text` with an embedded, subsetted
    /// DejaVu Sans, returning `(pdf_bytes, original_font_len)`.
    fn build_pdf_with_subset(text: &str) -> (Vec<u8>, usize) {
        let font_data = dejavu_bytes();
        let original_len = font_data.len();

        let mut doc = PdfDocument::new();
        let font_index = doc.embed_font(font_data).expect("test: embed font");

        let mut page = PdfPage::new(Length::from_pt(612.0), Length::from_pt(792.0));
        page.add_text_with_font_tracked(
            text,
            Length::from_pt(72.0),
            Length::from_pt(700.0),
            Length::from_pt(24.0),
            font_index,
            &mut doc.font_manager,
        );
        doc.add_page(page);

        let pdf_bytes = doc.to_bytes().expect("test: to_bytes");
        (pdf_bytes, original_len)
    }

    /// (c) Generate a PDF embedding a subset font, load it back through
    /// `fop-pdf-renderer`, and assert the text round-trips uncorrupted.
    #[test]
    fn pdf_embedded_subset_text_roundtrips() {
        let text = "Café Subset 42";
        let (pdf_bytes, original_len) = build_pdf_with_subset(text);

        // The whole PDF must be far smaller than the full font; if subsetting had
        // silently degraded to embedding the full font, the PDF alone would be
        // larger than the 750 KB source.
        assert!(
            pdf_bytes.len() < original_len,
            "PDF ({} bytes) should be smaller than the full font ({} bytes) — \
             was the font actually subsetted?",
            pdf_bytes.len(),
            original_len,
        );

        // Round-trip the bytes through a real temp file (per the project's
        // temp-file policy) before reloading them.
        let mut path = std::env::temp_dir();
        path.push(format!(
            "fop_subset_roundtrip_{}_{}.pdf",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0),
        ));
        std::fs::write(&path, &pdf_bytes).expect("test: write pdf");
        let reloaded = std::fs::read(&path).expect("test: read pdf back");
        let _ = std::fs::remove_file(&path);

        let renderer = PdfRenderer::from_bytes(&reloaded).expect("test: parse generated PDF");
        let extracted = renderer.extract_text(0).expect("test: extract text");

        assert!(
            extracted.contains(text),
            "expected the round-tripped text {text:?}, got {extracted:?}",
        );
        // A broken CID/ToUnicode mapping surfaces as '?' (composite fallback) or
        // the Unicode replacement character — neither must appear.
        assert!(
            !extracted.contains('?') && !extracted.contains('\u{FFFD}'),
            "extracted text contains corruption markers: {extracted:?}",
        );
    }

    /// (c, rendering half) Rasterising the embedded subset must paint real glyph
    /// ink. This exercises the `CID -> CIDToGIDMap -> glyph outline` path against
    /// the *remapped* glyph space; if `char_to_glyph` and the subset bytes spoke
    /// different glyph ID spaces, the page would come out blank (DejaVu's
    /// `.notdef` is an empty glyph).
    #[test]
    fn pdf_embedded_subset_renders_glyph_ink() {
        let (pdf_bytes, _original_len) = build_pdf_with_subset("Hello Subset");

        let renderer = PdfRenderer::from_bytes(&pdf_bytes).expect("test: parse generated PDF");
        let page = renderer.render_page(0, 96.0).expect("test: render page");

        // Count near-black, opaque pixels (the default text fill colour is black).
        let dark_pixels = page
            .pixels
            .chunks_exact(4)
            .filter(|px| px[0] < 96 && px[1] < 96 && px[2] < 96 && px[3] > 0)
            .count();

        assert!(
            dark_pixels > 200,
            "expected glyph ink from the embedded subset, found only {dark_pixels} dark pixels — \
             CIDToGIDMap likely inconsistent with the remapped subset glyph space",
        );
    }

    // ──────────────────────────────────────────────────────────────────────────
    // Non-BMP (astral / supplementary-plane) round-trip tests.
    //
    // These prove the CID invariant holds for characters above U+FFFF:
    //   1. The assigned CID fits in u16 (it is the original TrueType GID).
    //   2. The CID is present in the CIDToGIDMap stream at the correct offset.
    //   3. The ToUnicode CMap maps the CID to the correct Unicode scalar via a
    //      UTF-16BE surrogate-pair destination string.
    //   4. A BMP character in the same font still round-trips without regression.
    //
    // DejaVu Sans is used because it ships on the CI image and contains several
    // characters in the Supplementary Multilingual Plane:
    //   U+10300 𐌀 (Old Italic Letter A) is at GID 5373.
    //   U+1F600 😀 (GRINNING FACE) is at GID 5857.
    // ──────────────────────────────────────────────────────────────────────────

    /// Assert that an astral character (U+10300 𐌀) is assigned a u16-fitting CID
    /// (its original TrueType GID ≤ 65 535), that the CIDToGIDMap stream is indexed
    /// correctly by that CID, and that the ToUnicode CMap contains a surrogate-pair
    /// destination for the astral Unicode scalar.
    #[test]
    fn astral_char_cid_fits_in_u16_and_cidtogidmap_is_correct() {
        // Old Italic Letter A (U+10300) is in DejaVu Sans at GID 5373.
        let astral_char = '\u{10300}'; // 𐌀

        let font_data = dejavu_bytes();
        let original_len = font_data.len();

        let mut doc = PdfDocument::new();
        let font_index = doc.embed_font(font_data).expect("test: embed font");

        let mut page = PdfPage::new(
            fop_types::Length::from_pt(612.0),
            fop_types::Length::from_pt(792.0),
        );
        // Must use the *tracked* API so that char_to_orig_glyph is populated.
        page.add_text_with_font_tracked(
            &astral_char.to_string(),
            fop_types::Length::from_pt(72.0),
            fop_types::Length::from_pt(700.0),
            fop_types::Length::from_pt(24.0),
            font_index,
            &mut doc.font_manager,
        );
        doc.add_page(page);

        // Check that char_to_orig_glyph was populated and fits in u16.
        // The subsetting happens when we call generate_font_objects (inside to_bytes).
        let pdf_bytes = doc.to_bytes().expect("test: to_bytes");

        // The subsetted PDF must be much smaller than the full DejaVu Sans.
        assert!(
            pdf_bytes.len() < original_len,
            "PDF ({} bytes) should be smaller than the original font ({} bytes)",
            pdf_bytes.len(),
            original_len,
        );

        // Parse the PDF bytes as text and assert:
        //   (a) The ToUnicode CMap contains the UTF-16BE surrogate pair for U+10300.
        //       U+10300 = U+10000 + 0x300
        //       high = 0xD800 + (0x300 >> 10) = 0xD800 + 0  = 0xD800
        //       low  = 0xDC00 + (0x300 & 0x3FF) = 0xDC00 + 0x300 = 0xDF00
        //       UTF-16BE hex: D800DF00
        let pdf_str = String::from_utf8_lossy(&pdf_bytes);
        assert!(
            pdf_str.contains("D800DF00"),
            "ToUnicode CMap must contain the UTF-16BE surrogate pair D800DF00 for U+10300; \
             PDF content (first 4000 chars): {}",
            &pdf_str.chars().take(4000).collect::<String>(),
        );

        // (b) The CID written in the content stream is the original GID, not the
        //     raw code point 0x10300 (> 0xFFFF).  The original GID of U+10300 in
        //     DejaVu Sans is 5373 = 0x14FD.  The content stream must contain <14FD>
        //     (the 2-byte CID), not <D800DF00> (surrogate pair) or <010300>.
        assert!(
            pdf_str.contains("<14FD>"),
            "content stream must contain the 2-byte orig GID <14FD> for U+10300; \
             got PDF (snippet): {}",
            &pdf_str.chars().take(4000).collect::<String>(),
        );

        // (c) The code point 0x10300 must NOT appear as a literal 6-hex-digit CID
        //     in the content stream (that would be the old, broken encoding).
        assert!(
            !pdf_str.contains("<010300>"),
            "content stream must NOT contain the raw code point <010300> as CID",
        );
    }

    /// Prove that an astral character (U+10300 𐌀) round-trips through the PDF
    /// text extraction pipeline: extract_text() must return the original Unicode
    /// scalar, not a replacement character or empty string.
    ///
    /// This exercises the full chain:
    ///   content stream (orig GID as CID)
    ///   → CIDToGIDMap (orig GID → new GID)
    ///   → ToUnicode (orig GID → UTF-16BE surrogate pair → char)
    ///   → extracted text
    #[test]
    fn astral_char_roundtrips_through_to_unicode() {
        let astral_char = '\u{10300}'; // 𐌀 Old Italic Letter A (U+10300)
        let bmp_char = 'A'; // Regression guard: BMP must still work.

        let text = format!("{}{}", bmp_char, astral_char);
        let (pdf_bytes, _) = build_pdf_with_subset(&text);

        let renderer = PdfRenderer::from_bytes(&pdf_bytes).expect("test: parse PDF");
        let extracted = renderer.extract_text(0).expect("test: extract text");

        // The astral character must survive the round-trip.
        assert!(
            extracted.contains(astral_char),
            "astral char U+10300 must round-trip through ToUnicode; got: {extracted:?}",
        );
        // The BMP character must still work (regression guard).
        assert!(
            extracted.contains(bmp_char),
            "BMP char 'A' must still round-trip; got: {extracted:?}",
        );
        // Neither char should degrade to a replacement character.
        assert!(
            !extracted.contains('\u{FFFD}') && !extracted.contains('?'),
            "extracted text must not contain corruption markers; got: {extracted:?}",
        );
    }

    /// Prove the CIDToGIDMap stream sizes are u16-bounded for astral chars.
    /// The old code computed `offset = (c as u32) * 2` which for U+10300 gives
    /// 0x20600 bytes ≈ 132 KB per character.  The new code uses the original GID
    /// (5373 for U+10300) → (5373+1)*2 = 10 748 bytes.
    #[test]
    fn astral_char_cidtogidmap_is_u16_bounded() {
        use crate::pdf::cidfont::generate_cidtogidmap_stream;

        // Simulate what create_subset_font produces for U+10300 (orig GID 5373,
        // new GID 1 after subsetting with just this glyph + .notdef).
        let astral_char = '\u{10300}';
        let orig_gid: u16 = 5373;
        let new_gid: u16 = 1;

        let mut char_to_new = std::collections::HashMap::new();
        char_to_new.insert(astral_char, new_gid);
        let mut char_to_orig = std::collections::HashMap::new();
        char_to_orig.insert(astral_char, orig_gid);

        let stream = generate_cidtogidmap_stream(&char_to_new, &char_to_orig);

        // Stream length must be (5373+1)*2 = 10748, NOT (0x10300+1)*2 = 131 842.
        let expected_len = (orig_gid as usize + 1) * 2;
        assert_eq!(
            stream.len(),
            expected_len,
            "CIDToGIDMap stream for U+10300 must be {} bytes (orig GID {}, not raw codepoint)",
            expected_len,
            orig_gid,
        );

        // CIDToGIDMap[orig_gid] = new_gid
        let offset = (orig_gid as usize) * 2;
        assert_eq!(stream[offset], (new_gid >> 8) as u8);
        assert_eq!(stream[offset + 1], (new_gid & 0xFF) as u8);
    }
}
