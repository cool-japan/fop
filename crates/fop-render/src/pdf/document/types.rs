//! Core PDF type definitions
//!
//! Defines the fundamental data structures used throughout the PDF document module.

use std::collections::HashMap;

use fop_types::{Gradient, Length};

use crate::pdf::compliance::PdfCompliance;
use crate::pdf::font::FontManager;
use crate::pdf::image::ImageXObject;
use crate::pdf::security::EncryptionDict;

/// PDF gradient shading object
#[derive(Clone)]
pub struct PdfGradient {
    /// Gradient definition
    pub gradient: Gradient,
    /// PDF object ID (assigned during document generation)
    pub object_id: usize,
}

/// PDF Extended Graphics State (ExtGState) for transparency
#[derive(Clone, Debug)]
pub struct PdfExtGState {
    /// Fill opacity (0.0 = transparent, 1.0 = opaque)
    pub fill_opacity: f64,
    /// Stroke opacity (0.0 = transparent, 1.0 = opaque)
    pub stroke_opacity: f64,
    /// PDF object ID (assigned during document generation)
    pub object_id: usize,
}

/// PDF outline (bookmarks) structure
#[derive(Debug, Clone)]
pub struct PdfOutline {
    /// Top-level outline items
    pub items: Vec<PdfOutlineItem>,
}

/// PDF outline item (bookmark)
#[derive(Debug, Clone)]
pub struct PdfOutlineItem {
    /// Bookmark title
    pub title: String,

    /// Internal destination (page index for now)
    pub page_index: Option<usize>,

    /// External destination URL
    pub external_destination: Option<String>,

    /// Nested child bookmarks
    pub children: Vec<PdfOutlineItem>,
}

/// PDF object (indirect object)
#[derive(Debug, Clone)]
pub struct PdfObject {
    /// Object number
    pub id: u32,

    /// Generation number (usually 0)
    pub generation: u16,

    /// Object content
    pub content: PdfValue,
}

/// PDF value types
#[derive(Debug, Clone)]
pub enum PdfValue {
    Null,
    Boolean(bool),
    Integer(i32),
    Real(f64),
    String(String),
    Name(String),
    Array(Vec<PdfValue>),
    Dictionary(HashMap<String, PdfValue>),
    Stream(Vec<u8>),
}

/// Link annotation (hyperlink)
#[derive(Debug, Clone)]
pub struct LinkAnnotation {
    /// Rectangle of the clickable area (in PDF coordinates: bottom-left origin)
    pub rect: [f64; 4], // [x, y, width, height]

    /// Link destination
    pub destination: LinkDestination,
}

/// Link destination type
#[derive(Debug, Clone)]
pub enum LinkDestination {
    /// External URL
    External(String),

    /// Internal destination (element ID)
    Internal(String),
}

/// PDF page
pub struct PdfPage {
    /// Page width
    pub width: Length,

    /// Page height
    pub height: Length,

    /// Page content stream
    pub content: Vec<u8>,

    /// Link annotations on this page
    pub link_annotations: Vec<LinkAnnotation>,
}

/// PDF document information
#[derive(Debug, Clone, Default)]
pub struct PdfInfo {
    /// Document title
    pub title: Option<String>,

    /// Document author
    pub author: Option<String>,

    /// Document subject
    pub subject: Option<String>,

    /// Creation date
    pub creation_date: Option<String>,

    /// Document language (BCP 47 language tag, e.g. "en", "ja")
    /// Written as /Lang in the PDF catalog
    pub lang: Option<String>,
}

/// PDF document structure
pub struct PdfDocument {
    /// PDF version (e.g., "1.4")
    pub version: String,

    /// Objects in the PDF
    pub objects: Vec<PdfObject>,

    /// Page tree
    pub pages: Vec<PdfPage>,

    /// Document info
    pub info: PdfInfo,

    /// Image XObjects (shared resources)
    pub image_xobjects: Vec<ImageXObject>,

    /// Gradient shading patterns (shared resources)
    pub gradients: Vec<PdfGradient>,

    /// Extended Graphics States for transparency
    pub ext_g_states: Vec<PdfExtGState>,

    /// Document outline (bookmarks)
    pub outline: Option<PdfOutline>,

    /// Font manager for embedded fonts
    pub font_manager: FontManager,

    /// Encryption settings (if set, all streams/strings will be encrypted)
    pub encryption: Option<EncryptionDict>,

    /// File ID for the /ID entry in the trailer (required for encryption)
    pub file_id: Option<Vec<u8>>,

    /// PDF compliance mode (Standard, PDF/A-1b, PDF/UA-1, or both)
    pub compliance: PdfCompliance,
}
