use crate::aesthetics::{AesMap, AesValue, Aesthetic, AestheticDomain};
use crate::data::{DataSource, DiscreteType};
use crate::error::{PlotError, Result};
use crate::stat::Stat;
use crate::utils::data::{DiscreteDiscreteVisitor2, DiscreteVectorVisitor, Vectorable, visit_d, visit2_dd};
use crate::utils::dataframe::{DataFrame, IntVec};
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

impl Stat for Count {
    fn apply(
        &self,
        data: Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<Option<(Box<dyn DataSource>, AesMap)>> {
        if let Some(group_iter) = mapping.get_vector_iter(&Aesthetic::Group, data.as_ref()) {
            let x_iter = match mapping
                .get_vector_iter(&Aesthetic::X(AestheticDomain::Discrete), data.as_ref())
            {
                Some(iter) => iter,
                None => {
                    return Err(crate::error::PlotError::missing_stat_input(
                        "Count",
                        Aesthetic::X(AestheticDomain::Discrete),
                    ));
                }
            };

            let mut counter = GroupedValueCounter::new();
            visit2_dd(group_iter, x_iter, &mut counter)?;
            let GroupedValueCounter { data, mapping } = counter;

            Ok(Some((Box::new(data), mapping)))
        } else {
            let x_iter = match mapping
                .get_vector_iter(&Aesthetic::X(AestheticDomain::Discrete), data.as_ref())
            {
                Some(iter) => iter,
                None => {
                    return Err(crate::error::PlotError::missing_stat_input(
                        "Count",
                        Aesthetic::X(AestheticDomain::Discrete),
                    ));
                }
            };

            let mut counter = UngroupedValueCounter::new();
            visit_d(x_iter, &mut counter)?;
            let UngroupedValueCounter { data, mapping } = counter;
            Ok(Some((Box::new(data), mapping)))
        }
    }
}

struct UngroupedValueCounter {
    data: DataFrame,
    mapping: AesMap,
}

impl UngroupedValueCounter {
    fn new() -> Self {
        Self {
            data: DataFrame::new(),
            mapping: AesMap::new(),
        }
    }
}

impl DiscreteVectorVisitor for UngroupedValueCounter {
    type Output = ();
    fn visit<T: Vectorable + DiscreteType>(&mut self, values: impl Iterator<Item = T>) -> std::result::Result<Self::Output, PlotError> {
        let counts: HashMap<T::Sortable, i64> = {
            let mut map = HashMap::new();
            for val in values {
                let key = val.to_sortable();
                *map.entry(key).or_insert(0) += 1;
            }
            map
        };
        // Sort by x value
        let mut pairs: Vec<(T::Sortable, i64)> = counts.into_iter().collect();
        pairs.sort();
        let mut x_values: Vec<T> = Vec::with_capacity(pairs.len());
        let mut count_values: Vec<i64> = Vec::with_capacity(pairs.len());
        for (x, c) in pairs.into_iter() {
            x_values.push(T::from_sortable(x));
            count_values.push(c);
        }
        self.data.add_column("x", T::make_vector(x_values));
        self.data.add_column("count", Box::new(IntVec(count_values)));

        self.mapping.set(
            Aesthetic::X(AestheticDomain::Discrete),
            AesValue::column("x"),
        );
        self.mapping.set(
            Aesthetic::Y(AestheticDomain::Continuous),
            AesValue::column("count"),
        );
        Ok(())
    }
}

struct GroupedValueCounter {
    data: DataFrame,
    mapping: AesMap,
}

impl GroupedValueCounter {
    fn new() -> Self {
        Self {
            data: DataFrame::new(),
            mapping: AesMap::new(),
        }
    }
}

impl DiscreteDiscreteVisitor2 for GroupedValueCounter {
    fn visit<G: Vectorable + DiscreteType, T: Vectorable + DiscreteType>(
        &mut self,
        groups: impl Iterator<Item = G>,
        values: impl Iterator<Item = T>,
    ) {
        let mut counts: HashMap<(G::Sortable, T::Sortable), i64> = HashMap::new();

        for (val, group) in values.zip(groups) {
            let sortable = (group.to_sortable(), val.to_sortable());
            *counts.entry(sortable).or_insert(0) += 1;
        }

        // Sort by group then x value
        let mut pairs: Vec<((G::Sortable, T::Sortable), i64)> = counts.into_iter().collect();
        pairs.sort();

        let mut x_values: Vec<T> = Vec::with_capacity(pairs.len());
        let mut group_values: Vec<G> = Vec::with_capacity(pairs.len());
        let mut count_values: Vec<i64> = Vec::with_capacity(pairs.len());
        for ((g, x), v) in pairs.into_iter() {
            x_values.push(T::from_sortable(x));
            group_values.push(G::from_sortable(g));
            count_values.push(v);
        }

        self.data.add_column("x", T::make_vector(x_values));
        self.data.add_column("group", G::make_vector(group_values));
        self.data.add_column("count", Box::new(IntVec(count_values)));

        self.mapping.set(
            Aesthetic::X(AestheticDomain::Discrete),
            AesValue::column("x"),
        );
        self.mapping
            .set(Aesthetic::Group, AesValue::column("group"));
        self.mapping.set(
            Aesthetic::Y(AestheticDomain::Continuous),
            AesValue::column("count"),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::dataframe::{DataFrame, FloatVec, StrVec};

    #[test]
    fn test_count_basic() {
        // Create test data: x values [1, 1, 2, 2, 2, 3]
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(IntVec(vec![1, 1, 2, 2, 2, 3])));

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Discrete);

        let count = Count;
        let result = count.apply(Box::new(df), &mapping);
        assert!(result.is_ok());

        let option_result = result.unwrap();
        assert!(option_result.is_some());
        let (data, new_mapping) = option_result.unwrap();

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
        df.add_column("x", Box::new(IntVec(vec![5, 5, 5, 5])));

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Discrete);

