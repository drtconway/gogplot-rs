use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::data::DataSource;
use crate::error::Result;
use crate::stat::StatTransform;
use crate::utils::dataframe::{DataFrame, FloatVec, IntVec, StrVec};
use std::collections::HashMap;

/// Enum to handle integer, float, and string x values
enum XValues {
    Int(Vec<i64>),
    Float(Vec<f64>),
    Str(Vec<String>),
}

/// Count statistical transformation
///
/// Groups data by the x aesthetic and counts the number of observations in each group.
/// Produces a new `count` column and updates the y aesthetic to map to it.
///
/// # Example
///
/// If the original data has x values [1, 1, 2, 2, 2, 3], the count stat will produce:
/// - x: [1, 2, 3]
/// - count: [2, 3, 1]
pub struct Count;

impl StatTransform for Count {
    fn apply(
        &self,
        data: Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<Option<(Box<dyn DataSource>, AesMap)>> {
        // Get the x aesthetic - this is required for count
        let x_mapping = mapping.get(&Aesthetic::X).ok_or_else(|| {
            crate::error::PlotError::Generic("Count stat requires X aesthetic".to_string())
        })?;

        // Only support column mappings for now
        let x_col_name = match x_mapping {
            AesValue::Column(name) => name,
            _ => {
                return Err(crate::error::PlotError::Generic(
                    "Count stat requires X to be mapped to a column".to_string(),
                ));
            }
        };

        // Get the x column from data
        let x_col = data.get(x_col_name.as_str()).ok_or_else(|| {
            crate::error::PlotError::Generic(format!("Column '{}' not found in data", x_col_name))
        })?;

        // Count by x value - we need to handle different types
        let (x_values, counts) = if let Some(int_vec) = x_col.as_int() {
            let (x, c) = count_int_values(int_vec.iter().copied());
            (x, c)
        } else if let Some(float_vec) = x_col.as_float() {
            let (x, c) = count_float_values(float_vec.iter().copied());
            (x, c)
        } else if let Some(str_vec) = x_col.as_str() {
            let (x, c) = count_str_values(str_vec.iter());
            (x, c)
        } else {
            return Err(crate::error::PlotError::Generic(
                "Unsupported column type for count stat".to_string(),
            ));
        };

        // Create a new DataFrame with x and count columns
        let mut computed = DataFrame::new();
        // x_values is Vec<i64>, Vec<f64>, or Vec<String>; counts is always Vec<i64>
        match x_values {
            XValues::Int(vals) => {
                computed.add_column("x", Box::new(IntVec(vals)));
            }
            XValues::Float(vals) => {
                computed.add_column("x", Box::new(FloatVec(vals)));
            }
            XValues::Str(vals) => {
                // Keep string categories as strings - categorical scales will handle positioning
                computed.add_column("x", Box::new(StrVec(vals)));
            }
        }
        computed.add_column("count", Box::new(IntVec(counts)));

        // Update the mapping to use the count column for y
        let mut new_mapping = mapping.clone();
        new_mapping.set(Aesthetic::Y, AesValue::column("count"));
        new_mapping.set(Aesthetic::X, AesValue::column("x"));

        Ok(Some((Box::new(computed), new_mapping)))
    }
}

/// Count integer values and return sorted unique values with their counts
fn count_int_values(values: impl Iterator<Item = i64>) -> (XValues, Vec<i64>) {
    let mut counts: HashMap<i64, i64> = HashMap::new();

    for val in values {
        *counts.entry(val).or_insert(0) += 1;
    }

    // Sort by x value
    let mut pairs: Vec<(i64, i64)> = counts.into_iter().collect();
    pairs.sort_by_key(|(x, _)| *x);

    let x_values: Vec<i64> = pairs.iter().map(|(x, _)| *x).collect();
    let count_values: Vec<i64> = pairs.iter().map(|(_, c)| *c).collect();

    (XValues::Int(x_values), count_values)
}

/// Count float values and return sorted unique values with their counts
/// NaN values are dropped (not counted)
fn count_float_values(values: impl Iterator<Item = f64>) -> (XValues, Vec<i64>) {
    let mut counts: HashMap<u64, (f64, i64)> = HashMap::new();

    for val in values {
        // Skip NaN values - they won't compare properly
        if val.is_nan() {
            continue;
        }

        // Use bit representation as key for hashing
        let key = val.to_bits();
        counts
            .entry(key)
            .and_modify(|(_, count)| *count += 1)
            .or_insert((val, 1));
    }

    // Sort by x value
    let mut pairs: Vec<(f64, i64)> = counts.into_values().collect();
    pairs.sort_by(|(a, _), (b, _)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let x_values: Vec<f64> = pairs.iter().map(|(x, _)| *x).collect();
    let count_values: Vec<i64> = pairs.iter().map(|(_, c)| *c).collect();

    (XValues::Float(x_values), count_values)
}

/// Count string values and return sorted unique values with their counts
fn count_str_values<'a>(values: impl Iterator<Item = &'a String>) -> (XValues, Vec<i64>) {
    let mut counts: HashMap<String, i64> = HashMap::new();

    for val in values {
        match counts.get_mut(val) {
            Some(count) => *count += 1,
            None => {
                counts.insert(val.clone(), 1);
            }
        }
    }

    // Sort by x value (alphabetically)
    let mut pairs: Vec<(String, i64)> = counts.into_iter().collect();
    pairs.sort_by(|(a, _), (b, _)| a.cmp(b));

    let mut x_values: Vec<String> = Vec::with_capacity(pairs.len());
    let mut count_values: Vec<i64> = Vec::with_capacity(pairs.len());
    for (x, c) in pairs.into_iter() {
        x_values.push(x);
        count_values.push(c);
    }

    (XValues::Str(x_values), count_values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::dataframe::DataFrame;

    #[test]
    fn test_count_basic() {
        // Create test data: x values [1, 1, 2, 2, 2, 3]
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(IntVec(vec![1, 1, 2, 2, 2, 3])));

        let mut mapping = AesMap::new();
        mapping.x("x");

        let count = Count;
        let result = count.apply(Box::new(df), &mapping);
        assert!(result.is_ok());

        let option_result = result.unwrap();
        assert!(option_result.is_some());
        let (data, new_mapping) = option_result.unwrap();

        // Check that y is now mapped to count
        assert_eq!(
            new_mapping.get(&Aesthetic::Y),
            Some(&AesValue::column("count"))
        );

        // Check the computed values
        let x_col = data.get("x").unwrap();
        let x_vals: Vec<i64> = x_col.as_int().unwrap().iter().copied().collect();
        assert_eq!(x_vals, vec![1, 2, 3]);

        let count_col = data.get("count").unwrap();
        let count_vals: Vec<i64> = count_col.as_int().unwrap().iter().copied().collect();
        assert_eq!(count_vals, vec![2, 3, 1]);
    }

    #[test]
    fn test_count_single_value() {
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(IntVec(vec![5, 5, 5, 5])));

        let mut mapping = AesMap::new();
        mapping.x("x");

        let count = Count;
        let (data, _) = count.apply(Box::new(df), &mapping).unwrap().unwrap();

        let x_col = data.get("x").unwrap();
        let x_vals: Vec<i64> = x_col.as_int().unwrap().iter().copied().collect();
        assert_eq!(x_vals, vec![5]);

        let count_col = data.get("count").unwrap();
        let count_vals: Vec<i64> = count_col.as_int().unwrap().iter().copied().collect();
        assert_eq!(count_vals, vec![4]);
    }

