//! Boxplot statistics
//!
//! Computes the five-number summary (minimum, Q1, median, Q3, maximum)
//! and identifies outliers for creating box-and-whisker plots.

use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::data::DataSource;
use crate::error::{PlotError, Result};
use crate::stat::utils::get_numeric_values;
use crate::stat::StatTransform;
use crate::utils::dataframe::{DataFrame, FloatVec, IntVec, StrVec};
use crate::utils::grouping::{get_grouping_columns, create_composite_keys, group_by_key_sorted, split_composite_key};

/// Boxplot statistics computation
///
/// Computes five-number summary for each group:
/// - `ymin`: Lower whisker (minimum non-outlier value)
/// - `lower`: First quartile (Q1, 25th percentile)
/// - `middle`: Median (Q2, 50th percentile)
/// - `upper`: Third quartile (Q3, 75th percentile)
/// - `ymax`: Upper whisker (maximum non-outlier value)
///
/// Whisker endpoints are determined by the `coef` parameter (default 1.5).
/// The whiskers extend to the most extreme data point within `coef × IQR`
/// from the quartiles, where IQR (interquartile range) = Q3 - Q1.
///
/// Outlier detection and rendering is handled by the geom, not the stat.
/// This ensures each row in the computed data represents one box.
///
/// # Example
///
/// ```rust,ignore
/// use gogplot::stat::Stat;
/// 
/// Plot::new(data)
///     .aes(|a| {
///         a.x("category");
///         a.y("value");
///     })
///     .geom_boxplot()  // Uses Stat::Boxplot internally
/// ```
#[derive(Debug, Clone)]
pub struct Boxplot {
    /// Coefficient for outlier detection (default: 1.5)
    /// Outliers are values beyond coef * IQR from quartiles
    pub coef: f64,
}

impl Boxplot {
    /// Create a new Boxplot stat with default parameters
    pub fn new() -> Self {
        Self { coef: 1.5 }
    }

    /// Set the outlier coefficient
    ///
    /// Values beyond `coef * IQR` from the quartiles are considered outliers.
    /// Common values:
    /// - 1.5 (default): Standard Tukey boxplot
    /// - 3.0: Far outliers only
    pub fn with_coef(mut self, coef: f64) -> Self {
        self.coef = coef;
        self
    }

