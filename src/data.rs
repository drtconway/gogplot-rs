use internment::Intern;
use ordered_float::OrderedFloat;
use std::hash::Hash;

/// Interned string type for efficient cloning and comparison of categorical values.
/// Uses pointer comparison for equality and hashing.
pub type IStr = Intern<String>;

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
        PrimitiveValue::Str(IStr::new(self.clone()))
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
impl DiscreteType for IStr {}

// Implement PrimitiveType for IStr
impl PrimitiveType for IStr {
    type Sortable = IStr;

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
    Str(IStr),
    Bool(bool),
}

impl PrimitiveValue {
    pub fn to_f64(&self) -> Option<f64> {
        match self {
            PrimitiveValue::Int(i) => Some(*i as f64),
            PrimitiveValue::Float(f) => Some(*f),
            _ => None,
        }
    }
}

impl From<i64> for PrimitiveValue {
    fn from(i: i64) -> Self {
        PrimitiveValue::Int(i)
    }
}

impl From<f64> for PrimitiveValue {
    fn from(f: f64) -> Self {
        PrimitiveValue::Float(f)
    }
}

impl From<String> for PrimitiveValue {
    fn from(s: String) -> Self {
        PrimitiveValue::Str(IStr::new(s))
    }
}

impl From<&str> for PrimitiveValue {
    fn from(s: &str) -> Self {
        PrimitiveValue::Str(IStr::new(s.to_string()))
    }
}

