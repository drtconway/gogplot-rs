// Dodge position adjustment for bars

use super::PositionAdjust;
use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::data::{DataSource, PrimitiveValue};
use crate::error::{DataType, PlotError};
use crate::scale::ContinuousScale;
use crate::utils::dataframe::{DataFrame, FloatVec};
use std::collections::HashMap;

/// Dodge position adjustment
///
/// Places bars side-by-side at the same x position, making each bar narrower
/// to fit multiple groups without overlapping.
pub struct Dodge {
    /// Total width for all bars at one x position (None = auto-detect)
    pub width: Option<f64>,
    /// Padding between dodged bars as a fraction of bar width (default 0.1)
    pub padding: f64,
}

impl Default for Dodge {
    fn default() -> Self {
        Self {
            width: None,
            padding: 0.1,
        }
    }
}

impl PositionAdjust for Dodge {
    fn apply(
        &self,
        data: Box<dyn DataSource>,
        mapping: &AesMap,
        scales: &crate::plot::ScaleSet,
    ) -> Result<Option<(Box<dyn DataSource>, AesMap, Option<crate::plot::ScaleSet>)>, PlotError> {
        // Get x column name
        let x_col_name = match mapping.get(&Aesthetic::X) {
            Some(AesValue::Column(name)) => name.clone(),
            _ => return Ok(None),
        };

        // Find grouping aesthetics
        let group_aesthetics: Vec<(Aesthetic, String)> = mapping
            .iter()
            .filter(|(aes, _)| aes.is_grouping())
            .filter_map(|(aes, val)| {
                if let AesValue::Column(col_name) = val {
                    Some((*aes, col_name.clone()))
                } else {
                    None
                }
            })
            .collect();

        // If no grouping, no dodging needed
        if group_aesthetics.is_empty() {
            return Ok(None);
        }

        // Get x column
        let x_col = data
            .get(x_col_name.as_str())
            .ok_or_else(|| PlotError::missing_column(&x_col_name))?;

        // Get x values
        let x_values: Vec<PrimitiveValue> = if let Some(float_iter) = x_col.iter_float() {
            float_iter
                .map(|v| PrimitiveValue::Float(v))
                .collect()
        } else if let Some(int_iter) = x_col.iter_int() {
            int_iter.map(|v| PrimitiveValue::Int(v)).collect()
        } else if let Some(str_iter) = x_col.iter_str() {
            str_iter
                .map(|s| PrimitiveValue::Str(s.to_string()))
                .collect()
        } else {
            return Err(PlotError::InvalidAestheticType {
                aesthetic: Aesthetic::X,
                expected: DataType::Custom("numeric or string".to_string()),
                actual: DataType::Custom("unknown".to_string()),
            });
        };

        let n_rows = x_values.len();

        // Create composite keys for each row (to identify groups)
        let mut composite_keys: Vec<String> = Vec::with_capacity(n_rows);
        
        // Collect group column values for indexing
        let mut group_col_values: Vec<Vec<String>> = Vec::new();
        for (_aesthetic, col_name) in &group_aesthetics {
            let col = data.get(col_name.as_str()).unwrap();
            let values = if let Some(str_iter) = col.iter_str() {
                str_iter.map(|s| s.to_string()).collect()
            } else if let Some(int_iter) = col.iter_int() {
                int_iter.map(|v| v.to_string()).collect()
            } else if let Some(float_iter) = col.iter_float() {
                float_iter.map(|v| v.to_string()).collect()
            } else {
                vec![String::new(); n_rows]
            };
            group_col_values.push(values);
        }
        
        for i in 0..n_rows {
            let key_parts: Vec<&str> = group_col_values.iter()
                .map(|col_vals| col_vals[i].as_str())
                .collect();
            composite_keys.push(key_parts.join("__"));
        }

        // Determine bar width
        let bar_width = if let Some(w) = self.width {
            w
        } else {
            // Auto-detect: find minimum distance between x values, use 90% of that
            self.auto_detect_width(&x_values, scales.x.as_ref())
        };

        // Group data by x position and group
        // Map: x_key -> Vec<group_key>
        let mut x_to_groups: HashMap<String, Vec<String>> = HashMap::new();
        
        for (i, x_val) in x_values.iter().enumerate() {
            let x_key = format!("{:?}", x_val);
            let group_key = composite_keys[i].clone();
            
            let groups = x_to_groups.entry(x_key.clone()).or_default();
            if !groups.contains(&group_key) {
                groups.push(group_key);
            }
        }

        // Sort groups within each x position for consistency
        for groups in x_to_groups.values_mut() {
            groups.sort();
        }

        // Calculate xmin and xmax for each row
        let mut xmin_vals = Vec::with_capacity(n_rows);
        let mut xmax_vals = Vec::with_capacity(n_rows);

        // Get the x scale for mapping values
        let x_scale = scales.x.as_ref();
        
        for (i, x_val) in x_values.iter().enumerate() {
            // Map x value through the scale to get normalized position
            let x_center = if let Some(scale) = x_scale {
                match x_val {
                    PrimitiveValue::Float(f) => scale.map_value(*f).unwrap_or(0.0),
                    PrimitiveValue::Int(i) => scale.map_value(*i as f64).unwrap_or(0.0),
                    PrimitiveValue::Str(s) => scale.map_category(s).unwrap_or(0.0),
                }
            } else {
                self.x_to_f64(x_val)
            };
            
            let x_key = format!("{:?}", x_val);
            let group_key = &composite_keys[i];
            
            let groups = x_to_groups.get(&x_key).unwrap();
            let n_groups = groups.len() as f64;
            let group_index = groups.iter().position(|g| g == group_key).unwrap() as f64;
            
            // Width of each individual bar
            let individual_width = bar_width / n_groups;
            let padded_width = individual_width * (1.0 - self.padding);
            
            // Calculate position
            // Start from left edge of the total bar width
            let left_edge = x_center - bar_width / 2.0;
            let bar_left = left_edge + group_index * individual_width;
            let bar_center = bar_left + individual_width / 2.0;
            
            let xmin = bar_center - padded_width / 2.0;
            let xmax = bar_center + padded_width / 2.0;
            
            xmin_vals.push(xmin);
            xmax_vals.push(xmax);
        }

        // Create new dataframe with all original columns plus xmin/xmax
        let mut new_df = DataFrame::new();
        
        // Copy all original columns
        use crate::utils::dataframe::{IntVec, StrVec};
        for col_name in data.column_names() {
            let col = data.get(col_name.as_str()).unwrap();
            
            let new_col: Box<dyn crate::data::GenericVector> = if let Some(int_iter) = col.iter_int() {
                Box::new(IntVec(int_iter.collect()))
            } else if let Some(float_iter) = col.iter_float() {
                Box::new(FloatVec(float_iter.collect()))
            } else if let Some(str_iter) = col.iter_str() {
                Box::new(StrVec(str_iter.map(|s| s.to_string()).collect()))
            } else {
                continue;
            };
            new_df.add_column(&col_name, new_col);
        }

        // Add xmin and xmax columns
        new_df.add_column("xmin", Box::new(FloatVec(xmin_vals)));
        new_df.add_column("xmax", Box::new(FloatVec(xmax_vals)));

        // Update mapping to use xmin and xmax
        let mut new_mapping = mapping.clone();
        new_mapping.set(Aesthetic::Xmin, AesValue::column("xmin"));
        new_mapping.set(Aesthetic::Xmax, AesValue::column("xmax"));

        // TODO: Transform scales instead of outputting xmin/xmax columns
        Ok(Some((Box::new(new_df), new_mapping, None)))
    }
}

