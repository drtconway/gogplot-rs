//! Boxplot statistics
//!
//! Computes the five-number summary (minimum, Q1, median, Q3, maximum)
//! and identifies outliers for creating box-and-whisker plots.

use std::collections::HashMap;
use std::sync::Arc;

use ordered_float::OrderedFloat;

use crate::aesthetics::{AesMap, AesValue, Aesthetic, AestheticDomain};
use crate::data::{ContinuousType, DiscreteType, PrimitiveType};
use crate::error::{PlotError, Result};
use crate::stat::Stat;
use crate::utils::data::{
    DiscreteContinuousVisitor2, visit2_dc,
};
use crate::utils::dataframe::{DataFrame, FloatVec};

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
}

impl Default for Boxplot {
    fn default() -> Self {
        Self::new()
    }
}

impl Stat for Boxplot {
    fn compute_group(
            &self,
            aesthetics: Vec<Aesthetic>,
            iters: Vec<crate::data::VectorIter<'_>>,
            _params: Option<&dyn std::any::Any>,
        ) -> Result<(DataFrame, AesMap)> {
            let mut both = aesthetics.into_iter().zip(iters.into_iter());
        if let Some((_x_aesthetic, x_iter)) = both.next() {
            if let Some((_y_aesthetic, y_iter)) = both.next() {
                return visit2_dc(
                    x_iter,
                    y_iter,
                    &mut BoxplotCounter::new(self.coef),
                );
            }
        }
        panic!("No aesthetics provided");
    }
}

/// Compute a percentile using linear interpolation (R type 7)
fn percentile(sorted_data: &[OrderedFloat<f64>], p: f64) -> f64 {
    let n = sorted_data.len();

    if n == 0 {
        return 0.0;
    }

    if n == 1 {
        return sorted_data[0].0;
    }

    // Use R's type 7 quantile method (linear interpolation)
    let h = (n - 1) as f64 * p;
    let h_floor = h.floor();
    let h_ceil = h.ceil();

    let lower_idx = h_floor as usize;
    let upper_idx = h_ceil as usize;

    if lower_idx == upper_idx {
        sorted_data[lower_idx].0
    } else {
        let lower_val = sorted_data[lower_idx].0;
        let upper_val = sorted_data[upper_idx].0;
        lower_val + (h - h_floor) * (upper_val - lower_val)
    }
}

struct BoxplotCounter {
    coef: f64,
}

impl BoxplotCounter {
    fn new(coef: f64) -> Self {
        Self { coef }
    }
}

impl DiscreteContinuousVisitor2 for BoxplotCounter {
    type Output = (DataFrame, AesMap);

