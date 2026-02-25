//! Error types for FOP

use thiserror::Error;

/// Result type alias for FOP operations
pub type Result<T> = std::result::Result<T, FopError>;

/// Location information for errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

impl Location {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    pub fn unknown() -> Self {
        Self { line: 0, column: 0 }
    }
}

impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.line == 0 && self.column == 0 {
            write!(f, "unknown location")
        } else {
            write!(f, "line {}, column {}", self.line, self.column)
        }
    }
}

/// Base error type for FOP operations
#[derive(Error, Debug)]
pub enum FopError {
    /// XML parsing error with location
    #[error("XML parsing error at {location}: {message}")]
    XmlErrorWithLocation {
        message: String,
        location: Location,
        suggestion: Option<String>,
    },

    /// XML parsing error (without location)
    #[error("XML parsing error: {0}")]
    XmlError(String),

    /// Entity resolution error
    #[error("Entity resolution error at {location}: {message}")]
    EntityError { message: String, location: Location },

    /// Invalid property value
    #[error("Invalid property value for {property}: {value}")]
    InvalidPropertyValue { property: String, value: String },

    /// Unknown property
    #[error("Unknown property: {0}")]
    UnknownProperty(String),

    /// Invalid element
    #[error("Invalid element: {0}")]
    InvalidElement(String),

    /// Element nesting error
    #[error("Invalid element nesting: {child} cannot be a child of {parent}")]
    InvalidNesting { parent: String, child: String },

    /// Missing required attribute
    #[error("Missing required attribute {attribute} on element {element}")]
    MissingAttribute { element: String, attribute: String },

    /// Property validation error
    #[error("Property validation error for {property}: {value} - {reason}")]
    PropertyValidation {
        property: String,
        value: String,
        reason: String,
    },

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Generic error
    #[error("{0}")]
    Generic(String),
}
