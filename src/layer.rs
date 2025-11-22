// Layer scaffolding for grammar of graphics

use crate::aesthetics::AesMap;
use crate::data::DataSource;
use crate::geom::Geom;

/// Statistical transformation
#[derive(Clone)]
pub enum Stat {
    Identity,
    Count,
    Bin(crate::stat::bin::CumulativeBinStrategy),
    Smooth,
    // Add more as needed
}

/// Position adjustment for overlapping geoms
#[derive(Clone, Debug)]
pub enum Position {
    Identity,
    Stack,
    Dodge,
    Jitter,
    Fill,
    // Add more as needed
}

/// Layer struct - represents one layer in a plot
/// Each layer has its own geom, optional data, aesthetic mappings, stat, and position
pub struct Layer {
    pub geom: Box<dyn Geom>,
    pub data: Option<Box<dyn DataSource>>,
    pub mapping: AesMap,
    pub stat: Stat,
    pub position: Position,
    /// Computed stat data (filled in during stat computation phase)
    pub computed_data: Option<Box<dyn DataSource>>,
    /// Updated aesthetic mapping after stat computation (if stat was applied)
    pub computed_mapping: Option<AesMap>,
    /// Transformed scales after stat/position adjustments
    /// If None, uses plot-level scales. If Some, uses these for rendering this layer.
    pub computed_scales: Option<crate::plot::ScaleSet>,
}
