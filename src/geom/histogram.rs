use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::PlotError;
use crate::geom::context::compute_min_spacing;
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
            stat: Stat::Bin(crate::stat::bin::BinStrategy::Count(30).into()),
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

    /// Set the number of bins
    pub fn bins(&mut self, bins: usize) -> &mut Self {
        self.stat = Stat::Bin(crate::stat::bin::BinStrategy::Count(bins).into());
        self
    }

    /// Set the bin width
    pub fn binwidth(&mut self, binwidth: f64) -> &mut Self {
        self.stat = Stat::Bin(crate::stat::bin::BinStrategy::Width(binwidth).into());
        self
    }

    /// Enable or disable cumulative histogram
    pub fn cumulative(&mut self, cumulative: bool) -> &mut Self {
        // Extract the current strategy and update it with cumulative flag
        if let Stat::Bin(ref strategy) = self.stat {
            let mut new_strategy = strategy.clone();
            new_strategy.cumulative = cumulative;
            self.stat = Stat::Bin(new_strategy);
        }
        self
    }

    /// Set the stat to use (default is Bin)
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
            mapping: Some(mapping),
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

    fn setup_data(
        &self,
        data: &dyn crate::data::DataSource,
        mapping: &crate::aesthetics::AesMap,
    ) -> Result<(Option<Box<dyn crate::data::DataSource>>, Option<crate::aesthetics::AesMap>), PlotError> {
        // If the stat has already created Xmin/Xmax (e.g., from Stat::Bin), we don't need to do anything
        if mapping.contains(Aesthetic::Xmin) && mapping.contains(Aesthetic::Xmax) {
            return Ok((None, None));
        }

        // Otherwise, we need to create xmin/xmax from X values (similar to GeomBar)
        // This handles the case where histogram is used without Stat::Bin
        
        use crate::utils::dataframe::{DataFrame, FloatVec};
        use crate::error::DataType;

        // Get the X column name from the mapping
        let x_col_name = match mapping.get(&Aesthetic::X) {
            Some(AesValue::Column { name, .. }) => name.as_str(),
            _ => return Ok((None, None)), // No X column mapping, nothing to set up
        };

        // Get the X column from the data
        let x_col = data.get(x_col_name)
            .ok_or_else(|| PlotError::missing_column(x_col_name))?;

        // Convert x values to floats
        let x_vals: Vec<f64> = if let Some(int_iter) = x_col.iter_int() {
            int_iter.map(|v| v as f64).collect()
        } else if let Some(float_iter) = x_col.iter_float() {
            float_iter.collect()
        } else {
            return Err(PlotError::InvalidAestheticType {
                aesthetic: Aesthetic::X,
                expected: DataType::Custom("numeric".to_string()),
                actual: DataType::Custom("unknown".to_string()),
            });
        };

        // Find minimum spacing between unique values
        let mut unique_x: Vec<ordered_float::OrderedFloat<f64>> = x_vals
            .iter()
            .filter(|x| x.is_finite())
            .map(|&x| ordered_float::OrderedFloat(x))
            .collect();
        unique_x.sort();
        unique_x.dedup();
        
        let spacing = if unique_x.len() > 1 {
            let mut min_spacing = f64::MAX;
            for i in 1..unique_x.len() {
                let spacing = unique_x[i].0 - unique_x[i - 1].0;
                if spacing < min_spacing {
                    min_spacing = spacing;
                }
            }
            min_spacing
        } else {
            1.0
        };

        let half_width = spacing / 2.0;

        // Create xmin and xmax columns
        let xmin_vals: Vec<f64> = x_vals.iter().map(|x| x - half_width).collect();
        let xmax_vals: Vec<f64> = x_vals.iter().map(|x| x + half_width).collect();

        // Create a new dataframe with all original columns plus xmin/xmax
        let mut new_df = DataFrame::new();
        
        use crate::data::{VectorIter, GenericVector};
        use crate::utils::dataframe::{IntVec, StrVec, BoolVec};
        
        for col_name in data.column_names() {
            if let Some(col) = data.get(&col_name) {
                let new_col: Box<dyn GenericVector> = match col.iter() {
                    VectorIter::Int(iter) => Box::new(IntVec(iter.collect())),
                    VectorIter::Float(iter) => Box::new(FloatVec(iter.collect())),
                    VectorIter::Str(iter) => Box::new(StrVec(iter.map(|s| s.to_string()).collect())),
                    VectorIter::Bool(iter) => Box::new(BoolVec(iter.collect())),
                };
                new_df.add_column(&col_name, new_col);
            }
        }

        // Add xmin and xmax columns
        new_df.add_column("xmin", Box::new(FloatVec(xmin_vals)));
        new_df.add_column("xmax", Box::new(FloatVec(xmax_vals)));

        // Update the mapping to include Xmin and Xmax aesthetics
        let mut new_mapping = mapping.clone();
        new_mapping.set(Aesthetic::Xmin, AesValue::column("xmin"));
        new_mapping.set(Aesthetic::Xmax, AesValue::column("xmax"));

        Ok((Some(Box::new(new_df) as Box<dyn crate::data::DataSource>), Some(new_mapping)))
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        let mapping = ctx.mapping();
        
        // Determine if we have range aesthetics
        let has_x_range = mapping.contains(Aesthetic::Xmin) && mapping.contains(Aesthetic::Xmax);
        let has_y_range = mapping.contains(Aesthetic::Ymin) && mapping.contains(Aesthetic::Ymax);
        
        // Get y=0 baseline in normalized coordinates
        let y_baseline = if let Some(y_scale) = ctx.scales.y.as_deref() {
            y_scale.map_value(0.0).unwrap_or(0.0)
        } else {
            0.0
        };
        
        // Calculate bin half-width if needed (for bins without explicit boundaries)
        let bin_half_width_norm = if !has_x_range {
           compute_min_spacing(ctx.get_x_aesthetic_values(Aesthetic::X)?, 1.0)
        } else {
            0.0 // Not used when xmin/xmax provided
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
        
        // Build the combined iterator
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
        for (x_val, x_max_val, y_val, y_max_val, fill, color, alpha) in iter {
            // Compute effective xmin/xmax in normalized space
            let (xmin_norm, xmax_norm) = if let Some(x_max) = x_max_val {
                (x_val, x_max)
            } else {
                (x_val - bin_half_width_norm, x_val + bin_half_width_norm)
            };
            
            // Compute effective ymin/ymax in normalized space
            let (ymin_norm, ymax_norm) = if let Some(y_max) = y_max_val {
                (y_val, y_max)
            } else {
                (y_baseline, y_val)
            };
            
            // Map to device coordinates
            let x_left = ctx.map_x(xmin_norm);
            let x_right = ctx.map_x(xmax_norm);
            let y_top = ctx.map_y(ymax_norm);
            let y_bottom = ctx.map_y(ymin_norm);
            
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
