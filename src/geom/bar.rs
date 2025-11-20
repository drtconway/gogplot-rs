use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{Aesthetic, AesValue};
use crate::data::PrimitiveValue;
use crate::error::PlotError;
use crate::layer::{Position, Stat};

/// GeomBar renders bars from y=0 to y=value
/// By default, it uses Stat::Count to count occurrences at each x position
pub struct GeomBar {
    /// Default fill color (if not mapped)
    pub fill: Option<AesValue>,

    /// Default stroke color (if not mapped)
    pub color: Option<AesValue>,

    /// Default alpha/opacity (if not mapped)
    pub alpha: Option<AesValue>,

    /// Bar width (as a proportion of the spacing between x values)
    pub width: f64,

    /// The stat to use (default is Count)
    pub stat: Stat,

    /// The position adjustment (default is Identity, but Stack is common)
    pub position: Position,
}

impl GeomBar {
    /// Create a new bar geom with default settings
    pub fn new() -> Self {
        Self {
            fill: None,
            color: None,
            alpha: None,
            width: 0.9,
            stat: Stat::Count,
            position: Position::Identity,
        }
    }

    /// Set the default fill color
    pub fn fill(mut self, color: crate::theme::Color) -> Self {
        let rgba = ((color.0 as i64) << 24)
            | ((color.1 as i64) << 16)
            | ((color.2 as i64) << 8)
            | (color.3 as i64);
        self.fill = Some(AesValue::Constant(PrimitiveValue::Int(rgba)));
        self
    }

    /// Set the default stroke color
    pub fn color(mut self, color: crate::theme::Color) -> Self {
        let rgba = ((color.0 as i64) << 24)
            | ((color.1 as i64) << 16)
            | ((color.2 as i64) << 8)
            | (color.3 as i64);
        self.color = Some(AesValue::Constant(PrimitiveValue::Int(rgba)));
        self
    }

    /// Set the default alpha/opacity
    pub fn alpha(mut self, alpha: f64) -> Self {
        self.alpha = Some(AesValue::Constant(PrimitiveValue::Float(
            alpha.clamp(0.0, 1.0),
        )));
        self
    }

    /// Set the bar width (as a proportion of spacing, typically 0.0-1.0)
    pub fn width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    /// Set the stat to use (default is Count)
    pub fn stat(mut self, stat: Stat) -> Self {
        self.stat = stat;
        self
    }

    /// Set the position adjustment (default is Identity)
    pub fn position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }
}

impl Default for GeomBar {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoLayer for GeomBar {
    fn default_aesthetics(&self) -> Vec<(Aesthetic, AesValue)> {
        let mut defaults = Vec::new();

        if let Some(fill) = &self.fill {
            defaults.push((Aesthetic::Fill, fill.clone()));
        }
        if let Some(color) = &self.color {
            defaults.push((Aesthetic::Color, color.clone()));
        }
        if let Some(alpha) = &self.alpha {
            defaults.push((Aesthetic::Alpha, alpha.clone()));
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
        }
    }
}

impl Geom for GeomBar {
    fn required_aesthetics(&self) -> &[Aesthetic] {
        &[Aesthetic::X, Aesthetic::Y]
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        // Get aesthetic values
        let x_normalized = ctx.get_aesthetic_values(Aesthetic::X, ctx.scales.x.as_ref())?;
        let y_normalized = ctx.get_aesthetic_values(Aesthetic::Y, ctx.scales.y.as_ref())?;
        let fills = ctx.get_fill_color_values()?;
        let colors = ctx.get_color_values()?;
        let alphas = ctx.get_aesthetic_values(Aesthetic::Alpha, None)?;

        // Collect x values to compute bar width
        let x_norm_vec: Vec<f64> = x_normalized.collect();
        let y_norm_vec: Vec<f64> = y_normalized.collect();
        let fills_vec: Vec<crate::theme::Color> = fills.collect();
        let colors_vec: Vec<crate::theme::Color> = colors.collect();
        let alphas_vec: Vec<f64> = alphas.collect();

        // Calculate the width of bars based on x spacing
        let bar_width_normalized = if x_norm_vec.len() > 1 {
            // Find minimum spacing between consecutive x values
            let mut min_spacing = f64::INFINITY;
            let mut sorted_x = x_norm_vec.clone();
            sorted_x.sort_by(|a, b| a.partial_cmp(b).unwrap());

            for i in 1..sorted_x.len() {
                let spacing = sorted_x[i] - sorted_x[i - 1];
                if spacing > 0.0 && spacing < min_spacing {
                    min_spacing = spacing;
                }
            }

            if min_spacing.is_finite() {
                min_spacing * self.width
            } else {
                0.1 // Fallback width
            }
        } else {
            0.1 // Single bar fallback width
        };

        // Get y=0 in normalized coordinates
        let zero_normalized = if let Some(y_scale) = ctx.scales.y.as_ref() {
            y_scale.map_value(0.0).unwrap_or(0.0)
        } else {
            0.0
        };

        // Render bars
        for ((((x_norm, y_norm), fill), color), alpha) in x_norm_vec
            .iter()
            .zip(y_norm_vec.iter())
            .zip(fills_vec.iter())
            .zip(colors_vec.iter())
            .zip(alphas_vec.iter())
        {
            let x_norm = *x_norm;
            let y_norm = *y_norm;
            // Map to device coordinates
            let x_center = ctx.map_x(x_norm);
            let y_top = ctx.map_y(y_norm);
            let y_bottom = ctx.map_y(zero_normalized);

            // Calculate bar width in device coordinates
            let half_width = ctx.map_x(x_norm + bar_width_normalized / 2.0)
                - ctx.map_x(x_norm - bar_width_normalized / 2.0);
            let half_width = (half_width / 2.0).abs();

            let x_left = x_center - half_width;
            let width = half_width * 2.0;
            let height = (y_bottom - y_top).abs();
            let y = y_top.min(y_bottom);

            // Fill the bar
            ctx.set_color_alpha(fill, *alpha);
            ctx.cairo.rectangle(x_left, y, width, height);
            ctx.cairo.fill().ok();

            // Stroke the bar if a stroke color is defined
            if self.color.is_some() {
                ctx.set_color_alpha(color, *alpha);
                ctx.cairo.rectangle(x_left, y, width, height);
                ctx.cairo.stroke().ok();
            }
        }

        Ok(())
    }
}
