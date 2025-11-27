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
        },
    );

    (Some(Box::new(new_data)), Some(new_mapping))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::dataframe::{FloatVec, IntVec, StrVec};

    #[test]
    fn test_no_grouping_aesthetics() {
        // When there are no grouping aesthetics, should return (None, None)
        let mut data = DataFrame::new();
        data.add_column("x", Box::new(FloatVec::from(vec![1.0, 2.0, 3.0])));
        data.add_column("y", Box::new(FloatVec::from(vec![4.0, 5.0, 6.0])));

        let mut mapping = AesMap::new();
        mapping.set(
            Aesthetic::X,
            AesValue::Column {
                name: "x".to_string(),
                hint: Some(ScaleType::Continuous),
            },
        );
        mapping.set(
            Aesthetic::Y,
            AesValue::Column {
                name: "y".to_string(),
                hint: Some(ScaleType::Continuous),
            },
        );

        let (new_data, new_mapping) = establish_grouping(&data, &mapping);
        assert!(new_data.is_none());
        assert!(new_mapping.is_none());
    }

    #[test]
    fn test_existing_group_aesthetic() {
        // When Group aesthetic is already present, should return (None, None)
        let mut data = DataFrame::new();
        data.add_column("x", Box::new(FloatVec::from(vec![1.0, 2.0, 3.0])));
        data.add_column("g", Box::new(StrVec::from(vec!["a", "b", "c"])));

        let mut mapping = AesMap::new();
        mapping.set(
            Aesthetic::X,
            AesValue::Column {
                name: "x".to_string(),
                hint: Some(ScaleType::Continuous),
            },
        );
        mapping.set(
            Aesthetic::Group,
            AesValue::Column {
                name: "g".to_string(),
                hint: Some(ScaleType::Categorical),
            },
        );

        let (new_data, new_mapping) = establish_grouping(&data, &mapping);
        assert!(new_data.is_none());
        assert!(new_mapping.is_none());
    }

    #[test]
    fn test_single_grouping_aesthetic_fill() {
        // When there's a single grouping aesthetic (Fill), it should be mapped to Group
        let mut data = DataFrame::new();
        data.add_column("x", Box::new(FloatVec::from(vec![1.0, 2.0, 3.0])));
        data.add_column("category", Box::new(StrVec::from(vec!["a", "b", "c"])));

        let mut mapping = AesMap::new();
        mapping.set(
            Aesthetic::X,
            AesValue::Column {
                name: "x".to_string(),
                hint: Some(ScaleType::Continuous),
            },
        );
        mapping.set(
            Aesthetic::Fill,
            AesValue::Column {
                name: "category".to_string(),
                hint: Some(ScaleType::Categorical),
            },
        );

        let (new_data, new_mapping) = establish_grouping(&data, &mapping);
        assert!(new_data.is_some());
        assert!(new_mapping.is_some());

        let new_mapping = new_mapping.unwrap();
        // Should have Group aesthetic mapped to the same column as Fill
        let group_val = new_mapping.get(&Aesthetic::Group);
        assert!(group_val.is_some());
        if let Some(AesValue::Column { name, .. }) = group_val {
            assert_eq!(name, "category");
        } else {
            panic!("Expected Group to be mapped to a column");
        }
    }

    #[test]
    fn test_single_grouping_aesthetic_color() {
        // When there's a single grouping aesthetic (Color), it should be mapped to Group
        let mut data = DataFrame::new();
        data.add_column("x", Box::new(FloatVec::from(vec![1.0, 2.0, 3.0])));
        data.add_column("species", Box::new(StrVec::from(vec!["cat", "dog", "bird"])));

        let mut mapping = AesMap::new();
        mapping.set(
            Aesthetic::X,
            AesValue::Column {
                name: "x".to_string(),
                hint: Some(ScaleType::Continuous),
            },
        );
        mapping.set(
            Aesthetic::Color,
            AesValue::Column {
                name: "species".to_string(),
                hint: Some(ScaleType::Categorical),
            },
        );

        let (new_data, new_mapping) = establish_grouping(&data, &mapping);
        assert!(new_data.is_some());
        assert!(new_mapping.is_some());

        let new_mapping = new_mapping.unwrap();
        let group_val = new_mapping.get(&Aesthetic::Group);
        assert!(group_val.is_some());
        if let Some(AesValue::Column { name, .. }) = group_val {
            assert_eq!(name, "species");
        } else {
            panic!("Expected Group to be mapped to a column");
        }
    }

    #[test]
    fn test_multiple_grouping_aesthetics_string_columns() {
        // When there are multiple grouping aesthetics, should create composite group column
        let mut data = DataFrame::new();
        data.add_column("x", Box::new(FloatVec::from(vec![1.0, 2.0, 3.0, 4.0])));
        data.add_column("category", Box::new(StrVec::from(vec!["a", "b", "a", "b"])));
        data.add_column("region", Box::new(StrVec::from(vec!["north", "south", "south", "north"])));

        let mut mapping = AesMap::new();
        mapping.set(
            Aesthetic::X,
            AesValue::Column {
                name: "x".to_string(),
                hint: Some(ScaleType::Continuous),
            },
        );
        mapping.set(
            Aesthetic::Fill,
            AesValue::Column {
                name: "category".to_string(),
                hint: Some(ScaleType::Categorical),
            },
        );
        mapping.set(
            Aesthetic::Color,
            AesValue::Column {
                name: "region".to_string(),
                hint: Some(ScaleType::Categorical),
            },
        );

        let (new_data, new_mapping) = establish_grouping(&data, &mapping);
        assert!(new_data.is_some());
        assert!(new_mapping.is_some());

        let new_data = new_data.unwrap();
        let new_mapping = new_mapping.unwrap();

        // Should have a new Group column
        let group_val = new_mapping.get(&Aesthetic::Group);
        assert!(group_val.is_some());
        
        if let Some(AesValue::Column { name, .. }) = group_val {
            // The new column should exist in the data
            let group_col = new_data.get(name);
            assert!(group_col.is_some());

            // Check the composite values
            let group_col = group_col.unwrap();
            let mut values = Vec::new();
            if let crate::data::VectorIter::Str(iter) = group_col.iter() {
                values = iter.map(|s| s.to_string()).collect();
            }
            
            assert_eq!(values.len(), 4);
            // Aesthetics are sorted, so Color comes before Fill (region_category order)
            assert_eq!(values[0], "north_a");
            assert_eq!(values[1], "south_b");
            assert_eq!(values[2], "south_a");
            assert_eq!(values[3], "north_b");
        } else {
            panic!("Expected Group to be mapped to a column");
        }
    }

    #[test]
    fn test_multiple_grouping_aesthetics_mixed_types() {
        // Test with mixed data types (string and int)
        let mut data = DataFrame::new();
        data.add_column("x", Box::new(FloatVec::from(vec![1.0, 2.0, 3.0])));
        data.add_column("category", Box::new(StrVec::from(vec!["a", "b", "c"])));
        data.add_column("level", Box::new(IntVec::from(vec![1, 2, 1])));

        let mut mapping = AesMap::new();
        mapping.set(
            Aesthetic::X,
            AesValue::Column {
                name: "x".to_string(),
                hint: Some(ScaleType::Continuous),
            },
        );
        mapping.set(
            Aesthetic::Fill,
            AesValue::Column {
                name: "category".to_string(),
                hint: Some(ScaleType::Categorical),
            },
        );
        mapping.set(
            Aesthetic::Shape,
            AesValue::Column {
                name: "level".to_string(),
                hint: Some(ScaleType::Categorical),
            },
        );

        let (new_data, new_mapping) = establish_grouping(&data, &mapping);
        assert!(new_data.is_some());
        assert!(new_mapping.is_some());

        let new_data = new_data.unwrap();
        let new_mapping = new_mapping.unwrap();

        if let Some(AesValue::Column { name, .. }) = new_mapping.get(&Aesthetic::Group) {
            let group_col = new_data.get(name).unwrap();
            let mut values = Vec::new();
            if let crate::data::VectorIter::Str(iter) = group_col.iter() {
                values = iter.map(|s| s.to_string()).collect();
            }
            
            assert_eq!(values.len(), 3);
            assert_eq!(values[0], "a_1");
            assert_eq!(values[1], "b_2");
            assert_eq!(values[2], "c_1");
        } else {
            panic!("Expected Group to be mapped to a column");
        }
    }

    #[test]
    fn test_infer_grouping_aesthetics_empty() {
        let mapping = AesMap::new();
        let aesthetics = infer_grouping_aesthetics(&mapping);
        assert_eq!(aesthetics.len(), 0);
    }

    #[test]
    fn test_infer_grouping_aesthetics_with_group() {
        let mut mapping = AesMap::new();
        mapping.set(
            Aesthetic::Group,
            AesValue::Column {
                name: "g".to_string(),
                hint: Some(ScaleType::Categorical),
            },
        );
        mapping.set(
            Aesthetic::Fill,
            AesValue::Column {
                name: "f".to_string(),
                hint: Some(ScaleType::Categorical),
            },
        );

        let aesthetics = infer_grouping_aesthetics(&mapping);
        // Should only return Group when it's present
        assert_eq!(aesthetics.len(), 1);
        assert_eq!(aesthetics[0], Aesthetic::Group);
    }

    #[test]
    fn test_infer_grouping_aesthetics_multiple() {
        let mut mapping = AesMap::new();
        mapping.set(
            Aesthetic::Fill,
            AesValue::Column {
                name: "f".to_string(),
                hint: Some(ScaleType::Categorical),
            },
        );
        mapping.set(
            Aesthetic::Color,
            AesValue::Column {
                name: "c".to_string(),
                hint: Some(ScaleType::Categorical),
            },
        );
        mapping.set(
            Aesthetic::X,
            AesValue::Column {
                name: "x".to_string(),
                hint: Some(ScaleType::Continuous),
            },
        );

        let aesthetics = infer_grouping_aesthetics(&mapping);
        // Should return grouping aesthetics (Fill, Color) but not X
        assert!(aesthetics.contains(&Aesthetic::Fill));
        assert!(aesthetics.contains(&Aesthetic::Color));
        assert!(!aesthetics.contains(&Aesthetic::X));
    }

    #[test]
    fn test_composite_group_column_unique_name() {
        // Test that if there's a naming conflict, it generates a unique name
        let mut data = DataFrame::new();
        data.add_column("x", Box::new(FloatVec::from(vec![1.0, 2.0])));
        data.add_column("a", Box::new(StrVec::from(vec!["x", "y"])));
        data.add_column("b", Box::new(StrVec::from(vec!["1", "2"])));
        // Add a column with the name that would be generated (Color comes before Fill when sorted)
        data.add_column("b_a_1", Box::new(StrVec::from(vec!["conflict", "conflict"])));

        let mut mapping = AesMap::new();
        mapping.set(
            Aesthetic::Fill,
            AesValue::Column {
                name: "a".to_string(),
                hint: Some(ScaleType::Categorical),
            },
        );
        mapping.set(
            Aesthetic::Color,
            AesValue::Column {
                name: "b".to_string(),
                hint: Some(ScaleType::Categorical),
            },
        );

        let (new_data, new_mapping) = establish_grouping(&data, &mapping);
        assert!(new_data.is_some());
        assert!(new_mapping.is_some());

        let new_data = new_data.unwrap();
        let new_mapping = new_mapping.unwrap();

        if let Some(AesValue::Column { name, .. }) = new_mapping.get(&Aesthetic::Group) {
            // Should generate "b_a_2" instead of "b_a_1" because of conflict
            assert_eq!(name, "b_a_2");
            assert!(new_data.get(name).is_some());
        } else {
            panic!("Expected Group to be mapped to a column");
        }
    }

    #[test]
    fn test_multiple_grouping_aesthetics_same_column() {
        // When multiple grouping aesthetics map to the same column, it should only use it once
        let mut data = DataFrame::new();
        data.add_column("x", Box::new(FloatVec::from(vec![1.0, 2.0, 3.0])));
        data.add_column("category", Box::new(StrVec::from(vec!["a", "b", "c"])));

        let mut mapping = AesMap::new();
        mapping.set(
            Aesthetic::X,
            AesValue::Column {
                name: "x".to_string(),
                hint: Some(ScaleType::Continuous),
            },
        );
        // Both Fill and Color map to the same column
        mapping.set(
            Aesthetic::Fill,
            AesValue::Column {
                name: "category".to_string(),
                hint: Some(ScaleType::Categorical),
            },
        );
        mapping.set(
            Aesthetic::Color,
            AesValue::Column {
                name: "category".to_string(),
                hint: Some(ScaleType::Categorical),
            },
        );

        let (new_data, new_mapping) = establish_grouping(&data, &mapping);
        assert!(new_data.is_some());
        assert!(new_mapping.is_some());

        let new_data = new_data.unwrap();
        let new_mapping = new_mapping.unwrap();

        if let Some(AesValue::Column { name, .. }) = new_mapping.get(&Aesthetic::Group) {
            let group_col = new_data.get(name).unwrap();
            let mut values = Vec::new();
            if let crate::data::VectorIter::Str(iter) = group_col.iter() {
                values = iter.map(|s| s.to_string()).collect();
            }
            
            assert_eq!(values.len(), 3);
            // Should just be "a", "b", "c" not "a_a", "b_b", "c_c"
            assert_eq!(values[0], "a");
            assert_eq!(values[1], "b");
            assert_eq!(values[2], "c");
        } else {
            panic!("Expected Group to be mapped to a column");
        }
    }
}
