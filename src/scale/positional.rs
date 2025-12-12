use crate::{
    data::{ContinuousType, DiscreteType, DiscreteValue, compute_min_max},
    scale::transform::{IdentityTransform, Transform},
    utils::set::DiscreteSet,
};

#[derive(Debug, Clone)]
pub struct ContinuousPositionalScale {
    transform: Box<dyn Transform>,
    domain: (f64, f64),
    breaks: Vec<f64>,
    labels: Vec<String>,
    trained: bool,
    lower_bound: Option<f64>,
    upper_bound: Option<f64>,
}

impl ContinuousPositionalScale {
    pub fn new(transform: Box<dyn Transform>) -> Self {
        Self {
            transform,
            domain: (0.0, 1.0),
            breaks: Vec::new(),
            labels: Vec::new(),
            trained: false,
            lower_bound: None,
            upper_bound: None,
        }
    }

    pub fn with_limits(mut self, limits: (f64, f64)) -> Self {
        self.domain = limits;
        self.trained = true;
        self
    }

    pub fn with_breaks(mut self, breaks: Vec<f64>) -> Self {
        self.breaks = breaks;
        self
    }

    pub fn with_labels(mut self, labels: Vec<String>) -> Self {
        self.labels = labels;
        self
    }

    pub fn with_lower_bound(mut self, bound: f64) -> Self {
        self.lower_bound = Some(bound);
        self
    }

    pub fn with_upper_bound(mut self, bound: f64) -> Self {
        self.upper_bound = Some(bound);
        self
    }
}

impl Default for ContinuousPositionalScale {
    fn default() -> Self {
        Self::new(Box::new(IdentityTransform))
    }
}

impl super::traits::ScaleBase for ContinuousPositionalScale {
    fn scale_type(&self) -> super::ScaleType {
        super::ScaleType::Continuous
    }

    fn train(&mut self, data: &[&dyn crate::data::GenericVector]) {
        if let Some((mut min, mut max)) = compute_min_max(data) {
            if self.trained {
                let (curr_min, curr_max) = self.domain;
                min = min.min(curr_min);
                max = max.max(curr_max);
            }

            let (domain_min, domain_max) = self.transform.domain();
            if domain_min.is_finite() {
                min = min.max(domain_min);
            }
            if domain_max.is_finite() {
                max = max.min(domain_max);
            }

            if let Some(lower) = self.lower_bound {
                min = min.min(lower);
            }
            if let Some(upper) = self.upper_bound {
                max = max.max(upper);
            }

            let range = max - min;
            let expansion = if range.abs() < 1e-10 {
                if min.abs() < 1e-10 {
                    1.0
                } else {
                    min.abs() * 0.1
                }
            } else {
                range * 0.05
            };

            let lower_expansion = if self.lower_bound.is_some() && min == self.lower_bound.unwrap()
            {
                0.0
            } else {
                expansion
            };
            let upper_expansion = if self.upper_bound.is_some() && max == self.upper_bound.unwrap()
            {
                0.0
            } else {
                expansion
            };

            let expanded_min = min - lower_expansion;
            let expanded_max = max + upper_expansion;

            let final_min = if domain_min.is_finite() && expanded_min < domain_min {
                domain_min
            } else {
                expanded_min
            };
            let final_max = if domain_max.is_finite() && expanded_max > domain_max {
                domain_max
            } else {
                expanded_max
            };

            self.domain = (final_min, final_max);
            self.breaks = self.transform.breaks(self.domain, 5);
            self.labels = self
                .breaks
                .iter()
                .map(|b| self.transform.format(*b))
                .collect();
            self.trained = true;
        }
    }
}

impl super::traits::PositionalScale for ContinuousPositionalScale {
    fn breaks(&self) -> &[f64] {
        &self.breaks
    }

    fn labels(&self) -> &[String] {
        &self.labels
    }
}

impl super::traits::ContinuousPositionalScale for ContinuousPositionalScale {
    fn map_value<T: ContinuousType>(&self, value: &T) -> Option<f64> {
        let value = value.to_f64();
        let (d0, d1) = self.domain;
        if value < d0.min(d1) || value > d0.max(d1) {
            return None;
        }

        let transformed_data = self.transform.transform(value);
        let transformed_d0 = self.transform.transform(d0);
        let transformed_d1 = self.transform.transform(d1);

        if !transformed_data.is_finite() {
            return None;
        }

        Some((transformed_data - transformed_d0) / (transformed_d1 - transformed_d0))
    }
}

