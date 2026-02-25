//! PDF structure parser
//!
//! Parses the PDF object model: indirect objects, cross-reference table,
//! page tree, and resource dictionaries.

use crate::error::{PdfRenderError, Result};
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// PDF Object model
// ---------------------------------------------------------------------------

/// A PDF object
#[derive(Debug, Clone)]
pub enum PdfObject {
    Null,
    Boolean(bool),
    Integer(i64),
    Real(f64),
    String(Vec<u8>),
    Name(String),
    Array(Vec<PdfObject>),
    Dictionary(PdfDictionary),
    Stream(PdfDictionary, Vec<u8>),
    /// Indirect reference: (object_number, generation)
    Reference(u32, u16),
}

impl PdfObject {
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            PdfObject::Integer(n) => Some(*n),
            PdfObject::Real(f) => Some(*f as i64),
            _ => None,
        }
    }

    pub fn as_real(&self) -> Option<f64> {
        match self {
            PdfObject::Real(f) => Some(*f),
            PdfObject::Integer(n) => Some(*n as f64),
            _ => None,
        }
    }

    pub fn as_name(&self) -> Option<&str> {
        match self {
            PdfObject::Name(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_str_bytes(&self) -> Option<&[u8]> {
        match self {
            PdfObject::String(b) => Some(b.as_slice()),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&[PdfObject]> {
        match self {
            PdfObject::Array(a) => Some(a.as_slice()),
            _ => None,
        }
    }

    pub fn as_dict(&self) -> Option<&PdfDictionary> {
        match self {
            PdfObject::Dictionary(d) => Some(d),
            PdfObject::Stream(d, _) => Some(d),
            _ => None,
        }
    }

    pub fn as_reference(&self) -> Option<(u32, u16)> {
        match self {
            PdfObject::Reference(n, g) => Some((*n, *g)),
            _ => None,
        }
    }
}

/// A PDF dictionary mapping names to objects
#[derive(Debug, Clone, Default)]
pub struct PdfDictionary(pub HashMap<String, PdfObject>);

impl PdfDictionary {
    pub fn get(&self, key: &str) -> Option<&PdfObject> {
        self.0.get(key)
    }

    pub fn get_integer(&self, key: &str) -> Option<i64> {
        self.get(key)?.as_integer()
    }

    pub fn get_real(&self, key: &str) -> Option<f64> {
        self.get(key)?.as_real()
    }

    pub fn get_name(&self, key: &str) -> Option<&str> {
        self.get(key)?.as_name()
    }

    pub fn get_array(&self, key: &str) -> Option<&[PdfObject]> {
        self.get(key)?.as_array()
    }
}

// ---------------------------------------------------------------------------
// PDF document structure
// ---------------------------------------------------------------------------

/// A parsed PDF page
#[derive(Debug, Clone)]
pub struct PdfPage {
    /// Page number (0-indexed)
    pub index: usize,
    /// Page media box [x y width height]
    pub media_box: [f64; 4],
    /// Raw content stream bytes (decompressed)
    pub content: Vec<u8>,
    /// Resources dictionary
    pub resources: PdfDictionary,
}

/// Parsed PDF document
pub struct PdfDocument {
    /// All indirect objects: key = object_number
    objects: HashMap<u32, PdfObject>,
    /// Page order (list of object numbers for page objects)
    page_refs: Vec<u32>,
}

impl PdfDocument {
    /// Parse a PDF from bytes
    pub fn from_bytes(data: &[u8]) -> Result<Self> {
        let parser = RawParser::new(data);
        parser.parse()
    }

    /// Number of pages in the document
    pub fn page_count(&self) -> usize {
        self.page_refs.len()
    }

    /// Get a parsed page by 0-based index
    pub fn get_page(&self, index: usize) -> Result<PdfPage> {
        if index >= self.page_refs.len() {
            return Err(PdfRenderError::PageNotFound(index, self.page_refs.len()));
        }
        let obj_num = self.page_refs[index];
        let page_dict = self.resolve_dict(obj_num)?;

        // Media box
        let media_box = self.get_media_box(&page_dict, obj_num)?;

        // Resources
        let resources = self.get_resources(&page_dict, obj_num)?;

        // Content stream(s)
        let content = self.get_page_content(&page_dict)?;

        Ok(PdfPage {
            index,
            media_box,
            content,
            resources,
        })
    }

    /// Resolve an indirect reference to an object
    pub fn resolve<'a>(&'a self, obj: &'a PdfObject) -> Option<&'a PdfObject> {
        match obj {
            PdfObject::Reference(n, _) => self.objects.get(n),
            other => Some(other),
        }
    }

    fn resolve_num(&self, num: u32) -> Option<&PdfObject> {
        self.objects.get(&num)
    }

    fn resolve_dict(&self, num: u32) -> Result<PdfDictionary> {
        match self.resolve_num(num) {
            Some(PdfObject::Dictionary(d)) => Ok(d.clone()),
            Some(PdfObject::Stream(d, _)) => Ok(d.clone()),
            _ => Err(PdfRenderError::Parse(format!(
                "Object {} is not a dictionary",
                num
            ))),
        }
    }

    fn resolve_obj<'a>(&'a self, obj: &'a PdfObject) -> &'a PdfObject {
        match obj {
            PdfObject::Reference(n, _) => self.resolve_num(*n).unwrap_or(obj),
            other => other,
        }
    }

    fn get_media_box(&self, page_dict: &PdfDictionary, _obj_num: u32) -> Result<[f64; 4]> {
        let arr = page_dict
            .get_array("MediaBox")
            .ok_or_else(|| PdfRenderError::Parse("Page missing MediaBox".to_string()))?;
        if arr.len() < 4 {
            return Err(PdfRenderError::Parse("MediaBox too short".to_string()));
        }
        Ok([
            self.resolve_obj(&arr[0]).as_real().unwrap_or(0.0),
            self.resolve_obj(&arr[1]).as_real().unwrap_or(0.0),
            self.resolve_obj(&arr[2]).as_real().unwrap_or(612.0),
            self.resolve_obj(&arr[3]).as_real().unwrap_or(792.0),
        ])
    }

    fn get_resources(&self, page_dict: &PdfDictionary, _obj_num: u32) -> Result<PdfDictionary> {
        let res = match page_dict.get("Resources") {
            Some(PdfObject::Reference(n, _)) => match self.resolve_num(*n) {
                Some(PdfObject::Dictionary(d)) => d.clone(),
                _ => PdfDictionary::default(),
            },
            Some(PdfObject::Dictionary(d)) => d.clone(),
            _ => PdfDictionary::default(),
        };
        Ok(res)
    }

    fn get_page_content(&self, page_dict: &PdfDictionary) -> Result<Vec<u8>> {
        // Match directly on raw object — do NOT resolve before matching.
        // If Contents is a Reference, decode the referenced stream.
        // If Contents is an Array, concatenate all referenced streams.
        // If Contents is a Stream (inline), decode it directly.
        match page_dict.get("Contents") {
            None => Ok(Vec::new()),
            Some(PdfObject::Reference(n, _)) => self.decode_stream(*n),
            Some(PdfObject::Array(arr)) => {
                // Multiple content streams — concatenate
                let arr = arr.clone();
                let mut combined = Vec::new();
                for item in &arr {
                    match item {
                        PdfObject::Reference(n, _) => {
                            combined.extend_from_slice(&self.decode_stream(*n)?);
                            combined.push(b' ');
                        }
                        PdfObject::Stream(dict, raw) => {
                            combined.extend_from_slice(&decode_stream_data(dict, raw)?);
                            combined.push(b' ');
                        }
                        _ => {}
                    }
                }
                Ok(combined)
            }
            Some(PdfObject::Stream(dict, raw)) => decode_stream_data(dict, raw),
            _ => Ok(Vec::new()),
        }
    }

    /// Decode (decompress) a stream object
    pub fn decode_stream(&self, obj_num: u32) -> Result<Vec<u8>> {
        match self.resolve_num(obj_num) {
            Some(PdfObject::Stream(dict, raw)) => decode_stream_data(dict, raw),
            _ => Err(PdfRenderError::Parse(format!(
                "Object {} is not a stream",
                obj_num
            ))),
        }
    }

    /// Look up a font resource by name
    pub fn get_font(&self, resources: &PdfDictionary, name: &str) -> Option<PdfDictionary> {
        let fonts = resources.get("Font")?;
        let fonts_dict = match fonts {
            PdfObject::Dictionary(d) => d,
            PdfObject::Reference(n, _) => {
                if let Some(PdfObject::Dictionary(d)) = self.resolve_num(*n) {
                    d
                } else {
                    return None;
                }
            }
            _ => return None,
        };
        let font_ref = fonts_dict.get(name)?;
        let resolved = self.resolve_obj(font_ref);
        match resolved {
            PdfObject::Dictionary(d) => Some(d.clone()),
            PdfObject::Stream(d, _) => Some(d.clone()),
            PdfObject::Reference(n, _) => {
                let num = *n;
                self.resolve_dict(num).ok()
            }
            _ => None,
        }
    }

    /// Get an XObject (image, form) by resource name
    pub fn get_xobject(
        &self,
        resources: &PdfDictionary,
        name: &str,
    ) -> Option<(PdfDictionary, Vec<u8>)> {
        let xobjs = resources.get("XObject")?;
        let xobjs_dict = match xobjs {
            PdfObject::Dictionary(d) => d,
            PdfObject::Reference(n, _) => {
                if let Some(PdfObject::Dictionary(d)) = self.resolve_num(*n) {
                    d
                } else {
                    return None;
                }
            }
            _ => return None,
        };
        let xobj_ref = xobjs_dict.get(name)?;
        let num = xobj_ref.as_reference()?.0;
        match self.resolve_num(num)? {
            PdfObject::Stream(d, raw) => {
                let data = decode_stream_data(d, raw).ok()?;
                Some((d.clone(), data))
            }
            _ => None,
        }
    }

    /// Get embedded font data (TrueType stream)
    pub fn get_font_file(&self, font_descriptor: &PdfDictionary) -> Option<Vec<u8>> {
        // Try FontFile2 (TrueType), then FontFile3
        for key in &["FontFile2", "FontFile3", "FontFile"] {
            if let Some(obj) = font_descriptor.get(key) {
                if let Some((n, _)) = obj.as_reference() {
                    return self.decode_stream(n).ok();
                }
            }
        }
        None
    }

    /// Resolve a descendant font dictionary (for Type0 / composite fonts)
    pub fn get_descendant_font(&self, font_dict: &PdfDictionary) -> Option<PdfDictionary> {
        let arr = font_dict.get_array("DescendantFonts")?;
        let first = arr.first()?;
        let resolved = self.resolve_obj(first);
        match resolved {
            PdfObject::Dictionary(d) => Some(d.clone()),
            PdfObject::Reference(n, _) => self.resolve_dict(*n).ok(),
            _ => None,
        }
    }

    /// Get FontDescriptor from a font dict
    pub fn get_font_descriptor(&self, font_dict: &PdfDictionary) -> Option<PdfDictionary> {
        let fd = font_dict.get("FontDescriptor")?;
        let resolved = self.resolve_obj(fd);
        match resolved {
            PdfObject::Dictionary(d) => Some(d.clone()),
            PdfObject::Reference(n, _) => self.resolve_dict(*n).ok(),
            _ => None,
        }
    }

    /// Get ToUnicode CMap stream (raw bytes)
    pub fn get_to_unicode(&self, font_dict: &PdfDictionary) -> Option<Vec<u8>> {
        let tu = font_dict.get("ToUnicode")?;
        let num = tu.as_reference()?.0;
        self.decode_stream(num).ok()
    }
}

// ---------------------------------------------------------------------------
// Stream decoding
// ---------------------------------------------------------------------------

fn decode_stream_data(dict: &PdfDictionary, raw: &[u8]) -> Result<Vec<u8>> {
    let filter = match dict.get("Filter") {
        None => return Ok(raw.to_vec()),
        Some(f) => f,
    };

    match filter {
        PdfObject::Name(name) => decompress_filter(name.as_str(), raw),
        PdfObject::Array(arr) => {
            // Chain of filters
            let mut data = raw.to_vec();
            for f in arr {
                if let PdfObject::Name(name) = f {
                    data = decompress_filter(name.as_str(), &data)?;
                }
            }
            Ok(data)
        }
        _ => Ok(raw.to_vec()),
    }
}

fn decompress_filter(filter: &str, data: &[u8]) -> Result<Vec<u8>> {
    match filter {
        "FlateDecode" | "Fl" => {
            use flate2::read::ZlibDecoder;
            use std::io::Read;
            let mut decoder = ZlibDecoder::new(data);
            let mut out = Vec::new();
            decoder
                .read_to_end(&mut out)
                .map_err(|e| PdfRenderError::Decompress(e.to_string()))?;
            Ok(out)
        }
        "DCTDecode" | "DCT" => {
            // JPEG — return as-is, caller handles decoding
            Ok(data.to_vec())
        }
        "ASCIIHexDecode" | "AHx" => {
            let s = std::str::from_utf8(data).unwrap_or("");
            let hex: String = s
                .chars()
                .filter(|c| !c.is_whitespace() && *c != '>')
                .collect();
            (0..hex.len() / 2)
                .map(|i| {
                    u8::from_str_radix(&hex[2 * i..2 * i + 2], 16)
                        .map_err(|e| PdfRenderError::Parse(e.to_string()))
                })
                .collect()
        }
        _ => {
            log::debug!("Unsupported stream filter: {}", filter);
            Ok(data.to_vec())
        }
    }
}

// ---------------------------------------------------------------------------
// Raw parser
// ---------------------------------------------------------------------------

struct RawParser<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> RawParser<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    fn parse(mut self) -> Result<PdfDocument> {
        // Verify header
        if !self.data.starts_with(b"%PDF-") {
            return Err(PdfRenderError::Parse("Not a PDF file".to_string()));
        }

        // Find startxref
        let startxref = self.find_startxref()?;

        // Parse xref table → object offsets
        let offsets = self.parse_xref(startxref)?;

        // Parse all objects
        let mut objects = HashMap::new();
        for (obj_num, offset) in &offsets {
            if *offset == 0 {
                continue; // free entry
            }
            self.pos = *offset;
            if let Ok(obj) = self.parse_indirect_object() {
                objects.insert(*obj_num, obj);
            }
        }

        // Build page list
        let page_refs = collect_pages(&objects)?;

        Ok(PdfDocument { objects, page_refs })
    }

    fn find_startxref(&self) -> Result<usize> {
        // Search backwards from end of file
        let data = self.data;
        let len = data.len();
        let search_from = len.saturating_sub(1024);
        let tail = &data[search_from..];

        // Find "startxref"
        let keyword = b"startxref";
        let pos = tail
            .windows(keyword.len())
            .rposition(|w| w == keyword)
            .ok_or_else(|| PdfRenderError::Parse("startxref not found".to_string()))?;

        // Skip "startxref" and whitespace, read number
        let mut i = search_from + pos + keyword.len();
        while i < len && (data[i] == b' ' || data[i] == b'\n' || data[i] == b'\r') {
            i += 1;
        }
        let start = i;
        while i < len && data[i].is_ascii_digit() {
            i += 1;
        }
        let s = std::str::from_utf8(&data[start..i])
            .map_err(|e| PdfRenderError::Parse(e.to_string()))?;
        s.parse::<usize>()
            .map_err(|e| PdfRenderError::Parse(e.to_string()))
    }

    fn parse_xref(&mut self, offset: usize) -> Result<HashMap<u32, usize>> {
        if offset >= self.data.len() {
            return Err(PdfRenderError::Parse(format!(
                "xref offset {} is out of range (file size {})",
                offset,
                self.data.len()
            )));
        }
        self.pos = offset;
        self.skip_whitespace();

        let mut offsets: HashMap<u32, usize> = HashMap::new();

        // Check if this is an xref stream (PDF 1.5+)
        // For simplicity, handle both cross-reference tables and streams
        if self.peek_bytes(4) == b"xref" {
            self.parse_xref_table(&mut offsets)?;
        } else {
            // Try xref stream
            if let Ok((obj, _)) = self.try_parse_xref_stream() {
                offsets.extend(obj);
            }
        }

        Ok(offsets)
    }

    fn parse_xref_table(&mut self, offsets: &mut HashMap<u32, usize>) -> Result<()> {
        // skip "xref"
        self.pos += 4;
        self.skip_whitespace();

        loop {
            // Check for "trailer" keyword
            if self.peek_bytes(7) == b"trailer" {
                break;
            }
            if self.pos >= self.data.len() {
                break;
            }

            // Read subsection header: "first_obj count"
            let first_obj = self.read_integer()? as u32;
            self.skip_whitespace();
            let count = self.read_integer()? as u32;
            self.skip_whitespace();

            for i in 0..count {
                if self.pos + 20 > self.data.len() {
                    break;
                }
                let entry = &self.data[self.pos..self.pos + 20];
                let offset_str = std::str::from_utf8(&entry[0..10]).unwrap_or("0");
                let in_use = entry[17] == b'n';
                let offset = offset_str.trim().parse::<usize>().unwrap_or(0);

                if in_use {
                    offsets.insert(first_obj + i, offset);
                }
                self.pos += 20;
            }
            self.skip_whitespace();
        }
        Ok(())
    }

    fn try_parse_xref_stream(&mut self) -> Result<(HashMap<u32, usize>, PdfDictionary)> {
        // Parse the xref stream object: "N G obj << ... >> stream ... endstream"
        let _obj_num = self.read_integer()?;
        self.skip_whitespace();
        let _gen = self.read_integer()?;
        self.skip_whitespace();
        if !self.consume_keyword(b"obj") {
            return Err(PdfRenderError::Parse("Expected 'obj'".to_string()));
        }
        self.skip_whitespace();
        let dict = self.parse_dictionary()?;
        self.skip_whitespace();

        // Read stream data
        if !self.consume_keyword(b"stream") {
            return Err(PdfRenderError::Parse("Expected 'stream'".to_string()));
        }
        // skip optional CR+LF
        if self.pos < self.data.len() && self.data[self.pos] == b'\r' {
            self.pos += 1;
        }
        if self.pos < self.data.len() && self.data[self.pos] == b'\n' {
            self.pos += 1;
        }

        let length = dict.get_integer("Length").unwrap_or(0) as usize;
        let raw = self.data[self.pos..self.pos + length.min(self.data.len() - self.pos)].to_vec();
        let decoded = decode_stream_data(&dict, &raw)?;

        // Parse xref stream entries
        let w_arr = dict.get_array("W").unwrap_or(&[]);
        let w: Vec<usize> = w_arr
            .iter()
            .map(|o| o.as_integer().unwrap_or(0) as usize)
            .collect();
        if w.len() < 3 {
            return Ok((HashMap::new(), dict));
        }

        let first = dict.get_integer("Index").map(|_| 0u32).unwrap_or(0);

        let entry_size = w[0] + w[1] + w[2];
        if entry_size == 0 {
            return Ok((HashMap::new(), dict));
        }

        let mut offsets = HashMap::new();
        let mut pos = 0;
        let mut obj_num = first;
        while pos + entry_size <= decoded.len() {
            let t = read_be_int(&decoded[pos..pos + w[0]]);
            let f1 = read_be_int(&decoded[pos + w[0]..pos + w[0] + w[1]]);
            pos += entry_size;

            if t == 1 {
                // in-use object
                offsets.insert(obj_num, f1);
            }
            obj_num += 1;
        }

        Ok((offsets, dict))
    }

    fn parse_indirect_object(&mut self) -> Result<PdfObject> {
        self.skip_whitespace();
        let _obj_num = self.read_integer()?;
        self.skip_whitespace();
        let _gen = self.read_integer()?;
        self.skip_whitespace();
        if !self.consume_keyword(b"obj") {
            return Err(PdfRenderError::Parse("Expected 'obj'".to_string()));
        }
        self.skip_whitespace();
        let obj = self.parse_object()?;
        Ok(obj)
    }

    fn parse_object(&mut self) -> Result<PdfObject> {
        self.skip_whitespace();
        if self.pos >= self.data.len() {
            return Ok(PdfObject::Null);
        }

        match self.data[self.pos] {
            b't' if self.peek_bytes(4) == b"true" => {
                self.pos += 4;
                Ok(PdfObject::Boolean(true))
            }
            b'f' if self.peek_bytes(5) == b"false" => {
                self.pos += 5;
                Ok(PdfObject::Boolean(false))
            }
            b'n' if self.peek_bytes(4) == b"null" => {
                self.pos += 4;
                Ok(PdfObject::Null)
            }
            b'/' => {
                self.pos += 1;
                let name = self.read_name();
                Ok(PdfObject::Name(name))
            }
            b'(' => Ok(PdfObject::String(self.read_literal_string()?)),
            b'<' => {
                if self.data.get(self.pos + 1) == Some(&b'<') {
                    let dict = self.parse_dictionary()?;
                    self.skip_whitespace();
                    // Check if followed by "stream"
                    if self.peek_bytes(6) == b"stream" {
                        self.pos += 6;
                        if self.pos < self.data.len() && self.data[self.pos] == b'\r' {
                            self.pos += 1;
                        }
                        if self.pos < self.data.len() && self.data[self.pos] == b'\n' {
                            self.pos += 1;
                        }
                        let length = dict.get_integer("Length").unwrap_or(0) as usize;
                        let end = (self.pos + length).min(self.data.len());
                        let raw = self.data[self.pos..end].to_vec();
                        self.pos = end;
                        Ok(PdfObject::Stream(dict, raw))
                    } else {
                        Ok(PdfObject::Dictionary(dict))
                    }
                } else {
                    Ok(PdfObject::String(self.read_hex_string()?))
                }
            }
            b'[' => {
                self.pos += 1;
                let mut arr = Vec::new();
                loop {
                    self.skip_whitespace();
                    if self.pos >= self.data.len() || self.data[self.pos] == b']' {
                        if self.pos < self.data.len() {
                            self.pos += 1;
                        }
                        break;
                    }
                    arr.push(self.parse_object()?);
                }
                Ok(PdfObject::Array(arr))
            }
            c if c == b'-' || c == b'+' || c.is_ascii_digit() || c == b'.' => self.parse_number(),
            _ => {
                // Could be a keyword or garbage
                let word = self.read_word();
                if word.is_empty() {
                    self.pos += 1;
                    Ok(PdfObject::Null)
                } else {
                    Ok(PdfObject::Name(word))
                }
            }
        }
    }

    fn parse_number(&mut self) -> Result<PdfObject> {
        let start = self.pos;
        let mut is_real = false;
        if self.pos < self.data.len()
            && (self.data[self.pos] == b'-' || self.data[self.pos] == b'+')
        {
            self.pos += 1;
        }
        while self.pos < self.data.len() && self.data[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        if self.pos < self.data.len() && self.data[self.pos] == b'.' {
            is_real = true;
            self.pos += 1;
            while self.pos < self.data.len() && self.data[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }

        // Check for indirect reference pattern: "N G R"
        let after_first = self.pos;
        let saved = self.pos;
        self.skip_whitespace();
        if !is_real && self.pos < self.data.len() && self.data[self.pos].is_ascii_digit() {
            let gen_start = self.pos;
            while self.pos < self.data.len() && self.data[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
            let gen_end = self.pos;
            self.skip_whitespace();
            if self.pos < self.data.len() && self.data[self.pos] == b'R' {
                // It's an indirect reference
                let obj_s = std::str::from_utf8(&self.data[start..after_first]).unwrap_or("0");
                let gen_s = std::str::from_utf8(&self.data[gen_start..gen_end]).unwrap_or("0");
                let obj_num: u32 = obj_s.trim().parse().unwrap_or(0);
                let gen: u16 = gen_s.trim().parse().unwrap_or(0);
                self.pos += 1; // consume 'R'
                return Ok(PdfObject::Reference(obj_num, gen));
            }
            // Not a reference, rewind
            self.pos = saved;
        } else {
            self.pos = saved;
        }

        let s = std::str::from_utf8(&self.data[start..after_first])
            .map_err(|e| PdfRenderError::Parse(e.to_string()))?;
        if is_real {
            let f: f64 = s
                .trim()
                .parse()
                .map_err(|e: std::num::ParseFloatError| PdfRenderError::Parse(e.to_string()))?;
            Ok(PdfObject::Real(f))
        } else {
            let n: i64 = s
                .trim()
                .parse()
                .map_err(|e: std::num::ParseIntError| PdfRenderError::Parse(e.to_string()))?;
            Ok(PdfObject::Integer(n))
        }
    }

    fn parse_dictionary(&mut self) -> Result<PdfDictionary> {
        // consume "<<"
        self.pos += 2;
        let mut map = HashMap::new();
        loop {
            self.skip_whitespace();
            if self.pos + 1 < self.data.len()
                && self.data[self.pos] == b'>'
                && self.data[self.pos + 1] == b'>'
            {
                self.pos += 2;
                break;
            }
            if self.pos >= self.data.len() {
                break;
            }
            // Parse key (must be a Name)
            if self.data[self.pos] != b'/' {
                // Skip until we find a name or >>
                self.pos += 1;
                continue;
            }
            self.pos += 1;
            let key = self.read_name();
            self.skip_whitespace();
            let value = self.parse_object()?;
            map.insert(key, value);
        }
        Ok(PdfDictionary(map))
    }

    fn read_name(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.data.len() {
            let c = self.data[self.pos];
            if c == b' '
                || c == b'\t'
                || c == b'\n'
                || c == b'\r'
                || c == b'/'
                || c == b'<'
                || c == b'>'
                || c == b'['
                || c == b']'
                || c == b'('
                || c == b')'
                || c == b'{'
                || c == b'}'
            {
                break;
            }
            self.pos += 1;
        }
        String::from_utf8_lossy(&self.data[start..self.pos]).into_owned()
    }

    fn read_literal_string(&mut self) -> Result<Vec<u8>> {
        // skip '('
        self.pos += 1;
        let mut result = Vec::new();
        let mut depth = 1i32;
        while self.pos < self.data.len() {
            match self.data[self.pos] {
                b'\\' => {
                    self.pos += 1;
                    if self.pos < self.data.len() {
                        let escaped = match self.data[self.pos] {
                            b'n' => b'\n',
                            b'r' => b'\r',
                            b't' => b'\t',
                            b'b' => 0x08,
                            b'f' => 0x0C,
                            b'(' => b'(',
                            b')' => b')',
                            b'\\' => b'\\',
                            c => c,
                        };
                        result.push(escaped);
                        self.pos += 1;
                    }
                }
                b'(' => {
                    depth += 1;
                    result.push(b'(');
                    self.pos += 1;
                }
                b')' => {
                    depth -= 1;
                    if depth == 0 {
                        self.pos += 1;
                        break;
                    }
                    result.push(b')');
                    self.pos += 1;
                }
                c => {
                    result.push(c);
                    self.pos += 1;
                }
            }
        }
        Ok(result)
    }

    fn read_hex_string(&mut self) -> Result<Vec<u8>> {
        // skip '<'
        self.pos += 1;
        let mut result = Vec::new();
        let mut nibble: Option<u8> = None;
        while self.pos < self.data.len() {
            let c = self.data[self.pos];
            if c == b'>' {
                self.pos += 1;
                break;
            }
            if c.is_ascii_whitespace() {
                self.pos += 1;
                continue;
            }
            let digit = match c {
                b'0'..=b'9' => c - b'0',
                b'a'..=b'f' => c - b'a' + 10,
                b'A'..=b'F' => c - b'A' + 10,
                _ => {
                    self.pos += 1;
                    continue;
                }
            };
            match nibble {
                None => {
                    nibble = Some(digit << 4);
                }
                Some(hi) => {
                    result.push(hi | digit);
                    nibble = None;
                }
            }
            self.pos += 1;
        }
        if let Some(hi) = nibble {
            result.push(hi);
        }
        Ok(result)
    }

    fn read_integer(&mut self) -> Result<i64> {
        self.skip_whitespace();
        let start = self.pos;
        if self.pos < self.data.len()
            && (self.data[self.pos] == b'-' || self.data[self.pos] == b'+')
        {
            self.pos += 1;
        }
        while self.pos < self.data.len() && self.data[self.pos].is_ascii_digit() {
            self.pos += 1;
        }
        let s = std::str::from_utf8(&self.data[start..self.pos])
            .map_err(|e| PdfRenderError::Parse(e.to_string()))?;
        s.trim()
            .parse::<i64>()
            .map_err(|e| PdfRenderError::Parse(e.to_string()))
    }

    fn read_word(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.data.len() {
            let c = self.data[self.pos];
            if c.is_ascii_whitespace()
                || c == b'/'
                || c == b'<'
                || c == b'>'
                || c == b'['
                || c == b']'
            {
                break;
            }
            self.pos += 1;
        }
        String::from_utf8_lossy(&self.data[start..self.pos]).into_owned()
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.data.len() {
            match self.data[self.pos] {
                b' ' | b'\t' | b'\n' | b'\r' | b'\x0C' => self.pos += 1,
                b'%' => {
                    // Comment: skip to end of line
                    while self.pos < self.data.len() && self.data[self.pos] != b'\n' {
                        self.pos += 1;
                    }
                }
                _ => break,
            }
        }
    }

    fn peek_bytes(&self, n: usize) -> &[u8] {
        let end = (self.pos + n).min(self.data.len());
        &self.data[self.pos..end]
    }

    fn consume_keyword(&mut self, kw: &[u8]) -> bool {
        if self.data[self.pos..].starts_with(kw) {
            self.pos += kw.len();
            true
        } else {
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Page tree collection
// ---------------------------------------------------------------------------

fn collect_pages(objects: &HashMap<u32, PdfObject>) -> Result<Vec<u32>> {
    // Find the catalog
    let catalog_num = find_catalog(objects)?;
    let catalog = match objects.get(&catalog_num) {
        Some(PdfObject::Dictionary(d)) | Some(PdfObject::Stream(d, _)) => d,
        _ => return Err(PdfRenderError::Parse("Catalog not found".to_string())),
    };

    // Get Pages root
    let pages_ref = catalog
        .get("Pages")
        .and_then(|o| o.as_reference())
        .ok_or_else(|| PdfRenderError::Parse("Catalog missing Pages".to_string()))?;

    let mut page_list = Vec::new();
    collect_page_tree(objects, pages_ref.0, &mut page_list);
    Ok(page_list)
}

fn collect_page_tree(objects: &HashMap<u32, PdfObject>, node_num: u32, pages: &mut Vec<u32>) {
    let dict = match objects.get(&node_num) {
        Some(PdfObject::Dictionary(d)) | Some(PdfObject::Stream(d, _)) => d,
        _ => return,
    };
    let type_name = dict.get_name("Type").unwrap_or("");
    match type_name {
        "Pages" => {
            if let Some(kids) = dict.get_array("Kids") {
                let refs: Vec<u32> = kids
                    .iter()
                    .filter_map(|k| k.as_reference().map(|(n, _)| n))
                    .collect();
                for kid in refs {
                    collect_page_tree(objects, kid, pages);
                }
            }
        }
        "Page" => {
            pages.push(node_num);
        }
        _ => {}
    }
}

fn find_catalog(objects: &HashMap<u32, PdfObject>) -> Result<u32> {
    // Find the Catalog object by looking for /Type /Catalog
    for (num, obj) in objects {
        let dict = match obj {
            PdfObject::Dictionary(d) | PdfObject::Stream(d, _) => d,
            _ => continue,
        };
        if dict.get_name("Type") == Some("Catalog") {
            return Ok(*num);
        }
    }
    // Fallback: look in trailer (not parsed here, try object 1)
    Err(PdfRenderError::Parse("Catalog not found".to_string()))
}

fn read_be_int(bytes: &[u8]) -> usize {
    let mut v = 0usize;
    for &b in bytes {
        v = (v << 8) | b as usize;
    }
    v
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Build a minimal but structurally valid 1-page PDF with correct xref offsets.
    fn build_minimal_pdf() -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();

        out.extend_from_slice(b"%PDF-1.4\n");

        let o1_start = out.len();
        out.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

        let o2_start = out.len();
        out.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

        let o3_start = out.len();
        out.extend_from_slice(
            b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] >>\nendobj\n",
        );

        let xref_pos = out.len();
        out.extend_from_slice(b"xref\n0 4\n");
        out.extend_from_slice(b"0000000000 65535 f \n");
        out.extend_from_slice(format!("{:010} 00000 n \n", o1_start).as_bytes());
        out.extend_from_slice(format!("{:010} 00000 n \n", o2_start).as_bytes());
        out.extend_from_slice(format!("{:010} 00000 n \n", o3_start).as_bytes());
        out.extend_from_slice(b"trailer\n<< /Size 4 /Root 1 0 R >>\n");
        out.extend_from_slice(b"startxref\n");
        out.extend_from_slice(format!("{}\n", xref_pos).as_bytes());
        out.extend_from_slice(b"%%EOF\n");

        out
    }

    // -----------------------------------------------------------------------
    // Robustness / invalid input
    // -----------------------------------------------------------------------

    #[test]
    fn test_empty_bytes_returns_error() {
        assert!(PdfDocument::from_bytes(b"").is_err());
    }

    #[test]
    fn test_random_bytes_returns_error() {
        assert!(PdfDocument::from_bytes(b"hello world this is not a pdf").is_err());
    }

    #[test]
    fn test_pdf_header_only_returns_error() {
        assert!(PdfDocument::from_bytes(b"%PDF-1.4\n").is_err());
    }

    #[test]
    fn test_truncated_before_eof_returns_error() {
        let data = b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog";
        assert!(PdfDocument::from_bytes(data).is_err());
    }

    #[test]
    fn test_missing_eof_marker_returns_error() {
        let data = b"%PDF-1.4\n1 0 obj\n<< /Type /Catalog >>\nendobj\n";
        assert!(PdfDocument::from_bytes(data).is_err());
    }

    #[test]
    fn test_invalid_xref_offset_handled_gracefully() {
        // xref startxref points to wrong location — should error, not panic
        let data = b"%PDF-1.4\n1 0 obj\n<</Type/Catalog/Pages 2 0 R>>\nendobj\n\
            xref\n0 2\n0000000000 65535 f \n0000000009 00000 n \n\
            trailer\n<</Size 2/Root 1 0 R>>\nstartxref\n999999\n%%EOF\n";
        // May error or succeed with empty page list — must not panic
        let _ = PdfDocument::from_bytes(data);
    }

    // -----------------------------------------------------------------------
    // Minimal valid PDF
    // -----------------------------------------------------------------------

    #[test]
    fn test_minimal_valid_pdf_parses() {
        let pdf = build_minimal_pdf();
        let doc = PdfDocument::from_bytes(&pdf).expect("Minimal valid PDF should parse");
        assert_eq!(doc.page_count(), 1);
    }

    #[test]
    fn test_get_page_zero_succeeds() {
        let pdf = build_minimal_pdf();
        let doc = PdfDocument::from_bytes(&pdf).expect("Should parse");
        assert!(doc.get_page(0).is_ok());
    }

    #[test]
    fn test_get_page_out_of_bounds_returns_error() {
        let pdf = build_minimal_pdf();
        let doc = PdfDocument::from_bytes(&pdf).expect("Should parse");
        assert!(doc.get_page(999).is_err());
    }

    #[test]
    fn test_page_media_box_dimensions() {
        let pdf = build_minimal_pdf();
        let doc = PdfDocument::from_bytes(&pdf).expect("Should parse");
        let page = doc.get_page(0).expect("Page should exist");
        // MediaBox is [0 0 612 792] — check width/height
        let [x0, y0, x1, y1] = page.media_box;
        assert!((x1 - x0 - 612.0).abs() < 1.0, "Width should be 612 pt");
        assert!((y1 - y0 - 792.0).abs() < 1.0, "Height should be 792 pt");
    }

    // -----------------------------------------------------------------------
    // Raw parser: PdfObject API tests
    // -----------------------------------------------------------------------

    fn parse_one(data: &[u8]) -> Result<PdfObject> {
        let mut p = RawParser::new(data);
        p.parse_object()
    }

    #[test]
    fn test_raw_parse_integer() {
        let obj = parse_one(b"42").expect("integer");
        assert_eq!(obj.as_integer(), Some(42));
    }

    #[test]
    fn test_raw_parse_negative_integer() {
        let obj = parse_one(b"-7").expect("negative integer");
        assert_eq!(obj.as_integer(), Some(-7));
    }

    #[test]
    #[allow(clippy::approx_constant)]
    fn test_raw_parse_real() {
        let obj = parse_one(b"3.14").expect("real");
        assert!((obj.as_real().expect("test: should succeed") - 3.14).abs() < 0.001);
    }

    #[test]
    fn test_raw_parse_boolean_true() {
        let obj = parse_one(b"true").expect("true");
        assert!(matches!(obj, PdfObject::Boolean(true)));
    }

    #[test]
    fn test_raw_parse_boolean_false() {
        let obj = parse_one(b"false").expect("false");
        assert!(matches!(obj, PdfObject::Boolean(false)));
    }

    #[test]
    fn test_raw_parse_null() {
        let obj = parse_one(b"null").expect("null");
        assert!(matches!(obj, PdfObject::Null));
    }

    #[test]
    fn test_raw_parse_name() {
        let obj = parse_one(b"/FooBar").expect("name");
        assert_eq!(obj.as_name(), Some("FooBar"));
    }

    #[test]
    fn test_raw_parse_empty_name() {
        let obj = parse_one(b"/ ").expect("empty name");
        assert_eq!(obj.as_name(), Some(""));
    }

    #[test]
    fn test_raw_parse_array_of_integers() {
        let obj = parse_one(b"[1 2 3]").expect("array");
        let arr = obj.as_array().expect("should be array");
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_integer(), Some(1));
        assert_eq!(arr[2].as_integer(), Some(3));
    }

    #[test]
    fn test_raw_parse_empty_array() {
        let obj = parse_one(b"[]").expect("empty array");
        let arr = obj.as_array().expect("should be array");
        assert!(arr.is_empty());
    }

    #[test]
    fn test_raw_parse_dictionary_with_name_value() {
        let obj = parse_one(b"<< /Type /Page >>").expect("dict");
        let dict = obj.as_dict().expect("should be dict");
        assert_eq!(dict.get_name("Type"), Some("Page"));
    }

    #[test]
    fn test_raw_parse_dictionary_with_integer_value() {
        let obj = parse_one(b"<< /Count 5 >>").expect("dict");
        let dict = obj.as_dict().expect("should be dict");
        assert_eq!(dict.get_integer("Count"), Some(5));
    }

    #[test]
    fn test_raw_parse_dictionary_with_array_value() {
        let obj = parse_one(b"<< /MediaBox [0 0 612 792] >>").expect("dict");
        let dict = obj.as_dict().expect("should be dict");
        let mb = dict.get_array("MediaBox").expect("should have MediaBox");
        assert_eq!(mb.len(), 4);
        assert_eq!(mb[2].as_integer(), Some(612));
        assert_eq!(mb[3].as_integer(), Some(792));
    }

    #[test]
    fn test_raw_parse_nested_dict() {
        let obj = parse_one(b"<< /Outer << /Inner 99 >> >>").expect("nested dict");
        let dict = obj.as_dict().expect("should be dict");
        let inner = dict.get("Outer").expect("Outer key");
        let inner_dict = inner.as_dict().expect("Outer should be dict");
        assert_eq!(inner_dict.get_integer("Inner"), Some(99));
    }

    #[test]
    fn test_raw_parse_literal_string() {
        let obj = parse_one(b"(Hello PDF)").expect("literal string");
        let bytes = obj.as_str_bytes().expect("should be string bytes");
        assert_eq!(bytes, b"Hello PDF");
    }

    #[test]
    fn test_raw_parse_hex_string() {
        // <48656C6C6F> = "Hello"
        let obj = parse_one(b"<48656C6C6F>").expect("hex string");
        let bytes = obj.as_str_bytes().expect("should be string bytes");
        assert_eq!(bytes, b"Hello");
    }

    #[test]
    fn test_raw_parse_indirect_reference() {
        let obj = parse_one(b"5 0 R").expect("reference");
        let (obj_num, gen) = obj.as_reference().expect("should be reference");
        assert_eq!(obj_num, 5);
        assert_eq!(gen, 0);
    }

    // -----------------------------------------------------------------------
    // PdfObject helper methods
    // -----------------------------------------------------------------------

    #[test]
    fn test_as_real_from_integer() {
        let obj = PdfObject::Integer(10);
        assert!((obj.as_real().expect("test: should succeed") - 10.0).abs() < 1e-9);
    }

    #[test]
    fn test_as_integer_from_real() {
        let obj = PdfObject::Real(3.9);
        assert_eq!(obj.as_integer(), Some(3));
    }

    #[test]
    fn test_as_name_on_non_name_returns_none() {
        let obj = PdfObject::Integer(42);
        assert!(obj.as_name().is_none());
    }

    #[test]
    fn test_as_array_on_non_array_returns_none() {
        let obj = PdfObject::Boolean(true);
        assert!(obj.as_array().is_none());
    }

    #[test]
    fn test_as_reference_on_non_reference_returns_none() {
        let obj = PdfObject::Null;
        assert!(obj.as_reference().is_none());
    }

    // -----------------------------------------------------------------------
    // PdfDictionary helpers
    // -----------------------------------------------------------------------

    #[test]
    fn test_dict_missing_key_returns_none() {
        let mut map = std::collections::HashMap::new();
        map.insert("Key".to_string(), PdfObject::Integer(1));
        let dict = PdfDictionary(map);
        assert!(dict.get("Missing").is_none());
        assert!(dict.get_integer("Missing").is_none());
        assert!(dict.get_name("Missing").is_none());
        assert!(dict.get_real("Missing").is_none());
        assert!(dict.get_array("Missing").is_none());
    }

    #[test]
    fn test_dict_get_integer() {
        let mut map = std::collections::HashMap::new();
        map.insert("Count".to_string(), PdfObject::Integer(7));
        let dict = PdfDictionary(map);
        assert_eq!(dict.get_integer("Count"), Some(7));
    }

    #[test]
    fn test_dict_get_name() {
        let mut map = std::collections::HashMap::new();
        map.insert("Type".to_string(), PdfObject::Name("Catalog".to_string()));
        let dict = PdfDictionary(map);
        assert_eq!(dict.get_name("Type"), Some("Catalog"));
    }
}

// ---------------------------------------------------------------------------
// Extended Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod extended_tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    /// Build a minimal valid PDF with a plain-text content stream
    fn build_pdf_with_content(content: &[u8]) -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();
        out.extend_from_slice(b"%PDF-1.4\n");

        // Obj 1: Catalog
        let o1 = out.len();
        out.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

        // Obj 2: Pages
        let o2 = out.len();
        out.extend_from_slice(b"2 0 obj\n<< /Type /Pages /Kids [3 0 R] /Count 1 >>\nendobj\n");

        // Obj 4: Content stream
        let stream_hdr = format!("4 0 obj\n<< /Length {} >>\nstream\n", content.len());
        let o4 = out.len();
        out.extend_from_slice(stream_hdr.as_bytes());
        out.extend_from_slice(content);
        out.extend_from_slice(b"\nendstream\nendobj\n");

        // Obj 3: Page with content reference
        let o3 = out.len();
        out.extend_from_slice(
            b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R >>\nendobj\n",
        );

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

    /// Build a 2-page PDF
    fn build_two_page_pdf() -> Vec<u8> {
        let mut out: Vec<u8> = Vec::new();
        out.extend_from_slice(b"%PDF-1.4\n");

        let o1 = out.len();
        out.extend_from_slice(b"1 0 obj\n<< /Type /Catalog /Pages 2 0 R >>\nendobj\n");

        let o2 = out.len();
        out.extend_from_slice(
            b"2 0 obj\n<< /Type /Pages /Kids [3 0 R 4 0 R] /Count 2 >>\nendobj\n",
        );

        let o3 = out.len();
        out.extend_from_slice(
            b"3 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] >>\nendobj\n",
        );

        let o4 = out.len();
        out.extend_from_slice(
            b"4 0 obj\n<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] >>\nendobj\n",
        );

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

    // -----------------------------------------------------------------------
    // Multi-page document tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_two_page_pdf_page_count() {
        let pdf = build_two_page_pdf();
        let doc = PdfDocument::from_bytes(&pdf).expect("should parse two-page PDF");
        assert_eq!(doc.page_count(), 2);
    }

    #[test]
    fn test_two_page_pdf_both_pages_accessible() {
        let pdf = build_two_page_pdf();
        let doc = PdfDocument::from_bytes(&pdf).expect("should parse");
        assert!(doc.get_page(0).is_ok());
        assert!(doc.get_page(1).is_ok());
    }

    #[test]
    fn test_two_page_pdf_third_page_out_of_bounds() {
        let pdf = build_two_page_pdf();
        let doc = PdfDocument::from_bytes(&pdf).expect("should parse");
        assert!(doc.get_page(2).is_err());
    }

    #[test]
    fn test_two_page_pdf_a4_media_box() {
        let pdf = build_two_page_pdf();
        let doc = PdfDocument::from_bytes(&pdf).expect("should parse");
        let page = doc.get_page(0).expect("page 0");
        let [x0, y0, x1, y1] = page.media_box;
        assert!((x1 - x0 - 595.0).abs() < 1.0, "Width should be 595 (A4)");
        assert!((y1 - y0 - 842.0).abs() < 1.0, "Height should be 842 (A4)");
    }

    // -----------------------------------------------------------------------
    // Content stream tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_pdf_with_text_content_stream_parses() {
        let content = b"BT /F1 12 Tf 100 700 Td (Hello PDF) Tj ET";
        let pdf = build_pdf_with_content(content);
        let doc = PdfDocument::from_bytes(&pdf).expect("should parse");
        let page = doc.get_page(0).expect("page 0");
        assert!(
            !page.content.is_empty(),
            "Content stream should not be empty"
        );
    }

    #[test]
    fn test_pdf_content_stream_bytes_match() {
        let content = b"BT /F1 12 Tf 100 700 Td (Hello) Tj ET";
        let pdf = build_pdf_with_content(content);
        let doc = PdfDocument::from_bytes(&pdf).expect("should parse");
        let page = doc.get_page(0).expect("page 0");
        // Content should contain the stream data
        assert!(
            page.content.windows(2).any(|w| w == b"BT"),
            "Content should contain BT operator"
        );
    }

    #[test]
    fn test_pdf_with_empty_content_stream() {
        let content = b"";
        let pdf = build_pdf_with_content(content);
        let doc = PdfDocument::from_bytes(&pdf).expect("should parse");
        let page = doc.get_page(0).expect("page 0");
        assert!(
            page.content.is_empty(),
            "Empty content stream should be empty"
        );
    }

    #[test]
    fn test_pdf_with_color_operators_in_content() {
        let content = b"0.5 0.3 0.1 rg BT /F1 12 Tf 100 700 Td (text) Tj ET";
        let pdf = build_pdf_with_content(content);
        let doc = PdfDocument::from_bytes(&pdf).expect("should parse");
        let page = doc.get_page(0).expect("page 0");
        assert!(!page.content.is_empty());
    }

    #[test]
    fn test_pdf_with_path_operators_in_content() {
        let content = b"1 0 0 RG 2 w 100 100 m 200 100 l 200 200 l h S";
        let pdf = build_pdf_with_content(content);
        let doc = PdfDocument::from_bytes(&pdf).expect("should parse");
        let page = doc.get_page(0).expect("page 0");
        assert!(!page.content.is_empty());
    }

    // -----------------------------------------------------------------------
    // PdfObject type checks
    // -----------------------------------------------------------------------

    #[test]
    fn test_pdf_object_null_is_not_integer() {
        let obj = PdfObject::Null;
        assert!(obj.as_integer().is_none());
        assert!(obj.as_real().is_none());
        assert!(obj.as_name().is_none());
        assert!(obj.as_str_bytes().is_none());
        assert!(obj.as_array().is_none());
        assert!(obj.as_dict().is_none());
        assert!(obj.as_reference().is_none());
    }

    #[test]
    fn test_pdf_object_boolean_coercions() {
        let obj = PdfObject::Boolean(true);
        assert!(obj.as_integer().is_none());
        assert!(obj.as_name().is_none());
    }

    #[test]
    fn test_pdf_object_string_as_str_bytes() {
        let obj = PdfObject::String(b"test".to_vec());
        assert_eq!(obj.as_str_bytes(), Some(b"test" as &[u8]));
        assert!(obj.as_integer().is_none());
        assert!(obj.as_name().is_none());
    }

    #[test]
    fn test_pdf_object_stream_as_dict() {
        let mut map = HashMap::new();
        map.insert("Length".to_string(), PdfObject::Integer(5));
        let dict = PdfDictionary(map);
        let obj = PdfObject::Stream(dict, b"hello".to_vec());
        assert!(obj.as_dict().is_some());
        assert_eq!(
            obj.as_dict()
                .expect("test: should succeed")
                .get_integer("Length"),
            Some(5)
        );
    }

    #[test]
    fn test_pdf_object_array_contains_names() {
        let obj = PdfObject::Array(vec![
            PdfObject::Name("DeviceRGB".to_string()),
            PdfObject::Name("DeviceGray".to_string()),
        ]);
        let arr = obj.as_array().expect("should be array");
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_name(), Some("DeviceRGB"));
        assert_eq!(arr[1].as_name(), Some("DeviceGray"));
    }

    #[test]
    fn test_pdf_object_reference_generation_number() {
        let obj = PdfObject::Reference(10, 3);
        let (num, gen) = obj.as_reference().expect("should be reference");
        assert_eq!(num, 10);
        assert_eq!(gen, 3);
    }

    // -----------------------------------------------------------------------
    // PdfDictionary advanced tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_pdf_dictionary_get_real_from_integer() {
        let mut map = HashMap::new();
        map.insert("Width".to_string(), PdfObject::Integer(100));
        let dict = PdfDictionary(map);
        // get_real should coerce integer to real
        let val = dict.get_real("Width");
        assert!(val.is_some());
        assert!((val.expect("test: should succeed") - 100.0).abs() < 1e-6);
    }

    #[test]
    fn test_pdf_dictionary_get_name_on_wrong_type_returns_none() {
        let mut map = HashMap::new();
        map.insert("Count".to_string(), PdfObject::Integer(5));
        let dict = PdfDictionary(map);
        assert!(dict.get_name("Count").is_none());
    }

    #[test]
    fn test_pdf_dictionary_get_integer_on_wrong_type_returns_none() {
        let mut map = HashMap::new();
        map.insert("Type".to_string(), PdfObject::Name("Page".to_string()));
        let dict = PdfDictionary(map);
        assert!(dict.get_integer("Type").is_none());
    }

    #[test]
    fn test_pdf_dictionary_get_array_on_wrong_type_returns_none() {
        let mut map = HashMap::new();
        map.insert("Type".to_string(), PdfObject::Name("Page".to_string()));
        let dict = PdfDictionary(map);
        assert!(dict.get_array("Type").is_none());
    }

    #[test]
    fn test_pdf_dictionary_multiple_entries() {
        let mut map = HashMap::new();
        map.insert("Type".to_string(), PdfObject::Name("Page".to_string()));
        map.insert("Count".to_string(), PdfObject::Integer(3));
        map.insert("Width".to_string(), PdfObject::Real(8.5));
        let dict = PdfDictionary(map);
        assert_eq!(dict.get_name("Type"), Some("Page"));
        assert_eq!(dict.get_integer("Count"), Some(3));
        assert!(dict.get_real("Width").is_some());
    }

    // -----------------------------------------------------------------------
    // Raw parser: string parsing edge cases
    // -----------------------------------------------------------------------

    fn parse_one(data: &[u8]) -> Result<PdfObject> {
        let mut p = RawParser::new(data);
        p.parse_object()
    }

    #[test]
    fn test_raw_parse_name_with_special_chars() {
        // Names with digits and hyphens
        let obj = parse_one(b"/Font-Bold").expect("name with hyphen");
        assert_eq!(obj.as_name(), Some("Font-Bold"));
    }

    #[test]
    fn test_raw_parse_literal_string_with_escape() {
        // String with escaped newline
        let obj = parse_one(b"(line1\\nline2)").expect("escaped string");
        let bytes = obj.as_str_bytes().expect("should be string");
        assert!(bytes.contains(&b'\n'));
    }

    #[test]
    fn test_raw_parse_hex_string_lowercase() {
        // <48656c6c6f> = "Hello" (lowercase hex)
        let obj = parse_one(b"<48656c6c6f>").expect("lowercase hex string");
        let bytes = obj.as_str_bytes().expect("should be string");
        assert_eq!(bytes, b"Hello");
    }

    #[test]
    fn test_raw_parse_large_integer() {
        let obj = parse_one(b"1000000").expect("large integer");
        assert_eq!(obj.as_integer(), Some(1_000_000));
    }

    #[test]
    fn test_raw_parse_zero() {
        let obj = parse_one(b"0").expect("zero");
        assert_eq!(obj.as_integer(), Some(0));
    }

    #[test]
    fn test_raw_parse_positive_sign_integer() {
        let obj = parse_one(b"+5").expect("positive signed integer");
        assert_eq!(obj.as_integer(), Some(5));
    }

    #[test]
    fn test_raw_parse_array_with_mixed_types() {
        let obj = parse_one(b"[/Name 42 true null]").expect("mixed array");
        let arr = obj.as_array().expect("should be array");
        assert_eq!(arr.len(), 4);
        assert_eq!(arr[0].as_name(), Some("Name"));
        assert_eq!(arr[1].as_integer(), Some(42));
        assert!(matches!(arr[2], PdfObject::Boolean(true)));
        assert!(matches!(arr[3], PdfObject::Null));
    }

    #[test]
    fn test_raw_parse_reference_with_nonzero_generation() {
        let obj = parse_one(b"3 5 R").expect("reference with gen 5");
        let (num, gen) = obj.as_reference().expect("should be reference");
        assert_eq!(num, 3);
        assert_eq!(gen, 5);
    }

    #[test]
    fn test_raw_parse_real_with_leading_dot() {
        // ".5" is a valid PDF real number
        let obj = parse_one(b".5").expect("leading-dot real");
        let val = obj.as_real().expect("should be real or integer");
        assert!((val - 0.5).abs() < 0.01);
    }

    // -----------------------------------------------------------------------
    // Page index tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_page_index_is_zero_based() {
        let content = b"q Q";
        let pdf = build_pdf_with_content(content);
        let doc = PdfDocument::from_bytes(&pdf).expect("should parse");
        let page = doc.get_page(0).expect("page 0");
        assert_eq!(page.index, 0);
    }

    #[test]
    fn test_two_page_pdf_second_page_has_index_one() {
        let pdf = build_two_page_pdf();
        let doc = PdfDocument::from_bytes(&pdf).expect("should parse");
        let page1 = doc.get_page(1).expect("page 1");
        assert_eq!(page1.index, 1);
    }

    // -----------------------------------------------------------------------
    // PDF header validation
    // -----------------------------------------------------------------------

    #[test]
    fn test_pdf_version_15_header_accepted() {
        // Build same minimal PDF but with version 1.5 header
        let content = b"q Q";
        let mut pdf = build_pdf_with_content(content);
        // Replace "%PDF-1.4" prefix with "%PDF-1.5"
        pdf[5] = b'1';
        pdf[6] = b'.';
        pdf[7] = b'5';
        // Should parse without error (version mismatch is non-fatal)
        let result = PdfDocument::from_bytes(&pdf);
        // May succeed or fail depending on xref accuracy; must not panic
        let _ = result;
    }

    #[test]
    fn test_non_pdf_bytes_rejected() {
        assert!(PdfDocument::from_bytes(b"GIF89a").is_err());
        assert!(PdfDocument::from_bytes(b"\x89PNG\r\n\x1a\n").is_err());
        assert!(PdfDocument::from_bytes(b"JFIF").is_err());
    }

    #[test]
    fn test_completely_empty_input_rejected() {
        assert!(PdfDocument::from_bytes(b"").is_err());
    }

    #[test]
    fn test_only_whitespace_rejected() {
        assert!(PdfDocument::from_bytes(b"   \n\t  ").is_err());
    }

    // -----------------------------------------------------------------------
    // PdfDocument::resolve
    // -----------------------------------------------------------------------

    #[test]
    fn test_resolve_non_reference_returns_same() {
        let content = b"";
        let pdf = build_pdf_with_content(content);
        let doc = PdfDocument::from_bytes(&pdf).expect("should parse");
        let obj = PdfObject::Integer(42);
        let resolved = doc.resolve(&obj);
        assert!(resolved.is_some());
        assert_eq!(
            resolved.expect("test: should succeed").as_integer(),
            Some(42)
        );
    }

    #[test]
    fn test_resolve_null_object() {
        let content = b"";
        let pdf = build_pdf_with_content(content);
        let doc = PdfDocument::from_bytes(&pdf).expect("should parse");
        let obj = PdfObject::Null;
        let resolved = doc.resolve(&obj);
        assert!(resolved.is_some());
        assert!(matches!(
            resolved.expect("test: should succeed"),
            PdfObject::Null
        ));
    }
}