impl From<IStr> for PrimitiveValue {
    fn from(s: IStr) -> Self {
        PrimitiveValue::Str(s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiscreteValue {
    Str(IStr),
    Int(i64),
    Bool(bool),
}

impl DiscreteValue {
    pub fn as_str(&self) -> &str {
        match self {
            DiscreteValue::Str(s) => s.as_ref(),
            _ => panic!("DiscreteValue is not a string"),
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            DiscreteValue::Int(i) => i.to_string(),
            DiscreteValue::Str(s) => s.to_string(),
            DiscreteValue::Bool(b) => b.to_string(),
        }
    }

    pub fn to_istr(&self) -> IStr {
        match self {
            DiscreteValue::Int(i) => IStr::new(i.to_string()),
            DiscreteValue::Str(s) => s.clone(),
            DiscreteValue::Bool(b) => IStr::new(b.to_string()),
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

impl From<&str> for DiscreteValue {
    fn from(s: &str) -> Self {
        DiscreteValue::Str(IStr::new(s.to_string()))
    }
}

impl From<String> for DiscreteValue {
    fn from(s: String) -> Self {
        DiscreteValue::Str(IStr::new(s))
    }
}

impl From<IStr> for DiscreteValue {
    fn from(s: IStr) -> Self {
        DiscreteValue::Str(s)
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

#[derive(Debug, PartialEq)]
pub enum VectorValue {
    Int(Vec<i64>),
    Float(Vec<f64>),
    Str(Vec<String>),
    Bool(Vec<bool>),
}

impl VectorValue {
    pub fn len(&self) -> usize {
        match self {
            VectorValue::Int(v) => v.len(),
            VectorValue::Float(v) => v.len(),
            VectorValue::Str(v) => v.len(),
            VectorValue::Bool(v) => v.len(),
        }
    }

    pub fn vtype(&self) -> VectorType {
        match self {
            VectorValue::Int(_) => VectorType::Int,
            VectorValue::Float(_) => VectorType::Float,
            VectorValue::Str(_) => VectorType::Str,
            VectorValue::Bool(_) => VectorType::Bool,
        }
    }

    pub fn cmp_at_index(&self, i: usize, j: usize) -> std::cmp::Ordering {
        match self {
            VectorValue::Int(v) => v[i].cmp(&v[j]),
            VectorValue::Float(v) => v[i].partial_cmp(&v[j]).unwrap(),
            VectorValue::Str(v) => v[i].cmp(&v[j]),
            VectorValue::Bool(v) => v[i].cmp(&v[j]),
        }
    }

    pub fn subset_iter<'a>(&'a self, indices: &'a [usize]) -> VectorIter<'a> {
        match self {
            VectorValue::Int(v) => {
                let iter = indices.iter().map(move |&i| v[i]);
                VectorIter::Int(Box::new(iter))
            }
            VectorValue::Float(v) => {
                let iter = indices.iter().map(move |&i| v[i]);
                VectorIter::Float(Box::new(iter))
            }
            VectorValue::Str(v) => {
                let iter = indices.iter().map(move |&i| v[i].as_str());
                VectorIter::Str(Box::new(iter))
            }
            VectorValue::Bool(v) => {
                let iter = indices.iter().map(move |&i| v[i]);
                VectorIter::Bool(Box::new(iter))
            }
        }
    }

    pub fn empty_copy(&self) -> Self {
        match self {
            VectorValue::Int(_) => VectorValue::Int(Vec::new()),
            VectorValue::Float(_) => VectorValue::Float(Vec::new()),
            VectorValue::Str(_) => VectorValue::Str(Vec::new()),
            VectorValue::Bool(_) => VectorValue::Bool(Vec::new()),
        }
    }

    pub fn append(&mut self, other: &VectorValue) {
        match (self, other) {
            (VectorValue::Int(v1), VectorValue::Int(v2)) => v1.extend_from_slice(v2),
            (VectorValue::Float(v1), VectorValue::Float(v2)) => v1.extend_from_slice(v2),
            (VectorValue::Str(v1), VectorValue::Str(v2)) => v1.extend_from_slice(v2),
            (VectorValue::Bool(v1), VectorValue::Bool(v2)) => v1.extend_from_slice(v2),
            _ => panic!("Cannot append VectorValues of different types"),
        }
    }
}

impl From<Vec<i64>> for VectorValue {
    fn from(v: Vec<i64>) -> Self {
        VectorValue::Int(v)
    }
}

impl From<Vec<f64>> for VectorValue {
    fn from(v: Vec<f64>) -> Self {
        VectorValue::Float(v)
    }
}

impl From<Vec<String>> for VectorValue {
    fn from(v: Vec<String>) -> Self {
        VectorValue::Str(v)
    }
}

impl From<Vec<&str>> for VectorValue {
    fn from(v: Vec<&str>) -> Self {
        VectorValue::Str(v.iter().map(|s| s.to_string()).collect())
    }
}

impl From<Vec<bool>> for VectorValue {
    fn from(v: Vec<bool>) -> Self {
        VectorValue::Bool(v)
    }
}

impl GenericVector for VectorValue {
    fn len(&self) -> usize {
        self.len()
    }

    fn vtype(&self) -> VectorType {
        self.vtype()
    }

    fn iter(&self) -> VectorIter<'_> {
        match self {
            VectorValue::Int(v) => {
                let iter = v.iter().cloned();
                VectorIter::Int(Box::new(iter))
            }
            VectorValue::Float(v) => {
                let iter = v.iter().cloned();
                VectorIter::Float(Box::new(iter))
            }
            VectorValue::Str(v) => {
                let iter = v.iter().map(|s| s.as_str());
                VectorIter::Str(Box::new(iter))
            }
            VectorValue::Bool(v) => {
                let iter = v.iter().cloned();
                VectorIter::Bool(Box::new(iter))
            }
        }
    }
}

/// Discriminated union of iterators over vector data.
/// This makes it impossible to miss handling a data type.
pub enum VectorIter<'a> {
    Int(Box<dyn Iterator<Item = i64> + 'a>),
    Float(Box<dyn Iterator<Item = f64> + 'a>),
    Str(Box<dyn Iterator<Item = &'a str> + 'a>),
    Bool(Box<dyn Iterator<Item = bool> + 'a>),
}

impl<'a> VectorIter<'a> {
    pub fn to_vector(self) -> VectorValue {
        match self {
            VectorIter::Int(iter) => VectorValue::Int(iter.collect()),
            VectorIter::Float(iter) => VectorValue::Float(iter.collect()),
            VectorIter::Str(iter) => VectorValue::Str(iter.map(|s| s.to_string()).collect()),
            VectorIter::Bool(iter) => VectorValue::Bool(iter.collect()),
        }
    }

    /// Convert string iterator to interned strings (cheap clone)
    pub fn to_istr_iter(self) -> Option<Box<dyn Iterator<Item = IStr> + 'a>> {
        match self {
            VectorIter::Str(iter) => Some(Box::new(iter.map(|s| IStr::new(s.to_string())))),
            _ => None,
        }
    }

    /// Convert to DiscreteValue iterator (uses IStr for strings)
    pub fn to_discrete_iter(self) -> Box<dyn Iterator<Item = DiscreteValue> + 'a> {
        match self {
            VectorIter::Int(iter) => Box::new(iter.map(DiscreteValue::Int)),
            VectorIter::Str(iter) => Box::new(iter.map(|s| DiscreteValue::Str(IStr::new(s.to_string())))),
            VectorIter::Bool(iter) => Box::new(iter.map(DiscreteValue::Bool)),
            VectorIter::Float(iter) => Box::new(iter.map(|f| DiscreteValue::Int(f as i64))),
        }
    }

    pub fn vtype(&self) -> VectorType {
        match self {
            VectorIter::Int(_) => VectorType::Int,
            VectorIter::Float(_) => VectorType::Float,
            VectorIter::Str(_) => VectorType::Str,
            VectorIter::Bool(_) => VectorType::Bool,
        }
    }
}

impl<'a> Iterator for VectorIter<'a> {
    type Item = PrimitiveValue;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            VectorIter::Int(iter) => iter.next().map(PrimitiveValue::Int),
            VectorIter::Float(iter) => iter.next().map(PrimitiveValue::Float),
            VectorIter::Str(iter) => iter.next().map(|s| PrimitiveValue::Str(IStr::new(s.to_string()))),
            VectorIter::Bool(iter) => iter.next().map(PrimitiveValue::Bool),
        }
    }
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

    /// Get the vector's data type
    fn vtype(&self) -> VectorType;

    /// Get a discriminated union iterator over the vector's data.
    /// This is the preferred method as it makes exhaustive pattern matching possible.
    fn iter(&self) -> VectorIter<'_>;

    // Boxed iterator methods - these replace as_int/as_float/as_str for trait objects
    // Returns None if the vector is not of the requested type
    // These are convenience methods; prefer using iter() for exhaustive matching
    fn iter_int(&self) -> Option<Box<dyn Iterator<Item = i64> + '_>> {
        if let VectorIter::Int(iter) = self.iter() {
            Some(iter)
        } else {
            None
        }
    }
    fn iter_float(&self) -> Option<Box<dyn Iterator<Item = f64> + '_>> {
        if let VectorIter::Float(iter) = self.iter() {
            Some(iter)
        } else {
            None
        }
    }
    fn iter_str(&self) -> Option<Box<dyn Iterator<Item = &str> + '_>> {
        if let VectorIter::Str(iter) = self.iter() {
            Some(iter)
        } else {
            None
        }
    }
    fn iter_bool(&self) -> Option<Box<dyn Iterator<Item = bool> + '_>> {
        if let VectorIter::Bool(iter) = self.iter() {
            Some(iter)
        } else {
            None
        }
    }
}

pub trait StrVector: GenericVector + Send + Sync {
    type Iter<'a>: Iterator<Item = &'a str>
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_>;
}

// DataSource trait
pub trait DataSource: Send + Sync {
    /// Get a column by name
    fn get(&self, name: &str) -> Option<&dyn GenericVector>;

    /// Get all column names
    fn column_names(&self) -> Vec<String>;

    /// Get the number of rows in the data source
    fn len(&self) -> usize;

    /// Check if the data source is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clone into a new Box - required for cloning trait objects
    fn clone_box(&self) -> Box<dyn DataSource>;
}
