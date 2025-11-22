// LayerGeom wrapper for type-safe layer configuration

use crate::aesthetics::AesMap;
use crate::geom::Geom;
use crate::layer::Stat;

/// A builder struct that bundles a geom with its aesthetics and stat transformation
///
/// This allows for a fluent API where you can configure all aspects of a layer
/// while maintaining type specificity for the geom.
///
/// # Examples
///
/// ```ignore
/// plot.geom_hline_with(|layer| {
///     layer.aes.yintercept("mean");
///     layer.geom.color(color::RED).size(2.0);
///     layer.stat = Stat::Summary(vec![Aesthetic::Y]);
/// })
/// ```
pub struct LayerGeom<G: Geom> {
    /// The specific geom for this layer
    pub geom: G,
    
    /// Aesthetic mappings for this layer
    pub aes: AesMap,
    
    /// Statistical transformation for this layer
    pub stat: Stat,
}

impl<G: Geom> LayerGeom<G> {
    /// Create a new LayerGeom with the given geom
    pub fn new(geom: G) -> Self {
        Self {
            geom,
            aes: AesMap::new(),
            stat: Stat::Identity,
        }
    }
    
    /// Get the inner parts (consumes self)
    pub(crate) fn into_parts(self) -> (G, AesMap, Stat) {
        (self.geom, self.aes, self.stat)
    }
}
