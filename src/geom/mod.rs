use crate::aesthetics::Aesthetic;
use crate::error::PlotError;

pub mod context;
pub mod line;
pub mod point;
pub mod rect;

pub use context::RenderContext;
pub use line::GeomLine;
pub use point::GeomPoint;

pub trait Geom: Send + Sync {
    /// Returns the aesthetics that this geom requires
    fn required_aesthetics(&self) -> &[Aesthetic];

    /// Render the geom with the provided context
    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError>;
}
