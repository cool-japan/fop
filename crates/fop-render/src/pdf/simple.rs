//! Simple programmatic PDF document builder.
//!
//! Provides a high-level builder for generating text-only A4 PDF documents
//! without requiring an area tree or XSL-FO pipeline. Suitable for audit logs,
//! verification reports, and other programmatically-generated documents.

use std::collections::HashSet;

use crate::pdf::document::types::{PdfDocument, PdfPage};
use fop_types::Length;

/// The 14 standard PDF Type1 builtin fonts.
///
/// These fonts are guaranteed to be available in all PDF readers without embedding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinFont {
    /// Helvetica (sans-serif)
    Helvetica,
    /// Helvetica Bold
    HelveticaBold,
    /// Helvetica Oblique
    HelveticaOblique,
    /// Helvetica Bold Oblique
    HelveticaBoldOblique,
    /// Times Roman (serif)
    TimesRoman,
    /// Times Bold
    TimesBold,
    /// Times Italic
    TimesItalic,
    /// Times Bold Italic
    TimesBoldItalic,
    /// Courier (monospace)
    Courier,
    /// Courier Bold
    CourierBold,
    /// Courier Oblique
    CourierOblique,
    /// Courier Bold Oblique
    CourierBoldOblique,
    /// Symbol font
    Symbol,
    /// Zapf Dingbats font
    ZapfDingbats,
}

impl BuiltinFont {
    /// Returns the PDF font resource name (e.g. `/F1`) for use in content streams.
    fn resource_name(self) -> &'static str {
        match self {
            Self::Helvetica => "F1",
            Self::HelveticaBold => "F2",
            Self::HelveticaOblique => "F3",
            Self::HelveticaBoldOblique => "F4",
            Self::TimesRoman => "F5",
            Self::TimesBold => "F6",
            Self::TimesItalic => "F7",
            Self::TimesBoldItalic => "F8",
            Self::Courier => "F9",
            Self::CourierBold => "F10",
            Self::CourierOblique => "F11",
            Self::CourierBoldOblique => "F12",
            Self::Symbol => "F13",
            Self::ZapfDingbats => "F14",
        }
    }

    /// Returns the PDF BaseFont name (e.g. `Helvetica`).
    fn base_font_name(self) -> &'static str {
        match self {
            Self::Helvetica => "Helvetica",
            Self::HelveticaBold => "Helvetica-Bold",
            Self::HelveticaOblique => "Helvetica-Oblique",
            Self::HelveticaBoldOblique => "Helvetica-BoldOblique",
            Self::TimesRoman => "Times-Roman",
            Self::TimesBold => "Times-Bold",
            Self::TimesItalic => "Times-Italic",
            Self::TimesBoldItalic => "Times-BoldItalic",
            Self::Courier => "Courier",
            Self::CourierBold => "Courier-Bold",
            Self::CourierOblique => "Courier-Oblique",
            Self::CourierBoldOblique => "Courier-BoldOblique",
            Self::Symbol => "Symbol",
            Self::ZapfDingbats => "ZapfDingbats",
        }
    }

    /// All variants in definition order, used for generating font objects.
    fn all() -> &'static [BuiltinFont] {
        &[
            Self::Helvetica,
            Self::HelveticaBold,
            Self::HelveticaOblique,
            Self::HelveticaBoldOblique,
            Self::TimesRoman,
            Self::TimesBold,
            Self::TimesItalic,
            Self::TimesBoldItalic,
            Self::Courier,
            Self::CourierBold,
            Self::CourierOblique,
            Self::CourierBoldOblique,
            Self::Symbol,
            Self::ZapfDingbats,
        ]
    }
}

/// Convert mm to PDF points (1 pt = 1/72 inch, 1 inch = 25.4 mm).
#[inline]
fn mm_to_pt(mm: f32) -> f32 {
    mm * 72.0 / 25.4
}

