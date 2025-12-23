use crate::aesthetics::{AestheticDomain, AestheticProperty};
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

// Define what domains a geom accepts for an aesthetic
pub enum DomainConstraint {
    Any,
    MustBe(AestheticDomain),
}

pub struct AestheticRequirement {
    pub property: AestheticProperty,
    pub required: bool,                 // true = required, false = optional
    pub constraint: DomainConstraint,
}

pub trait GeomBuilder {
    fn build(self) -> Box<dyn Geom>;
}

pub trait Geom: Send + Sync {
    /// Get the list of aesthetic requirements for this geom
    fn aesthetic_requirements(&self) -> &'static [AestheticRequirement] {
        &[]
    }

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
