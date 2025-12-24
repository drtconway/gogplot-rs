use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic, AestheticProperty};
use crate::data::PrimitiveValue;
use crate::error::PlotError;
use crate::geom::properties::{ColorProperty, FloatProperty, PropertyVector};
use crate::layer::Layer;
use crate::scale::traits::{ContinuousRangeScale, ScaleBase};
use crate::utils::data::make_float_iter;
use crate::visuals::LineStyle;

/// GeomVLine renders vertical reference lines at specified x-intercepts
///
/// The x-intercept is specified via the XIntercept aesthetic mapping.
pub struct GeomVLine {
    pub x_intercept: Option<PrimitiveValue>,

    /// Default line color
    pub color: ColorProperty,

    /// Default line width
    pub size: FloatProperty,

    /// Default alpha/opacity
    pub alpha: FloatProperty,

    /// Default line style pattern
    pub linetype: Option<AesValue>,
}

impl GeomVLine {
    /// Create a new vertical line geom
    ///
    /// X-intercept should be specified via aesthetic mapping:
    /// - Constant: `.aes(|a| a.xintercept_const(value))`
    /// - Column: `.aes(|a| a.xintercept("column_name"))`
    pub fn new() -> Self {
        Self {
            x_intercept: None,
            color: ColorProperty::new(),
            size: FloatProperty::new(),
            alpha: FloatProperty::new(),
            linetype: None,
        }
    }

    /// Set the line style pattern
    pub fn linetype(&mut self, pattern: impl Into<String>) -> &mut Self {
        self.linetype = Some(AesValue::constant(PrimitiveValue::Str(pattern.into())));
        self
    }

    fn get_x_intercept<'a>(&'a self, layer: &'a Layer) -> Result<Box<dyn Iterator<Item = f64> + 'a>, PlotError> {
        if let Some(x) = &self.x_intercept {
            match x {
                PrimitiveValue::Int(i) => {
                    return Ok(Box::new(std::iter::once(*i as f64)));
                }
                PrimitiveValue::Float(f) => {
                    return Ok(Box::new(std::iter::once(*f)));
                }
                _ => panic!("XIntercept constant must be Int or Float"),
            }

        } else {
            let iter = layer.aesthetic_value_iter(&Aesthetic::XIntercept).ok_or(
                PlotError::MissingAesthetic {
                    aesthetic: Aesthetic::XIntercept,
                },
            )?;
            Ok(Box::new(make_float_iter(iter)))
        }
    }
}

impl Geom for GeomVLine {
    fn train_scales(&self, scales: &mut crate::scale::ScaleSet) {
        if let Some(value) = &self.x_intercept {
            scales.x_continuous.train_one(value);
        }
    }

    fn apply_scales(&mut self, scales: &crate::scale::ScaleSet) {
        if let Some(value) = &self.x_intercept {
            let mapped_value = scales.x_continuous.map_primitive_value(value);
            if let Some(mapped) = mapped_value {
                self.x_intercept = Some(PrimitiveValue::Float(mapped));
            }
        }
    }

    fn render(&self, ctx: &mut RenderContext, _properties: HashMap<AestheticProperty, PropertyVector>) -> Result<(), PlotError> {


        Ok(())
    }
}
