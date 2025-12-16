use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic, AestheticDomain};
use crate::data::PrimitiveValue;
use crate::error::PlotError;
use crate::layer::Layer;
use crate::theme::Color;
use crate::utils::Either;
use crate::utils::data::{make_color_iter, make_float_iter};

/// GeomVLine renders vertical reference lines at specified x-intercepts
///
/// The x-intercept is specified via the XIntercept aesthetic mapping.
pub struct GeomVLine {
    pub x_intercept: Option<PrimitiveValue>,

    /// Default line color
    pub color: Either<Color, AestheticDomain>,

    /// Default line width
    pub size: Either<f64, AestheticDomain>,

    /// Default alpha/opacity
    pub alpha: Either<f64, AestheticDomain>,

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
            color: None,
            size: None,
            alpha: None,
            linetype: None,
        }
    }

    /// Set the line color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        let rgba = color.into();
        self.color = Some(AesValue::constant(PrimitiveValue::Int(rgba)));
        self
    }

    /// Set the line width
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size = Some(AesValue::constant(PrimitiveValue::Float(size)));
        self
    }

    /// Set the alpha/opacity
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = Some(AesValue::constant(PrimitiveValue::Float(
            alpha.clamp(0.0, 1.0),
        )));
        self
    }

    /// Set the line style pattern
    pub fn linetype(&mut self, pattern: impl Into<String>) -> &mut Self {
        self.linetype = Some(AesValue::constant(PrimitiveValue::Str(pattern.into())));
        self
    }

    fn get_x_intercept(&self, layer: &Layer) -> Result<impl Iterator<Item = f64>, PlotError> {
        if let Some(x) = &self.x_intercept {
            x.as_f64()
                .map(std::iter::once)
                .ok_or(PlotError::AestheticMappingError(
                    Aesthetic::XIntercept,
                    "xintercept must be a numeric value".to_string(),
                ))
        } else {
            let iter = layer.aesthetic_value_iter(Aesthetic::XIntercept).ok_or(
                PlotError::MissingAesthetic {
                    aesthetic: Aesthetic::XIntercept,
                },
            )?;
            make_float_iter(iter)
        }
    }

    fn get_color(&self, layer: &Layer) -> Result<impl Iterator<Item = Color>, PlotError> {
        match &self.color {
            Either::Left(color) => Ok(std::iter::repeat(color.clone())),
            Either::Right(domain) => {
                let iter = layer.aesthetic_value_iter(Aesthetic::Color).ok_or(
                    PlotError::MissingAesthetic {
                        aesthetic: Aesthetic::Color,
                    },
                )?;
                make_color_iter(iter)
            }
        }
    }

    fn get_size(&self, layer: &Layer) -> Result<impl Iterator<Item = f64>, PlotError> {
        match &self.size {
            Either::Left(size) => Ok(std::iter::repeat(*size)),
            Either::Right(domain) => {
                let iter = layer.aesthetic_value_iter(Aesthetic::Size).ok_or(
                    PlotError::MissingAesthetic {
                        aesthetic: Aesthetic::Size,
                    },
                )?;
                make_float_iter(iter)
            }
        }
    }

    fn get_alpha(&self, layer: &Layer) -> Result<impl Iterator<Item = f64>, PlotError> {
        match &self.alpha {
            Either::Left(alpha) => Ok(std::iter::repeat(*alpha)),
            Either::Right(domain) => {
                let iter = layer.aesthetic_value_iter(Aesthetic::Alpha).ok_or(
                    PlotError::MissingAesthetic {
                        aesthetic: Aesthetic::Alpha,
                    },
                )?;
                make_float_iter(iter)
            }
        }
    }
}

impl IntoLayer for GeomVLine {
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

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        use crate::visuals::LineStyle;
        
        let x_intercepts = self.get_x_intercept(&ctx.layer)?;
        let colors = self.get_color(&ctx.layer)?;
        let alphas = self.get_alpha(&ctx.layer)?;
        let sizes = self.get_size(&ctx.layer)?;
        
        // Get linetype if specified
        let linetype_pattern = if let Some(AesValue::Constant {
            value: PrimitiveValue::Str(pattern),
            ..
        }) = ctx.mapping().get(&Aesthetic::Linetype)
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
        let (y0, y1) = ctx.y_range;

        for (((x_intercept, color), alpha), size) in x_intercepts.zip(colors).zip(alphas).zip(sizes) {
            let x_visual = ctx.map_x(x_intercept);

            // Set drawing properties for this line
            ctx.set_color_alpha(&color, alpha);
            ctx.cairo.set_line_width(size);

            // Draw line from left to right edge of plot area
            ctx.cairo.move_to(x_visual, y0);
            ctx.cairo.line_to(x_visual, y1);
            ctx.cairo.stroke().ok();
        }

        Ok(())
    }
}
