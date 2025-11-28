//! Utility functions for statistical transformations

use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::data::{DataSource, PrimitiveValue, VectorType};
use crate::error::{PlotError, Result};

/// Iterator over aesthetic values extracted from data
pub enum AestheticValueIter<'a> {
    /// Float values from a float column
    Float(Box<dyn Iterator<Item = f64> + 'a>),
    /// Integer values converted to f64
    Int(Box<dyn Iterator<Item = f64> + 'a>),
    /// String values
    Str(Box<dyn Iterator<Item = &'a str> + 'a>),
    /// Constant value repeated n times
    ConstantFloat(f64, usize),
    /// Constant integer repeated n times (as f64)
    ConstantInt(i64, usize),
}

impl<'a> Iterator for AestheticValueIter<'a> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            AestheticValueIter::Float(iter) => iter.next(),
            AestheticValueIter::Int(iter) => iter.next(),
            AestheticValueIter::Str(_) => None, // Can't convert strings to f64 directly
            AestheticValueIter::ConstantFloat(val, n) => {
                if *n > 0 {
                    *n -= 1;
                    Some(*val)
                } else {
                    None
                }
            }
            AestheticValueIter::ConstantInt(val, n) => {
                if *n > 0 {
                    *n -= 1;
                    Some(*val as f64)
                } else {
                    None
                }
            }
        }
    }
}

/// Get aesthetic values as an iterator
///
/// This function handles both column-mapped and constant values.
/// For columns, it returns an iterator over the data.
/// For constants, it returns an iterator that repeats the constant value.
///
/// # Example
///
/// ```ignore
/// let x_values = get_aesthetic_values(data, mapping, Aesthetic::X)?;
/// let x_vec: Vec<f64> = x_values.collect();
/// ```
pub fn get_aesthetic_values<'a>(
    data: &'a dyn DataSource,
    mapping: &'a AesMap,
    aesthetic: Aesthetic,
) -> Result<AestheticValueIter<'a>> {
    let aes_value = mapping
        .get(&aesthetic)
        .ok_or_else(|| PlotError::missing_stat_input("stat", aesthetic))?;

    match aes_value {
        AesValue::Column{ name: col_name, ..} => {
            let series = data
                .get(col_name.as_str())
                .ok_or_else(|| PlotError::missing_column(col_name.as_str()))?;

            match series.vtype() {
                VectorType::Float => {
                    let iter = series
                        .iter_float()
                        .ok_or_else(|| PlotError::missing_column(col_name.as_str()))?;
                    Ok(AestheticValueIter::Float(Box::new(
                        iter.filter(|v| v.is_finite()),
                    )))
                }
                VectorType::Int => {
                    let iter = series
                        .iter_int()
                        .ok_or_else(|| PlotError::missing_column(col_name.as_str()))?;
                    Ok(AestheticValueIter::Int(Box::new(iter.map(|i| i as f64))))
                }
                VectorType::Str => {
                    let iter = series
                        .iter_str()
                        .ok_or_else(|| PlotError::missing_column(col_name.as_str()))?;
                    Ok(AestheticValueIter::Str(Box::new(iter)))
                }
                VectorType::Bool => Err(PlotError::invalid_column_type(
                    col_name.as_str(),
                    "numeric (int or float)",
                )),
            }
        }
        AesValue::Constant{ value: prim_val, hint: _ } => {
            let n = data.len();
            match prim_val {
                PrimitiveValue::Float(f) => Ok(AestheticValueIter::ConstantFloat(*f, n)),
                PrimitiveValue::Int(i) => Ok(AestheticValueIter::ConstantInt(*i, n)),
                _ => Err(PlotError::invalid_column_type("constant", "numeric")),
            }
        }
    }
}

/// Get numeric values from an aesthetic mapping
///
/// This is a convenience function that collects the iterator into a Vec<f64>.
/// Useful for stats that need all values at once.
pub fn get_numeric_values(
    data: &dyn DataSource,
    mapping: &AesMap,
    aesthetic: Aesthetic,
) -> Result<Vec<f64>> {
    Ok(get_aesthetic_values(data, mapping, aesthetic)?.collect())
}
