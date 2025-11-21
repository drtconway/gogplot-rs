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

pub enum VectorType {
    Int,
    Float,
    Str,
}

pub trait GenericVector: Send + Sync {
    fn len(&self) -> usize;
    fn vtype(&self) -> VectorType;
    fn as_int(&self) -> Option<&dyn IntVector> {
        None
    }
    fn as_float(&self) -> Option<&dyn FloatVector> {
        None
    }
    fn as_str(&self) -> Option<&dyn StrVector> {
        None
    }
}

pub trait IntVector: GenericVector + Send + Sync {
    fn iter(&self) -> std::slice::Iter<'_, i64>;
}

pub trait FloatVector: GenericVector + Send + Sync {
    fn iter(&self) -> std::slice::Iter<'_, f64>;
}

pub trait StrVector: GenericVector + Send + Sync {
    fn iter(&self) -> std::slice::Iter<'_, String>;
}

/// Helper function to compute min and max from a slice of GenericVectors
/// without copying all the data. Returns None if all vectors are empty.
pub(crate) fn compute_min_max(data: &[&dyn crate::data::GenericVector]) -> Option<(f64, f64)> {
    let mut min = f64::INFINITY;
    let mut max = f64::NEG_INFINITY;
    let mut found_any = false;

    for vec in data {
        if let Some(float_vec) = vec.as_float() {
            for &value in float_vec.iter() {
                min = min.min(value);
                max = max.max(value);
                found_any = true;
            }
        } else if let Some(int_vec) = vec.as_int() {
            for &value in int_vec.iter() {
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
