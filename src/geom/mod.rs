use crate::aesthetics::{Aesthetic, AesMap, AesValue};
use crate::error::PlotError;
use crate::layer::{Layer, Stat, Position};

pub mod bar;
pub mod context;
pub mod density;
pub mod hline;
pub mod line;
pub mod point;
pub mod rect;
pub mod segment;
pub mod vline;

pub use bar::GeomBar;
pub use context::RenderContext;
pub use density::GeomDensity;
pub use hline::GeomHLine;
pub use line::GeomLine;
pub use point::GeomPoint;
pub use rect::GeomRect;
pub use segment::GeomSegment;
pub use vline::GeomVLine;

use crate::data::DataSource;
use crate::utils::dataframe::DataFrame;

pub trait Geom: Send + Sync {
    /// Returns the aesthetics that this geom requires
    fn required_aesthetics(&self) -> &[Aesthetic];

    /// Render the geom with the provided context
    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError>;
    
    /// Compute any statistical transformations needed before scale training
    /// 
    /// This allows geoms like density to compute derived values (e.g., density estimates)
    /// before scales are trained. Returns a new DataFrame with computed values and an
    /// updated aesthetic mapping, or None if no transformation is needed.
    /// 
    /// The updated mapping specifies how aesthetics should map to the computed columns.
    /// For example, a density stat might return a DataFrame with "x" and "density" columns
    /// and a mapping that sets Y -> "density".
    /// 
    /// # Returns
    /// 
    /// * `Ok(Some((dataframe, mapping)))` - Computed data and updated aesthetic mapping
    /// * `Ok(None)` - No transformation needed, use original data and mapping
    /// * `Err(...)` - Computation failed
    fn compute_stat(&self, _data: &dyn DataSource, _mapping: &AesMap) -> Result<Option<(DataFrame, AesMap)>, PlotError> {
        Ok(None)
    }
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
            computed_data: None,
            computed_mapping: None,
        }
    }
}
