pub mod bin;
pub mod boxplot;
pub mod count;
pub mod density;
pub mod smooth;
pub mod summary;

use std::any::Any;

use crate::PlotError;
use crate::aesthetics::{AesMap, Aesthetic, AestheticProperty};
use crate::data::{DataSource, PrimitiveType, VectorIter};
use crate::error::Result;
use crate::utils::data::Vectorable;
use crate::utils::dataframe::DataFrame;

pub struct StatAestheticRequirements {
    /// The main aesthetic property required by the stat
    pub main: AestheticProperty,

    /// An optional secondary aesthetic property required by the stat
    pub secondary: Option<AestheticProperty>,
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
        }
    }

    fn compute_params(
        &self,
        _data: &dyn DataSource,
        _mapping: &AesMap,
        _aesthetics: &[Aesthetic]
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

        // Ignore grouping for the moment - just compute on the full vector

        let mut iters = Vec::new();
        for aes in &aesthetics {
            let iter = mapping.get_vector_iter(&aes, data).ok_or(
                PlotError::MissingAesthetic {
                    aesthetic: *aes,
                },
            )?;
            iters.push(iter);
        }

        self.compute_group(aesthetics, iters, params.as_deref())
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
