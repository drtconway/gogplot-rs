// Dodge position adjustment for bars

use super::PositionAdjust;
use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::data::DataSource;
use crate::error::{DataType, PlotError};
use crate::scale::ContinuousScale;
use crate::utils::dataframe::{DataFrame, FloatVec};
use crate::utils::grouping::{get_grouping_columns, create_composite_keys};
use std::collections::BTreeMap;

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
            padding: 0.1,  // 10% padding between bars within a cluster
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
            Some(AesValue::Column { name, .. }) => name.clone(),
            _ => return Ok(None),
        };

        // Get grouping columns using utility
        let group_aesthetics = get_grouping_columns(mapping);

        // If no grouping, no dodging needed
        if group_aesthetics.is_empty() {
            return Ok(None);
        }

        // Get x column
        let x_col = data
            .get(x_col_name.as_str())
            .ok_or_else(|| PlotError::missing_column(&x_col_name))?;

        // Get x values - these are the ORIGINAL data values (before scaling)
        let x_values: Vec<String> = if let Some(float_iter) = x_col.iter_float() {
            float_iter.map(|v| v.to_string()).collect()
        } else if let Some(int_iter) = x_col.iter_int() {
            int_iter.map(|v| v.to_string()).collect()
        } else if let Some(str_iter) = x_col.iter_str() {
            str_iter.map(|s| s.to_string()).collect()
        } else {
            return Err(PlotError::InvalidAestheticType {
                aesthetic: Aesthetic::X,
                expected: DataType::Custom("numeric or string".to_string()),
                actual: DataType::Custom("unknown".to_string()),
            });
        };

        let n_rows = x_values.len();

        // Create composite keys for groups using utility
        let composite_keys = create_composite_keys(data.as_ref(), &group_aesthetics);

        // Get the scale to access mapping information
        let x_scale = scales.x.as_ref()
            .ok_or_else(|| PlotError::InvalidColumnType {
                column: x_col_name.clone(),
                expected: DataType::Custom("X scale required for dodge".to_string()),
            })?;
        
        // Try to get categorical info from the scale
        let (category_width, scale_padding) = if let Some(cat_info) = x_scale.categorical_info() {
            // Use the categorical scale's actual width and padding
            (cat_info.category_width, cat_info.padding)
        } else {
            // Fall back to auto-detection for continuous scales
            let detected = self.auto_detect_width_continuous(&x_values, x_scale);
            (detected, 0.1)
        };

        // Calculate the effective width after scale padding (this is the space available within each category)
        let effective_width = category_width * (1.0 - 2.0 * scale_padding);

        // Group data by x position and collect unique groups per position
        // Map: x_value -> Vec<group_key>
        let mut x_to_groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
        
        for (i, x_val) in x_values.iter().enumerate() {
            let group_key = composite_keys[i].clone();
            
            let groups = x_to_groups.entry(x_val.clone()).or_default();
            if !groups.contains(&group_key) {
                groups.push(group_key);
            }
        }

        // Sort groups within each x position for deterministic ordering
        for groups in x_to_groups.values_mut() {
            groups.sort();
        }

        // Calculate xmin, x, and xmax for each row
        let mut xmin_vals = Vec::with_capacity(n_rows);
        let mut x_vals = Vec::with_capacity(n_rows);
        let mut xmax_vals = Vec::with_capacity(n_rows);
        
        for (i, x_cat) in x_values.iter().enumerate() {
            // X values are already in normalized space after scale application
            // Parse them as floats (they're the category center positions)
            let x_center = x_cat.parse::<f64>().unwrap_or(0.5);
            
            // Get groups at this x position and find this row's group index
            let groups = x_to_groups.get(x_cat).unwrap();
            let n_groups = groups.len();
            let group_key = &composite_keys[i];
            let group_index = groups.iter().position(|g| g == group_key).unwrap();
            
            // Calculate the category's xmin/xmax range using the categorical scale info
            // The category is centered at x_center with half-width on each side
            let half_width = effective_width / 2.0;
            let category_xmin = x_center - half_width;
            let category_xmax = x_center + half_width;
            let category_range = category_xmax - category_xmin;
            
            // Model the category as [0, 1] interval and divide into n_groups equal parts
            // Group i gets the interval [i/n, (i+1)/n]
            let normalized_start = group_index as f64 / n_groups as f64;
            let normalized_end = (group_index + 1) as f64 / n_groups as f64;
            
            // Map from [0, 1] to [category_xmin, category_xmax]
            let xmin = category_xmin + normalized_start * category_range;
            let xmax = category_xmin + normalized_end * category_range;
            let group_center = (xmin + xmax) / 2.0;
            
            x_vals.push(group_center);
            xmin_vals.push(xmin);
            xmax_vals.push(xmax);
        }

        // Create sorted index: sort by x center position, then by group (for deterministic order)
        let mut indices: Vec<usize> = (0..n_rows).collect();
        indices.sort_by(|&a, &b| {
            // First compare by x center positions
            let x_cmp = x_vals[a].partial_cmp(&x_vals[b]).unwrap_or(std::cmp::Ordering::Equal);
            if x_cmp != std::cmp::Ordering::Equal {
                return x_cmp;
            }
            // Then by group key for stable ordering
            composite_keys[a].cmp(&composite_keys[b])
        });

        // Create new dataframe with all original columns plus xmin/xmax, in sorted order
        let mut new_df = DataFrame::new();
        
        
        // Copy all original columns in sorted order
        use crate::utils::dataframe::{IntVec, StrVec};
        for col_name in data.column_names() {
            let col = data.get(col_name.as_str()).unwrap();
            
            let new_col: Box<dyn crate::data::GenericVector> = if let Some(int_iter) = col.iter_int() {
                let vals: Vec<i64> = int_iter.collect();
                Box::new(IntVec(indices.iter().map(|&i| vals[i]).collect()))
            } else if let Some(float_iter) = col.iter_float() {
                let vals: Vec<f64> = float_iter.collect();
                if col_name == "y" {
                }
                let sorted_vals: Vec<f64> = indices.iter().map(|&i| vals[i]).collect();
                Box::new(FloatVec(sorted_vals))
            } else if let Some(str_iter) = col.iter_str() {
                let vals: Vec<String> = str_iter.map(|s| s.to_string()).collect();
                Box::new(StrVec(indices.iter().map(|&i| vals[i].clone()).collect()))
            } else {
                continue;
            };
            new_df.add_column(&col_name, new_col);
        }

        // Add xmin and xmax columns in sorted order
        new_df.add_column("xmin", Box::new(FloatVec(indices.iter().map(|&i| xmin_vals[i]).collect())));
        new_df.add_column("xmax", Box::new(FloatVec(indices.iter().map(|&i| xmax_vals[i]).collect())));

        // Update mapping to use xmin and xmax
        let mut new_mapping = mapping.clone();
        new_mapping.set(Aesthetic::Xmin, AesValue::column("xmin"));
        new_mapping.set(Aesthetic::Xmax, AesValue::column("xmax"));

        // xmin/xmax are now in data space, so no need for computed_scales
        // The normal scale pipeline will handle the transformation to normalized space
        Ok(Some((Box::new(new_df), new_mapping, None)))
    }
}

