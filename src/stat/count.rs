use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain};
use crate::data::VectorIter;
use crate::error::Result;
use crate::stat::Stat;
use crate::utils::data::Vectorable;
use crate::utils::dataframe::DataFrame;
use std::any::Any;
use std::collections::HashMap;

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

impl Count {
    pub fn new() -> Self {
        Count
    }

    fn compute_group_inner<T: crate::data::PrimitiveType + Vectorable>(
        &self,
        aesthetic: Aesthetic,
        iter: impl Iterator<Item = T>,
    ) -> Result<(DataFrame, AesMap)> {
        let mut counts: HashMap<T::Sortable, i64> = HashMap::new();

        for val in iter {
            let key = val.to_sortable();
            *counts.entry(key).or_insert(0) += 1;
        }

        let mut pairs: Vec<(T::Sortable, i64)> = counts.into_iter().collect();
        pairs.sort();
        let mut x_values: Vec<T> = Vec::with_capacity(pairs.len());
        let mut count_values: Vec<i64> = Vec::with_capacity(pairs.len());
        for (x, c) in pairs.into_iter() {
            x_values.push(T::from_sortable(x));
            count_values.push(c);
        }

        let mut df = DataFrame::new();
        df.add_column("x", T::make_vector(x_values));
        df.add_column("count", count_values);

        let mut mapping = AesMap::new();
        mapping.x("x", aesthetic.domain());
        mapping.y("count", AestheticDomain::Continuous);

        Ok((df, mapping))
    }
}

impl Default for Count {
    fn default() -> Self {
        Self::new()
    }
}

impl Stat for Count {
    fn compute_group(
        &self,
        aesthetics: Vec<Aesthetic>,
        iters: Vec<VectorIter<'_>>,
        _params: Option<&dyn Any>,
    ) -> Result<(DataFrame, AesMap)> {
        for (aes, iter) in aesthetics.into_iter().zip(iters.into_iter()) {
            match iter {
                VectorIter::Int(it) => {
                    return self.compute_group_inner(aes, it);
                }
                VectorIter::Float(it) => {
                    return self.compute_group_inner(aes, it);
                }
                VectorIter::Str(it) => {
                    return self.compute_group_inner(aes, it.map(|s| s.to_string()));
                }
                VectorIter::Bool(it) => {
                    return self.compute_group_inner(aes, it);
                }
            }
        }
        panic!("No aesthetics provided");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        aesthetics::{AesMap, AesValue, Aesthetic, AestheticDomain},
        data::{DataSource, VectorValue},
        utils::dataframe::DataFrame,
    };

    #[test]
    fn test_count_basic() {
        // Create test data: x values [1, 1, 2, 2, 2, 3]
        let mut df = DataFrame::new();
        df.add_column("x", vec![1, 1, 2, 2, 2, 3]);
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Discrete);

        let count = Count;
        let (data, new_mapping) = count.compute(df.as_ref(), &mapping).expect("compute failed");

        // Check that y is now mapped to count
        assert_eq!(
            new_mapping.get(&Aesthetic::Y(AestheticDomain::Continuous)),
            Some(&AesValue::column("count"))
        );

        // Check the computed values
        let x_col = data.get("x").unwrap();
        let x_vals: Vec<i64> = x_col.iter_int().unwrap().collect();
        assert_eq!(x_vals, vec![1, 2, 3]);

