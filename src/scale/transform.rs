//! Scale transformations
//!
//! This module provides a trait and implementations for scale transformations,
//! following the ggplot2/scales package design where transformations are
//! separate, composable objects that can be plugged into scales.
//!
//! A transformation bundles together:
//! - `transform()`: Maps from data space to transformed space
//! - `inverse()`: Maps from transformed space back to data space
//! - `domain()`: Valid input range for the transformation
//! - `breaks()`: Generates sensible break points in the transformed space
//! - `format()`: Formats values for display
//!
//! # Example
//!
//! ```ignore
//! use gogplot::scale::transform::{Transform, Log10Transform};
//!
//! let trans = Log10Transform;
//! let x = 100.0;
//! let transformed = trans.transform(x);  // 2.0 (log10(100))
//! let back = trans.inverse(transformed); // 100.0
//! ```

/// A transformation that can be applied to scale data.
///
/// Transformations are applied to data values before they are mapped to
/// the plot coordinate system. They are reversible, meaning you can transform
/// data forward and then back to the original space.
///
/// # Transformation Pipeline
///
/// 1. **Data → Transformed Space**: `transform(x)` is called on raw data values
/// 2. **Scale Operations**: Limits, breaks, and mapping happen in transformed space
/// 3. **Display**: `inverse()` is used to convert breaks back for labeling
///
/// # Domain Constraints
///
/// Each transformation has a valid domain (range of input values). For example:
/// - Log transformations require positive values: domain = (ε, ∞)
/// - Identity transformation accepts all finite values: domain = (-∞, ∞)
///
/// Values outside the domain should either be filtered or handled gracefully.
pub trait Transform: Send + Sync {
    /// Apply the forward transformation to a value.
    ///
    /// Maps from data space to transformed space. For example:
    /// - Log10: `transform(100.0) = 2.0`
    /// - Sqrt: `transform(9.0) = 3.0`
    /// - Identity: `transform(x) = x`
    ///
    /// # Arguments
    ///
    /// * `x` - Value in data space
    ///
    /// # Returns
    ///
    /// Value in transformed space, or NaN if x is outside the valid domain
    fn transform(&self, x: f64) -> f64;

    /// Apply the inverse transformation to a value.
    ///
    /// Maps from transformed space back to data space. This is used to:
    /// - Convert break positions back to data values for labels
    /// - Ensure round-trip consistency: `inverse(transform(x)) ≈ x`
    ///
    /// # Arguments
    ///
    /// * `x` - Value in transformed space
    ///
    /// # Returns
    ///
    /// Value in data space
    fn inverse(&self, x: f64) -> f64;

    /// Get the valid domain (input range) for this transformation.
    ///
    /// Values outside this domain may produce NaN or infinite results.
    ///
    /// # Returns
    ///
    /// A tuple (min, max) representing the valid domain.
    /// Use f64::NEG_INFINITY or f64::INFINITY for unbounded domains.
    ///
    /// # Examples
    ///
    /// - Identity: `(-∞, ∞)` - accepts all finite values
    /// - Log10: `(0.001, ∞)` - requires positive values (with small positive lower bound)
    /// - Sqrt: `(0.0, ∞)` - requires non-negative values
    fn domain(&self) -> (f64, f64);

    /// Generate sensible break positions in transformed space.
    ///
    /// This method generates break points that are appropriate for the
    /// transformation. For example:
    /// - Log scales: breaks at powers of 10 (1, 10, 100, 1000)
    /// - Linear scales: evenly spaced breaks
    /// - Sqrt scales: breaks that look good when square-rooted
    ///
    /// # Arguments
    ///
    /// * `limits` - The data limits (min, max) in data space
    /// * `n` - Desired number of breaks (hint - actual number may differ)
    ///
    /// # Returns
    ///
    /// A vector of break positions in data space
    fn breaks(&self, limits: (f64, f64), n: usize) -> Vec<f64>;

    /// Format a value for display (e.g., axis labels).
    ///
    /// Provides transformation-specific formatting. For example:
    /// - Log scales might show "10³" or "1000"
    /// - Linear scales might show "1.5" or "1,500"
    ///
    /// # Arguments
    ///
    /// * `x` - Value in data space
    ///
    /// # Returns
    ///
    /// A formatted string representation of the value
    fn format(&self, x: f64) -> String {
        // Default implementation: simple decimal formatting
        if x.abs() < 1e-10 {
            "0".to_string()
        } else if x.abs() < 0.001 || x.abs() > 9999.0 {
            format!("{:.2e}", x)
        } else if (x.round() - x).abs() < 1e-10 {
            format!("{:.0}", x)
        } else {
            format!("{:.2}", x)
        }
    }

