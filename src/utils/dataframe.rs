use crate::data::{DataSource, GenericVector, StrVector, VectorIter, VectorType};
use std::collections::HashMap;

// Concrete vector implementations
pub struct IntVec(pub Vec<i64>);

impl GenericVector for IntVec {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn vtype(&self) -> VectorType {
        VectorType::Int
    }

    fn iter(&self) -> VectorIter<'_> {
        VectorIter::Int(Box::new(self.0.iter().copied()))
    }

    fn iter_int(&self) -> Option<Box<dyn Iterator<Item = i64> + '_>> {
        Some(Box::new(self.0.iter().copied()))
    }
}

pub struct FloatVec(pub Vec<f64>);

impl GenericVector for FloatVec {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn vtype(&self) -> VectorType {
        VectorType::Float
    }

    fn iter(&self) -> VectorIter<'_> {
        VectorIter::Float(Box::new(self.0.iter().copied()))
    }

    fn iter_float(&self) -> Option<Box<dyn Iterator<Item = f64> + '_>> {
        Some(Box::new(self.0.iter().copied()))
    }
}

pub struct StrVec(pub Vec<String>);

impl GenericVector for StrVec {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn vtype(&self) -> VectorType {
        VectorType::Str
    }

    fn iter(&self) -> VectorIter<'_> {
        VectorIter::Str(Box::new(self.0.iter().map(|s| s.as_str())))
    }

    fn iter_str(&self) -> Option<Box<dyn Iterator<Item = &str> + '_>> {
        Some(Box::new(self.0.iter().map(|s| s.as_str())))
    }
}

impl StrVector for StrVec {
    type Iter<'a> = std::iter::Map<std::slice::Iter<'a, String>, fn(&'a String) -> &'a str> where Self: 'a;
    
    fn iter(&self) -> Self::Iter<'_> {
        self.0.iter().map(|s| s.as_str())
    }
}

impl From<Vec<&str>> for StrVec {
    fn from(vec: Vec<&str>) -> Self {
        StrVec(vec.into_iter().map(|s| s.to_string()).collect())
    }
}

pub struct BoolVec(pub Vec<bool>);

impl GenericVector for BoolVec {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn vtype(&self) -> VectorType {
        VectorType::Bool
    }

    fn iter(&self) -> VectorIter<'_> {
        VectorIter::Bool(Box::new(self.0.iter().copied()))
    }

    fn iter_bool(&self) -> Option<Box<dyn Iterator<Item = bool> + '_>> {
        Some(Box::new(self.0.iter().copied()))
    }
}

/// A simple DataFrame implementation that holds named columns of data.
///
/// Each column is a vector of values of the same type (i64, f64, or String).
/// All columns must have the same length.
///
/// # Examples
///
/// ```ignore
/// use gogplot::utils::dataframe::{DataFrame, IntVec, FloatVec};
///
/// let mut df = DataFrame::new();
/// df.add_column("x", Box::new(IntVec(vec![1, 2, 3, 4, 5])));
/// df.add_column("y", Box::new(FloatVec(vec![2.0, 4.0, 6.0, 8.0, 10.0])));
/// ```
pub struct DataFrame {
    columns: HashMap<String, Box<dyn GenericVector>>,
    len: usize,
}

impl DataFrame {
    /// Create a new empty DataFrame
    pub fn new() -> Self {
        Self {
            columns: HashMap::new(),
            len: 0,
        }
    }

    /// Add a column to the DataFrame
    ///
    /// # Panics
    ///
    /// Panics if the column length doesn't match existing columns
    pub fn add_column(&mut self, name: impl Into<String>, column: Box<dyn GenericVector>) {
        let name = name.into();
        let col_len = column.len();

        if self.columns.is_empty() {
            self.len = col_len;
        } else if col_len != self.len {
            panic!(
                "Column '{}' has length {} but DataFrame has length {}",
                name, col_len, self.len
            );
        }

        self.columns.insert(name, column);
    }

    /// Create a new DataFrame with the given columns
    ///
    /// # Panics
    ///
    /// Panics if columns have different lengths
    pub fn from_columns(columns: Vec<(impl Into<String>, Box<dyn GenericVector>)>) -> Self {
        let mut df = Self::new();
        for (name, column) in columns {
            df.add_column(name, column);
        }
        df
    }
}