        let count_col = data.get("count").unwrap();
        let count_vals: Vec<i64> = count_col.iter_int().unwrap().collect();
        assert_eq!(count_vals, vec![2, 3, 1]);
    }

    #[test]
    fn test_count_single_value() {
        let mut df = DataFrame::new();
        df.add_column("x", vec![5, 5, 5, 5]);
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Discrete);

        let count = Count;
        let (data, _) = count.compute(df.as_ref(), &mapping).unwrap();

        let x_col = data.get("x").unwrap();
        let x_vals: Vec<i64> = x_col.iter_int().unwrap().collect();
        assert_eq!(x_vals, vec![5]);

        let count_col = data.get("count").unwrap();
        let count_vals: Vec<i64> = count_col.iter_int().unwrap().collect();
        assert_eq!(count_vals, vec![4]);
    }

    #[test]
    fn test_count_all_unique() {
        let mut df = DataFrame::new();
        df.add_column("x", vec![1, 2, 3, 4, 5]);
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Discrete);

        let count = Count;
        let (data, _) = count.compute(df.as_ref(), &mapping).unwrap();

        let count_col = data.get("count").unwrap();
        let count_vals: Vec<i64> = count_col.iter_int().unwrap().collect();
        assert_eq!(count_vals, vec![1, 1, 1, 1, 1]);
    }

    #[test]
    fn test_count_requires_x() {
        let mut df = DataFrame::new();
        df.add_column("y", vec![1, 2, 3]);
        let df: Box<dyn DataSource> = Box::new(df);

        let mapping = AesMap::new(); // No x mapping

        let count = Count;
        let result = count.compute(df.as_ref(), &mapping);
        assert!(result.is_err());
    }

    #[test]
    fn test_count_floats() {
        let mut df = DataFrame::new();
        df.add_column("x", vec![1.5, 1.5, 2.5, 2.5, 2.5, 3.5]);
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Discrete);

        let count = Count;
        let (data, _) = count.compute(df.as_ref(), &mapping).unwrap();

        let x_col = data.get("x").unwrap();
        let x_vals: Vec<f64> = x_col.iter_float().unwrap().collect();
        assert_eq!(x_vals, vec![1.5, 2.5, 3.5]);

        let count_col = data.get("count").unwrap();
        let count_vals: Vec<i64> = count_col.iter_int().unwrap().collect();
        assert_eq!(count_vals, vec![2, 3, 1]);
    }

    #[test]
    fn test_count_floats_with_nan() {
        let mut df = DataFrame::new();
        df.add_column(
            "x",
            VectorValue::from(vec![1.0, f64::NAN, 2.0, f64::NAN, 1.0, 2.0, 2.0]),
        );
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Discrete);

        let count = Count;
        let (data, _) = count.compute(df.as_ref(), &mapping).unwrap();

        let x_col = data.get("x").unwrap();
        let x_vals: Vec<f64> = x_col.iter_float().unwrap().collect();
        // NaN values are dropped, so we only have 1.0 and 2.0
        assert_eq!(x_vals.len(), 3);
        assert_eq!(x_vals[0], 1.0);
        assert_eq!(x_vals[1], 2.0);
        assert!(x_vals[2].is_nan());

        let count_col = data.get("count").unwrap();
        let count_vals: Vec<i64> = count_col.iter_int().unwrap().collect();
        assert_eq!(count_vals, vec![2, 3, 2]); // 2 ones, 3 twos (NaNs dropped)
    }

    #[test]
    fn test_count_floats_with_infinity() {
        let mut df = DataFrame::new();
        df.add_column(
            "x",
            VectorValue::from(vec![
                1.0,
                f64::INFINITY,
                2.0,
                f64::NEG_INFINITY,
                1.0,
                f64::INFINITY,
            ]),
        );
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Discrete);

        let count = Count;
        let (data, _) = count.compute(df.as_ref(), &mapping).unwrap();

        let x_col = data.get("x").unwrap();
        let x_vals: Vec<f64> = x_col.iter_float().unwrap().collect();
        // Should be: -inf, 1.0, 2.0, +inf
        assert_eq!(x_vals[0], f64::NEG_INFINITY);
        assert_eq!(x_vals[1], 1.0);
        assert_eq!(x_vals[2], 2.0);
        assert_eq!(x_vals[3], f64::INFINITY);

        let count_col = data.get("count").unwrap();
        let count_vals: Vec<i64> = count_col.iter_int().unwrap().collect();
        assert_eq!(count_vals, vec![1, 2, 1, 2]); // 1 -inf, 2 ones, 1 two, 2 +inf
    }

    #[test]
    fn test_count_strings() {
        let mut df = DataFrame::new();
        df.add_column(
            "x",
            VectorValue::from(vec![
                "apple", "banana", "apple", "cherry", "banana", "apple",
            ]),
        );
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Discrete);

        let count = Count;
        let (data, _) = count.compute(df.as_ref(), &mapping).unwrap();

        // String x values are kept as strings (categorical scale will handle positioning)
        let x_col = data.get("x").unwrap();
        let x_vals: Vec<String> = x_col.iter_str().unwrap().map(|s| s.to_string()).collect();
        // Should be alphabetically sorted
        assert_eq!(x_vals, vec!["apple", "banana", "cherry"]);

        let count_col = data.get("count").unwrap();
        let count_vals: Vec<i64> = count_col.iter_int().unwrap().collect();
        assert_eq!(count_vals, vec![3, 2, 1]);
    }

    #[test]
    fn test_count_strings_single() {
        let mut df = DataFrame::new();
        df.add_column("x", vec!["test", "test", "test"]);
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Discrete);

        let count = Count;
        let (data, _) = count.compute(df.as_ref(), &mapping).unwrap();

        // String x values are kept as strings (categorical scale will handle positioning)
        let x_col = data.get("x").unwrap();
        let x_vals: Vec<String> = x_col.iter_str().unwrap().map(|s| s.to_string()).collect();
        assert_eq!(x_vals, vec!["test"]);

        let count_col = data.get("count").unwrap();
        let count_vals: Vec<i64> = count_col.iter_int().unwrap().collect();
        assert_eq!(count_vals, vec![3]);
    }
}