impl Dodge {
    /// Auto-detect bar width for continuous scales from spacing between x values
    fn auto_detect_width_continuous(&self, x_values: &[String], x_scale: &Box<dyn ContinuousScale>) -> f64 {
        // Get unique x values mapped through scale to visual space
        let mut unique_x: Vec<f64> = x_values
            .iter()
            .filter_map(|v| {
                // Try parsing as number for continuous scales
                v.parse::<f64>().ok()
                    .and_then(|f| x_scale.map_value(f))
                    .or_else(|| x_scale.map_category(v, Aesthetic::X))
            })
            .collect();
        unique_x.sort_by(|a, b| a.partial_cmp(b).unwrap());
        unique_x.dedup();

        if unique_x.len() < 2 {
            // Only one x value, use small default width (5% of normalized space)
            return 0.05;
        }

        // Find minimum distance
        let mut min_dist = f64::MAX;
        for i in 1..unique_x.len() {
            let dist = unique_x[i] - unique_x[i - 1];
            if dist > 0.0 && dist < min_dist {
                min_dist = dist;
            }
        }

        // Use 90% of spacing to leave 10% gap between bar clusters at different x positions
        min_dist * 0.9
    }
}

/// Apply dodge position adjustment to normalized data.
/// 
/// This function is designed to work with data that has already been scaled to [0, 1] space.
/// It takes x positions and groups, then creates xmin/xmax columns with non-overlapping bars.
///
/// # Arguments
/// * `data` - DataSource with normalized x positions (already scaled to [0,1])
/// * `mapping` - Aesthetic mapping including X and Group aesthetics
/// * `width` - Total width available for all groups at each x position (in normalized space)
/// * `padding` - Proportion of space to leave as padding between bars (0.0 = no padding, 0.5 = 50% padding)
///
/// # Returns
/// * `Ok(None)` if no grouping aesthetic is present (no dodge needed)
/// * `Ok(Some((datasource, mapping)))` with xmin/xmax columns added and mapping updated
/// * `Err` if required columns are missing or invalid
pub fn apply_dodge_normalized(
    data: Box<dyn DataSource>,
    mapping: &AesMap,
    width: f64,
    padding: f64,
) -> Result<Option<(Box<dyn DataSource>, AesMap)>, PlotError> {
    use crate::aesthetics::Aesthetic;

    // Get x column name from mapping
    let x_col_name = match mapping.get(&Aesthetic::X) {
        Some(AesValue::Column { name, .. }) => name.clone(),
        Some(AesValue::Constant { .. }) => {
            // Can't dodge constants
            return Ok(None);
        }
        None => return Ok(None),
    };

    // Check for group aesthetic
    let group_col_name = match mapping.get(&Aesthetic::Group) {
        Some(AesValue::Column { name, .. }) => Some(name.clone()),
        _ => None,
    };

    // If no grouping, no dodging needed
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

    // Group data by x position: Map x_value -> Vec<group_key>
    let mut x_to_groups: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for (i, &x_val) in x_values.iter().enumerate() {
        let x_key = format!("{:.10}", x_val); // Use fixed precision for grouping
        let group_key = group_keys[i].clone();

        let groups = x_to_groups.entry(x_key).or_default();
        if !groups.contains(&group_key) {
            groups.push(group_key);
        }
    }

    // Sort groups within each x position for deterministic ordering
    for groups in x_to_groups.values_mut() {
        groups.sort();
    }

    // Calculate xmin and xmax for each row (in normalized [0,1] space)
    let mut xmin_vals = Vec::with_capacity(n_rows);
    let mut xmax_vals = Vec::with_capacity(n_rows);

    for (i, &x_center) in x_values.iter().enumerate() {
        let x_key = format!("{:.10}", x_center);
        let group_key = &group_keys[i];

        let groups = x_to_groups.get(&x_key).unwrap();
        let n_groups = groups.len() as f64;
        let group_index = groups.iter().position(|g| g == group_key).unwrap() as f64;

        // Width of each individual bar (in normalized space)
        let individual_width = width / n_groups;
        let padded_width = individual_width * (1.0 - padding);

        // Calculate position in normalized space
        let left_edge = x_center - width / 2.0;
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
    use crate::utils::dataframe::{BoolVec, IntVec, StrVec};
    for col_name in data.column_names() {
        let col = data.get(&col_name).unwrap();

        if let Some(iter) = col.iter_int() {
            new_df.add_column(&col_name, Box::new(IntVec(iter.collect())));
        } else if let Some(iter) = col.iter_float() {
            new_df.add_column(&col_name, Box::new(FloatVec(iter.collect())));
        } else if let Some(iter) = col.iter_str() {
            new_df.add_column(&col_name, Box::new(StrVec(iter.map(|s| s.to_string()).collect())));
        } else if let Some(iter) = col.iter_bool() {
            new_df.add_column(&col_name, Box::new(BoolVec(iter.collect())));
        }
    }

    // Add xmin and xmax columns
    new_df.add_column("xmin", Box::new(FloatVec(xmin_vals)));
    new_df.add_column("xmax", Box::new(FloatVec(xmax_vals)));

    // Update mapping to use xmin and xmax
    let mut new_mapping = mapping.clone();
    new_mapping.set(Aesthetic::Xmin, AesValue::column("xmin"));
    new_mapping.set(Aesthetic::Xmax, AesValue::column("xmax"));

    Ok(Some((Box::new(new_df), new_mapping)))
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
    fn test_dodge_normalized_basic() {
        // Test with pre-normalized data (already scaled to [0,1])
        let mut df = DataFrame::new();
        // Two groups at x=0.25 and x=0.75
        df.add_column("x", Box::new(FloatVec(vec![0.25, 0.25, 0.75, 0.75])));
        df.add_column("group", Box::new(StrVec(vec![
            "A".to_string(), "B".to_string(),
            "A".to_string(), "B".to_string(),
        ])));
        df.add_column("y", Box::new(FloatVec(vec![10.0, 20.0, 15.0, 25.0])));

        let mapping = create_mapping_with_group();
        let result = apply_dodge_normalized(Box::new(df), &mapping, 0.1, 0.1).unwrap();

        assert!(result.is_some());
        let (new_data, new_mapping) = result.unwrap();

        // Check that xmin and xmax columns were created
        assert!(new_data.get("xmin").is_some());
        assert!(new_data.get("xmax").is_some());

        // Check that mapping was updated
        assert!(matches!(new_mapping.get(&Aesthetic::Xmin), Some(AesValue::Column { name, .. }) if name == "xmin"));
        assert!(matches!(new_mapping.get(&Aesthetic::Xmax), Some(AesValue::Column { name, .. }) if name == "xmax"));

        // Get the xmin/xmax values
        let xmin_vals: Vec<f64> = new_data.get("xmin").unwrap().iter_float().unwrap().collect();
        let xmax_vals: Vec<f64> = new_data.get("xmax").unwrap().iter_float().unwrap().collect();

        // For two groups with width=0.1, each bar gets width 0.05 minus padding
        // Group A (index 0) should be on the left, Group B (index 1) on the right
        
        // At x=0.25: Group A left, Group B right
        assert!(xmin_vals[0] < 0.25); // Group A left of center
        assert!(xmax_vals[0] <= 0.25);
        assert!(xmin_vals[1] >= 0.25); // Group B right of center
        assert!(xmax_vals[1] > 0.25);

        // Bars should not overlap
        assert!(xmax_vals[0] <= xmin_vals[1]);
    }

    #[test]
    fn test_dodge_normalized_three_groups() {
        let mut df = DataFrame::new();
        // Three groups at x=0.5
        df.add_column("x", Box::new(FloatVec(vec![0.5, 0.5, 0.5])));
        df.add_column("group", Box::new(StrVec(vec![
            "A".to_string(), "B".to_string(), "C".to_string(),
        ])));
        df.add_column("y", Box::new(FloatVec(vec![10.0, 20.0, 30.0])));

        let mapping = create_mapping_with_group();
        let result = apply_dodge_normalized(Box::new(df), &mapping, 0.3, 0.1).unwrap();

        assert!(result.is_some());
        let (new_data, _) = result.unwrap();

        let xmin_vals: Vec<f64> = new_data.get("xmin").unwrap().iter_float().unwrap().collect();
        let xmax_vals: Vec<f64> = new_data.get("xmax").unwrap().iter_float().unwrap().collect();

        // Three groups should be arranged left to right
        assert!(xmin_vals[0] < xmin_vals[1]);
        assert!(xmin_vals[1] < xmin_vals[2]);
        assert!(xmax_vals[0] < xmax_vals[1]);
        assert!(xmax_vals[1] < xmax_vals[2]);

        // No overlaps
        assert!(xmax_vals[0] <= xmin_vals[1]);
        assert!(xmax_vals[1] <= xmin_vals[2]);

        // All bars should be within the width around x=0.5
        let total_width = 0.3;
        assert!(xmin_vals[0] >= 0.5 - total_width / 2.0);
        assert!(xmax_vals[2] <= 0.5 + total_width / 2.0);
    }

    #[test]
    fn test_dodge_normalized_no_groups() {
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(FloatVec(vec![0.5, 0.75])));
        df.add_column("y", Box::new(FloatVec(vec![10.0, 20.0])));

        let mut mapping = AesMap::new();
        mapping.set(Aesthetic::X, AesValue::column("x"));
        mapping.set(Aesthetic::Y, AesValue::column("y"));
        // No group aesthetic

        let result = apply_dodge_normalized(Box::new(df), &mapping, 0.1, 0.1).unwrap();

        // Should return None since there's no grouping
        assert!(result.is_none());
    }

    #[test]
    fn test_dodge_normalized_padding() {
        let mut df1 = DataFrame::new();
        df1.add_column("x", Box::new(FloatVec(vec![0.5, 0.5])));
        df1.add_column("group", Box::new(StrVec(vec![
            "A".to_string(), "B".to_string(),
        ])));
        df1.add_column("y", Box::new(FloatVec(vec![10.0, 20.0])));

        let df2 = df1.clone();

        let mapping = create_mapping_with_group();
        
        // Test with no padding
        let result1 = apply_dodge_normalized(Box::new(df1), &mapping, 0.2, 0.0).unwrap();
        let (data1, _) = result1.unwrap();
        let xmin1: Vec<f64> = data1.get("xmin").unwrap().iter_float().unwrap().collect();
        let xmax1: Vec<f64> = data1.get("xmax").unwrap().iter_float().unwrap().collect();
        let width1_a = xmax1[0] - xmin1[0];
        
        // Test with 50% padding
        let result2 = apply_dodge_normalized(Box::new(df2), &mapping, 0.2, 0.5).unwrap();
        let (data2, _) = result2.unwrap();
        let xmin2: Vec<f64> = data2.get("xmin").unwrap().iter_float().unwrap().collect();
        let xmax2: Vec<f64> = data2.get("xmax").unwrap().iter_float().unwrap().collect();
        let width2_a = xmax2[0] - xmin2[0];

        // With more padding, bars should be narrower
        assert!(width2_a < width1_a);
        assert!((width2_a / width1_a - 0.5).abs() < 0.01); // Should be half the width
    }

    #[test]
    fn test_dodge_normalized_preserves_other_columns() {
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(FloatVec(vec![0.5, 0.5])));
        df.add_column("group", Box::new(StrVec(vec![
            "A".to_string(), "B".to_string(),
        ])));
        df.add_column("y", Box::new(FloatVec(vec![10.0, 20.0])));
        df.add_column("color", Box::new(StrVec(vec![
            "red".to_string(), "blue".to_string(),
        ])));

        let mapping = create_mapping_with_group();
        let result = apply_dodge_normalized(Box::new(df), &mapping, 0.1, 0.1).unwrap();

        let (new_data, _) = result.unwrap();

        // All original columns should be preserved
        assert!(new_data.get("x").is_some());
        assert!(new_data.get("group").is_some());
        assert!(new_data.get("y").is_some());
        assert!(new_data.get("color").is_some());

        // New columns should be added
        assert!(new_data.get("xmin").is_some());
        assert!(new_data.get("xmax").is_some());

        // Original data should be unchanged
        let y_vals: Vec<f64> = new_data.get("y").unwrap().iter_float().unwrap().collect();
        assert_eq!(y_vals, vec![10.0, 20.0]);
    }

    #[test]
    fn test_dodge_normalized_multiple_x_positions() {
        let mut df = DataFrame::new();
        // Two groups at three different x positions
        df.add_column("x", Box::new(FloatVec(vec![
            0.2, 0.2,  // Position 1
            0.5, 0.5,  // Position 2
            0.8, 0.8,  // Position 3
        ])));
        df.add_column("group", Box::new(StrVec(vec![
            "A".to_string(), "B".to_string(),
            "A".to_string(), "B".to_string(),
            "A".to_string(), "B".to_string(),
        ])));
        df.add_column("y", Box::new(FloatVec(vec![10.0, 20.0, 15.0, 25.0, 12.0, 22.0])));

        let mapping = create_mapping_with_group();
        let result = apply_dodge_normalized(Box::new(df), &mapping, 0.1, 0.1).unwrap();

        let (new_data, _) = result.unwrap();

        let xmin_vals: Vec<f64> = new_data.get("xmin").unwrap().iter_float().unwrap().collect();
        let xmax_vals: Vec<f64> = new_data.get("xmax").unwrap().iter_float().unwrap().collect();

        // At each x position, group A should be left of group B
        assert!(xmax_vals[0] <= xmin_vals[1]); // Position 1
        assert!(xmax_vals[2] <= xmin_vals[3]); // Position 2
        assert!(xmax_vals[4] <= xmin_vals[5]); // Position 3

        // Groups should be clustered around their x centers
        let center1 = (xmin_vals[0] + xmax_vals[1]) / 2.0;
        let center2 = (xmin_vals[2] + xmax_vals[3]) / 2.0;
        let center3 = (xmin_vals[4] + xmax_vals[5]) / 2.0;

        assert!((center1 - 0.2).abs() < 0.05);
        assert!((center2 - 0.5).abs() < 0.05);
        assert!((center3 - 0.8).abs() < 0.05);
    }

    #[test]
    fn test_dodge_normalized_group_ordering_stable() {
        let mut df = DataFrame::new();
        // Same groups in different order
        df.add_column("x", Box::new(FloatVec(vec![0.5, 0.5, 0.5])));
        df.add_column("group", Box::new(StrVec(vec![
            "C".to_string(), "A".to_string(), "B".to_string(),
        ])));
        df.add_column("y", Box::new(FloatVec(vec![30.0, 10.0, 20.0])));

        let mapping = create_mapping_with_group();
        let result = apply_dodge_normalized(Box::new(df), &mapping, 0.3, 0.0).unwrap();

        let (new_data, _) = result.unwrap();

        let xmin_vals: Vec<f64> = new_data.get("xmin").unwrap().iter_float().unwrap().collect();
        let group_vals: Vec<String> = new_data.get("group").unwrap()
            .iter_str().unwrap()
            .map(|s| s.to_string())
            .collect();

        // Groups are sorted alphabetically, so we should have A, B, C in that order
        // But the data rows are still in original order: C, A, B
        // So positions should reflect that:
        // Row 0: C (index 2 in sorted list)
        // Row 1: A (index 0 in sorted list)
        // Row 2: B (index 1 in sorted list)
        
        // Build a map of group -> xmin position
        let mut group_positions = std::collections::HashMap::new();
        for (group, xmin) in group_vals.iter().zip(xmin_vals.iter()) {
            group_positions.insert(group.clone(), *xmin);
        }

        // Check that A < B < C in terms of position
        assert!(group_positions["A"] < group_positions["B"]);
        assert!(group_positions["B"] < group_positions["C"]);
    }
}
