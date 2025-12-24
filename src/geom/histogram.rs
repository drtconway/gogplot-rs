use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::{AesValue, AestheticProperty};
use crate::data::PrimitiveValue;
use crate::error::PlotError;
use crate::geom::properties::PropertyVector;

/// GeomHistogram renders a histogram by binning continuous data
/// By default, it uses Stat::Bin to divide the data into bins
pub struct GeomHistogram {
    /// Default fill color (if not mapped)
    pub fill: Option<AesValue>,

    /// Default stroke color (if not mapped)
    pub color: Option<AesValue>,

    /// Default alpha/opacity (if not mapped)
    pub alpha: Option<AesValue>,
}

impl GeomHistogram {
    /// Create a new histogram geom with default settings
    pub fn new() -> Self {
        Self {
            fill: None,
            color: None,
            alpha: None,
        }
    }

    /// Set the default fill color
    pub fn fill(&mut self, color: crate::theme::Color) -> &mut Self {
        self.fill = Some(AesValue::constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the default stroke color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        self.color = Some(AesValue::constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the default alpha/opacity
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = Some(AesValue::constant(PrimitiveValue::Float(
            alpha.clamp(0.0, 1.0),
        )));
        self
    }
}

impl Default for GeomHistogram {
    fn default() -> Self {
        Self::new()
    }
}

impl Geom for GeomHistogram {
    fn train_scales(&self, _scales: &mut crate::scale::ScaleSet) {
        
    }

    fn apply_scales(&mut self, _scales: &crate::scale::ScaleSet) {
        
    }

    fn render(&self, ctx: &mut RenderContext, _properties: HashMap<AestheticProperty, PropertyVector>) -> Result<(), PlotError> {

        Ok(())
    }
}
