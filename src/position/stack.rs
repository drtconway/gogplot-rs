// Stack position adjustment for bars

use super::Position;
use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::data::{DataSource, DiscreteType};
use crate::error::{DataType, PlotError};
use crate::utils::data::{DiscreteVectorVisitor, Vectorable, visit_d};
use crate::utils::dataframe::{DataFrame, FloatVec};
use std::collections::HashMap;

/// Stack position adjustment
///
/// Stacks bars on top of each other at the same x position.
/// Requires grouping aesthetics (fill, color, etc.) to determine which bars to stack.
pub struct Stack;

impl Position for Stack {
    fn apply(
        &self,
        data: &Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<Option<(Box<dyn DataSource>, AesMap)>, PlotError>
    {
        if !mapping.has_aesthetic(&Aesthetic::Group) {
            // No grouping aesthetic, cannot stack
            return Err(PlotError::MissingAesthetic {
                aesthetic: Aesthetic::Group,
            });
        }

        let mut y_like_data: HashMap<Aesthetic, Vec<f64>> = HashMap::new();
        for aes in mapping.aesthetics() {
            if aes.is_y_like() && aes.is_continuous() {
                let y_like_values = mapping.get_iter_float(aes, data.as_ref()).unwrap();
                let y_like_values = y_like_values.collect::<Vec<f64>>();
                y_like_data.insert(*aes, y_like_values);
            }
        }
        let group_values = mapping
        .get_vector_iter(&Aesthetic::Group, data.as_ref()).unwrap();

        let mut grouped_stacker = GroupStacker::new(y_like_data);

        let max_val = visit_d(group_values, &mut grouped_stacker)?;

        let y_like_data = grouped_stacker.y_like_data;

        // Create new dataframe with all original columns, replacing y-like aesthetics with stacked versions
        let mut new_data: DataFrame = DataFrame::new();
        let mut new_mapping = AesMap::new();
        for (aes, aes_value) in mapping.iter() {
            if let Some(stacked_values) = y_like_data.remove(aes) {
                let name = get_or_invent_column_name(&aes_value, &data, &new_data, &aes);
                new_data.add_column(
                    &name,
                    Box::new(FloatVec::from_vec(stacked_values.clone())),
                );
                new_mapping.set(*aes, AesValue::column(name));
            } else {
                let (new_aes_value, opt_name_and_vector) = aes_value.duplicate(data.as_ref())?;
                if let Some((name, vector)) = opt_name_and_vector {
                    new_data.add_column(&name, vector);
                }
                new_mapping.set(*aes, new_aes_value);
            }
        }
        
        Ok(Some((Box::new(new_data), new_mapping) ))
    }
}

struct GroupStacker {
    y_like_data: HashMap<Aesthetic, Vec<f64>>,
}

impl GroupStacker {
    fn new(y_like_data: HashMap<Aesthetic, Vec<f64>>) -> Self {
        Self { y_like_data }
    }
}

impl DiscreteVectorVisitor for GroupStacker {
    type Output = f64;

    fn visit<T: Vectorable + DiscreteType>(&mut self, group_values: impl Iterator<Item = T>) -> std::result::Result<Self::Output, PlotError> {

        let group_values: Vec<T::Sortable> = group_values.map(|v| v.to_sortable()).collect();

        // Collect maxima per group
        let mut maxima: HashMap<T::Sortable, _> = HashMap::new();
        for (i, group_value) in group_values.enumerate() {
            for vals in self.y_like_data.values_mut() {
                let entry = maxima.entry(group_value).or_insert(f64::NEG_INFINITY);
                if *entry < vals[i] {
                    *entry = vals[i];
                }
            }
        }

        // Get the keys in sorted order
        let mut sorted_keys: Vec<_> = maxima.keys().cloned().collect();
        sorted_keys.sort();

        // Compute the per-group offsets
        let mut cumulative = 0.0;
        for key in &sorted_keys {
            let max_val = maxima.get_mut(key).unwrap();
            let val = *max_val;
            *max_val = cumulative;
            cumulative += val;
        }

        let offsets = maxima;

        // Apply offsets to y-like data
        for (aes, vals) in self.y_like_data.iter_mut() {
            for (i, group_value) in group_values.iter().enumerate() {
                if let Some(offset) = offsets.get(group_value) {
                    vals[i] += offset;
                }
            }
        }

        Ok(cumulative)
    }
}

fn get_or_invent_column_name(
    aes_value: &AesValue,
    original_data: &Box<dyn DataSource>,
    new_data: &DataFrame,
    aes: &Aesthetic,
) -> String {
    if let Some(name) = aes_value.column_name() {
        name
    } else {
        // Invent a new column name
        let aes_name = aes.to_str();
        let mut proposed_name = format!("stacked_{}", aes_name).to_lowercase();
        let mut counter = 1;
        while original_data.get(&proposed_name).is_some() || new_data.get(&proposed_name).is_some() {
            proposed_name = format!("stacked_{}_{}", aes_name, counter).to_lowercase();
            counter += 1;
        }
        proposed_name
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
