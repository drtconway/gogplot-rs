use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::error::PlotError;
use crate::layer::{Layer, Position, Stat};
use crate::scale::ScaleType;

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

use crate::data::DataSource;

pub trait Geom: Send + Sync {
    /// Returns the aesthetics that this geom requires
    fn required_aesthetics(&self) -> &[Aesthetic];

    /// Returns the required scale type for a given aesthetic.
    /// 
    /// This allows geoms to specify whether they need continuous or categorical scales.
    /// For example, boxplots typically require X to be categorical and Y to be continuous.
    /// 
    /// Default implementation returns `ScaleType::Either` for all aesthetics, meaning
    /// the scale type will be determined by the data type.
    fn aesthetic_scale_type(&self, _aesthetic: Aesthetic) -> ScaleType {
        ScaleType::Either
    }

    /// Render the geom with the provided context
    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError>;

    /// Set up any required data columns before scale training
    ///
    /// This is called after stat computation but before scale training, allowing geoms
    /// to add necessary columns to the data. For example, bar charts need xmin/xmax
    /// columns which should be created from x values with appropriate widths.
    ///
    /// This step happens BEFORE scales are trained so that the scales can see all
    /// the columns that will be used for rendering (e.g., both x and xmin/xmax for bars).
    ///
    /// The method receives the mapping to know which aesthetics are mapped and can
    /// return an updated mapping if it changes which columns aesthetics point to.
    ///
    /// # Returns
    ///
    /// * `Ok(Some((dataframe, mapping)))` - Data with added columns and updated mapping
    /// * `Ok(None)` - No setup needed, use original data and mapping
    /// * `Err(...)` - Setup failed
    fn setup_data(
        &self,
        _data: &dyn DataSource,
        _mapping: &AesMap,
    ) -> Result<Option<(Box<dyn DataSource>, AesMap)>, PlotError> {
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
            mapping: Some(mapping),
            stat: Stat::Identity,
            position: Position::Identity,
            computed_data: None,
            computed_mapping: None,
            computed_scales: None,
        }
    }
}
