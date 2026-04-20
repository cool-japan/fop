//! PDF text extraction
//!
//! Walks a PDF content stream and extracts human-readable text without
//! rasterizing anything.  Best-effort: PDFs with obfuscated encodings or
//! missing ToUnicode CMaps may contain '?' placeholders.

use crate::content::{Token, Tokenizer};
use crate::error::Result;
use crate::font::LoadedFont;
use crate::parser::{PdfDocument, PdfPage};
use crate::text::decode_string_to_cids;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Threshold for treating vertical movement as a line break (in text-space units)
// ---------------------------------------------------------------------------

const V_MOVE_NEWLINE_THRESHOLD: f32 = 0.5;

// ---------------------------------------------------------------------------
// TextExtractor
// ---------------------------------------------------------------------------

/// Walks a PDF content stream and collects visible text.
///
/// Fonts are loaded on demand and cached per instance, so the same extractor
/// can be reused across multiple pages of the same document.
pub struct TextExtractor<'a> {
    doc: &'a PdfDocument,
    font_cache: HashMap<String, LoadedFont>,
    /// Currently active font resource name (as set by `Tf`)
    current_font_name: String,
    /// Previous text-matrix `f` component (vertical position).
    /// Used to detect line breaks via `Td`/`TD` vertical moves.
    prev_tm_f: Option<f32>,
}

impl<'a> TextExtractor<'a> {
    /// Create a new extractor bound to the given document.
    pub fn new(doc: &'a PdfDocument) -> Self {
        Self {
            doc,
            font_cache: HashMap::new(),
            current_font_name: String::new(),
            prev_tm_f: None,
        }
    }

