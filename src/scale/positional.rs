use crate::{
    data::{PrimitiveType, VectorIter},
    scale::traits::{ContinuousDomainScale, DiscreteDomainScale},
    utils::set::DiscreteSet,
};

#[derive(Debug, Clone)]
pub struct ContinuousPositionalScale {
    domain: Option<(f64, f64)>,
    breaks: Vec<f64>,
    labels: Vec<String>,
    lower_bound: Option<f64>,
    upper_bound: Option<f64>,
}

impl ContinuousPositionalScale {
    pub fn new() -> Self {
        Self {
            domain: None,
            breaks: Vec::new(),
            labels: Vec::new(),
            lower_bound: None,
            upper_bound: None,
        }
    }

    /// Compute breaks and labels for this scale
    /// 
    /// Uses the domain to generate nice break positions and format them as labels.
    /// Should be called after training the domain.
    pub fn compute_breaks(&mut self, n: usize) {
        if let Some((min, max)) = self.domain {
            self.breaks = super::utils::extended_breaks((min, max), n);
            self.labels = super::utils::format_breaks(&self.breaks);
        }
    }
}

impl Default for ContinuousPositionalScale {
    fn default() -> Self {
        Self::new()
    }
}

impl super::traits::ScaleBase for ContinuousPositionalScale {
    fn train<'a>(&mut self, iter: VectorIter<'a>) {
        self.train_continuous(iter);
    }
}

impl super::traits::ContinuousDomainScale for ContinuousPositionalScale {
    fn domain(&self) -> Option<(f64, f64)> {
        self.domain
    }

    fn set_domain(&mut self, domain: (f64, f64)) {
        self.domain = Some(domain);
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
    fn map_value<T: PrimitiveType>(&self, value: &T) -> Option<f64> {
        let value = match value.to_primitive() {
            crate::data::PrimitiveValue::Int(x) => Some(x as f64),
            crate::data::PrimitiveValue::Float(x) => Some(x),
            crate::data::PrimitiveValue::Str(_) => None,
            crate::data::PrimitiveValue::Bool(_) => None,
        }?;
        let (d0, d1) = self.domain.unwrap();
        if value < d0.min(d1) || value > d0.max(d1) {
            return None;
        }

        let normalized = (value - d0) / (d1 - d0);
        log::debug!("Mapping {} with domain ({}, {}) -> {}", value, d0, d1, normalized);
        Some(normalized)
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
    fn map_value<T: PrimitiveType>(&self, value: &T) -> Option<f64> {
        let ordinal = match value.to_primitive() {
            crate::data::PrimitiveValue::Int(x) => Some(self.elements.ordinal(&x)?),
            crate::data::PrimitiveValue::Float(_) => None,
            crate::data::PrimitiveValue::Str(x) => Some(self.elements.ordinal(&x.to_string())?),
            crate::data::PrimitiveValue::Bool(x) => Some(self.elements.ordinal(&x)?),
        }?;
        let n = self.len() as f64;
        Some((ordinal as f64 + 0.5) / n)
    }
}
