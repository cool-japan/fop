//! Core FO tree parsing and property system for Apache FOP
//!
//! This crate provides the foundational components for parsing XSL-FO documents
//! and building the Formatting Object tree.
//!
//! # Features
//!
//! - **294 XSL-FO 1.1 properties** with type-safe enums
//! - **Property inheritance** with caching for performance
//! - **Arena-allocated tree** for zero-overhead node handles
//! - **SAX-like parsing** for streaming XML processing
//! - **29 FO element types** covering layout, blocks, tables, lists
//! - **Shorthand expansion** for margin, padding, border properties
//! - **Element nesting validation** per XSL-FO 1.1 spec
//!
//! # Quick Start
//!
//! ```no_run
//! use fop_core::FoTreeBuilder;
//! use std::io::Cursor;
//!
//! let xml = r#"<?xml version="1.0"?>
//! <fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
//!     <fo:layout-master-set>
//!         <fo:simple-page-master master-name="A4" page-width="210mm" page-height="297mm">
//!             <fo:region-body margin="1in"/>
//!         </fo:simple-page-master>
//!     </fo:layout-master-set>
//!     <fo:page-sequence master-reference="A4">
//!         <fo:flow flow-name="xsl-region-body">
//!             <fo:block font-size="14pt">Hello, FOP!</fo:block>
//!         </fo:flow>
//!     </fo:page-sequence>
//! </fo:root>"#;
//!
//! let builder = FoTreeBuilder::new();
//! let arena = builder.parse(Cursor::new(xml)).unwrap();
//!
//! for (id, node) in arena.iter() {
//!     println!("{}: {}", id, node.data.element_name());
//! }
//! ```
//!
//! # Architecture
//!
//! The crate is organized into several modules:
//!
//! - [`properties`] - Property system (IDs, values, lists, inheritance)
//! - [`tree`] - FO tree structure (arena, nodes, builder)
//! - [`xml`] - XML parsing utilities (parser, namespaces)

pub mod properties;
pub mod tree;
pub mod xml;

pub use fop_types::{
    Color, ColorStop, FopError, Gradient, Length, Percentage, Point, Rect, Result, Size,
};
pub use properties::{
    PropertyId, PropertyList, PropertyValidator, PropertyValue, RelativeFontSize, ShorthandExpander,
};
pub use tree::{FoArena, FoNode, FoNodeData, FoTreeBuilder, IdRegistry, NestingValidator, NodeId};
pub use xml::{Namespace, XmlParser};
