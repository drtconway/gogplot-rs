use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::{DataType, PlotError};

/// GeomVLine renders vertical reference lines at specified x-intercepts
pub struct GeomVLine {
    /// X-intercept value(s) for the vertical line(s)
    pub xintercept: AesValue,

    /// Default line color
    pub color: Option<AesValue>,

    /// Default line width
    pub size: Option<AesValue>,

    /// Default alpha/opacity
    pub alpha: Option<AesValue>,

    /// Default line style pattern
    pub linetype: Option<AesValue>,
}

impl GeomVLine {
    /// Create a new vertical line geom at the specified x-intercept
    pub fn new(xintercept: f64) -> Self {
        Self {
            xintercept: AesValue::Constant(PrimitiveValue::Float(xintercept)),
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

impl IntoLayer for GeomVLine {
    fn default_aesthetics(&self) -> Vec<(Aesthetic, AesValue)> {
        let mut defaults = vec![(Aesthetic::XBegin, self.xintercept.clone())];

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

impl Geom for GeomVLine {
    fn required_aesthetics(&self) -> &[Aesthetic] {
        // No required aesthetics - xintercept is provided in constructor
        &[]
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        // Get the x-intercept value(s)
        let x_values = match &self.xintercept {
            AesValue::Constant(PrimitiveValue::Float(x)) => vec![*x],
            AesValue::Column(col) => {
                let vec = ctx
                    .data
                    .get(col.as_str())
                    .ok_or_else(|| PlotError::missing_column(col))?;
                if let Some(floats) = vec.as_float() {
                    floats.iter().copied().collect()
                } else if let Some(ints) = vec.as_int() {
                    ints.iter().map(|&i| i as f64).collect()
                } else {
                    return Err(PlotError::invalid_column_type(col, "numeric"));
                }
            }
            _ => {
                return Err(PlotError::InvalidAestheticType {
                    aesthetic: Aesthetic::X,
                    expected: DataType::Custom("numeric constant or column".to_string()),
                    actual: DataType::Custom("invalid value".to_string()),
                });
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
            style.apply(ctx.cairo);
        } else {
            LineStyle::default().apply(ctx.cairo);
        }

        // Draw vertical line(s) across the full height of the plot
        for x_data in x_values {
            // Map x value to visual coordinates
            if let Some(x_normalized) = ctx.scales.x.as_deref().and_then(|s| s.map_value(x_data)) {
                let x_visual = ctx.map_x(x_normalized);

                // Draw line from bottom to top edge of plot area
                let (y0, y1) = ctx.y_range;
                ctx.cairo.move_to(x_visual, y0);
                ctx.cairo.line_to(x_visual, y1);
                ctx.cairo.stroke().ok();
            }
        }

        Ok(())
    }
}
