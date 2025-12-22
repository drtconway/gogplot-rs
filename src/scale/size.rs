use crate::{data::{PrimitiveType, VectorIter}, scale::traits::{ContinuousDomainScale, DiscreteDomainScale}, utils::set::DiscreteSet};



#[derive(Debug, Clone)]
pub struct ContinuousSizeScale {
    domain: (f64, f64),
    lower_bound: Option<f64>,
    upper_bound: Option<f64>,
    /// Output size range (min_size, max_size) in visual units (e.g., pixels)
    /// Default is (1.0, 6.0) following ggplot2
    range: (f64, f64),
    /// Whether to scale by area (true) or by radius (false)
    /// Default is true for perceptual uniformity (ggplot2 default)
    scale_area: bool,
}

impl ContinuousSizeScale {
    pub fn new() -> Self {
        Self {
            domain: (0.0, 1.0),
            lower_bound: None,
            upper_bound: None,
            range: (1.0, 6.0),
            scale_area: true,
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

    /// Set the output size range (min, max) in visual units
    pub fn with_range(mut self, range: (f64, f64)) -> Self {
        self.range = range;
        self
    }

    /// Set whether to scale by area (true) or radius (false)
    /// When true, sizes are perceptually uniform (2x value = 2x area)
    pub fn with_scale_area(mut self, scale_area: bool) -> Self {
        self.scale_area = scale_area;
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
        
        // Normalize to [0, 1]
        let t = if max_domain > min_domain {
            (v - min_domain) / (max_domain - min_domain)
        } else {
            0.5 // If domain has no range, use middle of output range
        };
        
        let (min_size, max_size) = self.range;
        
        // Apply area scaling if enabled (ggplot2 default behavior)
        let size = if self.scale_area {
            // Scale by area: size = sqrt(min_area + t * (max_area - min_area))
            // where area is proportional to radius^2
            let min_area = min_size * min_size;
            let max_area = max_size * max_size;
            let area = min_area + t * (max_area - min_area);
            area.sqrt()
        } else {
            // Linear scaling by radius
            min_size + t * (max_size - min_size)
        };
        
        Some(size)
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