pub mod bin;
pub mod boxplot;
pub mod count;
pub mod density;
pub mod smooth;
pub mod summary;

use std::any::Any;
use std::collections::{HashMap, HashSet};

use crate::PlotError;
use crate::aesthetics::values::AesValueBuilder;
use crate::aesthetics::{AesMap, AesValue, Aesthetic, AestheticProperty};
use crate::data::{DataSource, PrimitiveType, VectorIter, VectorValue};
use crate::error::Result;
use crate::utils::GroupByExt;
use crate::utils::data::Vectorable;
use crate::utils::dataframe::DataFrame;

pub struct StatAestheticRequirements {
    /// The main aesthetic property required by the stat
    pub main: AestheticProperty,

    /// An optional secondary aesthetic property required by the stat
    pub secondary: Option<AestheticProperty>,

    pub additional: Vec<AestheticProperty>,
}

impl From<AestheticProperty> for StatAestheticRequirements {
    fn from(property: AestheticProperty) -> Self {
        Self {
            main: property,
            secondary: None,
            additional: vec![],
        }
    }
}

impl From<(AestheticProperty, AestheticProperty)> for StatAestheticRequirements {
    fn from(props: (AestheticProperty, AestheticProperty)) -> Self {
        Self {
            main: props.0,
            secondary: Some(props.1),
            additional: vec![],
        }
    }
}

impl From<Vec<AestheticProperty>> for StatAestheticRequirements {
    fn from(props: Vec<AestheticProperty>) -> Self {
        if props.is_empty() {
            panic!("Cannot create StatAestheticRequirements from empty Vec");
        }
        let main = props[0];
        let secondary = if props.len() > 1 {
            Some(props[1])
        } else {
            None
        };
        let additional = if props.len() > 2 {
            props[2..].to_vec()
        } else {
            vec![]
        };
        Self {
            main,
            secondary,
            additional,
        }
    }
}

/// Trait for statistical transformations
///
/// Stats transform data before rendering. They take the original data and aesthetic mapping,
/// and produce a new data source (potentially with computed columns) and an updated mapping.
///
/// Common stats include:
/// - Identity: No transformation (pass-through)
/// - Count: Count observations in each group
/// - Bin: Bin data into ranges and count
/// - Boxplot: Compute five-number summary statistics
/// - Density: Compute kernel density estimate
pub trait Stat: Send + Sync {
    fn aesthetic_requirements(&self) -> StatAestheticRequirements {
        StatAestheticRequirements {
            main: AestheticProperty::X,
            secondary: None,
            additional: vec![],
        }
    }

    fn compute_params(
        &self,
        _data: &dyn DataSource,
        _mapping: &AesMap,
        _aesthetics: &[Aesthetic],
    ) -> Result<Option<Box<dyn Any>>> {
        Ok(None)
    }

    fn compute_group(
        &self,
        aesthetics: Vec<Aesthetic>,
        iters: Vec<VectorIter<'_>>,
        params: Option<&dyn Any>,
    ) -> Result<(DataFrame, AesMap)>;