#[derive(Debug, Clone)]
pub struct DiscretePositionalScale {
    elements: DiscreteSet,
    breaks: Vec<f64>,
    labels: Vec<String>,
}

impl DiscretePositionalScale {
    pub fn new() -> Self {
        Self {
            elements: DiscreteSet::new(),
            breaks: Vec::new(),
            labels: Vec::new(),
        }
    }
}

impl Default for DiscretePositionalScale {
    fn default() -> Self {
        Self::new()
    }
}

impl super::traits::ScaleBase for DiscretePositionalScale {
    fn train(&mut self, data: &[&dyn crate::data::GenericVector]) {
        for vec in data {
            if let Some(ints) = vec.iter_int() {
                for v in ints {
                    self.elements.add(&v);
                }
            } else if let Some(strs) = vec.iter_str() {
                for v in strs {
                    self.elements.add(&v);
                }
            } else if let Some(bools) = vec.iter_bool() {
                for v in bools {
                    self.elements.add(&v);
                }
            }
        }
        self.elements.build();

        let n = self.elements.len() as f64;
        self.breaks = (0..self.elements.len())
            .map(|i| (i as f64 + 0.5) / n)
            .collect();
        self.labels = self
            .elements
            .iter()
            .map(|v| match v {
                DiscreteValue::Int(i) => i.to_string(),
                DiscreteValue::Str(s) => s.clone(),
                DiscreteValue::Bool(b) => b.to_string(),
            })
            .collect();
    }

    fn scale_type(&self) -> super::ScaleType {
        super::ScaleType::Categorical
    }
}

impl super::traits::PositionalScale for DiscretePositionalScale {
    fn breaks(&self) -> &[f64] {
        &self.breaks
    }

    fn labels(&self) -> &[String] {
        &self.labels
    }
}

impl super::traits::DiscretePositionalScale for DiscretePositionalScale {
    fn len(&self) -> usize {
        self.elements.len()
    }

    fn ordinal<T: DiscreteType>(&self, value: &T) -> Option<usize> {
        self.elements.ordinal(value)
    }

    fn map_value<T: DiscreteType>(&self, value: &T) -> Option<f64> {
        let n = self.len() as f64;
        self.ordinal(value).map(|idx| (idx as f64 + 0.5) / n)
    }
}

/// Generate axis breaks using Wilkinson's Extended algorithm.
/// Returns a vector of break positions given a data range and desired number of breaks.
pub fn extended_breaks(domain: (f64, f64), n: usize) -> Vec<f64> {
    // Nice numbers to use for step sizes
    const Q: [f64; 5] = [1.0, 5.0, 2.0, 2.5, 4.0];

    let (min, max) = domain;
    if n < 2 {
        return vec![min, max];
    }

    // Handle degenerate case: single value
    if (min - max).abs() < 1e-10 {
        // Create a symmetric range around the value
        if min.abs() < 1e-10 {
            // Value is ~0, create range around 0
            return vec![-1.0, 0.0, 1.0];
        } else {
            // Create range ±10% around the value
            let range = min.abs() * 0.1;
            return vec![min - range, min, min + range];
        }
    }
    let range = max - min;
    let mut best_score = std::f64::NEG_INFINITY;
    let mut best = vec![min, max];

    for &q in &Q {
        let w = (range / (n as f64 - 1.0)) / q;
        let step = q * 10f64.powf(w.log10().floor());
        let start = (min / step).floor() * step;
        let end = (max / step).ceil() * step;
        let mut breaks = Vec::new();
        let mut x = start;
        while x <= end + 1e-10 {
            breaks.push(x);
            x += step;
        }
        // Score: coverage, simplicity, density, legibility (simplified)
        let coverage = (min - breaks[0]).abs() + (breaks.last().unwrap() - max).abs();
        let simplicity = if q == 1.0 { 1.0 } else { 0.5 };
        let density = (breaks.len() as f64 - n as f64).abs();
        let score = -coverage - density + simplicity;
        if score > best_score {
            best_score = score;
            best = breaks;
        }
    }
    best
}
