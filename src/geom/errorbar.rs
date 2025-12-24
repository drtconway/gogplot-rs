use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic, AestheticProperty};
use crate::data::PrimitiveValue;
use crate::error::PlotError;
use crate::geom::properties::PropertyVector;

/// GeomErrorbar renders vertical error bars with optional caps
pub struct GeomErrorbar {
    /// Default line color (if not mapped)
    pub color: Option<AesValue>,

    /// Default line width (if not mapped)
    pub size: Option<AesValue>,

    /// Default alpha/opacity (if not mapped)
    pub alpha: Option<AesValue>,

    /// Width of the caps at the ends of the error bars (in data coordinates)
    pub width: f64,
}

impl GeomErrorbar {
    /// Create a new errorbar geom with default settings
    pub fn new() -> Self {
        Self {
            color: None,
            size: None,
            alpha: None,
            width: 0.5,
        }
    }

    /// Set the default line color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        self.color = Some(AesValue::constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the default line width
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size = Some(AesValue::constant(PrimitiveValue::Float(size)));
        self
    }

    /// Set the default alpha/opacity
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = Some(AesValue::constant(PrimitiveValue::Float(
            alpha.clamp(0.0, 1.0),
        )));
        self
    }

    /// Set the width of the caps (in data coordinates)
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width.max(0.0);
        self
    }
}

impl Default for GeomErrorbar {
    fn default() -> Self {
        Self::new()
    }
}

impl Geom for GeomErrorbar {
    fn train_scales(&self, _scales: &mut crate::scale::ScaleSet) {
        
    }

    fn apply_scales(&mut self, scales: &crate::scale::ScaleSet) {
        
    }

    fn render(&self, ctx: &mut RenderContext, _properties: HashMap<AestheticProperty, PropertyVector>) -> Result<(), PlotError> {

        Ok(())
    }
}