impl Default for DataFrame {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for DataFrame {
    fn clone(&self) -> Self {
        let mut new_columns = HashMap::new();
        for (name, col) in &self.columns {
            // Reconstruct each column vector using discriminated union
            let new_col: Box<dyn GenericVector> = match col.iter() {
                VectorIter::Int(iter) => Box::new(IntVec(iter.collect())),
                VectorIter::Float(iter) => Box::new(FloatVec(iter.collect())),
                VectorIter::Str(iter) => Box::new(StrVec(iter.map(|s| s.to_string()).collect())),
                VectorIter::Bool(iter) => Box::new(BoolVec(iter.collect())),
            };
            new_columns.insert(name.clone(), new_col);
        }
        DataFrame {
            columns: new_columns,
            len: self.len,
        }
    }
}

impl DataSource for DataFrame {
    fn get(&self, name: &str) -> Option<&dyn GenericVector> {
        self.columns.get(name).map(|b| b.as_ref())
    }

    fn column_names(&self) -> Vec<String> {
        self.columns.keys().cloned().collect()
    }

    fn len(&self) -> usize {
        self.len
    }
    
    fn clone_box(&self) -> Box<dyn DataSource> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intvec_len() {
        let vec = IntVec(vec![1, 2, 3, 4, 5]);
        assert_eq!(vec.len(), 5);
    }

    #[test]
    fn test_intvec_iter() {
        let vec = IntVec(vec![1, 2, 3]);
        let values: Vec<i64> = vec.iter_int().unwrap().collect();
        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn test_intvec_as_int() {
        let vec = IntVec(vec![1, 2, 3]);
        assert!(vec.iter_int().is_some());
        assert!(vec.iter_float().is_none());
        assert!(vec.iter_str().is_none());
    }

    #[test]
    fn test_floatvec_len() {
        let vec = FloatVec(vec![1.0, 2.0, 3.0]);
        assert_eq!(vec.len(), 3);
    }

    #[test]
    fn test_floatvec_iter() {
        let vec = FloatVec(vec![1.5, 2.5, 3.5]);
        let values: Vec<f64> = vec.iter_float().unwrap().collect();
        assert_eq!(values, vec![1.5, 2.5, 3.5]);
    }

    #[test]
    fn test_floatvec_as_float() {
        let vec = FloatVec(vec![1.0, 2.0]);
        assert!(vec.iter_float().is_some());
        assert!(vec.iter_int().is_none());
        assert!(vec.iter_str().is_none());
    }

    #[test]
    fn test_strvec_len() {
        let vec = StrVec(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(vec.len(), 2);
    }

    #[test]
    fn test_strvec_iter() {
        let vec = StrVec(vec!["hello".to_string(), "world".to_string()]);
        let values: Vec<String> = StrVector::iter(&vec).map(|s| s.to_string()).collect();
        assert_eq!(values, vec!["hello".to_string(), "world".to_string()]);
    }

    #[test]
    fn test_strvec_as_str() {
        let vec = StrVec(vec!["test".to_string()]);
        assert!(vec.iter_str().is_some());
        assert!(vec.iter_int().is_none());
        assert!(vec.iter_float().is_none());
    }

    #[test]
    fn test_dataframe_new() {
        let df = DataFrame::new();
        assert_eq!(df.len(), 0);
        assert!(df.is_empty());
        assert_eq!(df.column_names().len(), 0);
    }

    #[test]
    fn test_dataframe_add_column() {
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(IntVec(vec![1, 2, 3])));

        assert_eq!(df.len(), 3);
        assert!(!df.is_empty());
        assert_eq!(df.column_names().len(), 1);
        assert!(df.column_names().contains(&"x".to_string()));
    }

    #[test]
    fn test_dataframe_multiple_columns() {
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(IntVec(vec![1, 2, 3])));
        df.add_column("y", Box::new(FloatVec(vec![1.0, 2.0, 3.0])));
        df.add_column(
            "label",
            Box::new(StrVec(vec![
                "a".to_string(),
                "b".to_string(),
                "c".to_string(),
            ])),
        );

        assert_eq!(df.len(), 3);
        assert_eq!(df.column_names().len(), 3);
    }

    #[test]
    fn test_dataframe_get_column() {
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(IntVec(vec![1, 2, 3])));

        let col = df.get("x");
        assert!(col.is_some());
        assert_eq!(col.unwrap().len(), 3);

        let missing = df.get("z");
        assert!(missing.is_none());
    }

    #[test]
    fn test_dataframe_get_column_as_int() {
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(IntVec(vec![10, 20, 30])));

