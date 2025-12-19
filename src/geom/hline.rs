use core::panic;

use super::{Geom, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::PlotError;
use crate::geom::properties::{ColorProperty, FloatProperty};
use crate::layer::Layer;
use crate::scale::traits::{ContinuousRangeScale, ScaleBase};
use crate::utils::data::make_float_iter;
use crate::visuals::LineStyle;

/// GeomHLine renders horizontal reference lines at specified y-intercepts
///
/// The y-intercept is specified via the YIntercept aesthetic mapping.
pub struct GeomHLine {
    pub y_intercept: Option<PrimitiveValue>,

    /// Default line color
    pub color: ColorProperty,

    /// Default line width
    pub size: FloatProperty,

    /// Default alpha/opacity
    pub alpha: FloatProperty,

    /// Default line style pattern
    pub linetype: Option<LineStyle>,
}

impl GeomHLine {
    /// Create a new horizontal line geom
    ///
    /// Y-intercept should be specified via aesthetic mapping:
    /// - Constant: `.aes(|a| a.yintercept_const(value))`
    /// - Column: `.aes(|a| a.yintercept("column_name"))`
    pub fn new() -> Self {
        Self {
            y_intercept: None,
            color: ColorProperty::new(),
            size: FloatProperty::new(),
            alpha: FloatProperty::new(),
            linetype: None,
        }
    }

    pub fn y_intercept(&mut self, value: impl Into<PrimitiveValue>) -> &mut Self {
        self.y_intercept = Some(value.into());
        self
    }

    /// Set the line color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        self.color.color(color);
        self
    }

    /// Set the line width
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size.value(size);
        self
    }

    /// Set the alpha/opacity
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha.value(alpha);
        self
    }

    /// Set the line style pattern
    pub fn linetype(&mut self, pattern: impl Into<String>) -> &mut Self {
        self.linetype = Some(LineStyle::from(pattern.into().as_str()));
        self
    }

    fn get_y_intercept<'a>(&'a self, layer: &'a Layer) -> Result<Box<dyn Iterator<Item = f64> + 'a>, PlotError> {
        if let Some(y) = &self.y_intercept {
            match y {
                PrimitiveValue::Int(x) => {
                     Ok(Box::new(std::iter::once(*x as f64)))
                }
                PrimitiveValue::Float(x) => {
                     Ok(Box::new(std::iter::once(*x)))
                }
                _ => panic!("yintercept must be a numeric value"),
            }
        } else {
            let iter = layer.aesthetic_value_iter(&Aesthetic::YIntercept).ok_or(
                PlotError::MissingAesthetic {
                    aesthetic: Aesthetic::YIntercept,
                },
            )?;
            Ok(Box::new(make_float_iter(iter)))
        }
    }
}

impl Geom for GeomHLine {
    fn train_scales(&self, scales: &mut crate::scale::ScaleSet) {
        if let Some(value) = &self.y_intercept {
            scales.y_continuous.train_one(value);
        }
    }

    fn apply_scales(&mut self, scales: &crate::scale::ScaleSet) {
        if let Some(value) = &self.y_intercept {
            let mapped_value = scales.y_continuous.map_primitive_value(value);
            if let Some(mapped) = mapped_value {
                self.y_intercept = Some(PrimitiveValue::Float(mapped));
            }
        }
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        let data = ctx.layer.data(ctx.data());
        let mapping = ctx.layer.mapping(ctx.mapping());
        let y_intercepts = self.get_y_intercept(&ctx.layer)?;
        let colors = self.color.iter(data, mapping)?;
        let alphas = self.alpha.iter(data, mapping)?;
        let sizes = self.size.iter(data, mapping)?;

        // Get linetype if specified
        let linetype_pattern = if let Some(AesValue::Constant {
            value: PrimitiveValue::Str(pattern),
            ..
        }) = mapping.get(&Aesthetic::Linetype)
        {
            Some(pattern.clone())
        } else {
            None
        };

        // Draw horizontal line(s) across the full width of the plot
        let (x0, x1) = ctx.x_range;


        Ok(())
    }
}
