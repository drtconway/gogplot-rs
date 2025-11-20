use crate::aesthetics::{Aesthetic, AesMap, AesValue};
use crate::error::PlotError;
use crate::layer::{Layer, Stat, Position};

pub mod context;
pub mod hline;
pub mod line;
pub mod point;
pub mod rect;
pub mod segment;
pub mod vline;

pub use context::RenderContext;
pub use hline::GeomHLine;
pub use line::GeomLine;
pub use point::GeomPoint;
pub use rect::GeomRect;
pub use segment::GeomSegment;
pub use vline::GeomVLine;

pub trait Geom: Send + Sync {
    /// Returns the aesthetics that this geom requires
    fn required_aesthetics(&self) -> &[Aesthetic];

    /// Render the geom with the provided context
    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError>;
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
            mapping,
            stat: Stat::Identity,
            position: Position::Identity,
        }
    }
}