    /// Get a human-readable name for this transformation.
    ///
    /// Used for error messages, debugging, and documentation.
    ///
    /// # Returns
    ///
    /// A string identifier for this transformation (e.g., "log10", "sqrt", "identity")
    fn name(&self) -> &str;

    /// Check if a value is within the valid domain.
    ///
    /// # Arguments
    ///
    /// * `x` - Value to check
    ///
    /// # Returns
    ///
    /// `true` if x is within the domain, `false` otherwise
    fn is_in_domain(&self, x: f64) -> bool {
        let (min, max) = self.domain();
        x >= min && x <= max && x.is_finite()
    }

    /// Clone this transformation into a Box.
    ///
    /// This is needed because Transform is a trait object and we need
    /// to support cloning scales that contain transformations.
    fn box_clone(&self) -> Box<dyn Transform>;
}

// Implement Clone for Box<dyn Transform> using the box_clone method
impl Clone for Box<dyn Transform> {
    fn clone(&self) -> Box<dyn Transform> {
        self.box_clone()
    }
}

/// Helper function to compute nice breaks using Wilkinson's Extended algorithm.
///
/// This is used as a fallback when transformations don't provide their own
/// break generation logic.
pub(crate) fn compute_breaks(min: f64, max: f64, n: usize) -> Vec<f64> {
    if !min.is_finite() || !max.is_finite() || min >= max || n == 0 {
        return vec![];
    }

    // Use Wilkinson's Extended algorithm (simplified version)
    let range = max - min;
    let target_step = range / (n as f64);

    // Find a "nice" step size (1, 2, or 5 times a power of 10)
    let magnitude = 10_f64.powf(target_step.log10().floor());
    let normalized = target_step / magnitude;

    let nice_step = magnitude
        * if normalized < 1.5 {
            1.0
        } else if normalized < 3.0 {
            2.0
        } else if normalized < 7.0 {
            5.0
        } else {
            10.0
        };

    // Generate breaks
    let start = (min / nice_step).floor() * nice_step;
    let mut breaks = vec![];
    let mut value = start;

    while value <= max + nice_step * 0.01 {
        // Small tolerance for floating point
        if value >= min - nice_step * 0.01 {
            breaks.push(value);
        }
        value += nice_step;
    }

    breaks
}

// ============================================================================
// Concrete Transform Implementations
// ============================================================================

/// Identity transformation: f(x) = x
///
/// This is the default transformation that leaves values unchanged.
/// It's used for standard linear scales.
///
/// # Example
///
/// ```ignore
/// let trans = IdentityTransform;
/// assert_eq!(trans.transform(5.0), 5.0);
/// assert_eq!(trans.inverse(5.0), 5.0);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct IdentityTransform;

impl Transform for IdentityTransform {
    fn transform(&self, x: f64) -> f64 {
        x
    }

    fn inverse(&self, x: f64) -> f64 {
        x
    }

    fn domain(&self) -> (f64, f64) {
        (f64::NEG_INFINITY, f64::INFINITY)
    }

    fn breaks(&self, limits: (f64, f64), n: usize) -> Vec<f64> {
        compute_breaks(limits.0, limits.1, n)
    }

    fn name(&self) -> &str {
        "identity"
    }

    fn box_clone(&self) -> Box<dyn Transform> {
        Box::new(*self)
    }
}

/// Square root transformation: f(x) = √x
///
/// Useful for data with moderate right skew. Compresses large values
/// more than small values, making patterns more visible.
///
/// # Domain
///
/// Requires non-negative values: [0, ∞)
///
/// # Example
///
/// ```ignore
/// let trans = SqrtTransform;
/// assert_eq!(trans.transform(9.0), 3.0);
/// assert_eq!(trans.inverse(3.0), 9.0);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct SqrtTransform;

impl Transform for SqrtTransform {
    fn transform(&self, x: f64) -> f64 {
        if x < 0.0 { f64::NAN } else { x.sqrt() }
    }

    fn inverse(&self, x: f64) -> f64 {
        x * x
    }

    fn domain(&self) -> (f64, f64) {
        (0.0, f64::INFINITY)
    }

    fn breaks(&self, limits: (f64, f64), n: usize) -> Vec<f64> {
        // Transform limits to sqrt space, compute breaks, then transform back
        let (min, max) = limits;
        if min < 0.0 {
            return vec![];
        }

        // Compute breaks in transformed (sqrt) space
        let sqrt_breaks = compute_breaks(min.sqrt(), max.sqrt(), n);

        // Transform back to data space
        sqrt_breaks.into_iter().map(|b| b * b).collect()
    }

    fn format(&self, x: f64) -> String {
        if x < 0.0 {
            "NA".to_string()
        } else if x.abs() < 0.001 {
            format!("{:.2e}", x)
        } else if x < 1.0 {
            format!("{:.3}", x)
        } else if x < 100.0 {
            format!("{:.1}", x)
        } else {
            format!("{:.0}", x)
        }
    }

