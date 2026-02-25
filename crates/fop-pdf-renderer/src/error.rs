//! Error types for fop-pdf-renderer

use thiserror::Error;

/// Errors that can occur during PDF rendering
#[derive(Debug, Error)]
pub enum PdfRenderError {
    /// PDF structure parsing error
    #[error("PDF parse error: {0}")]
    Parse(String),

    /// Unsupported PDF feature
    #[error("Unsupported PDF feature: {0}")]
    Unsupported(String),

    /// Page index out of bounds
    #[error("Page {0} not found (document has {1} pages)")]
    PageNotFound(usize, usize),

    /// Font error
    #[error("Font error: {0}")]
    Font(String),

    /// Image decoding error
    #[error("Image error: {0}")]
    Image(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Decompression error
    #[error("Decompression error: {0}")]
    Decompress(String),
}

/// Result type for PDF rendering operations
pub type Result<T> = std::result::Result<T, PdfRenderError>;
