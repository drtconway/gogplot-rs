// Scale application - convert data through scales to normalized coordinates

use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::data::DataSource;
use crate::error::PlotError;
use crate::plot::ScaleSet;
use crate::scale::ScaleType;
use crate::utils::dataframe::DataFrame;

/// Apply scales to a layer's data, converting all aesthetic columns to normalized form.
///
/// This function:
/// - Takes raw data and aesthetic mappings
/// - Applies the appropriate scale to each mapped aesthetic
/// - Returns a new DataFrame where aesthetic columns are normalized
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
/// A new DataFrame with scaled aesthetic columns and preserved non-aesthetic columns
pub fn apply_scales(
    data: &dyn DataSource,
    mapping: &AesMap,
    scales: &ScaleSet,
) -> Result<DataFrame, PlotError> {
    use std::collections::HashMap;
    
    let mut result = DataFrame::new();
    
    // Build reverse mapping: column name -> list of aesthetics that use it
    let mut column_to_aesthetics: HashMap<String, Vec<Aesthetic>> = HashMap::new();
    
    for (aesthetic, aes_value) in mapping.iter() {
        if let AesValue::Column { name, .. } = aes_value {
            column_to_aesthetics
                .entry(name.clone())
                .or_insert_with(Vec::new)
                .push(*aesthetic);
        }
    }
    
    // Process each column in the input data
    for col_name in data.column_names() {
        if let Some(aesthetics) = column_to_aesthetics.get(&col_name) {
            // This column is mapped to one or more aesthetics
            // For now, we'll handle each aesthetic separately
            // TODO: Handle columns mapped to multiple aesthetics
            for aesthetic in aesthetics {
                apply_aesthetic_scale(
                    data,
                    &col_name,
                    *aesthetic,
                    mapping,
                    scales,
                    &mut result,
                )?;
            }
        } else {
            // Not mapped to any aesthetic - preserve as-is
            // This includes grouping columns and other non-aesthetic data
            let col = data.get(&col_name)
                .ok_or_else(|| PlotError::missing_column(&col_name))?;
            result.add_column_from_iter(col_name, col.iter());
        }
    }
    
    Ok(result)
}

/// Apply a scale to a single aesthetic column
fn apply_aesthetic_scale(
    data: &dyn DataSource,
    col_name: &str,
    aesthetic: Aesthetic,
    mapping: &AesMap,
    scales: &ScaleSet,
    result: &mut DataFrame,
) -> Result<(), PlotError> {
    // TODO: Implement scale application for each aesthetic type
    // Different aesthetics need different scales and sentinel values
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::dataframe::{FloatVec, IntVec};
    
    #[test]
    fn test_apply_scales_basic() {
        // TODO: Add tests once implementation is complete
    }
}
