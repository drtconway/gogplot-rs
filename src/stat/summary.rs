// Statistical summary transformation

use crate::aesthetics::{AesMap, Aesthetic};
use crate::data::{DataSource, GenericVector, VectorType};
use crate::error::{PlotError, Result};
use crate::stat::StatTransform;
use crate::utils::dataframe::{DataFrame, FloatVec, IntVec, StrVec};
use std::collections::HashMap;

/// Summary stat - computes scalar summary statistics for specified aesthetics
///
/// For continuous data: min, max, mean, median, sd
/// For categorical data: min, max, mode, n_unique
///
/// Output column naming:
/// - Single aesthetic: "min", "max", "mean", "median", "sd"
/// - Multiple aesthetics: "{aes}_min", "{aes}_max", etc.
pub struct Summary {
    pub aesthetics: Vec<Aesthetic>,
}

impl Summary {
    pub fn new(aesthetics: Vec<Aesthetic>) -> Self {
        Self { aesthetics }
    }

    /// Compute summary statistics for a single aesthetic
    fn compute_for_aesthetic(
        &self,
        data: &dyn DataSource,
        aesthetic: Aesthetic,
        column_name: &str,
        use_prefix: bool,
    ) -> Result<HashMap<String, Box<dyn GenericVector>>> {
        let column = data
            .get(column_name)
            .ok_or_else(|| PlotError::missing_column(column_name))?;

        let prefix = if use_prefix {
            format!("{:?}_", aesthetic).to_lowercase()
        } else {
            String::new()
        };

        let mut result = HashMap::new();

        match column.vtype() {
            VectorType::Int | VectorType::Float => {
                // Continuous statistics
                let values: Vec<f64> = match column.vtype() {
                    VectorType::Int => {
                        if let Some(iter) = column.iter_int() {
                            iter.map(|v| v as f64).collect()
                        } else {
                            return Err(PlotError::invalid_column_type(column_name, "int"));
                        }
                    }
                    VectorType::Float => {
                        if let Some(iter) = column.iter_float() {
                            iter.collect()
                        } else {
                            return Err(PlotError::invalid_column_type(column_name, "float"));
                        }
                    }
                    _ => unreachable!(),
                };

                // Filter out NaN and infinity
                let valid_values: Vec<f64> = values.into_iter().filter(|v| v.is_finite()).collect();

                if valid_values.is_empty() {
                    // All values were invalid - return NaN for all stats
                    result.insert(
                        format!("{}min", prefix),
                        Box::new(FloatVec(vec![f64::NAN])) as Box<dyn GenericVector>,
                    );
                    result.insert(
                        format!("{}max", prefix),
                        Box::new(FloatVec(vec![f64::NAN])) as Box<dyn GenericVector>,
                    );
                    result.insert(
                        format!("{}mean", prefix),
                        Box::new(FloatVec(vec![f64::NAN])) as Box<dyn GenericVector>,
                    );
                    result.insert(
                        format!("{}median", prefix),
                        Box::new(FloatVec(vec![f64::NAN])) as Box<dyn GenericVector>,
                    );
                    result.insert(
                        format!("{}sd", prefix),
                        Box::new(FloatVec(vec![f64::NAN])) as Box<dyn GenericVector>,
                    );
                } else {
                    let min = valid_values.iter().copied().fold(f64::INFINITY, f64::min);
                    let max = valid_values
                        .iter()
                        .copied()
                        .fold(f64::NEG_INFINITY, f64::max);
                    let mean = valid_values.iter().sum::<f64>() / valid_values.len() as f64;

                    // Median
                    let mut sorted = valid_values.clone();
                    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    let median = if sorted.len() % 2 == 0 {
                        let mid = sorted.len() / 2;
                        (sorted[mid - 1] + sorted[mid]) / 2.0
                    } else {
                        sorted[sorted.len() / 2]
                    };

                    // Standard deviation
                    let variance = valid_values.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
                        / valid_values.len() as f64;
                    let sd = variance.sqrt();

                    result.insert(
                        format!("{}min", prefix),
                        Box::new(FloatVec(vec![min])) as Box<dyn GenericVector>,
                    );
                    result.insert(
                        format!("{}max", prefix),
                        Box::new(FloatVec(vec![max])) as Box<dyn GenericVector>,
                    );
                    result.insert(
                        format!("{}mean", prefix),
                        Box::new(FloatVec(vec![mean])) as Box<dyn GenericVector>,
                    );
                    result.insert(
                        format!("{}median", prefix),
                        Box::new(FloatVec(vec![median])) as Box<dyn GenericVector>,
                    );
                    result.insert(
                        format!("{}sd", prefix),
                        Box::new(FloatVec(vec![sd])) as Box<dyn GenericVector>,
                    );
                }
            }
            VectorType::Str | VectorType::Bool => {
                // Categorical statistics
                let values: Vec<String> = match column.vtype() {
                    VectorType::Str => {
                        if let Some(iter) = column.iter_str() {
                            iter.map(|s| s.to_string()).collect()
                        } else {
                            return Err(PlotError::invalid_column_type(column_name, "string"));
                        }
                    }
                    VectorType::Bool => {
                        if let Some(iter) = column.iter_bool() {
                            iter.map(|b| b.to_string()).collect()
                        } else {
                            return Err(PlotError::invalid_column_type(column_name, "boolean"));
                        }
                    }
                    _ => unreachable!(),
                };

                if values.is_empty() {
                    result.insert(
                        format!("{}min", prefix),
                        Box::new(StrVec(vec![String::new()])) as Box<dyn GenericVector>,
                    );
                    result.insert(
                        format!("{}max", prefix),
                        Box::new(StrVec(vec![String::new()])) as Box<dyn GenericVector>,
                    );
                    result.insert(
                        format!("{}mode", prefix),
                        Box::new(StrVec(vec![String::new()])) as Box<dyn GenericVector>,
                    );
                    result.insert(
                        format!("{}n_unique", prefix),
                        Box::new(IntVec(vec![0])) as Box<dyn GenericVector>,
                    );
                } else {
                    // Min/max - lexicographic ordering
                    let min = values.iter().min().unwrap().clone();
                    let max = values.iter().max().unwrap().clone();

                    // Mode - most common value (first in case of tie)
                    let mut counts: HashMap<&str, usize> = HashMap::new();
                    for value in &values {
                        *counts.entry(value.as_str()).or_insert(0) += 1;
                    }
                    let mode = counts
                        .iter()
                        .max_by_key(|(_, count)| *count)
                        .map(|(value, _)| value.to_string())
                        .unwrap_or_default();

                    // Number of unique values
                    let n_unique = counts.len() as i64;

                    result.insert(
                        format!("{}min", prefix),
                        Box::new(StrVec(vec![min])) as Box<dyn GenericVector>,
                    );
                    result.insert(
                        format!("{}max", prefix),
                        Box::new(StrVec(vec![max])) as Box<dyn GenericVector>,
                    );
                    result.insert(
                        format!("{}mode", prefix),
                        Box::new(StrVec(vec![mode])) as Box<dyn GenericVector>,
                    );
                    result.insert(
                        format!("{}n_unique", prefix),
                        Box::new(IntVec(vec![n_unique])) as Box<dyn GenericVector>,
                    );
                }
            }
        }

        Ok(result)
    }
}

