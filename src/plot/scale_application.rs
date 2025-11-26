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
    use std::collections::{HashMap, HashSet};

    let mut result = DataFrame::new();
    let mut result_mapping = AesMap::new();

    // Build mapping: column name -> list of (aesthetic, output_column_name) pairs
    // Decide on output names upfront to handle duplicates and collisions
    let mut column_to_aesthetic: HashMap<String, Vec<Aesthetic>> = HashMap::new();

    for (aesthetic, aes_value) in mapping.iter() {
        match aes_value {
            AesValue::Column { name, .. } => {
                column_to_aesthetic
                    .entry(name.clone())
                    .or_insert_with(Vec::new)
                    .push(*aesthetic);
            },
            AesValue::Constant { .. } => {
                // Constants will be handled separately after we know row count
                continue;
            },
        }
    }

    let mut used_names: HashSet<String> = data.column_names().into_iter().collect();

    let mut column_to_aesthetic_and_name: HashMap<String, Vec<(Aesthetic, String)>> = HashMap::new();
    for (col_name, aesthetics) in column_to_aesthetic.iter() {
        if aesthetics.len() == 1 {
            // Single aesthetic - use standard name
            let aesthetic = aesthetics[0];
            column_to_aesthetic_and_name
                .entry(col_name.clone())
                .or_insert_with(Vec::new)
                .push((aesthetic, col_name.clone()));
            continue;
        }

        // Multiple aesthetics mapped to same column - disambiguate names
        for aesthetic in aesthetics {
            let aesthetic_name = aesthetic.to_string();

            let mut suffix = 1;
            loop {
                let output_name = format!("{}_{}_{}", col_name, aesthetic_name, suffix);
                if !used_names.contains(&output_name) {
                    used_names.insert(output_name.clone());
                    column_to_aesthetic_and_name
                        .entry(col_name.clone())
                        .or_insert_with(Vec::new)
                        .push((*aesthetic, output_name));
                    break;
                }
                suffix += 1;
            }
        }
    }

    for (col_name, aesthetic_outputs) in column_to_aesthetic_and_name.iter() {
        for (aesthetic, output_name) in aesthetic_outputs {
            apply_aesthetic_scale(
                data,
                col_name,
                output_name,
                *aesthetic,
                scales,
                &mut result,
            )?;

            // Add to new mapping
            result_mapping.set(
                *aesthetic,
                AesValue::Column {
                    name: output_name.clone(),
                    hint: None,
                },
            );
        }
    }

    // Process each column in the input data
    for col_name in data.column_names() {
        if let Some(aesthetic_outputs) = column_to_aesthetic_and_name.get(&col_name) {
            // This column is mapped to one or more aesthetics
            for (aesthetic, output_name) in aesthetic_outputs {
                apply_aesthetic_scale(
                    data,
                    &col_name,
                    output_name,
                    *aesthetic,
                    scales,
                    &mut result,
                )?;

                // Add to new mapping
                result_mapping.set(
                    *aesthetic,
                    AesValue::Column {
                        name: output_name.clone(),
                        hint: None,
                    },
                );
            }
        } else {
            // Not mapped to any aesthetic - preserve as-is
            // This includes grouping columns and other non-aesthetic data
            let col = data
                .get(&col_name)
                .ok_or_else(|| PlotError::missing_column(&col_name))?;
            result.add_column_from_iter(col_name, col.iter());
        }
    }

    // Now handle constants - scale them and add to result mapping as constants
    for (aesthetic, aes_value) in mapping.iter() {
        if let AesValue::Constant { value, hint } = aes_value {
            let scaled_value = apply_scale_to_constant(*aesthetic, value, *hint, scales)?;
            result_mapping.set(
                *aesthetic,
                AesValue::Constant {
                    value: crate::data::PrimitiveValue::Float(scaled_value),
                    hint: None, // Already scaled, no hint needed
                },
            );
        }
    }

    Ok((result, result_mapping))
}

