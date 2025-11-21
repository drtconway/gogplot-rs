use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::PlotError;
use crate::layer::{Position, Stat};

/// GeomHistogram renders a histogram by binning continuous data
/// By default, it uses Stat::Bin to divide the data into bins
pub struct GeomHistogram {
    /// Default fill color (if not mapped)
    pub fill: Option<AesValue>,

    /// Default stroke color (if not mapped)
    pub color: Option<AesValue>,

    /// Default alpha/opacity (if not mapped)
    pub alpha: Option<AesValue>,

    /// The stat to use (default is Bin with 30 bins)
    pub stat: Stat,

    /// The position adjustment (default is Identity, but Stack is common)
    pub position: Position,
}

impl GeomHistogram {
    /// Create a new histogram geom with default settings
    pub fn new() -> Self {
        Self {
            fill: None,
            color: None,
            alpha: None,
            stat: Stat::Bin(crate::stat::bin::BinStrategy::Count(30)),
            position: Position::Identity,
        }
    }

    /// Set the default fill color
    pub fn fill(mut self, color: crate::theme::Color) -> Self {
        self.fill = Some(AesValue::Constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the default stroke color
    pub fn color(mut self, color: crate::theme::Color) -> Self {
        self.color = Some(AesValue::Constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the default alpha/opacity
    pub fn alpha(mut self, alpha: f64) -> Self {
        self.alpha = Some(AesValue::Constant(PrimitiveValue::Float(
            alpha.clamp(0.0, 1.0),
        )));
        self
    }

    /// Set the number of bins
    pub fn bins(mut self, bins: usize) -> Self {
        self.stat = Stat::Bin(crate::stat::bin::BinStrategy::Count(bins));
        self
    }

    /// Set the bin width
    pub fn binwidth(mut self, binwidth: f64) -> Self {
        self.stat = Stat::Bin(crate::stat::bin::BinStrategy::Width(binwidth));
        self
    }

    /// Set the stat to use (default is Bin)
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

impl Default for GeomHistogram {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoLayer for GeomHistogram {
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
            computed_data: None,
            computed_mapping: None,
            computed_scales: None,
        }
    }
}

impl Geom for GeomHistogram {
    fn required_aesthetics(&self) -> &[Aesthetic] {
        // Histogram only needs X for input
        &[Aesthetic::X]
    }

    fn compute_stat(
        &self,
        _data: &dyn crate::data::DataSource,
        _mapping: &crate::aesthetics::AesMap,
    ) -> Result<
        Option<(
            crate::utils::dataframe::DataFrame,
            crate::aesthetics::AesMap,
        )>,
        PlotError,
    > {
        // Stat is applied in the plot's apply_stats phase
        // We don't compute it here
        Ok(None)
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        // After stat transformation, we should have:
        // - X: bin centers (for positioning labels if needed)
        // - Y or count: bin counts
        // - Xmin, Xmax: bin boundaries (mapped from xmin/xmax columns)

        // Try to use Xmin and Xmax for precise bin boundaries
        let has_bin_boundaries = ctx.mapping.get(&Aesthetic::Xmin).is_some()
            && ctx.mapping.get(&Aesthetic::Xmax).is_some();

        if has_bin_boundaries {
            self.render_with_boundaries(ctx)
        } else {
            self.render_without_boundaries(ctx)
        }
    }
}

impl GeomHistogram {
    /// Render histogram using Xmin/Xmax bin boundaries (preferred)
    fn render_with_boundaries(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        // Get bin boundaries from aesthetics
        let xmin_values = ctx.get_aesthetic_values(Aesthetic::Xmin, None)?;
        let xmax_values = ctx.get_aesthetic_values(Aesthetic::Xmax, None)?;

        let xmin_vec: Vec<f64> = xmin_values.collect();
        let xmax_vec: Vec<f64> = xmax_values.collect();

        // Check if we have Ymin/Ymax (from position adjustment like Stack)
        let has_ymin_ymax = ctx.data.get("ymin").is_some() && ctx.data.get("ymax").is_some();

        // Get y values (counts) or ymin/ymax for stacking
        let (ymin_normalized, ymax_normalized, y_normalized) = if has_ymin_ymax {
            let ymin = ctx.get_aesthetic_values(Aesthetic::Ymin, ctx.scales.y.as_deref())?;
            let ymax = ctx.get_aesthetic_values(Aesthetic::Ymax, ctx.scales.y.as_deref())?;
            (Some(ymin), Some(ymax), None)
        } else {
            let y = ctx.get_aesthetic_values(Aesthetic::Y, ctx.scales.y.as_deref())?;
            (None, None, Some(y))
        };

        let (ymin_norm_vec, ymax_norm_vec, y_norm_vec) = if has_ymin_ymax {
            let ymin_vec: Vec<f64> = ymin_normalized.unwrap().collect();
            let ymax_vec: Vec<f64> = ymax_normalized.unwrap().collect();
            (Some(ymin_vec), Some(ymax_vec), None)
        } else {
            let y_vec: Vec<f64> = y_normalized.unwrap().collect();
            (None, None, Some(y_vec))
        };

        // Get fill, color, and alpha
        let fills = ctx.get_fill_color_values()?;
        let colors = ctx.get_color_values()?;
        let alphas = ctx.get_aesthetic_values(Aesthetic::Alpha, None)?;

        let fills_vec: Vec<crate::theme::Color> = fills.collect();
        let colors_vec: Vec<crate::theme::Color> = colors.collect();
        let alphas_vec: Vec<f64> = alphas.collect();

        // Get y=0 in normalized coordinates (for non-stacked histograms)
        let zero_normalized = if let Some(y_scale) = ctx.scales.y.as_deref() {
            y_scale.map_value(0.0).unwrap_or(0.0)
        } else {
            0.0
        };

        // Render bars using bin boundaries
        let n = xmin_vec.len();
        for i in 0..n {
            let xmin = xmin_vec[i];
            let xmax = xmax_vec[i];
            let fill = &fills_vec[i];
            let color = &colors_vec[i];
            let alpha = alphas_vec[i];
            
            // Determine y_top and y_bottom based on whether we're stacking
            let (y_top_norm, y_bottom_norm) = if has_ymin_ymax {
                (ymax_norm_vec.as_ref().unwrap()[i], ymin_norm_vec.as_ref().unwrap()[i])
            } else {
                (y_norm_vec.as_ref().unwrap()[i], zero_normalized)
            };
            // Normalize x boundaries
            let xmin_norm = if let Some(x_scale) = ctx.scales.x.as_deref() {
                x_scale.map_value(xmin).unwrap_or(0.0)
            } else {
                xmin
            };
            let xmax_norm = if let Some(x_scale) = ctx.scales.x.as_deref() {
                x_scale.map_value(xmax).unwrap_or(1.0)
            } else {
                xmax
            };

            // Map to device coordinates
            let x_left = ctx.map_x(xmin_norm);
            let x_right = ctx.map_x(xmax_norm);
            let y_top = ctx.map_y(y_top_norm);
            let y_bottom = ctx.map_y(y_bottom_norm);

            let width = (x_right - x_left).abs();
            let height = (y_bottom - y_top).abs();
            let y = y_top.min(y_bottom);

            // Fill the bar
            ctx.set_color_alpha(fill, alpha);
            ctx.cairo.rectangle(x_left, y, width, height);
            ctx.cairo.fill().ok();

            // Stroke the bar if a stroke color is defined
            if self.color.is_some() {
                ctx.set_color_alpha(color, alpha);
                ctx.cairo.rectangle(x_left, y, width, height);
                ctx.cairo.stroke().ok();
            }
        }

        Ok(())
    }

    /// Render histogram without explicit bin boundaries (fallback)
    fn render_without_boundaries(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        // Fallback to using x centers and computing width from spacing
        let x_normalized = ctx.get_aesthetic_values(Aesthetic::X, ctx.scales.x.as_deref())?;
        let y_normalized = ctx.get_aesthetic_values(Aesthetic::Y, ctx.scales.y.as_deref())?;

        let x_norm_vec: Vec<f64> = x_normalized.collect();
        let y_norm_vec: Vec<f64> = y_normalized.collect();

        let fills = ctx.get_fill_color_values()?;
        let colors = ctx.get_color_values()?;
        let alphas = ctx.get_aesthetic_values(Aesthetic::Alpha, None)?;

        let fills_vec: Vec<crate::theme::Color> = fills.collect();
        let colors_vec: Vec<crate::theme::Color> = colors.collect();
        let alphas_vec: Vec<f64> = alphas.collect();

        // Calculate bar width based on x spacing
        let bar_width_normalized = if x_norm_vec.len() > 1 {
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
                min_spacing
            } else {
                0.1
            }
        } else {
            0.1
        };

        // Get y=0 in normalized coordinates
        let zero_normalized = if let Some(y_scale) = ctx.scales.y.as_deref() {
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
            let x_center = ctx.map_x(*x_norm);
            let y_top = ctx.map_y(*y_norm);
            let y_bottom = ctx.map_y(zero_normalized);

            let half_width = ctx.map_x(*x_norm + bar_width_normalized / 2.0)
                - ctx.map_x(*x_norm - bar_width_normalized / 2.0);
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