    #[test]
    fn test_count_all_unique() {
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(IntVec(vec![1, 2, 3, 4, 5])));

        let mut mapping = AesMap::new();
        mapping.x("x");

        let count = Count;
        let (data, _) = count.apply(Box::new(df), &mapping).unwrap().unwrap();

        let count_col = data.get("count").unwrap();
        let count_vals: Vec<i64> = count_col.as_int().unwrap().iter().copied().collect();
        assert_eq!(count_vals, vec![1, 1, 1, 1, 1]);
    }



    #[test]
    fn test_count_requires_x() {
        let mut df = DataFrame::new();
        df.add_column("y", Box::new(IntVec(vec![1, 2, 3])));

        let mapping = AesMap::new(); // No x mapping

        let count = Count;
        let result = count.apply(Box::new(df), &mapping);
        assert!(result.is_err());
    }

    #[test]
    fn test_count_int_values_helper() {
        let (x, counts) = count_int_values(vec![3, 1, 2, 1, 3, 3].into_iter());
        match x {
            XValues::Int(vals) => assert_eq!(vals, vec![1, 2, 3]),
            XValues::Float(_) => panic!("Expected Int values"),
            XValues::Str(_) => panic!("Expected Int values"),
        }
        assert_eq!(counts, vec![2, 1, 3]);
    }

    #[test]
    fn test_count_floats() {
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(FloatVec(vec![1.5, 1.5, 2.5, 2.5, 2.5, 3.5])));

        let mut mapping = AesMap::new();
        mapping.x("x");

        let count = Count;
        let (data, _) = count.apply(Box::new(df), &mapping).unwrap().unwrap();

        let x_col = data.get("x").unwrap();
        let x_vals: Vec<f64> = x_col.as_float().unwrap().iter().copied().collect();
        assert_eq!(x_vals, vec![1.5, 2.5, 3.5]);

        let count_col = data.get("count").unwrap();
        let count_vals: Vec<i64> = count_col.as_int().unwrap().iter().copied().collect();
        assert_eq!(count_vals, vec![2, 3, 1]);
    }

    #[test]
    fn test_count_floats_with_nan() {
        let mut df = DataFrame::new();
        df.add_column(
            "x",
            Box::new(FloatVec(vec![1.0, f64::NAN, 2.0, f64::NAN, 1.0, 2.0, 2.0])),
        );

        let mut mapping = AesMap::new();
        mapping.x("x");

        let count = Count;
        let (data, _) = count.apply(Box::new(df), &mapping).unwrap().unwrap();

        let x_col = data.get("x").unwrap();
        let x_vals: Vec<f64> = x_col.as_float().unwrap().iter().copied().collect();
        // NaN values are dropped, so we only have 1.0 and 2.0
        assert_eq!(x_vals.len(), 2);
        assert_eq!(x_vals[0], 1.0);
        assert_eq!(x_vals[1], 2.0);

        let count_col = data.get("count").unwrap();
        let count_vals: Vec<i64> = count_col.as_int().unwrap().iter().copied().collect();
        assert_eq!(count_vals, vec![2, 3]); // 2 ones, 3 twos (NaNs dropped)
    }

    #[test]
    fn test_count_floats_with_infinity() {
        let mut df = DataFrame::new();
        df.add_column(
            "x",
            Box::new(FloatVec(vec![
                1.0,
                f64::INFINITY,
                2.0,
                f64::NEG_INFINITY,
                1.0,
                f64::INFINITY,
            ])),
        );

        let mut mapping = AesMap::new();
        mapping.x("x");

        let count = Count;
        let (data, _) = count.apply(Box::new(df), &mapping).unwrap().unwrap();

        let x_col = data.get("x").unwrap();
        let x_vals: Vec<f64> = x_col.as_float().unwrap().iter().copied().collect();
        // Should be: -inf, 1.0, 2.0, +inf
        assert_eq!(x_vals[0], f64::NEG_INFINITY);
        assert_eq!(x_vals[1], 1.0);
        assert_eq!(x_vals[2], 2.0);
        assert_eq!(x_vals[3], f64::INFINITY);

        let count_col = data.get("count").unwrap();
        let count_vals: Vec<i64> = count_col.as_int().unwrap().iter().copied().collect();
        assert_eq!(count_vals, vec![1, 2, 1, 2]); // 1 -inf, 2 ones, 1 two, 2 +inf
    }

    #[test]
    fn test_count_strings() {
        let mut df = DataFrame::new();
        df.add_column(
            "x",
            Box::new(StrVec(vec![
                "apple".to_string(),
                "banana".to_string(),
                "apple".to_string(),
                "cherry".to_string(),
                "banana".to_string(),
                "apple".to_string(),
            ])),
        );

        let mut mapping = AesMap::new();
        mapping.x("x");

        let count = Count;
        let (data, _) = count.apply(Box::new(df), &mapping).unwrap().unwrap();

        // String x values are kept as strings (categorical scale will handle positioning)
        let x_col = data.get("x").unwrap();
        let x_vals: Vec<String> = x_col.as_str().unwrap().iter().cloned().collect();
        // Should be alphabetically sorted
        assert_eq!(x_vals, vec!["apple", "banana", "cherry"]);

        let count_col = data.get("count").unwrap();
        let count_vals: Vec<i64> = count_col.as_int().unwrap().iter().copied().collect();
        assert_eq!(count_vals, vec![3, 2, 1]);
    }

    #[test]
    fn test_count_strings_single() {
        let mut df = DataFrame::new();
        df.add_column(
            "x",
            Box::new(StrVec(vec![
                "test".to_string(),
                "test".to_string(),
                "test".to_string(),
            ])),
        );

        let mut mapping = AesMap::new();
        mapping.x("x");

        let count = Count;
        let (data, _) = count.apply(Box::new(df), &mapping).unwrap().unwrap();

        // String x values are kept as strings (categorical scale will handle positioning)
        let x_col = data.get("x").unwrap();
        let x_vals: Vec<String> = x_col.as_str().unwrap().iter().cloned().collect();
        assert_eq!(x_vals, vec!["test"]);

        let count_col = data.get("count").unwrap();
        let count_vals: Vec<i64> = count_col.as_int().unwrap().iter().copied().collect();
        assert_eq!(count_vals, vec![3]);
    }
}
