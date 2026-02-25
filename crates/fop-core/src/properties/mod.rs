//! Property system for XSL-FO
//!
//! This module provides the property system including:
//! - Property IDs and names (294 properties from XSL-FO 1.1)
//! - Property values (Length, Color, Enum, String, etc.)
//! - Property inheritance (parent chain traversal with caching)
//! - Property lists (sparse array storage with O(1) access)
//! - Shorthand expansion (margin → margin-top/right/bottom/left)
//! - Property validation (range checks, enum values, type constraints)
//! - Initial values (default values for all properties)

pub mod initial_values;
pub mod property_id;
pub mod property_list;
pub mod property_value;
pub mod shorthand;
pub mod validation;

pub use initial_values::get_initial_value;
pub use property_id::PropertyId;
pub use property_list::PropertyList;
pub use property_value::{PropertyValue, RelativeFontSize};
pub use shorthand::ShorthandExpander;
pub use validation::PropertyValidator;
