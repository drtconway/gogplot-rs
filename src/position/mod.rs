// Position adjustments for overlapping geoms

pub mod stack;

use crate::aesthetics::AesMap;
use crate::data::DataSource;
use crate::error::PlotError;

/// Trait for position adjustments
///
/// Position adjustments follow the same pattern as StatTransform:
/// they take ownership of data, potentially transform it, and return
/// new data with updated mappings.
pub trait PositionAdjust {
    /// Apply position adjustment to data
    ///
    /// Returns None if no adjustment is needed, or Some((adjusted_data, adjusted_mapping))
    fn apply(
        &self,
        data: Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<Option<(Box<dyn DataSource>, AesMap)>, PlotError>;
}
