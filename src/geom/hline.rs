use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::PlotError;
use crate::geom::properties::{ColorProperty, FloatProperty};
use crate::layer::Layer;
use crate::scale::traits::ScaleBase;
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
            color: None,
            size: None,
            alpha: None,
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
        self.linetype = Some(AesValue::constant(PrimitiveValue::Str(pattern.into())));
        self
    }

    fn get_y_intercept(&self, layer: &Layer) -> Result<impl Iterator<Item = f64>, PlotError> {
        if let Some(y) = &self.y_intercept {
            y.as_f64()
                .map(std::iter::once)
                .ok_or(PlotError::AestheticMappingError(
                    Aesthetic::YIntercept,
                    "yintercept must be a numeric value".to_string(),
                ))
        } else {
            let iter = layer.aesthetic_value_iter(Aesthetic::YIntercept).ok_or(
                PlotError::MissingAesthetic {
                    aesthetic: Aesthetic::YIntercept,
                },
            )?;
            make_float_iter(iter)
        }
    }
}

impl IntoLayer for GeomHLine {
    fn default_aesthetics(&self) -> Vec<(Aesthetic, AesValue)> {
        let mut defaults = Vec::new();

        if let Some(color) = &self.color {
            defaults.push((Aesthetic::Color, color.clone()));
        }
        if let Some(alpha) = &self.alpha {
            defaults.push((Aesthetic::Alpha, alpha.clone()));
        }
        if let Some(size) = &self.size {
            defaults.push((Aesthetic::Size, size.clone()));
        }
        if let Some(linetype) = &self.linetype {
            defaults.push((Aesthetic::Linetype, linetype.clone()));
        }

        defaults
    }

    fn into_layer(self) -> crate::layer::Layer
    where
        Self: Geom + 'static,
    {
        let mut mapping = crate::aesthetics::AesMap::new();

        // Set default aesthetics from geom settings if provided
        for (aesthetic, value) in self.default_aesthetics() {
            mapping.set(aesthetic, value);
        }

        // Get stat and position before consuming self
        let stat = self.stat.clone();
        let position = self.position.clone();

        crate::layer::Layer {
            geom: Box::new(self),
            data: None,
            mapping: Some(mapping),
            stat,
            position,
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
        let y_intercepts = self.get_y_intercept(&ctx.layer)?;
        let colors = self.color.iter(&ctx.layer.data, ctx.layer.mapping)?;
        let alphas = self.alpha.iter(&ctx.layer.data, ctx.layer.mapping)?;
        let sizes = self.size.iter(&ctx.layer.data, ctx.layer.mapping)?;

        // Get linetype if specified
        let linetype_pattern = if let Some(AesValue::Constant {
            value: PrimitiveValue::Str(pattern),
            ..
        }) = ctx.layer.mapping.get(&Aesthetic::Linetype)
        {
            Some(pattern.clone())
        } else {
            None
        };

        // Apply line style once
        if let Some(pattern) = &linetype_pattern {
            let style = LineStyle::from(pattern.as_str());
            style.apply(ctx.cairo);
        } else {
            LineStyle::default().apply(ctx.cairo);
        }

        // Draw horizontal line(s) across the full width of the plot
        let (x0, x1) = ctx.x_range;

        for (((y_intercept, color), alpha), size) in y_intercepts.zip(colors).zip(alphas).zip(sizes)
        {
            let y_visual = ctx.map_y(y_intercept);

            // Set drawing properties for this line
            ctx.set_color_alpha(&color, alpha);
            ctx.cairo.set_line_width(size);

            // Draw line from left to right edge of plot area
            ctx.cairo.move_to(x0, y_visual);
            ctx.cairo.line_to(x1, y_visual);
            ctx.cairo.stroke().ok();
        }

        Ok(())
    }
}
