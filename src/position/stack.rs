// Stack position adjustment for bars

use super::Position;
use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::data::{DataSource, DiscreteType};
use crate::error::PlotError;
use crate::utils::data::{DiscreteVectorVisitor, Vectorable, visit_d};
use crate::utils::dataframe::{DataFrame, FloatVec};
use std::collections::HashMap;
use std::sync::Arc;

/// Stack position adjustment
///
/// Stacks bars on top of each other at the same x position.
/// Requires grouping aesthetics (fill, color, etc.) to determine which bars to stack.
pub struct Stack;

impl Position for Stack {
    fn apply(
        &self,
        data: &Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<Option<(Box<dyn DataSource>, AesMap)>, PlotError>
    {
        if !mapping.contains(Aesthetic::Group) {
            // No grouping aesthetic, cannot stack
            return Err(PlotError::MissingAesthetic {
                aesthetic: Aesthetic::Group,
            });
        }

        let mut y_like_data: HashMap<Aesthetic, Vec<f64>> = HashMap::new();
        for aes in mapping.aesthetics() {
            if aes.is_y_like() && aes.is_continuous() {
                let y_like_values = mapping.get_iter_float(aes, data.as_ref()).unwrap();
                let y_like_values = y_like_values.collect::<Vec<f64>>();
                y_like_data.insert(*aes, y_like_values);
            }
        }
        let group_values = mapping
        .get_vector_iter(&Aesthetic::Group, data.as_ref()).unwrap();

        let mut grouped_stacker = GroupStacker::new(y_like_data);

        let _max_val = visit_d(group_values, &mut grouped_stacker)?;

        let mut y_like_data = grouped_stacker.y_like_data;

        // Create new dataframe with all original columns, replacing y-like aesthetics with stacked versions
        let new_data: DataFrame = DataFrame::new();
        let mut new_mapping = AesMap::new();
        for (aes, aes_value) in mapping.iter() {
            if let Some(stacked_values) = y_like_data.remove(aes) {
                let original_name = mapping.get(aes).and_then(|v| v.as_column_name()).map(|s| s.to_string());
                new_mapping.set(*aes, AesValue::vector(Arc::new( FloatVec(stacked_values)), original_name));
            } else {
                new_mapping.set(*aes, aes_value.clone());
            }
        }
        
        Ok(Some((Box::new(new_data), new_mapping) ))
    }
}

struct GroupStacker {
    y_like_data: HashMap<Aesthetic, Vec<f64>>,
}

impl GroupStacker {
    fn new(y_like_data: HashMap<Aesthetic, Vec<f64>>) -> Self {
        Self { y_like_data }
    }
}

impl DiscreteVectorVisitor for GroupStacker {
    type Output = f64;

    fn visit<T: Vectorable + DiscreteType>(&mut self, group_values: impl Iterator<Item = T>) -> std::result::Result<Self::Output, PlotError> {

        let group_values: Vec<T::Sortable> = group_values.map(|v| v.to_sortable()).collect();

        // Collect maxima per group
        let mut maxima: HashMap<T::Sortable, _> = HashMap::new();
        for (i, group_value) in group_values.iter().enumerate() {
            for vals in self.y_like_data.values_mut() {
                let entry = maxima.entry(group_value.clone()).or_insert(f64::NEG_INFINITY);
                if *entry < vals[i] {
                    *entry = vals[i];
                }
            }
        }

        // Get the keys in sorted order
        let mut sorted_keys: Vec<_> = maxima.keys().cloned().collect();
        sorted_keys.sort();

        // Compute the per-group offsets
        let mut cumulative = 0.0;
        for key in &sorted_keys {
            let max_val = maxima.get_mut(key).unwrap();
            let val = *max_val;
            *max_val = cumulative;
            cumulative += val;
        }

        let offsets = maxima;

        // Apply offsets to y-like data
        for (_aes, vals) in self.y_like_data.iter_mut() {
            for (i, group_value) in group_values.iter().enumerate() {
                if let Some(offset) = offsets.get(group_value) {
                    vals[i] += offset;
                }
            }
        }

        Ok(cumulative)
    }
}
