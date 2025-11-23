//! Smooth statistics for fitted curves with confidence intervals
//!
//! Computes fitted values and confidence intervals using various smoothing methods.

use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::data::DataSource;
use crate::error::{PlotError, Result};
use crate::stat::utils::get_numeric_values;
use crate::stat::StatTransform;
use crate::utils::dataframe::{DataFrame, FloatVec};
use crate::utils::grouping::{create_composite_keys, get_grouping_columns, group_by_key_sorted};

/// Smoothing method to use
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Method {
    /// Linear model (simple linear regression)
    Lm,
    /// Cubic spline interpolation
    Spline,
    /// Local polynomial regression (loess) - not yet implemented
    Loess,
}

/// Smooth statistics computation
///
/// Fits a curve to the data and computes confidence intervals.
/// Currently supports:
/// - `lm`: Linear regression (y = a + bx)
/// - `spline`: Cubic spline interpolation
///
/// # Output columns
/// - `x`: x values at which predictions are made (evenly spaced)
/// - `y`: predicted y values
/// - `ymin`: lower confidence bound
/// - `ymax`: upper confidence bound
/// - `se`: standard error of prediction
///
/// # Example
///
/// ```rust,ignore
/// use gogplot::stat::Stat;
/// 
/// Plot::new(data)
///     .aes(|a| {
///         a.x("x");
///         a.y("y");
///     })
///     .geom_smooth()  // Uses Stat::Smooth internally
/// ```
#[derive(Debug, Clone)]
pub struct Smooth {
    /// Smoothing method
    pub method: Method,
    
    /// Confidence level (default 0.95 for 95% CI)
    pub level: f64,
    
    /// Number of points to compute predictions at (default 80)
    pub n: usize,
    
    /// Whether to compute standard errors (default true)
    pub se: bool,
}

impl Smooth {
    /// Create a new Smooth stat with default parameters
    pub fn new() -> Self {
        Self {
            method: Method::Lm,
            level: 0.95,
            n: 80,
            se: true,
        }
    }
    
    /// Set the smoothing method
    pub fn method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }
    
    /// Set the confidence level (e.g., 0.95 for 95% CI)
    pub fn level(mut self, level: f64) -> Self {
        self.level = level.clamp(0.0, 1.0);
        self
    }
    
    /// Set the number of prediction points
    pub fn n(mut self, n: usize) -> Self {
        self.n = n.max(2);
        self
    }
    
    /// Set whether to compute standard errors
    pub fn se(mut self, se: bool) -> Self {
        self.se = se;
        self
    }
}

