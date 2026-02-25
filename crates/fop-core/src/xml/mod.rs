//! XML parsing utilities for XSL-FO documents

pub mod namespace;
pub mod parser;

pub use namespace::Namespace;
pub use parser::{EntityResolver, ProcessingInstruction, XmlParser};
