// Geom builder methods for Plot

use crate::aesthetics::AesMap;
use crate::data::DataSource;
use crate::layer::Layer;

/// Trait for adding geom layers to a plot
pub trait GeomBuilder: Sized {
    /// Get mutable access to layers
    fn layers_mut(&mut self) -> &mut Vec<Layer>;

    /// Get access to default aesthetics
    fn default_aes(&self) -> &AesMap;

    /// Get mutable access to plot data
    fn data_mut(&mut self) -> &mut Option<Box<dyn DataSource>>;

    fn geom_point(self) -> Self;

}