/// Apply a scale to a single aesthetic column.
fn apply_aesthetic_scale(
    data: &dyn DataSource,
    col_name: &str,
    output_name: &str,
    aesthetic: Aesthetic,
    scales: &ScaleSet,
    result: &mut DataFrame,
) -> Result<(), PlotError> {
    use crate::utils::dataframe::FloatVec;

    // Get the source column
    let col = data
        .get(col_name)
        .ok_or_else(|| PlotError::missing_column(col_name))?;

    // Determine which scale to use based on aesthetic type
    match aesthetic {
        // Positional aesthetics use x or y scale, output normalized [0,1] with NaN sentinel
        Aesthetic::X
        | Aesthetic::Xmin
        | Aesthetic::Xmax
        | Aesthetic::XBegin
        | Aesthetic::XEnd
        | Aesthetic::XIntercept => {
            let scale = scales
                .x
                .as_ref()
                .ok_or_else(|| PlotError::MissingAesthetic { aesthetic })?;

            let mapped_values = apply_positional_scale(col, scale.as_ref(), aesthetic)?;
            result.add_column(output_name, Box::new(FloatVec(mapped_values)));
        }

        Aesthetic::Y
        | Aesthetic::Ymin
        | Aesthetic::Ymax
        | Aesthetic::YBegin
        | Aesthetic::YEnd
        | Aesthetic::YIntercept
        | Aesthetic::Lower
        | Aesthetic::Middle
        | Aesthetic::Upper => {
            let scale = scales
                .y
                .as_ref()
                .ok_or_else(|| PlotError::MissingAesthetic { aesthetic })?;

            let mapped_values = apply_positional_scale(col, scale.as_ref(), aesthetic)?;
            result.add_column(output_name, Box::new(FloatVec(mapped_values)));
        }

        // Size uses continuous scale, NaN sentinel
        Aesthetic::Size => {
            // Size might not have a scale if it's a constant
            if let Some(scale) = scales.size.as_ref() {
                let mapped_values = apply_positional_scale(col, scale.as_ref(), aesthetic)?;
                result.add_column(output_name, Box::new(FloatVec(mapped_values)));
            } else {
                // No scale, copy as-is (might be constant from geom)
                result.add_column_from_iter(output_name, col.iter());
            }
        }

        // Alpha uses continuous scale, 0.0 sentinel (transparent)
        Aesthetic::Alpha => {
            if let Some(scale) = scales.alpha.as_ref() {
                let mapped_values = apply_positional_scale(col, scale.as_ref(), aesthetic)?;
                result.add_column(output_name, Box::new(FloatVec(mapped_values)));
            } else {
                result.add_column_from_iter(output_name, col.iter());
            }
        }

        // Color aesthetics - handled separately (TODO)
        Aesthetic::Color | Aesthetic::Fill => {
            // TODO: Implement color scale application
            // Need to handle both continuous (gradients) and categorical (discrete colors)
            // Sentinel: -1 for missing
            result.add_column_from_iter(output_name, col.iter());
        }

        // Shape and linetype use categorical scales, -1 sentinel
        Aesthetic::Shape | Aesthetic::Linetype => {
            // TODO: Implement shape/linetype scale application
            result.add_column_from_iter(output_name, col.iter());
        }

        // Group doesn't get scaled, just preserved
        Aesthetic::Group => {
            result.add_column_from_iter(output_name, col.iter());
        }

        // Label doesn't get scaled
        Aesthetic::Label => {
            result.add_column_from_iter(output_name, col.iter());
        }
    }

    Ok(())
}

/// Apply a scale to a constant value, returning the scaled f64.
fn apply_scale_to_constant(
    aesthetic: Aesthetic,
    value: &crate::data::PrimitiveValue,
    hint: Option<ScaleType>,
    scales: &ScaleSet,
) -> Result<f64, PlotError> {
    // Determine which scale to use and apply it
    match aesthetic {
        // Positional aesthetics use x or y scale
        Aesthetic::X | Aesthetic::Xmin | Aesthetic::Xmax | Aesthetic::XBegin | Aesthetic::XEnd | Aesthetic::XIntercept => {
            if let Some(scale) = scales.x.as_ref() {
                scale_primitive_value(value, hint, scale.as_ref(), aesthetic)
            } else {
                // No scale, convert to float
                primitive_to_float(value)
            }
        }

        Aesthetic::Y | Aesthetic::Ymin | Aesthetic::Ymax | Aesthetic::YBegin | Aesthetic::YEnd | 
        Aesthetic::YIntercept | Aesthetic::Lower | Aesthetic::Middle | Aesthetic::Upper => {
            if let Some(scale) = scales.y.as_ref() {
                scale_primitive_value(value, hint, scale.as_ref(), aesthetic)
            } else {
                primitive_to_float(value)
            }
        }

        // Size uses size scale
        Aesthetic::Size => {
            if let Some(scale) = scales.size.as_ref() {
                scale_primitive_value(value, hint, scale.as_ref(), aesthetic)
            } else {
                primitive_to_float(value)
            }
        }

        // Alpha uses alpha scale
        Aesthetic::Alpha => {
            if let Some(scale) = scales.alpha.as_ref() {
                scale_primitive_value(value, hint, scale.as_ref(), aesthetic)
            } else {
                Ok(primitive_to_float(value).unwrap_or(0.0))
            }
        }

        // Color/Fill/Shape/Linetype - TODO: handle these properly
        Aesthetic::Color | Aesthetic::Fill | Aesthetic::Shape | Aesthetic::Linetype => {
            // For now, just convert to float or use -1 as sentinel
            Ok(primitive_to_float(value).unwrap_or(-1.0))
        }

        // Group and Label don't get scaled
        Aesthetic::Group | Aesthetic::Label => {
            // These shouldn't be constants, but handle gracefully
            Ok(primitive_to_float(value).unwrap_or(f64::NAN))
        }
    }
}

