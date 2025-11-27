use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::{PlotError, to_plot_error};

/// GeomDensity renders kernel density estimates
///
/// This geom automatically computes the density using the specified stat parameters
/// and renders it as a line plot.
pub struct GeomDensity {
    /// Default line color (if not mapped)
    pub color: Option<AesValue>,

    /// Default line width (if not mapped)
    pub size: Option<AesValue>,

    /// Default alpha/opacity (if not mapped)
    pub alpha: Option<AesValue>,

    /// Bandwidth adjustment multiplier (default 1.0)
    pub adjust: f64,

    /// Number of evaluation points (default 512)
    pub n: usize,
}

impl GeomDensity {
    /// Create a new density geom with default settings
    pub fn new() -> Self {
        Self {
            color: None,
            size: None,
            alpha: None,
            adjust: 1.0,
            n: 512,
        }
    }

    /// Set the default line color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        let rgba = color.into();
        self.color = Some(AesValue::constant(PrimitiveValue::Int(rgba)));
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

    /// Set bandwidth adjustment multiplier
    pub fn adjust(&mut self, adjust: f64) -> &mut Self {
        self.adjust = adjust;
        self
    }

    /// Set number of evaluation points
    pub fn n(&mut self, n: usize) -> &mut Self {
        self.n = n;
        self
    }
}

impl Default for GeomDensity {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoLayer for GeomDensity {
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

        defaults
    }

    fn into_layer(self) -> crate::layer::Layer {
        use crate::layer::{Layer, Stat, Position};
        use crate::aesthetics::AesMap;

        let adjust = self.adjust;
        let n = self.n;

        let mut mapping = AesMap::new();

        // Set default aesthetics from geom settings if provided
        for (aesthetic, value) in self.default_aesthetics() {
            mapping.set(aesthetic, value);
        }

        Layer {
            geom: Box::new(self),
            data: None,
            mapping: Some(mapping),
            stat: Stat::Density { adjust, n },
            position: Position::Identity,
            computed_data: None,
            computed_mapping: None,
            computed_scales: None,
        }
    }
}

impl Geom for GeomDensity {
    fn required_aesthetics(&self) -> &[Aesthetic] {
        // Density only requires X - Y is computed
        &[Aesthetic::X]
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        // Get x and y aesthetic values
        let x_vals = ctx.get_x_aesthetic_values(Aesthetic::X)?;
        let y_vals = ctx.get_y_aesthetic_values(Aesthetic::Y)?;

        // Get color, alpha, and size
        let colors = ctx.get_color_values()?;
        let alphas = ctx.get_aesthetic_values(Aesthetic::Alpha, None)?;
        let sizes = ctx.get_aesthetic_values(Aesthetic::Size, None)?;

        let colors_vec: Vec<_> = colors.collect();
        let alphas_vec: Vec<_> = alphas.collect();
        let sizes_vec: Vec<_> = sizes.collect();

        // Use first value for line properties (density is a single curve)
        let color = &colors_vec[0];
        let alpha = alphas_vec[0];
        let size = sizes_vec[0];

        // Set drawing properties
        ctx.set_color_alpha(color, alpha);
        ctx.cairo.set_line_width(size);

        // Draw the density curve
        for (i, (x_norm, y_norm)) in x_vals.zip(y_vals).enumerate() {
            let x_visual = ctx.map_x(x_norm);
            let y_visual = ctx.map_y(y_norm);

            if i == 0 {
                ctx.cairo.move_to(x_visual, y_visual);
            } else {
                ctx.cairo.line_to(x_visual, y_visual);
            }
        }
        ctx.cairo.stroke().map_err(to_plot_error)?;
        Ok(())
    }
}
