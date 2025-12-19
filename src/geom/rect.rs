use super::{Geom, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::PlotError;

/// GeomRect renders rectangles defined by xmin, xmax, ymin, ymax
pub struct GeomRect {
    /// Default fill color (if not mapped)
    pub fill: Option<AesValue>,

    /// Default stroke color (if not mapped)
    pub color: Option<AesValue>,

    /// Default alpha/opacity (if not mapped)
    pub alpha: Option<AesValue>,
}

impl GeomRect {
    /// Create a new rect geom with default settings
    pub fn new() -> Self {
        Self {
            fill: None,
            color: None,
            alpha: None,
        }
    }

    /// Set the default fill color
    pub fn fill(&mut self, color: crate::theme::Color) -> &mut Self {
        let rgba = color.into();
        self.fill = Some(AesValue::constant(PrimitiveValue::Int(rgba)));
        self
    }

    /// Set the default stroke color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        let rgba = color.into();
        self.color = Some(AesValue::constant(PrimitiveValue::Int(rgba)));
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

impl Default for GeomRect {
    fn default() -> Self {
        Self::new()
    }
}

impl Geom for GeomRect {
    fn train_scales(&self, _scales: &mut crate::scale::ScaleSet) {

    }

    fn apply_scales(&mut self, _scales: &crate::scale::ScaleSet) {
        
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        Ok(())
    }
}
