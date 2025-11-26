use crate::data::GenericVector;
use crate::theme::Color;
use crate::visuals::Shape;

pub mod transform;

/// Specifies the type of scale required for an aesthetic
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleType {
    /// Aesthetic must use a continuous scale (numeric data)
    Continuous,
    /// Aesthetic must use a categorical/discrete scale
    Categorical,
    /// Aesthetic can use either continuous or categorical (decide based on data type)
    Either,
}

/// Base trait for all scales providing common functionality.
pub trait ScaleBase: Send + Sync {
    /// Train the scale on data to automatically determine the domain.
    ///
    /// This method allows the scale to learn appropriate domain bounds by examining
    /// the data. For continuous scales, this typically computes min/max values
    /// across all provided data sources. For categorical scales, this extracts
    /// unique categories.
    ///
    /// If the scale's domain is already explicitly set (e.g., via a builder),
    /// this method may be a no-op.
    ///
    /// # Arguments
    /// * `data` - A slice of data vectors to train on (e.g., for rectangles this
    ///            would include both xmin and xmax to get the full range)
    fn train(&mut self, data: &[&dyn GenericVector]);
    
    /// Returns the type of this scale (Continuous or Categorical).
    ///
    /// This allows geoms to detect whether they're working with categorical
    /// or continuous data and adjust their rendering accordingly (e.g., bar
    /// widths computed differently for categorical vs continuous x scales).
    fn scale_type(&self) -> ScaleType;
}

/// Scales that map to continuous [0, 1] normalized coordinates.
///
/// Used for position (x, y), size, alpha, and other continuous aesthetics.
/// These scales transform data values to normalized [0, 1] space, which
/// the rendering layer then maps to actual viewport coordinates.
pub trait ContinuousScale: ScaleBase {
    /// Map a value from the data domain to normalized [0, 1] coordinates.
    ///
    /// # Arguments
    /// * `value` - A value in the data domain
    ///
    /// # Returns
    /// * `Some(normalized_value)` - The corresponding value in [0, 1] range
    /// * `None` - If the value is outside the scale's domain bounds (will be filtered out)
    fn map_value(&self, value: f64) -> Option<f64>;

    /// Map a categorical value to a numeric position.
    ///
    /// This is used for categorical scales that need to map string categories to positions.
    /// For purely continuous scales, this should return None.
    ///
    /// # Arguments
    /// * `category` - A category name/value
    /// * `aesthetic` - The aesthetic being mapped (e.g., X, Xmin, Xmax) to determine
    ///                 whether to return the center, left edge, or right edge of the category
    ///
    /// # Returns
    /// * `Some(position)` - The numeric position for this category
    /// * `None` - If this is not a categorical scale or category is not recognized
    fn map_category(&self, _category: &str, _aesthetic: crate::aesthetics::Aesthetic) -> Option<f64> {
        None // Default: not a categorical scale
    }

    /// Map a normalized [0, 1] value back to the data domain (inverse mapping).
    ///
    /// This is useful for interactive features like zooming or reading values from
    /// the plot. It performs the reverse transformation of `map_value`.
    ///
    /// # Arguments
    /// * `value` - A normalized value in [0, 1] range
    ///
    /// # Returns
    /// The corresponding value in the data domain
    fn inverse(&self, value: f64) -> f64;

    /// Get the axis break positions in data coordinates.
    ///
    /// Breaks are the positions where tick marks and grid lines should be drawn.
    /// These are values in the data domain that will be mapped to visual positions.
    ///
    /// # Returns
    /// A slice of break positions in data coordinates
    fn breaks(&self) -> &[f64];

    /// Get the axis labels corresponding to each break.
    ///
    /// Labels are the text displayed at each break position. There should be
    /// one label for each break returned by `breaks()`.
    ///
    /// # Returns
    /// A slice of formatted label strings
    fn labels(&self) -> &[String];
}

/// Scales that map data values to colors.
///
/// Can handle both continuous domains (gradients) and discrete domains (palettes).
/// The implementation determines whether it accepts continuous or categorical input.
pub trait ColorScale: ScaleBase {
    /// Map continuous numeric values to colors.
    ///
    /// Used for gradient color scales where numeric data maps to a color gradient.
    ///
    /// # Arguments
    /// * `value` - A numeric value in the data domain
    ///
    /// # Returns
    /// * `Some(color)` - The corresponding color
    /// * `None` - If the value is outside the scale's domain
    fn map_continuous_to_color(&self, value: f64) -> Option<Color> {
        let _ = value;
        None // Default: not supported
    }

    /// Map discrete categorical values to colors.
    ///
    /// Used for categorical color scales where distinct categories get distinct colors.
    ///
    /// # Arguments
    /// * `category` - A category name/value
    ///
    /// # Returns
    /// * `Some(color)` - The corresponding color for this category
    /// * `None` - If the category is not in the scale's domain
    fn map_discrete_to_color(&self, category: &str) -> Option<Color> {
        let _ = category;
        None // Default: not supported
    }

    /// Get legend breaks as formatted strings.
    fn legend_breaks(&self) -> Vec<String>;

    /// Returns true if this is a continuous color scale (gradient), false if discrete.
    fn is_continuous(&self) -> bool {
        false // Default: discrete
    }

    /// Get the domain for continuous color scales.
    /// Returns None for discrete scales.
    fn get_continuous_domain(&self) -> Option<(f64, f64)> {
        None
    }
}

/// Scales that map data values to point shapes.
///
/// Typically used for discrete/categorical data where each category
/// gets a distinct shape.
pub trait ShapeScale: ScaleBase {
    /// Map categorical values to shapes.
    ///
    /// # Arguments
    /// * `category` - A category name/value
    ///
    /// # Returns
    /// * `Some(shape)` - The corresponding shape for this category
    /// * `None` - If the category is not in the scale's domain
    fn map_to_shape(&self, category: &str) -> Option<Shape>;

    /// Get legend breaks as formatted strings.
    fn legend_breaks(&self) -> Vec<String>;
}

pub mod categorical;
pub mod color;
pub mod continuous;
pub mod shape;
