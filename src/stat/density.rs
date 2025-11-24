use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::data::DataSource;
use crate::error::{DataType, PlotError};
use crate::stat::StatTransform;
use crate::utils::dataframe::{DataFrame, FloatVec, StrVec};
use crate::utils::grouping::{get_grouping_columns, create_composite_keys, group_by_key, split_composite_key};

/// Kernel density estimation stat
///
/// Computes a smoothed density estimate using a Gaussian kernel.
/// Uses Scott's rule for bandwidth selection by default.
pub struct Density {
    /// Number of equally-spaced points to evaluate density at (default 512)
    n: usize,
    /// Bandwidth adjustment multiplier (default 1.0)
    adjust: f64,
}

impl Density {
    pub fn new() -> Self {
        Self {
            n: 512,
            adjust: 1.0,
        }
    }

    pub fn n(mut self, n: usize) -> Self {
        self.n = n;
        self
    }

    pub fn adjust(mut self, adjust: f64) -> Self {
        self.adjust = adjust;
        self
    }

    /// Compute kernel density estimate
    ///
    /// Returns a DataFrame with columns: "x", "density", "count", "scaled", "n"
    pub fn compute(&self, data: &[f64]) -> Result<DataFrame, PlotError> {
        if data.is_empty() {
            return Err(PlotError::no_valid_data(
                "cannot compute density of empty data"
            ));
        }

        // Remove NaN values
        let clean_data: Vec<f64> = data.iter().filter(|x| x.is_finite()).copied().collect();

        if clean_data.is_empty() {
            return Err(PlotError::no_valid_data(
                "no finite values in data"
            ));
        }

        let n_obs = clean_data.len() as f64;

        // Compute bandwidth using Scott's rule: h = n^(-1/5) * σ
        let mean = clean_data.iter().sum::<f64>() / n_obs;
        let variance = clean_data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n_obs - 1.0);
        let std_dev = variance.sqrt();

        let bandwidth = std_dev * n_obs.powf(-0.2) * self.adjust;

        // Determine evaluation range
        let min_val = clean_data.iter().copied().fold(f64::INFINITY, f64::min);
        let max_val = clean_data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let padding = 3.0 * bandwidth; // Extend range by 3 bandwidths
        let x_min = min_val - padding;
        let x_max = max_val + padding;

        // Create evaluation points
        let mut x_vals = Vec::with_capacity(self.n);
        let mut density_vals = Vec::with_capacity(self.n);

        for i in 0..self.n {
            let x = x_min + (x_max - x_min) * i as f64 / (self.n - 1) as f64;
            x_vals.push(x);

            // Compute density at this point using Gaussian kernel
            // K(u) = (1/sqrt(2π)) * exp(-u²/2)
            // Density at x = (1/(n*h)) * Σ K((x - x_i) / h)
            let density: f64 = clean_data
                .iter()
                .map(|&xi| {
                    let u = (x - xi) / bandwidth;
                    
                    (-0.5 * u * u).exp() / (2.0 * std::f64::consts::PI).sqrt()
                })
                .sum::<f64>()
                / (n_obs * bandwidth);

            density_vals.push(density);
        }

        // Compute derived variables
        let max_density = density_vals
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);
        let count_vals: Vec<f64> = density_vals.iter().map(|d| d * n_obs).collect();
        let scaled_vals: Vec<f64> = density_vals.iter().map(|d| d / max_density).collect();

        let mut result = DataFrame::new();
        result.add_column("x", Box::new(FloatVec(x_vals)));
        result.add_column("density", Box::new(FloatVec(density_vals)));
        result.add_column("count", Box::new(FloatVec(count_vals)));
        result.add_column("scaled", Box::new(FloatVec(scaled_vals)));
        result.add_column("n", Box::new(FloatVec(vec![n_obs; self.n])));

        Ok(result)
    }
}