/// Escape a string for use in a PDF literal string `(...)`.
///
/// Parentheses, backslashes, and non-printable characters must be escaped.
fn escape_pdf_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '(' => out.push_str("\\("),
            ')' => out.push_str("\\)"),
            '\\' => out.push_str("\\\\"),
            '\r' => out.push_str("\\r"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c if c.is_ascii() => out.push(c),
            // Non-ASCII: use octal escape for each byte
            c => {
                let mut buf = [0u8; 4];
                let encoded = c.encode_utf8(&mut buf);
                for byte in encoded.bytes() {
                    out.push_str(&format!("\\{:03o}", byte));
                }
            }
        }
    }
    out
}

/// Internal per-page state accumulating a raw PDF content stream.
struct PageState {
    content: Vec<u8>,
}

impl PageState {
    fn new() -> Self {
        Self {
            content: Vec::new(),
        }
    }

    /// Append a text item at the given absolute position (PDF points, bottom-left origin).
    fn add_text(&mut self, text: &str, size_pt: f32, x_pt: f32, y_pt: f32, font: BuiltinFont) {
        let escaped = escape_pdf_string(text);
        let op = format!(
            "BT\n/{} {} Tf\n{} {} Td\n({}) Tj\nET\n",
            font.resource_name(),
            size_pt,
            x_pt,
            y_pt,
            escaped,
        );
        self.content.extend_from_slice(op.as_bytes());
    }
}

/// High-level builder for simple text-only A4 PDF documents.
///
/// Does NOT require an area tree or FO pipeline — suitable for reports
/// generated programmatically (audit logs, verification reports, etc.).
///
/// # Example
/// ```no_run
/// use fop_render::pdf::simple::{BuiltinFont, SimpleDocumentBuilder};
///
/// let mut builder = SimpleDocumentBuilder::new("My Report");
/// builder.text("Hello, world!", 12.0, 20.0, 280.0, BuiltinFont::Helvetica);
/// let bytes = builder.save();
/// assert!(bytes.starts_with(b"%PDF-"));
/// ```
pub struct SimpleDocumentBuilder {
    title: String,
    /// Pages accumulated so far (each is a complete `PageState`).
    completed_pages: Vec<PageState>,
    /// The page currently being written to.
    current_page: PageState,
    /// Set of fonts actually used across all pages (for resource generation).
    used_fonts: HashSet<BuiltinFont>,
}