impl Dodge {
    /// Convert PrimitiveValue to f64 for position calculations
    fn x_to_f64(&self, val: &PrimitiveValue) -> f64 {
        match val {
            PrimitiveValue::Float(f) => *f,
            PrimitiveValue::Int(i) => *i as f64,
            PrimitiveValue::Str(_) => {
                // For categorical x, we'd need the scale mapping
                // For now, treat as 0 (this should be handled by scale)
                0.0
            }
        }
    }

    /// Auto-detect bar width from spacing between x values
    fn auto_detect_width(&self, x_values: &[PrimitiveValue], x_scale: Option<&Box<dyn ContinuousScale>>) -> f64 {
        // Get unique x values mapped through scale
        let mut unique_x: Vec<f64> = x_values
            .iter()
            .filter_map(|v| {
                if let Some(scale) = x_scale {
                    match v {
                        PrimitiveValue::Float(f) => scale.map_value(*f),
                        PrimitiveValue::Int(i) => scale.map_value(*i as f64),
                        PrimitiveValue::Str(s) => scale.map_category(s),
                    }
                } else {
                    Some(self.x_to_f64(v))
                }
            })
            .collect();
        unique_x.sort_by(|a, b| a.partial_cmp(b).unwrap());
        unique_x.dedup();

        if unique_x.len() < 2 {
            // Only one x value, use default width
            return 0.9;
        }

        // Find minimum distance
        let mut min_dist = f64::MAX;
        for i in 1..unique_x.len() {
            let dist = unique_x[i] - unique_x[i - 1];
            if dist > 0.0 && dist < min_dist {
                min_dist = dist;
            }
        }

        // Use 90% of minimum distance as bar width
        min_dist * 0.9
    }
}