        let col = df.get("x").unwrap();
        let int_iter = col.iter_int().unwrap();
        let values: Vec<i64> = int_iter.collect();
        assert_eq!(values, vec![10, 20, 30]);
    }

    #[test]
    fn test_dataframe_get_column_as_float() {
        let mut df = DataFrame::new();
        df.add_column("y", Box::new(FloatVec(vec![1.5, 2.5, 3.5])));

        let col = df.get("y").unwrap();
        let float_iter = col.iter_float().unwrap();
        let values: Vec<f64> = float_iter.collect();
        assert_eq!(values, vec![1.5, 2.5, 3.5]);
    }

    #[test]
    fn test_dataframe_get_column_as_str() {
        let mut df = DataFrame::new();
        df.add_column(
            "label",
            Box::new(StrVec(vec!["a".to_string(), "b".to_string()])),
        );

        let col = df.get("label").unwrap();
        let str_iter = col.iter_str().unwrap();
        let values: Vec<String> = str_iter.map(|s| s.to_string()).collect();
        assert_eq!(values, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    #[should_panic(expected = "Column 'y' has length 2 but DataFrame has length 3")]
    fn test_dataframe_mismatched_length() {
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(IntVec(vec![1, 2, 3])));
        df.add_column("y", Box::new(FloatVec(vec![1.0, 2.0]))); // Wrong length!
    }

    #[test]
    fn test_dataframe_from_columns() {
        let df = DataFrame::from_columns(vec![
            (
                "x",
                Box::new(IntVec(vec![1, 2, 3])) as Box<dyn GenericVector>,
            ),
            (
                "y",
                Box::new(FloatVec(vec![4.0, 5.0, 6.0])) as Box<dyn GenericVector>,
            ),
        ]);

        assert_eq!(df.len(), 3);
        assert_eq!(df.column_names().len(), 2);
        assert!(df.get("x").is_some());
        assert!(df.get("y").is_some());
    }

    #[test]
    fn test_dataframe_default() {
        let df = DataFrame::default();
        assert_eq!(df.len(), 0);
        assert!(df.is_empty());
    }

    #[test]
    fn test_vector_type() {
        let int_vec = IntVec(vec![1, 2]);
        let float_vec = FloatVec(vec![1.0, 2.0]);
        let str_vec = StrVec(vec!["a".to_string()]);
        let bool_vec = BoolVec(vec![true, false]);

        matches!(int_vec.vtype(), VectorType::Int);
        matches!(float_vec.vtype(), VectorType::Float);
        matches!(str_vec.vtype(), VectorType::Str);
        matches!(bool_vec.vtype(), VectorType::Bool);
    }

    #[test]
    fn test_boolvec_len() {
        let vec = BoolVec(vec![true, false, true]);
        assert_eq!(vec.len(), 3);
    }

    #[test]
    fn test_boolvec_iter() {
        let vec = BoolVec(vec![true, false, true]);
        let values: Vec<bool> = vec.iter_bool().unwrap().collect();
        assert_eq!(values, vec![true, false, true]);
    }

    #[test]
    fn test_boolvec_as_bool() {
        let vec = BoolVec(vec![true, false]);
        assert!(vec.iter_bool().is_some());
        assert!(vec.iter_int().is_none());
        assert!(vec.iter_float().is_none());
        assert!(vec.iter_str().is_none());
    }

    #[test]
    fn test_vector_iter_discriminated() {
        let int_vec = IntVec(vec![1, 2, 3]);
        let float_vec = FloatVec(vec![1.5, 2.5]);
        let str_vec = StrVec(vec!["a".to_string(), "b".to_string()]);
        let bool_vec = BoolVec(vec![true, false]);

        match GenericVector::iter(&int_vec) {
            VectorIter::Int(mut iter) => assert_eq!(iter.next(), Some(1)),
            _ => panic!("Expected Int variant"),
        }

        match GenericVector::iter(&float_vec) {
            VectorIter::Float(mut iter) => assert_eq!(iter.next(), Some(1.5)),
            _ => panic!("Expected Float variant"),
        }

        match GenericVector::iter(&str_vec) {
            VectorIter::Str(mut iter) => assert_eq!(iter.next(), Some("a")),
            _ => panic!("Expected Str variant"),
        }

        match GenericVector::iter(&bool_vec) {
            VectorIter::Bool(mut iter) => assert_eq!(iter.next(), Some(true)),
            _ => panic!("Expected Bool variant"),
        }
    }
}
