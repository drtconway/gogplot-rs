use crate::{
    data::{DiscreteType, DiscreteValue},
    utils::set::DiscreteSet,
};

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
        self.labels = self.elements.iter().map(|v| match v {
            DiscreteValue::Int(i) => i.to_string(),
            DiscreteValue::Str(s) => s.clone(),
            DiscreteValue::Bool(b) => b.to_string(),
        }).collect();
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