impl Default for Smooth {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute linear regression: y = a + bx
/// Returns (intercept, slope, residual_standard_error)
fn fit_linear_model(x: &[f64], y: &[f64]) -> Result<(f64, f64, f64)> {
    let n = x.len();
    if n < 2 {
        return Err(PlotError::no_valid_data(
            "Need at least 2 points for linear regression",
        ));
    }
    
    // Compute means
    let x_mean = x.iter().sum::<f64>() / n as f64;
    let y_mean = y.iter().sum::<f64>() / n as f64;
    
    // Compute slope: b = sum((x - x_mean) * (y - y_mean)) / sum((x - x_mean)^2)
    let mut numerator = 0.0;
    let mut denominator = 0.0;
    
    for i in 0..n {
        let x_dev = x[i] - x_mean;
        let y_dev = y[i] - y_mean;
        numerator += x_dev * y_dev;
        denominator += x_dev * x_dev;
    }
    
    if denominator.abs() < 1e-10 {
        return Err(PlotError::no_valid_data(
            "Cannot fit linear model: x values are constant",
        ));
    }
    
    let slope = numerator / denominator;
    let intercept = y_mean - slope * x_mean;
    
    // Compute residual standard error
    let mut sse = 0.0;  // Sum of squared errors
    for i in 0..n {
        let predicted = intercept + slope * x[i];
        let residual = y[i] - predicted;
        sse += residual * residual;
    }
    
    // Residual standard error: sqrt(SSE / (n - 2))
    let rse = if n > 2 {
        (sse / (n - 2) as f64).sqrt()
    } else {
        0.0
    };
    
    Ok((intercept, slope, rse))
}

/// Compute cubic spline smoothing
/// Returns (y_pred, ymin, ymax, se) where ymin/ymax are computed using
/// residual-based confidence bands
fn fit_cubic_spline(
    x_data: &[f64],
    y_data: &[f64],
    x_pred: &[f64],
    level: f64,
) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>)> {
    use splines::{Interpolation, Key, Spline};
    
    let n = x_data.len();
    if n < 4 {
        return Err(PlotError::no_valid_data(
            "Need at least 4 points for cubic spline interpolation",
        ));
    }
    
    // Sort data by x (spline requires sorted keys)
    let mut paired: Vec<(f64, f64)> = x_data.iter().zip(y_data.iter())
        .map(|(&x, &y)| (x, y))
        .collect();
    paired.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    
    // Create spline keys
    let keys: Vec<Key<f64, f64>> = paired
        .iter()
        .map(|(x, y)| Key::new(*x, *y, Interpolation::CatmullRom))
        .collect();
    
    let spline = Spline::from_vec(keys);
    
    // Compute predictions
    let mut y_pred = Vec::with_capacity(x_pred.len());
    for &x in x_pred {
        if let Some(y) = spline.clamped_sample(x) {
            y_pred.push(y);
        } else {
            // Extrapolate using endpoints
            if x < paired[0].0 {
                y_pred.push(paired[0].1);
            } else {
                y_pred.push(paired[paired.len() - 1].1);
            }
        }
    }
    
    // Compute residual-based confidence intervals
    // Calculate residuals from spline fit at original data points
    let mut residuals = Vec::with_capacity(n);
    for (x, y) in &paired {
        if let Some(y_fit) = spline.sample(*x) {
            residuals.push(y - y_fit);
        }
    }
    
    // Compute residual standard error
    let mean_residual = residuals.iter().sum::<f64>() / residuals.len() as f64;
    let sse: f64 = residuals.iter()
        .map(|r| (r - mean_residual).powi(2))
        .sum();
    let rse = (sse / (n.saturating_sub(4)) as f64).sqrt();
    
    // Use approximate z-value for confidence bands
    let z = match level {
        l if l >= 0.99 => 2.576,
        l if l >= 0.95 => 1.96,
        l if l >= 0.90 => 1.645,
        _ => 1.96,
    };
    
    let se_vec = vec![rse; x_pred.len()];
    let ymin = y_pred.iter().map(|&y| y - z * rse).collect();
    let ymax = y_pred.iter().map(|&y| y + z * rse).collect();
    
    Ok((y_pred, ymin, ymax, se_vec))
}

/// Compute predictions and confidence intervals for linear model
fn predict_linear_model(
    x_pred: &[f64],
    x_data: &[f64],
    intercept: f64,
    slope: f64,
    rse: f64,
    level: f64,
) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let n = x_data.len();
    let x_mean = x_data.iter().sum::<f64>() / n as f64;
    
    // Compute sum of squared deviations for x
    let sxx: f64 = x_data.iter().map(|&x| (x - x_mean).powi(2)).sum();
    
    // t-value for confidence interval (approximate with normal for now)
    // For 95% CI, z ≈ 1.96
    let z = match level {
        l if l >= 0.99 => 2.576,
        l if l >= 0.95 => 1.96,
        l if l >= 0.90 => 1.645,
        _ => 1.96,
    };
    
    let mut y_pred = Vec::with_capacity(x_pred.len());
    let mut ymin = Vec::with_capacity(x_pred.len());
    let mut ymax = Vec::with_capacity(x_pred.len());
    let mut se = Vec::with_capacity(x_pred.len());
    
    for &x in x_pred {
        let y = intercept + slope * x;
        
        // Standard error of prediction
        // SE = s * sqrt(1/n + (x - x_mean)^2 / sxx)
        let se_pred = if sxx > 1e-10 {
            rse * (1.0 / n as f64 + (x - x_mean).powi(2) / sxx).sqrt()
        } else {
            rse
        };
        
        y_pred.push(y);
        ymin.push(y - z * se_pred);
        ymax.push(y + z * se_pred);
        se.push(se_pred);
    }
    
    (y_pred, ymin, ymax, se)
}

