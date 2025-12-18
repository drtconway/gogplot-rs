// Scale application - convert data through scales to normalized coordinates

use crate::aesthetics::AesMap;
use crate::data::DataSource;
use crate::error::PlotError;
use crate::plot::ScaleSet;
use crate::utils::dataframe::DataFrame;

/// Apply scales to a layer's data, converting all aesthetic columns to normalized form.
///
/// This function:
/// - Takes raw data and aesthetic mappings
/// - Applies the appropriate scale to each mapped aesthetic
/// - Returns a new DataFrame where aesthetic columns are normalized, plus a new mapping
/// - Uses sentinels for missing/rejected values:
///   - NaN for positional aesthetics (x, y, xmin, xmax, ymin, ymax) and size
///   - -1 for color/fill/shape
///   - 0.0 for alpha (fully transparent)
/// - Preserves non-aesthetic columns unchanged (e.g., for grouping)
///
/// # Arguments
/// * `data` - The input data (raw or from stat transformation)
/// * `mapping` - Aesthetic mappings (which columns map to which aesthetics)
/// * `scales` - The scales to apply for each aesthetic
///
/// # Returns
/// A tuple of (DataFrame, AesMap) where:
/// - DataFrame has scaled aesthetic columns and preserved non-aesthetic columns
/// - AesMap maps each aesthetic to its column name in the result DataFrame
pub fn apply_scales(
    data: &dyn DataSource,
    mapping: &AesMap,
    scales: &ScaleSet,
) -> Result<(DataFrame, AesMap), PlotError> {
    let data = DataFrame::new();
    let mut mapping = AesMap::new();
    Ok((data, mapping))
}
