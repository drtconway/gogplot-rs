use crate::aesthetics::{AesMap, AesValue, Aesthetic, AestheticDomain};
use crate::data::PrimitiveValue;
use crate::error::PlotError;
use crate::scale::{ScaleIdentifier, ScaleSet};

pub mod properties;

pub mod bar;
pub mod boxplot;
pub mod context;
pub mod density;
pub mod errorbar;
pub mod histogram;
pub mod hline;
pub mod label;
pub mod line;
pub mod point;
pub mod rect;
pub mod segment;
pub mod smooth;
pub mod text;
pub mod vline;

pub use bar::GeomBar;
pub use boxplot::GeomBoxplot;
pub use context::RenderContext;
pub use density::GeomDensity;
pub use errorbar::GeomErrorbar;
pub use histogram::GeomHistogram;
pub use hline::GeomHLine;
pub use label::GeomLabel;
pub use line::GeomLine;
pub use point::GeomPoint;
pub use rect::GeomRect;
pub use segment::GeomSegment;
pub use smooth::GeomSmooth;
pub use text::GeomText;
pub use vline::GeomVLine;

pub enum GeomConstant<T: Clone> {
    None,
    Scaled(PrimitiveValue),
    Visual(T),
}

impl<T: Clone> GeomConstant<T> {
    pub fn or_value(&self, value: T) -> T {
        match self {
            GeomConstant::None => value,
            GeomConstant::Scaled(_) => value,
            GeomConstant::Visual(v) => v.clone(),
        }
    }
}

impl<T: Clone> Default for GeomConstant<T> {
    fn default() -> Self {
        GeomConstant::None
    }
}

pub trait AesMapBuilderTrait {
    fn aes(&mut self) -> &mut AesMap;
}

pub trait XContininuousAesBuilder: AesMapBuilderTrait {
    fn x(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::X(AestheticDomain::Continuous),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait XDiscreteAesBuilder: AesMapBuilderTrait {
    fn x(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::X(AestheticDomain::Discrete),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait YContininuousAesBuilder: AesMapBuilderTrait {
    fn y(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Y(AestheticDomain::Continuous),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait YDiscreteAesBuilder: AesMapBuilderTrait {
    fn y(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Y(AestheticDomain::Discrete),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait ColorContinuousAesBuilder: AesMapBuilderTrait {
    fn color_continuous(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Color(AestheticDomain::Continuous),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait ColorDiscreteAesBuilder: AesMapBuilderTrait {
    fn color_discrete(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Color(AestheticDomain::Discrete),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait FillContinuousAesBuilder: AesMapBuilderTrait {
    fn fill(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Fill(AestheticDomain::Continuous),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait FillDiscreteAesBuilder: AesMapBuilderTrait {
    fn fill_discrete(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Fill(AestheticDomain::Discrete),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait GroupAesBuilder: AesMapBuilderTrait {
    fn group(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Group,
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub struct AesMapBuilder {
    aes_map: crate::aesthetics::AesMap,
}

impl AesMapBuilder {
    pub fn new() -> Self {
        Self {
            aes_map: crate::aesthetics::AesMap::new(),
        }
    }

    pub fn build(mut self, parent_mapping: &AesMap) -> AesMap {
        for (aes, aes_value) in parent_mapping.iter() {
            if !self.aes_map.contains(*aes) {
                self.aes_map.set(*aes, aes_value.clone());
            }
        }
        self.aes_map
    }
}

impl AesMapBuilderTrait for AesMapBuilder {
    fn aes(&mut self) -> &mut AesMap {
        &mut self.aes_map
    }
}

impl XContininuousAesBuilder for AesMapBuilder {}
impl XDiscreteAesBuilder for AesMapBuilder {}
impl YContininuousAesBuilder for AesMapBuilder {}
impl YDiscreteAesBuilder for AesMapBuilder {}
impl ColorContinuousAesBuilder for AesMapBuilder {}
impl ColorDiscreteAesBuilder for AesMapBuilder {}
impl FillContinuousAesBuilder for AesMapBuilder {}
impl FillDiscreteAesBuilder for AesMapBuilder {}
impl GroupAesBuilder for AesMapBuilder {}

pub trait GeomBuilder {
    fn build(self) -> Box<dyn Geom>;
}

pub trait Geom: Send + Sync {
    /// Get the list of scales required by this geom
    fn required_scales(&self) -> Vec<ScaleIdentifier> {
        Vec::new()
    }

    /// Train the provided scales based on the geom's constants where necessary
    fn train_scales(&self, scales: &mut ScaleSet);

    /// Apply the provided scales to the geom's aesthetic constants where necessary
    fn apply_scales(&mut self, scales: &ScaleSet);

    /// Render the geom with the provided context
    fn render<'a>(&self, ctx: &mut RenderContext<'a>) -> Result<(), PlotError>;
}
