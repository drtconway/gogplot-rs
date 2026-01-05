use crate::{
    data::{PrimitiveType, VectorIter},
    scale::traits::{ContinuousDomainScale, DiscreteDomainScale},
    scale::transform::{Transform, IdentityTransform},
    utils::set::DiscreteSet,
};

#[derive(Clone)]
pub struct ContinuousPositionalScale {
    domain: Option<(f64, f64)>,  // Domain in transformed space
    breaks: Vec<f64>,            // Breaks in data space
    labels: Vec<String>,
    lower_bound: Option<f64>,
    upper_bound: Option<f64>,
    pub transform: Box<dyn Transform>,
}

impl ContinuousPositionalScale {
    pub fn new() -> Self {
        Self {
            domain: None,
            breaks: Vec::new(),
            labels: Vec::new(),
            lower_bound: None,
            upper_bound: None,
            transform: Box::new(IdentityTransform),
        }
    }

    /// Create a new scale with a specific transformation
    pub fn with_transform(transform: Box<dyn Transform>) -> Self {
        Self {
            domain: None,
            breaks: Vec::new(),
            labels: Vec::new(),
            lower_bound: None,
            upper_bound: None,
            transform,
        }
    }

    /// Set the transformation for this scale
    pub fn set_transform(&mut self, transform: Box<dyn Transform>) {
        self.transform = transform;
        // Clear domain and breaks since they're no longer valid
        self.domain = None;
        self.breaks.clear();
        self.labels.clear();
    }

    /// Compute breaks and labels for this scale
    ///
    /// Uses the transform's break generation and formatting.
    /// Should be called after training the domain.
    pub fn compute_breaks(&mut self, n: usize) {
        if let Some((min_transformed, max_transformed)) = self.domain {
            // Inverse transform to get data space limits
            let min_data = self.transform.inverse(min_transformed);
            let max_data = self.transform.inverse(max_transformed);
            
            // Use transform's break generation (returns breaks in data space)
            self.breaks = self.transform.breaks((min_data, max_data), n);
            
            // Use transform's formatting for labels
            self.labels = self.breaks.iter()
                .map(|&b| self.transform.format(b))
                .collect();
        }
    }
}

impl Default for ContinuousPositionalScale {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ContinuousPositionalScale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ContinuousPositionalScale")
            .field("domain", &self.domain)
            .field("breaks", &self.breaks)
            .field("labels", &self.labels)
            .field("lower_bound", &self.lower_bound)
            .field("upper_bound", &self.upper_bound)
            .field("transform", &self.transform.name())
            .finish()
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

    fn train_continuous<'a>(&mut self, iter: VectorIter<'a>) {
        use crate::scale::ContinuousScaleTrainer;
        use crate::utils::data::visit_c;
        
        let mut trainer = ContinuousScaleTrainer::new();
        visit_c(iter, &mut trainer).unwrap();
        
        if let Some((obs_min_value, obs_max_value)) = trainer.bounds {
            // Apply transformation to the observed bounds
            let transformed_min = self.transform.transform(obs_min_value);
            let transformed_max = self.transform.transform(obs_max_value);
            
            // Check if transformation is valid
            if !transformed_min.is_finite() || !transformed_max.is_finite() {
                log::warn!(
                    "Transform {} produced non-finite values for domain ({}, {})",
                    self.transform.name(),
                    obs_min_value,
                    obs_max_value
                );
                return;
            }
            
            // Note: limits are stored in data space, need to transform them
            let (min_limit, max_limit) = self.limits();
            let min_value = if let Some(limit) = min_limit {
                self.transform.transform(limit)
            } else {
                transformed_min
            };
            let max_value = if let Some(limit) = max_limit {
                self.transform.transform(limit)
            } else {
                transformed_max
            };

            // Apply 5% expansion on each side (ggplot2 default) in transformed space
            let range = max_value - min_value;
            let expansion = range * 0.05;
            let min_value = min_value - expansion;
            let max_value = max_value + expansion;

            // Merge with existing domain if present
            if let Some((min_existing, max_existing)) = self.domain() {
                let min_value = min_value.min(min_existing);
                let max_value = max_value.max(max_existing);
                self.set_domain((min_value, max_value));
            } else {
                self.set_domain((min_value, max_value));
            }
            
            log::info!(
                "Trained continuous scale domain to ({}, {}) in {} space with 5% expansion",
                min_value,
                max_value,
                self.transform.name()
            );
        }
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
        
        // Apply the transformation to the data value
        let transformed_value = self.transform.transform(value);
        
        // If the transformed value is not finite, skip it (but preserve row alignment
        // by returning None rather than panicking or removing the row entirely)
        // This handles NaN placeholders used by geoms like boxplot
        if !transformed_value.is_finite() {
            return None;
        }
        
        let (d0, d1) = self.domain.unwrap();
        if transformed_value < d0.min(d1) || transformed_value > d0.max(d1) {
            return None;
        }

        let normalized = (transformed_value - d0) / (d1 - d0);
        log::debug!(
            "Mapping {} (transformed: {}) with domain ({}, {}) -> {}",
            value,
            transformed_value,
            d0,
            d1,
            normalized
        );
        Some(normalized)
    }
}

impl super::traits::ContinuousPositionalScale for ContinuousPositionalScale {}

#[derive(Debug, Clone)]
pub struct DiscretePositionalScale {
    elements: DiscreteSet,
}

impl DiscretePositionalScale {
    pub fn new() -> Self {
        Self {
            elements: DiscreteSet::new(),
        }
    }

    /// Get the break positions for discrete categories
    /// Returns the normalized position (0-1) for each category
    pub fn breaks(&self) -> Vec<f64> {
        let n = self.elements.len() as f64;
        if n == 0.0 {
            return Vec::new();
        }
        (0..self.elements.len())
            .map(|i| (i as f64 + 0.5) / n)
            .collect()
    }

    /// Get the labels for discrete categories
    /// Returns the string representation of each category
    pub fn labels(&self) -> Vec<String> {
        self.elements.iter().map(|v| v.to_string()).collect()
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
