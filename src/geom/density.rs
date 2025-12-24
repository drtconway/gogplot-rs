use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic, AestheticProperty};
use crate::data::PrimitiveValue;
use crate::error::{PlotError, to_plot_error};
use crate::geom::properties::PropertyVector;

/// GeomDensity renders kernel density estimates
///
/// This geom automatically computes the density using the specified stat parameters
/// and renders it as a line plot.
pub struct GeomDensity {
    /// Default line color (if not mapped)
    pub color: Option<AesValue>,

    /// Default line width (if not mapped)
    pub size: Option<AesValue>,

    /// Default alpha/opacity (if not mapped)
    pub alpha: Option<AesValue>,

    /// Bandwidth adjustment multiplier (default 1.0)
    pub adjust: f64,

    /// Number of evaluation points (default 512)
    pub n: usize,
}

impl GeomDensity {
    /// Create a new density geom with default settings
    pub fn new() -> Self {
        Self {
            color: None,
            size: None,
            alpha: None,
            adjust: 1.0,
            n: 512,
        }
    }

    /// Set the default line color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        let rgba = color.into();
        self.color = Some(AesValue::constant(PrimitiveValue::Int(rgba)));
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

    /// Set bandwidth adjustment multiplier
    pub fn adjust(&mut self, adjust: f64) -> &mut Self {
        self.adjust = adjust;
        self
    }

    /// Set number of evaluation points
    pub fn n(&mut self, n: usize) -> &mut Self {
        self.n = n;
        self
    }
}

impl Default for GeomDensity {
    fn default() -> Self {
        Self::new()
    }
}

impl Geom for GeomDensity {
    fn train_scales(&self, _scales: &mut crate::scale::ScaleSet) {
        
    }

    fn apply_scales(&mut self, scales: &crate::scale::ScaleSet) {
        
    }

    fn render(&self, ctx: &mut RenderContext, _properties: HashMap<AestheticProperty, PropertyVector>) -> Result<(), PlotError> {

        Ok(())
    }
}
