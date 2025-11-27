use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::PlotError;

/// GeomLabel renders text labels with a background box at specified positions
pub struct GeomLabel {
    /// Default text color (if not mapped)
    pub color: Option<AesValue>,

    /// Default text size (if not mapped)
    pub size: Option<AesValue>,

    /// Default alpha/opacity for text (if not mapped)
    pub alpha: Option<AesValue>,

    /// Default fill color for label background (if not mapped)
    pub fill: Option<AesValue>,

    /// Horizontal adjustment: 0 = left, 0.5 = center, 1 = right
    pub hjust: f64,

    /// Vertical adjustment: 0 = bottom, 0.5 = middle, 1 = top
    pub vjust: f64,

    /// Angle of text rotation in degrees (0 = horizontal, 90 = vertical)
    pub angle: f64,

    /// Padding around text in the label box (in points)
    pub padding: f64,

    /// Corner radius for rounded label boxes (0 = sharp corners)
    pub radius: f64,
}

impl GeomLabel {
    /// Create a new label geom with default settings from theme
    pub fn new() -> Self {
        use crate::theme::Theme;
        let theme = Theme::default();
        
        Self {
            color: None,
            size: None,
            alpha: None,
            fill: None,
            hjust: theme.geom_text.hjust,
            vjust: theme.geom_text.vjust,
            angle: 0.0,
            padding: 2.0,
            radius: 2.0,
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

    /// Set the default alpha/opacity for text
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = Some(AesValue::constant(PrimitiveValue::Float(
            alpha.clamp(0.0, 1.0),
        )));
        self
    }

    /// Set the default fill color for label background
    pub fn fill(&mut self, color: crate::theme::Color) -> &mut Self {
        self.fill = Some(AesValue::constant(PrimitiveValue::Int(color.into())));
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

    /// Set padding around text in the label box
    pub fn padding(&mut self, padding: f64) -> &mut Self {
        self.padding = padding.max(0.0);
        self
    }

    /// Set corner radius for rounded label boxes
    pub fn radius(&mut self, radius: f64) -> &mut Self {
        self.radius = radius.max(0.0);
        self
    }
}

impl Default for GeomLabel {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoLayer for GeomLabel {
    fn default_aesthetics(&self) -> Vec<(Aesthetic, AesValue)> {
        use crate::theme::Theme;
        use crate::data::PrimitiveValue;
        
        let mut defaults = Vec::new();
        let theme = Theme::default();

        if let Some(color) = &self.color {
            defaults.push((Aesthetic::Color, color.clone()));
        } else {
            defaults.push((Aesthetic::Color, AesValue::constant(PrimitiveValue::Int(theme.geom_text.color.into()))));
        }
        
        if let Some(alpha) = &self.alpha {
            defaults.push((Aesthetic::Alpha, alpha.clone()));
        } else {
            defaults.push((Aesthetic::Alpha, AesValue::constant(PrimitiveValue::Float(theme.geom_text.alpha))));
        }
        
        if let Some(size) = &self.size {
            defaults.push((Aesthetic::Size, size.clone()));
        } else {
            defaults.push((Aesthetic::Size, AesValue::constant(PrimitiveValue::Float(theme.geom_text.size))));
        }

        if let Some(fill) = &self.fill {
            defaults.push((Aesthetic::Fill, fill.clone()));
        } else {
            // Default to white background for labels
            defaults.push((Aesthetic::Fill, AesValue::constant(PrimitiveValue::Int(
                crate::theme::Color(255, 255, 255, 230).into()
            ))));
        }

        defaults
    }
}

impl Geom for GeomLabel {
    fn required_aesthetics(&self) -> &[Aesthetic] {
        &[Aesthetic::X, Aesthetic::Y, Aesthetic::Label]
    }

    fn setup_data(
        &self,
        _data: &dyn crate::data::DataSource,
        _mapping: &crate::aesthetics::AesMap,
    ) -> Result<(Option<Box<dyn crate::data::DataSource>>, Option<crate::aesthetics::AesMap>), PlotError> {
        // Geom doesn't need to add any columns
        Ok((None, None))
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        // Get position aesthetics
        let x_normalized = ctx.get_x_aesthetic_values(Aesthetic::X)?;
        let y_normalized = ctx.get_y_aesthetic_values(Aesthetic::Y)?;

        // Get label data
        let labels = ctx.get_label_values()?;

        // Get other aesthetics
        let colors = ctx.get_color_values()?;
        let fills = ctx.get_fill_color_values()?;
        let alphas = ctx.get_aesthetic_values(Aesthetic::Alpha, None)?;
        let sizes = ctx.get_aesthetic_values(Aesthetic::Size, None)?;

        // Zip all iterators together
        let iter = x_normalized
            .zip(y_normalized)
            .zip(labels)
            .zip(colors)
            .zip(fills)
            .zip(alphas)
            .zip(sizes);

        for ((((((x_norm, y_norm), label), color), fill), alpha), size) in iter {
            let x_visual = ctx.map_x(x_norm);
            let y_visual = ctx.map_y(y_norm);

            // Save the current transformation matrix
            ctx.cairo.save().ok();

            // Move to the position
            ctx.cairo.translate(x_visual, y_visual);

            // Rotate if needed
            if self.angle != 0.0 {
                ctx.cairo.rotate(self.angle.to_radians());
            }

            // Set font size for measuring
            ctx.cairo.set_font_size(size);

            // Get text extents for sizing the label box
            if let Ok(extents) = ctx.cairo.text_extents(&label) {
                // Get font extents for proper vertical metrics
                let font_extents = ctx.cairo.font_extents().ok();
                
                // For vertical metrics, use font extents if available (includes full ascent/descent)
                // This ensures tall characters like 'L', 'd', etc. are not clipped
                let (effective_height, effective_y_bearing) = if let Some(fe) = font_extents {
                    (fe.ascent() + fe.descent(), -fe.ascent())
                } else {
                    (extents.height(), extents.y_bearing())
                };
                
                // Calculate text offset based on hjust and vjust
                let text_x_offset = -extents.width() * self.hjust - extents.x_bearing();
                let text_y_offset = -effective_height * self.vjust - effective_y_bearing;

                // Calculate box dimensions with padding
                let box_width = extents.width() + 2.0 * self.padding;
                let box_height = effective_height + 2.0 * self.padding;
                
                // Position box to surround the text
                let box_x = text_x_offset + extents.x_bearing() - self.padding;
                let box_y = text_y_offset + effective_y_bearing - self.padding;

                // Draw rounded rectangle background
                if self.radius > 0.0 {
                    let r = self.radius.min(box_width / 2.0).min(box_height / 2.0);
                    ctx.cairo.new_path();
                    ctx.cairo.arc(box_x + r, box_y + r, r, std::f64::consts::PI, 3.0 * std::f64::consts::PI / 2.0);
                    ctx.cairo.arc(box_x + box_width - r, box_y + r, r, 3.0 * std::f64::consts::PI / 2.0, 0.0);
                    ctx.cairo.arc(box_x + box_width - r, box_y + box_height - r, r, 0.0, std::f64::consts::PI / 2.0);
                    ctx.cairo.arc(box_x + r, box_y + box_height - r, r, std::f64::consts::PI / 2.0, std::f64::consts::PI);
                    ctx.cairo.close_path();
                } else {
                    ctx.cairo.rectangle(box_x, box_y, box_width, box_height);
                }

                // Fill the background
                ctx.set_color(&fill);
                ctx.cairo.fill_preserve().ok();

                // Draw border (using text color with lower alpha)
                ctx.set_color_alpha(&color, alpha * 0.5);
                ctx.cairo.set_line_width(0.5);
                ctx.cairo.stroke().ok();

                // Draw the text
                ctx.set_color_alpha(&color, alpha);
                ctx.cairo.move_to(text_x_offset, text_y_offset);
                ctx.cairo.show_text(&label).ok();
            }

            // Restore the transformation matrix
            ctx.cairo.restore().ok();
        }

        Ok(())
    }
}