    /// Extract the user-visible text from `page`, returned as a `String`.
    ///
    /// `Td`/`TD`/`T*` operators that represent a significant vertical
    /// movement insert a `\n` into the output heuristically.
    pub fn extract_page(&mut self, page: &PdfPage) -> Result<String> {
        let mut out = String::new();

        // Text-matrix `f` component (vertical position in text space).
        // We only track it inside a BT…ET block.
        let mut tm_f: f32 = 0.0;
        // Leading value for T* calculation.
        let mut leading: f32 = 0.0;

        // We need to reproduce the full tokenizer loop instead of reusing
        // ContentInterpreter because that struct requires tiny_skia types
        // (DrawCommand etc.) and we do not want any graphics dependency here.
        let mut tokenizer = Tokenizer::new(&page.content);
        let mut operand_stack: Vec<Token> = Vec::new();
        let mut array_mode = false;
        let mut array_buf: Vec<Token> = Vec::new();

        // Reset per-page state
        self.prev_tm_f = None;

        while let Some(token) = tokenizer.next_token() {
            match &token {
                Token::ArrayOpen => {
                    array_mode = true;
                    array_buf.clear();
                }
                Token::ArrayClose => {
                    array_mode = false;
                    // Flatten: keep StringBytes only, discard Number (kerning).
                    // This mirrors the ContentInterpreter's array handling.
                    let arr = std::mem::take(&mut array_buf);
                    let combined: Vec<u8> = arr
                        .into_iter()
                        .flat_map(|t| match t {
                            Token::StringBytes(b) => b,
                            _ => Vec::new(),
                        })
                        .collect();
                    operand_stack.push(Token::StringBytes(combined));
                }
                Token::Operator(op) => {
                    self.handle_operator(
                        op.as_str(),
                        &mut operand_stack,
                        &mut out,
                        &page.resources,
                        &mut tm_f,
                        &mut leading,
                    );
                    operand_stack.clear();
                }
                _ => {
                    if array_mode {
                        array_buf.push(token);
                    } else {
                        operand_stack.push(token);
                    }
                }
            }
        }

        Ok(out)
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn handle_operator(
        &mut self,
        op: &str,
        stack: &mut Vec<Token>,
        out: &mut String,
        resources: &crate::parser::PdfDictionary,
        tm_f: &mut f32,
        leading: &mut f32,
    ) {
        match op {
            // ---- Text object boundaries ----
            "BT" => {
                *tm_f = 0.0;
                self.prev_tm_f = None;
            }
            "ET" => {}

            // ---- Font selection ----
            "Tf" => {
                let _size = pop_f(stack);
                let name = pop_name(stack);
                self.ensure_font_loaded(&name, resources);
                self.current_font_name = name;
            }

            // ---- Text positioning: detect vertical moves ----
            "Td" | "TD" => {
                let ty = pop_f(stack);
                let _tx = pop_f(stack);
                if op == "TD" {
                    *leading = -ty;
                }
                if ty.abs() > V_MOVE_NEWLINE_THRESHOLD {
                    out.push('\n');
                }
                *tm_f += ty;
            }
            "Tm" => {
                // 6 operands: a b c d e f
                let f = pop_f(stack);
                let _e = pop_f(stack);
                let _d = pop_f(stack);
                let _c = pop_f(stack);
                let _b = pop_f(stack);
                let _a = pop_f(stack);
                if let Some(prev) = self.prev_tm_f {
                    if (f - prev).abs() > V_MOVE_NEWLINE_THRESHOLD {
                        out.push('\n');
                    }
                }
                *tm_f = f;
                self.prev_tm_f = Some(f);
            }
            "T*" => {
                // Move to next line by -leading
                if leading.abs() > V_MOVE_NEWLINE_THRESHOLD {
                    out.push('\n');
                }
            }

            // ---- Text showing ----
            "Tj" => {
                let bytes = pop_str(stack);
                self.decode_and_append(&bytes, out);
            }
            "TJ" => {
                // Array already flattened to a single StringBytes by loop above
                let bytes = pop_str(stack);
                self.decode_and_append(&bytes, out);
            }
            "'" => {
                // T* then Tj
                if leading.abs() > V_MOVE_NEWLINE_THRESHOLD {
                    out.push('\n');
                }
                let bytes = pop_str(stack);
                self.decode_and_append(&bytes, out);
            }
            "\"" => {
                // aw ac string  →  set word/char spacing, T*, Tj
                let bytes = pop_str(stack);
                let _ac = pop_f(stack);
                let _aw = pop_f(stack);
                if leading.abs() > V_MOVE_NEWLINE_THRESHOLD {
                    out.push('\n');
                }
                self.decode_and_append(&bytes, out);
            }

            // ---- Text state parameters (consume stack cleanly) ----
            "Tc" | "Tw" | "Tz" | "TL" | "Ts" | "Tr" => {
                stack.clear();
            }

            // ---- Graphics state (consume silently) ----
            "q" | "Q" => {}
            "cm" => {
                stack.clear();
            }
            "w" | "J" | "j" | "M" | "i" | "ri" => {
                stack.clear();
            }
            "d" => {
                stack.clear();
            }
            "gs" => {
                stack.clear();
            }

            // ---- Color operators ----
            "g" | "G" | "rg" | "RG" | "k" | "K" | "cs" | "CS" | "sc" | "SC" | "scn" | "SCN" => {
                stack.clear();
            }

            // ---- Path and painting (consume silently) ----
            "m" | "l" | "c" | "v" | "y" | "h" | "re" | "S" | "s" | "f" | "F" | "f*" | "B"
            | "B*" | "b" | "b*" | "n" | "W" | "W*" => {
                stack.clear();
            }

            // ---- XObject, inline image, misc ----
            "Do" | "BI" | "EI" | "ID" | "sh" | "BX" | "EX" | "BMC" | "BDC" | "EMC" | "MP"
            | "DP" => {
                stack.clear();
            }

            _ => {
                // Unknown operator — consume silently
                log::trace!("text_extract: unknown operator '{}'", op);
                stack.clear();
            }
        }
    }

    /// Decode `bytes` through the current font (if any) and append to `out`.
    fn decode_and_append(&self, bytes: &[u8], out: &mut String) {
        if bytes.is_empty() {
            return;
        }

        let is_composite = self
            .font_cache
            .get(&self.current_font_name)
            .map(|f| f.subtype == "Type0")
            .unwrap_or(false);

        let cids = decode_string_to_cids(bytes, is_composite);

        for cid in cids {
            let ch = self.resolve_cid_to_char(cid);
            out.push(ch);
        }
    }

    /// Map a CID to a `char`.
    ///
    /// Priority:
    /// 1. ToUnicode CMap via `LoadedFont::cid_to_char`
    /// 2. For simple (non-Type0) fonts: treat CID as a WinAnsi/Latin-1 byte.
    ///    Bytes 0x20–0x7E are printable ASCII. Bytes 0x80–0xFF are Latin-1
    ///    supplement.  Below 0x20 → '?' (control characters).
    /// 3. No font loaded → Latin-1 fallback on CID.
    fn resolve_cid_to_char(&self, cid: u32) -> char {
        // Try the ToUnicode map first
        if let Some(font) = self.font_cache.get(&self.current_font_name) {
            if let Some(ch) = font.cid_to_char(cid) {
                return ch;
            }
            // No ToUnicode entry.  For composite (Type0) fonts we have no
            // reliable fallback — emit '?'.
            if font.subtype == "Type0" {
                return '?';
            }
        }

        // Simple font (or no font loaded): WinAnsi / Latin-1 fallback.
        // `char::from_u32` covers Unicode scalar values; for bytes ≥ 0x80 that
        // is the Latin-1 supplement, which matches WinAnsi for 0xA0–0xFF.
        // Bytes 0x00–0x1F are control codes — map to '?'.
        if cid < 0x20 {
            return '?';
        }
        char::from_u32(cid).unwrap_or('?')
    }

    /// Load `name` into `font_cache` if it is not already cached.
    fn ensure_font_loaded(&mut self, name: &str, resources: &crate::parser::PdfDictionary) {
        if self.font_cache.contains_key(name) {
            return;
        }
        if let Some(font_dict) = self.doc.get_font(resources, name) {
            let loaded = LoadedFont::load(self.doc, &font_dict);
            self.font_cache.insert(name.to_owned(), loaded);
        }
    }
}

// ---------------------------------------------------------------------------
// Operand-stack pop helpers (mirrors those in content.rs)
// ---------------------------------------------------------------------------

fn pop_f(stack: &mut Vec<Token>) -> f32 {
    match stack.pop() {
        Some(Token::Number(n)) => n as f32,
        _ => 0.0,
    }
}

fn pop_str(stack: &mut Vec<Token>) -> Vec<u8> {
    match stack.pop() {
        Some(Token::StringBytes(b)) => b,
        _ => Vec::new(),
    }
}

fn pop_name(stack: &mut Vec<Token>) -> String {
    match stack.pop() {
        Some(Token::Name(n)) => n,
        Some(Token::Operator(n)) => n,
        _ => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::PdfDocument;

    // -----------------------------------------------------------------------
    // Helpers shared by tests
    // -----------------------------------------------------------------------

    /// Build a minimal 1-page PDF whose content stream is `content`.
    /// If `font_dict_entry` is Some, it is inserted into the Resources/Font dict:
    ///   e.g. `"/F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>"`
    fn build_pdf_with_content_and_font(content: &[u8], font_dict_entry: Option<&str>) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();
        out.extend_from_slice(b"%PDF-1.4\n");

        // Object 1: Catalog
        let o1 = out.len();
        out.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

        // Object 2: Pages
        let o2 = out.len();
        out.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

        // Object 4: Content stream
        let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content.len());
        let o4 = out.len();
        out.extend_from_slice(stream_hdr.as_bytes());
        out.extend_from_slice(content);
        out.extend_from_slice(b"\nendstream\nendobj\n");

        // Object 3: Page — includes Resources/Font if requested
        let o3 = out.len();
        let resources_str = if let Some(font_entry) = font_dict_entry {
            format!("/Resources << /Font << {} >> >>", font_entry)
        } else {
            String::new()
        };
        let page_dict = format!(
            "3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R {} >>\nendobj\n",
            resources_str
        );
        out.extend_from_slice(page_dict.as_bytes());

        let xref_pos = out.len();
        out.extend_from_slice(b"xref\n0 5\n");
        out.extend_from_slice(b"0000000000 65535 f \n");
        out.extend_from_slice(format!("{:010} 00000 n \n", o1).as_bytes());
        out.extend_from_slice(format!("{:010} 00000 n \n", o2).as_bytes());
        out.extend_from_slice(format!("{:010} 00000 n \n", o3).as_bytes());
        out.extend_from_slice(format!("{:010} 00000 n \n", o4).as_bytes());
        out.extend_from_slice(b"trailer\n<< /Size 5 /Root 1 0 R >>\n");
        out.extend_from_slice(b"startxref\n");
        out.extend_from_slice(format!("{}\n", xref_pos).as_bytes());
        out.extend_from_slice(b"%%EOF\n");
        out
    }

