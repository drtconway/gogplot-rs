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
pub trait PositionAdjust {
    /// Apply position adjustment to data
    ///
    /// # Arguments
    /// * `data` - The data to adjust
    /// * `mapping` - The aesthetic mappings
    /// * `scales` - The current scales (for reference/transformation)
    ///
    /// # Returns
    /// * `None` - No adjustment needed
    /// * `Some((data, mapping, scales))` - Adjusted data, mapping, and optionally transformed scales
    fn apply(
        &self,
        data: &Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<Option<(Box<dyn DataSource>, AesMap)>, PlotError>;
}