impl StatTransform for Summary {
    fn apply(
        &self,
        data: Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<Option<(Box<dyn DataSource>, AesMap)>> {
        // Determine if we should use prefixes (multiple aesthetics)
        let use_prefix = self.aesthetics.len() > 1;

        let mut all_columns: HashMap<String, Box<dyn GenericVector>> = HashMap::new();

        // Compute summaries for each aesthetic
        for aesthetic in &self.aesthetics {
            // Get the column name mapped to this aesthetic
            let aes_value = mapping
                .get(aesthetic)
                .ok_or_else(|| PlotError::missing_stat_input("Summary", *aesthetic))?;

            let column_name = aes_value.as_column_name().ok_or_else(|| {
                PlotError::no_valid_data(format!(
                    "Aesthetic {:?} must be mapped to a column for Summary stat",
                    aesthetic
                ))
            })?;

            // Compute summaries for this aesthetic
            let aesthetic_columns =
                self.compute_for_aesthetic(data.as_ref(), *aesthetic, column_name, use_prefix)?;

            // Add to result
            all_columns.extend(aesthetic_columns);
        }

        // Create output DataFrame
        let mut df = DataFrame::new();
        for (name, column) in all_columns {
            df.add_column(name, column);
        }

        // The mapping doesn't change - downstream geoms will map to the computed columns
        // (e.g., .geom_hline(Aes::y("mean")) or .geom_hline(Aes::y("y_mean")))
        Ok(Some((Box::new(df), mapping.clone())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        aesthetics::AestheticDomain,
        utils::dataframe::{DataFrame, FloatVec, StrVec},
    };

    #[test]
    fn test_summary_single_continuous() {
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 5.0])));

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Continuous);

        let stat = Summary::new(vec![Aesthetic::X(AestheticDomain::Continuous)]);
        let result = stat.apply(Box::new(df), &mapping).unwrap();

        assert!(result.is_some());
        let (output, _) = result.unwrap();

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
    fn test_summary_multiple_continuous() {
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0])));
        df.add_column("y", Box::new(FloatVec(vec![10.0, 20.0, 30.0])));

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Continuous);
        mapping.y("y", AestheticDomain::Continuous);

        let stat = Summary::new(vec![
            Aesthetic::X(AestheticDomain::Continuous),
            Aesthetic::Y(AestheticDomain::Continuous),
        ]);
        let result = stat.apply(Box::new(df), &mapping).unwrap();

        assert!(result.is_some());
        let (output, _) = result.unwrap();

        // Should produce columns with prefix
        assert!(output.get("x_min").is_some());
        assert!(output.get("x_mean").is_some());
        assert!(output.get("y_min").is_some());
        assert!(output.get("y_mean").is_some());

        let x_mean = output
            .get("x_mean")
            .unwrap()
            .iter_float()
            .unwrap()
            .next()
            .unwrap();
        assert!((x_mean - 2.0).abs() < 1e-10);

        let y_mean = output
            .get("y_mean")
            .unwrap()
            .iter_float()
            .unwrap()
            .next()
            .unwrap();
        assert!((y_mean - 20.0).abs() < 1e-10);
    }

    #[test]
    fn test_summary_categorical() {
        let mut df = DataFrame::new();
        df.add_column(
            "group",
            Box::new(StrVec(vec![
                "a".to_string(),
                "b".to_string(),
                "a".to_string(),
                "c".to_string(),
            ])),
        );

        let mut mapping = AesMap::new();
        mapping.x("group", AestheticDomain::Discrete);

        let stat = Summary::new(vec![Aesthetic::X(AestheticDomain::Discrete)]);
        let result = stat.apply(Box::new(df), &mapping).unwrap();

        assert!(result.is_some());
        let (output, _) = result.unwrap();

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
