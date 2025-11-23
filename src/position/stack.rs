// Stack position adjustment for bars

use super::PositionAdjust;
use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::data::{DataSource, PrimitiveValue};
use crate::error::{DataType, PlotError};
use crate::utils::dataframe::{DataFrame, FloatVec};
use crate::utils::grouping::{get_grouping_columns, create_composite_keys};
use std::collections::HashMap;

/// Stack position adjustment
///
/// Stacks bars on top of each other at the same x position.
/// Requires grouping aesthetics (fill, color, etc.) to determine which bars to stack.
pub struct Stack;

impl PositionAdjust for Stack {
    fn apply(
        &self,
        data: Box<dyn DataSource>,
        mapping: &AesMap,
        _scales: &crate::plot::ScaleSet,
    ) -> Result<Option<(Box<dyn DataSource>, AesMap, Option<crate::plot::ScaleSet>)>, PlotError> {
        // Get x and y column names
        let x_col_name = match mapping.get(&Aesthetic::X) {
            Some(AesValue::Column(name)) => name.clone(),
            _ => return Ok(None),
        };

        let y_col_name = match mapping.get(&Aesthetic::Y) {
            Some(AesValue::Column(name)) => name.clone(),
            _ => return Ok(None),
        };

        // Get grouping columns using utility
        let group_aesthetics = get_grouping_columns(mapping);

        // If no grouping, no stacking needed
        if group_aesthetics.is_empty() {
            return Ok(None);
        }

        // Get columns
        let x_col = data
            .get(x_col_name.as_str())
            .ok_or_else(|| PlotError::missing_column(&x_col_name))?;
        let y_col = data
            .get(y_col_name.as_str())
            .ok_or_else(|| PlotError::missing_column(&y_col_name))?;

        // Get y values as floats
        let y_values: Vec<f64> = if let Some(float_iter) = y_col.iter_float() {
            float_iter.collect()
        } else if let Some(int_iter) = y_col.iter_int() {
            int_iter.map(|v| v as f64).collect()
        } else {
            return Err(PlotError::InvalidAestheticType {
                aesthetic: Aesthetic::Y,
                expected: DataType::Numeric,
                actual: DataType::Custom("string or other".to_string()),
            });
        };

        // Create composite keys using utility
        let composite_keys = create_composite_keys(data.as_ref(), &group_aesthetics);
        let n_rows = y_values.len();

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

        // Group by x value and composite key, computing cumulative sums
        // For each x position, we need to stack bars
        let mut stacked_data: HashMap<String, Vec<(String, f64, f64)>> = HashMap::new();
        
        for (i, x_val) in x_values.iter().enumerate() {
            let x_key = format!("{:?}", x_val);
            let group_key = &composite_keys[i];
            let y_val = y_values[i];
            
            stacked_data
                .entry(x_key)
                .or_default()
                .push((group_key.clone(), y_val, 0.0));
        }

        // Compute stack positions (y0, y1) for each group at each x
        let mut y_bottom: HashMap<(String, String), f64> = HashMap::new(); // (x_key, group) -> y_bottom
        let mut y_top: HashMap<(String, String), f64> = HashMap::new(); // (x_key, group) -> y_top
        
        for (x_key, groups) in &mut stacked_data {
            let mut cumsum = 0.0;
            for (group_key, y_val, _) in groups.iter_mut() {
                let key = (x_key.clone(), group_key.clone());
                y_bottom.insert(key.clone(), cumsum);
                cumsum += *y_val;
                y_top.insert(key, cumsum);
            }
        }

        // Create new data with ymin and ymax columns
        let mut new_df = DataFrame::new();
        
        // Copy all original columns by reconstructing them
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

        // Add ymin and ymax columns
        let mut ymin_vals = Vec::with_capacity(n_rows);
        let mut ymax_vals = Vec::with_capacity(n_rows);
        
        for (i, x_val) in x_values.iter().enumerate() {
            let x_key = format!("{:?}", x_val);
            let group_key = &composite_keys[i];
            let key = (x_key, group_key.clone());
            
            let bottom = *y_bottom.get(&key).unwrap_or(&0.0);
            let top = *y_top.get(&key).unwrap_or(&y_values[i]);
            
            ymin_vals.push(bottom);
            ymax_vals.push(top);
        }

        new_df.add_column("ymin", Box::new(FloatVec(ymin_vals)));
        new_df.add_column("ymax", Box::new(FloatVec(ymax_vals)));

        // Update mapping to use ymin and ymax
        let mut new_mapping = mapping.clone();
        new_mapping.set(Aesthetic::Ymin, AesValue::column("ymin"));
        new_mapping.set(Aesthetic::Ymax, AesValue::column("ymax"));

        // Stack doesn't transform scales, returns None
        Ok(Some((Box::new(new_df), new_mapping, None)))
    }
}
