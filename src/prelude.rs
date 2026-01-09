//! Convenience re-exports for building plots.

// Core plot builder
pub use crate::plot::plot;

// Common geoms (extend as needed)
pub use crate::geom::point::geom_point;
pub use crate::geom::line::geom_line;
pub use crate::geom::bar::geom_bar;
pub use crate::geom::density::geom_density;
pub use crate::geom::smooth::geom_smooth;
pub use crate::geom::histogram::geom_histogram;
pub use crate::geom::boxplot::geom_boxplot;
pub use crate::geom::hline::geom_hline;
pub use crate::geom::vline::geom_vline;
pub use crate::geom::rect::geom_rect;
pub use crate::geom::segment::geom_segment;
pub use crate::geom::text::geom_text;
pub use crate::geom::label::geom_label;

// Aesthetic builder traits (x/y/color/size/etc.)
pub use crate::aesthetics::builder::*;

// Theme helpers and element trait setters (color/size/etc.)
pub use crate::theme::{color, traits::*};

// Visual primitives
pub use crate::visuals::{LineStyle, Shape};

// Data utilities
pub use crate::data::{DataSource, IStr, VectorValue};
pub use crate::utils::dataframe::{BoolVec, DataFrame, FloatVec, IntVec, StrVec};

