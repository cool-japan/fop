//! Layout engine for Apache FOP
//!
//! This crate implements the layout algorithms that transform the FO tree
//! into an area tree suitable for rendering.
//!
//! # Architecture
//!
//! The layout process follows these steps:
//!
//! 1. **Area Tree Generation**: Convert FO tree to area tree
//! 2. **Block Layout**: Position block-level areas vertically
//! 3. **Inline Layout**: Position inline-level areas horizontally with line breaking
//! 4. **Page Breaking**: Split content across pages
//!
//! # Example
//!
//! ```no_run
//! use fop_layout::LayoutEngine;
//! use fop_core::FoArena;
//!
//! // Assume we have an FO tree in `arena`
//! # let arena = FoArena::new();
//! let engine = LayoutEngine::new();
//! let area_tree = engine.layout(&arena).unwrap();
//! ```

pub mod area;
pub mod layout;

pub use area::{Area, AreaId, AreaTree, AreaType};
pub use fop_types::{FopError, Result};
pub use layout::properties::measure_text_width;
pub use layout::{
    ColumnWidth, KnuthPlassBreaker, LayoutEngine, ListLayout, ListMarkerStyle, PageBreaker,
    TableLayout,
};