    fn extract_from_content(content: &[u8], font_entry: Option<&str>) -> String {
        let pdf = build_pdf_with_content_and_font(content, font_entry);
        let doc = PdfDocument::from_bytes(&pdf).expect("parse PDF");
        let page = doc.get_page(0).expect("get page 0");
        let mut extractor = TextExtractor::new(&doc);
        extractor.extract_page(&page).expect("extract_page")
    }

    // -----------------------------------------------------------------------
    // Test: empty content stream returns ""
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_text_returns_empty_for_blank_page() {
        let text = extract_from_content(b"", None);
        assert!(
            text.is_empty(),
            "blank page should produce empty string, got {:?}",
            text
        );
    }

    // -----------------------------------------------------------------------
    // Test: BT/ET with no text operators returns ""
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_text_bt_et_only_returns_empty() {
        let text = extract_from_content(b"BT ET", None);
        assert!(
            text.is_empty(),
            "BT/ET without Tf/Tj should be empty, got {:?}",
            text
        );
    }

    // -----------------------------------------------------------------------
    // Test: Tj with WinAnsi font — basic ASCII round-trip
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_text_tj_winанsi_font() {
        // (Hello) Tj — using WinAnsi fallback (no ToUnicode CMap)
        let content = b"BT /F1 12 Tf (Hello) Tj ET";
        let font_entry = "/F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>";
        let text = extract_from_content(content, Some(font_entry));
        assert!(
            text.contains("Hello"),
            "Expected 'Hello' in text, got {:?}",
            text
        );
    }

