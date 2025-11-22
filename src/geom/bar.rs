use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::{DataType, PlotError};
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
    pub fn fill(&mut self, color: crate::theme::Color) -> &mut Self {
        self.fill = Some(AesValue::Constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the default stroke color
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

    /// Set the bar width (as a proportion of spacing, typically 0.0-1.0)
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width;
        self
    }

    /// Set the stat to use (default is Count)
    pub fn stat(&mut self, stat: Stat) -> &mut Self {
        self.stat = stat;
        self
    }

    /// Set the position adjustment (default is Identity)
    pub fn position(&mut self, position: Position) -> &mut Self {
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
            computed_data: None,
            computed_mapping: None,
            computed_scales: None,
        }
    }
}

impl Geom for GeomBar {
    fn required_aesthetics(&self) -> &[Aesthetic] {
        match self.stat {
            Stat::Count => &[Aesthetic::X],                  // Count only needs X
            Stat::Identity => &[Aesthetic::X, Aesthetic::Y], // Identity needs both
            _ => &[Aesthetic::X, Aesthetic::Y],
        }
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
        match self.stat {
            Stat::Count => {
                // Apply count stat
                // Count stat needs data ownership, so we manually implement it here

                let x_col_name = match mapping.get(&Aesthetic::X) {
                    Some(AesValue::Column(name)) => name,
                    _ => return Ok(None),
                };

                let x_col = data.get(x_col_name.as_str()).ok_or_else(|| {
                    PlotError::missing_column(x_col_name.as_str())
                })?;

                // Count occurrences
                use crate::utils::dataframe::{DataFrame, FloatVec, IntVec, StrVec};
                use std::collections::HashMap;

                let mut df = DataFrame::new();

                if let Some(int_iter) = x_col.iter_int() {
                    let mut counts: HashMap<i64, i64> = HashMap::new();
                    for val in int_iter {
                        *counts.entry(val).or_insert(0) += 1;
                    }
                    let mut pairs: Vec<(i64, i64)> = counts.into_iter().collect();
                    pairs.sort_by_key(|(x, _)| *x);

                    let x_vals: Vec<i64> = pairs.iter().map(|(x, _)| *x).collect();
                    let y_vals: Vec<i64> = pairs.iter().map(|(_, c)| *c).collect();

                    df.add_column("x", Box::new(IntVec(x_vals)));
                    df.add_column("y", Box::new(IntVec(y_vals)));
                } else if let Some(float_iter) = x_col.iter_float() {
                    let mut counts: HashMap<u64, (f64, i64)> = HashMap::new();
                    for val in float_iter {
                        if val.is_nan() {
                            continue;
                        }
                        let key = val.to_bits();
                        counts
                            .entry(key)
                            .and_modify(|(_, count)| *count += 1)
                            .or_insert((val, 1));
                    }
                    let mut pairs: Vec<(f64, i64)> = counts.into_values().collect();
                    pairs.sort_by(|(a, _), (b, _)| {
                        a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)
                    });

                    let x_vals: Vec<f64> = pairs.iter().map(|(x, _)| *x).collect();
                    let y_vals: Vec<i64> = pairs.iter().map(|(_, c)| *c).collect();

                    df.add_column("x", Box::new(FloatVec(x_vals)));
                    df.add_column("y", Box::new(IntVec(y_vals)));
                } else if let Some(str_iter) = x_col.iter_str() {
                    let mut counts: HashMap<String, i64> = HashMap::new();
                    for val in str_iter {
                        *counts.entry(val.to_string()).or_insert(0) += 1;
                    }
                    let mut pairs: Vec<(String, i64)> = counts.into_iter().collect();
                    pairs.sort_by(|(a, _), (b, _)| a.cmp(b));

                    let x_vals: Vec<String> = pairs.iter().map(|(x, _)| x.clone()).collect();
                    let y_vals: Vec<i64> = pairs.iter().map(|(_, c)| *c).collect();

                    df.add_column("x", Box::new(StrVec(x_vals)));
                    df.add_column("y", Box::new(IntVec(y_vals)));
                } else {
                    return Err(PlotError::InvalidAestheticType {
                        aesthetic: Aesthetic::X,
                        expected: DataType::Custom("numeric or string".to_string()),
                        actual: DataType::Custom("unknown".to_string()),
                    });
                }

                // Create updated mapping with Y pointing to computed "y" column
                let mut new_mapping = mapping.clone();
                new_mapping.set(Aesthetic::Y, AesValue::column("y"));

                Ok(Some((df, new_mapping)))
            }
            Stat::Identity => {
                // No transformation needed
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        // Check if we have Ymin/Ymax (from position adjustment like Stack)
        let has_ymin_ymax = ctx.data.get("ymin").is_some() && ctx.data.get("ymax").is_some();
        
        // Check if we have Xmin/Xmax (from position adjustment like Dodge)
        let has_xmin_xmax = ctx.data.get("xmin").is_some() && ctx.data.get("xmax").is_some();
        
        // Get aesthetic values
        let x_normalized = if !has_xmin_xmax {
            Some(ctx.get_aesthetic_values(Aesthetic::X, ctx.scales.x.as_deref())?)
        } else {
            None
        };
        
        let (xmin_normalized, xmax_normalized) = if has_xmin_xmax {
            // Xmin/Xmax are already in normalized [0,1] space from position adjustment
            // Don't map them through the scale again (that would be double-mapping!)
            let xmin = ctx.get_aesthetic_values(Aesthetic::Xmin, None)?;
            let xmax = ctx.get_aesthetic_values(Aesthetic::Xmax, None)?;
            (Some(xmin), Some(xmax))
        } else {
            (None, None)
        };
        
        let (ymin_normalized, ymax_normalized) = if has_ymin_ymax {
            // Use Ymin/Ymax for stacked bars
            let ymin = ctx.get_aesthetic_values(Aesthetic::Ymin, ctx.scales.y.as_deref())?;
            let ymax = ctx.get_aesthetic_values(Aesthetic::Ymax, ctx.scales.y.as_deref())?;
            (Some(ymin), Some(ymax))
        } else {
            (None, None)
        };
        
        let y_normalized = if !has_ymin_ymax {
            Some(ctx.get_aesthetic_values(Aesthetic::Y, ctx.scales.y.as_deref())?)
        } else {
            None
        };
        
        let fills = ctx.get_fill_color_values()?;
        let colors = ctx.get_color_values()?;
        let alphas = ctx.get_aesthetic_values(Aesthetic::Alpha, None)?;

        // Collect x values to compute bar width
        let (x_norm_vec, xmin_norm_vec, xmax_norm_vec) = if has_xmin_xmax {
            let xmin_vec: Vec<f64> = xmin_normalized.unwrap().collect();
            let xmax_vec: Vec<f64> = xmax_normalized.unwrap().collect();
            (None, Some(xmin_vec), Some(xmax_vec))
        } else {
            let x_vec: Vec<f64> = x_normalized.unwrap().collect();
            (Some(x_vec), None, None)
        };
        
        let (ymin_norm_vec, ymax_norm_vec, y_norm_vec) = if has_ymin_ymax {
            let ymin_vec: Vec<f64> = ymin_normalized.unwrap().collect();
            let ymax_vec: Vec<f64> = ymax_normalized.unwrap().collect();
            (Some(ymin_vec), Some(ymax_vec), None)
        } else {
            let y_vec: Vec<f64> = y_normalized.unwrap().collect();
            (None, None, Some(y_vec))
        };
        
        let fills_vec: Vec<crate::theme::Color> = fills.collect();
        let colors_vec: Vec<crate::theme::Color> = colors.collect();
        let alphas_vec: Vec<f64> = alphas.collect();

        // Calculate the width of bars based on x spacing (only if not using xmin/xmax)
        let bar_width_normalized = if !has_xmin_xmax {
            let x_vec = x_norm_vec.as_ref().unwrap();
            if x_vec.len() > 1 {
                // Find minimum spacing between consecutive x values
                let mut min_spacing = f64::INFINITY;
                let mut sorted_x = x_vec.clone();
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
            }
        } else {
            0.0 // Not used when xmin/xmax are provided
        };

        // Get y=0 in normalized coordinates (for non-stacked bars)
        let zero_normalized = if let Some(y_scale) = ctx.scales.y.as_deref() {
            y_scale.map_value(0.0).unwrap_or(0.0)
        } else {
            0.0
        };

        // Render bars
        let mut n = if has_xmin_xmax {
            xmin_norm_vec.as_ref().unwrap().len()
        } else {
            x_norm_vec.as_ref().unwrap().len()
        };
        
        // Ensure aesthetic vectors match data length
        n = n.min(fills_vec.len()).min(colors_vec.len()).min(alphas_vec.len());
        
        if has_ymin_ymax {
            let ymin_len = ymin_norm_vec.as_ref().unwrap().len();
            let ymax_len = ymax_norm_vec.as_ref().unwrap().len();
            n = n.min(ymin_len).min(ymax_len);
        } else {
            let y_len = y_norm_vec.as_ref().unwrap().len();
            n = n.min(y_len);
        }
        
        for i in 0..n {
            let fill = fills_vec[i];
            let color = colors_vec[i];
            let alpha = alphas_vec[i];
            
            // Determine y_top and y_bottom based on whether we're stacking
            let (y_top_norm, y_bottom_norm) = if has_ymin_ymax {
                (ymax_norm_vec.as_ref().unwrap()[i], ymin_norm_vec.as_ref().unwrap()[i])
            } else {
                (y_norm_vec.as_ref().unwrap()[i], zero_normalized)
            };
            
            // Map to device coordinates
            let y_top = ctx.map_y(y_top_norm);
            let y_bottom = ctx.map_y(y_bottom_norm);

            // Calculate bar position and width in device coordinates
            let (x_left, width) = if has_xmin_xmax {
                // Use xmin/xmax directly (from dodge position adjustment)
                let xmin_norm = xmin_norm_vec.as_ref().unwrap()[i];
                let xmax_norm = xmax_norm_vec.as_ref().unwrap()[i];
                let x_left = ctx.map_x(xmin_norm);
                let x_right = ctx.map_x(xmax_norm);
                (x_left, (x_right - x_left).abs())
            } else {
                // Use x center and calculated width
                let x_norm = x_norm_vec.as_ref().unwrap()[i];
                let x_center = ctx.map_x(x_norm);
                let half_width = ctx.map_x(x_norm + bar_width_normalized / 2.0)
                    - ctx.map_x(x_norm - bar_width_normalized / 2.0);
                let half_width = (half_width / 2.0).abs();
                (x_center - half_width, half_width * 2.0)
            };
            let height = (y_bottom - y_top).abs();
            let y = y_top.min(y_bottom);

            // Fill the bar
            ctx.set_color_alpha(&fill, alpha);
            ctx.cairo.rectangle(x_left, y, width, height);
            ctx.cairo.fill().ok();

            // Stroke the bar if a stroke color is defined
            if self.color.is_some() {
                ctx.set_color_alpha(&color, alpha);
                ctx.cairo.rectangle(x_left, y, width, height);
                ctx.cairo.stroke().ok();
            }
        }

        Ok(())
    }
}
