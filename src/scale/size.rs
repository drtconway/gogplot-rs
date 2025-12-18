use crate::{data::{PrimitiveType, VectorIter}, scale::traits::{ContinuousDomainScale, DiscreteDomainScale}, utils::set::DiscreteSet};



#[derive(Debug, Clone)]
pub struct ContinuousSizeScale {
    domain: (f64, f64),
    lower_bound: Option<f64>,
    upper_bound: Option<f64>,
}

impl ContinuousSizeScale {
    pub fn new() -> Self {
        Self {
            domain: (0.0, 1.0),
            lower_bound: None,
            upper_bound: None,
         }
    }

    pub fn with_limits(mut self, limits: (f64, f64)) -> Self {
        self.domain = limits;
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

impl Default for ContinuousSizeScale {
    fn default() -> Self {
        Self::new()
    }
}

impl super::traits::ScaleBase for ContinuousSizeScale {
    fn train<'a>(&mut self, iter: VectorIter<'a>) {
        self.train_continuous(iter);
    }
}

impl super::traits::ContinuousDomainScale for ContinuousSizeScale {
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
        &[]
    }

    fn labels(&self) -> &[String] {
        &[]
    }
}

impl super::traits::ContinuousRangeScale for ContinuousSizeScale {
    fn map_value<T: crate::data::PrimitiveType>(&self, value: &T) -> Option<f64> {
        let v = match value.to_primitive() {
            crate::data::PrimitiveValue::Int(x) => Some(x as f64),
            crate::data::PrimitiveValue::Float(x) => Some(x),
            crate::data::PrimitiveValue::Str(_) => None,
            crate::data::PrimitiveValue::Bool(_) => None,
        }?;
        let (min_domain, max_domain) = self.domain;
        if v < min_domain || v > max_domain {
            return None;
        }
        let t = (v - min_domain) / (max_domain - min_domain);
        Some(t)
    }
}

#[derive(Clone, Debug)]
pub struct DiscreteSizeScale {
    elements: DiscreteSet,
}

impl DiscreteSizeScale {
    /// Create a new discrete size scale.
    pub fn new() -> Self {
        Self {
            elements: DiscreteSet::new(),
        }
    }
}

impl Default for DiscreteSizeScale {
    fn default() -> Self {
        Self::new()
    }
}

impl super::traits::ScaleBase for DiscreteSizeScale {
    fn train<'a>(&mut self, iter: VectorIter<'a>) {
        self.train_discrete(iter);
    }
}

impl super::traits::DiscreteDomainScale for DiscreteSizeScale {
    fn categories(&self) -> &DiscreteSet {
        &self.elements
    }

    fn add_categories(&mut self, categories: DiscreteSet) {
        self.elements.union(&categories);
    }
}

impl super::traits::ContinuousRangeScale for DiscreteSizeScale {
    fn map_value<T: PrimitiveType>(&self, value: &T) -> Option<f64> {
        let ordinal = match value.to_primitive() {
            crate::data::PrimitiveValue::Int(x) => self.elements.ordinal(&x),
            crate::data::PrimitiveValue::Float(_) => None,
            crate::data::PrimitiveValue::Str(x) => self.elements.ordinal(&x),
            crate::data::PrimitiveValue::Bool(x) => self.elements.ordinal(&x),
        }?;
        let size = (ordinal as f64 + 1.0) / (self.elements.len().max(1) as f64);
        Some(size)
    }
}