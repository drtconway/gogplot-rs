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

// DataSource trait
pub trait DataSource: Send + Sync {
    fn get(&self, name: &str) -> Option<&dyn GenericVector>;
    fn column_names(&self) -> Vec<String>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}