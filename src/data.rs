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

/// A DataSource that layers multiple data sources, checking them in order.
/// When looking up a column, it searches through the sources from first to last,
/// returning the first match found. This allows computed data (e.g., from stats)
/// to shadow original data columns.
pub struct StackedDataSource {
    sources: Vec<Box<dyn DataSource>>,
}

impl StackedDataSource {
    /// Creates a new StackedDataSource from a vector of data sources.
    /// Sources are searched in order when looking up columns.
    pub fn new(sources: Vec<Box<dyn DataSource>>) -> Self {
        Self { sources }
    }

    /// Creates a StackedDataSource with two layers.
    /// The top layer is searched first, allowing it to shadow the bottom layer.
    pub fn two_layer(top: Box<dyn DataSource>, bottom: Box<dyn DataSource>) -> Self {
        Self {
            sources: vec![top, bottom],
        }
    }
}

impl DataSource for StackedDataSource {
    fn get(&self, name: &str) -> Option<&dyn GenericVector> {
        // Search through sources in order, return first match
        self.sources.iter().find_map(|source| source.get(name))
    }

    fn column_names(&self) -> Vec<String> {
        // Collect all unique column names from all sources
        let mut names = Vec::new();
        let mut seen = std::collections::HashSet::new();
        
        for source in &self.sources {
            for name in source.column_names() {
                if seen.insert(name.clone()) {
                    names.push(name);
                }
            }
        }
        names
    }

    fn len(&self) -> usize {
        // Return length of first source, or 0 if no sources
        self.sources.first().map_or(0, |s| s.len())
    }
}

#[cfg(test)]
mod stacked_tests {
    use super::*;
    use crate::utils::dataframe::{DataFrame, IntVec};

    #[test]
    fn test_stacked_get_from_first_source() {
        let mut df1 = DataFrame::new();
        df1.add_column("x", Box::new(IntVec(vec![1, 2, 3])));
        df1.add_column("y", Box::new(IntVec(vec![10, 20, 30])));
        
        let mut df2 = DataFrame::new();
        df2.add_column("z", Box::new(IntVec(vec![100, 200, 300])));
        
        let stacked = StackedDataSource::new(vec![Box::new(df1), Box::new(df2)]);
        
        let x = stacked.get("x").unwrap();
        assert_eq!(x.as_int().unwrap().iter().copied().collect::<Vec<_>>(), vec![1, 2, 3]);
    }

    #[test]
    fn test_stacked_get_from_second_source() {
        let mut df1 = DataFrame::new();
        df1.add_column("x", Box::new(IntVec(vec![1, 2, 3])));
        
        let mut df2 = DataFrame::new();
        df2.add_column("y", Box::new(IntVec(vec![10, 20, 30])));
        
        let stacked = StackedDataSource::new(vec![Box::new(df1), Box::new(df2)]);
        
        let y = stacked.get("y").unwrap();
        assert_eq!(y.as_int().unwrap().iter().copied().collect::<Vec<_>>(), vec![10, 20, 30]);
    }

    #[test]
    fn test_stacked_shadowing() {
        // First source should shadow second when both have same column
        let mut df1 = DataFrame::new();
        df1.add_column("x", Box::new(IntVec(vec![1, 2, 3])));
        df1.add_column("y", Box::new(IntVec(vec![100, 200, 300]))); // This should be returned
        
        let mut df2 = DataFrame::new();
        df2.add_column("y", Box::new(IntVec(vec![10, 20, 30]))); // This should be shadowed
        df2.add_column("z", Box::new(IntVec(vec![1000, 2000, 3000])));
        
        let stacked = StackedDataSource::new(vec![Box::new(df1), Box::new(df2)]);
        
        // y should come from first source
        let y = stacked.get("y").unwrap();
        assert_eq!(y.as_int().unwrap().iter().copied().collect::<Vec<_>>(), vec![100, 200, 300]);
        
        // z should come from second source
        let z = stacked.get("z").unwrap();
        assert_eq!(z.as_int().unwrap().iter().copied().collect::<Vec<_>>(), vec![1000, 2000, 3000]);
    }

    #[test]
    fn test_stacked_two_layer() {
        let mut computed = DataFrame::new();
        computed.add_column("count", Box::new(IntVec(vec![5, 10, 15])));
        
        let mut original = DataFrame::new();
        original.add_column("x", Box::new(IntVec(vec![1, 2, 3])));
        original.add_column("y", Box::new(IntVec(vec![10, 20, 30])));
        
        let stacked = StackedDataSource::two_layer(Box::new(computed), Box::new(original));
        
        // Should find count in top layer
        let count = stacked.get("count").unwrap();
        assert_eq!(count.as_int().unwrap().iter().copied().collect::<Vec<_>>(), vec![5, 10, 15]);
        
        // Should find x in bottom layer
        let x = stacked.get("x").unwrap();
        assert_eq!(x.as_int().unwrap().iter().copied().collect::<Vec<_>>(), vec![1, 2, 3]);
    }

    #[test]
    fn test_stacked_column_names_unique() {
        let mut df1 = DataFrame::new();
        df1.add_column("x", Box::new(IntVec(vec![1, 2, 3])));
        df1.add_column("y", Box::new(IntVec(vec![10, 20, 30])));
        
        let mut df2 = DataFrame::new();
        df2.add_column("y", Box::new(IntVec(vec![100, 200, 300]))); // Duplicate name
        df2.add_column("z", Box::new(IntVec(vec![1000, 2000, 3000])));
        
        let stacked = StackedDataSource::new(vec![Box::new(df1), Box::new(df2)]);
        
        let names = stacked.column_names();
        assert_eq!(names.len(), 3); // x, y, z (no duplicates)
        assert!(names.contains(&"x".to_string()));
        assert!(names.contains(&"y".to_string()));
        assert!(names.contains(&"z".to_string()));
    }

    #[test]
    fn test_stacked_len() {
        let mut df1 = DataFrame::new();
        df1.add_column("x", Box::new(IntVec(vec![1, 2, 3])));
        
        let mut df2 = DataFrame::new();
        df2.add_column("y", Box::new(IntVec(vec![10, 20])));
        
        let stacked = StackedDataSource::new(vec![Box::new(df1), Box::new(df2)]);
        
        // Length should be from first source
        assert_eq!(stacked.len(), 3);
    }

    #[test]
    fn test_stacked_empty() {
        let stacked = StackedDataSource::new(vec![]);
        assert_eq!(stacked.len(), 0);
        assert!(stacked.is_empty());
        assert!(stacked.get("x").is_none());
    }

    #[test]
    fn test_stacked_get_nonexistent() {
        let mut df1 = DataFrame::new();
        df1.add_column("x", Box::new(IntVec(vec![1, 2, 3])));
        
        let stacked = StackedDataSource::new(vec![Box::new(df1)]);
        assert!(stacked.get("nonexistent").is_none());
    }
}