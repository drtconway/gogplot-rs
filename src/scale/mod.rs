

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

pub mod traits;

pub mod color;
//pub mod continuous;
//pub mod discrete;
pub mod positional;
pub mod shape;



pub struct ScaleSet {
    x_continuous: positional::ContinuousPositionalScale,
    x_discrete: positional::DiscretePositionalScale,
    y_continuous: positional::ContinuousPositionalScale,
    y_discrete: positional::DiscretePositionalScale,
    color_continuous: color::ContinuousColorScale,
    color_discrete: color::DiscreteColorScale,
    fill_continuous: color::ContinuousColorScale,
    fill_discrete: color::DiscreteColorScale,
    shape_scale: shape::ShapeScale,
}
