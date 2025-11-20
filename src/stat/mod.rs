pub mod bin;
pub mod count;
pub mod density;

use crate::aesthetics::AesMap;
use crate::data::DataSource;
use crate::error::Result;

/// Trait for statistical transformations
///
/// Stats transform data before rendering. They take the original data and aesthetic mapping,
/// and produce a new data source (potentially with computed columns) and an updated mapping.
///
/// Common stats include:
/// - Identity: No transformation (pass-through)
/// - Count: Count observations in each group
/// - Bin: Bin data into ranges and count
/// - Boxplot: Compute five-number summary statistics
/// - Density: Compute kernel density estimate
pub trait StatTransform: Send + Sync {
    /// Apply the statistical transformation.
    ///
    /// Takes ownership of the original data source and returns a new data source
    /// (which may be a StackedDataSource layering computed data over the original)
    /// and an updated aesthetic mapping that references the computed columns.
    ///
    /// For the Identity stat, this returns None to signal that the original data
    /// should be used unchanged.
    ///
    /// # Arguments
    ///
    /// * `data` - The original data source (consumed)
    /// * `mapping` - The original aesthetic mapping
    ///
    /// # Returns
    ///
    /// An Option containing a tuple of (transformed_data, updated_mapping).
    /// Returns None for Identity stat (use original data/mapping).
    fn apply(
        &self,
        data: Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<Option<(Box<dyn DataSource>, AesMap)>>;
}

/// Identity transformation - returns None to signal no transformation
pub struct Identity;

impl StatTransform for Identity {
    fn apply(
        &self,
        _data: Box<dyn DataSource>,
        _mapping: &AesMap,
    ) -> Result<Option<(Box<dyn DataSource>, AesMap)>> {
        // Identity stat returns None to indicate no transformation
        Ok(None)
    }
}