    fn name(&self) -> &str {
        "sqrt"
    }

    fn box_clone(&self) -> Box<dyn Transform> {
        Box::new(*self)
    }
}

/// Base-10 logarithmic transformation: f(x) = log₁₀(x)
///
/// Ideal for data spanning multiple orders of magnitude. Makes
/// multiplicative relationships appear linear.
///
/// # Domain
///
/// Requires positive values. We use a small positive lower bound (0.001)
/// to handle values very close to zero gracefully.
///
/// # Break Generation
///
/// Generates breaks at powers of 10 (1, 10, 100, 1000, etc.) which
/// are natural for logarithmic scales.
///
/// # Example
///
/// ```ignore
/// let trans = Log10Transform;
/// assert_eq!(trans.transform(100.0), 2.0);
/// assert_eq!(trans.inverse(2.0), 100.0);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Log10Transform;

impl Transform for Log10Transform {
    fn transform(&self, x: f64) -> f64 {
        if x <= 0.0 { f64::NAN } else { x.log10() }
    }

    fn inverse(&self, x: f64) -> f64 {
        10_f64.powf(x)
    }

    fn domain(&self) -> (f64, f64) {
        // Log is undefined for non-positive values
        // Use a very small positive value to support tiny p-values
        (1e-300, f64::INFINITY)
    }

    fn breaks(&self, limits: (f64, f64), n: usize) -> Vec<f64> {
        let (min, max) = limits;
        if min <= 0.0 || max <= 0.0 {
            return vec![];
        }

        // For log scales, we want breaks at powers of 10
        let log_min = min.log10().floor();
        let log_max = max.log10().ceil();

        let mut breaks = vec![];
        let mut exp = log_min;

        // Generate breaks at powers of 10
        while exp <= log_max {
            let value = 10_f64.powf(exp);
            if value >= min && value <= max {
                breaks.push(value);
            }
            exp += 1.0;
        }

        // If we have too few breaks, add intermediate values (2, 5, etc.)
        if breaks.len() < n.min(3) {
            let mut enhanced_breaks = vec![];
            exp = log_min;
            while exp <= log_max {
                let base = 10_f64.powf(exp);
                for mult in [1.0, 2.0, 5.0] {
                    let value = base * mult;
                    if value >= min && value <= max {
                        enhanced_breaks.push(value);
                    }
                }
                exp += 1.0;
            }
            enhanced_breaks.sort_by(|a, b| a.partial_cmp(b).unwrap());
            enhanced_breaks.dedup();
            return enhanced_breaks;
        }

        breaks
    }

    fn format(&self, x: f64) -> String {
        if x <= 0.0 {
            "NA".to_string()
        } else if x >= 1000.0 {
            format!("{:.0}", x)
        } else if x >= 1.0 {
            format!("{:.0}", x)
        } else if x >= 0.01 {
            format!("{:.2}", x)
        } else {
            format!("{:.2e}", x)
        }
    }

    fn name(&self) -> &str {
        "log10"
    }

    fn box_clone(&self) -> Box<dyn Transform> {
        Box::new(*self)
    }
}

/// Reverse transformation: f(x) = -x
///
/// Flips the direction of the scale, making high values appear
/// on the left/bottom and low values on the right/top.
///
/// # Example
///
/// ```ignore
/// let trans = ReverseTransform;
/// assert_eq!(trans.transform(5.0), -5.0);
/// assert_eq!(trans.inverse(-5.0), 5.0);
/// ```
#[derive(Debug, Clone, Copy)]
pub struct ReverseTransform;

impl Transform for ReverseTransform {
    fn transform(&self, x: f64) -> f64 {
        -x
    }

    fn inverse(&self, x: f64) -> f64 {
        -x
    }

    fn domain(&self) -> (f64, f64) {
        (f64::NEG_INFINITY, f64::INFINITY)
    }

    fn breaks(&self, limits: (f64, f64), n: usize) -> Vec<f64> {
        // Reverse the limits, compute breaks, then reverse back
        let (min, max) = limits;
        let breaks = compute_breaks(min, max, n);
        // Breaks are already in the correct order for the data space
        breaks
    }

    fn name(&self) -> &str {
        "reverse"
    }

    fn box_clone(&self) -> Box<dyn Transform> {
        Box::new(*self)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        aesthetics::builder::{XContinuousAesBuilder, YContinuousAesBuilder}, data::DataSource, error::to_io_error, geom::point::geom_point, plot::plot, scale::{scale_x_continuous, scale_y_continuous}, utils::dataframe::DataFrame
    };

    use super::*;

    #[test]
    fn test_compute_breaks() {
        let breaks = compute_breaks(0.0, 100.0, 5);
        assert!(!breaks.is_empty());
        assert!(breaks[0] >= 0.0);
        assert!(*breaks.last().unwrap() <= 100.0);
    }

