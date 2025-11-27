// Layer scaffolding for grammar of graphics

use crate::aesthetics::AesMap;
use crate::data::DataSource;
use crate::geom::Geom;

/// Statistical transformation
#[derive(Clone)]
pub enum Stat {
    /// No stat specified - use the geom's default
    None,
    /// Identity stat - use data as-is (no transformation)
    Identity,
    Count,
    Bin(crate::stat::bin::CumulativeBinStrategy),
    Boxplot {
        coef: f64,
    },
    Density {
        adjust: f64,
        n: usize,
    },
    Summary(Vec<crate::aesthetics::Aesthetic>),
    Smooth {
        method: crate::stat::smooth::Method,
        level: f64,
        n: usize,
        span: f64,
    },
    // Add more as needed
}

/// Position adjustment for overlapping geoms (user-facing API)
#[derive(Clone, Debug)]
pub enum Position {
    Identity,
    Stack,
    Dodge,
    Jitter,
    Fill,
    // Add more as needed
}

/// Position specification with parameters (internal representation)
/// This is created from Position + geom context and used in the pipeline
#[derive(Clone, Debug)]
pub enum PositionSpec {
    Identity,
    Stack { reverse: bool },
    Dodge { width: f64, padding: f64 },
    Jitter { width: f64, height: f64 },
    Fill,
}

impl PositionSpec {
    /// Convert user-facing Position + geom parameters into PositionSpec
    pub fn from_position(position: &Position, geom_width: Option<f64>) -> Self {
        match position {
            Position::Identity => PositionSpec::Identity,
            Position::Stack => PositionSpec::Stack { reverse: false },
            Position::Dodge => PositionSpec::Dodge {
                width: geom_width.unwrap_or(0.9),
                padding: 0.1,
            },
            Position::Jitter => PositionSpec::Jitter {
                width: 0.4,
                height: 0.4,
            },
            Position::Fill => PositionSpec::Fill,
        }
    }

    /// Apply position adjustment to normalized data
    ///
    /// Takes data where all aesthetic values are already normalized to [0,1] via scales.
    /// Returns modified data (typically with adjusted x/xmin/xmax or y/ymin/ymax columns)
    /// and potentially an updated mapping.
    ///
    /// Returns None if no adjustment is needed.
    pub fn apply(
        &self,
        _data: Box<dyn DataSource>,
        _mapping: &AesMap,
    ) -> Result<Option<(Box<dyn DataSource>, AesMap)>, crate::error::PlotError> {
        match self {
            PositionSpec::Identity => Ok(None),
            PositionSpec::Dodge { width, padding } => {
                crate::position::dodge::apply_dodge_normalized(_data, _mapping, *width, *padding)
            }
            PositionSpec::Stack { reverse } => {
                crate::position::stack::apply_stack_normalized(_data, _mapping, *reverse)
            }
            PositionSpec::Jitter {
                width: _,
                height: _,
            } => {
                // TODO: Implement jitter
                Ok(None)
            }
            PositionSpec::Fill => {
                // Fill is like stack but normalizes to [0, 1] within each x position
                // TODO: Implement fill
                Ok(None)
            }
        }
    }
}

/// Layer struct - represents one layer in a plot
/// Each layer has its own geom, optional data, aesthetic mappings, stat, and position
pub struct Layer {
    pub geom: Box<dyn Geom>,
    pub data: Option<Box<dyn DataSource>>,
    pub mapping: Option<AesMap>,
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

impl Layer {
    /// Access the data of the layer, with the global data as fallback
    pub fn get_data<'a>(
        &'a self,
        global_data: &'a Option<&'a dyn DataSource>,
    ) -> Option<&'a dyn DataSource> {
        if let Some(computed_data) = &self.computed_data {
            Some(computed_data.as_ref())
        } else if let Some(ref layer_data) = self.data {
            Some(layer_data.as_ref())
        } else {
            global_data.map(|d| d)
        }
    }

    // Access the mapping of the layer, with the global mapping as fallback
    pub fn get_mapping<'a>(&'a self, global_mapping: &'a AesMap) -> &'a AesMap {
        if let Some(ref computed_mapping) = self.computed_mapping {
            computed_mapping
        } else if let Some(ref layer_mapping) = self.mapping {
            layer_mapping
        } else {
            global_mapping
        }
    }
}
