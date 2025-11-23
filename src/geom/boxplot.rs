use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::PlotError;
use crate::layer::Stat;

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
        self.fill = Some(AesValue::Constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the default outline color for the box and whiskers
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        self.color = Some(AesValue::Constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the default alpha/opacity
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = Some(AesValue::Constant(PrimitiveValue::Float(
            alpha.clamp(0.0, 1.0),
        )));
        self
    }

    /// Set the default line width for box outline and whiskers
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size = Some(AesValue::Constant(PrimitiveValue::Float(size)));
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
            mapping,
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

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        use crate::geom::context::compute_min_spacing;

        let mapping = ctx.mapping();
        
        // Check if we have Xmin/Xmax (provided by position adjustments like Dodge)
        let has_x_range = mapping.contains(Aesthetic::Xmin) && mapping.contains(Aesthetic::Xmax);

        // Calculate box half-width in normalized space (only if not using xmin/xmax)
        let box_half_width_norm = if !has_x_range {
            compute_min_spacing(
                ctx.get_x_aesthetic_values(Aesthetic::X)?,
                self.width,
            )
        } else {
            0.0 // Not used when xmin/xmax provided
        };

        // Get x value iterators (center or min/max)
        let x_normalized = if has_x_range {
            ctx.get_x_aesthetic_values(Aesthetic::Xmin)?
        } else {
            ctx.get_x_aesthetic_values(Aesthetic::X)?
        };
        
        let x_max_normalized = if has_x_range {
            Some(ctx.get_x_aesthetic_values(Aesthetic::Xmax)?)
        } else {
            None
        };
        
        // Get box statistics - need manual scaling to preserve NaN values for outlier rows
        let lower_col = ctx.data().get("lower").ok_or_else(|| PlotError::missing_column("lower"))?;
        let lower_raw: Vec<f64> = lower_col.iter_float()
            .ok_or_else(|| PlotError::invalid_column_type("lower", "float"))?
            .collect();
            
        let middle_col = ctx.data().get("middle").ok_or_else(|| PlotError::missing_column("middle"))?;
        let middle_raw: Vec<f64> = middle_col.iter_float()
            .ok_or_else(|| PlotError::invalid_column_type("middle", "float"))?
            .collect();
            
        let upper_col = ctx.data().get("upper").ok_or_else(|| PlotError::missing_column("upper"))?;
        let upper_raw: Vec<f64> = upper_col.iter_float()
            .ok_or_else(|| PlotError::invalid_column_type("upper", "float"))?
            .collect();
            
        let ymin_col = ctx.data().get("ymin").ok_or_else(|| PlotError::missing_column("ymin"))?;
        let ymin_raw: Vec<f64> = ymin_col.iter_float()
            .ok_or_else(|| PlotError::invalid_column_type("ymin", "float"))?
            .collect();
            
        let ymax_col = ctx.data().get("ymax").ok_or_else(|| PlotError::missing_column("ymax"))?;
        let ymax_raw: Vec<f64> = ymax_col.iter_float()
            .ok_or_else(|| PlotError::invalid_column_type("ymax", "float"))?
            .collect();
        
        // Apply y scale to non-NaN values, preserve NaN for outlier rows
        let y_scale = ctx.scales.y.as_ref();
        let lower_normalized: Vec<f64> = lower_raw.iter().map(|&v| {
            if v.is_nan() {
                v
            } else if let Some(scale) = y_scale {
                scale.map_value(v).unwrap_or(v)
            } else {
                v
            }
        }).collect();
        
        let middle_normalized: Vec<f64> = middle_raw.iter().map(|&v| {
            if v.is_nan() {
                v
            } else if let Some(scale) = y_scale {
                scale.map_value(v).unwrap_or(v)
            } else {
                v
            }
        }).collect();
        
        let upper_normalized: Vec<f64> = upper_raw.iter().map(|&v| {
            if v.is_nan() {
                v
            } else if let Some(scale) = y_scale {
                scale.map_value(v).unwrap_or(v)
            } else {
                v
            }
        }).collect();
        
        let ymin_normalized: Vec<f64> = ymin_raw.iter().map(|&v| {
            if v.is_nan() {
                v
            } else if let Some(scale) = y_scale {
                scale.map_value(v).unwrap_or(v)
            } else {
                v
            }
        }).collect();
        
        let ymax_normalized: Vec<f64> = ymax_raw.iter().map(|&v| {
            if v.is_nan() {
                v
            } else if let Some(scale) = y_scale {
                scale.map_value(v).unwrap_or(v)
            } else {
                v
            }
        }).collect();

        // Get styling aesthetics
        let fills = ctx.get_fill_color_values()?;
        let colors = ctx.get_color_values()?;
        let alphas = ctx.get_unscaled_aesthetic_values(Aesthetic::Alpha)?;
        let sizes = ctx.get_unscaled_aesthetic_values(Aesthetic::Size)?;
        
        // Get y values (for outliers) - need special handling to preserve NaN
        let y_col = ctx.data().get("y").ok_or_else(|| PlotError::missing_column("y"))?;
        let y_raw: Vec<f64> = y_col.iter_float()
            .ok_or_else(|| PlotError::invalid_column_type("y", "float"))?
            .collect();
        
        // Apply y scale to non-NaN values, preserve NaN for box rows
        let y_normalized: Vec<f64> = if let Some(y_scale) = ctx.scales.y.as_ref() {
            y_raw.iter().map(|&v| {
                if v.is_nan() {
                    v  // Preserve NaN
                } else {
                    y_scale.map_value(v).unwrap_or(v)  // Scale non-NaN values
                }
            }).collect()
        } else {
            y_raw
        };

        // Collect iterators into vectors for building the combined iterator
        use crate::theme::Color;
        let x_vec: Vec<f64> = x_normalized.collect();
        let x_max_vec: Option<Vec<f64>> = x_max_normalized.map(|iter| iter.collect());
        let y_vec: Vec<f64> = y_normalized;  // Already a Vec
        let lower_vec: Vec<f64> = lower_normalized;  // Already a Vec
        let middle_vec: Vec<f64> = middle_normalized;  // Already a Vec
        let upper_vec: Vec<f64> = upper_normalized;  // Already a Vec
        let ymin_vec: Vec<f64> = ymin_normalized;  // Already a Vec
        let ymax_vec: Vec<f64> = ymax_normalized;  // Already a Vec
        let fills_vec: Vec<Color> = fills.collect();
        let colors_vec: Vec<Color> = colors.collect();
        let alphas_vec: Vec<f64> = alphas.collect();
        let sizes_vec: Vec<f64> = sizes.collect();
        


        // Build the combined iterator based on whether we have x range aesthetics
        let iter: Box<dyn Iterator<Item = (f64, Option<f64>, f64, f64, f64, f64, f64, f64, Color, Color, f64, f64)>> = 
            if has_x_range {
                let x_max_vec_unwrapped = x_max_vec.unwrap();
                Box::new(
                    x_vec.into_iter()
                        .zip(x_max_vec_unwrapped.into_iter())
                        .zip(y_vec.into_iter())
                        .zip(lower_vec.into_iter())
                        .zip(middle_vec.into_iter())
                        .zip(upper_vec.into_iter())
                        .zip(ymin_vec.into_iter())
                        .zip(ymax_vec.into_iter())
                        .zip(fills_vec.into_iter())
                        .zip(colors_vec.into_iter())
                        .zip(alphas_vec.into_iter())
                        .zip(sizes_vec.into_iter())
                        .map(|(((((((((((xmin, xmax), y), lower), middle), upper), ymin), ymax), fill), color), alpha), size)| {
                            (xmin, Some(xmax), y, lower, middle, upper, ymin, ymax, fill, color, alpha, size)
                        })
                )
            } else {
                Box::new(
                    x_vec.into_iter()
                        .zip(y_vec.into_iter())
                        .zip(lower_vec.into_iter())
                        .zip(middle_vec.into_iter())
                        .zip(upper_vec.into_iter())
                        .zip(ymin_vec.into_iter())
                        .zip(ymax_vec.into_iter())
                        .zip(fills_vec.into_iter())
                        .zip(colors_vec.into_iter())
                        .zip(alphas_vec.into_iter())
                        .zip(sizes_vec.into_iter())
                        .map(|((((((((((x, y), lower), middle), upper), ymin), ymax), fill), color), alpha), size)| {
                            (x, None, y, lower, middle, upper, ymin, ymax, fill, color, alpha, size)
                        })
                )
            };

        let mut count = 0;
        for (_i, (x_norm, x_max_opt, y_norm, lower_norm, middle_norm, upper_norm, ymin_norm, ymax_norm, fill, color, alpha, size)) in iter.enumerate() {
            count += 1;
            // Check if this is an outlier row (middle is NaN)
            if middle_norm.is_nan() {
                // This is an outlier - draw as a point
                if !y_norm.is_nan() {
                    let (box_left_norm, box_right_norm) = if let Some(x_max) = x_max_opt {
                        (x_norm, x_max)
                    } else {
                        (x_norm - box_half_width_norm, x_norm + box_half_width_norm)
                    };
                    let x_center_norm = (box_left_norm + box_right_norm) / 2.0;
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

            // Calculate box left and right edges
            let (box_left_norm, box_right_norm) = if let Some(x_max) = x_max_opt {
                // Use provided xmin/xmax (from position adjustments)
                (x_norm, x_max)
            } else {
                // Calculate from center x and width
                (x_norm - box_half_width_norm, x_norm + box_half_width_norm)
            };
            
            let box_left = ctx.map_x(box_left_norm);
            let box_right = ctx.map_x(box_right_norm);
            let x_center_norm = (box_left_norm + box_right_norm) / 2.0;
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

