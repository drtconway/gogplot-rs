use std::collections::HashMap;

use crate::aesthetics::{AesMap, AesValue, Aesthetic, AestheticDomain};
use crate::data::{ContinuousType, DataSource, DiscreteType, PrimitiveType, VectorIter};
use crate::error::PlotError;
use crate::stat::Stat;
use crate::utils::data::{
    ContinuousVectorVisitor, DiscreteContinuousVisitor2, Vectorable, visit_c, visit2_dc,
};
use crate::utils::dataframe::{BoolVec, DataFrame, FloatVec, IntVec, StrVec};

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
    pub fn compute_ungrouped<T: ContinuousType>(
        &self,
        values: impl Iterator<Item = T>,
        new_data: &mut DataFrame,
        new_mapping: &mut AesMap,
    ) -> Result<(), PlotError> {
        let clean_data: Vec<f64> = values
            .map(|v| v.to_f64())
            .filter(|x| x.is_finite())
            .collect();

        if clean_data.is_empty() {
            return Err(PlotError::no_valid_data("no finite values in data"));
        }

        let (x_vals, density_vals, count_vals, scaled_vals) = self.compute_inner(&clean_data);

        new_data.add_column("x", Box::new(FloatVec(x_vals)));
        new_mapping.set(
            Aesthetic::X(AestheticDomain::Continuous),
            AesValue::column("x"),
        );

        new_data.add_column("density", Box::new(FloatVec(density_vals)));
        new_mapping.set(
            Aesthetic::Y(AestheticDomain::Continuous),
            AesValue::column("density"),
        );

        new_data.add_column("count", Box::new(FloatVec(count_vals)));
        new_data.add_column("scaled", Box::new(FloatVec(scaled_vals)));
        new_data.add_column(
            "n",
            Box::new(FloatVec(vec![clean_data.len() as f64; self.n])),
        );

        Ok(())
    }

    pub fn compute_grouped<'a, T: ContinuousType>(
        &self,
        values: impl Iterator<Item = T>,
        group_values: VectorIter<'a>,
        new_data: &mut DataFrame,
        new_mapping: &mut AesMap,
    ) -> Result<(), PlotError> {
        match group_values {
            VectorIter::Int(iter) => {
                let group_values = self.compute_grouped_inner(values, iter, new_data);
                new_data.add_column("group", Box::new(IntVec(group_values)));
            }
            VectorIter::Float(iter) => {
                let group_values = self.compute_grouped_inner(values, iter, new_data);
                new_data.add_column("group", Box::new(FloatVec(group_values)));
            }
            VectorIter::Str(iter) => {
                let iter = iter.map(|v| v.to_string());
                let group_values = self.compute_grouped_inner(values, iter, new_data);
                new_data.add_column("group", Box::new(StrVec(group_values)));
            }
            VectorIter::Bool(iter) => {
                let group_values = self.compute_grouped_inner(values, iter, new_data);
                new_data.add_column("group", Box::new(BoolVec(group_values)));
            }
        }

        new_mapping.set(
            Aesthetic::X(AestheticDomain::Continuous),
            AesValue::column("x"),
        );
        new_mapping.set(
            Aesthetic::Y(AestheticDomain::Continuous),
            AesValue::column("density"),
        );
        new_mapping.set(Aesthetic::Group, AesValue::column("group"));

        Ok(())
    }
    pub fn compute_grouped_inner<T: ContinuousType, G: PrimitiveType>(
        &self,
        values: impl Iterator<Item = T>,
        group_values: impl Iterator<Item = G>,
        data: &mut DataFrame,
    ) -> Vec<G> {
        let mut groups: HashMap<G::Sortable, Vec<f64>> = std::collections::HashMap::new();
        for (v, g) in values.zip(group_values) {
            let v = v.to_f64();
            if v.is_finite() {
                groups.entry(g.to_sortable()).or_default().push(v);
            }
        }

        let mut pairs: Vec<(G::Sortable, Vec<f64>)> = groups.into_iter().collect();
        pairs.sort_by_key(|(k, _)| k.clone());

        let mut x_values = Vec::new();
        let mut density_values = Vec::new();
        let mut count_values = Vec::new();
        let mut scaled_values = Vec::new();
        let mut group_values = Vec::new();

        for (group_key, group_data) in pairs {
            let group_key = G::from_sortable(group_key);
            let (x_vals, density_vals, count_vals, scaled_vals) = self.compute_inner(&group_data);

            let n_points = x_vals.len();
            x_values.extend(x_vals);
            density_values.extend(density_vals);
            count_values.extend(count_vals);
            scaled_values.extend(scaled_vals);
            group_values.extend(std::iter::repeat(group_key.clone()).take(n_points));
        }
        data.add_column("x", Box::new(FloatVec(x_values)));
        data.add_column("density", Box::new(FloatVec(density_values)));
        data.add_column("count", Box::new(FloatVec(count_values)));
        data.add_column("scaled", Box::new(FloatVec(scaled_values)));
        group_values
    }

    fn compute_inner(&self, data: &[f64]) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
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
    fn apply(
        &self,
        data: &Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<Option<(Box<dyn DataSource>, AesMap)>, PlotError> {
        // Get x values
        let x_values = mapping
            .get_vector_iter(
                &Aesthetic::X(crate::aesthetics::AestheticDomain::Continuous),
                data.as_ref(),
            )
            .ok_or_else(|| {
                PlotError::missing_stat_input("Density", Aesthetic::X(AestheticDomain::Continuous))
            })?;

        if let Some(group_iter) = mapping.get_vector_iter(&Aesthetic::Group, data.as_ref()) {
            let mut visitor = DensityVisitor::from(self);
            visit2_dc(group_iter, x_values, &mut visitor).map(|(new_data, new_mapping)| {
                Some((Box::new(new_data) as Box<dyn DataSource>, new_mapping))
            })
        } else {
            let mut visitor = DensityVisitor::from(self);
            visit_c(x_values, &mut visitor).map(|(new_data, new_mapping)| {
                Some((Box::new(new_data) as Box<dyn DataSource>, new_mapping))
            })
        }
    }
}

