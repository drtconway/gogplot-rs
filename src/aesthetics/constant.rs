use crate::data::{GenericVector, StrVector, VectorIter, VectorType};

pub struct ConstantIntVector {
    value: i64,
    len: usize,
}

impl ConstantIntVector {
    pub fn new(value: i64, len: usize) -> Self {
        Self { value, len }
    }
}

impl GenericVector for ConstantIntVector {
    fn vtype(&self) -> VectorType {
        VectorType::Int
    }

    fn len(&self) -> usize {
        self.len
    }

    fn iter(&self) -> VectorIter<'_> {
        VectorIter::Int(Box::new(std::iter::repeat(self.value).take(self.len)))
    }

    fn iter_int(&self) -> Option<Box<dyn Iterator<Item = i64> + '_>> {
        Some(Box::new(std::iter::repeat(self.value).take(self.len)))
    }
}

pub struct ConstantFloatVector {
    value: f64,
    len: usize,
}

impl ConstantFloatVector {
    pub fn new(value: f64, len: usize) -> Self {
        Self { value, len }
    }
}

impl GenericVector for ConstantFloatVector {
    fn vtype(&self) -> VectorType {
        VectorType::Float
    }

    fn len(&self) -> usize {
        self.len
    }

    fn iter(&self) -> VectorIter<'_> {
        VectorIter::Float(Box::new(std::iter::repeat(self.value).take(self.len)))
    }

    fn iter_float(&self) -> Option<Box<dyn Iterator<Item = f64> + '_>> {
        Some(Box::new(std::iter::repeat(self.value).take(self.len)))
    }
}

pub struct ConstantBoolVector {
    value: bool,
    len: usize,
}

impl ConstantBoolVector {
    pub fn new(value: bool, len: usize) -> Self {
        Self { value, len }
    }
}

impl GenericVector for ConstantBoolVector {
    fn vtype(&self) -> VectorType {
        VectorType::Bool
    }

    fn len(&self) -> usize {
        self.len
    }

    fn iter(&self) -> VectorIter<'_> {
        VectorIter::Bool(Box::new(std::iter::repeat(self.value).take(self.len)))
    }

    fn iter_bool(&self) -> Option<Box<dyn Iterator<Item = bool> + '_>> {
        Some(Box::new(std::iter::repeat(self.value).take(self.len)))
    }
}

pub struct ConstantStrVector {
    value: String,
    len: usize,
}

impl ConstantStrVector {
    pub fn new(value: String, len: usize) -> Self {
        Self { value, len }
    }
}

impl StrVector for ConstantStrVector {
    type Iter<'a>
        = std::iter::Take<std::iter::Repeat<&'a str>>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        std::iter::repeat(self.value.as_str()).take(self.len)
    }
}

impl GenericVector for ConstantStrVector {
    fn vtype(&self) -> VectorType {
        VectorType::Str
    }

    fn len(&self) -> usize {
        self.len
    }

    fn iter(&self) -> VectorIter<'_> {
        VectorIter::Str(Box::new(StrVector::iter(self)))
    }

    fn iter_str(&self) -> Option<Box<dyn Iterator<Item = &str> + '_>> {
        Some(Box::new(StrVector::iter(self)))
    }
}
