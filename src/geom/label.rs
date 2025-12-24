use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic, AestheticProperty};
use crate::data::PrimitiveValue;
use crate::error::PlotError;
use crate::geom::properties::PropertyVector;

/// GeomLabel renders text labels with a background box at specified positions
pub struct GeomLabel {
    /// Default text color (if not mapped)
    pub color: Option<AesValue>,

    /// Default text size (if not mapped)
    pub size: Option<AesValue>,

    /// Default alpha/opacity for text (if not mapped)
    pub alpha: Option<AesValue>,

    /// Default fill color for label background (if not mapped)
    pub fill: Option<AesValue>,

    /// Horizontal adjustment: 0 = left, 0.5 = center, 1 = right
    pub hjust: f64,

    /// Vertical adjustment: 0 = bottom, 0.5 = middle, 1 = top
    pub vjust: f64,

    /// Angle of text rotation in degrees (0 = horizontal, 90 = vertical)
    pub angle: f64,

    /// Padding around text in the label box (in points)
    pub padding: f64,

    /// Corner radius for rounded label boxes (0 = sharp corners)
    pub radius: f64,
}

impl GeomLabel {
    /// Create a new label geom with default settings from theme
    pub fn new() -> Self {
        use crate::theme::Theme;
        let theme = Theme::default();
        
        Self {
            color: None,
            size: None,
            alpha: None,
            fill: None,
            hjust: theme.geom_text.hjust,
            vjust: theme.geom_text.vjust,
            angle: 0.0,
            padding: 2.0,
            radius: 2.0,
        }
    }

    /// Set the default text color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        self.color = Some(AesValue::constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the default text size
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size = Some(AesValue::constant(PrimitiveValue::Float(size)));
        self
    }

    /// Set the default alpha/opacity for text
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = Some(AesValue::constant(PrimitiveValue::Float(
            alpha.clamp(0.0, 1.0),
        )));
        self
    }

    /// Set the default fill color for label background
    pub fn fill(&mut self, color: crate::theme::Color) -> &mut Self {
        self.fill = Some(AesValue::constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set horizontal justification (0 = left, 0.5 = center, 1 = right)
    pub fn hjust(&mut self, hjust: f64) -> &mut Self {
        self.hjust = hjust.clamp(0.0, 1.0);
        self
    }

    /// Set vertical justification (0 = bottom, 0.5 = middle, 1 = top)
    pub fn vjust(&mut self, vjust: f64) -> &mut Self {
        self.vjust = vjust.clamp(0.0, 1.0);
        self
    }

    /// Set text rotation angle in degrees
    pub fn angle(&mut self, angle: f64) -> &mut Self {
        self.angle = angle;
        self
    }

    /// Set padding around text in the label box
    pub fn padding(&mut self, padding: f64) -> &mut Self {
        self.padding = padding.max(0.0);
        self
    }

    /// Set corner radius for rounded label boxes
    pub fn radius(&mut self, radius: f64) -> &mut Self {
        self.radius = radius.max(0.0);
        self
    }
}

impl Default for GeomLabel {
    fn default() -> Self {
        Self::new()
    }
}

impl Geom for GeomLabel {
    fn train_scales(&self, scales: &mut crate::scale::ScaleSet) {
        
    }

    fn apply_scales(&mut self, scales: &crate::scale::ScaleSet) {
        
    }

    fn render(&self, ctx: &mut RenderContext, _properties: HashMap<AestheticProperty, PropertyVector>) -> Result<(), PlotError> {
        Ok(())
    }
}
