//! PDF rendering backend
//!
//! Generates PDF documents from area trees.

pub mod cidfont;
pub mod compliance;
pub mod document;
pub mod font;
pub mod font_config;
pub mod graphics;
pub mod image;
pub mod outline;
pub mod security;
pub mod streaming;
pub mod validator;
pub mod writer;

pub use compliance::PdfCompliance;
pub use document::PdfDocument;
pub use font::{FontManager, PdfFont};
pub use font_config::FontConfig;
pub use graphics::PdfGraphics;
pub use image::ImageXObject;
pub use outline::extract_outline_from_fo_tree;
pub use security::{EncryptionAlgorithm, EncryptionDict, PdfPermissions, PdfSecurity};
pub use streaming::StreamingPdfRenderer;
pub use validator::{PdfValidator, ValidationResult};
pub use writer::PdfRenderer;
