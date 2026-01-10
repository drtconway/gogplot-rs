// Position adjustments for overlapping geoms

pub mod dodge;
pub mod stack;

use crate::aesthetics::AesMap;
use crate::error::PlotError;

/// Trait for position adjustments
///
/// Position adjustments can transform data, aesthetic mappings, and scales.
/// This allows adjustments like dodge to modify how data is positioned without
/// requiring intermediate data-space coordinates.
///
/// Note: Position adjustments run after resolution, so all Column references
/// in the mapping have been converted to Vector values. No DataSource is needed.
pub trait Position {
    /// Apply position adjustment to data
    ///
    /// # Arguments
    /// * `mapping` - The aesthetic mappings (already resolved)
    ///
    /// # Returns
    /// * `None` - No adjustment needed
    /// * `Some(mapping)` - Adjusted mapping
    fn apply(
        &self,
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