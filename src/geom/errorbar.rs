use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::PlotError;

/// GeomErrorbar renders vertical error bars with optional caps
pub struct GeomErrorbar {
    /// Default line color (if not mapped)
    pub color: Option<AesValue>,

    /// Default line width (if not mapped)
    pub size: Option<AesValue>,

    /// Default alpha/opacity (if not mapped)
    pub alpha: Option<AesValue>,

    /// Width of the caps at the ends of the error bars (in data coordinates)
    pub width: f64,
}

impl GeomErrorbar {
    /// Create a new errorbar geom with default settings
    pub fn new() -> Self {
        Self {
            color: None,
            size: None,
            alpha: None,
            width: 0.5,
        }
    }

    /// Set the default line color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        self.color = Some(AesValue::constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the default line width
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size = Some(AesValue::constant(PrimitiveValue::Float(size)));
        self
    }

    /// Set the default alpha/opacity
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = Some(AesValue::constant(PrimitiveValue::Float(
            alpha.clamp(0.0, 1.0),
        )));
        self
    }

    /// Set the width of the caps (in data coordinates)
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width.max(0.0);
        self
    }
}

impl Default for GeomErrorbar {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoLayer for GeomErrorbar {
    fn default_aesthetics(&self) -> Vec<(Aesthetic, AesValue)> {
        use crate::theme::Theme;
        
        let mut defaults = Vec::new();
        let theme = Theme::default();

        if let Some(color) = &self.color {
            defaults.push((Aesthetic::Color, color.clone()));
        } else {
            defaults.push((Aesthetic::Color, AesValue::constant(PrimitiveValue::Int(theme.geom_line.color.into()))));
        }
        
        if let Some(alpha) = &self.alpha {
            defaults.push((Aesthetic::Alpha, alpha.clone()));
        } else {
            defaults.push((Aesthetic::Alpha, AesValue::constant(PrimitiveValue::Float(theme.geom_line.alpha))));
        }
        
        if let Some(size) = &self.size {
            defaults.push((Aesthetic::Size, size.clone()));
        } else {
            defaults.push((Aesthetic::Size, AesValue::constant(PrimitiveValue::Float(theme.geom_line.size))));
        }

        defaults
    }
}

impl Geom for GeomErrorbar {
    fn required_aesthetics(&self) -> &[Aesthetic] {
        &[Aesthetic::X, Aesthetic::Ymin, Aesthetic::Ymax]
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        // Get position aesthetics
        let x_normalized = ctx.get_x_aesthetic_values(Aesthetic::X)?;
        let ymin_normalized = ctx.get_y_aesthetic_values(Aesthetic::Ymin)?;
        let ymax_normalized = ctx.get_y_aesthetic_values(Aesthetic::Ymax)?;

        // Get styling aesthetics
        let colors = ctx.get_color_values()?;
        let alphas = ctx.get_aesthetic_values(Aesthetic::Alpha, None)?;
        let sizes = ctx.get_aesthetic_values(Aesthetic::Size, None)?;

        // Calculate cap width in normalized coordinates
        // Convert data width to normalized coordinates
        let x_scale = ctx.scales.x.as_ref()
            .ok_or_else(|| PlotError::MissingAesthetic { aesthetic: Aesthetic::X })?;
        
        // Get two points in data space separated by self.width
        let x_center = 0.0;
        let x_left = x_center - self.width / 2.0;
        let x_right = x_center + self.width / 2.0;
        
        let x_left_norm = x_scale.map_value(x_left).unwrap_or(0.0);
        let x_right_norm = x_scale.map_value(x_right).unwrap_or(0.0);
        
        let cap_width_norm = x_right_norm - x_left_norm;

        // Zip all iterators together
        let iter = x_normalized
            .zip(ymin_normalized)
            .zip(ymax_normalized)
            .zip(colors)
            .zip(alphas)
            .zip(sizes);

        for (((((x_norm, ymin_norm), ymax_norm), color), alpha), size) in iter {
            let x_visual = ctx.map_x(x_norm);
            let ymin_visual = ctx.map_y(ymin_norm);
            let ymax_visual = ctx.map_y(ymax_norm);

            // Calculate cap positions
            let cap_half_width = ctx.map_x(x_norm + cap_width_norm / 2.0) - x_visual;

            // Set drawing properties
            ctx.set_color_alpha(&color, alpha);
            ctx.cairo.set_line_width(size);

            // Draw vertical line from ymin to ymax
            ctx.cairo.move_to(x_visual, ymin_visual);
            ctx.cairo.line_to(x_visual, ymax_visual);
            ctx.cairo.stroke().ok();

            // Draw bottom cap
            if self.width > 0.0 {
                ctx.cairo.move_to(x_visual - cap_half_width, ymin_visual);
                ctx.cairo.line_to(x_visual + cap_half_width, ymin_visual);
                ctx.cairo.stroke().ok();

                // Draw top cap
                ctx.cairo.move_to(x_visual - cap_half_width, ymax_visual);
                ctx.cairo.line_to(x_visual + cap_half_width, ymax_visual);
                ctx.cairo.stroke().ok();
            }
        }

        Ok(())
    }
}