        let count = Count;
        let (data, _) = count.apply(Box::new(df), &mapping).unwrap().unwrap();

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
        df.add_column("x", Box::new(IntVec(vec![1, 2, 3, 4, 5])));

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Discrete);

        let count = Count;
        let (data, _) = count.apply(Box::new(df), &mapping).unwrap().unwrap();

        let count_col = data.get("count").unwrap();
        let count_vals: Vec<i64> = count_col.iter_int().unwrap().collect();
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
    fn test_count_floats() {
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(FloatVec(vec![1.5, 1.5, 2.5, 2.5, 2.5, 3.5])));

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Discrete);

        let count = Count;
        let (data, _) = count.apply(Box::new(df), &mapping).unwrap().unwrap();

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
            Box::new(FloatVec(vec![1.0, f64::NAN, 2.0, f64::NAN, 1.0, 2.0, 2.0])),
        );

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Discrete);

        let count = Count;
        let (data, _) = count.apply(Box::new(df), &mapping).unwrap().unwrap();

        let x_col = data.get("x").unwrap();
        let x_vals: Vec<f64> = x_col.iter_float().unwrap().collect();
        // NaN values are dropped, so we only have 1.0 and 2.0
        assert_eq!(x_vals.len(), 2);
        assert_eq!(x_vals[0], 1.0);
        assert_eq!(x_vals[1], 2.0);

        let count_col = data.get("count").unwrap();
        let count_vals: Vec<i64> = count_col.iter_int().unwrap().collect();
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
        mapping.x("x", AestheticDomain::Discrete);

        let count = Count;
        let (data, _) = count.apply(Box::new(df), &mapping).unwrap().unwrap();

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
        mapping.x("x", AestheticDomain::Discrete);

        let count = Count;
        let (data, _) = count.apply(Box::new(df), &mapping).unwrap().unwrap();

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
        df.add_column(
            "x",
            Box::new(StrVec(vec![
                "test".to_string(),
                "test".to_string(),
                "test".to_string(),
            ])),
        );

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Discrete);

        let count = Count;
        let (data, _) = count.apply(Box::new(df), &mapping).unwrap().unwrap();

        // String x values are kept as strings (categorical scale will handle positioning)
        let x_col = data.get("x").unwrap();
        let x_vals: Vec<String> = x_col.iter_str().unwrap().map(|s| s.to_string()).collect();
        assert_eq!(x_vals, vec!["test"]);

        let count_col = data.get("count").unwrap();
        let count_vals: Vec<i64> = count_col.iter_int().unwrap().collect();
        assert_eq!(count_vals, vec![3]);
    }
}
