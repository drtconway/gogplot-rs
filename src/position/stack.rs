// Stack position adjustment for bars

use super::PositionAdjust;
use crate::aesthetics::{AesMap, AesValue, Aesthetic, AestheticDomain};
use crate::data::{DataSource, PrimitiveValue};
use crate::error::{DataType, PlotError};
use crate::utils::dataframe::{DataFrame, FloatVec};
use crate::utils::grouping::{create_composite_keys, get_grouping_columns};
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
    ) -> Result<Option<(Box<dyn DataSource>, AesMap, Option<crate::plot::ScaleSet>)>, PlotError>
    {
        let x_values = mapping
            .get_vector_iter(&Aesthetic::X(AestheticDomain::Discrete), data.as_ref())
            .ok_or_else(|| PlotError::MissingAesthetic {
                aesthetic: Aesthetic::X(AestheticDomain::Discrete),
            })?;

        let y_values = mapping.get_vector_iter(&Aesthetic::Y(()), data)
        // Get x and y column names
        let x_col_name = match mapping.get(&Aesthetic::X) {
            Some(AesValue::Column { name, .. }) => name.clone(),
            _ => return Ok(None),
        };

        let y_col_name = match mapping.get(&Aesthetic::Y) {
            Some(AesValue::Column { name, .. }) => name.clone(),
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
            float_iter.map(|v| PrimitiveValue::Float(v)).collect()
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

            let new_col: Box<dyn crate::data::GenericVector> =
                if let Some(int_iter) = col.iter_int() {
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

/// Apply stack position adjustment to normalized data.
///
/// This function is designed to work with data that has already been scaled to [0, 1] space.
/// It stacks groups vertically by offsetting all y-like aesthetics (ymin, y, ymax) by a cumulative amount.
///
/// For each group at an x position:
/// 1. The height is determined by max(ymin, y, ymax) - this is the cumulative contribution
/// 2. All y-like aesthetics are offset by the cumulative stack height
/// 3. This preserves gaps if ymin > 0, and maintains the relationship between ymin/y/ymax
///
/// # Arguments
/// * `data` - DataSource with normalized y positions (already scaled to [0,1])
/// * `mapping` - Aesthetic mapping including at least one of Y/Ymin/Ymax and Group aesthetic
/// * `reverse` - If true, stack from top to bottom instead of bottom to top
///
/// # Returns
/// * `Ok(None)` if no grouping aesthetic is present (no stacking needed)
/// * `Ok(Some((datasource, mapping)))` with y-like aesthetics offset by stack position
/// * `Err` if required columns are missing or invalid
pub fn apply_stack_normalized(
    data: Box<dyn DataSource>,
    mapping: &AesMap,
    reverse: bool,
) -> Result<Option<(Box<dyn DataSource>, AesMap)>, PlotError> {
    use crate::aesthetics::Aesthetic;

    // Get x column name
    let x_col_name = match mapping.get(&Aesthetic::X) {
        Some(AesValue::Column { name, .. }) => name.clone(),
        Some(AesValue::Constant { .. }) => {
            // Can't stack constants
            return Ok(None);
        }
        None => return Ok(None),
    };

    // Check which y-like aesthetics are present
    let y_col_name = match mapping.get(&Aesthetic::Y) {
        Some(AesValue::Column { name, .. }) => Some(name.clone()),
        _ => None,
    };

    let ymin_col_name = match mapping.get(&Aesthetic::Ymin) {
        Some(AesValue::Column { name, .. }) => Some(name.clone()),
        _ => None,
    };

    let ymax_col_name = match mapping.get(&Aesthetic::Ymax) {
        Some(AesValue::Column { name, .. }) => Some(name.clone()),
        _ => None,
    };

    // Need at least one y-like aesthetic
    if y_col_name.is_none() && ymin_col_name.is_none() && ymax_col_name.is_none() {
        return Ok(None);
    }

    // Check for group aesthetic
    let group_col_name = match mapping.get(&Aesthetic::Group) {
        Some(AesValue::Column { name, .. }) => Some(name.clone()),
        _ => None,
    };

    // If no grouping, no stacking needed
    if group_col_name.is_none() {
        return Ok(None);
    }

    let group_col_name = group_col_name.unwrap();

    // Get x column (already normalized to [0,1])
    let x_col = data
        .get(&x_col_name)
        .ok_or_else(|| PlotError::missing_column(&x_col_name))?;

    // Get x values as f64 (should already be normalized floats)
    let x_values: Vec<f64> = if let Some(float_iter) = x_col.iter_float() {
        float_iter.collect()
    } else {
        return Err(PlotError::InvalidAestheticType {
            aesthetic: Aesthetic::X,
            expected: DataType::Custom("float (normalized)".to_string()),
            actual: DataType::Custom("not float".to_string()),
        });
    };

    let n_rows = x_values.len();

    // Get y-like values
    let y_values: Option<Vec<f64>> = if let Some(ref name) = y_col_name {
        let col = data
            .get(name)
            .ok_or_else(|| PlotError::missing_column(name))?;
        Some(
            col.iter_float()
                .ok_or_else(|| PlotError::InvalidAestheticType {
                    aesthetic: Aesthetic::Y,
                    expected: DataType::Custom("float (normalized)".to_string()),
                    actual: DataType::Custom("not float".to_string()),
                })?
                .collect(),
        )
    } else {
        None
    };

    let ymin_values: Option<Vec<f64>> = if let Some(ref name) = ymin_col_name {
        let col = data
            .get(name)
            .ok_or_else(|| PlotError::missing_column(name))?;
        Some(
            col.iter_float()
                .ok_or_else(|| PlotError::InvalidAestheticType {
                    aesthetic: Aesthetic::Ymin,
                    expected: DataType::Custom("float (normalized)".to_string()),
                    actual: DataType::Custom("not float".to_string()),
                })?
                .collect(),
        )
    } else {
        None
    };

    let ymax_values: Option<Vec<f64>> = if let Some(ref name) = ymax_col_name {
        let col = data
            .get(name)
            .ok_or_else(|| PlotError::missing_column(name))?;
        Some(
            col.iter_float()
                .ok_or_else(|| PlotError::InvalidAestheticType {
                    aesthetic: Aesthetic::Ymax,
                    expected: DataType::Custom("float (normalized)".to_string()),
                    actual: DataType::Custom("not float".to_string()),
                })?
                .collect(),
        )
    } else {
        None
    };

    // Get group column
    let group_col = data
        .get(&group_col_name)
        .ok_or_else(|| PlotError::missing_column(&group_col_name))?;

    // Get group keys as strings
    let group_keys: Vec<String> = if let Some(str_iter) = group_col.iter_str() {
        str_iter.map(|s| s.to_string()).collect()
    } else if let Some(int_iter) = group_col.iter_int() {
        int_iter.map(|i| i.to_string()).collect()
    } else if let Some(float_iter) = group_col.iter_float() {
        float_iter.map(|f| f.to_string()).collect()
    } else {
        return Err(PlotError::missing_column(&group_col_name));
    };

    // For each row, compute the max of y-like values (this is the height contribution)
    let mut heights = Vec::with_capacity(n_rows);
    for i in 0..n_rows {
        let mut max_val: f64 = 0.0;
        if let Some(ref vals) = y_values {
            max_val = max_val.max(vals[i]);
        }
        if let Some(ref vals) = ymin_values {
            max_val = max_val.max(vals[i]);
        }
        if let Some(ref vals) = ymax_values {
            max_val = max_val.max(vals[i]);
        }
        heights.push(max_val);
    }

    // Group by x position: Map x_value -> Vec<(group_key, height, row_index)>
    let mut x_to_groups: HashMap<String, Vec<(String, f64, usize)>> = HashMap::new();

    for (i, &x_val) in x_values.iter().enumerate() {
        let x_key = format!("{:.10}", x_val); // Use fixed precision for grouping
        let group_key = group_keys[i].clone();
        let height = heights[i];

        x_to_groups
            .entry(x_key)
            .or_default()
            .push((group_key, height, i));
    }

    // Sort groups within each x position for deterministic ordering
    for groups in x_to_groups.values_mut() {
        groups.sort_by(|a, b| a.0.cmp(&b.0));
    }

    // Calculate offset for each row
    let mut offsets = vec![0.0; n_rows];

    for groups in x_to_groups.values() {
        if reverse {
            // Stack from top (1.0) downward
            let mut cumsum = 1.0;
            for (_, height, row_idx) in groups {
                cumsum -= height;
                offsets[*row_idx] = cumsum;
            }
        } else {
            // Stack from bottom (0.0) upward
            let mut cumsum = 0.0;
            for (_, height, row_idx) in groups {
                offsets[*row_idx] = cumsum;
                cumsum += height;
            }
        }
    }

    // Create new dataframe with all original columns, replacing y-like aesthetics with offset versions
    let mut new_df = DataFrame::new();

    // Copy all original columns, but replace y-like aesthetics
    use crate::utils::dataframe::{BoolVec, IntVec, StrVec};
    for col_name in data.column_names() {
        let col = data.get(&col_name).unwrap();

        // Check if this is a y-like aesthetic we need to offset
        let is_y = y_col_name.as_ref().map_or(false, |n| n == &col_name);
        let is_ymin = ymin_col_name.as_ref().map_or(false, |n| n == &col_name);
        let is_ymax = ymax_col_name.as_ref().map_or(false, |n| n == &col_name);

        if is_y || is_ymin || is_ymax {
            // Offset the y-like values
            let vals: Vec<f64> = col.iter_float().unwrap().collect();
            let offset_vals: Vec<f64> = vals
                .iter()
                .zip(offsets.iter())
                .map(|(v, offset)| v + offset)
                .collect();
            new_df.add_column(&col_name, Box::new(FloatVec(offset_vals)));
        } else {
            // Copy as-is
            if let Some(iter) = col.iter_int() {
                new_df.add_column(&col_name, Box::new(IntVec(iter.collect())));
            } else if let Some(iter) = col.iter_float() {
                new_df.add_column(&col_name, Box::new(FloatVec(iter.collect())));
            } else if let Some(iter) = col.iter_str() {
                new_df.add_column(
                    &col_name,
                    Box::new(StrVec(iter.map(|s| s.to_string()).collect())),
                );
            } else if let Some(iter) = col.iter_bool() {
                new_df.add_column(&col_name, Box::new(BoolVec(iter.collect())));
            }
        }
    }

    // Mapping stays the same - we just offset existing columns
    Ok(Some((Box::new(new_df), mapping.clone())))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::dataframe::{DataFrame, FloatVec, StrVec};

    fn create_mapping_with_group() -> AesMap {
        let mut mapping = AesMap::new();
        mapping.set(Aesthetic::X, AesValue::column("x"));
        mapping.set(Aesthetic::Y, AesValue::column("y"));
        mapping.set(Aesthetic::Group, AesValue::column("group"));
        mapping
    }

    #[test]
    fn test_stack_normalized_basic() {
        // Test with pre-normalized data (already scaled to [0,1])
        let mut df = DataFrame::new();
        // Two groups at x=0.25 with y values 0.3 and 0.4
        df.add_column("x", Box::new(FloatVec(vec![0.25, 0.25])));
        df.add_column("y", Box::new(FloatVec(vec![0.3, 0.4])));
        df.add_column(
            "group",
            Box::new(StrVec(vec!["A".to_string(), "B".to_string()])),
        );

        let mapping = create_mapping_with_group();
        let result = apply_stack_normalized(Box::new(df), &mapping, false).unwrap();

        assert!(result.is_some());
        let (new_data, _) = result.unwrap();

        // y values should be offset by cumulative stack height
        let y_vals: Vec<f64> = new_data.get("y").unwrap().iter_float().unwrap().collect();

        // Group A at bottom: offset=0, so y stays at 0.3
        assert!((y_vals[0] - 0.3).abs() < 1e-10);

        // Group B stacked on top: offset=0.3 (A's height), so y becomes 0.3 + 0.4 = 0.7
        assert!((y_vals[1] - 0.7).abs() < 1e-10);
    }

    #[test]
    fn test_stack_normalized_three_groups() {
        let mut df = DataFrame::new();
        // Three groups at x=0.5
        df.add_column("x", Box::new(FloatVec(vec![0.5, 0.5, 0.5])));
        df.add_column("y", Box::new(FloatVec(vec![0.2, 0.3, 0.1])));
        df.add_column(
            "group",
            Box::new(StrVec(vec![
                "A".to_string(),
                "B".to_string(),
                "C".to_string(),
            ])),
        );

        let mapping = create_mapping_with_group();
        let result = apply_stack_normalized(Box::new(df), &mapping, false).unwrap();

        assert!(result.is_some());
        let (new_data, _) = result.unwrap();

        let y_vals: Vec<f64> = new_data.get("y").unwrap().iter_float().unwrap().collect();

        // Group A: offset=0, y=0.2
        assert!((y_vals[0] - 0.2).abs() < 1e-10);

        // Group B: offset=0.2 (A's height), y=0.2+0.3=0.5
        assert!((y_vals[1] - 0.5).abs() < 1e-10);

        // Group C: offset=0.5 (A+B's height), y=0.5+0.1=0.6
        assert!((y_vals[2] - 0.6).abs() < 1e-10);
    }

    #[test]
    fn test_stack_normalized_reverse() {
        let mut df = DataFrame::new();
        // Two groups with reverse stacking (from top down)
        df.add_column("x", Box::new(FloatVec(vec![0.5, 0.5])));
        df.add_column("y", Box::new(FloatVec(vec![0.3, 0.4])));
        df.add_column(
            "group",
            Box::new(StrVec(vec!["A".to_string(), "B".to_string()])),
        );

        let mapping = create_mapping_with_group();
        let result = apply_stack_normalized(Box::new(df), &mapping, true).unwrap();

        assert!(result.is_some());
        let (new_data, _) = result.unwrap();

        let y_vals: Vec<f64> = new_data.get("y").unwrap().iter_float().unwrap().collect();

        // With reverse, Group A at top: offset=1.0-0.3=0.7, y=0.7+0.3=1.0
        assert!((y_vals[0] - 1.0).abs() < 1e-10);

        // Group B below it: offset=0.7-0.4=0.3, y=0.3+0.4=0.7
        assert!((y_vals[1] - 0.7).abs() < 1e-10);
    }

    #[test]
    fn test_stack_normalized_no_groups() {
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(FloatVec(vec![0.5, 0.75])));
        df.add_column("y", Box::new(FloatVec(vec![0.3, 0.4])));

        let mut mapping = AesMap::new();
        mapping.set(Aesthetic::X, AesValue::column("x"));
        mapping.set(Aesthetic::Y, AesValue::column("y"));
        // No group aesthetic

        let result = apply_stack_normalized(Box::new(df), &mapping, false).unwrap();

        // Should return None since there's no grouping
        assert!(result.is_none());
    }

    #[test]
    fn test_stack_normalized_multiple_x_positions() {
        let mut df = DataFrame::new();
        // Two groups at two different x positions
        df.add_column(
            "x",
            Box::new(FloatVec(vec![
                0.2, 0.2, // Position 1
                0.8, 0.8, // Position 2
            ])),
        );
        df.add_column("y", Box::new(FloatVec(vec![0.3, 0.2, 0.4, 0.1])));
        df.add_column(
            "group",
            Box::new(StrVec(vec![
                "A".to_string(),
                "B".to_string(),
                "A".to_string(),
                "B".to_string(),
            ])),
        );

        let mapping = create_mapping_with_group();
        let result = apply_stack_normalized(Box::new(df), &mapping, false).unwrap();

        let (new_data, _) = result.unwrap();

        let y_vals: Vec<f64> = new_data.get("y").unwrap().iter_float().unwrap().collect();

        // At x=0.2: Group A offset=0 y=0.3, Group B offset=0.3 y=0.3+0.2=0.5
        assert!((y_vals[0] - 0.3).abs() < 1e-10);
        assert!((y_vals[1] - 0.5).abs() < 1e-10);

        // At x=0.8: Group A offset=0 y=0.4, Group B offset=0.4 y=0.4+0.1=0.5
        assert!((y_vals[2] - 0.4).abs() < 1e-10);
        assert!((y_vals[3] - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_stack_normalized_preserves_other_columns() {
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(FloatVec(vec![0.5, 0.5])));
        df.add_column("y", Box::new(FloatVec(vec![0.3, 0.4])));
        df.add_column(
            "group",
            Box::new(StrVec(vec!["A".to_string(), "B".to_string()])),
        );
        df.add_column(
            "color",
            Box::new(StrVec(vec!["red".to_string(), "blue".to_string()])),
        );

        let mapping = create_mapping_with_group();
        let result = apply_stack_normalized(Box::new(df), &mapping, false).unwrap();

        let (new_data, _) = result.unwrap();

        // All original columns should be preserved
        assert!(new_data.get("x").is_some());
        assert!(new_data.get("y").is_some());
        assert!(new_data.get("group").is_some());
        assert!(new_data.get("color").is_some());

        // y column should be modified (offset), color should be unchanged
        let color_vals: Vec<String> = new_data
            .get("color")
            .unwrap()
            .iter_str()
            .unwrap()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(color_vals, vec!["red".to_string(), "blue".to_string()]);
    }

    #[test]
    fn test_stack_normalized_group_ordering_stable() {
        let mut df = DataFrame::new();
        // Groups in non-alphabetical order
        df.add_column("x", Box::new(FloatVec(vec![0.5, 0.5, 0.5])));
        df.add_column("y", Box::new(FloatVec(vec![0.2, 0.3, 0.1])));
        df.add_column(
            "group",
            Box::new(StrVec(vec![
                "C".to_string(),
                "A".to_string(),
                "B".to_string(),
            ])),
        );

        let mapping = create_mapping_with_group();
        let result = apply_stack_normalized(Box::new(df), &mapping, false).unwrap();

        let (new_data, _) = result.unwrap();

        let y_vals: Vec<f64> = new_data.get("y").unwrap().iter_float().unwrap().collect();
        let group_vals: Vec<String> = new_data
            .get("group")
            .unwrap()
            .iter_str()
            .unwrap()
            .map(|s| s.to_string())
            .collect();

        // Build a map of group -> y
        let mut group_positions = HashMap::new();
        for i in 0..group_vals.len() {
            group_positions.insert(group_vals[i].clone(), y_vals[i]);
        }

        // Groups are sorted alphabetically, so A should be bottom, then B, then C
        let a_y = group_positions["A"];
        let b_y = group_positions["B"];
        let c_y = group_positions["C"];

        // A at bottom: offset=0, y=0.3
        assert!((a_y - 0.3).abs() < 1e-10);

        // B stacked on A: offset=0.3, y=0.3+0.1=0.4
        assert!((b_y - 0.4).abs() < 1e-10);

        // C stacked on B: offset=0.4, y=0.4+0.2=0.6
        assert!((c_y - 0.6).abs() < 1e-10);
    }
}
