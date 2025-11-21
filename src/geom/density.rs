use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::PlotError;
use crate::stat::density::Density as DensityStat;

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
    pub fn color(mut self, color: crate::theme::Color) -> Self {
        let rgba = color.into();
        self.color = Some(AesValue::Constant(PrimitiveValue::Int(rgba)));
        self
    }

    /// Set the default line width
    pub fn size(mut self, size: f64) -> Self {
        self.size = Some(AesValue::Constant(PrimitiveValue::Float(size)));
        self
    }

    /// Set the default alpha/opacity
    pub fn alpha(mut self, alpha: f64) -> Self {
        self.alpha = Some(AesValue::Constant(PrimitiveValue::Float(
            alpha.clamp(0.0, 1.0),
        )));
        self
    }

    /// Set bandwidth adjustment multiplier
    pub fn adjust(mut self, adjust: f64) -> Self {
        self.adjust = adjust;
        self
    }

    /// Set number of evaluation points
    pub fn n(mut self, n: usize) -> Self {
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
}

impl Geom for GeomDensity {
    fn required_aesthetics(&self) -> &[Aesthetic] {
        // Density only requires X - Y is computed
        &[Aesthetic::X]
    }

    fn compute_stat(
        &self,
        data: &dyn crate::data::DataSource,
        mapping: &crate::aesthetics::AesMap,
    ) -> Result<
        Option<(
            crate::utils::dataframe::DataFrame,
            crate::aesthetics::AesMap,
        )>,
        PlotError,
    > {
        use crate::aesthetics::AesValue;

        // Get x column name from mapping
        let x_col = match mapping.get(&Aesthetic::X) {
            Some(AesValue::Column(col)) => col,
            _ => return Ok(None), // No column mapping, can't compute
        };

        // Get x data
        let x_vec = data
            .get(x_col.as_str())
            .ok_or_else(|| PlotError::missing_column(x_col))?;

        let x_float = x_vec
            .as_float()
            .ok_or_else(|| PlotError::invalid_column_type(x_col, "numeric"))?;

        // Collect x values
        let x_values: Vec<f64> = x_float.iter().copied().collect();

        // Compute density
        let density_stat = DensityStat::new().adjust(self.adjust).n(self.n);

        let density_df = density_stat.compute(&x_values)?;

        // Create updated mapping with Y pointing to "density" column
        let mut new_mapping = mapping.clone();
        new_mapping.set(Aesthetic::Y, AesValue::column("density"));

        Ok(Some((density_df, new_mapping)))
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        // At this point, ctx.data should contain the computed density with columns: x, density
        let x_vec = ctx
            .data
            .get("x")
            .ok_or_else(|| PlotError::missing_column("x"))?;
        let y_vec = ctx
            .data
            .get("density")
            .ok_or_else(|| PlotError::missing_column("density"))?;

        let x_float = x_vec
            .as_float()
            .ok_or_else(|| PlotError::invalid_column_type("x", "numeric"))?;
        let y_float = y_vec
            .as_float()
            .ok_or_else(|| PlotError::invalid_column_type("density", "numeric"))?;

        // Normalize using scales
        let x_vals: Vec<f64> = if let Some(x_scale) = ctx.scales.x.as_deref() {
            x_float
                .iter()
                .filter_map(|&x| x_scale.map_value(x))
                .collect()
        } else {
            x_float.iter().copied().collect()
        };

        let y_vals: Vec<f64> = if let Some(y_scale) = ctx.scales.y.as_deref() {
            y_float
                .iter()
                .filter_map(|&y| y_scale.map_value(y))
                .collect()
        } else {
            y_float.iter().copied().collect()
        };

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
        if !x_vals.is_empty() {
            let x0_visual = ctx.map_x(x_vals[0]);
            let y0_visual = ctx.map_y(y_vals[0]);
            ctx.cairo.move_to(x0_visual, y0_visual);

            for i in 1..x_vals.len() {
                let x_visual = ctx.map_x(x_vals[i]);
                let y_visual = ctx.map_y(y_vals[i]);
                ctx.cairo.line_to(x_visual, y_visual);
            }

            ctx.cairo.stroke().ok();
        }

        Ok(())
    }
}