    #[test]
    fn test_compute_breaks_small_range() {
        let breaks = compute_breaks(0.0, 1.0, 5);
        assert!(!breaks.is_empty());
        // Should generate breaks like 0.0, 0.2, 0.4, 0.6, 0.8, 1.0
    }

    #[test]
    fn test_identity_transform() {
        let trans = IdentityTransform;
        assert_eq!(trans.transform(5.0), 5.0);
        assert_eq!(trans.inverse(5.0), 5.0);
        assert_eq!(trans.name(), "identity");
        assert!(trans.is_in_domain(0.0));
        assert!(trans.is_in_domain(-100.0));
        assert!(trans.is_in_domain(100.0));
    }

    #[test]
    fn test_sqrt_transform() {
        let trans = SqrtTransform;
        assert_eq!(trans.transform(9.0), 3.0);
        assert_eq!(trans.inverse(3.0), 9.0);
        assert_eq!(trans.name(), "sqrt");
        assert!(trans.is_in_domain(0.0));
        assert!(trans.is_in_domain(100.0));
        assert!(!trans.is_in_domain(-1.0));
        assert!(trans.transform(-1.0).is_nan());
    }

    #[test]
    fn test_log10_transform() {
        let trans = Log10Transform;
        assert_eq!(trans.transform(100.0), 2.0);
        assert_eq!(trans.inverse(2.0), 100.0);
        assert_eq!(trans.name(), "log10");
        assert!(trans.is_in_domain(1.0));
        assert!(trans.is_in_domain(0.01)); // Above lower bound
        assert!(!trans.is_in_domain(0.0));
        assert!(!trans.is_in_domain(-1.0));
        assert!(trans.transform(0.0).is_nan());
        assert!(trans.transform(-1.0).is_nan());
    }

    #[test]
    fn test_log10_breaks() {
        let trans = Log10Transform;
        let breaks = trans.breaks((1.0, 1000.0), 5);
        assert!(!breaks.is_empty());
        // Should contain powers of 10: 1, 10, 100, 1000
        assert!(breaks.contains(&1.0));
        assert!(breaks.contains(&10.0));
        assert!(breaks.contains(&100.0));
        assert!(breaks.contains(&1000.0));
    }

    #[test]
    fn test_reverse_transform() {
        let trans = ReverseTransform;
        assert_eq!(trans.transform(5.0), -5.0);
        assert_eq!(trans.inverse(-5.0), 5.0);
        assert_eq!(trans.name(), "reverse");
        // Reverse is its own inverse
        let x = 42.0;
        assert_eq!(trans.inverse(trans.transform(x)), x);
    }

    #[test]
    fn test_sqrt_breaks() {
        let trans = SqrtTransform;
        let breaks = trans.breaks((0.0, 100.0), 5);
        assert!(!breaks.is_empty());
        assert!(breaks[0] >= 0.0);
        assert!(*breaks.last().unwrap() <= 100.0);
    }

    // ========================================================================
    // Integration tests with actual plots
    // ========================================================================

    #[test]
    fn test_log10_transform_with_plot() {
        // Create data spanning multiple orders of magnitude (ideal for log scale)
        let x = vec![1.0, 10.0, 100.0, 1000.0, 10000.0];
        let y = vec![1.0, 100.0, 10000.0, 1000000.0, 100000000.0];

        let data: Box<dyn DataSource> =
            Box::new(DataFrame::from_columns(vec![("x", x), ("y", y)]));

        let builder = plot(&data).aes(|a| {
            a.x_continuous("x");
            a.y_continuous("y");
        }) + geom_point().size(3.0).alpha(0.5)
            + scale_x_continuous().transform(Box::new(Log10Transform))
            + scale_y_continuous().transform(Box::new(Log10Transform));

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/transform_points_log10.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn test_sqrt_transform_with_plot() {
        // Create data with moderate right skew (good for sqrt transform)
        let x = vec![0.0, 1.0, 4.0, 9.0, 16.0, 25.0, 36.0, 49.0, 64.0, 81.0, 100.0];
        let y = vec![0.0, 1.0, 4.0, 9.0, 16.0, 25.0, 36.0, 49.0, 64.0, 81.0, 100.0];

        let data: Box<dyn DataSource> =
            Box::new(DataFrame::from_columns(vec![("x", x), ("y", y)]));

        let builder = plot(&data).aes(|a| {
            a.x_continuous("x");
            a.y_continuous("y");
        }) + geom_point().size(4.0).alpha(0.7)
            + scale_x_continuous().transform(Box::new(SqrtTransform))
            + scale_y_continuous().transform(Box::new(SqrtTransform));

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/transform_points_sqrt.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }
}
