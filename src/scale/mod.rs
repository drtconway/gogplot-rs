use crate::aesthetics::AestheticDomain;
use crate::data::{ContinuousType, DiscreteType, GenericVector};
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

/// Information about categorical scales that is preserved when converting to continuous.
///
/// This allows position adjustments like Dodge to access the original categorical
/// scale's geometric properties even after it has been converted to a continuous
/// scale for normalization.
#[derive(Debug, Clone, Copy)]
pub struct CategoricalInfo {
    /// The width of each category in normalized [0, 1] space
    pub category_width: f64,
    /// The padding fraction applied to each category (e.g., 0.1 = 10% padding on each side)
    pub padding: f64,
    /// The number of categories in the original scale
    pub n_categories: usize,
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

pub trait PositionalScale: ScaleBase {
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

/// Scales that map to continuous [0, 1] normalized coordinates.
///
/// Used for position (x, y), size, alpha, and other continuous aesthetics.
/// These scales transform data values to normalized [0, 1] space, which
/// the rendering layer then maps to actual viewport coordinates.
pub trait ContinuousPositionalScale: PositionalScale {
    /// Map a value from the data domain to normalized [0, 1] coordinates.
    ///
    /// # Arguments
    /// * `value` - A value in the data domain
    ///
    /// # Returns
    /// * `Some(normalized_value)` - The corresponding value in [0, 1] range
    /// * `None` - If the value is outside the scale's domain bounds (will be filtered out)
    fn map_value<T: ContinuousType>(&self, value: &T) -> Option<f64>;
}

pub trait DiscretePositionalScale: PositionalScale {
    fn len(&self) -> usize;

    fn ordinal<T: DiscreteType>(&self, value: &T) -> Option<usize>;

    /// Map a discrete category to normalized [0, 1] coordinates.
    ///
    /// # Arguments
    /// * `category` - A category name/value
    ///
    /// # Returns
    /// * `Some(normalized_value)` - The corresponding value in [0, 1] range
    /// * `None` - If the category is not in the scale's domain
    fn map_value<T: DiscreteType>(&self, value: &T) -> Option<f64>;
}

pub trait ColorScale: ScaleBase {
    fn aesthetic_domain(&self) -> AestheticDomain;

    fn breaks(&self) -> &[Color];

    fn labels(&self) -> &[String];
}

/// Scales that map data values to colors.
///
/// Can handle both continuous domains (gradients) and discrete domains (palettes).
/// The implementation determines whether it accepts continuous or categorical input.
pub trait ContinuousColorScale: ScaleBase {
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
    fn map_value<T: ContinuousType>(&self, value: &T) -> Option<Color>;

    fn domain(&self) -> Option<(f64, f64)>;
}


pub trait DiscreteColorScale: ScaleBase {
    /// Map discrete categories to colors.
    ///
    /// Used for categorical color scales where each category maps to a distinct color.
    ///
    /// # Arguments
    /// * `category` - A category name/value
    ///
    /// # Returns
    /// * `Some(color)` - The corresponding color for this category
    /// * `None` - If the category is not in the scale's domain
    fn map_value<T: DiscreteType>(&self, value: &T) -> Option<Color>;
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
    fn map_value<T: DiscreteType>(&self, value: &T) -> Option<Shape>;

    fn breaks(&self) -> &[Shape];

    fn labels(&self) -> &[String];
}

pub mod color;
pub mod continuous;
pub mod discrete;
pub mod shape;