impl SimpleDocumentBuilder {
    /// Create a new builder for a document with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            completed_pages: Vec::new(),
            current_page: PageState::new(),
            used_fonts: HashSet::new(),
        }
    }

    /// Write text at an absolute position on the current page.
    ///
    /// Coordinates are in millimetres from the bottom-left corner of the page
    /// (standard PDF coordinate system). The font size is in PDF points.
    pub fn text(&mut self, text: &str, size_pt: f32, x_mm: f32, y_mm: f32, font: BuiltinFont) {
        self.used_fonts.insert(font);
        let x_pt = mm_to_pt(x_mm);
        let y_pt = mm_to_pt(y_mm);
        self.current_page.add_text(text, size_pt, x_pt, y_pt, font);
    }

    /// Finalise the current page and start a new blank page.
    ///
    /// The current page is preserved even if it is empty, matching the behaviour
    /// expected by callers that create explicit page-break points.
    pub fn new_page(&mut self) {
        let finished = std::mem::replace(&mut self.current_page, PageState::new());
        self.completed_pages.push(finished);
    }

    /// Serialise all pages to a minimal valid PDF 1.4 byte stream.
    ///
    /// The current (last) page is automatically finalised. If no calls to
    /// `text()` or `new_page()` have been made the resulting PDF will contain
    /// a single empty page, which is valid.
    pub fn save(mut self) -> Vec<u8> {
        // Push the final page (may be empty, that is still a valid page)
        self.completed_pages.push(self.current_page);

        // Destructure to avoid partial-move issues in the loop below.
        let SimpleDocumentBuilder {
            title,
            completed_pages,
            current_page: _,
            used_fonts,
        } = self;

        // Build a PdfDocument using the existing serialiser. We create one
        // PdfPage per accumulated PageState and let PdfDocument::to_bytes()
        // handle the cross-reference table, trailer, etc.
        let mut doc = PdfDocument::new();
        doc.info.title = Some(title.clone());

        for page_state in completed_pages {
            let mut pdf_page = PdfPage::new(
                Length::from_mm(210.0),
                Length::from_mm(297.0),
            );
            // Append our raw content stream directly into the page content.
            pdf_page.content.extend_from_slice(&page_state.content);
            doc.add_page(pdf_page);
        }

        // Generate font resource objects for every builtin font that was used.
        // The existing PdfDocument serialiser always emits F1=Helvetica as the
        // sole builtin font resource (object 3). We need to emit the additional
        // fonts so the content streams can reference them. Since PdfDocument
        // doesn't natively support multiple builtin fonts we write our own
        // minimal serialiser that is self-contained.

        // Decide whether we need multi-font support: if only Helvetica (F1) is
        // used, we can delegate entirely to the existing serialiser and avoid
        // duplicating serialisation logic.
        let needs_extra_fonts = used_fonts
            .iter()
            .any(|f| *f != BuiltinFont::Helvetica);

        if needs_extra_fonts {
            // Write our own minimal PDF that supports all 14 builtin fonts.
            write_minimal_pdf(doc, &title)
        } else {
            // Fast path: let the existing, well-tested serialiser handle it.
            match doc.to_bytes() {
                Ok(bytes) => bytes,
                Err(_) => write_minimal_pdf_fallback(),
            }
        }
    }

    /// Returns the page height in millimetres (always 297 mm for A4).
    pub fn page_height_mm(&self) -> f32 {
        297.0
    }

}

// ── Module-level helpers ──────────────────────────────────────────────────────

