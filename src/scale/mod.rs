use crate::{
    aesthetics::{AesMap, AesValue, Aesthetic},
    data::{ContinuousType, DataSource, DiscreteType},
    error::PlotError,
    scale::traits::{ColorRangeScale, ContinuousRangeScale, ScaleBase, ShapeRangeScale},
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
pub mod utils;

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
    pub alpha_continuous: positional::ContinuousPositionalScale,
    pub alpha_discrete: positional::DiscretePositionalScale,
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
            alpha_continuous: positional::ContinuousPositionalScale::default(),
            alpha_discrete: positional::DiscretePositionalScale::default(),
            size_continuous: size::ContinuousSizeScale::default(),
            size_discrete: size::DiscreteSizeScale::default(),
        }
    }
}

impl ScaleSet {
    pub fn train(
        &mut self,
        aesthetic: &Aesthetic,
        mapping: &AesMap,
        data: &dyn DataSource,
    ) -> Result<(), PlotError> {
        use crate::aesthetics::AestheticDomain::*;

        let iter = mapping
            .get_vector_iter(aesthetic, data)
            .ok_or(PlotError::MissingAesthetic {
                aesthetic: *aesthetic,
            })?;
        match aesthetic {
            Aesthetic::X(domain) | Aesthetic::Xmin(domain) | Aesthetic::Xmax(domain) => {
                match domain {
                    Continuous => self.x_continuous.train(iter),
                    Discrete => self.x_discrete.train(iter),
                }
            }
            Aesthetic::XIntercept | Aesthetic::XBegin | Aesthetic::XEnd | Aesthetic::XOffset => {
                self.x_continuous.train(iter)
            }
            Aesthetic::Y(domain) | Aesthetic::Ymin(domain) | Aesthetic::Ymax(domain) => {
                match domain {
                    Continuous => self.y_continuous.train(iter),
                    Discrete => self.y_discrete.train(iter),
                }
            }
            Aesthetic::YIntercept | Aesthetic::YBegin | Aesthetic::YEnd | Aesthetic::YOffset => {
                self.y_continuous.train(iter)
            }
            Aesthetic::Lower => self.y_continuous.train(iter),
            Aesthetic::Middle => self.y_continuous.train(iter),
            Aesthetic::Upper => self.y_continuous.train(iter),
            Aesthetic::Color(domain) => match domain {
                Continuous => self.color_continuous.train(iter),
                Discrete => self.color_discrete.train(iter),
            },
            Aesthetic::Fill(domain) => match domain {
                Continuous => self.fill_continuous.train(iter),
                Discrete => self.fill_discrete.train(iter),
            },
            Aesthetic::Alpha(domain) => match domain {
                Continuous => self.alpha_continuous.train(iter),
                Discrete => self.alpha_discrete.train(iter),
            },
            Aesthetic::Size(domain) => match domain {
                Continuous => self.size_continuous.train(iter),
                Discrete => self.size_discrete.train(iter),
            },
            Aesthetic::Shape => self.shape_scale.train(iter),
            Aesthetic::Linetype | Aesthetic::Group | Aesthetic::Label | Aesthetic::Width | Aesthetic::Height => {
                // No scale training needed for these aesthetics
            }
        }
        Ok(())
    }

