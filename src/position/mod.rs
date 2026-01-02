// Position adjustments for overlapping geoms

pub mod dodge;
pub mod stack;

use crate::aesthetics::AesMap;
use crate::data::DataSource;
use crate::error::PlotError;

/// Trait for position adjustments
///
/// Position adjustments can transform data, aesthetic mappings, and scales.
/// This allows adjustments like dodge to modify how data is positioned without
/// requiring intermediate data-space coordinates.
pub trait Position {
    /// Apply position adjustment to data
    ///
    /// # Arguments
    /// * `data` - The data to adjust
    /// * `mapping` - The aesthetic mappings
    /// * `scales` - The current scales (for reference/transformation)
    ///
    /// # Returns
    /// * `None` - No adjustment needed
    /// * `Some(mapping)` - Adjusted mapping
    fn apply(
        &self,
        data: &Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<Option<AesMap>, PlotError>;
}

impl From<&str> for Box<dyn Position> {
    fn from(s: &str) -> Self {
        match s {
            "dodge" => Box::new(dodge::Dodge::default()),
            "stack" => Box::new(stack::Stack::default()),
            _ => panic!("Unknown position adjustment: {}", s),
        }
    }
}