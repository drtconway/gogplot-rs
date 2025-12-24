use std::collections::HashMap;

use crate::aesthetics::{AestheticDomain, AestheticProperty};
use crate::error::PlotError;
use crate::geom::properties::{Property, PropertyValue, PropertyVector};
use crate::scale::{ScaleIdentifier, ScaleSet};
use crate::theme::Theme;

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

// Define what domains a geom accepts for an aesthetic
pub enum DomainConstraint {
    Any,
    MustBe(AestheticDomain),
}

pub struct AestheticRequirement {
    pub property: AestheticProperty,
    pub required: bool,
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

    fn properties(&self) -> HashMap<AestheticProperty, Property> {
        HashMap::new()
    }

    fn property_defaults(&self, theme: &Theme) -> HashMap<AestheticProperty, PropertyValue> {
        HashMap::new()
    }

    /// Get the list of scales required by this geom
    fn required_scales(&self) -> Vec<ScaleIdentifier> {
        Vec::new()
    }

    /// Train the provided scales based on the geom's constants where necessary
    fn train_scales(&self, scales: &mut ScaleSet);

    /// Apply the provided scales to the geom's aesthetic constants where necessary
    fn apply_scales(&mut self, scales: &ScaleSet);

    /// Render the geom with the provided context, data, and aesthetic properties for a single group.
    fn render<'a>(&self, ctx: &mut RenderContext<'a>, data: HashMap<AestheticProperty, PropertyVector>) -> Result<(), PlotError>;
}
