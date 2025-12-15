use crate::{
    data::{ContinuousType, DiscreteType, VectorIter},
    scale::transform::{IdentityTransform, Transform},
    utils::set::DiscreteSet,
};

#[derive(Debug, Clone)]
pub struct ContinuousPositionalScale {
    transform: Box<dyn Transform>,
    domain: (f64, f64),
    breaks: Vec<f64>,
    labels: Vec<String>,
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
    fn train<'a>(&mut self, iter: VectorIter<'a>) {
        self.train_continuous(iter);
    }
}

impl super::traits::ContinuousDomainScale for ContinuousPositionalScale {
    fn domain(&self) -> Option<(f64, f64)> {
        Some(self.domain)
    }

    fn set_domain(&mut self, domain: (f64, f64)) {
        self.domain = domain;
    }

    fn limits(&self) -> (Option<f64>, Option<f64>) {
        (self.lower_bound, self.upper_bound)
    }

    fn breaks(&self) -> &[f64] {
        &self.breaks
    }

    fn labels(&self) -> &[String] {
        &self.labels
    }
}

impl super::traits::ContinuousRangeScale for ContinuousPositionalScale {
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

impl super::traits::ContinuousPositionalScale for ContinuousPositionalScale {}

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
    fn train<'a>(&mut self, iter: VectorIter<'a>) {
        self.train_discrete(iter);
    }
}

impl super::traits::DiscreteDomainScale for DiscretePositionalScale {
    fn categories(&self) -> &DiscreteSet {
        &self.elements
    }

    fn add_categories(&mut self, categories: DiscreteSet) {
        self.elements.union(&categories);
    }
}

impl super::traits::ContinuousRangeScale for DiscretePositionalScale {
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
