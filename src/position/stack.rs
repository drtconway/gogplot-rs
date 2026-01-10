// Stack position adjustment for bars

use super::Position;
use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::error::PlotError;
use std::collections::HashMap;

/// Stack position adjustment
///
/// Stacks bars on top of each other at the same x position.
/// Requires grouping aesthetics (fill, color, etc.) to determine which bars to stack.
pub struct Stack;

impl Position for Stack {
    fn apply(
        &self,
        mapping: &AesMap,
    ) -> Result<Option<AesMap>, PlotError>
    {
        if !mapping.contains(Aesthetic::Group) {
            // No grouping aesthetic, cannot stack
            return Err(PlotError::MissingAesthetic {
                aesthetic: Aesthetic::Group,
            });
        }

        // Get X values (should be discrete for stacking)
        let x_aes = if mapping.contains(Aesthetic::X(crate::aesthetics::AestheticDomain::Discrete)) {
            Aesthetic::X(crate::aesthetics::AestheticDomain::Discrete)
        } else if mapping.contains(Aesthetic::X(crate::aesthetics::AestheticDomain::Continuous)) {
            Aesthetic::X(crate::aesthetics::AestheticDomain::Continuous)
        } else {
            return Err(PlotError::MissingAesthetic {
                aesthetic: Aesthetic::X(crate::aesthetics::AestheticDomain::Discrete),
            });
        };

        let x_iter = mapping.get_resolved_iter(&x_aes).unwrap();
        let group_iter = mapping.get_resolved_iter(&Aesthetic::Group).unwrap();
        
        // Collect X and Group values
        let x_values: Vec<String> = match x_iter {
            crate::data::VectorIter::Int(it) => it.map(|v| v.to_string()).collect(),
            crate::data::VectorIter::Str(it) => it.map(|s| s.to_string()).collect(),
            crate::data::VectorIter::Float(it) => it.map(|f| f.to_string()).collect(),
            crate::data::VectorIter::Bool(it) => it.map(|b| b.to_string()).collect(),
        };
        
        let group_values: Vec<String> = match group_iter {
            crate::data::VectorIter::Int(it) => it.map(|v| v.to_string()).collect(),
            crate::data::VectorIter::Str(it) => it.map(|s| s.to_string()).collect(),
            crate::data::VectorIter::Float(it) => it.map(|f| f.to_string()).collect(),
            crate::data::VectorIter::Bool(it) => it.map(|b| b.to_string()).collect(),
        };

        // Get Y values
        let y_values: Vec<f64> = if let Some(y_iter) = mapping.get_resolved_float(&Aesthetic::Y(crate::aesthetics::AestheticDomain::Continuous)) {
            y_iter.collect()
        } else {
            return Err(PlotError::MissingAesthetic {
                aesthetic: Aesthetic::Y(crate::aesthetics::AestheticDomain::Continuous),
            });
        };

        // Group data by X, then stack by Group within each X
        let mut x_group_map: HashMap<String, HashMap<String, Vec<(usize, f64)>>> = HashMap::new();
        
        for (i, (x_val, group_val)) in x_values.iter().zip(group_values.iter()).enumerate() {
            x_group_map
                .entry(x_val.clone())
                .or_insert_with(HashMap::new)
                .entry(group_val.clone())
                .or_insert_with(Vec::new)
                .push((i, y_values[i]));
        }

        // Compute stacked Y values and offsets for each X position
        let mut new_y_values = vec![0.0; y_values.len()];
        let mut y_offsets = vec![0.0; y_values.len()];
        
        for (_x_val, groups) in x_group_map.iter() {
            // Get unique groups at this X position and sort them
            let mut unique_groups: Vec<String> = groups.keys().cloned().collect();
            unique_groups.sort();
            
            // Stack groups in order
            let mut cumulative = 0.0;
            for group_val in unique_groups {
                if let Some(indices_and_values) = groups.get(&group_val) {
                    for (idx, y_val) in indices_and_values {
                        y_offsets[*idx] = cumulative;
                        new_y_values[*idx] = cumulative + y_val;
                    }
                    // Move cumulative up by the maximum Y in this group
                    if let Some(max_y) = indices_and_values.iter().map(|(_, y)| y).max_by(|a, b| a.partial_cmp(b).unwrap()) {
                        cumulative += max_y;
                    }
                }
            }
        }

        // Create new mapping with stacked values
        let mut new_mapping = AesMap::new();
        for (aes, aes_value) in mapping.iter() {
            if aes == &Aesthetic::Y(crate::aesthetics::AestheticDomain::Continuous) {
                new_mapping.set(*aes, AesValue::vector(new_y_values.clone(), None));
            } else {
                new_mapping.set(*aes, aes_value.clone());
            }
        }
        
        // Add YOffset for the bottom of each stacked bar
        new_mapping.set(Aesthetic::YOffset, AesValue::vector(y_offsets, None));
        
        Ok(Some(new_mapping))
    }
}

impl Default for Stack {
    fn default() -> Self {
        Self {}
    }
}
