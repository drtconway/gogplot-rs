use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::PlotError;
use crate::layer::Stat;
use crate::scale::ScaleType;

/// GeomBoxplot renders box-and-whisker plots
///
/// Box-and-whisker plots display the distribution of a continuous variable.
/// They show five key statistics (computed by stat_boxplot):
/// - Ymin: Lower whisker extent
/// - Lower: First quartile (Q1)
/// - Middle: Median
/// - Upper: Third quartile (Q3)
/// - Ymax: Upper whisker extent
///
/// The box spans from Q1 to Q3, with a line at the median.
/// Whiskers extend to Ymin and Ymax (typically 1.5 * IQR from the box).
/// Outliers beyond the whiskers are shown as points.
///
/// # Required Aesthetics
///
/// When using Stat::Boxplot (default), only X and Y are required.
/// The stat computes Lower, Middle, Upper, Ymin, Ymax.
///
/// When using Stat::Identity, these are required:
/// - X: Position along x-axis (typically categorical)
/// - Lower: First quartile (Q1)
/// - Middle: Median
/// - Upper: Third quartile (Q3)
/// - Ymin: Lower whisker extent
/// - Ymax: Upper whisker extent
///
/// # Optional Aesthetics
///
/// - Fill: Box fill color (can be constant or mapped)
/// - Color: Box outline and whisker color
/// - Alpha: Transparency (0.0 = transparent, 1.0 = opaque)
/// - Size: Line width for box outline and whiskers
pub struct GeomBoxplot {
    /// Default fill color (if not mapped)
    pub fill: Option<AesValue>,

    /// Default stroke color (if not mapped)
    pub color: Option<AesValue>,

    /// Default alpha/opacity (if not mapped)
    pub alpha: Option<AesValue>,

    /// Default line width (if not mapped)
    pub size: Option<AesValue>,

    /// Box width (as proportion of spacing between x values)
    pub width: f64,

    /// Stat to use (default is Boxplot, can be Identity if data already computed)
    pub stat: Stat,

    /// Position adjustment (default is Identity, but Dodge is useful for grouped boxplots)
    pub position: crate::layer::Position,

    /// IQR coefficient for outlier detection (default 1.5)
    pub coef: f64,
}

impl GeomBoxplot {
    /// Create a new boxplot geom with default settings
    pub fn new() -> Self {
        Self {
            fill: None,
            color: None,
            alpha: None,
            size: None,
            width: 0.5,
            stat: Stat::Boxplot { coef: 1.5 },
            position: crate::layer::Position::Identity,
            coef: 1.5,
        }
    }

    /// Set the default fill color for the box
    pub fn fill(&mut self, color: crate::theme::Color) -> &mut Self {
        self.fill = Some(AesValue::constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the default outline color for the box and whiskers
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        self.color = Some(AesValue::constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the default alpha/opacity
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = Some(AesValue::constant(PrimitiveValue::Float(
            alpha.clamp(0.0, 1.0),
        )));
        self
    }

    /// Set the default line width for box outline and whiskers
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size = Some(AesValue::constant(PrimitiveValue::Float(size)));
        self
    }

    /// Set the box width (as proportion of spacing between x values, typically 0.0-1.0)
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width;
        self
    }

    /// Set the stat to use (default is Boxplot)
    pub fn stat(&mut self, stat: Stat) -> &mut Self {
        self.stat = stat;
        self
    }

    /// Set the IQR coefficient for outlier detection (default 1.5)
    pub fn coef(&mut self, coef: f64) -> &mut Self {
        self.coef = coef;
        self
    }

    /// Set the position adjustment (default is Identity)
    pub fn position(&mut self, position: crate::layer::Position) -> &mut Self {
        self.position = position;
        self
    }
}

impl Default for GeomBoxplot {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoLayer for GeomBoxplot {
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
        if let Some(size) = &self.size {
            defaults.push((Aesthetic::Size, size.clone()));
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
            mapping: Some(mapping),
            stat,
            position,
            computed_data: None,
            computed_mapping: None,
            computed_scales: None,
        }
    }
}