    /// Compute five-number summary from sorted data
    fn compute_five_number_summary(&self, sorted_data: &[f64]) -> (f64, f64, f64, f64, f64, f64, f64) {
        let n = sorted_data.len();
        
        if n == 0 {
            return (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        }
        
        if n == 1 {
            let val = sorted_data[0];
            return (val, val, val, val, val, val, val);
        }

        // Compute quartiles using the midpoint method (R type 7)
        let q1 = percentile(sorted_data, 0.25);
        let median = percentile(sorted_data, 0.5);
        let q3 = percentile(sorted_data, 0.75);

        // Compute whiskers and outliers
        let iqr = q3 - q1;
        let lower_fence = q1 - self.coef * iqr;
        let upper_fence = q3 + self.coef * iqr;

        // Find whisker endpoints (min/max values within fences)
        let ymin = sorted_data
            .iter()
            .find(|&&v| v >= lower_fence)
            .copied()
            .unwrap_or(sorted_data[0]);

        let ymax = sorted_data
            .iter()
            .rev()
            .find(|&&v| v <= upper_fence)
            .copied()
            .unwrap_or(sorted_data[n - 1]);

        (ymin, q1, median, q3, ymax, lower_fence, upper_fence)
    }
}

impl Default for Boxplot {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to compute boxplot statistics for a specific x value type
fn compute_boxplot_stats<T>(
    x_values: Vec<T>,
    y_values: Vec<f64>,
    composite_keys: Vec<String>,
    _all_group_cols: &[(Aesthetic, String)],
    coef: f64,
) -> (Vec<T>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<String>)
where
    T: Clone + std::fmt::Debug,
{

    
    let mut result_x = Vec::new();
    let mut result_y = Vec::new();
    let mut result_ymin = Vec::new();
    let mut result_lower = Vec::new();
    let mut result_middle = Vec::new();
    let mut result_upper = Vec::new();
    let mut result_ymax = Vec::new();
    let mut result_group_keys = Vec::new();

    // Group y values by composite key and sort for deterministic order
    let sorted_groups = group_by_key_sorted(&y_values, &composite_keys);
    let boxplot = Boxplot { coef };

    for (group_key, mut group_values) in sorted_groups {
        if group_values.is_empty() {
            continue;
        }

        // Sort values for quantile computation
        group_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let (ymin, q1, median, q3, ymax, lower_fence, upper_fence) = boxplot.compute_five_number_summary(&group_values);

        // Get the x value for this group by finding the first matching row
        let group_x_idx = composite_keys.iter()
            .position(|k| k == &group_key)
            .expect("Composite key should exist in keys");
        
        let x_val = x_values[group_x_idx].clone();

        // Add box statistics (one row per box)
        result_x.push(x_val.clone());
        result_y.push(f64::NAN);
        result_ymin.push(ymin);
        result_lower.push(q1);
        result_middle.push(median);
        result_upper.push(q3);
        result_ymax.push(ymax);
        result_group_keys.push(group_key.clone());

        // Add outlier rows (check against fences, not whiskers)
        for &value in &group_values {
            if value < lower_fence || value > upper_fence {
                result_x.push(x_val.clone());
                result_y.push(value);
                result_ymin.push(f64::NAN);
                result_lower.push(f64::NAN);
                result_middle.push(f64::NAN);
                result_upper.push(f64::NAN);
                result_ymax.push(f64::NAN);
                result_group_keys.push(group_key.clone());
            }
        }
    }

    (result_x, result_y, result_ymin, result_lower, result_middle, result_upper, result_ymax, result_group_keys)
}

impl StatTransform for Boxplot {
    fn apply(
        &self,
        data: Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<Option<(Box<dyn DataSource>, AesMap)>> {
        // Get x and y column names from mapping
        // Get grouping aesthetics for splitting data
        let group_cols = get_grouping_columns(mapping);

        // Get x column and y values
        let x_col_name = mapping
            .get(&Aesthetic::X)
            .and_then(|v| v.as_column_name())
            .ok_or_else(|| PlotError::missing_stat_input("boxplot", Aesthetic::X))?;
        
        let x_col = data
            .get(x_col_name)
            .ok_or_else(|| PlotError::missing_column(x_col_name))?;
        
        let y_values = get_numeric_values(data.as_ref(), mapping, Aesthetic::Y)?;

        // Extract x values in their native type and parallel vectors for grouping
        enum XValues {
            Int(Vec<i64>),
            Float(Vec<f64>),
            Str(Vec<String>),
        }
        
        let x_values = if let Some(int_iter) = x_col.iter_int() {
            XValues::Int(int_iter.collect())
        } else if let Some(float_iter) = x_col.iter_float() {
            XValues::Float(float_iter.collect())
        } else if let Some(str_iter) = x_col.iter_str() {
            XValues::Str(str_iter.map(|s| s.to_string()).collect())
        } else {
            return Err(PlotError::invalid_column_type(
                x_col_name,
                "int, float, or string",
            ));
        };

        // Add x column to grouping if not already present
        let mut all_group_cols = vec![(Aesthetic::X, x_col_name.to_string())];
        for (aes, col_name) in &group_cols {
            if col_name != x_col_name {
                all_group_cols.push((*aes, col_name.clone()));
            }
        }

        // Create composite keys combining x and other grouping columns
        let composite_keys = create_composite_keys(data.as_ref(), &all_group_cols);

        // Compute statistics using the generic helper based on x value type
        let (result_x, result_y, result_ymin, result_lower, result_middle, result_upper, result_ymax, result_group_keys) = match x_values {
            XValues::Int(vals) => {
                let (x, y, ymin, lower, middle, upper, ymax, keys) = 
                    compute_boxplot_stats(vals, y_values, composite_keys, &all_group_cols, self.coef);
                (XValues::Int(x), y, ymin, lower, middle, upper, ymax, keys)
            }
            XValues::Float(vals) => {
                let (x, y, ymin, lower, middle, upper, ymax, keys) = 
                    compute_boxplot_stats(vals, y_values, composite_keys, &all_group_cols, self.coef);
                (XValues::Float(x), y, ymin, lower, middle, upper, ymax, keys)
            }
            XValues::Str(vals) => {
                let (x, y, ymin, lower, middle, upper, ymax, keys) = 
                    compute_boxplot_stats(vals, y_values, composite_keys, &all_group_cols, self.coef);
                (XValues::Str(x), y, ymin, lower, middle, upper, ymax, keys)
            }
        };

        // Create output dataframe with box statistics and outliers
        let mut computed = DataFrame::new();
        
        // Add x column with the same type as the input
        match result_x {
            XValues::Int(vals) => {
                computed.add_column("x", Box::new(IntVec(vals)));
            }
            XValues::Float(vals) => {
                computed.add_column("x", Box::new(FloatVec(vals)));
            }
            XValues::Str(vals) => {
                computed.add_column("x", Box::new(StrVec(vals)));
            }
        }
        
        computed.add_column("y", Box::new(FloatVec(result_y)));
        computed.add_column("ymin", Box::new(FloatVec(result_ymin)));
        computed.add_column("lower", Box::new(FloatVec(result_lower)));
        computed.add_column("middle", Box::new(FloatVec(result_middle)));
        computed.add_column("upper", Box::new(FloatVec(result_upper)));
        computed.add_column("ymax", Box::new(FloatVec(result_ymax)));

        // Add grouping columns by splitting composite keys
        for (aesthetic, col_name) in &all_group_cols {
            if col_name == x_col_name {
                continue; // x already added
            }
            let values: Vec<String> = result_group_keys
                .iter()
                .map(|key| {
                    let key_parts = split_composite_key(key, &all_group_cols);
                    key_parts
                        .iter()
                        .find(|(aes, _)| aes == aesthetic)
                        .map(|(_, val)| val.clone())
                        .unwrap_or_default()
                })
                .collect();
            computed.add_column(col_name, Box::new(StrVec(values)));
        }

        // Update mapping
        let mut new_mapping = mapping.clone();
        new_mapping.set(Aesthetic::X, AesValue::column("x"));
        new_mapping.set(Aesthetic::Y, AesValue::column("y"));
        new_mapping.set(Aesthetic::Ymin, AesValue::column("ymin"));
        new_mapping.set(Aesthetic::Lower, AesValue::column("lower"));
        new_mapping.set(Aesthetic::Middle, AesValue::column("middle"));
        new_mapping.set(Aesthetic::Upper, AesValue::column("upper"));
        new_mapping.set(Aesthetic::Ymax, AesValue::column("ymax"));

        // If any grouping aesthetics were mapped to the same column as X,
        // remap them to "x" in the computed data as categorical
        // (X is categorical for boxplots, so Fill/Color should be too)
        for (aesthetic, col_name) in &group_cols {
            if col_name == x_col_name {
                new_mapping.set(*aesthetic, AesValue::categorical("x"));
            }
        }

        Ok(Some((Box::new(computed), new_mapping)))
    }
}

/// Compute a percentile using linear interpolation (R type 7)
fn percentile(sorted_data: &[f64], p: f64) -> f64 {
    let n = sorted_data.len();
    
    if n == 0 {
        return 0.0;
    }
    
    if n == 1 {
        return sorted_data[0];
    }

    // Use R's type 7 quantile method (linear interpolation)
    let h = (n - 1) as f64 * p;
    let h_floor = h.floor();
    let h_ceil = h.ceil();
    
    let lower_idx = h_floor as usize;
    let upper_idx = h_ceil as usize;
    
    if lower_idx == upper_idx {
        sorted_data[lower_idx]
    } else {
        let lower_val = sorted_data[lower_idx];
        let upper_val = sorted_data[upper_idx];
        lower_val + (h - h_floor) * (upper_val - lower_val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_percentile() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        assert_eq!(percentile(&data, 0.0), 1.0);
        assert_eq!(percentile(&data, 0.5), 3.0);
        assert_eq!(percentile(&data, 1.0), 5.0);
        assert_eq!(percentile(&data, 0.25), 2.0);
        assert_eq!(percentile(&data, 0.75), 4.0);
    }

    #[test]
    fn test_five_number_summary() {
        let boxplot = Boxplot::new();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let (ymin, q1, median, q3, ymax, _lower_fence, _upper_fence) = boxplot.compute_five_number_summary(&data);
        
        assert_eq!(median, 5.5);
        assert_eq!(q1, 3.25);
        assert_eq!(q3, 7.75);
        // Whiskers should be at 1.0 and 10.0 (no outliers)
        assert_eq!(ymin, 1.0);
        assert_eq!(ymax, 10.0);
    }

    #[test]
    fn test_boxplot_with_outliers() {
        let boxplot = Boxplot::new();
        // Data with clear outliers: [1, 10, 10, 10, 10, 10, 10, 100]
        let data = vec![1.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 100.0];
        let (ymin, q1, median, q3, ymax, _lower_fence, _upper_fence) = boxplot.compute_five_number_summary(&data);
        
        // Q1 = 10, Q3 = 10, IQR = 0, fences at 10 ± 0 = 10
        // So 1.0 and 100.0 should be outliers, whiskers at 10
        assert_eq!(median, 10.0);
        assert_eq!(q1, 10.0);
        assert_eq!(q3, 10.0);
        assert_eq!(ymin, 10.0);
        assert_eq!(ymax, 10.0);
    }

    #[test]
    fn test_boxplot_stat_basic() {
        use crate::utils::dataframe::{DataFrame, FloatVec, IntVec};
        use crate::aesthetics::{AesMap, AesValue, Aesthetic};
        
        // Create simple test data: x = [1, 1, 2, 2], y = [10, 20, 30, 40]
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(IntVec(vec![1, 1, 2, 2])));
        df.add_column("y", Box::new(FloatVec(vec![10.0, 20.0, 30.0, 40.0])));
        
        let mut mapping = AesMap::new();
        mapping.set(Aesthetic::X, AesValue::column("x"));
        mapping.set(Aesthetic::Y, AesValue::column("y"));
        
        let boxplot = Boxplot::new();
        let result = boxplot.apply(Box::new(df), &mapping).unwrap();
        
        assert!(result.is_some());
        let (computed, new_mapping) = result.unwrap();
        
        // Should have 2 box rows (one per x group), no outliers
        // Verify we have data by checking column exists
        assert!(computed.get("x").is_some());
        
        // Check x column
        let x_col = computed.get("x").unwrap();
        let x_vals: Vec<i64> = x_col.iter_int().unwrap().collect();
        assert!(x_vals.contains(&1));
        assert!(x_vals.contains(&2));
        
        // Check that middle values are not NaN (these are box rows, not outliers)
        let middle_col = computed.get("middle").unwrap();
        let middle_vals: Vec<f64> = middle_col.iter_float().unwrap().collect();
        assert_eq!(middle_vals.len(), 2);
        assert!(!middle_vals[0].is_nan());
        assert!(!middle_vals[1].is_nan());
        
        // Verify mapping was updated
        assert_eq!(new_mapping.get(&Aesthetic::X), Some(&AesValue::column("x")));
        assert_eq!(new_mapping.get(&Aesthetic::Middle), Some(&AesValue::column("middle")));
    }

    #[test]
    fn test_boxplot_stat_with_outliers() {
        use crate::utils::dataframe::{DataFrame, FloatVec, IntVec};
        use crate::aesthetics::{AesMap, AesValue, Aesthetic};
        
        // Create data with outliers: x=[1,1,1,1,1], y=[1, 10, 11, 12, 100]
        // Q1=10, Q3=12, IQR=2, fences at 7 and 15
        // 1 and 100 should be outliers
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(IntVec(vec![1, 1, 1, 1, 1])));
        df.add_column("y", Box::new(FloatVec(vec![1.0, 10.0, 11.0, 12.0, 100.0])));
        
        let mut mapping = AesMap::new();
        mapping.set(Aesthetic::X, AesValue::column("x"));
        mapping.set(Aesthetic::Y, AesValue::column("y"));
        
        let boxplot = Boxplot::new();
        let result = boxplot.apply(Box::new(df), &mapping).unwrap();
        
        assert!(result.is_some());
        let (computed, _) = result.unwrap();
        
        // Check middle column: 1 non-NaN (box), 2 NaN (outliers)
        let middle_col = computed.get("middle").unwrap();
        let middle_vals: Vec<f64> = middle_col.iter_float().unwrap().collect();
        let nan_count = middle_vals.iter().filter(|v| v.is_nan()).count();
        assert_eq!(nan_count, 2, "Should have 2 outlier rows with NaN middle values");
        
        // Check y column: 1 NaN (box), 2 non-NaN (outliers)
        let y_col = computed.get("y").unwrap();
        let y_vals: Vec<f64> = y_col.iter_float().unwrap().collect();
        let y_nan_count = y_vals.iter().filter(|v| v.is_nan()).count();
        assert_eq!(y_nan_count, 1, "Should have 1 box row with NaN y value");
        
        // Verify outlier values
        let outlier_y_vals: Vec<f64> = y_vals.iter().filter(|v| !v.is_nan()).copied().collect();
        assert_eq!(outlier_y_vals.len(), 2);
        assert!(outlier_y_vals.contains(&1.0));
        assert!(outlier_y_vals.contains(&100.0));
    }

