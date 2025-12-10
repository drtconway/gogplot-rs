use ordered_float::OrderedFloat;
use std::hash::Hash;

pub trait PrimitiveType: PartialEq + PartialOrd + Clone + Sized + Send + Sync + 'static {
    type Sortable: Eq + Ord + Hash + Clone;

    fn to_primitive(&self) -> PrimitiveValue;

    fn to_sortable(&self) -> Self::Sortable;

    fn from_sortable(sortable: Self::Sortable) -> Self;
}

impl PrimitiveType for i64 {
    type Sortable = i64;

    fn to_primitive(&self) -> PrimitiveValue {
        PrimitiveValue::Int(*self)
    }

    fn to_sortable(&self) -> Self::Sortable {
        *self
    }

    fn from_sortable(sortable: Self::Sortable) -> Self {
        sortable
    }
}

impl PrimitiveType for f64 {
    type Sortable = OrderedFloat<f64>;

    fn to_primitive(&self) -> PrimitiveValue {
        PrimitiveValue::Float(*self)
    }

    fn to_sortable(&self) -> Self::Sortable {
        OrderedFloat(*self)
    }

    fn from_sortable(sortable: Self::Sortable) -> Self {
        sortable.0
    }
}

impl PrimitiveType for String {
    type Sortable = String;

    fn to_primitive(&self) -> PrimitiveValue {
        PrimitiveValue::Str(self.clone())
    }

    fn to_sortable(&self) -> Self::Sortable {
        self.clone()
    }

    fn from_sortable(sortable: Self::Sortable) -> Self {
        sortable
    }
}

impl PrimitiveType for bool {
    type Sortable = bool;

    fn to_primitive(&self) -> PrimitiveValue {
        PrimitiveValue::Bool(*self)
    }

    fn to_sortable(&self) -> Self::Sortable {
        *self
    }

    fn from_sortable(sortable: Self::Sortable) -> Self {
        sortable
    }
}

pub trait DiscreteType: PrimitiveType + Eq + Ord + Hash {}

impl DiscreteType for i64 {}
impl DiscreteType for String {}
impl DiscreteType for bool {}

pub trait ContinuousType: PrimitiveType {
    fn to_f64(&self) -> f64;
}

impl ContinuousType for f64 {
    fn to_f64(&self) -> f64 {
        *self
    }
}

impl ContinuousType for i64 {
    fn to_f64(&self) -> f64 {
        *self as f64
    }
}

// Primitive value types for constant aesthetics
#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveValue {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiscreteValue {
    Str(String),
    Int(i64),
    Bool(bool),
}

impl DiscreteValue {
    pub fn to_string(&self) -> String {
        match self {
            DiscreteValue::Int(i) => i.to_string(),
            DiscreteValue::Str(s) => s.clone(),
            DiscreteValue::Bool(b) => b.to_string(),
        }
    }
}

impl From<PrimitiveValue> for DiscreteValue {
    fn from(pv: PrimitiveValue) -> Self {
        match pv {
            PrimitiveValue::Str(s) => DiscreteValue::Str(s),
            PrimitiveValue::Int(i) => DiscreteValue::Int(i),
            PrimitiveValue::Bool(b) => DiscreteValue::Bool(b),
            _ => panic!("Unsupported primitive type for DiscreteValue"),
        }
    }
}

/// Simplified data type classification for determining scale types.
/// This distinguishes numeric types (which can use continuous scales)
/// from string types (which require categorical scales).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColumnDataType {
    /// Numeric types (Int, Float, Bool) - can use continuous or categorical scales
    Numeric,
    /// String type - must use categorical scales
    String,
}

/// Discriminated union of iterators over vector data.
/// This makes it impossible to miss handling a data type.
pub enum VectorIter<'a> {
    Int(Box<dyn Iterator<Item = i64> + 'a>),
    Float(Box<dyn Iterator<Item = f64> + 'a>),
    Str(Box<dyn Iterator<Item = &'a str> + 'a>),
    Bool(Box<dyn Iterator<Item = bool> + 'a>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorType {
    Int,
    Float,
    Str,
    Bool,
}

impl VectorType {
    /// Convert VectorType to simplified ColumnDataType classification
    pub fn to_column_data_type(self) -> ColumnDataType {
        match self {
            VectorType::Int | VectorType::Float | VectorType::Bool => ColumnDataType::Numeric,
            VectorType::Str => ColumnDataType::String,
        }
    }
}

impl std::fmt::Display for VectorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VectorType::Int => write!(f, "integer"),
            VectorType::Float => write!(f, "float"),
            VectorType::Str => write!(f, "string"),
            VectorType::Bool => write!(f, "boolean"),
        }
    }
}

pub trait GenericVector: Send + Sync {
    fn len(&self) -> usize;
    fn vtype(&self) -> VectorType;

    /// Get a discriminated union iterator over the vector's data.
    /// This is the preferred method as it makes exhaustive pattern matching possible.
    fn iter(&self) -> VectorIter<'_>;

    // Boxed iterator methods - these replace as_int/as_float/as_str for trait objects
    // Returns None if the vector is not of the requested type
    // These are convenience methods; prefer using iter() for exhaustive matching
    fn iter_int(&self) -> Option<Box<dyn Iterator<Item = i64> + '_>> {
        None
    }
    fn iter_float(&self) -> Option<Box<dyn Iterator<Item = f64> + '_>> {
        None
    }
    fn iter_str(&self) -> Option<Box<dyn Iterator<Item = &str> + '_>> {
        None
    }
    fn iter_bool(&self) -> Option<Box<dyn Iterator<Item = bool> + '_>> {
        None
    }
}

pub trait StrVector: GenericVector + Send + Sync {
    type Iter<'a>: Iterator<Item = &'a str>
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_>;
}

/// Helper function to compute min and max from a slice of GenericVectors
/// without copying all the data. Returns None if all vectors are empty.
pub(crate) fn compute_min_max(data: &[&dyn crate::data::GenericVector]) -> Option<(f64, f64)> {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut found_any = false;

    for vec in data {
        if let Some(float_iter) = vec.iter_float() {
            for value in float_iter {
                min = min.min(value);
                max = max.max(value);
                found_any = true;
            }
        } else if let Some(int_iter) = vec.iter_int() {
            for value in int_iter {
                let value_f64 = value as f64;
                min = min.min(value_f64);
                max = max.max(value_f64);
                found_any = true;
            }
        }
    }

    if found_any { Some((min, max)) } else { None }
}

// DataSource trait
pub trait DataSource: Send + Sync {
    fn get(&self, name: &str) -> Option<&dyn GenericVector>;
    fn column_names(&self) -> Vec<String>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clone into a new Box - required for cloning trait objects
    fn clone_box(&self) -> Box<dyn DataSource>;
}