/// Apply a scale to a PrimitiveValue
fn scale_primitive_value(
    value: &crate::data::PrimitiveValue,
    hint: Option<ScaleType>,
    scale: &dyn crate::scale::ContinuousScale,
    aesthetic: Aesthetic,
) -> Result<f64, PlotError> {
    use crate::data::PrimitiveValue;

    // Check if we should use categorical mapping (based on hint or value type)
    let use_categorical = hint == Some(ScaleType::Categorical) 
        || matches!(value, PrimitiveValue::Str(_))
        || scale.scale_type() == ScaleType::Categorical;

    if use_categorical {
        // Categorical scale - convert to string for mapping
        let val_str = match value {
            PrimitiveValue::Int(v) => v.to_string(),
            PrimitiveValue::Float(v) => v.to_string(),
            PrimitiveValue::Str(s) => s.clone(),
            PrimitiveValue::Bool(b) => b.to_string(),
        };
        Ok(scale.map_category(&val_str, aesthetic).unwrap_or(f64::NAN))
    } else {
        // Continuous scale - convert to number
        let num_value = primitive_to_float(value)?;
        Ok(scale.map_value(num_value).unwrap_or(f64::NAN))
    }
}

/// Convert PrimitiveValue to f64
fn primitive_to_float(value: &crate::data::PrimitiveValue) -> Result<f64, PlotError> {
    use crate::data::PrimitiveValue;
    
    Ok(match value {
        PrimitiveValue::Int(v) => *v as f64,
        PrimitiveValue::Float(v) => *v,
        PrimitiveValue::Bool(b) => if *b { 1.0 } else { 0.0 },
        PrimitiveValue::Str(s) => {
            // Parse as number, or NaN if not parseable
            s.parse::<f64>().unwrap_or(f64::NAN)
        }
    })
}

/// Apply a positional scale to a column, handling both continuous and categorical scales.
/// Returns normalized positions in [0, 1] with NaN for missing/rejected values.
fn apply_positional_scale(
    col: &dyn crate::data::GenericVector,
    scale: &dyn crate::scale::ContinuousScale,
    aesthetic: Aesthetic,
) -> Result<Vec<f64>, PlotError> {
    use crate::data::VectorIter;

    let mut result = Vec::new();

    // Handle different input types
    match col.iter() {
        VectorIter::Float(iter) => {
            // Check if scale is categorical
            use ScaleType;
            if scale.scale_type() == ScaleType::Categorical {
                // Convert floats to strings for categorical mapping
                for val in iter {
                    let val_str = val.to_string();
                    let mapped = scale.map_category(&val_str, aesthetic).unwrap_or(f64::NAN);
                    result.push(mapped);
                }
            } else {
                // Continuous mapping
                for val in iter {
                    let mapped = scale.map_value(val).unwrap_or(f64::NAN);
                    result.push(mapped);
                }
            }
        }

        VectorIter::Int(iter) => {
            if scale.scale_type() == ScaleType::Categorical {
                // Convert integers to strings for categorical mapping
                for val in iter {
                    let val_str = val.to_string();
                    let mapped = scale.map_category(&val_str, aesthetic).unwrap_or(f64::NAN);
                    result.push(mapped);
                }
            } else {
                // Continuous mapping - convert int to f64
                for val in iter {
                    let mapped = scale.map_value(val as f64).unwrap_or(f64::NAN);
                    result.push(mapped);
                }
            }
        }

        VectorIter::Str(iter) => {
            // Strings always use categorical mapping
            for val in iter {
                let mapped = scale.map_category(val, aesthetic).unwrap_or(f64::NAN);
                result.push(mapped);
            }
        }

        VectorIter::Bool(iter) => {
            // Treat bools as 0/1 for continuous, or categories for categorical
            if scale.scale_type() == ScaleType::Categorical {
                for val in iter {
                    let val_str = val.to_string();
                    let mapped = scale.map_category(&val_str, aesthetic).unwrap_or(f64::NAN);
                    result.push(mapped);
                }
            } else {
                for val in iter {
                    let val_f64 = if val { 1.0 } else { 0.0 };
                    let mapped = scale.map_value(val_f64).unwrap_or(f64::NAN);
                    result.push(mapped);
                }
            }
        }
    }

    Ok(result)
}

/// Helper to convert Aesthetic to column name for scaled output
impl Aesthetic {
    fn to_column_name(&self) -> &'static str {
        match self {
            Aesthetic::X => "x",
            Aesthetic::Y => "y",
            Aesthetic::Xmin => "xmin",
            Aesthetic::Xmax => "xmax",
            Aesthetic::Ymin => "ymin",
            Aesthetic::Ymax => "ymax",
            Aesthetic::Lower => "lower",
            Aesthetic::Middle => "middle",
            Aesthetic::Upper => "upper",
            Aesthetic::XBegin => "xbegin",
            Aesthetic::XEnd => "xend",
            Aesthetic::YBegin => "ybegin",
            Aesthetic::YEnd => "yend",
            Aesthetic::XIntercept => "xintercept",
            Aesthetic::YIntercept => "yintercept",
            Aesthetic::Color => "color",
            Aesthetic::Fill => "fill",
            Aesthetic::Alpha => "alpha",
            Aesthetic::Size => "size",
            Aesthetic::Shape => "shape",
            Aesthetic::Linetype => "linetype",
            Aesthetic::Group => "group",
            Aesthetic::Label => "label",
        }
    }
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