    #[test]
    fn test_boxplot_preserves_x_type_int() {
        use crate::utils::dataframe::{DataFrame, FloatVec, IntVec};
        use crate::aesthetics::{AesMap, AesValue, Aesthetic};
        
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(IntVec(vec![4, 4, 6, 6])));
        df.add_column("y", Box::new(FloatVec(vec![10.0, 20.0, 30.0, 40.0])));
        
        let mut mapping = AesMap::new();
        mapping.set(Aesthetic::X, AesValue::column("x"));
        mapping.set(Aesthetic::Y, AesValue::column("y"));
        
        let boxplot = Boxplot::new();
        let result = boxplot.apply(Box::new(df), &mapping).unwrap().unwrap();
        let (computed, _) = result;
        
        // X column should be IntVec
        let x_col = computed.get("x").unwrap();
        assert!(x_col.iter_int().is_some(), "X column should be integer type");
        
        let x_vals: Vec<i64> = x_col.iter_int().unwrap().collect();
        assert_eq!(x_vals.len(), 2); // 2 boxes
        assert!(x_vals.contains(&4));
        assert!(x_vals.contains(&6));
    }

    #[test]
    fn test_boxplot_preserves_x_type_float() {
        use crate::utils::dataframe::{DataFrame, FloatVec};
        use crate::aesthetics::{AesMap, AesValue, Aesthetic};
        
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(FloatVec(vec![1.5, 1.5, 2.5, 2.5])));
        df.add_column("y", Box::new(FloatVec(vec![10.0, 20.0, 30.0, 40.0])));
        