impl StatTransform for Density {
    fn apply(
        &self,
        data: Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<Option<(Box<dyn DataSource>, AesMap)>, PlotError> {
        // Get the x aesthetic - this is required for density
        let x_mapping = mapping.get(&Aesthetic::X).ok_or_else(|| {
            PlotError::missing_stat_input("Density", Aesthetic::X)
        })?;

        // Only support column mappings for now
        let x_col_name = match x_mapping {
            AesValue::Column { name, .. } => name,
            _ => {
                return Err(PlotError::InvalidAestheticType {
                    aesthetic: Aesthetic::X,
                    expected: DataType::ColumnMapping,
                    actual: DataType::Custom("constant".to_string()),
                });
            }
        };

        // Get the x column from data
        let x_col = data.get(x_col_name.as_str()).ok_or_else(|| {
            PlotError::missing_column(x_col_name.as_str())
        })?;

        // Extract float values
        let x_values: Vec<f64> = x_col
            .iter_float()
            .ok_or_else(|| PlotError::invalid_column_type(x_col_name.as_str(), "float"))?
            .collect();

        // Get grouping columns
        let group_cols = get_grouping_columns(mapping);

        if group_cols.is_empty() {
            // No grouping - compute density for all data
            let computed = self.compute(&x_values)?;

            // Update the mapping to use the computed density column for y
            let mut new_mapping = mapping.clone();
            new_mapping.set(Aesthetic::Y, AesValue::column("density"));
            new_mapping.set(Aesthetic::X, AesValue::column("x"));

            Ok(Some((Box::new(computed), new_mapping)))
        } else {
            // Compute density for each group
            let composite_keys = create_composite_keys(data.as_ref(), &group_cols);
            let groups = group_by_key(&x_values, &composite_keys);

            // Compute density for each group and combine results
            let mut all_x = Vec::new();
            let mut all_density = Vec::new();
            let mut all_count = Vec::new();
            let mut all_scaled = Vec::new();
            let mut all_n = Vec::new();
            let mut all_group_keys = Vec::new();

            for (group_key, group_values) in groups {
                let group_result = self.compute(&group_values)?;
                
                let x = group_result.get("x").unwrap().iter_float().unwrap().collect::<Vec<_>>();
                let density = group_result.get("density").unwrap().iter_float().unwrap().collect::<Vec<_>>();
                let count = group_result.get("count").unwrap().iter_float().unwrap().collect::<Vec<_>>();
                let scaled = group_result.get("scaled").unwrap().iter_float().unwrap().collect::<Vec<_>>();
                let n = group_result.get("n").unwrap().iter_float().unwrap().collect::<Vec<_>>();

                let n_points = x.len();
                all_x.extend(x);
                all_density.extend(density);
                all_count.extend(count);
                all_scaled.extend(scaled);
                all_n.extend(n);
                all_group_keys.extend(vec![group_key; n_points]);
            }

            // Create combined dataframe
            let mut computed = DataFrame::new();
            computed.add_column("x", Box::new(FloatVec(all_x)));
            computed.add_column("density", Box::new(FloatVec(all_density)));
            computed.add_column("count", Box::new(FloatVec(all_count)));
            computed.add_column("scaled", Box::new(FloatVec(all_scaled)));
            computed.add_column("n", Box::new(FloatVec(all_n)));

            // Add grouping columns by splitting composite keys
            for (aesthetic, col_name) in &group_cols {
                let values: Vec<String> = all_group_keys
                    .iter()
                    .map(|key| {
                        let key_parts = split_composite_key(key, &group_cols);
                        key_parts
                            .iter()
                            .find(|(aes, _)| aes == aesthetic)
                            .map(|(_, val)| val.clone())
                            .unwrap_or_default()
                    })
                    .collect();
                computed.add_column(col_name, Box::new(StrVec(values)));
            }

            // Update the mapping
            let mut new_mapping = mapping.clone();
            new_mapping.set(Aesthetic::Y, AesValue::column("density"));
            new_mapping.set(Aesthetic::X, AesValue::column("x"));

            Ok(Some((Box::new(computed), new_mapping)))
        }
    }
}

impl Default for Density {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataSource;

    #[test]
    fn test_density_basic() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let density = Density::new();
        let result = density.compute(&data).unwrap();

        assert!(result.get("x").is_some());
        assert!(result.get("density").is_some());
        assert!(result.get("count").is_some());
        assert!(result.get("scaled").is_some());
        assert!(result.get("n").is_some());

        let x = result.get("x").unwrap().iter_float().unwrap();
        assert_eq!(x.count(), 512);

        let scaled = result.get("scaled").unwrap().iter_float().unwrap();
        let max_scaled = scaled.fold(f64::NEG_INFINITY, f64::max);
        assert!((max_scaled - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_density_custom_n() {
        let data = vec![1.0, 2.0, 3.0];
        let density = Density::new().n(100);
        let result = density.compute(&data).unwrap();

        let x = result.get("x").unwrap().iter_float().unwrap();
        assert_eq!(x.count(), 100);
    }

    #[test]
    fn test_density_adjust() {
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        let density1 = Density::new().adjust(0.5);
        let result1 = density1.compute(&data).unwrap();

        let density2 = Density::new().adjust(2.0);
        let result2 = density2.compute(&data).unwrap();

        // Smaller adjust should give more peaked density
        // Just check that we get different results
        let d1 = result1.get("density").unwrap().iter_float().unwrap();
        let d2 = result2.get("density").unwrap().iter_float().unwrap();
        let max1 = d1.fold(f64::NEG_INFINITY, f64::max);
        let max2 = d2.fold(f64::NEG_INFINITY, f64::max);
        assert!(max1 > max2);
    }
}