    fn compute(&self, data: &dyn DataSource, mapping: &AesMap) -> Result<(DataFrame, AesMap)> {
        let aesthetics = self.determine_aesthetics(mapping);

        let reqs = self.aesthetic_requirements();
        if aesthetics.is_empty() {
            return Err(crate::error::PlotError::MissingAestheticProperty {
                aesthetic_property: reqs.main,
            });
        }

        let params = self.compute_params(data, mapping, &aesthetics)?;

        let grouping_aesthetics: Vec<Aesthetic> = mapping
            .aesthetics()
            .cloned()
            .filter(|aes| aes.is_grouping())
            .collect();

        let mut unique_grouping_aesthetics: Vec<Aesthetic> = Vec::new();
        let mut seen_columns: HashSet<String> = HashSet::new();
        let mut duplicate_grouping_aesthetics: Vec<(Aesthetic, AesValue)> = Vec::new();
        for aes in grouping_aesthetics {
            let value = mapping.get(&aes).unwrap();
            if let AesValue::Column { name, .. } = value {
                if seen_columns.contains(name) {
                    duplicate_grouping_aesthetics.push((aes, value.clone()));
                } else {
                    seen_columns.insert(name.clone());
                    unique_grouping_aesthetics.push(aes);
                }
            } else {
                unique_grouping_aesthetics.push(aes);
            }
        }

        if unique_grouping_aesthetics.is_empty() {
            let mut iters = Vec::new();
            for aes in &aesthetics {
                let iter = mapping
                    .get_vector_iter(&aes, data)
                    .ok_or(PlotError::MissingAesthetic { aesthetic: *aes })?;
                iters.push(iter);
            }

            return self.compute_group(aesthetics, iters, params.as_deref());
        }

        if unique_grouping_aesthetics.len() == 1 {
            // Implement the optimization for single grouping aesthetic
        }

        let group_values: Vec<VectorValue> = unique_grouping_aesthetics
            .iter()
            .map(|aes| {
                mapping
                    .get_vector_iter(aes, data)
                    .ok_or(PlotError::MissingAesthetic { aesthetic: *aes })
                    .and_then(|iter| Ok(iter.to_vector()))
            })
            .collect::<Result<Vec<VectorValue>>>()?;

        fn cmp(i: usize, j: usize, group_values: &Vec<VectorValue>) -> std::cmp::Ordering {
            for gv in group_values {
                let ord = gv.cmp_at_index(i, j);
                if ord != std::cmp::Ordering::Equal {
                    return ord;
                }
            }
            std::cmp::Ordering::Equal
        }

        let mut permutation: Vec<usize> = (0..group_values[0].len()).collect();
        permutation.sort_by(|&i, &j| cmp(i, j, &group_values));

        let aesthetic_values: Vec<VectorValue> = aesthetics
            .iter()
            .map(|aes| {
                mapping
                    .get_vector_iter(aes, data)
                    .ok_or(PlotError::MissingAesthetic { aesthetic: *aes })
                    .and_then(|iter| Ok(iter.to_vector()))
            })
            .collect::<Result<Vec<VectorValue>>>()?;

        let mut group_aesthetic_values: Vec<AesValueBuilder> = unique_grouping_aesthetics
            .iter()
            .map(|aes| {
                let av = mapping
                    .get(aes)
                    .ok_or(PlotError::MissingAesthetic { aesthetic: *aes })?;
                Ok(AesValueBuilder::from(av.clone()))
            })
            .collect::<Result<Vec<AesValueBuilder>>>()?;

        let mut final_data = DataFrame::new();
        let mut final_mapping = AesMap::new();

        for group_indices in permutation
            .into_iter()
            .group_by(|&i, &j| cmp(i, j, &group_values))
        {
            let iters: Vec<VectorIter<'_>> = aesthetic_values
                .iter()
                .map(|av| av.subset_iter(&group_indices))
                .collect();

            let (mut group_data, group_mapping) =
                self.compute_group(aesthetics.clone(), iters, params.as_deref())?;

            let n = group_data.len();
            let group_index_vector = vec![group_indices[0]; n];

            for (gv, avb) in group_values.iter().zip(group_aesthetic_values.iter_mut()) {
                let group_column = gv.subset_iter(&group_index_vector).to_vector();
                avb.append(&mut group_data, group_column);
            }

            // Accumulate group_data into overall data by appending rows
            if final_data.is_empty() {
                final_data = group_data;
            } else {
                final_data.append(group_data);
            }

            // The AesMap should be the same for all groups, so we can just use the last one
            final_mapping = group_mapping;
        }

        // Add grouping aesthetics to final mapping
        for (aes, avb) in unique_grouping_aesthetics
            .iter()
            .zip(group_aesthetic_values.into_iter())
        {
            final_mapping.set(*aes, avb.build());
        }
        for (aes, av) in duplicate_grouping_aesthetics.into_iter() {
            final_mapping.set(aes, av);
        }

        Ok((final_data, final_mapping))
    }

    /// Determine which aesthetics from the mapping are relevant for this stat
    fn determine_aesthetics(&self, mapping: &AesMap) -> Vec<Aesthetic> {
        let reqs = self.aesthetic_requirements();
        let mut aesthetics = Vec::new();
        for aes in reqs.main.aesthetics() {
            if mapping.contains(*aes) {
                aesthetics.push(*aes);
            }
        }
        if let Some(secondary) = reqs.secondary {
            for aes in secondary.aesthetics() {
                if mapping.contains(*aes) {
                    aesthetics.push(*aes);
                }
            }
        }
        for additional in reqs.additional {
            for aes in additional.aesthetics() {
                if mapping.contains(*aes) {
                    aesthetics.push(*aes);
                }
            }
        }
        aesthetics
    }
}

/// Identity transformation - returns None to signal no transformation
pub struct Identity;

impl Identity {
    pub fn new() -> Self {
        Self {}
    }

    fn compute_group_inner<T: PrimitiveType + Vectorable>(
        &self,
        aesthetic: Aesthetic,
        iter: impl Iterator<Item = T>,
    ) -> Result<(DataFrame, AesMap)> {
        let values: Vec<T> = iter.collect();

        let mut df = DataFrame::new();
        df.add_column("x", T::make_vector(values));

        let mut mapping = AesMap::new();
        mapping.x("x", aesthetic.domain());

        Ok((df, mapping))
    }
}
impl Stat for Identity {
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