impl Default for Density {
    fn default() -> Self {
        Self::new()
    }
}

struct DensityVisitor {
    n: usize,
    adjust: f64,
}

impl DensityVisitor {
    fn compute_inner(&self, data: &[f64]) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
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

impl From<&Density> for DensityVisitor {
    fn from(density: &Density) -> Self {
        Self {
            n: density.n,
            adjust: density.adjust,
        }
    }
}

impl ContinuousVectorVisitor for DensityVisitor {
    type Output = (DataFrame, AesMap);

    fn visit<T: Vectorable + ContinuousType>(
        &mut self,
        values: impl Iterator<Item = T>,
    ) -> std::result::Result<(DataFrame, AesMap), PlotError> {
        let clean_data: Vec<f64> = values
            .map(|x| x.to_f64())
            .filter(|x| x.is_finite())
            .collect();

        if clean_data.is_empty() {
            return Err(PlotError::no_valid_data("no finite values in data"));
        }

        let (x_vals, density_vals, count_vals, scaled_vals) = self.compute_inner(&clean_data);

        let mut data = DataFrame::new();
        let mut mapping = AesMap::new();

        data.add_column("x", Box::new(FloatVec(x_vals)));
        mapping.set(
            Aesthetic::X(AestheticDomain::Continuous),
            AesValue::column("x"),
        );

        data.add_column("density", Box::new(FloatVec(density_vals)));
        mapping.set(
            Aesthetic::Y(AestheticDomain::Continuous),
            AesValue::column("density"),
        );

        data.add_column("count", Box::new(FloatVec(count_vals)));
        data.add_column("scaled", Box::new(FloatVec(scaled_vals)));
        data.add_column(
            "n",
            Box::new(FloatVec(vec![clean_data.len() as f64; self.n])),
        );

        Ok((data, mapping))
    }
}

impl DiscreteContinuousVisitor2 for DensityVisitor {
    type Output = (DataFrame, AesMap);

    fn visit<G: Vectorable + DiscreteType, T: Vectorable + ContinuousType>(
        &mut self,
        group_values: impl Iterator<Item = G>,
        values: impl Iterator<Item = T>,
    ) -> std::result::Result<(DataFrame, AesMap), PlotError> {
        let mut groups: HashMap<G::Sortable, Vec<f64>> = std::collections::HashMap::new();
        for (v, g) in values.zip(group_values) {
            let v = v.to_f64();
            if v.is_finite() {
                groups.entry(g.to_sortable()).or_default().push(v);
            }
        }

        let mut pairs: Vec<(G::Sortable, Vec<f64>)> = groups.into_iter().collect();
        pairs.sort_by_key(|(k, _)| k.clone());

        let mut x_values = Vec::new();
        let mut density_values = Vec::new();
        let mut count_values = Vec::new();
        let mut scaled_values = Vec::new();
        let mut group_values = Vec::new();

        for (group_key, group_data) in pairs {
            let group_key = G::from_sortable(group_key);
            let (x_vals, density_vals, count_vals, scaled_vals) = self.compute_inner(&group_data);

            let n_points = x_vals.len();
            x_values.extend(x_vals);
            density_values.extend(density_vals);
            count_values.extend(count_vals);
            scaled_values.extend(scaled_vals);
            group_values.extend(std::iter::repeat(group_key.clone()).take(n_points));
        }

        let mut data = DataFrame::new();
        let mut mapping = AesMap::new();

        data.add_column("x", Box::new(FloatVec(x_values)));
        data.add_column("density", Box::new(FloatVec(density_values)));
        data.add_column("count", Box::new(FloatVec(count_values)));
        data.add_column("scaled", Box::new(FloatVec(scaled_values)));
        data.add_column("group", G::make_vector(group_values));

        mapping.set(
            Aesthetic::X(AestheticDomain::Continuous),
            AesValue::column("x"),
        );
        mapping.set(
            Aesthetic::Y(AestheticDomain::Continuous),
            AesValue::column("density"),
        );
        mapping.set(Aesthetic::Group, AesValue::column("group"));

        Ok((data, mapping))
    }
}
