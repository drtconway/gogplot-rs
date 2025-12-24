use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic, AestheticProperty};
use crate::data::PrimitiveValue;
use crate::error::Result;
use crate::geom::properties::PropertyVector;

/// GeomSmooth renders fitted curves with confidence intervals
///
/// This geom combines a line (the fitted curve) and a ribbon (the confidence interval).
/// It's typically used with Stat::Smooth which computes the fitted values.
///
/// # Required Aesthetics
///
/// When using Stat::Smooth (default):
/// - X: Predictor variable
/// - Y: Response variable
///
/// The stat computes: x, y (fitted), ymin (lower CI), ymax (upper CI)
///
/// When using Stat::Identity, these are required:
/// - X: x values
/// - Y: fitted y values
/// - Ymin: lower confidence bound
/// - Ymax: upper confidence bound
///
/// # Optional Aesthetics
///
/// - Color: Line color (default: blue)
/// - Fill: Ribbon fill color (default: gray with alpha)
/// - Alpha: Transparency for ribbon (default: 0.4)
/// - Size: Line width (default: 1.0)
///
/// # Examples
///
/// ```rust,ignore
/// // Basic smooth with linear regression
/// Plot::new(data)
///     .aes(|a| { a.x("x"); a.y("y"); })
///     .geom_point()
///     .geom_smooth()
///     .save("smooth.png", 800, 600)?;
///
/// // Smooth per group with different colors
/// Plot::new(data)
///     .aes(|a| { 
///         a.x("x"); 
///         a.y("y"); 
///         a.color("group");
///     })
///     .geom_point()
///     .geom_smooth()
///     .save("smooth_grouped.png", 800, 600)?;
///
/// // Smooth without confidence interval
/// Plot::new(data)
///     .aes(|a| { a.x("x"); a.y("y"); })
///     .geom_smooth_with(|layer| {
///         layer.geom.se(false);
///     })
///     .save("smooth_no_se.png", 800, 600)?;
/// ```
pub struct GeomSmooth {
    /// Default line color (if not mapped)
    pub color: Option<AesValue>,

    /// Default fill color for confidence ribbon (if not mapped)
    pub fill: Option<AesValue>,

    /// Default alpha/opacity for ribbon
    pub alpha: Option<AesValue>,

    /// Default line width
    pub size: Option<AesValue>,

    /// Whether to show confidence interval
    pub se: bool,
}

impl GeomSmooth {
    /// Create a new smooth geom with default settings
    pub fn new() -> Self {
        Self {
            color: Some(AesValue::constant(PrimitiveValue::Int(
                crate::theme::Color::rgb(0, 114, 178).into(), // Blue
            ))),
            fill: Some(AesValue::constant(PrimitiveValue::Int(
                crate::theme::Color::rgb(128, 128, 128).into(), // Gray
            ))),
            alpha: Some(AesValue::constant(PrimitiveValue::Float(0.4))),
            size: Some(AesValue::constant(PrimitiveValue::Float(1.0))),
            se: true,
        }
    }

    /// Set the line color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        self.color = Some(AesValue::constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the ribbon fill color
    pub fn fill(&mut self, color: crate::theme::Color) -> &mut Self {
        self.fill = Some(AesValue::constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the ribbon alpha/opacity
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = Some(AesValue::constant(PrimitiveValue::Float(
            alpha.clamp(0.0, 1.0),
        )));
        self
    }

    /// Set the line width
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size = Some(AesValue::constant(PrimitiveValue::Float(size.max(0.0))));
        self
    }

    /// Set whether to show confidence interval
    pub fn se(&mut self, se: bool) -> &mut Self {
        self.se = se;
        self
    }

}

impl Default for GeomSmooth {
    fn default() -> Self {
        Self::new()
    }
}

impl Geom for GeomSmooth {
    fn train_scales(&self, scales: &mut crate::scale::ScaleSet) {
        
    }

    fn apply_scales(&mut self, scales: &crate::scale::ScaleSet) {
        
    }

    fn render(&self, ctx: &mut RenderContext, _properties: HashMap<AestheticProperty, PropertyVector>) -> Result<()> {
        Ok(())
    }
}

impl Clone for GeomSmooth {
    fn clone(&self) -> Self {
        Self {
            color: self.color.clone(),
            fill: self.fill.clone(),
            alpha: self.alpha.clone(),
            size: self.size.clone(),
            se: self.se,
        }
    }
}
