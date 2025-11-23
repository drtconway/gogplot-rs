//! Boxplot statistics
//!
//! Computes the five-number summary (minimum, Q1, median, Q3, maximum)
//! and identifies outliers for creating box-and-whisker plots.

use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::data::DataSource;
use crate::error::{PlotError, Result};
use crate::stat::StatTransform;
use crate::utils::dataframe::{DataFrame, FloatVec, StrVec};
use crate::utils::grouping::{get_grouping_columns, create_composite_keys, group_by_key, split_composite_key};

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
    fn compute_five_number_summary(&self, sorted_data: &[f64]) -> (f64, f64, f64, f64, f64) {
        let n = sorted_data.len();
        
        if n == 0 {
            return (0.0, 0.0, 0.0, 0.0, 0.0);
        }
        
        if n == 1 {
            let val = sorted_data[0];
            return (val, val, val, val, val);
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

        (ymin, q1, median, q3, ymax)
    }
}

impl Default for Boxplot {
    fn default() -> Self {
        Self::new()
    }
}

impl StatTransform for Boxplot {
    fn apply(
        &self,
        data: Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<Option<(Box<dyn DataSource>, AesMap)>> {
        // Get x and y column names from mapping
        let x_col_name = mapping
            .get(&Aesthetic::X)
            .and_then(|v| v.as_column_name())
            .ok_or_else(|| PlotError::missing_stat_input("boxplot", Aesthetic::X))?;

        let y_col_name = mapping
            .get(&Aesthetic::Y)
            .and_then(|v| v.as_column_name())
            .ok_or_else(|| PlotError::missing_stat_input("boxplot", Aesthetic::Y))?;

        // Get grouping aesthetics for splitting data
        let group_cols = get_grouping_columns(mapping);

        // Get y column for statistics
        let y_col = data
            .get(y_col_name)
            .ok_or_else(|| PlotError::missing_column(y_col_name))?;

        let y_values: Vec<f64> = if let Some(float_iter) = y_col.iter_float() {
            float_iter.filter(|v| v.is_finite()).collect()
        } else if let Some(int_iter) = y_col.iter_int() {
            int_iter.map(|i| i as f64).collect()
        } else {
            return Err(PlotError::invalid_column_type(
                y_col_name,
                "numeric (int or float)",
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

        // Group y values by composite key
        let groups = group_by_key(&y_values, &composite_keys);

        // Compute statistics for each group
        let mut result_x = Vec::new();
        let mut result_ymin = Vec::new();
        let mut result_lower = Vec::new();
        let mut result_middle = Vec::new();
        let mut result_upper = Vec::new();
        let mut result_ymax = Vec::new();
        let mut result_group_keys = Vec::new();

        for (group_key, mut group_values) in groups {
            if group_values.is_empty() {
                continue;
            }

            // Sort values for quantile computation
            group_values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            let (ymin, q1, median, q3, ymax) = self.compute_five_number_summary(&group_values);

            // Extract x value from composite key (first part since X is always first in all_group_cols)
            let key_parts = split_composite_key(&group_key, &all_group_cols);
            let x_value = key_parts.first()
                .map(|(_, val)| val.clone())
                .unwrap_or_else(|| group_key.clone());

            // Add box statistics (one row per box)
            result_x.push(x_value);
            result_ymin.push(ymin);
            result_lower.push(q1);
            result_middle.push(median);
            result_upper.push(q3);
            result_ymax.push(ymax);
            result_group_keys.push(group_key.clone());
        }

        // Create output dataframe with box statistics
        let mut computed = DataFrame::new();
        computed.add_column("x", Box::new(StrVec(result_x)));
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
        new_mapping.set(Aesthetic::Ymin, AesValue::column("ymin"));
        new_mapping.set(Aesthetic::Lower, AesValue::column("lower"));
        new_mapping.set(Aesthetic::Middle, AesValue::column("middle"));
        new_mapping.set(Aesthetic::Upper, AesValue::column("upper"));
        new_mapping.set(Aesthetic::Ymax, AesValue::column("ymax"));

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
        let (ymin, q1, median, q3, ymax) = boxplot.compute_five_number_summary(&data);
        
        assert_eq!(median, 5.5);
        assert_eq!(q1, 3.25);
        assert_eq!(q3, 7.75);
        // Whiskers should be at 1.0 and 10.0 (no outliers)
        assert_eq!(ymin, 1.0);
        assert_eq!(ymax, 10.0);
    }

}
