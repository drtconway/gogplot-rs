//! Smooth statistics for fitted curves with confidence intervals
//!
//! Computes fitted values and confidence intervals using various smoothing methods.

use crate::aesthetics::{AesMap, AesValue, Aesthetic, AestheticDomain};
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
    /// Local polynomial regression (LOESS)
    Loess,
}

/// Smooth statistics computation
///
/// Fits a curve to the data and computes confidence intervals.
/// Currently supports:
/// - `lm`: Linear regression (y = a + bx)
/// - `spline`: Cubic spline interpolation
/// - `loess`: Local polynomial regression (locally weighted scatterplot smoothing)
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
    
    /// Span for LOESS smoothing (default 0.75)
    /// Controls the amount of smoothing - smaller values = more local (wigglier)
    pub span: f64,
}

impl Smooth {
    /// Create a new Smooth stat with default parameters
    pub fn new() -> Self {
        Self {
            method: Method::Loess,
            level: 0.95,
            n: 80,
            se: true,
            span: 0.75,
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
    
    /// Set the span for LOESS smoothing (0.0 to 1.0)
    /// Smaller values produce wigglier curves, larger values produce smoother curves
    pub fn span(mut self, span: f64) -> Self {
        self.span = span.clamp(0.0, 1.0);
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

/// Compute LOESS (Locally Estimated Scatterplot Smoothing)
/// Returns (y_pred, ymin, ymax, se)
fn fit_loess(
    x_data: &[f64],
    y_data: &[f64],
    x_pred: &[f64],
    span: f64,
    level: f64,
) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>)> {
    let n = x_data.len();
    if n < 3 {
        return Err(PlotError::no_valid_data(
            "Need at least 3 points for LOESS smoothing",
        ));
    }
    
    let span = span.clamp(0.0, 1.0);
    let q = ((n as f64 * span).max(2.0) as usize).min(n);
    
    // Sort data by x
    let mut paired: Vec<(f64, f64)> = x_data.iter().zip(y_data.iter())
        .map(|(&x, &y)| (x, y))
        .collect();
    paired.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    
    let x_sorted: Vec<f64> = paired.iter().map(|(x, _)| *x).collect();
    let y_sorted: Vec<f64> = paired.iter().map(|(_, y)| *y).collect();
    
    // Compute predictions using local weighted linear regression
    let mut y_pred = Vec::with_capacity(x_pred.len());
    let mut residuals_at_pred = Vec::with_capacity(x_pred.len());
    
    for &x0 in x_pred {
        // Find q nearest neighbors
        let mut distances: Vec<(usize, f64)> = x_sorted.iter().enumerate()
            .map(|(i, &x)| (i, (x - x0).abs()))
            .collect();
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        // Get the q nearest points
        let neighbors = &distances[..q];
        let max_dist = neighbors.last().unwrap().1.max(1e-10);
        
        // Compute tricube weights: w(u) = (1 - |u|^3)^3 for |u| < 1
        let weights: Vec<f64> = neighbors.iter()
            .map(|(_, d)| {
                let u = d / max_dist;
                if u < 1.0 {
                    let t = 1.0 - u.powi(3);
                    t.powi(3)
                } else {
                    0.0
                }
            })
            .collect();
        
        // Fit weighted linear regression: y = a + b*x
        let mut sum_w = 0.0;
        let mut sum_wx = 0.0;
        let mut sum_wy = 0.0;
        let mut sum_wxx = 0.0;
        let mut sum_wxy = 0.0;
        
        for (j, &(i, _)) in neighbors.iter().enumerate() {
            let w = weights[j];
            let x = x_sorted[i];
            let y = y_sorted[i];
            
            sum_w += w;
            sum_wx += w * x;
            sum_wy += w * y;
            sum_wxx += w * x * x;
            sum_wxy += w * x * y;
        }
        
        // Solve for slope and intercept
        let denom = sum_w * sum_wxx - sum_wx * sum_wx;
        let (a, b) = if denom.abs() > 1e-10 {
            let b = (sum_w * sum_wxy - sum_wx * sum_wy) / denom;
            let a = (sum_wy - b * sum_wx) / sum_w;
            (a, b)
        } else {
            // If denominator is too small, just use weighted mean
            (sum_wy / sum_w, 0.0)
        };
        
        let y_fit = a + b * x0;
        y_pred.push(y_fit);
        
        // Estimate local residual variance for this prediction
        let mut local_var = 0.0;
        for (j, &(i, _)) in neighbors.iter().enumerate() {
            let w = weights[j];
            let x = x_sorted[i];
            let y = y_sorted[i];
            let residual = y - (a + b * x);
            local_var += w * residual * residual;
        }
        local_var /= sum_w;
        residuals_at_pred.push(local_var.sqrt());
    }
    
    // Compute global residual standard error
    let mut all_residuals = Vec::with_capacity(n);
    for i in 0..n {
        let x = x_sorted[i];
        let y = y_sorted[i];
        
        // Fit at this point (same as above but for data point)
        let mut distances: Vec<(usize, f64)> = x_sorted.iter().enumerate()
            .map(|(j, &xj)| (j, (xj - x).abs()))
            .collect();
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        let neighbors = &distances[..q];
        let max_dist = neighbors.last().unwrap().1.max(1e-10);
        
        let weights: Vec<f64> = neighbors.iter()
            .map(|(_, d)| {
                let u = d / max_dist;
                if u < 1.0 {
                    let t = 1.0 - u.powi(3);
                    t.powi(3)
                } else {
                    0.0
                }
            })
            .collect();
        
        let mut sum_w = 0.0;
        let mut sum_wx = 0.0;
        let mut sum_wy = 0.0;
        let mut sum_wxx = 0.0;
        let mut sum_wxy = 0.0;
        
        for (j, &(idx, _)) in neighbors.iter().enumerate() {
            let w = weights[j];
            let xj = x_sorted[idx];
            let yj = y_sorted[idx];
            
            sum_w += w;
            sum_wx += w * xj;
            sum_wy += w * yj;
            sum_wxx += w * xj * xj;
            sum_wxy += w * xj * yj;
        }
        
        let denom = sum_w * sum_wxx - sum_wx * sum_wx;
        let (a, b) = if denom.abs() > 1e-10 {
            let b = (sum_w * sum_wxy - sum_wx * sum_wy) / denom;
            let a = (sum_wy - b * sum_wx) / sum_w;
            (a, b)
        } else {
            (sum_wy / sum_w, 0.0)
        };
        
        let y_fit = a + b * x;
        all_residuals.push(y - y_fit);
    }
    
    let rse = (all_residuals.iter().map(|r| r.powi(2)).sum::<f64>() / (n - 2) as f64).sqrt();
    
    // Use t-value approximation for confidence bands
    let z = match level {
        l if l >= 0.99 => 2.576,
        l if l >= 0.95 => 1.96,
        l if l >= 0.90 => 1.645,
        _ => 1.96,
    };
    
    // Combine global and local uncertainty
    let se_vec: Vec<f64> = residuals_at_pred.iter()
        .map(|&local_se| (rse.powi(2) + local_se.powi(2)).sqrt())
        .collect();
    
    let ymin: Vec<f64> = y_pred.iter().zip(se_vec.iter())
        .map(|(&y, &se)| y - z * se)
        .collect();
    let ymax: Vec<f64> = y_pred.iter().zip(se_vec.iter())
        .map(|(&y, &se)| y + z * se)
        .collect();
    
    Ok((y_pred, ymin, ymax, se_vec))
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
    
    // For smoothing, use fewer knots than data points
    // Use approximately 12-18 evenly spaced knots across the range
    let num_knots = (n / 17).clamp(12, 18);
    let mut smoothed_keys: Vec<(f64, f64)> = Vec::new();
    
    let x_min = paired[0].0;
    let x_max = paired[paired.len() - 1].0;
    let x_range = x_max - x_min;
    
    // Add extra knot near the start for some boundary flexibility
    smoothed_keys.push(paired[0]);
    
    // Add one point near the start (within first 5% of range)
    let target_x = x_min + x_range * 0.05;
    let idx = paired.iter()
        .enumerate()
        .min_by(|(_, (x1, _)), (_, (x2, _))| {
            (x1 - target_x).abs().partial_cmp(&(x2 - target_x).abs()).unwrap()
        })
        .map(|(i, _)| i)
        .unwrap();
    if !smoothed_keys.contains(&paired[idx]) {
        smoothed_keys.push(paired[idx]);
    }
    
    // Add evenly spaced knots across the range
    for i in 1..num_knots {
        let frac = i as f64 / num_knots as f64;
        let target_x = x_min + x_range * frac;
        
        let idx = paired.iter()
            .enumerate()
            .min_by(|(_, (x1, _)), (_, (x2, _))| {
                (x1 - target_x).abs().partial_cmp(&(x2 - target_x).abs()).unwrap()
            })
            .map(|(i, _)| i)
            .unwrap();
        
        if !smoothed_keys.contains(&paired[idx]) {
            smoothed_keys.push(paired[idx]);
        }
    }
    
    // Add one point near the end (within last 5% of range)
    let target_x = x_min + x_range * 0.95;
    let idx = paired.iter()
        .enumerate()
        .min_by(|(_, (x1, _)), (_, (x2, _))| {
            (x1 - target_x).abs().partial_cmp(&(x2 - target_x).abs()).unwrap()
        })
        .map(|(i, _)| i)
        .unwrap();
    if !smoothed_keys.contains(&paired[idx]) {
        smoothed_keys.push(paired[idx]);
    }
    
    // Ensure last point is included
    if smoothed_keys.last() != Some(&paired[paired.len() - 1]) {
        smoothed_keys.push(paired[paired.len() - 1]);
    }
    
    // Sort by x value to ensure proper spline construction
    smoothed_keys.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    
    // Create spline keys from smoothed points
    let keys: Vec<Key<f64, f64>> = smoothed_keys
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
    
    // Compute confidence intervals using local residual variance
    // Similar to LOESS approach but using spline predictions
    
    let x_sorted: Vec<f64> = paired.iter().map(|(x, _)| *x).collect();
    let y_sorted: Vec<f64> = paired.iter().map(|(_, y)| *y).collect();
    
    // Choose neighborhood size (similar to LOESS span concept)
    // Use about 50% of data for local variance estimation (wider than LOESS to account for smoothing)
    let q = ((n as f64 * 0.5).max(3.0) as usize).min(n);
    
    // Compute local variance at each prediction point
    let mut residuals_at_pred = Vec::with_capacity(x_pred.len());
    
    for &x0 in x_pred {
        // Find q nearest neighbors
        let mut distances: Vec<(usize, f64)> = x_sorted.iter().enumerate()
            .map(|(i, &x)| (i, (x - x0).abs()))
            .collect();
        distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        
        let neighbors = &distances[..q];
        let max_dist = neighbors.last().unwrap().1.max(1e-10);
        
        // Compute tricube weights for local variance
        let weights: Vec<f64> = neighbors.iter()
            .map(|(_, d)| {
                let u = d / max_dist;
                if u < 1.0 {
                    let t = 1.0 - u.powi(3);
                    t.powi(3)
                } else {
                    0.0
                }
            })
            .collect();
        
        // Compute weighted local variance
        let mut sum_w = 0.0;
        let mut local_var = 0.0;
        
        for (j, &(i, _)) in neighbors.iter().enumerate() {
            let w = weights[j];
            let x = x_sorted[i];
            let y = y_sorted[i];
            
            // Get spline prediction at this data point
            if let Some(y_fit) = spline.sample(x) {
                let residual = y - y_fit;
                local_var += w * residual * residual;
                sum_w += w;
            }
        }
        
        if sum_w > 0.0 {
            local_var /= sum_w;
        }
        
        residuals_at_pred.push(local_var.sqrt());
    }
    
    // Compute global residual standard error
    let mut all_residuals = Vec::with_capacity(n);
    for (x, y) in &paired {
        if let Some(y_fit) = spline.sample(*x) {
            all_residuals.push(y - y_fit);
        }
    }
    
    let rse = (all_residuals.iter().map(|r| r.powi(2)).sum::<f64>() / (n - 4) as f64).sqrt();
    
    // Use t-value approximation for confidence bands
    let z = match level {
        l if l >= 0.99 => 2.576,
        l if l >= 0.95 => 1.96,
        l if l >= 0.90 => 1.645,
        _ => 1.96,
    };
    
    // Combine global and local uncertainty (same as LOESS)
    let se_vec: Vec<f64> = residuals_at_pred.iter()
        .map(|&local_se| (rse.powi(2) + local_se.powi(2)).sqrt())
        .collect();
    
    let ymin: Vec<f64> = y_pred.iter().zip(se_vec.iter())
        .map(|(&y, &se)| y - z * se)
        .collect();
    let ymax: Vec<f64> = y_pred.iter().zip(se_vec.iter())
        .map(|(&y, &se)| y + z * se)
        .collect();
    
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
        let x_values = get_numeric_values(data.as_ref(), mapping, Aesthetic::X(AestheticDomain::Continuous))?;
        let y_values = get_numeric_values(data.as_ref(), mapping, Aesthetic::Y(AestheticDomain::Continuous))?;

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
                    fit_loess(&group_x, &group_y, &x_pred, self.span, self.level)?
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
        new_mapping.set(Aesthetic::X(AestheticDomain::Continuous), AesValue::column("x"));
        new_mapping.set(Aesthetic::Y(AestheticDomain::Continuous), AesValue::column("y"));
        new_mapping.set(Aesthetic::Ymin(AestheticDomain::Continuous), AesValue::column("ymin"));
        new_mapping.set(Aesthetic::Ymax(AestheticDomain::Continuous), AesValue::column("ymax"));

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
