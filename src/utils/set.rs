use crate::data::{DiscreteType, DiscreteValue};

#[derive(Debug, Clone)]
pub struct DiscreteSet {
    ints: Vec<i64>,
    strings: Vec<String>,
    bools: Vec<bool>,
}

impl DiscreteSet {
    pub fn new() -> Self {
        Self {
            ints: Vec::new(),
            strings: Vec::new(),
            bools: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.ints.len() + self.strings.len() + self.bools.len()
    }

    pub fn add<T: DiscreteType>(&mut self, value: &T) {
        let value = DiscreteValue::from(value.to_primitive());
        match value {
            DiscreteValue::Int(v) => {
                self.ints.push(v);
            }
            DiscreteValue::Str(v) => {
                self.strings.push(v);
            }
            DiscreteValue::Bool(v) => {
                self.bools.push(v);
            }
        }
    }

    pub fn build(&mut self) {
        self.ints.sort();
        self.ints.dedup();
        self.strings.sort();
        self.strings.dedup();
        self.bools.sort();
        self.bools.dedup();
    }

    pub fn contains<T: DiscreteType>(&self, value: &T) -> bool {
        let value = DiscreteValue::from(value.to_primitive());
        match value {
            DiscreteValue::Int(v) => {
                let index = Self::lower_bound(&self.ints, &v);
                index < self.ints.len() && self.ints[index] == v
            }
            DiscreteValue::Str(v) => {
                let index = Self::lower_bound(&self.strings, &v);
                index < self.strings.len() && self.strings[index] == v
            }
            DiscreteValue::Bool(v) => {
                let index = Self::lower_bound(&self.bools, &v);
                index < self.bools.len() && self.bools[index] == v
            }
        }
    }

    pub fn ordinal<T: DiscreteType>(&self, value: &T) -> Option<usize> {
        let value = DiscreteValue::from(value.to_primitive());
        match value {
            DiscreteValue::Int(v) => {
                let index = Self::lower_bound(&self.ints, &v);
                if index < self.ints.len() && self.ints[index] == v {
                    Some(index)
                } else {
                    None
                }
            }
            DiscreteValue::Str(v) => {
                let index = Self::lower_bound(&self.strings, &v);
                if index < self.strings.len() && self.strings[index] == v {
                    Some(self.ints.len() + index)
                } else {
                    None
                }
            }
            DiscreteValue::Bool(v) => {
                let index = Self::lower_bound(&self.bools, &v);
                if index < self.bools.len() && self.bools[index] == v {
                    Some(self.ints.len() + self.strings.len() + index)
                } else {
                    None
                }
            }
        }
    }

    pub fn union(&mut self, other: &DiscreteSet) {
        for v in &other.ints {
            if !self.contains(v) {
                self.ints.push(*v);
            }
        }
        for v in &other.strings {
            if !self.contains(v) {
                self.strings.push(v.clone());
            }
        }
        for v in &other.bools {
            if !self.contains(v) {
                self.bools.push(*v);
            }
        }
        self.build();
    }
    pub fn iter(&self) -> impl Iterator<Item = DiscreteValue> + '_ {
        self.ints
            .iter()
            .cloned()
            .map(DiscreteValue::Int)
            .chain(self.strings.iter().cloned().map(DiscreteValue::Str))
            .chain(self.bools.iter().cloned().map(DiscreteValue::Bool))
    }

    /// Get the value at a specific ordinal position (0-indexed)
    pub fn get_at(&self, index: usize) -> Option<DiscreteValue> {
        if index < self.ints.len() {
            Some(DiscreteValue::Int(self.ints[index]))
        } else if index < self.ints.len() + self.strings.len() {
            Some(DiscreteValue::Str(self.strings[index - self.ints.len()].clone()))
        } else if index < self.len() {
            Some(DiscreteValue::Bool(self.bools[index - self.ints.len() - self.strings.len()]))
        } else {
            None
        }
    }

    fn lower_bound<T: Eq + Ord>(slice: &[T], value: &T) -> usize {
        let mut low = 0;
        let mut high = slice.len();

        while low < high {
            let mid = (low + high) / 2;
            if &slice[mid] < value {
                low = mid + 1;
            } else {
                high = mid;
            }
        }
        low
    }
}
