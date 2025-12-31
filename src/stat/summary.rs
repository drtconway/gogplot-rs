// Statistical summary transformation

use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain};
use crate::data::{ContinuousType, DiscreteType, VectorIter};
use crate::error::Result;
use crate::stat::Stat;
use crate::utils::data::Vectorable;
use crate::utils::dataframe::DataFrame;

/// Summary stat - computes scalar summary statistics for specified aesthetics
///
/// For continuous data: min, max, mean, median, sd
/// For categorical data: min, max, mode, n_unique
///
/// Output column naming:
/// - Single aesthetic: "min", "max", "mean", "median", "sd"
/// - Multiple aesthetics: "{aes}_min", "{aes}_max", etc.
pub struct Summary {
    pub aesthetic: Aesthetic,
}

impl Summary {
    pub fn new(aesthetic: Aesthetic) -> Self {
        Self { aesthetic }
    }

    fn compute_group_inner_continuous<T: ContinuousType + Vectorable>(
        &self,
        iter: impl Iterator<Item = T>,
    ) -> Result<(DataFrame, AesMap)> {
        let values: Vec<f64> = iter.map(|v| v.to_f64()).filter(|v| v.is_finite()).collect();

        let mut data = DataFrame::new();
        let mapping = AesMap::new();

        if values.len() == 0 {
            data.add_column("min", vec![f64::NAN]);
            data.add_column("max", vec![f64::NAN]);
            data.add_column("mean", vec![f64::NAN]);
            data.add_column("median", vec![f64::NAN]);
            data.add_column("sd", vec![f64::NAN]);
        } else {
            let min = values.iter().copied().fold(f64::INFINITY, f64::min);
            let max = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
            let mean = values.iter().sum::<f64>() / values.len() as f64;

            let mut sorted = values.clone();
            sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let median = if sorted.len() % 2 == 0 {
                let mid = sorted.len() / 2;
                (sorted[mid - 1] + sorted[mid]) / 2.0
            } else {
                sorted[sorted.len() / 2]
            };

            let variance =
                values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;
            let sd = variance.sqrt();

            data.add_column("min", vec![min]);
            data.add_column("max", vec![max]);
            data.add_column("mean", vec![mean]);
            data.add_column("median", vec![median]);
            data.add_column("sd", vec![sd]);
        }

        Ok((data, mapping))
    }

    fn compute_group_inner_discrete<T: DiscreteType + Vectorable>(
        &self,
        iter: impl Iterator<Item = T>,
    ) -> Result<(DataFrame, AesMap)> {
        let mut values: Vec<T> = iter.collect();
        values.sort();
        let mut groups = Vec::new();
        groups.push((values[0].clone(), 1usize));
        for v in values.into_iter().skip(1) {
            if v == groups.last().unwrap().0 {
                let last = groups.last_mut().unwrap();
                last.1 += 1;
            } else {
                groups.push((v.clone(), 1usize));
            }
        }

        let mut data = DataFrame::new();
        let mapping = AesMap::new();

        if groups.len() == 0 {
            data.add_column("min", T::make_vector(Vec::<T>::new()));
            data.add_column("max", T::make_vector(Vec::<T>::new()));
            data.add_column("mode", T::make_vector(Vec::<T>::new()));
            data.add_column("n_unique", Vec::<i64>::new());
        } else {
            let min = groups.first().unwrap().0.clone();
            let max = groups.last().unwrap().0.clone();
            let mode = groups
                .iter()
                .max_by_key(|(_, count)| *count)
                .map(|(value, _)| value.clone())
                .unwrap();
            let n_unique = groups.len() as i64;

            data.add_column("min", T::make_vector(vec![min]));
            data.add_column("max", T::make_vector(vec![max]));
            data.add_column("mode", T::make_vector(vec![mode]));
            data.add_column("n_unique", vec![n_unique]);
        }

        Ok((data, mapping))
    }
}

impl Stat for Summary {
    fn aesthetic_requirements(&self) -> super::StatAestheticRequirements {
        super::StatAestheticRequirements {
            main: self.aesthetic.to_property().unwrap(),
            secondary: None,
        }
    }

    fn compute_group(
        &self,
        aesthetics: Vec<Aesthetic>,
        iters: Vec<crate::data::VectorIter<'_>>,
        _params: Option<&dyn std::any::Any>,
    ) -> Result<(DataFrame, AesMap)> {
        for (aesthetic, iter) in aesthetics.iter().zip(iters.into_iter()) {
            return match iter {
                VectorIter::Int(iter) => {
                    if aesthetic.domain() == AestheticDomain::Continuous {
                        self.compute_group_inner_continuous(iter)
                    } else {
                        self.compute_group_inner_discrete(iter)
                    }
                }
                VectorIter::Float(iter) => self.compute_group_inner_continuous(iter),
                VectorIter::Str(iter) => {
                    self.compute_group_inner_discrete(iter.map(|s| s.to_string()))
                }
                VectorIter::Bool(iter) => self.compute_group_inner_discrete(iter),
            };
        }
        panic!("No aesthetics provided for Summary stat");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{aesthetics::AestheticDomain, data::DataSource, utils::dataframe::DataFrame};

    #[test]
    fn test_summary_single_continuous() {
        let mut df = DataFrame::new();
        df.add_column("x", vec![1.0, 2.0, 3.0, 4.0, 5.0]);
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Continuous);

        let stat = Summary::new(Aesthetic::X(AestheticDomain::Continuous));
        let (output, _) = stat.compute(df.as_ref(), &mapping).unwrap();

        // Should produce columns without prefix
        assert!(output.get("min").is_some());
        assert!(output.get("max").is_some());
        assert!(output.get("mean").is_some());
        assert!(output.get("median").is_some());
        assert!(output.get("sd").is_some());

        // Check values
        let mean = output
            .get("mean")
            .unwrap()
            .iter_float()
            .unwrap()
            .next()
            .unwrap();
        assert!((mean - 3.0).abs() < 1e-10);

        let median = output
            .get("median")
            .unwrap()
            .iter_float()
            .unwrap()
            .next()
            .unwrap();
        assert!((median - 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_summary_categorical() {
        let mut df = DataFrame::new();
        df.add_column("group", vec!["a", "b", "a", "c"]);
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.x("group", AestheticDomain::Discrete);

        let stat = Summary::new(Aesthetic::X(AestheticDomain::Discrete));
        let (output, _) = stat.compute(df.as_ref(), &mapping).unwrap();

        // Should produce categorical stats without prefix
        assert!(output.get("min").is_some());
        assert!(output.get("max").is_some());
        assert!(output.get("mode").is_some());
        assert!(output.get("n_unique").is_some());

        let mode = output
            .get("mode")
            .unwrap()
            .iter_str()
            .unwrap()
            .next()
            .unwrap();
        assert_eq!(mode, "a"); // "a" appears twice

        let n_unique = output
            .get("n_unique")
            .unwrap()
            .iter_int()
            .unwrap()
            .next()
            .unwrap();
        assert_eq!(n_unique, 3);
    }
}