    // -----------------------------------------------------------------------
    // Test: TJ array handling — strings extracted, integer kerning skipped
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_text_from_tj_array() {
        // [(He) -20 (ll) 10 (o)] TJ
        // The tokenizer flattens this to b"Hello" before the TJ operator fires.
        let content = b"BT /F1 12 Tf [(He) -20 (ll) 10 (o)] TJ ET";
        let font_entry = "/F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>";
        let text = extract_from_content(content, Some(font_entry));
        assert!(
            text.contains("Hello"),
            "TJ array should decode to 'Hello', got {:?}",
            text
        );
    }

    // -----------------------------------------------------------------------
    // Test: apostrophe operator (T* + Tj)
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_text_apostrophe_operator() {
        // ' is T* then Tj — with leading=14 it should insert a newline
        let content = b"BT /F1 12 Tf 14 TL (First) Tj (Second) ' ET";
        let font_entry = "/F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>";
        let text = extract_from_content(content, Some(font_entry));
        assert!(
            text.contains("First") && text.contains("Second"),
            "' operator: expected both strings in output, got {:?}",
            text
        );
    }

    // -----------------------------------------------------------------------
    // Test: missing ToUnicode CMap — graceful fallback, no panic
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_text_handles_missing_to_unicode() {
        // Font with no ToUnicode CMap.  Bytes 0x48='H', 0x69='i' → ASCII.
        let content = b"BT /F1 12 Tf (Hi) Tj ET";
        let font_entry = "/F1 << /Type /Font /Subtype /Type1 /BaseFont /Courier >>";
        // Must not panic; should return best-effort text
        let result = {
            let pdf = build_pdf_with_content_and_font(content, Some(font_entry));
            let doc = PdfDocument::from_bytes(&pdf).expect("parse");
            let page = doc.get_page(0).expect("page");
            let mut ex = TextExtractor::new(&doc);
            ex.extract_page(&page)
        };
        assert!(result.is_ok(), "should not error on missing ToUnicode");
        let text = result.expect("extract_page");
        // 'H' (0x48) and 'i' (0x69) are printable ASCII — fallback should work
        assert!(
            text.contains('H') && text.contains('i'),
            "WinAnsi fallback: expected 'Hi', got {:?}",
            text
        );
    }