    fn visit<
        T: crate::utils::data::Vectorable + DiscreteType,
        U: crate::utils::data::Vectorable + ContinuousType,
    >(
        &mut self,
        x_iter: impl Iterator<Item = T>,
        y_iter: impl Iterator<Item = U>,
    ) -> std::result::Result<Self::Output, PlotError> {
        let mut grouped_data: HashMap<T::Sortable, Vec<OrderedFloat<f64>>> = HashMap::new();
        for (x, y) in x_iter.zip(y_iter) {
            let y = y.to_f64();
            if y.is_finite() {
                grouped_data
                    .entry(x.to_sortable())
                    .or_default()
                    .push(y.to_sortable());
            }
        }

        let mut pairs = grouped_data.into_iter().collect::<Vec<_>>();
        pairs.sort_by(|a, b| a.0.cmp(&b.0));

        let mut result_x: Vec<T> = Vec::new();
        let mut result_y = Vec::new();
        let mut result_ymin = Vec::new();
        let mut result_lower = Vec::new();
        let mut result_middle = Vec::new();
        let mut result_upper = Vec::new();
        let mut result_ymax = Vec::new();

        for (x, mut values) in pairs.into_iter() {
            if values.is_empty() {
                continue;
            }

            values.sort();

            let (ymin, q1, median, q3, ymax, lower_fence, upper_fence) =
                compute_five_number_summary(self.coef, &values);

            // Add box statistics
            result_x.push(T::from_sortable(x.clone()));
            result_y.push(f64::NAN);
            result_ymin.push(ymin);
            result_lower.push(q1);
            result_middle.push(median);
            result_upper.push(q3);
            result_ymax.push(ymax);

            // Add outliers
            for &value in &values {
                if value.0 < lower_fence || value.0 > upper_fence {
                    result_x.push(T::from_sortable(x.clone()));
                    result_y.push(value.0);
                    result_ymin.push(f64::NAN);
                    result_lower.push(f64::NAN);
                    result_middle.push(f64::NAN);
                    result_upper.push(f64::NAN);
                    result_ymax.push(f64::NAN);
                }
            }
        }

        let mut data = DataFrame::new();
        let mut mapping = AesMap::new();

        data.add_column("x", T::make_vector(result_x));
        data.add_column("y", Arc::new(FloatVec(result_y)));
        data.add_column("ymin", Arc::new(FloatVec(result_ymin)));
        data.add_column("lower", Arc::new(FloatVec(result_lower)));
        data.add_column("middle", Arc::new(FloatVec(result_middle)));
        data.add_column("upper", Arc::new(FloatVec(result_upper)));
        data.add_column("ymax", Arc::new(FloatVec(result_ymax)));

        // Update mapping
        mapping.set(
            Aesthetic::X(AestheticDomain::Discrete),
            AesValue::column("x"),
        );
        mapping.set(
            Aesthetic::Xmin(AestheticDomain::Discrete),
            AesValue::column("x"),
        );
        mapping.set(
            Aesthetic::Xmax(AestheticDomain::Discrete),
            AesValue::column("x"),
        );
        mapping.set(
            Aesthetic::Y(AestheticDomain::Continuous),
            AesValue::column("y"),
        );
        mapping.set(
            Aesthetic::Ymin(AestheticDomain::Continuous),
            AesValue::column("ymin"),
        );
        mapping.set(
            Aesthetic::Ymax(AestheticDomain::Continuous),
            AesValue::column("ymax"),
        );
        mapping.set(Aesthetic::Lower, AesValue::column("lower"));
        mapping.set(Aesthetic::Middle, AesValue::column("middle"));
        mapping.set(Aesthetic::Upper, AesValue::column("upper"));

        Ok((data, mapping))
    }
}