/// Write a minimal but complete PDF 1.4 file supporting all 14 builtin fonts.
///
/// Object layout:
///   1  – Catalog
///   2  – Pages (page tree root)
///   3–16 – Font dictionaries (one per builtin font F1–F14)
///   17..(17 + N*2 - 1) – Page object + content stream pairs
fn write_minimal_pdf(doc: PdfDocument, title: &str) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    let mut xref_offsets: Vec<usize> = Vec::new();

    // Header
    bytes.extend_from_slice(b"%PDF-1.4\n");
    bytes.extend_from_slice(b"%\xE2\xE3\xCF\xD3\n"); // Binary-safe comment

    // Object 0: always free
    xref_offsets.push(0);

    // Decide which fonts actually need resource objects. Even if only a
    // subset is used we number them consistently so that the resource names
    // in content streams (F1, F2, …) always match.
    let all_fonts = BuiltinFont::all();
    let num_fonts = all_fonts.len(); // 14

    // First page object id = 1 (catalog) + 1 (pages) + 14 (fonts) + 1 = 17
    let first_page_obj_id = 3 + num_fonts; // object IDs are 1-based

    // Object 1: Catalog
    xref_offsets.push(bytes.len());
    bytes.extend_from_slice(b"1 0 obj\n<<\n/Type /Catalog\n/Pages 2 0 R\n>>\nendobj\n");

    // Object 2: Pages
    xref_offsets.push(bytes.len());
    bytes.extend_from_slice(b"2 0 obj\n<<\n/Type /Pages\n/Kids [");
    let page_count = doc.pages.len();
    for i in 0..page_count {
        let page_id = first_page_obj_id + i * 2;
        bytes.extend_from_slice(format!("{} 0 R ", page_id).as_bytes());
    }
    bytes.extend_from_slice(format!("]\n/Count {}\n>>\nendobj\n", page_count).as_bytes());

    // Objects 3..16: Font resource dictionaries (F1..F14)
    for (idx, font) in all_fonts.iter().enumerate() {
        let obj_id = 3 + idx;
        xref_offsets.push(bytes.len());
        bytes.extend_from_slice(format!("{} 0 obj\n", obj_id).as_bytes());
        bytes.extend_from_slice(b"<<\n/Type /Font\n/Subtype /Type1\n");
        bytes.extend_from_slice(
            format!("/BaseFont /{}\n", font.base_font_name()).as_bytes(),
        );
        bytes.extend_from_slice(b">>\nendobj\n");
    }

    // Build font resource dictionary string (referenced by every page)
    let mut font_resources = String::from("/Font <<\n");
    for (idx, font) in all_fonts.iter().enumerate() {
        let obj_id = 3 + idx;
        font_resources.push_str(&format!(
            "  /{} {} 0 R\n",
            font.resource_name(),
            obj_id
        ));
    }
    font_resources.push_str(">>\n");

    // Page objects + content streams
    for (page_idx, page) in doc.pages.iter().enumerate() {
        let page_obj_id = first_page_obj_id + page_idx * 2;
        let content_obj_id = page_obj_id + 1;

        // Page dictionary
        xref_offsets.push(bytes.len());
        bytes.extend_from_slice(format!("{} 0 obj\n", page_obj_id).as_bytes());
        bytes.extend_from_slice(b"<<\n/Type /Page\n/Parent 2 0 R\n");
        bytes.extend_from_slice(
            format!(
                "/MediaBox [0 0 {} {}]\n",
                page.width.to_pt(),
                page.height.to_pt()
            )
            .as_bytes(),
        );
        bytes.extend_from_slice(b"/Resources <<\n");
        bytes.extend_from_slice(font_resources.as_bytes());
        bytes.extend_from_slice(b">>\n");
        bytes.extend_from_slice(
            format!("/Contents {} 0 R\n", content_obj_id).as_bytes(),
        );
        bytes.extend_from_slice(b">>\nendobj\n");

        // Content stream
        xref_offsets.push(bytes.len());
        bytes.extend_from_slice(format!("{} 0 obj\n", content_obj_id).as_bytes());
        bytes.extend_from_slice(
            format!("<<\n/Length {}\n>>\nstream\n", page.content.len()).as_bytes(),
        );
        bytes.extend_from_slice(&page.content);
        bytes.extend_from_slice(b"\nendstream\nendobj\n");
    }

    // Title in Info dict
    let info_obj_id = first_page_obj_id + page_count * 2;
    let has_title = !title.is_empty();
    if has_title {
        xref_offsets.push(bytes.len());
        bytes.extend_from_slice(format!("{} 0 obj\n", info_obj_id).as_bytes());
        bytes.extend_from_slice(b"<<\n");
        bytes.extend_from_slice(
            format!("/Title ({})\n", escape_pdf_string(title)).as_bytes(),
        );
        bytes.extend_from_slice(b">>\nendobj\n");
    }

    // Cross-reference table
    let xref_offset = bytes.len();
    bytes.extend_from_slice(b"xref\n");
    bytes.extend_from_slice(format!("0 {}\n", xref_offsets.len()).as_bytes());
    bytes.extend_from_slice(b"0000000000 65535 f \n");
    for offset in xref_offsets.iter().skip(1) {
        bytes.extend_from_slice(format!("{:010} 00000 n \n", offset).as_bytes());
    }

    // Trailer
    bytes.extend_from_slice(b"trailer\n<<\n");
    bytes.extend_from_slice(format!("/Size {}\n", xref_offsets.len()).as_bytes());
    bytes.extend_from_slice(b"/Root 1 0 R\n");
    if has_title {
        bytes.extend_from_slice(format!("/Info {} 0 R\n", info_obj_id).as_bytes());
    }
    bytes.extend_from_slice(b">>\nstartxref\n");
    bytes.extend_from_slice(format!("{}\n", xref_offset).as_bytes());
    bytes.extend_from_slice(b"%%EOF\n");

    bytes
}

