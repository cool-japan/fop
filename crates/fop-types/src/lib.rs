//! Core types for Apache FOP Rust implementation
//!
//! This crate provides foundational types used throughout the FOP system:
//! - Length: Dimensional measurements with various units
//! - Color: RGBA color values
//! - Geometry: Points, rectangles, and sizes
//! - Errors: Base error types

pub mod color;
pub mod error;
pub mod expression;
pub mod font_metrics;
pub mod geometry;
pub mod gradient;
pub mod length;
pub mod percentage;

pub use color::Color;
pub use error::{FopError, Location, Result};
pub use expression::{EvalContext, Expression};
pub use font_metrics::{FontMetrics, FontRegistry};
pub use geometry::{Point, Rect, Size};
pub use gradient::{ColorStop, Gradient};
pub use length::{FontContext, Length, LengthUnit};
pub use percentage::Percentage;
