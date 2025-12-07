use crate::aesthetics::{AesMap, AesValue, Aesthetic, AestheticDomain};
use crate::data::{DataSource, PrimitiveType, VectorIter};
use crate::error::Result;
use crate::stat::StatTransform;
use crate::utils::dataframe::{BoolVec, DataFrame, FloatVec, IntVec, StrVec};
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

            let mut new_data = DataFrame::new();
            let mut new_mapping = AesMap::new();

            count_grouped_values(x_iter, group_iter, &mut new_data, &mut new_mapping);

            Ok(Some((Box::new(new_data), new_mapping)))
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

            let mut new_data = DataFrame::new();
            let mut new_mapping = AesMap::new();

            match x_iter {
                VectorIter::Int(int_iter) => {
                    let (x_values, counts) = count_ungrouped_values(int_iter);
                    new_data.add_column("x", Box::new(IntVec(x_values)));
                    new_data.add_column("count", Box::new(IntVec(counts)));
                    new_mapping.set(
                        Aesthetic::X(AestheticDomain::Discrete),
                        AesValue::column("x"),
                    );
                    new_mapping.set(
                        Aesthetic::Y(AestheticDomain::Continuous),
                        AesValue::column("count"),
                    );
                    Ok(Some((Box::new(new_data), new_mapping)))
                }
                VectorIter::Float(float_iter) => {
                    let (x_values, counts) = count_ungrouped_values(float_iter);
                    new_data.add_column("x", Box::new(FloatVec(x_values)));
                    new_data.add_column("count", Box::new(IntVec(counts)));
                    new_mapping.set(
                        Aesthetic::X(AestheticDomain::Discrete),
                        AesValue::column("x"),
                    );
                    new_mapping.set(
                        Aesthetic::Y(AestheticDomain::Continuous),
                        AesValue::column("count"),
                    );
                    Ok(Some((Box::new(new_data), new_mapping)))
                }
                VectorIter::Str(str_iter) => {
                    let (x_values, counts) = count_ungrouped_values(str_iter.map(|s| s.to_string()));
                    new_data.add_column("x", Box::new(StrVec(x_values)));
                    new_data.add_column("count", Box::new(IntVec(counts)));
                    new_mapping.set(
                        Aesthetic::X(AestheticDomain::Discrete),
                        AesValue::column("x"),
                    );
                    new_mapping.set(
                        Aesthetic::Y(AestheticDomain::Continuous),
                        AesValue::column("count"),
                    );
                    Ok(Some((Box::new(new_data), new_mapping)))
                }
                VectorIter::Bool(iterator) => {
                    let (x_values, counts) = count_ungrouped_values(iterator);
                    new_data.add_column("x", Box::new(BoolVec(x_values)));
                    new_data.add_column("count", Box::new(IntVec(counts)));
                    new_mapping.set(
                        Aesthetic::X(AestheticDomain::Discrete),
                        AesValue::column("x"),
                    );
                    new_mapping.set(
                        Aesthetic::Y(AestheticDomain::Continuous),
                        AesValue::column("count"),
                    );
                    Ok(Some((Box::new(new_data), new_mapping)))
                }
            }
        }
    }
}

fn count_ungrouped_values<T: PrimitiveType>(
    values: impl Iterator<Item = T>,
) -> (Vec<T>, Vec<i64>) {
    let mut counts: HashMap<T::Sortable, i64> = HashMap::new();

    for val in values {
        let key = val.to_sortable();
        *counts.entry(key).or_insert(0) += 1;
    }

    // Sort by x value
    let mut pairs: Vec<(T::Sortable, i64)> = counts.into_iter().collect();
    pairs.sort();

    let mut x_values: Vec<T> = Vec::with_capacity(pairs.len());
    let mut count_values: Vec<i64> = Vec::with_capacity(pairs.len());
    for (x, c) in pairs.into_iter() {
        x_values.push(T::from_sortable(x));
        count_values.push(c);
    }

    (x_values, count_values)
}

