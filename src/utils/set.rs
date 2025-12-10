use crate::data::{DiscreteType, DiscreteValue};

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

    pub fn iter(&self) -> impl Iterator<Item = DiscreteValue> + '_ {
        self.ints
            .iter()
            .cloned()
            .map(DiscreteValue::Int)
            .chain(self.strings.iter().cloned().map(DiscreteValue::Str))
            .chain(self.bools.iter().cloned().map(DiscreteValue::Bool))
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