    // -----------------------------------------------------------------------
    // Test: composite (Type0) font with no ToUnicode emits '?'
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_text_type0_no_to_unicode_emits_question_marks() {
        // Two 2-byte CIDs → two '?' because no ToUnicode
        let content = b"BT /F1 12 Tf <00410042> Tj ET";
        let font_entry = "/F1 << /Type /Font /Subtype /Type0 /BaseFont /HiraginoSans /Encoding /Identity-H /DescendantFonts [<< /Type /Font /Subtype /CIDFontType2 /BaseFont /HiraginoSans >>] >>";
        let text = extract_from_content(content, Some(font_entry));
        // Each 2-byte CID with no ToUnicode → '?'
        assert!(
            text.chars().all(|c| c == '?'),
            "Type0 without ToUnicode should emit only '?', got {:?}",
            text
        );
        assert_eq!(text.len(), 2, "Two CIDs should give two '?' chars");
    }

    // -----------------------------------------------------------------------
    // Test: Td vertical movement inserts newline
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_text_td_vertical_inserts_newline() {
        // 0 -14 Td moves down — should insert '\n' heuristically
        let content = b"BT /F1 12 Tf (Line1) Tj 0 -14 Td (Line2) Tj ET";
        let font_entry = "/F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>";
        let text = extract_from_content(content, Some(font_entry));
        assert!(
            text.contains("Line1") && text.contains("Line2"),
            "expected both lines, got {:?}",
            text
        );
        assert!(
            text.contains('\n'),
            "Td with ty=-14 should insert newline, got {:?}",
            text
        );
    }

    // -----------------------------------------------------------------------
    // Test: horizontal-only Td does NOT insert newline
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_text_horizontal_td_no_newline() {
        // 10 0 Td — only horizontal movement, no newline expected
        let content = b"BT /F1 12 Tf (A) Tj 10 0 Td (B) Tj ET";
        let font_entry = "/F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>";
        let text = extract_from_content(content, Some(font_entry));
        assert!(
            !text.contains('\n'),
            "horizontal Td should not insert newline, got {:?}",
            text
        );
        assert!(
            text.contains('A') && text.contains('B'),
            "expected both chars, got {:?}",
            text
        );
    }

    // -----------------------------------------------------------------------
    // Test: non-text content (path operators) does not pollute text output
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_text_ignores_path_operators() {
        let content = b"0 0 0 rg 10 10 100 100 re f BT /F1 12 Tf (OK) Tj ET";
        let font_entry = "/F1 << /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>";
        let text = extract_from_content(content, Some(font_entry));
        assert_eq!(
            text, "OK",
            "path operators should not appear in text output, got {:?}",
            text
        );
    }

    // -----------------------------------------------------------------------
    // Test: no font set — bytes decoded via WinAnsi fallback
    // -----------------------------------------------------------------------

    #[test]
    fn test_extract_text_no_font_winанsi_fallback() {
        // No Tf — still should not panic, return Latin-1 decoded chars
        let content = b"BT (Hi) Tj ET";
        let text = extract_from_content(content, None);
        // 'H' = 0x48, 'i' = 0x69 — WinAnsi fallback
        assert!(
            text.contains('H') && text.contains('i'),
            "no-font fallback: expected 'Hi', got {:?}",
            text
        );
    }
}