fn count_grouped_values<'a>(
    values: VectorIter<'a>,
    groups: VectorIter<'a>,
    data: &mut DataFrame,
    mapping: &mut AesMap,
) {
    match values {
        VectorIter::Int(int_iter) => {
            let x_values = count_grouped_values_outer(int_iter, groups, data, mapping);
            data.add_column("x", Box::new(IntVec(x_values)));
            mapping.set(
                Aesthetic::X(AestheticDomain::Discrete),
                AesValue::column("x"),
            );
        }
        VectorIter::Float(float_iter) => {
            let x_values = count_grouped_values_outer(float_iter, groups, data, mapping);
            data.add_column("x", Box::new(FloatVec(x_values)));
            mapping.set(
                Aesthetic::X(AestheticDomain::Discrete),
                AesValue::column("x"),
            );
        }
        VectorIter::Str(str_iter) => {
            let x_values =
                count_grouped_values_outer(str_iter.map(|s| s.to_string()), groups, data, mapping);
            data.add_column("x", Box::new(StrVec(x_values)));
            mapping.set(
                Aesthetic::X(AestheticDomain::Discrete),
                AesValue::column("x"),
            );
        }
        VectorIter::Bool(bool_iter) => {
            let x_values = count_grouped_values_outer(bool_iter, groups, data, mapping);
            data.add_column("x", Box::new(BoolVec(x_values)));
            mapping.set(
                Aesthetic::X(AestheticDomain::Discrete),
                AesValue::column("x"),
            );
        }
    }
}

fn count_grouped_values_outer<'a, T: PrimitiveType>(
    values: impl Iterator<Item = T>,
    groups: VectorIter<'a>,
    data: &mut DataFrame,
    mapping: &mut AesMap,
) -> Vec<T> {
    match groups {
        VectorIter::Int(iterator) => {
            let (x_values, group_values, count_values) =
                count_grouped_values_inner(values, iterator);
            data.add_column("group", Box::new(IntVec(group_values)));
            mapping.set(Aesthetic::Group, AesValue::column("group"));

            data.add_column("count", Box::new(IntVec(count_values)));
            mapping.set(
                Aesthetic::Y(AestheticDomain::Continuous),
                AesValue::column("count"),
            );
            x_values
        }
        VectorIter::Float(iterator) => {
            let (x_values, group_values, count_values) =
                count_grouped_values_inner(values, iterator);
            data.add_column("group", Box::new(FloatVec(group_values)));
            mapping.set(Aesthetic::Group, AesValue::column("group"));

            data.add_column("count", Box::new(IntVec(count_values)));
            mapping.set(
                Aesthetic::Y(AestheticDomain::Continuous),
                AesValue::column("count"),
            );
            x_values
        }
        VectorIter::Str(iterator) => {
            let (x_values, group_values, count_values) =
                count_grouped_values_inner(values, iterator.map(|s| s.to_string()));
            data.add_column("group", Box::new(StrVec(group_values)));
            mapping.set(Aesthetic::Group, AesValue::column("group"));

            data.add_column("count", Box::new(IntVec(count_values)));
            mapping.set(
                Aesthetic::Y(AestheticDomain::Continuous),
                AesValue::column("count"),
            );
            x_values
        }
        VectorIter::Bool(iterator) => {
            let (x_values, group_values, count_values) =
                count_grouped_values_inner(values, iterator);
            data.add_column("group", Box::new(BoolVec(group_values)));
            mapping.set(Aesthetic::Group, AesValue::column("group"));

            data.add_column("count", Box::new(IntVec(count_values)));
            mapping.set(
                Aesthetic::Y(AestheticDomain::Continuous),
                AesValue::column("count"),
            );
            x_values
        }
    }
}

fn count_grouped_values_inner<T: PrimitiveType, G: PrimitiveType>(
    values: impl Iterator<Item = T>,
    groups: impl Iterator<Item = G>,
) -> (Vec<T>, Vec<G>, Vec<i64>) {
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

    (x_values, group_values, count_values)
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
