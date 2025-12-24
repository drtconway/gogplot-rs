use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::data::{DataSource, VectorType};
use crate::error::{DataType, PlotError};
use crate::layer::Layer;
use crate::scale::{ScaleSet, ScaleType};
use crate::scale::traits::ScaleBase;
use crate::theme::{self, Color};
use cairo::Context;
use ordered_float::OrderedFloat;

/// Enum to hold aesthetic values that can be either borrowed or owned
pub enum AestheticValues<'a> {
    /// Borrowed float slice iterator
    FloatRef(std::slice::Iter<'a, f64>),
    /// Owned vector (needed for type conversions or scale applications)
    Owned(Vec<f64>),
    /// Constant value repeated n times
    Constant(f64, usize),
}

impl<'a> Iterator for AestheticValues<'a> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            AestheticValues::FloatRef(iter) => iter.next().copied(),
            AestheticValues::Owned(vec) => {
                if vec.is_empty() {
                    None
                } else {
                    Some(vec.remove(0))
                }
            }
            AestheticValues::Constant(value, n) => {
                if *n > 0 {
                    *n -= 1;
                    Some(*value)
                } else {
                    None
                }
            }
        }
    }
}

/// Enum to hold color values
pub enum ColorValues {
    /// Constant color repeated n times
    Constant(Color, usize),
    /// Mapped colors from data
    Mapped(Vec<Color>),
}

impl Iterator for ColorValues {
    type Item = Color;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ColorValues::Constant(color, n) => {
                if *n > 0 {
                    *n -= 1;
                    Some(*color)
                } else {
                    None
                }
            }
            ColorValues::Mapped(vec) => {
                if vec.is_empty() {
                    None
                } else {
                    Some(vec.remove(0))
                }
            }
        }
    }
}

/// Enum to hold shape values
pub enum ShapeValues {
    /// Constant shape repeated n times
    Constant(crate::visuals::Shape, usize),
    /// Mapped shapes from data
    Mapped(Vec<crate::visuals::Shape>),
}

impl Iterator for ShapeValues {
    type Item = crate::visuals::Shape;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ShapeValues::Constant(shape, n) => {
                if *n > 0 {
                    *n -= 1;
                    Some(*shape)
                } else {
                    None
                }
            }
            ShapeValues::Mapped(vec) => {
                if vec.is_empty() {
                    None
                } else {
                    Some(vec.remove(0))
                }
            }
        }
    }
}

/// Encapsulates the rendering context needed by geoms
/// Layer handles data/mapping/grouping, geom just renders with this context
pub struct RenderContext<'a> {
    /// Cairo drawing context
    pub cairo: &'a mut Context,

    /// Theme for styling
    pub theme: &'a theme::Theme,

    /// X viewport range (min, max) in device coordinates
    pub x_range: (f64, f64),

    /// Y viewport range (min, max) in device coordinates
    pub y_range: (f64, f64),
}

impl<'a> RenderContext<'a> {
    pub fn new(
        cairo: &'a mut Context,
        theme: &'a theme::Theme,
        x_range: (f64, f64),
        y_range: (f64, f64),
    ) -> Self {
        Self {
            cairo,
            theme,
            x_range,
            y_range,
        }
    }

    /// Map normalized [0, 1] x-coordinate to viewport coordinate
    pub fn map_x(&self, normalized: f64) -> f64 {
        let (x0, x1) = self.x_range;
        x0 + normalized * (x1 - x0)
    }

    /// Map normalized [0, 1] y-coordinate to viewport coordinate
    pub fn map_y(&self, normalized: f64) -> f64 {
        let (y0, y1) = self.y_range;
        y0 + normalized * (y1 - y0)
    }

}

pub(crate) fn compute_min_spacing(aesthetic_values: AestheticValues<'_>, width: f64) -> f64 {
    let mut x_set: Vec<OrderedFloat<f64>> = aesthetic_values
        .filter(|x| x.is_finite())
        .map(|x| OrderedFloat(x))
        .collect();
    x_set.sort();
    x_set.dedup();
    let x_set: Vec<f64> = x_set.into_iter().map(|of| of.0).collect();

    if x_set.len() > 1 {
        // Find minimum spacing between consecutive x values
        let mut min_spacing = f64::MAX;
        for i in 1..x_set.len() {
            let spacing = x_set[i] - x_set[i - 1];
            if spacing < min_spacing {
                min_spacing = spacing;
            }
        }
        min_spacing * width / 2.0
    } else {
        0.05 // Single bar fallback half-width
    }
}
