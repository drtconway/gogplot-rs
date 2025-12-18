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
    fn train_scales(&self, _scales: &mut crate::scale::ScaleSet) {
        
    }

    fn apply_scales(&mut self, scales: &crate::scale::ScaleSet) {
        
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        // Get position aesthetics (all pre-normalized to [0,1])
        let x_normalized = ctx.get_x_aesthetic_values(Aesthetic::X)?;
        let xmin_normalized = ctx.get_x_aesthetic_values(Aesthetic::Xmin)?;
        let xmax_normalized = ctx.get_x_aesthetic_values(Aesthetic::Xmax)?;
        let ymin_normalized = ctx.get_y_aesthetic_values(Aesthetic::Ymin)?;
        let ymax_normalized = ctx.get_y_aesthetic_values(Aesthetic::Ymax)?;

        // Get styling aesthetics
        let colors = ctx.get_color_values()?;
        let alphas = ctx.get_aesthetic_values(Aesthetic::Alpha, None)?;
        let sizes = ctx.get_aesthetic_values(Aesthetic::Size, None)?;

        // Zip all iterators together
        let iter = x_normalized
            .zip(xmin_normalized)
            .zip(xmax_normalized)
            .zip(ymin_normalized)
            .zip(ymax_normalized)
            .zip(colors)
            .zip(alphas)
            .zip(sizes);

        for (((((((x_norm, xmin_norm), xmax_norm), ymin_norm), ymax_norm), color), alpha), size) in iter {
            // Map normalized [0,1] coordinates to device coordinates
            let x_visual = ctx.map_x(x_norm);
            let xmin_visual = ctx.map_x(xmin_norm);
            let xmax_visual = ctx.map_x(xmax_norm);
            let ymin_visual = ctx.map_y(ymin_norm);
            let ymax_visual = ctx.map_y(ymax_norm);

            // Set drawing properties
            ctx.set_color_alpha(&color, alpha);
            ctx.cairo.set_line_width(size);

            // Draw vertical line from ymin to ymax
            ctx.cairo.move_to(x_visual, ymin_visual);
            ctx.cairo.line_to(x_visual, ymax_visual);
            ctx.cairo.stroke().ok();

            // Draw caps if width > 0
            if self.width > 0.0 {
                // Draw bottom cap
                ctx.cairo.move_to(xmin_visual, ymin_visual);
                ctx.cairo.line_to(xmax_visual, ymin_visual);
                ctx.cairo.stroke().ok();

                // Draw top cap
                ctx.cairo.move_to(xmin_visual, ymax_visual);
                ctx.cairo.line_to(xmax_visual, ymax_visual);
                ctx.cairo.stroke().ok();
            }
        }

        Ok(())
    }
}
