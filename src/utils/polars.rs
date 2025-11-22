//! Polars integration for gogplot.
//!
//! This module provides an implementation of `DataSource` for Polars `DataFrame`.
//! This enables direct plotting of Polars data without conversion.
//!
//! # Usage
//!
//! Enable the `polars` feature in your `Cargo.toml`:
//! ```toml
//! gogplot = { version = "0.1", features = ["polars"] }
//! ```
//!
//! Then you can use Polars `DataFrame` directly as a data source:
//! ```ignore
//! use polars::prelude::*;
//! use gogplot::plot::Plot;
//!
//! let df = /* create or load Polars DataFrame */;
//! let plot = Plot::new(Some(Box::new(df)))
//!     .aes(|a| {
//!         a.x("column1");
//!         a.y("column2");
//!     })
//!     .geom_point();
//! ```

use crate::data::{DataSource, GenericVector, VectorIter};
use polars::prelude::*;

impl DataSource for DataFrame {
    fn get(&self, name: &str) -> Option<&dyn GenericVector> {
        // In Polars, column() returns a &Column which wraps a Series
        // We need to get the Series from the Column
        let column = self.column(name).ok()?;
        let series = column.as_materialized_series();
        // Return a reference to the series as a GenericVector
        Some(series as &dyn GenericVector)
    }

    fn len(&self) -> usize {
        self.height()
    }

    fn is_empty(&self) -> bool {
        self.height() == 0
    }

    fn column_names(&self) -> Vec<String> {
        self.get_column_names()
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    fn clone_box(&self) -> Box<dyn DataSource> {
        Box::new(self.clone())
    }
}

impl GenericVector for Series {
    fn len(&self) -> usize {
        // Access the underlying chunked array to get length
        // This avoids calling the trait method recursively
        self.chunk_lengths().sum()
    }

    fn vtype(&self) -> crate::data::VectorType {
        use polars::datatypes::DataType as PolarsDataType;
        match self.dtype() {
            PolarsDataType::Int8 | PolarsDataType::Int16 | PolarsDataType::Int32 | PolarsDataType::Int64 
            | PolarsDataType::UInt8 | PolarsDataType::UInt16 | PolarsDataType::UInt32 | PolarsDataType::UInt64 => {
                crate::data::VectorType::Int
            }
            PolarsDataType::Float32 | PolarsDataType::Float64 => crate::data::VectorType::Float,
            PolarsDataType::String => crate::data::VectorType::Str,
            PolarsDataType::Boolean => crate::data::VectorType::Bool,
            _ => {
                // Default to string for unsupported types
                crate::data::VectorType::Str
            }
        }
    }