/// Produce a minimal valid (but empty) PDF – last-resort fallback.
fn write_minimal_pdf_fallback() -> Vec<u8> {
    b"%PDF-1.4\n1 0 obj\n<<\n/Type /Catalog\n/Pages 2 0 R\n>>\nendobj\n\
2 0 obj\n<<\n/Type /Pages\n/Kids []\n/Count 0\n>>\nendobj\n\
xref\n0 3\n0000000000 65535 f \n0000000009 00000 n \n0000000058 00000 n \n\
trailer\n<<\n/Size 3\n/Root 1 0 R\n>>\nstartxref\n113\n%%EOF\n"
        .to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_builder_produces_pdf_header() {
        let builder = SimpleDocumentBuilder::new("Test");
        let bytes = builder.save();
        assert!(bytes.starts_with(b"%PDF-"), "output must start with %PDF-");
    }

    #[test]
    fn test_simple_builder_contains_eof() {
        let builder = SimpleDocumentBuilder::new("Test");
        let bytes = builder.save();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("%%EOF"), "output must contain %%EOF");
    }

    #[test]
    fn test_simple_builder_text_appears_in_output() {
        let mut builder = SimpleDocumentBuilder::new("Test");
        builder.text("Hello World", 12.0, 20.0, 280.0, BuiltinFont::Helvetica);
        let bytes = builder.save();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("Hello World"), "text must appear in PDF bytes");
    }

    #[test]
    fn test_simple_builder_bold_font_in_output() {
        let mut builder = SimpleDocumentBuilder::new("Bold Test");
        builder.text("Bold Title", 18.0, 20.0, 280.0, BuiltinFont::HelveticaBold);
        let bytes = builder.save();
        let content = String::from_utf8_lossy(&bytes);
        // F2 is HelveticaBold
        assert!(content.contains("F2"), "HelveticaBold must be referenced as F2");
        assert!(content.contains("Helvetica-Bold"), "Helvetica-Bold font must appear in resources");
    }

    #[test]
    fn test_simple_builder_page_height() {
        let builder = SimpleDocumentBuilder::new("Test");
        assert!((builder.page_height_mm() - 297.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_simple_builder_new_page_creates_multiple_pages() {
        let mut builder = SimpleDocumentBuilder::new("Multi-page");
        builder.text("Page 1", 12.0, 20.0, 280.0, BuiltinFont::Helvetica);
        builder.new_page();
        builder.text("Page 2", 12.0, 20.0, 280.0, BuiltinFont::Helvetica);
        let bytes = builder.save();
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("Page 1"), "page 1 text must appear");
        assert!(content.contains("Page 2"), "page 2 text must appear");
        // At least /Count 2 must be present
        assert!(content.contains("/Count 2"), "PDF must report 2 pages");
    }

    #[test]
    fn test_mm_to_pt_conversion() {
        // 25.4mm = 72pt (1 inch)
        let pt = mm_to_pt(25.4);
        assert!((pt - 72.0).abs() < 0.001);
    }

    #[test]
    fn test_escape_pdf_string_parens() {
        let escaped = escape_pdf_string("(hello)");
        assert_eq!(escaped, "\\(hello\\)");
    }

    #[test]
    fn test_escape_pdf_string_backslash() {
        let escaped = escape_pdf_string("back\\slash");
        assert_eq!(escaped, "back\\\\slash");
    }

    #[test]
    fn test_simple_builder_empty_document_is_valid_pdf() {
        let builder = SimpleDocumentBuilder::new("Empty");
        let bytes = builder.save();
        // Must have PDF header, xref, startxref, %%EOF
        let content = String::from_utf8_lossy(&bytes);
        assert!(content.contains("%PDF-"));
        assert!(content.contains("xref"));
        assert!(content.contains("startxref"));
        assert!(content.contains("%%EOF"));
    }
}