impl Geom for GeomBoxplot {
    fn required_aesthetics(&self) -> &[Aesthetic] {
        // The stat computes Lower, Middle, Upper, Ymin, Ymax from X and Y
        // So we only require X here - the layer will handle Y requirement for the stat
        &[Aesthetic::X]
    }

    fn aesthetic_scale_type(&self, aesthetic: Aesthetic) -> ScaleType {
        match aesthetic {
            // Boxplots typically have categorical X axis (one box per category)
            Aesthetic::X => ScaleType::Categorical,
            // Y-axis and related aesthetics should be continuous
            Aesthetic::Y | Aesthetic::Ymin | Aesthetic::Ymax 
            | Aesthetic::Lower | Aesthetic::Middle | Aesthetic::Upper => ScaleType::Continuous,
            // Other aesthetics can be either
            _ => ScaleType::Either,
        }
    }

    fn setup_data(
        &self,
        _data: &dyn crate::data::DataSource,
        mapping: &crate::aesthetics::AesMap,
    ) -> Result<(Option<Box<dyn crate::data::DataSource>>, Option<crate::aesthetics::AesMap>), PlotError> {
        // If Xmin/Xmax are already in the mapping (e.g., from previous setup or position adjustment),
        // we don't need to do anything
        if mapping.contains(Aesthetic::Xmin) && mapping.contains(Aesthetic::Xmax) {
            return Ok((None, None));
        }

        // For boxplot, X is categorical, so we map both Xmin and Xmax to the same X aesthetic
        // This allows the categorical scale to position them correctly, and position adjustments
        // like Dodge can modify these mappings if needed
        let x_aes = match mapping.get(&Aesthetic::X) {
            Some(aes) => aes,
            None => return Ok((None, None)), // No X mapping, nothing to set up
        };

        let mut new_mapping = mapping.clone();
        new_mapping.set(Aesthetic::Xmin, x_aes.clone());
        new_mapping.set(Aesthetic::Xmax, x_aes.clone());

        Ok((None, Some(new_mapping)))
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        let mapping = ctx.mapping();
        
        // Verify required aesthetics are present (setup_data should have created Xmin/Xmax)
        if !mapping.contains(Aesthetic::X) {
            return Err(PlotError::MissingAesthetic {
                aesthetic: Aesthetic::X,
            });
        }
        if !mapping.contains(Aesthetic::Xmin) {
            return Err(PlotError::MissingAesthetic {
                aesthetic: Aesthetic::Xmin,
            });
        }
        if !mapping.contains(Aesthetic::Xmax) {
            return Err(PlotError::MissingAesthetic {
                aesthetic: Aesthetic::Xmax,
            });
        }

        // Get x value iterators - all pre-normalized by apply_scales
        // After setup_data, Xmin and Xmax both map to X (same position)
        // Position adjustments like Dodge may modify Xmin/Xmax to create actual ranges
        let xmin_normalized = ctx.get_x_aesthetic_values(Aesthetic::Xmin)?;
        let x_normalized = ctx.get_x_aesthetic_values(Aesthetic::X)?;
        let xmax_normalized = ctx.get_x_aesthetic_values(Aesthetic::Xmax)?;
        
        // Get box statistics - all pre-normalized by apply_scales
        let lower_normalized = ctx.get_y_aesthetic_values(Aesthetic::Lower)?;
        let middle_normalized = ctx.get_y_aesthetic_values(Aesthetic::Middle)?;
        let upper_normalized = ctx.get_y_aesthetic_values(Aesthetic::Upper)?;
        let ymin_normalized = ctx.get_y_aesthetic_values(Aesthetic::Ymin)?;
        let ymax_normalized = ctx.get_y_aesthetic_values(Aesthetic::Ymax)?;
        
        // Get y values (for outliers) - pre-normalized by apply_scales
        let y_normalized = ctx.get_y_aesthetic_values(Aesthetic::Y)?;

        // Get styling aesthetics
        let fills = ctx.get_fill_color_values()?;
        let colors = ctx.get_color_values()?;
        let alphas = ctx.get_unscaled_aesthetic_values(Aesthetic::Alpha)?;
        let sizes = ctx.get_unscaled_aesthetic_values(Aesthetic::Size)?;

        // Build the combined iterator with all aesthetics
        // Zip in stages to avoid deeply nested tuples
        let x_iter = xmin_normalized.zip(x_normalized).zip(xmax_normalized);
        let y_box_iter = lower_normalized.zip(middle_normalized).zip(upper_normalized);
        let y_whisker_iter = ymin_normalized.zip(ymax_normalized).zip(y_normalized);
        let style_iter = fills.zip(colors).zip(alphas).zip(sizes);
        
        let iter = x_iter.zip(y_box_iter).zip(y_whisker_iter).zip(style_iter);

        for (((x_vals, y_box_vals), y_whisker_vals), style_vals) in iter {
            let ((xmin_norm, _x_norm), xmax_norm) = x_vals;
            let ((lower_norm, middle_norm), upper_norm) = y_box_vals;
            let ((ymin_norm, ymax_norm), y_norm) = y_whisker_vals;
            let (((fill, color), alpha), size) = style_vals;
            // Check if this is an outlier row (middle is NaN)
            if middle_norm.is_nan() {
                // This is an outlier - draw as a point
                if !y_norm.is_nan() {
                    // Use xmin/xmax for box position
                    let x_center_norm = (xmin_norm + xmax_norm) / 2.0;
                    let x_visual = ctx.map_x(x_center_norm);
                    let outlier_y_visual = ctx.map_y(y_norm);
                    
                    ctx.set_color_alpha(&color, alpha);
                    ctx.cairo.arc(x_visual, outlier_y_visual, 2.0, 0.0, 2.0 * std::f64::consts::PI);
                    ctx.cairo.fill().ok();
                }
                continue;
            }

            // This is a box row - draw box, whiskers, etc.
            let lower_visual = ctx.map_y(lower_norm);
            let middle_visual = ctx.map_y(middle_norm);
            let upper_visual = ctx.map_y(upper_norm);
            let ymin_visual = ctx.map_y(ymin_norm);
            let ymax_visual = ctx.map_y(ymax_norm);

            // Use xmin/xmax for box edges
            let box_left = ctx.map_x(xmin_norm);
            let box_right = ctx.map_x(xmax_norm);
            let x_center_norm = (xmin_norm + xmax_norm) / 2.0;
            let x_visual = ctx.map_x(x_center_norm);

            let box_width = (box_right - box_left).abs();
            let box_height = (upper_visual - lower_visual).abs();

            ctx.cairo.set_line_width(size);

            // Draw the box (Q1 to Q3)
            ctx.set_color_alpha(&fill, alpha);
            ctx.cairo.rectangle(
                box_left,
                lower_visual.min(upper_visual),
                box_width,
                box_height,
            );
            ctx.cairo.fill_preserve().ok();

            // Stroke the box outline
            ctx.set_color_alpha(&color, alpha);
            ctx.cairo.stroke().ok();

            // Draw the median line
            ctx.cairo.move_to(box_left, middle_visual);
            ctx.cairo.line_to(box_right, middle_visual);
            ctx.cairo.stroke().ok();

            // Draw lower whisker (vertical line from ymin to Q1)
            ctx.cairo.move_to(x_visual, ymin_visual);
            ctx.cairo.line_to(x_visual, lower_visual);
            ctx.cairo.stroke().ok();

            // Draw lower whisker cap (horizontal line at ymin)
            let whisker_cap_width = box_width * 0.5;
            ctx.cairo.move_to(x_visual - whisker_cap_width / 2.0, ymin_visual);
            ctx.cairo.line_to(x_visual + whisker_cap_width / 2.0, ymin_visual);
            ctx.cairo.stroke().ok();

            // Draw upper whisker (vertical line from Q3 to ymax)
            ctx.cairo.move_to(x_visual, upper_visual);
            ctx.cairo.line_to(x_visual, ymax_visual);
            ctx.cairo.stroke().ok();

            // Draw upper whisker cap (horizontal line at ymax)
            ctx.cairo.move_to(x_visual - whisker_cap_width / 2.0, ymax_visual);
            ctx.cairo.line_to(x_visual + whisker_cap_width / 2.0, ymax_visual);
            ctx.cairo.stroke().ok();
        }

        Ok(())
    }
}

