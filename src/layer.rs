// Layer scaffolding for grammar of graphics

use crate::aesthetics::AesMap;
use crate::data::DataSource;
use crate::geom::Geom;
use crate::position::Position;
use crate::stat::Stat;

/// Layer struct - represents one layer in a plot
/// Each layer has its own geom, optional data, aesthetic mappings, stat, and position
pub struct Layer {
    pub stat: Option<Box<dyn Stat>>,
    pub position: Option<Box<dyn Position>>,
    pub geom: Box<dyn Geom>,
    pub data: Option<Box<dyn DataSource>>,
    pub mapping: Option<AesMap>,
}

impl Layer {
    /// Create a new layer with the specified geom
    pub fn new(geom: Box<dyn Geom>) -> Self {
        Self {
            stat: None,
            position: None,
            geom,
            data: None,
            mapping: None,
        }
    }
}
