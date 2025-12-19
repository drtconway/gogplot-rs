// LayerGeom wrapper for type-safe layer configuration

use crate::aesthetics::AesMap;
use crate::data::DataSource;
use crate::geom::Geom;
use crate::stat::Stat;

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
    pub stat: Box<dyn Stat>,
    
    /// Optional layer-specific data
    pub data: Option<Box<dyn DataSource>>,
}

impl<G: Geom> LayerGeom<G> {
    /// Create a new LayerGeom with the given geom and default aesthetics
    pub fn new(geom: G, default_aes: &AesMap) -> Self {
        Self {
            geom,
            aes: default_aes.clone(),
            stat: Box::new(crate::stat::Identity {}),
            data: None,
        }
    }
    
    /// Set layer-specific data (builder style)
    pub fn data(&mut self, data: Box<dyn DataSource>) -> &mut Self {
        self.data = Some(data);
        self
    }
    
    /// Configure aesthetics using a closure (builder style)
    pub fn aes<F>(&mut self, f: F) -> &mut Self
    where
        F: FnOnce(&mut AesMap),
    {
        f(&mut self.aes);
        self
    }
    
    /// Get the inner parts (consumes self)
    pub(crate) fn into_parts(self) -> (G, AesMap, Box<dyn Stat>, Option<Box<dyn DataSource>>) {
        (self.geom, self.aes, self.stat, self.data)
    }
}
