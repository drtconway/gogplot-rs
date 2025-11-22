use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::{DataType, PlotError};
use crate::layer::{Position, Stat};

/// GeomHLine renders horizontal reference lines at specified y-intercepts
/// 
/// The y-intercept is specified via the YIntercept aesthetic mapping.
pub struct GeomHLine {
    /// Default line color
    pub color: Option<AesValue>,

    /// Default line width
    pub size: Option<AesValue>,

    /// Default alpha/opacity
    pub alpha: Option<AesValue>,

    /// Default line style pattern
    pub linetype: Option<AesValue>,

    /// The stat to use (default is Identity)
    pub stat: Stat,

    /// The position adjustment (default is Identity)
    pub position: Position,
}

impl GeomHLine {
    /// Create a new horizontal line geom
    /// 
    /// Y-intercept should be specified via aesthetic mapping:
    /// - Constant: `.aes(|a| a.yintercept_const(value))`
    /// - Column: `.aes(|a| a.yintercept("column_name"))`
    pub fn new() -> Self {
        Self {
            color: None,
            size: None,
            alpha: None,
            linetype: None,
            stat: Stat::Identity,
            position: Position::Identity,
        }
    }

    /// Set the line color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        let rgba = color.into();
        self.color = Some(AesValue::Constant(PrimitiveValue::Int(rgba)));
        self
    }

    /// Set the line width
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size = Some(AesValue::Constant(PrimitiveValue::Float(size)));
        self
    }

    /// Set the alpha/opacity
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = Some(AesValue::Constant(PrimitiveValue::Float(
            alpha.clamp(0.0, 1.0),
        )));
        self
    }

    /// Set the line style pattern
    pub fn linetype(&mut self, pattern: impl Into<String>) -> &mut Self {
        self.linetype = Some(AesValue::Constant(PrimitiveValue::Str(pattern.into())));
        self
    }

    /// Set the stat to use (default is Identity)
    pub fn stat(&mut self, stat: Stat) -> &mut Self {
        self.stat = stat;
        self
    }

    /// Set the position adjustment (default is Identity)
    pub fn position(&mut self, position: Position) -> &mut Self {
        self.position = position;
        self
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
            mapping,
            stat,
            position,
            computed_data: None,
            computed_mapping: None,
            computed_scales: None,
        }
    }
}

impl Geom for GeomHLine {
    fn required_aesthetics(&self) -> &[Aesthetic] {
        &[Aesthetic::YIntercept]
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        // Get the y-intercept value(s) from YIntercept aesthetic
        let y_aes = ctx.mapping.get(&Aesthetic::YIntercept)
            .ok_or_else(|| PlotError::MissingAesthetic { aesthetic: Aesthetic::YIntercept })?;

        let y_values = match y_aes {
            AesValue::Constant(PrimitiveValue::Float(y)) => vec![*y],
            AesValue::Constant(PrimitiveValue::Int(i)) => vec![*i as f64],
            AesValue::Column(col) => {
                let vec = ctx
                    .data
                    .get(col.as_str())
                    .ok_or_else(|| PlotError::missing_column(col))?;
                if let Some(floats) = vec.iter_float() {
                    floats.collect()
                } else if let Some(ints) = vec.iter_int() {
                    ints.map(|i| i as f64).collect()
                } else {
                    return Err(PlotError::invalid_column_type(col, "numeric"));
                }
            }
            _ => {
                return Err(PlotError::InvalidAestheticType {
                    aesthetic: Aesthetic::YIntercept,
                    expected: DataType::Numeric,
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

        // Draw horizontal line(s) across the full width of the plot
        for y_data in y_values {
            // Map y value to visual coordinates
            if let Some(y_normalized) = ctx.scales.y.as_deref().and_then(|s| s.map_value(y_data)) {
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
