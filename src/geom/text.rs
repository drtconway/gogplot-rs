use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::PlotError;

/// GeomText renders text labels at specified positions
pub struct GeomText {
    /// Default text color (if not mapped)
    pub color: Option<AesValue>,

    /// Default text size (if not mapped)
    pub size: Option<AesValue>,

    /// Default alpha/opacity (if not mapped)
    pub alpha: Option<AesValue>,

    /// Horizontal adjustment: 0 = left, 0.5 = center, 1 = right
    pub hjust: f64,

    /// Vertical adjustment: 0 = bottom, 0.5 = middle, 1 = top
    pub vjust: f64,

    /// Angle of text rotation in degrees (0 = horizontal, 90 = vertical)
    pub angle: f64,
}

impl GeomText {
    /// Create a new text geom with default settings from theme
    pub fn new() -> Self {
        use crate::theme::Theme;
        let theme = Theme::default();

        Self {
            color: None,
            size: None,
            alpha: None,
            hjust: theme.geom_text.hjust,
            vjust: theme.geom_text.vjust,
            angle: 0.0,
        }
    }

    /// Set the default text color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        self.color = Some(AesValue::constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the default text size
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

    /// Set horizontal justification (0 = left, 0.5 = center, 1 = right)
    pub fn hjust(&mut self, hjust: f64) -> &mut Self {
        self.hjust = hjust.clamp(0.0, 1.0);
        self
    }

    /// Set vertical justification (0 = bottom, 0.5 = middle, 1 = top)
    pub fn vjust(&mut self, vjust: f64) -> &mut Self {
        self.vjust = vjust.clamp(0.0, 1.0);
        self
    }

    /// Set text rotation angle in degrees
    pub fn angle(&mut self, angle: f64) -> &mut Self {
        self.angle = angle;
        self
    }
}

impl Default for GeomText {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoLayer for GeomText {
    fn default_aesthetics(&self) -> Vec<(Aesthetic, AesValue)> {
        use crate::data::PrimitiveValue;
        use crate::theme::Theme;

        let mut defaults = Vec::new();
        let theme = Theme::default();

        if let Some(color) = &self.color {
            defaults.push((Aesthetic::Color, color.clone()));
        } else {
            defaults.push((
                Aesthetic::Color,
                AesValue::constant(PrimitiveValue::Int(theme.geom_text.color.into())),
            ));
        }

        if let Some(alpha) = &self.alpha {
            defaults.push((Aesthetic::Alpha, alpha.clone()));
        } else {
            defaults.push((
                Aesthetic::Alpha,
                AesValue::constant(PrimitiveValue::Float(theme.geom_text.alpha)),
            ));
        }

        if let Some(size) = &self.size {
            defaults.push((Aesthetic::Size, size.clone()));
        } else {
            defaults.push((
                Aesthetic::Size,
                AesValue::constant(PrimitiveValue::Float(theme.geom_text.size)),
            ));
        }

        defaults
    }
}

impl Geom for GeomText {
    fn train_scales(&self, scales: &mut crate::scale::ScaleSet) {}

    fn apply_scales(&mut self, scales: &crate::scale::ScaleSet) {}

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        // Get position aesthetics
        let x_normalized = ctx.get_x_aesthetic_values(Aesthetic::X)?;
        let y_normalized = ctx.get_y_aesthetic_values(Aesthetic::Y)?;

        // Get label data using the helper method
        let labels = ctx.get_label_values()?;

        // Get other aesthetics
        let colors = ctx.get_color_values()?;
        let alphas = ctx.get_aesthetic_values(Aesthetic::Alpha, None)?;
        let sizes = ctx.get_aesthetic_values(Aesthetic::Size, None)?;

        // Zip all iterators together
        let iter = x_normalized
            .zip(y_normalized)
            .zip(labels)
            .zip(colors)
            .zip(alphas)
            .zip(sizes);

        for (((((x_norm, y_norm), label), color), alpha), size) in iter {
            let x_visual = ctx.map_x(x_norm);
            let y_visual = ctx.map_y(y_norm);

            // Set drawing properties
            ctx.set_color_alpha(&color, alpha);
            ctx.cairo.set_font_size(size);

            // Save the current transformation matrix
            ctx.cairo.save().ok();

            // Move to the position
            ctx.cairo.translate(x_visual, y_visual);

            // Rotate if needed
            if self.angle != 0.0 {
                ctx.cairo.rotate(self.angle.to_radians());
            }

            // Get text extents for alignment
            if let Ok(extents) = ctx.cairo.text_extents(&label) {
                // Calculate offset based on hjust and vjust
                let x_offset = -extents.width() * self.hjust - extents.x_bearing();
                let y_offset = -extents.height() * self.vjust - extents.y_bearing();

                ctx.cairo.move_to(x_offset, y_offset);
                ctx.cairo.show_text(&label).ok();
            }

            // Restore the transformation matrix
            ctx.cairo.restore().ok();
        }

        Ok(())
    }
}
