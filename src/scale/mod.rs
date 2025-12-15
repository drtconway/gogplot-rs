use crate::{
    data::{ContinuousType, DiscreteType},
    error::PlotError,
    utils::{
        data::{ContinuousVectorVisitor, DiscreteVectorVisitor, Vectorable},
        set::DiscreteSet,
    },
};

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
pub mod positional;
pub mod shape;
pub mod size;

#[derive(Debug, Clone)]
pub struct ScaleSet {
    pub x_continuous: positional::ContinuousPositionalScale,
    pub x_discrete: positional::DiscretePositionalScale,
    pub y_continuous: positional::ContinuousPositionalScale,
    pub y_discrete: positional::DiscretePositionalScale,
    pub color_continuous: color::ContinuousColorScale,
    pub color_discrete: color::DiscreteColorScale,
    pub fill_continuous: color::ContinuousColorScale,
    pub fill_discrete: color::DiscreteColorScale,
    pub shape_scale: shape::ShapeScale,
    pub alpha_scale: positional::ContinuousPositionalScale,
    pub size_continuous: size::ContinuousSizeScale,
    pub size_discrete: size::DiscreteSizeScale,
}

impl Default for ScaleSet {
    fn default() -> Self {
        Self {
            x_continuous: positional::ContinuousPositionalScale::default(),
            x_discrete: positional::DiscretePositionalScale::default(),
            y_continuous: positional::ContinuousPositionalScale::default(),
            y_discrete: positional::DiscretePositionalScale::default(),
            color_continuous: color::ContinuousColorScale::default(),
            color_discrete: color::DiscreteColorScale::default(),
            fill_continuous: color::ContinuousColorScale::default(),
            fill_discrete: color::DiscreteColorScale::default(),
            shape_scale: shape::ShapeScale::default(),
            alpha_scale: positional::ContinuousPositionalScale::default(),
            size_continuous: size::ContinuousSizeScale::default(),
            size_discrete: size::DiscreteSizeScale::default(),
        }
    }
}

pub(crate) struct ContinuousScaleTrainer {
    pub bounds: Option<(f64, f64)>,
}

impl ContinuousScaleTrainer {
    pub fn new() -> Self {
        Self { bounds: None }
    }
}

impl ContinuousVectorVisitor for ContinuousScaleTrainer {
    type Output = ();

    fn visit<T: Vectorable + ContinuousType>(
        &mut self,
        value: impl Iterator<Item = T>,
    ) -> std::result::Result<Self::Output, PlotError> {
        let mut min_value = f64::INFINITY;
        let mut max_value = f64::NEG_INFINITY;
        for v in value {
            let v_f64 = v.to_f64();
            if v_f64 < min_value {
                min_value = v_f64;
            }
            if v_f64 > max_value {
                max_value = v_f64;
            }
        }
        if min_value != f64::INFINITY || max_value != f64::NEG_INFINITY {
            self.bounds = Some((min_value, max_value));
        }
        Ok(())
    }
}

pub(crate) struct DiscreteScaleTrainer {
    pub categories: DiscreteSet,
}

impl DiscreteScaleTrainer {
    pub fn new() -> Self {
        Self {
            categories: DiscreteSet::new(),
        }
    }
}

impl DiscreteVectorVisitor for DiscreteScaleTrainer {
    type Output = ();

    fn visit<T: Vectorable + DiscreteType>(
        &mut self,
        value: impl Iterator<Item = T>,
    ) -> std::result::Result<Self::Output, PlotError> {
        for v in value {
            self.categories.add(&v);
        }
        Ok(())
    }
}