/// Compute five-number summary from sorted data
fn compute_five_number_summary(
    coef: f64,
    sorted_data: &[OrderedFloat<f64>],
) -> (f64, f64, f64, f64, f64, f64, f64) {
    let n = sorted_data.len();

    if n == 0 {
        return (0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    }

    if n == 1 {
        let val = sorted_data[0].0;
        return (val, val, val, val, val, val, val);
    }

    // Compute quartiles using the midpoint method (R type 7)
    let q1 = percentile(sorted_data, 0.25);
    let median = percentile(sorted_data, 0.5);
    let q3 = percentile(sorted_data, 0.75);

    // Compute whiskers and outliers
    let iqr = q3 - q1;
    let lower_fence = q1 - coef * iqr;
    let upper_fence = q3 + coef * iqr;

    // Find whisker endpoints (min/max values within fences)
    let ymin = sorted_data
        .iter()
        .find(|&&v| v.0 >= lower_fence)
        .copied()
        .unwrap_or(sorted_data[0])
        .0;

    let ymax = sorted_data
        .iter()
        .rev()
        .find(|&&v| v.0 <= upper_fence)
        .copied()
        .unwrap_or(sorted_data[n - 1])
        .0;

    (ymin, q1, median, q3, ymax, lower_fence, upper_fence)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::data::DataSource;

    use super::*;

    #[test]
    fn test_percentile() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let data: Vec<OrderedFloat<f64>> = data.into_iter().map(OrderedFloat).collect();
        assert_eq!(percentile(&data, 0.0), 1.0);
        assert_eq!(percentile(&data, 0.5), 3.0);
        assert_eq!(percentile(&data, 1.0), 5.0);
        assert_eq!(percentile(&data, 0.25), 2.0);
        assert_eq!(percentile(&data, 0.75), 4.0);
    }

    #[test]
    fn test_five_number_summary() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
        let data: Vec<OrderedFloat<f64>> = data.into_iter().map(OrderedFloat).collect();
        let (ymin, q1, median, q3, ymax, _lower_fence, _upper_fence) =
            compute_five_number_summary(1.5, &data);

        assert_eq!(median, 5.5);
        assert_eq!(q1, 3.25);
        assert_eq!(q3, 7.75);
        // Whiskers should be at 1.0 and 10.0 (no outliers)
        assert_eq!(ymin, 1.0);
        assert_eq!(ymax, 10.0);
    }

    #[test]
    fn test_boxplot_with_outliers() {
        // Data with clear outliers: [1, 10, 10, 10, 10, 10, 10, 100]
        let data = vec![1.0, 10.0, 10.0, 10.0, 10.0, 10.0, 10.0, 100.0];
        let data: Vec<OrderedFloat<f64>> = data.into_iter().map(OrderedFloat).collect();
        let (ymin, q1, median, q3, ymax, _lower_fence, _upper_fence) =
            compute_five_number_summary(1.5, &data);

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
        use crate::aesthetics::{AesMap, AesValue, Aesthetic};
        use crate::utils::dataframe::{DataFrame, FloatVec, IntVec};

        // Create simple test data: x = [1, 1, 2, 2], y = [10, 20, 30, 40]
        let mut df = DataFrame::new();
        df.add_column("x", Arc::new(IntVec(vec![1, 1, 2, 2])));
        df.add_column("y", Arc::new(FloatVec(vec![10.0, 20.0, 30.0, 40.0])));
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.set(
            Aesthetic::X(AestheticDomain::Discrete),
            AesValue::column("x"),
        );
        mapping.set(
            Aesthetic::Y(AestheticDomain::Continuous),
            AesValue::column("y"),
        );

        let boxplot = Boxplot::new();
        let (computed, new_mapping) = boxplot.compute(df.as_ref(), &mapping).unwrap();

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
        assert_eq!(
            new_mapping.get(&Aesthetic::X(AestheticDomain::Discrete)),
            Some(&AesValue::column("x"))
        );
        assert_eq!(
            new_mapping.get(&Aesthetic::Middle),
            Some(&AesValue::column("middle"))
        );
    }

    #[test]
    fn test_boxplot_stat_with_outliers() {
        use crate::aesthetics::{AesMap, AesValue, Aesthetic};
        use crate::utils::dataframe::{DataFrame, FloatVec, IntVec};

        // Create data with outliers: x=[1,1,1,1,1], y=[1, 10, 11, 12, 100]
        // Q1=10, Q3=12, IQR=2, fences at 7 and 15
        // 1 and 100 should be outliers
        let mut df = DataFrame::new();
        df.add_column("x", Arc::new(IntVec(vec![1, 1, 1, 1, 1])));
        df.add_column("y", Arc::new(FloatVec(vec![1.0, 10.0, 11.0, 12.0, 100.0])));
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.set(
            Aesthetic::X(AestheticDomain::Discrete),
            AesValue::column("x"),
        );
        mapping.set(
            Aesthetic::Y(AestheticDomain::Continuous),
            AesValue::column("y"),
        );

        let boxplot = Boxplot::new();
        let (computed, _) = boxplot.compute(df.as_ref(), &mapping).unwrap();

        // Check middle column: 1 non-NaN (box), 2 NaN (outliers)
        let middle_col = computed.get("middle").unwrap();
        let middle_vals: Vec<f64> = middle_col.iter_float().unwrap().collect();
        let nan_count = middle_vals.iter().filter(|v| v.is_nan()).count();
        assert_eq!(
            nan_count, 2,
            "Should have 2 outlier rows with NaN middle values"
        );

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
        use crate::aesthetics::{AesMap, AesValue, Aesthetic};
        use crate::utils::dataframe::{DataFrame, FloatVec, IntVec};

        let mut df = DataFrame::new();
        df.add_column("x", Arc::new(IntVec(vec![4, 4, 6, 6])));
        df.add_column("y", Arc::new(FloatVec(vec![10.0, 20.0, 30.0, 40.0])));
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.set(
            Aesthetic::X(AestheticDomain::Discrete),
            AesValue::column("x"),
        );
        mapping.set(
            Aesthetic::Y(AestheticDomain::Continuous),
            AesValue::column("y"),
        );

        let boxplot = Boxplot::new();
        let (computed, _) = boxplot.compute(df.as_ref(), &mapping).unwrap();

        // X column should be IntVec
        let x_col = computed.get("x").unwrap();
        assert!(
            x_col.iter_int().is_some(),
            "X column should be integer type"
        );

        let x_vals: Vec<i64> = x_col.iter_int().unwrap().collect();
        assert_eq!(x_vals.len(), 2); // 2 boxes
        assert!(x_vals.contains(&4));
        assert!(x_vals.contains(&6));
    }

    #[test]
    fn test_boxplot_preserves_x_type_float() {
        use crate::aesthetics::{AesMap, AesValue, Aesthetic};
        use crate::utils::dataframe::{DataFrame, FloatVec};

        let mut df = DataFrame::new();
        df.add_column("x", Arc::new(FloatVec(vec![1.5, 1.5, 2.5, 2.5])));
        df.add_column("y", Arc::new(FloatVec(vec![10.0, 20.0, 30.0, 40.0])));
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.set(
            Aesthetic::X(AestheticDomain::Discrete),
            AesValue::column("x"),
        );
        mapping.set(
            Aesthetic::Y(AestheticDomain::Continuous),
            AesValue::column("y"),
        );

        let boxplot = Boxplot::new();
        let (computed, _) = boxplot.compute(df.as_ref(), &mapping).unwrap();

        // X column should be FloatVec
        let x_col = computed.get("x").unwrap();
        assert!(
            x_col.iter_float().is_some(),
            "X column should be float type"
        );

        let x_vals: Vec<f64> = x_col.iter_float().unwrap().collect();
        assert_eq!(x_vals.len(), 2);
        assert!(x_vals.contains(&1.5));
        assert!(x_vals.contains(&2.5));
    }

    #[test]
    fn test_boxplot_preserves_x_type_string() {
        use crate::aesthetics::{AesMap, AesValue, Aesthetic};
        use crate::utils::dataframe::{DataFrame, FloatVec, StrVec};

        let mut df = DataFrame::new();
        df.add_column("x", Arc::new(StrVec::from(vec!["A", "A", "B", "B"])));
        df.add_column("y", Arc::new(FloatVec(vec![10.0, 20.0, 30.0, 40.0])));
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.set(
            Aesthetic::X(AestheticDomain::Discrete),
            AesValue::column("x"),
        );
        mapping.set(
            Aesthetic::Y(AestheticDomain::Continuous),
            AesValue::column("y"),
        );

        let boxplot = Boxplot::new();
        let (computed, _) = boxplot.compute(df.as_ref(), &mapping).unwrap();

        // X column should be StrVec
        let x_col = computed.get("x").unwrap();
        assert!(x_col.iter_str().is_some(), "X column should be string type");

        let x_vals: Vec<String> = x_col.iter_str().unwrap().map(|s| s.to_string()).collect();
        assert_eq!(x_vals.len(), 2);
        assert!(x_vals.contains(&"A".to_string()));
        assert!(x_vals.contains(&"B".to_string()));
    }
}