    fn iter(&self) -> VectorIter<'_> {
        use polars::datatypes::DataType as PolarsDataType;
        match self.dtype() {
            // Integer types
            PolarsDataType::Int64 => {
                if let Ok(ca) = self.i64() {
                    VectorIter::Int(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(0))))
                } else {
                    VectorIter::Int(Box::new(std::iter::empty()))
                }
            }
            PolarsDataType::Int32 => {
                if let Ok(ca) = self.i32() {
                    VectorIter::Int(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(0) as i64)))
                } else {
                    VectorIter::Int(Box::new(std::iter::empty()))
                }
            }
            PolarsDataType::Int16 => {
                if let Ok(ca) = self.i16() {
                    VectorIter::Int(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(0) as i64)))
                } else {
                    VectorIter::Int(Box::new(std::iter::empty()))
                }
            }
            PolarsDataType::Int8 => {
                if let Ok(ca) = self.i8() {
                    VectorIter::Int(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(0) as i64)))
                } else {
                    VectorIter::Int(Box::new(std::iter::empty()))
                }
            }
            PolarsDataType::UInt64 => {
                if let Ok(ca) = self.u64() {
                    VectorIter::Int(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(0) as i64)))
                } else {
                    VectorIter::Int(Box::new(std::iter::empty()))
                }
            }
            PolarsDataType::UInt32 => {
                if let Ok(ca) = self.u32() {
                    VectorIter::Int(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(0) as i64)))
                } else {
                    VectorIter::Int(Box::new(std::iter::empty()))
                }
            }
            PolarsDataType::UInt16 => {
                if let Ok(ca) = self.u16() {
                    VectorIter::Int(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(0) as i64)))
                } else {
                    VectorIter::Int(Box::new(std::iter::empty()))
                }
            }
            PolarsDataType::UInt8 => {
                if let Ok(ca) = self.u8() {
                    VectorIter::Int(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(0) as i64)))
                } else {
                    VectorIter::Int(Box::new(std::iter::empty()))
                }
            }
            // Float types
            PolarsDataType::Float64 => {
                if let Ok(ca) = self.f64() {
                    VectorIter::Float(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(0.0))))
                } else {
                    VectorIter::Float(Box::new(std::iter::empty()))
                }
            }
            PolarsDataType::Float32 => {
                if let Ok(ca) = self.f32() {
                    VectorIter::Float(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(0.0) as f64)))
                } else {
                    VectorIter::Float(Box::new(std::iter::empty()))
                }
            }
            // String type
            PolarsDataType::String => {
                if let Ok(ca) = self.str() {
                    VectorIter::Str(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(""))))
                } else {
                    VectorIter::Str(Box::new(std::iter::empty()))
                }
            }
            // Boolean type
            PolarsDataType::Boolean => {
                if let Ok(ca) = self.bool() {
                    VectorIter::Bool(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(false))))
                } else {
                    VectorIter::Bool(Box::new(std::iter::empty()))
                }
            }
            // Default fallback
            _ => VectorIter::Str(Box::new(std::iter::empty())),
        }
    }

    fn iter_int(&self) -> Option<Box<dyn Iterator<Item = i64> + '_>> {
        use polars::datatypes::DataType as PolarsDataType;
        match self.dtype() {
            PolarsDataType::Int64 => {
                let ca = self.i64().ok()?;
                Some(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(0))))
            }
            PolarsDataType::Int32 => {
                let ca = self.i32().ok()?;
                Some(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(0) as i64)))
            }
            PolarsDataType::Int16 => {
                let ca = self.i16().ok()?;
                Some(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(0) as i64)))
            }
            PolarsDataType::Int8 => {
                let ca = self.i8().ok()?;
                Some(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(0) as i64)))
            }
            PolarsDataType::UInt64 => {
                let ca = self.u64().ok()?;
                Some(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(0) as i64)))
            }
            PolarsDataType::UInt32 => {
                let ca = self.u32().ok()?;
                Some(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(0) as i64)))
            }
            PolarsDataType::UInt16 => {
                let ca = self.u16().ok()?;
                Some(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(0) as i64)))
            }
            PolarsDataType::UInt8 => {
                let ca = self.u8().ok()?;
                Some(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(0) as i64)))
            }
            _ => None,
        }
    }

    fn iter_float(&self) -> Option<Box<dyn Iterator<Item = f64> + '_>> {
        use polars::datatypes::DataType as PolarsDataType;
        match self.dtype() {
            PolarsDataType::Float64 => {
                let ca = self.f64().ok()?;
                Some(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(0.0))))
            }
            PolarsDataType::Float32 => {
                let ca = self.f32().ok()?;
                Some(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(0.0) as f64)))
            }
            _ => None,
        }
    }

    fn iter_str(&self) -> Option<Box<dyn Iterator<Item = &str> + '_>> {
        use polars::datatypes::DataType as PolarsDataType;
        match self.dtype() {
            PolarsDataType::String => {
                let ca = self.str().ok()?;
                Some(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(""))))
            }
            _ => None,
        }
    }

    fn iter_bool(&self) -> Option<Box<dyn Iterator<Item = bool> + '_>> {
        use polars::datatypes::DataType as PolarsDataType;
        match self.dtype() {
            PolarsDataType::Boolean => {
                let ca = self.bool().ok()?;
                Some(Box::new(ca.into_iter().map(|opt| opt.unwrap_or(false))))
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataSource;
    use polars::prelude::*;

    #[test]
    fn test_polars_dataframe_as_datasource() {
        let df = df! {
            "x" => &[1i64, 2, 3, 4],
            "y" => &[1.5, 2.5, 3.5, 4.5],
            "label" => &["a", "b", "c", "d"]
        }
        .unwrap();

        // Test DataSource methods
        assert_eq!(df.len(), 4);
        assert!(!df.is_empty());
        assert_eq!(df.column_names().len(), 3);
        assert!(df.column_names().contains(&"x".to_string()));
        assert!(df.column_names().contains(&"y".to_string()));
        assert!(df.column_names().contains(&"label".to_string()));

        // Test getting integer column using DataSource trait
        let x_col = DataSource::get(&df, "x").unwrap();
        assert_eq!(x_col.len(), 4);
        let x_values: Vec<i64> = x_col.iter_int().unwrap().collect();
        assert_eq!(x_values, vec![1, 2, 3, 4]);

        // Test getting float column
        let y_col = DataSource::get(&df, "y").unwrap();
        let y_values: Vec<f64> = y_col.iter_float().unwrap().collect();
        assert_eq!(y_values, vec![1.5, 2.5, 3.5, 4.5]);

        // Test getting string column
        let label_col = DataSource::get(&df, "label").unwrap();
        let label_values: Vec<String> = label_col.iter_str().unwrap().map(|s| s.to_string()).collect();
        assert_eq!(label_values, vec!["a", "b", "c", "d"]);
    }

    #[test]
    fn test_polars_int32_column() {
        let df = df! {
            "x" => &[10i32, 20, 30]
        }
        .unwrap();

        let col = DataSource::get(&df, "x").unwrap();
        let values: Vec<i64> = col.iter_int().unwrap().collect();
        assert_eq!(values, vec![10, 20, 30]);
    }

    #[test]
    fn test_polars_mixed_numeric_columns() {
        let df = df! {
            "int" => &[1i64, 2, 3],
            "float" => &[1.5, 2.5, 3.5]
        }
        .unwrap();

        let int_col = DataSource::get(&df, "int").unwrap();
        let int_values: Vec<i64> = int_col.iter_int().unwrap().collect();
        assert_eq!(int_values, vec![1, 2, 3]);

        let float_col = DataSource::get(&df, "float").unwrap();
        let float_values: Vec<f64> = float_col.iter_float().unwrap().collect();
        assert_eq!(float_values, vec![1.5, 2.5, 3.5]);
    }

    #[test]
    fn test_polars_float32_column() {
        let df = df! {
            "y" => &[1.5f32, 2.5, 3.5]
        }
        .unwrap();

        let col = DataSource::get(&df, "y").unwrap();
        let values: Vec<f64> = col.iter_float().unwrap().collect();
        assert_eq!(values, vec![1.5, 2.5, 3.5]);
    }

    #[test]
    fn test_polars_empty_dataframe() {
        let df = df! {
            "x" => Vec::<i64>::new(),
            "y" => Vec::<f64>::new()
        }
        .unwrap();

        assert_eq!(df.len(), 0);
        assert!(df.is_empty());
        assert_eq!(df.column_names().len(), 2);
    }

    #[test]
    fn test_polars_nonexistent_column() {
        let df = df! {
            "x" => &[1i64, 2, 3]
        }
        .unwrap();

        assert!(DataSource::get(&df, "nonexistent").is_none());
    }

    #[test]
    fn test_polars_boolean_column() {
        let df = df! {
            "flag" => &[true, false, true, false]
        }
        .unwrap();

        let col = DataSource::get(&df, "flag").unwrap();
        let values: Vec<bool> = col.iter_bool().unwrap().collect();
        assert_eq!(values, vec![true, false, true, false]);
    }

    #[test]
    fn test_polars_boolean_vector_iter() {
        let df = df! {
            "active" => &[true, false, true]
        }
        .unwrap();

        let col = DataSource::get(&df, "active").unwrap();
        match col.iter() {
            VectorIter::Bool(mut iter) => {
                assert_eq!(iter.next(), Some(true));
                assert_eq!(iter.next(), Some(false));
                assert_eq!(iter.next(), Some(true));
                assert_eq!(iter.next(), None);
            }
            _ => panic!("Expected Bool variant"),
        }
    }

    #[test]
    fn test_polars_mixed_with_boolean() {
        let df = df! {
            "x" => &[1i64, 2, 3],
            "active" => &[true, false, true],
            "label" => &["a", "b", "c"]
        }
        .unwrap();

        assert_eq!(df.len(), 3);
        assert_eq!(df.column_names().len(), 3);

        // Check boolean values
        let bool_col = DataSource::get(&df, "active").unwrap();
        let bool_values: Vec<bool> = bool_col.iter_bool().unwrap().collect();
        assert_eq!(bool_values, vec![true, false, true]);

        // Check int values
        let int_col = DataSource::get(&df, "x").unwrap();
        let int_values: Vec<i64> = int_col.iter_int().unwrap().collect();
        assert_eq!(int_values, vec![1, 2, 3]);

        // Check string values
        let str_col = DataSource::get(&df, "label").unwrap();
        let str_values: Vec<String> = str_col.iter_str().unwrap().map(|s| s.to_string()).collect();
        assert_eq!(str_values, vec!["a", "b", "c"]);
    }
}