    pub fn apply(
        &self,
        aesthetic: &Aesthetic,
        value: &AesValue,
        data: &dyn DataSource,
    ) -> Result<AesValue, PlotError> {
        use crate::aesthetics::AestheticDomain::*;
        
        match aesthetic {
            Aesthetic::X(domain) | Aesthetic::Xmin(domain) | Aesthetic::Xmax(domain) => {
                match domain {
                    Continuous => self.x_continuous.map_aesthetic_value(value, data),
                    Discrete => self.x_discrete.map_aesthetic_value(value, data),
                }
            }
            Aesthetic::XIntercept | Aesthetic::XBegin | Aesthetic::XEnd => {
                self.x_continuous.map_aesthetic_value(value, data)
            }
            Aesthetic::Y(domain) | Aesthetic::Ymin(domain) | Aesthetic::Ymax(domain) => {
                match domain {
                    Continuous => self.y_continuous.map_aesthetic_value(value, data),
                    Discrete => self.y_discrete.map_aesthetic_value(value, data),
                }
            }
            Aesthetic::YIntercept | Aesthetic::YBegin | Aesthetic::YEnd | Aesthetic::Lower
            | Aesthetic::Middle | Aesthetic::Upper | Aesthetic::YOffset => {
                self.y_continuous.map_aesthetic_value(value, data)
            }
            Aesthetic::Color(domain) => match domain {
                Continuous => self.color_continuous.map_aesthetic_value(value, data),
                Discrete => self.color_discrete.map_aesthetic_value(value, data),
            },
            Aesthetic::Fill(domain) => match domain {
                Continuous => self.fill_continuous.map_aesthetic_value(value, data),
                Discrete => self.fill_discrete.map_aesthetic_value(value, data),
            },
            Aesthetic::Alpha(domain) => match domain {
                Continuous => self.alpha_continuous.map_aesthetic_value(value, data),
                Discrete => self.alpha_discrete.map_aesthetic_value(value, data),
            },
            Aesthetic::Size(domain) => match domain {
                Continuous => self.size_continuous.map_aesthetic_value(value, data),
                Discrete => self.size_discrete.map_aesthetic_value(value, data),
            },
            Aesthetic::Shape => self.shape_scale.map_aesthetic_value(value, data),
            _ => Ok(value.clone()), // No scaling needed for other aesthetics
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ScaleIdentifier {
    XContinuous,
    XDiscrete,
    YContinuous,
    YDiscrete,
    ColorContinuous,
    ColorDiscrete,
    FillContinuous,
    FillDiscrete,
    Shape,
    Alpha,
    SizeContinuous,
    SizeDiscrete,
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

// ============================================================================
// Scale Builders
// ============================================================================

/// Builder for configuring continuous positional scales (x, y axes)
#[derive(Clone)]
pub struct ContinuousScaleBuilder {
    pub(crate) aesthetic: ScaleAesthetic,
    pub(crate) transform: Option<Box<dyn transform::Transform>>,
    pub(crate) limits: Option<(f64, f64)>,
    pub(crate) breaks: Option<Vec<f64>>,
    pub(crate) labels: Option<Vec<String>>,
}

/// Identifies which aesthetic this scale applies to
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleAesthetic {
    XContinuous,
    YContinuous,
    ColorContinuous,
    FillContinuous,
    SizeContinuous,
    AlphaContinuous,
}

impl ContinuousScaleBuilder {
    pub fn new(aesthetic: ScaleAesthetic) -> Self {
        Self {
            aesthetic,
            transform: None,
            limits: None,
            breaks: None,
            labels: None,
        }
    }

    /// Set the transformation for this scale (log10, sqrt, etc.)
    pub fn transform(mut self, transform: Box<dyn transform::Transform>) -> Self {
        self.transform = Some(transform);
        self
    }

    /// Set explicit limits for this scale
    pub fn limits(mut self, min: f64, max: f64) -> Self {
        self.limits = Some((min, max));
        self
    }

    /// Set explicit break positions
    pub fn breaks(mut self, breaks: Vec<f64>) -> Self {
        self.breaks = Some(breaks);
        self
    }

    /// Set explicit labels for breaks
    pub fn labels(mut self, labels: Vec<String>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Apply this builder's configuration to a scale set
    pub(crate) fn apply_to(self, scales: &mut ScaleSet) {
        use ScaleAesthetic::*;
        
        match self.aesthetic {
            XContinuous => {
                if let Some(transform) = self.transform {
                    scales.x_continuous.set_transform(transform);
                }
                // TODO: apply limits, breaks, labels when those features are added
            }
            YContinuous => {
                if let Some(transform) = self.transform {
                    scales.y_continuous.set_transform(transform);
                }
            }
            ColorContinuous => {
                // TODO: implement when continuous color scales support transforms
            }
            FillContinuous => {
                // TODO: implement when continuous fill scales support transforms
            }
            SizeContinuous => {
                // TODO: implement when continuous size scales support transforms
            }
            AlphaContinuous => {
                if let Some(transform) = self.transform {
                    scales.alpha_continuous.set_transform(transform);
                }
            }
        }
    }
}

/// Create a continuous scale builder for the x aesthetic
pub fn scale_x_continuous() -> ContinuousScaleBuilder {
    ContinuousScaleBuilder::new(ScaleAesthetic::XContinuous)
}

/// Create a continuous scale builder for the y aesthetic
pub fn scale_y_continuous() -> ContinuousScaleBuilder {
    ContinuousScaleBuilder::new(ScaleAesthetic::YContinuous)
}

/// Create a continuous scale builder for the color aesthetic
pub fn scale_color_continuous() -> ContinuousScaleBuilder {
    ContinuousScaleBuilder::new(ScaleAesthetic::ColorContinuous)
}

/// Create a continuous scale builder for the fill aesthetic
pub fn scale_fill_continuous() -> ContinuousScaleBuilder {
    ContinuousScaleBuilder::new(ScaleAesthetic::FillContinuous)
}

/// Create a continuous scale builder for the size aesthetic
pub fn scale_size_continuous() -> ContinuousScaleBuilder {
    ContinuousScaleBuilder::new(ScaleAesthetic::SizeContinuous)
}

/// Create a continuous scale builder for the alpha aesthetic
pub fn scale_alpha_continuous() -> ContinuousScaleBuilder {
    ContinuousScaleBuilder::new(ScaleAesthetic::AlphaContinuous)
}
