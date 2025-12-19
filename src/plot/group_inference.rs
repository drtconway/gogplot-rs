// Group inference for position adjustments

use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::data::DataSource;
use crate::scale::ScaleType;
use crate::utils::dataframe::DataFrame;

fn infer_grouping_aesthetics(mapping: &AesMap) -> Vec<Aesthetic> {
    let mut grouping_aesthetics = Vec::new();

    if mapping.contains(Aesthetic::Group) {
        grouping_aesthetics.push(Aesthetic::Group);
        return grouping_aesthetics;
    }

    for (aes, _value) in mapping.iter() {
        if aes.is_grouping() {
            grouping_aesthetics.push(*aes);
        }
    }
    grouping_aesthetics.sort();
    grouping_aesthetics
}

/// Infer the grouping, if any, and synthesize a Group aesthetic, creating a new mapping and data if needed.
pub fn establish_grouping(
    data: &dyn DataSource,
    mapping: &AesMap,
) -> (Option<Box<dyn DataSource>>, Option<AesMap>) {
    let grouping_aesthetics = infer_grouping_aesthetics(mapping);
    if grouping_aesthetics.is_empty() {
        return (None, None);
    }

    if grouping_aesthetics.len() == 1 {
        if grouping_aesthetics[0] == Aesthetic::Group {
            // Group is already set, nothing to do
            return (None, None);
        } else {
            // Single grouping aesthetic - map it to Group
            let mut new_mapping = mapping.clone();
            let value = mapping.get(&grouping_aesthetics[0]).unwrap();
            new_mapping.set(Aesthetic::Group, value.clone());
            return (Some(data.clone_box()), Some(new_mapping));
        }
    }

    // Multiple grouping aesthetics - create composite Group

    let mut new_data = DataFrame::from(data);

    // Create new mapping with Group aesthetic if not already present
    let mut new_mapping = mapping.clone();

    // Collect unique column names to avoid duplicates when multiple aesthetics map to same column
    let mut unique_columns = Vec::new();
    let mut seen_columns = std::collections::HashSet::new();

    for aes in &grouping_aesthetics {
        if let Some(AesValue::Column { name, .. }) = mapping.get(aes) {
            if !seen_columns.contains(name) {
                unique_columns.push(name.clone());
                seen_columns.insert(name.clone());
            }
        }
    }

    // Create a composite group column name from unique columns
    let base_group_col_name = unique_columns.join("_");

    let mut i = 1;
    let group_name = loop {
        let group_name = format!("{}_{}", base_group_col_name, i);
        if new_data.get(&group_name).is_none() {
            break group_name;
        }
        i += 1;
    };

    let mut group_iters: Vec<Box<dyn Iterator<Item = String>>> = Vec::new();
    for col_name in &unique_columns {
        if let Some(col) = data.get(col_name) {
            let iter = col.iter();
            let iter = match iter {
                crate::data::VectorIter::Str(s_iter) => {
                    Box::new(s_iter.map(|s| s.to_string())) as Box<dyn Iterator<Item = String>>
                }
                crate::data::VectorIter::Int(i_iter) => {
                    Box::new(i_iter.map(|v| v.to_string())) as Box<dyn Iterator<Item = String>>
                }
                crate::data::VectorIter::Float(f_iter) => {
                    Box::new(f_iter.map(|v| v.to_string())) as Box<dyn Iterator<Item = String>>
                }
                crate::data::VectorIter::Bool(b_iter) => {
                    Box::new(b_iter.map(|v| v.to_string())) as Box<dyn Iterator<Item = String>>
                }
            };
            group_iters.push(iter);
        }
    }
    let mut group_values: Vec<String> = Vec::new();
    for _i in 0..new_data.len() {
        let mut group_parts: Vec<String> = Vec::new();
        for iter in &mut group_iters {
            if let Some(value) = iter.next() {
                group_parts.push(value);
            }
        }
        let group_value = group_parts.join("_");
        group_values.push(group_value);
    }
    let group_column = crate::utils::dataframe::StrVec::from(group_values);
    new_data.add_column(&group_name, Box::new(group_column));

    new_mapping.set(
        Aesthetic::Group,
        AesValue::Column {
            name: group_name,
            hint: Some(ScaleType::Categorical),
            original_name: None,
        },
    );

    (Some(Box::new(new_data)), Some(new_mapping))
}
