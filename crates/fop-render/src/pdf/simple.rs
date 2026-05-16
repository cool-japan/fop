//! Simple programmatic PDF document builder.
//!
//! Provides a high-level builder for generating text-only A4 PDF documents
//! without requiring an area tree or XSL-FO pipeline. Suitable for audit logs,
//! verification reports, and other programmatically-generated documents.

use std::collections::HashSet;

use crate::pdf::document::types::{PdfDocument, PdfInfo, PdfPage};
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
    author: Option<String>,
    subject: Option<String>,
    creation_date: Option<String>,
    lang: Option<String>,
    xmp_metadata: Option<String>,
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
            author: None,
            subject: None,
            creation_date: None,
            lang: None,
            xmp_metadata: None,
            completed_pages: Vec::new(),
            current_page: PageState::new(),
            used_fonts: HashSet::new(),
        }
    }

    /// Set the document author.
    pub fn set_author(&mut self, s: impl Into<String>) -> &mut Self {
        self.author = Some(s.into());
        self
    }

    /// Set the document subject.
    pub fn set_subject(&mut self, s: impl Into<String>) -> &mut Self {
        self.subject = Some(s.into());
        self
    }

    /// Set the document creation date (PDF date format, e.g. `D:20260515120000`).
    pub fn set_creation_date(&mut self, s: impl Into<String>) -> &mut Self {
        self.creation_date = Some(s.into());
        self
    }

    /// Set the document language tag (BCP 47, e.g. `en-US`).
    pub fn set_lang(&mut self, s: impl Into<String>) -> &mut Self {
        self.lang = Some(s.into());
        self
    }

    /// Set a raw XMP metadata packet to embed in the PDF `/Metadata` stream.
    ///
    /// Dublin Core fields (`dc:title`, `dc:creator`, `dc:description`) are
    /// extracted and merged into the `/Info` dict if the corresponding builder
    /// field is not already set. Caller-set values always win.
    pub fn set_xmp_metadata(&mut self, s: impl Into<String>) -> &mut Self {
        self.xmp_metadata = Some(s.into());
        self
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
            author,
            subject,
            creation_date,
            lang,
            xmp_metadata,
            completed_pages,
            current_page: _,
            used_fonts,
        } = self;

        // Build a PdfDocument using the existing serialiser. We create one
        // PdfPage per accumulated PageState and let PdfDocument::to_bytes()
        // handle the cross-reference table, trailer, etc.
        let mut doc = PdfDocument::new();

        // Populate /Info fields — caller-set values win over DC-extracted values.
        if !title.is_empty() {
            doc.info.title = Some(title.clone());
        }
        if let Some(ref a) = author {
            doc.info.author = Some(a.clone());
        }
        if let Some(ref s) = subject {
            doc.info.subject = Some(s.clone());
        }
        if let Some(ref d) = creation_date {
            doc.info.creation_date = Some(d.clone());
        }
        if let Some(ref l) = lang {
            doc.info.lang = Some(l.clone());
        }

        // Merge Dublin Core fields from XMP (caller-set fields take priority).
        if let Some(ref packet) = xmp_metadata {
            merge_dc_into_info(packet, &mut doc.info);
            doc.set_xmp_metadata(packet.clone());
        }

        // Set /ID whenever any metadata is present.
        let has_any_metadata = doc.info.title.is_some()
            || doc.info.author.is_some()
            || doc.info.subject.is_some()
            || doc.info.creation_date.is_some()
            || doc.info.lang.is_some()
            || xmp_metadata.is_some();
        if has_any_metadata {
            let page_count = completed_pages.len();
            let seed = compute_file_id_seed(
                &title,
                page_count,
                xmp_metadata.as_deref(),
            );
            doc.file_id = Some(crate::pdf::security::generate_file_id(&seed));
        }

        for page_state in completed_pages {
            let mut pdf_page = PdfPage::new(Length::from_mm(210.0), Length::from_mm(297.0));
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
        let needs_extra_fonts = used_fonts.iter().any(|f| *f != BuiltinFont::Helvetica);

        if needs_extra_fonts {
            // Write our own minimal PDF that supports all 14 builtin fonts.
            // doc already has info, file_id, and xmp_metadata set above.
            write_minimal_pdf(doc)
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

/// A simple djb2 hash over bytes — used for deterministic file ID seeds.
fn simple_djb2_hash(bytes: &[u8]) -> u64 {
    let mut h: u64 = 5381;
    for &b in bytes {
        h = h.wrapping_mul(33).wrapping_add(b as u64);
    }
    h
}

/// Build the seed string for the deterministic file ID.
fn compute_file_id_seed(title: &str, page_count: usize, xmp: Option<&str>) -> String {
    let xmp_len = xmp.map(|x| x.len()).unwrap_or(0);
    let xmp_hash = xmp.map(|x| simple_djb2_hash(x.as_bytes())).unwrap_or(0);
    format!(
        "{}|pages={}|xmp_len={}|xmp_hash={:x}",
        title, page_count, xmp_len, xmp_hash
    )
}

/// Merge Dublin Core fields extracted from an XMP packet into a [`PdfInfo`].
///
/// Fields already set on `info` (caller-set) are never overwritten.
fn merge_dc_into_info(xmp: &str, info: &mut PdfInfo) {
    let dc = crate::pdf::compliance::extract_dc_fields(xmp);
    if info.title.is_none() {
        info.title = dc.title;
    }
    if info.author.is_none() {
        info.author = dc.creator;
    }
    if info.subject.is_none() {
        info.subject = dc.description;
    }
}

/// Emit a `/Metadata` XMP stream object into `buf`, recording its offset.
fn emit_xmp_metadata_object(
    buf: &mut Vec<u8>,
    xref_offsets: &mut Vec<usize>,
    obj_id: usize,
    packet: &str,
) {
    let xmp_content = crate::pdf::compliance::reconcile_xmp(
        packet,
        crate::pdf::compliance::PdfCompliance::Standard,
    );
    let xmp_bytes = xmp_content.as_bytes();
    xref_offsets.push(buf.len());
    buf.extend_from_slice(format!("{} 0 obj\n", obj_id).as_bytes());
    buf.extend_from_slice(b"<<\n/Type /Metadata\n/Subtype /XML\n");
    buf.extend_from_slice(format!("/Length {}\n", xmp_bytes.len()).as_bytes());
    buf.extend_from_slice(b">>\nstream\n");
    buf.extend_from_slice(xmp_bytes);
    buf.extend_from_slice(b"\nendstream\nendobj\n");
}

/// Write a minimal but complete PDF 1.4 file supporting all 14 builtin fonts.
///
/// All metadata (title, author, subject, creation_date, lang, xmp_metadata,
/// file_id) is read from `doc` directly — they were populated by `save()`.
///
/// Object layout (without XMP):
///   1  – Catalog
///   2  – Pages (page tree root)
///   3–16 – Font dictionaries (one per builtin font F1–F14)
///   17..(17 + N*2 - 1) – Page object + content stream pairs
///   (17 + N*2)  – /Info dict (if any metadata present)
///
/// Object layout (with XMP):
///   1  – Catalog (with /Metadata 17 0 R)
///   2  – Pages
///   3–16 – Font dictionaries
///   17 – /Metadata XMP stream
///   18..(18 + N*2 - 1) – Page object + content stream pairs
///   (18 + N*2)  – /Info dict (if any metadata present)
fn write_minimal_pdf(doc: PdfDocument) -> Vec<u8> {
    let mut bytes: Vec<u8> = Vec::new();
    let mut xref_offsets: Vec<usize> = Vec::new();

    // Header
    bytes.extend_from_slice(b"%PDF-1.4\n");
    bytes.extend_from_slice(b"%\xE2\xE3\xCF\xD3\n"); // Binary-safe comment

    // Object 0: always free
    xref_offsets.push(0);

    let all_fonts = BuiltinFont::all();
    let num_fonts = all_fonts.len(); // 14

    // Determine whether an XMP metadata stream object is needed.
    let xmp_obj_id_opt: Option<usize> = if doc.xmp_metadata.is_some() {
        Some(3 + num_fonts) // obj 17
    } else {
        None
    };

    // First page object id depends on whether XMP is present.
    let first_page_obj_id = match xmp_obj_id_opt {
        Some(xmp_id) => xmp_id + 1, // 18
        None => 3 + num_fonts,      // 17
    };

    // Object 1: Catalog
    xref_offsets.push(bytes.len());
    bytes.extend_from_slice(b"1 0 obj\n<<\n/Type /Catalog\n/Pages 2 0 R\n");
    if let Some(xmp_id) = xmp_obj_id_opt {
        bytes.extend_from_slice(format!("/Metadata {} 0 R\n", xmp_id).as_bytes());
    }
    if let Some(ref l) = doc.info.lang {
        bytes.extend_from_slice(format!("/Lang ({})\n", escape_pdf_string(l)).as_bytes());
    }
    bytes.extend_from_slice(b">>\nendobj\n");

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
        bytes.extend_from_slice(format!("/BaseFont /{}\n", font.base_font_name()).as_bytes());
        bytes.extend_from_slice(b">>\nendobj\n");
    }

    // Object 17 (optional): XMP /Metadata stream
    if let Some(xmp_id) = xmp_obj_id_opt {
        if let Some(ref packet) = doc.xmp_metadata {
            emit_xmp_metadata_object(&mut bytes, &mut xref_offsets, xmp_id, packet);
        }
    }

    // Build font resource dictionary string (referenced by every page)
    let mut font_resources = String::from("/Font <<\n");
    for (idx, font) in all_fonts.iter().enumerate() {
        let obj_id = 3 + idx;
        font_resources.push_str(&format!("  /{} {} 0 R\n", font.resource_name(), obj_id));
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
        bytes.extend_from_slice(format!("/Contents {} 0 R\n", content_obj_id).as_bytes());
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

    // /Info dict (written last, only if any field is set)
    let has_title = doc.info.title.as_ref().map(|t| !t.is_empty()).unwrap_or(false);
    let has_info = has_title
        || doc.info.author.is_some()
        || doc.info.subject.is_some()
        || doc.info.creation_date.is_some();

    let info_obj_id = first_page_obj_id + page_count * 2;
    if has_info {
        xref_offsets.push(bytes.len());
        bytes.extend_from_slice(format!("{} 0 obj\n<<\n", info_obj_id).as_bytes());
        if let Some(ref t) = doc.info.title {
            if !t.is_empty() {
                bytes.extend_from_slice(
                    format!("/Title ({})\n", escape_pdf_string(t)).as_bytes(),
                );
            }
        }
        if let Some(ref a) = doc.info.author {
            bytes.extend_from_slice(
                format!("/Author ({})\n", escape_pdf_string(a)).as_bytes(),
            );
        }
        if let Some(ref s) = doc.info.subject {
            bytes.extend_from_slice(
                format!("/Subject ({})\n", escape_pdf_string(s)).as_bytes(),
            );
        }
        if let Some(ref d) = doc.info.creation_date {
            bytes.extend_from_slice(
                format!("/CreationDate ({})\n", escape_pdf_string(d)).as_bytes(),
            );
        }
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
    if has_info {
        bytes.extend_from_slice(format!("/Info {} 0 R\n", info_obj_id).as_bytes());
    }
    // /ID array — emit whenever we have a file_id (set when any metadata is present)
    if let Some(ref fid) = doc.file_id {
        let hex: String = fid.iter().map(|b| format!("{:02X}", b)).collect();
        bytes.extend_from_slice(format!("/ID [<{}> <{}>]\n", hex, hex).as_bytes());
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
        assert!(
            content.contains("Hello World"),
            "text must appear in PDF bytes"
        );
    }

    #[test]
    fn test_simple_builder_bold_font_in_output() {
        let mut builder = SimpleDocumentBuilder::new("Bold Test");
        builder.text("Bold Title", 18.0, 20.0, 280.0, BuiltinFont::HelveticaBold);
        let bytes = builder.save();
        let content = String::from_utf8_lossy(&bytes);
        // F2 is HelveticaBold
        assert!(
            content.contains("F2"),
            "HelveticaBold must be referenced as F2"
        );
        assert!(
            content.contains("Helvetica-Bold"),
            "Helvetica-Bold font must appear in resources"
        );
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

    // ── XMP / metadata tests ──────────────────────────────────────────────────

    #[test]
    fn test_simple_builder_xmp_emits_metadata_stream_fast_path() {
        // Helvetica only → fast path
        let mut b = SimpleDocumentBuilder::new("XMP Fast");
        let xmp = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/"><rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"/></x:xmpmeta>"#;
        b.set_xmp_metadata(xmp);
        b.text("hello", 12.0, 100.0, 700.0, BuiltinFont::Helvetica);
        let bytes = b.save();
        let output = String::from_utf8_lossy(&bytes);
        assert!(
            output.contains("/Type /Metadata"),
            "should have /Type /Metadata"
        );
        assert!(
            output.contains("/Subtype /XML"),
            "should have /Subtype /XML"
        );
    }

    #[test]
    fn test_simple_builder_xmp_emits_metadata_stream_slow_path() {
        // HelveticaBold → slow path
        let mut b = SimpleDocumentBuilder::new("XMP Slow");
        let xmp = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/"><rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"/></x:xmpmeta>"#;
        b.set_xmp_metadata(xmp);
        b.text("hello", 12.0, 100.0, 700.0, BuiltinFont::HelveticaBold);
        let bytes = b.save();
        let output = String::from_utf8_lossy(&bytes);
        assert!(
            output.contains("/Type /Metadata"),
            "should have /Type /Metadata"
        );
        assert!(
            output.contains("/Subtype /XML"),
            "should have /Subtype /XML"
        );
    }

    #[test]
    fn test_simple_builder_xmp_syncs_dc_creator_to_author() {
        let mut b = SimpleDocumentBuilder::new("DC Test");
        let xmp = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:dc="http://purl.org/dc/elements/1.1/"><rdf:RDF><rdf:Description rdf:about=""><dc:creator><rdf:Bag><rdf:li>Alice</rdf:li></rdf:Bag></dc:creator></rdf:Description></rdf:RDF></x:xmpmeta>"#;
        b.set_xmp_metadata(xmp);
        b.text("x", 12.0, 100.0, 700.0, BuiltinFont::HelveticaBold); // slow path
        let bytes = b.save();
        let output = String::from_utf8_lossy(&bytes);
        assert!(output.contains("/Author"), "should have /Author from DC creator");
        assert!(output.contains("Alice"), "should contain Alice from dc:creator");
    }

    #[test]
    fn test_simple_builder_caller_author_wins_over_dc() {
        let mut b = SimpleDocumentBuilder::new("Priority Test");
        let xmp = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:dc="http://purl.org/dc/elements/1.1/"><rdf:RDF><rdf:Description rdf:about=""><dc:creator><rdf:Bag><rdf:li>Alice</rdf:li></rdf:Bag></dc:creator></rdf:Description></rdf:RDF></x:xmpmeta>"#;
        b.set_xmp_metadata(xmp);
        b.set_author("Bob");
        b.text("x", 12.0, 100.0, 700.0, BuiltinFont::HelveticaBold); // slow path
        let bytes = b.save();
        let output = String::from_utf8_lossy(&bytes);
        assert!(output.contains("Bob"), "Bob should be in output");
        // Alice should NOT appear in the /Info dict (Bob wins)
        // Note: Alice may appear in the XMP stream body itself — we check /Author only.
        // Find the /Author entry
        let info_author_bob = output.contains("/Author (Bob)");
        let info_author_alice = output.contains("/Author (Alice)");
        assert!(info_author_bob, "Bob should be /Author value");
        assert!(!info_author_alice, "Alice from DC should not be /Author (Bob wins)");
    }

    #[test]
    fn test_simple_builder_emits_id_trailer_when_metadata_present() {
        let mut b = SimpleDocumentBuilder::new("ID Test");
        b.set_author("Someone");
        b.text("x", 12.0, 100.0, 700.0, BuiltinFont::HelveticaBold); // slow path
        let bytes = b.save();
        let output = String::from_utf8_lossy(&bytes);
        // Find trailer section and check for /ID
        let trailer_pos = output.rfind("trailer").expect("should have trailer");
        let trailer_section = &output[trailer_pos..];
        assert!(
            trailer_section.contains("/ID [<"),
            "trailer should contain /ID [<..."
        );
    }

    #[test]
    fn test_simple_builder_lang_in_catalog() {
        let mut b = SimpleDocumentBuilder::new("Lang Test");
        b.set_lang("en-US");
        b.text("x", 12.0, 100.0, 700.0, BuiltinFont::HelveticaBold); // slow path
        let bytes = b.save();
        let output = String::from_utf8_lossy(&bytes);
        assert!(output.contains("/Lang"), "should have /Lang in catalog");
        assert!(output.contains("en-US"), "should contain en-US");
    }

    #[test]
    fn test_simple_builder_escapes_parens_in_info() {
        let mut b = SimpleDocumentBuilder::new("Paren Test");
        b.set_author("(parenthesised)");
        b.text("x", 12.0, 100.0, 700.0, BuiltinFont::HelveticaBold); // slow path
        let bytes = b.save();
        let output = String::from_utf8_lossy(&bytes);
        // After escaping, ( → \( and ) → \)
        assert!(
            output.contains(r"\(parenthesised\)"),
            "parentheses in /Author must be escaped; output snippet: {:?}",
            &output[output.find("/Author").unwrap_or(0)..std::cmp::min(output.len(), output.find("/Author").unwrap_or(0) + 60)]
        );
    }
}
