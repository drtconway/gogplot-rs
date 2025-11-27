

use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::{DataType, PlotError};
use crate::geom::context::AestheticValues;
use crate::layer::{Position, Stat};
use crate::scale::ScaleType;

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
            mapping: Some(mapping),
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

    fn setup_data(
        &self,
        data: &dyn crate::data::DataSource,
        mapping: &crate::aesthetics::AesMap,
    ) -> Result<(Option<Box<dyn crate::data::DataSource>>, Option<crate::aesthetics::AesMap>), PlotError> {
        use crate::utils::dataframe::{DataFrame, FloatVec};

        // Get the X column name from the mapping
        let x_col_name = match mapping.get(&Aesthetic::X) {
            Some(AesValue::Column { name, .. }) => name.as_str(),
            _ => return Ok((None, None)), // No X mapping, nothing to set up
        };

        // Get the X column from the data
        let x_col = data.get(x_col_name)
            .ok_or_else(|| PlotError::missing_column(x_col_name))?;

        // Convert x values to floats for width calculations
        let x_vals: Vec<f64> = if let Some(int_iter) = x_col.iter_int() {
            int_iter.map(|v| v as f64).collect()
        } else if let Some(float_iter) = x_col.iter_float() {
            float_iter.collect()
        } else if x_col.iter_str().is_some() {
            // For categorical x, we'll use indices (0, 1, 2, ...)
            // This will be mapped by the categorical scale later
            (0..data.len()).map(|i| i as f64).collect()
        } else {
            return Err(PlotError::InvalidAestheticType {
                aesthetic: Aesthetic::X,
                expected: DataType::Custom("numeric or string".to_string()),
                actual: DataType::Custom("unknown".to_string()),
            });
        };

        // Compute the spacing for width calculation
        // For categorical data or when we can't compute spacing, use 1.0
        // For continuous data, find minimum spacing between unique values
        let spacing = if x_col.iter_str().is_some() {
            1.0
        } else {
            let mut unique_x: Vec<ordered_float::OrderedFloat<f64>> = x_vals
                .iter()
                .filter(|x| x.is_finite())
                .map(|&x| ordered_float::OrderedFloat(x))
                .collect();
            unique_x.sort();
            unique_x.dedup();
            
            if unique_x.len() > 1 {
                // Find minimum spacing
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
            }
        };

        let half_width = (self.width * spacing) / 2.0;

        // Create xmin and xmax columns
        let xmin_vals: Vec<f64> = x_vals.iter().map(|x| x - half_width).collect();
        let xmax_vals: Vec<f64> = x_vals.iter().map(|x| x + half_width).collect();

        // Create a new dataframe with all original columns plus xmin/xmax
        let mut new_df = DataFrame::new();
        
        // Copy all original columns using iter() to reconstruct them
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
        // NEW SIMPLIFIED RENDER: Data is pre-normalized to [0,1] and position-adjusted
        // setup_data() has already created xmin/xmax columns
        // apply_scales() has already normalized everything to [0,1]
        // position adjustments (dodge/stack) have already been applied
        
        let mapping = ctx.mapping();
        
        // Get normalized [0,1] coordinates - all data is already scaled
        let xmin_vals = ctx.get_x_aesthetic_values(Aesthetic::Xmin)?;
        let xmax_vals = ctx.get_x_aesthetic_values(Aesthetic::Xmax)?;
        
        // For Y, check if we have Ymin/Ymax (from stack) or just Y
        let has_y_range = mapping.contains(Aesthetic::Ymin) && mapping.contains(Aesthetic::Ymax);
        
        let ymin_vals = if has_y_range {
            ctx.get_y_aesthetic_values(Aesthetic::Ymin)?
        } else {
            // Y baseline is at 0, which is already normalized by apply_scales
            // Get the normalized y=0 position
            let y_zero_norm = if let Some(y_scale) = ctx.scales.y.as_deref() {
                y_scale.map_value(0.0).unwrap_or(0.0)
            } else {
                0.0
            };
            // Create iterator that repeats y_zero for each bar
            let n = ctx.data().len();
            AestheticValues::Owned(vec![y_zero_norm; n])
        };
        
        let ymax_vals = if has_y_range {
            ctx.get_y_aesthetic_values(Aesthetic::Ymax)?
        } else {
            ctx.get_y_aesthetic_values(Aesthetic::Y)?
        };
        
        // Get styling aesthetics
        let fills = ctx.get_fill_color_values()?;
        let colors = ctx.get_color_values()?;
        let alphas = ctx.get_unscaled_aesthetic_values(Aesthetic::Alpha)?;
        
        // Render each bar
        for (((((xmin, xmax), ymin), ymax), fill), (color, alpha)) in 
            xmin_vals.zip(xmax_vals).zip(ymin_vals).zip(ymax_vals)
                .zip(fills).zip(colors.zip(alphas))
        {
            // Map normalized [0,1] coordinates to device coordinates
            let x_left = ctx.map_x(xmin);
            let x_right = ctx.map_x(xmax);
            let y_top = ctx.map_y(ymax);
            let y_bottom = ctx.map_y(ymin);
            
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
