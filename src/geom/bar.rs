

use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::{DataType, PlotError};
use crate::geom::context::compute_min_spacing;
use crate::layer::{Position, Stat};
use crate::scale::ScaleType;
use ordered_float::OrderedFloat;

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
        self.fill = Some(AesValue::constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the default stroke color
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

    fn aesthetic_scale_type(&self, aesthetic: Aesthetic) -> ScaleType {
        match aesthetic {
            // Bar charts typically have categorical X axis (one bar per category)
            Aesthetic::X => ScaleType::Categorical,
            // Y-axis should be continuous (heights/counts)
            Aesthetic::Y => ScaleType::Continuous,
            // Other aesthetics can be either
            _ => ScaleType::Either,
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
                    Some(AesValue::Column { name, .. }) => name,
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
        let mapping = ctx.mapping();
        
        eprintln!("DEBUG GeomBar::render");
        eprintln!("  data len: {}", ctx.data().len());
        eprintln!("  mapping: {:?}", mapping.iter().map(|(k, v)| format!("{:?}: {:?}", k, v)).collect::<Vec<_>>());
        
        // Determine if we have range aesthetics
        let has_x_range = mapping.contains(Aesthetic::Xmin) && mapping.contains(Aesthetic::Xmax);
        let has_y_range = mapping.contains(Aesthetic::Ymin) && mapping.contains(Aesthetic::Ymax);
        
        eprintln!("  has_x_range: {}, has_y_range: {}", has_x_range, has_y_range);
        
        // Calculate bar half-width in normalized space (only if not using xmin/xmax)
        let bar_half_width_norm = if !has_x_range {
            // Check if the X scale is categorical
            let is_categorical = ctx.scales.x.as_ref()
                .map(|scale| scale.scale_type() == ScaleType::Categorical)
                .unwrap_or(false);
            
            if is_categorical {
                // For categorical x scales, bars should fill a proportion of each category bin
                // Categorical scales space categories evenly with step = (range.max - range.min) / n_categories
                // Each bar should occupy self.width (e.g., 0.9) of that step
                // The normalized positions are already at the center of each bin
                
                // Collect unique x positions to determine the categorical spacing
                let x_values: Vec<f64> = ctx.get_x_aesthetic_values(Aesthetic::X)?
                    .filter(|x| x.is_finite())
                    .collect();
                
                if x_values.len() > 1 {
                    // Get unique sorted positions
                    let mut unique_x: Vec<OrderedFloat<f64>> = x_values.iter()
                        .map(|&x| OrderedFloat(x))
                        .collect();
                    unique_x.sort();
                    unique_x.dedup();
                    
                    // The spacing between consecutive categories is the categorical step
                    let categorical_step = unique_x[1].0 - unique_x[0].0;
                    let bar_half_width = categorical_step * self.width / 2.0;
                    
                    bar_half_width
                } else {
                    // Single category - assume full normalized range [0,1] is one category
                    // Bar half-width is (1.0 * width) / 2
                    self.width / 2.0
                }
            } else {
                // For continuous x scales, use the old behavior
                compute_min_spacing(ctx.get_x_aesthetic_values(Aesthetic::X)?, self.width)
            }
        } else {
            0.0 // Not used when xmin/xmax provided
        };
        
        // Get y=0 baseline in normalized coordinates
        let y_baseline = if let Some(y_scale) = ctx.scales.y.as_deref() {
            y_scale.map_value(0.0).unwrap_or(0.0)
        } else {
            0.0
        };
        
        // Get x value iterators (center or min/max)
        let x_vals = if has_x_range {
            ctx.get_x_aesthetic_values(Aesthetic::Xmin)?
        } else {
            ctx.get_x_aesthetic_values(Aesthetic::X)?
        };
        
        let x_max_vals = if has_x_range {
            Some(ctx.get_x_aesthetic_values(Aesthetic::Xmax)?)
        } else {
            None
        };
        
        // Get y value iterators (top or min/max)
        let y_vals = if has_y_range {
            ctx.get_y_aesthetic_values(Aesthetic::Ymin)?
        } else {
            ctx.get_y_aesthetic_values(Aesthetic::Y)?
        };
        
        let y_max_vals = if has_y_range {
            Some(ctx.get_y_aesthetic_values(Aesthetic::Ymax)?)
        } else {
            None
        };
        
        // Get styling aesthetic iterators
        let fills = ctx.get_fill_color_values()?;
        let colors = ctx.get_color_values()?;
        let alphas = ctx.get_unscaled_aesthetic_values(Aesthetic::Alpha)?;
        
        // Build the combined iterator based on whether we have range aesthetics
        let iter: Box<dyn Iterator<Item = (f64, Option<f64>, f64, Option<f64>, _, _, f64)>> = 
            if has_x_range && has_y_range {
                Box::new(
                    x_vals
                        .zip(x_max_vals.unwrap())
                        .zip(y_vals)
                        .zip(y_max_vals.unwrap())
                        .zip(fills)
                        .zip(colors)
                        .zip(alphas)
                        .map(|((((((x, x_max), y), y_max), fill), color), alpha)| {
                            (x, Some(x_max), y, Some(y_max), fill, color, alpha)
                        })
                )
            } else if has_x_range {
                Box::new(
                    x_vals
                        .zip(x_max_vals.unwrap())
                        .zip(y_vals)
                        .zip(fills)
                        .zip(colors)
                        .zip(alphas)
                        .map(move |(((((x, x_max), y), fill), color), alpha)| {
                            (x, Some(x_max), y, None, fill, color, alpha)
                        })
                )
            } else if has_y_range {
                Box::new(
                    x_vals
                        .zip(y_vals)
                        .zip(y_max_vals.unwrap())
                        .zip(fills)
                        .zip(colors)
                        .zip(alphas)
                        .map(move |(((((x, y), y_max), fill), color), alpha)| {
                            (x, None, y, Some(y_max), fill, color, alpha)
                        })
                )
            } else {
                Box::new(
                    x_vals
                        .zip(y_vals)
                        .zip(fills)
                        .zip(colors)
                        .zip(alphas)
                        .map(move |((((x, y), fill), color), alpha)| {
                            (x, None, y, None, fill, color, alpha)
                        })
                )
            };
        
        // Render each bar
        let mut bar_count = 0;
        for (x_val, x_max_val, y_val, y_max_val, fill, color, alpha) in iter {
            // Compute effective xmin/xmax in normalized space
            let (xmin_norm, xmax_norm) = if let Some(x_max) = x_max_val {
                (x_val, x_max)
            } else {
                (x_val - bar_half_width_norm, x_val + bar_half_width_norm)
            };
            
            // Compute effective ymin/ymax in normalized space
            let (ymin_norm, ymax_norm) = if let Some(y_max) = y_max_val {
                (y_val, y_max)
            } else {
                (y_baseline, y_val)
            };
            
            eprintln!("  Bar {}: xmin_norm={}, xmax_norm={}, ymin_norm={}, ymax_norm={}", bar_count, xmin_norm, xmax_norm, ymin_norm, ymax_norm);
            
            // Map to device coordinates
            let x_left = ctx.map_x(xmin_norm);
            let x_right = ctx.map_x(xmax_norm);
            let y_top = ctx.map_y(ymax_norm);
            let y_bottom = ctx.map_y(ymin_norm);
            
            eprintln!("    Device: x={:.1}, y={:.1}, width={:.1}, height={:.1}", x_left.min(x_right), y_top.min(y_bottom), (x_right - x_left).abs(), (y_bottom - y_top).abs());
            bar_count += 1;
            
            let x = x_left.min(x_right);
            let y = y_top.min(y_bottom);
            let width = (x_right - x_left).abs();
            let height = (y_bottom - y_top).abs();
            
            // Fill the bar
            ctx.set_color_alpha(&fill, alpha);
            ctx.cairo.rectangle(x, y, width, height);
            ctx.cairo.fill().ok();
            
            // Stroke the bar if a stroke color is defined
            if self.color.is_some() {
                ctx.set_color_alpha(&color, alpha);
                ctx.cairo.rectangle(x, y, width, height);
                ctx.cairo.stroke().ok();
            }
        }
        
        Ok(())
    }
}
