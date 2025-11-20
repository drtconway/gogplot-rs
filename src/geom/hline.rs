use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::PlotError;

/// GeomHLine renders horizontal reference lines at specified y-intercepts
pub struct GeomHLine {
    /// Y-intercept value(s) for the horizontal line(s)
    pub yintercept: AesValue,

    /// Default line color
    pub color: Option<AesValue>,

    /// Default line width
    pub size: Option<AesValue>,

    /// Default alpha/opacity
    pub alpha: Option<AesValue>,

    /// Default line style pattern
    pub linetype: Option<AesValue>,
}

impl GeomHLine {
    /// Create a new horizontal line geom at the specified y-intercept
    pub fn new(yintercept: f64) -> Self {
        Self {
            yintercept: AesValue::Constant(PrimitiveValue::Float(yintercept)),
            color: None,
            size: None,
            alpha: None,
            linetype: None,
        }
    }

    /// Set the line color
    pub fn color(mut self, color: crate::theme::Color) -> Self {
        let rgba = color.into();
        self.color = Some(AesValue::Constant(PrimitiveValue::Int(rgba)));
        self
    }

    /// Set the line width
    pub fn size(mut self, size: f64) -> Self {
        self.size = Some(AesValue::Constant(PrimitiveValue::Float(size)));
        self
    }

    /// Set the alpha/opacity
    pub fn alpha(mut self, alpha: f64) -> Self {
        self.alpha = Some(AesValue::Constant(PrimitiveValue::Float(
            alpha.clamp(0.0, 1.0),
        )));
        self
    }

    /// Set the line style pattern
    pub fn linetype(mut self, pattern: impl Into<String>) -> Self {
        self.linetype = Some(AesValue::Constant(PrimitiveValue::Str(pattern.into())));
        self
    }
}

impl IntoLayer for GeomHLine {
    fn default_aesthetics(&self) -> Vec<(Aesthetic, AesValue)> {
        let mut defaults = vec![(Aesthetic::YBegin, self.yintercept.clone())];

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
}

impl Geom for GeomHLine {
    fn required_aesthetics(&self) -> &[Aesthetic] {
        // No required aesthetics - yintercept is provided in constructor
        &[]
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        // Get the y-intercept value(s)
        let y_values = match &self.yintercept {
            AesValue::Constant(PrimitiveValue::Float(y)) => vec![*y],
            AesValue::Column(col) => {
                let vec = ctx
                    .data
                    .get(col.as_str())
                    .ok_or_else(|| PlotError::MissingAesthetic(format!("column '{}'", col)))?;
                if let Some(floats) = vec.as_float() {
                    floats.iter().copied().collect()
                } else if let Some(ints) = vec.as_int() {
                    ints.iter().map(|&i| i as f64).collect()
                } else {
                    return Err(PlotError::InvalidAestheticType(
                        "yintercept must be numeric".to_string(),
                    ));
                }
            }
            _ => {
                return Err(PlotError::InvalidAestheticType(
                    "yintercept must be numeric".to_string(),
                ));
            }
        };

        // Get visual properties (use first value if multiple)
        let colors = ctx.get_color_values()?;
        let alphas = ctx.get_aesthetic_values(Aesthetic::Alpha, None)?;
        let sizes = ctx.get_aesthetic_values(Aesthetic::Size, None)?;

        let colors_vec: Vec<_> = colors.collect();
        let alphas_vec: Vec<_> = alphas.collect();
        let sizes_vec: Vec<_> = sizes.collect();

        let color = &colors_vec[0];
        let alpha = alphas_vec[0];
        let size = sizes_vec[0];

        // Get linetype if specified
        let linetype_pattern = if let Some(AesValue::Constant(PrimitiveValue::Str(pattern))) =
            ctx.mapping.get(&Aesthetic::Linetype)
        {
            Some(pattern.clone())
        } else {
            None
        };

        // Set drawing properties
        ctx.set_color_alpha(color, alpha);
        ctx.cairo.set_line_width(size);

        // Apply line style
        use crate::visuals::LineStyle;
        if let Some(pattern) = linetype_pattern {
            let style = LineStyle::from(pattern.as_str());
            style.apply(&mut ctx.cairo);
        } else {
            LineStyle::default().apply(&mut ctx.cairo);
        }

        // Draw horizontal line(s) across the full width of the plot
        for y_data in y_values {
            // Map y value to visual coordinates
            if let Some(y_normalized) = ctx.scales.y.as_ref().and_then(|s| s.map_value(y_data)) {
                let y_visual = ctx.map_y(y_normalized);

                // Draw line from left to right edge of plot area
                let (x0, x1) = ctx.x_range;
                ctx.cairo.move_to(x0, y_visual);
                ctx.cairo.line_to(x1, y_visual);
                ctx.cairo.stroke().ok();
            }
        }

        Ok(())
    }
}
