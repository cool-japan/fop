//! FO tree data structures
//!
//! This module provides the tree structure for storing formatting objects.
//! Uses arena allocation with index-based handles for memory efficiency.
//!
//! # Architecture
//!
//! - **Arena allocation**: All nodes stored in contiguous memory (cache-friendly)
//! - **Index-based handles**: NodeId is just usize (no Rc/RefCell overhead)
//! - **Tree builder**: SAX-like parser that constructs tree from XML events
//! - **Validation**: Element nesting rules enforcement
//!
//! # Example
//!
//! ```no_run
//! use fop_core::FoTreeBuilder;
//! use std::io::Cursor;
//!
//! let xml = r#"<?xml version="1.0"?>
//! <fo:root xmlns:fo="http://www.w3.org/1999/XSL/Format">
//!     <fo:layout-master-set>
//!         <fo:simple-page-master master-name="A4">
//!             <fo:region-body/>
//!         </fo:simple-page-master>
//!     </fo:layout-master-set>
//! </fo:root>"#;
//!
//! let builder = FoTreeBuilder::new();
//! let arena = builder.parse(Cursor::new(xml)).unwrap();
//!
//! println!("Parsed {} nodes", arena.len());
//! ```

pub mod arena;
pub mod builder;
pub mod id_registry;
pub mod node;
pub mod validation;

pub use arena::{FoArena, NodeId};
pub use builder::FoTreeBuilder;
pub use id_registry::IdRegistry;
pub use node::{BlankOrNotBlank, FoNode, FoNodeData, OddOrEven, PagePosition, RetrievePosition};
pub use validation::NestingValidator;
