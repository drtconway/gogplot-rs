//! Utilities for grouping data by aesthetic mappings

use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::data::DataSource;
use std::collections::HashMap;

/// Extract grouping column information from an aesthetic mapping
///
/// Returns a vector of (Aesthetic, column_name) pairs for all aesthetics that
/// are grouping aesthetics (Color, Fill, Shape, Linetype, Group) and are mapped
/// to columns (not constants).
pub fn get_grouping_columns(mapping: &AesMap) -> Vec<(Aesthetic, String)> {
    mapping
        .iter()
        .filter(|(aes, _)| aes.is_grouping())
        .filter_map(|(aes, aes_value)| match aes_value {
            AesValue::Column { name, .. } => Some((*aes, name.clone())),
            _ => None,
        })
        .collect()
}

/// Create composite group keys from data and grouping columns
///
/// Creates a unique string key for each row by concatenating values from all
/// grouping columns, separated by "__". This allows grouping by multiple
/// aesthetics simultaneously.
///
/// # Arguments
///
/// * `data` - The data source
/// * `group_cols` - List of (aesthetic, column_name) pairs to group by
///
/// # Returns
///
/// A vector of composite key strings, one per row
pub fn create_composite_keys(
    data: &dyn DataSource,
    group_cols: &[(Aesthetic, String)],
) -> Vec<String> {
    let n_rows = data.len();
    
    if group_cols.is_empty() {
        // No grouping - all rows in one group
        return vec!["__all__".to_string(); n_rows];
    }

    // Collect values from each grouping column
    let mut group_col_values: Vec<Vec<String>> = Vec::new();
    
    for (_aesthetic, col_name) in group_cols {
        let col = data.get(col_name.as_str()).unwrap();
        let values = if let Some(str_iter) = col.iter_str() {
            str_iter.map(|s| s.to_string()).collect()
        } else if let Some(int_iter) = col.iter_int() {
            int_iter.map(|v| v.to_string()).collect()
        } else if let Some(float_iter) = col.iter_float() {
            float_iter.map(|v| v.to_string()).collect()
        } else {
            vec![String::new(); n_rows]
        };
        group_col_values.push(values);
    }

    // Create composite keys by joining values from all grouping columns
    (0..n_rows)
        .map(|i| {
            group_col_values
                .iter()
                .map(|col_vals| col_vals[i].as_str())
                .collect::<Vec<_>>()
                .join("__")
        })
        .collect()
}

/// Split data into groups based on composite keys
///
/// Groups data values by their composite key. Useful for applying
/// transformations to each group separately.
///
/// # Type Parameters
///
/// * `T` - The type of values to group (typically f64)
///
/// # Arguments
///
/// * `values` - Vector of values to group
/// * `composite_keys` - Composite key for each value
///
/// # Returns
///
/// A HashMap mapping composite keys to vectors of values
pub fn group_by_key<T: Clone>(
    values: &[T],
    composite_keys: &[String],
) -> HashMap<String, Vec<T>> {
    let mut groups: HashMap<String, Vec<T>> = HashMap::new();
    
    for (i, key) in composite_keys.iter().enumerate() {
        if i < values.len() {
            groups.entry(key.clone()).or_default().push(values[i].clone());
        }
    }
    
    groups
}

/// Split data into groups and return them in sorted order by key
///
/// Like `group_by_key`, but returns a Vec sorted by composite key for
/// deterministic ordering. This ensures consistent output regardless of
/// HashMap iteration order.
///
/// # Type Parameters
///
/// * `T` - The type of values to group (typically f64)
///
/// # Arguments
///
/// * `values` - Vector of values to group
/// * `composite_keys` - Composite key for each value
///
/// # Returns
///
/// A Vec of (key, values) pairs sorted by key
pub fn group_by_key_sorted<T: Clone>(
    values: &[T],
    composite_keys: &[String],
) -> Vec<(String, Vec<T>)> {
    let groups = group_by_key(values, composite_keys);
    let mut sorted_groups: Vec<(String, Vec<T>)> = groups.into_iter().collect();
    sorted_groups.sort_by(|(a, _), (b, _)| a.cmp(b));
    sorted_groups
}