        let mut mapping = AesMap::new();
        mapping.set(Aesthetic::X, AesValue::column("x"));
        mapping.set(Aesthetic::Y, AesValue::column("y"));
        
        let boxplot = Boxplot::new();
        let result = boxplot.apply(Box::new(df), &mapping).unwrap().unwrap();
        let (computed, _) = result;
        
        // X column should be FloatVec
        let x_col = computed.get("x").unwrap();
        assert!(x_col.iter_float().is_some(), "X column should be float type");
        
        let x_vals: Vec<f64> = x_col.iter_float().unwrap().collect();
        assert_eq!(x_vals.len(), 2);
        assert!(x_vals.contains(&1.5));
        assert!(x_vals.contains(&2.5));
    }

    #[test]
    fn test_boxplot_preserves_x_type_string() {
        use crate::utils::dataframe::{DataFrame, FloatVec, StrVec};
        use crate::aesthetics::{AesMap, AesValue, Aesthetic};
        
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(StrVec(vec!["A".to_string(), "A".to_string(), "B".to_string(), "B".to_string()])));
        df.add_column("y", Box::new(FloatVec(vec![10.0, 20.0, 30.0, 40.0])));
        
        let mut mapping = AesMap::new();
        mapping.set(Aesthetic::X, AesValue::column("x"));
        mapping.set(Aesthetic::Y, AesValue::column("y"));
        
        let boxplot = Boxplot::new();
        let result = boxplot.apply(Box::new(df), &mapping).unwrap().unwrap();
        let (computed, _) = result;
        
        // X column should be StrVec
        let x_col = computed.get("x").unwrap();
        assert!(x_col.iter_str().is_some(), "X column should be string type");
        
        let x_vals: Vec<String> = x_col.iter_str().unwrap().map(|s| s.to_string()).collect();
        assert_eq!(x_vals.len(), 2);
        assert!(x_vals.contains(&"A".to_string()));
        assert!(x_vals.contains(&"B".to_string()));
    }

}