impl StatTransform for Smooth {
    fn apply(
        &self,
        data: Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<Option<(Box<dyn DataSource>, AesMap)>> {
        // Get x and y values from aesthetic mappings
        let x_values = get_numeric_values(data.as_ref(), mapping, Aesthetic::X)?;
        let y_values = get_numeric_values(data.as_ref(), mapping, Aesthetic::Y)?;

        // Get grouping aesthetics
        let group_cols = get_grouping_columns(mapping);

        if x_values.len() != y_values.len() {
            return Err(PlotError::no_valid_data(
                "x and y must have the same length",
            ));
        }

        // Create composite keys for grouping
        let all_group_cols = group_cols.clone();
        let composite_keys = create_composite_keys(data.as_ref(), &all_group_cols);

        // Group data by composite keys
        let mut result_x = Vec::new();
        let mut result_y = Vec::new();
        let mut result_ymin = Vec::new();
        let mut result_ymax = Vec::new();
        let mut result_se = Vec::new();
        let mut result_group_keys = Vec::new();

        // Process each group separately
        let sorted_groups = group_by_key_sorted(&(0..x_values.len()).collect::<Vec<_>>(), &composite_keys);
        
        for (group_key, indices) in sorted_groups {
            // Extract x and y for this group
            let group_x: Vec<f64> = indices.iter().map(|&i| x_values[i]).collect();
            let group_y: Vec<f64> = indices.iter().map(|&i| y_values[i]).collect();
            
            if group_x.len() < 2 {
                continue;  // Need at least 2 points to fit
            }
            
            // Compute range for predictions
            let x_min = group_x.iter().cloned().fold(f64::INFINITY, f64::min);
            let x_max = group_x.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            
            // Generate evenly spaced x values for prediction
            let x_pred: Vec<f64> = (0..self.n)
                .map(|i| x_min + (x_max - x_min) * i as f64 / (self.n - 1) as f64)
                .collect();
            
            // Fit model and compute predictions
            let (y_pred, ymin_pred, ymax_pred, se_pred) = match self.method {
                Method::Lm => {
                    let (intercept, slope, rse) = fit_linear_model(&group_x, &group_y)?;
                    predict_linear_model(&x_pred, &group_x, intercept, slope, rse, self.level)
                }
                Method::Spline => {
                    fit_cubic_spline(&group_x, &group_y, &x_pred, self.level)?
                }
                Method::Loess => {
                    return Err(PlotError::no_valid_data(
                        "Loess smoothing not yet implemented",
                    ));
                }
            };
            
            // Add results for this group
            for i in 0..self.n {
                result_x.push(x_pred[i]);
                result_y.push(y_pred[i]);
                result_ymin.push(ymin_pred[i]);
                result_ymax.push(ymax_pred[i]);
                result_se.push(se_pred[i]);
                result_group_keys.push(group_key.clone());
            }
        }

        // Create output dataframe
        let mut computed = DataFrame::new();
        computed.add_column("x", Box::new(FloatVec(result_x)));
        computed.add_column("y", Box::new(FloatVec(result_y)));
        computed.add_column("ymin", Box::new(FloatVec(result_ymin)));
        computed.add_column("ymax", Box::new(FloatVec(result_ymax)));
        computed.add_column("se", Box::new(FloatVec(result_se)));

        // Add grouping columns back
        use crate::utils::dataframe::StrVec;
        use crate::utils::grouping::split_composite_key;
        
        for (aesthetic, col_name) in &all_group_cols {
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
        new_mapping.set(Aesthetic::Ymax, AesValue::column("ymax"));

        Ok(Some((Box::new(computed), new_mapping)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_regression() {
        // Test with simple linear data: y = 2 + 3x
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![5.0, 8.0, 11.0, 14.0, 17.0];
        
        let (intercept, slope, _rse) = fit_linear_model(&x, &y).unwrap();
        
        assert!((intercept - 2.0).abs() < 1e-10, "Intercept should be 2.0, got {}", intercept);
        assert!((slope - 3.0).abs() < 1e-10, "Slope should be 3.0, got {}", slope);
    }
    
    #[test]
    fn test_linear_regression_with_noise() {
        // Test with noisy data
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![5.1, 7.9, 11.2, 13.8, 17.1];
        
        let (intercept, slope, rse) = fit_linear_model(&x, &y).unwrap();
        
        // Should be close to y = 2 + 3x
        assert!((intercept - 2.0).abs() < 0.5, "Intercept should be close to 2.0");
        assert!((slope - 3.0).abs() < 0.5, "Slope should be close to 3.0");
        assert!(rse > 0.0, "RSE should be positive");
    }
}
