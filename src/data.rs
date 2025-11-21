pub trait PrimitiveType: PartialEq + PartialOrd + Clone + Sized + Send + Sync + 'static {}

impl PrimitiveType for i64 {}
impl PrimitiveType for f64 {}
impl PrimitiveType for String {}

// Primitive value types for constant aesthetics
#[derive(Debug, Clone, PartialEq)]
pub enum PrimitiveValue {
    Int(i64),
    Float(f64),
    Str(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VectorType {
    Int,
    Float,
    Str,
}

impl std::fmt::Display for VectorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VectorType::Int => write!(f, "integer"),
            VectorType::Float => write!(f, "float"),
            VectorType::Str => write!(f, "string"),
        }
    }
}

pub trait GenericVector: Send + Sync {
    fn len(&self) -> usize;
    fn vtype(&self) -> VectorType;
    
    // Boxed iterator methods - these replace as_int/as_float/as_str for trait objects
    // Returns None if the vector is not of the requested type
    fn iter_int(&self) -> Option<Box<dyn Iterator<Item = &i64> + '_>> {
        None
    }
    fn iter_float(&self) -> Option<Box<dyn Iterator<Item = &f64> + '_>> {
        None
    }
    fn iter_str(&self) -> Option<Box<dyn Iterator<Item = &str> + '_>> {
        None
    }
}

// These traits are kept for concrete types that want zero-cost iteration
// but they're not dyn-compatible due to GATs
pub trait IntVector: GenericVector + Send + Sync {
    type Iter<'a>: Iterator<Item = &'a i64>
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_>;
}

pub trait FloatVector: GenericVector + Send + Sync {
    type Iter<'a>: Iterator<Item = &'a f64>
    where
        Self: 'a;
    fn iter(&self) -> Self::Iter<'_>;
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
            for &value in float_iter {
                min = min.min(value);
                max = max.max(value);
                found_any = true;
            }
        } else if let Some(int_iter) = vec.iter_int() {
            for &value in int_iter {
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
