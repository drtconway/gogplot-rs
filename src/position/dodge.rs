use super::PositionAdjust;
use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain};
use crate::data::{DataSource, DiscreteType};
use crate::error::PlotError;
use crate::utils::data::{DiscreteDiscreteVisitor2, Vectorable};
use std::collections::{HashMap, HashSet};

/// Dodge position adjustment
///
/// Places bars side-by-side at the same x position, making each bar narrower
/// to fit multiple groups without overlapping.
pub struct Dodge {
    /// Total width for all bars at one x position (None = auto-detect)
    pub width: Option<f64>,
    /// Padding between dodged bars as a fraction of bar width (default 0.1)
    pub padding: f64,
}

impl Default for Dodge {
    fn default() -> Self {
        Self {
            width: None,
            padding: 0.1, // 10% padding between bars within a cluster
        }
    }
}

impl PositionAdjust for Dodge {
    fn apply(
        &self,
        data: &Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<Option<(Box<dyn DataSource>, AesMap)>, PlotError> {
        if !mapping.has_aesthetic(&Aesthetic::Group) {
            // No grouping aesthetic, cannot dodge
            return Err(PlotError::MissingAesthetic {
                aesthetic: Aesthetic::Group,
            });
        }
        if !mapping.has_aesthetic(&Aesthetic::X(AestheticDomain::Discrete)) {
            // No discrete x aesthetic, cannot dodge
            return Err(PlotError::MissingAesthetic {
                aesthetic: Aesthetic::X(AestheticDomain::Discrete),
            });
        }

        let mut x_like_data: HashMap<Aesthetic, Vec<f64>> = HashMap::new();
        for aes in mapping.aesthetics() {
            if aes.is_x_like() && aes.is_continuous() {
                let x_like_values = mapping.get_iter_float(aes, data.as_ref())?;
                let x_like_values = x_like_values.collect();
                x_like_data.insert(aes, x_like_values);
            }
        }
        todo!()
    }
}

struct GroupDodger {
    x_like_data: HashMap<Aesthetic, Vec<f64>>,
}

impl GroupDodger {
    fn new(x_like_data: HashMap<Aesthetic, Vec<f64>>) -> Self {
        Self { x_like_data }
    }
}

impl DiscreteDiscreteVisitor2 for GroupDodger {
    fn visit<G: Vectorable + DiscreteType, T: Vectorable + DiscreteType>(
        &mut self,
        group_values: impl Iterator<Item = G>,
        x_values: impl Iterator<Item = T>,
    ) {
        let mut group_values: Vec<G::Sortable> = group_values.map(|v| v.to_sortable()).collect();
        let x_values: Vec<T::Sortable> = x_values.map(|v| v.to_sortable()).collect();

        let mut distinct_groups: HashSet<G::Sortable> = HashSet::new();
        let mut ranges = HashMap::new();
        for (i, group) in group_values.iter().enumerate() {
            distinct_groups.insert(group.clone());
            let x = &x_values[i];
            let key = (group.clone(), x.clone());
            let entry = ranges.entry(key).or_insert((f64::INFINITY, f64::NEG_INFINITY));
            let (min, max) = entry;
            for values in self.x_like_data.values() {
                let x0 = values[i];
                if x0 < *min {
                    *min = x0;
                }
                if x0 > *max {
                    *max = x0;
                }
            }
        }

        // Make a little index numbering the distinct groups
        // so we can map x-like values to dodged positions
        let mut distinct_groups: Vec<G::Sortable> = distinct_groups.into_iter().collect();
        distinct_groups.sort();

        let group_indexs: HashMap<G::Sortable, usize> = distinct_groups
            .iter()
            .enumerate()
            .map(|(i, g)| (g.clone(), i))
            .collect();

        let n_groups = distinct_groups.len() as f64;

        for (i, (group, x)) in group_values.iter().zip(x_values.iter()).enumerate() {
            let key = (group.clone(), x.clone());
            let (min, max) = ranges.get(&key).unwrap();
            let r = max - min;
            let w = r / n_groups;
            let group_idx = group_indexs.get(group).unwrap();
            let group_pos = *group_idx as f64;

            for values in self.x_like_data.values_mut() {
                let x0 = values[i];
                // Map x0 from [min, max] to dodged position
                let dodged_x = min + (x0 - min) / r * w
                    + (group_pos + 0.5) * w;
                values[i] = dodged_x;
            }
        }
    }
}