/// Extract the value for a specific aesthetic from a composite key
///
/// Composite keys are formed by joining values from multiple aesthetics with "__".
/// This function extracts the value for a specific aesthetic from the key.
///
/// # Arguments
///
/// * `composite_key` - The composite key string
/// * `group_cols` - The list of (aesthetic, column_name) pairs used to create the key
/// * `aesthetic` - The aesthetic to extract
///
/// # Returns
///
/// The value for the specified aesthetic, or an empty string if not found
pub fn extract_aesthetic_value(
    composite_key: &str,
    group_cols: &[(Aesthetic, String)],
    aesthetic: &Aesthetic,
) -> String {
    let parts: Vec<&str> = composite_key.split("__").collect();
    
    group_cols
        .iter()
        .position(|(aes, _)| aes == aesthetic)
        .and_then(|idx| parts.get(idx))
        .unwrap_or(&"")
        .to_string()
}

/// Extract all aesthetic values from a composite key
///
/// Splits a composite key back into individual aesthetic values, in the same
/// order as the grouping columns.
///
/// # Arguments
///
/// * `composite_key` - The composite key string
/// * `group_cols` - The list of (aesthetic, column_name) pairs used to create the key
///
/// # Returns
///
/// A vector of (aesthetic, value) pairs
pub fn split_composite_key(
    composite_key: &str,
    group_cols: &[(Aesthetic, String)],
) -> Vec<(Aesthetic, String)> {
    let parts: Vec<&str> = composite_key.split("__").collect();
    
    group_cols
        .iter()
        .enumerate()
        .map(|(i, (aes, _col_name))| {
            let value = parts.get(i).unwrap_or(&"").to_string();
            (*aes, value)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{aesthetics::AestheticDomain, utils::dataframe::{DataFrame, StrVec}};

    #[test]
    fn test_create_composite_keys() {
        let mut df = DataFrame::new();
        df.add_column("category", Box::new(StrVec(vec!["A".to_string(), "B".to_string(), "A".to_string()])));
        df.add_column("group", Box::new(StrVec(vec!["X".to_string(), "Y".to_string(), "X".to_string()])));
        
        let group_cols = vec![
            (Aesthetic::Fill(AestheticDomain::Discrete), "category".to_string()),
            (Aesthetic::Shape, "group".to_string()),
        ];
        
        let keys = create_composite_keys(&df, &group_cols);
        
        assert_eq!(keys.len(), 3);
        assert_eq!(keys[0], "A__X");
        assert_eq!(keys[1], "B__Y");
        assert_eq!(keys[2], "A__X");
    }

    #[test]
    fn test_group_by_key() {
        let values = vec![1.0, 2.0, 3.0, 4.0];
        let keys = vec!["A".to_string(), "B".to_string(), "A".to_string(), "B".to_string()];
        
        let groups = group_by_key(&values, &keys);
        
        assert_eq!(groups.len(), 2);
        assert_eq!(groups.get("A").unwrap(), &vec![1.0, 3.0]);
        assert_eq!(groups.get("B").unwrap(), &vec![2.0, 4.0]);
    }

    #[test]
    fn test_extract_aesthetic_value() {
        let group_cols = vec![
            (Aesthetic::Fill(AestheticDomain::Discrete), "category".to_string()),
            (Aesthetic::Shape, "group".to_string()),
        ];
        
        let key = "A__X";
        assert_eq!(extract_aesthetic_value(key, &group_cols, &Aesthetic::Fill(AestheticDomain::Discrete)), "A");
        assert_eq!(extract_aesthetic_value(key, &group_cols, &Aesthetic::Shape), "X");
    }
}
