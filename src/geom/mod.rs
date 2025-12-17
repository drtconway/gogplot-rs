use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::PlotError;
use crate::layer::Layer;
use crate::position::Position;
use crate::scale::ScaleSet;
use crate::stat::Stat;

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
    Visual(T)
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

pub trait Geom: Send + Sync {
    /// Train the provided scales based on the geom's constants where necessary
    fn train_scales(&self, scales: &mut ScaleSet);

    /// Apply the provided scales to the geom's aesthetic constants where necessary
    fn apply_scales(&mut self, scales: &ScaleSet);

    /// Render the geom with the provided context
    fn render<'a>(&self, ctx: &mut RenderContext<'a>) -> Result<(), PlotError>;
}

/// Trait for geoms that can be converted into layers with their default aesthetics
pub trait IntoLayer: Sized {
    /// Get the default aesthetic values for this geom
    fn default_aesthetics(&self) -> Vec<(Aesthetic, AesValue)>;

    /// Convert this geom into a layer, consuming self
    fn into_layer(self) -> Layer
    where
        Self: Geom + 'static,
    {
        let mut mapping = AesMap::new();

        // Set default aesthetics from geom settings if provided
        for (aesthetic, value) in self.default_aesthetics() {
            mapping.set(aesthetic, value);
        }

        Layer {
            geom: Box::new(self),
            data: None,
            mapping: Some(mapping),
            stat: Stat::Identity,
            position: Position::Identity,
        }
    }
}
