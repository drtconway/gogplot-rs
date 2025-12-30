use std::sync::Arc;

use crate::aesthetics::{AesMap, AesValue, Aesthetic, AestheticDomain};
use crate::data::{ContinuousType, VectorIter};
use crate::error::PlotError;
use crate::stat::Stat;
use crate::utils::dataframe::{DataFrame, FloatVec};

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
    pub fn compute_group_inner<T: ContinuousType>(
        &self,
        values: impl Iterator<Item = T>,
    ) -> Result<(DataFrame, AesMap), PlotError> {
        let clean_data: Vec<f64> = values
            .map(|v| v.to_f64())
            .filter(|x| x.is_finite())
            .collect();

        if clean_data.is_empty() {
            return Err(PlotError::no_valid_data("no finite values in data"));
        }

        let (x_vals, density_vals, count_vals, scaled_vals) = self.compute_density(&clean_data);

        let mut data = DataFrame::new();
        let mut mapping = AesMap::new();

        data.add_column("x", Arc::new(FloatVec(x_vals)));
        mapping.set(
            Aesthetic::X(AestheticDomain::Continuous),
            AesValue::column("x"),
        );

        data.add_column("density", Arc::new(FloatVec(density_vals)));
        mapping.set(
            Aesthetic::Y(AestheticDomain::Continuous),
            AesValue::column("density"),
        );

        data.add_column("count", Arc::new(FloatVec(count_vals)));
        data.add_column("scaled", Arc::new(FloatVec(scaled_vals)));
        data.add_column(
            "n",
            Arc::new(FloatVec(vec![clean_data.len() as f64; self.n])),
        );

        Ok((data, mapping))
    }

    fn compute_density(&self, data: &[f64]) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
        let n_obs = data.len() as f64;

        // Compute bandwidth using Scott's rule: h = n^(-1/5) * σ
        let mean = data.iter().sum::<f64>() / n_obs;
        let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n_obs - 1.0);
        let std_dev = variance.sqrt();

        let bandwidth = std_dev * n_obs.powf(-0.2) * self.adjust;

        // Determine evaluation range
        let min_val = data.iter().copied().fold(f64::INFINITY, f64::min);
        let max_val = data.iter().copied().fold(f64::NEG_INFINITY, f64::max);
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
            let density: f64 = data
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

        (x_vals, density_vals, count_vals, scaled_vals)
    }
}

impl Stat for Density {
    fn compute_group(
        &self,
        aesthetics: Vec<Aesthetic>,
        iters: Vec<VectorIter<'_>>,
        _params: Option<&dyn std::any::Any>,
    ) -> Result<(DataFrame, AesMap), PlotError> {
        for (aesthetic, iter) in aesthetics.into_iter().zip(iters.into_iter()) {
            return match iter {
                VectorIter::Int(iter) => self.compute_group_inner(iter),
                VectorIter::Float(iter) => self.compute_group_inner(iter),
                _ => Err(PlotError::InvalidAestheticType {
                    aesthetic,
                    expected: crate::error::DataType::Continuous,
                    actual: crate::error::DataType::Discrete,
                }),
            };
        }
        panic!("No aesthetics provided");
    }
}

impl Default for Density {
    fn default() -> Self {
        Self::new()
    }
}
