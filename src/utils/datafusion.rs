//! Apache Arrow integration for gogplot.
//!
//! This module provides implementations of `GenericVector`, `StrVector`, and `DataSource`
//! for Apache Arrow array types and `RecordBatch`. This enables zero-copy plotting of data
//! from Arrow/DataFusion/Polars and other Arrow-based data processing frameworks.
//!
//! # Usage
//!
//! Enable the `arrow` feature in your `Cargo.toml`:
//! ```toml
//! gogplot = { version = "0.1", features = ["arrow"] }
//! ```
//!
//! Then you can use `RecordBatch` directly as a data source:
//! ```ignore
//! use arrow::record_batch::RecordBatch;
//! use gogplot::plot::Plot;
//!
//! let batch = /* create or load RecordBatch */;
//! let plot = Plot::new(Some(Box::new(batch)))
//!     .aes(|a| {
//!         a.x("column1");
//!         a.y("column2");
//!     })
//!     .geom_point();
//! ```

use crate::data::{GenericVector, StrVector, VectorIter};
use arrow::array::{
    Array, BooleanArray, DictionaryArray, Float32Array, Float64Array, Int8Array, Int16Array,
    Int32Array, Int64Array, LargeStringArray, StringArray, StringViewArray, UInt8Array,
    UInt16Array, UInt32Array, UInt64Array,
};
use arrow::datatypes::{DataType, Int32Type};
use arrow::record_batch::RecordBatch;

impl GenericVector for BooleanArray {
    fn len(&self) -> usize {
        arrow::array::Array::len(self)
    }

    fn vtype(&self) -> crate::data::VectorType {
        crate::data::VectorType::Bool
    }

    fn iter(&self) -> VectorIter<'_> {
        VectorIter::Bool(Box::new((0..arrow::array::Array::len(self)).map(|i| self.value(i))))
    }

    fn iter_bool(&self) -> Option<Box<dyn Iterator<Item = bool> + '_>> {
        Some(Box::new((0..arrow::array::Array::len(self)).map(|i| self.value(i))))
    }
}

impl GenericVector for Int64Array {
    fn len(&self) -> usize {
        arrow::array::Array::len(self)
    }

    fn vtype(&self) -> crate::data::VectorType {
        crate::data::VectorType::Int
    }

    fn iter(&self) -> VectorIter<'_> {
        VectorIter::Int(Box::new(self.values().iter().copied()))
    }

    fn iter_int(&self) -> Option<Box<dyn Iterator<Item = i64> + '_>> {
        Some(Box::new(self.values().iter().copied()))
    }
}

// Note: For integer types other than i64, we would need a GAT trait to return zero-copy iterators
// because it requires returning &i64, but these arrays store native types (i8, i16, etc.).
// However, they do implement GenericVector::iter_int() which returns Iterator<Item = i64>
// by value (with casting). Similarly for Float32Array with FloatVector.

impl GenericVector for Int8Array {
    fn len(&self) -> usize {
        arrow::array::Array::len(self)
    }

    fn vtype(&self) -> crate::data::VectorType {
        crate::data::VectorType::Int
    }

    fn iter(&self) -> VectorIter<'_> {
        VectorIter::Int(Box::new(self.values().iter().map(|&v| v as i64)))
    }

    fn iter_int(&self) -> Option<Box<dyn Iterator<Item = i64> + '_>> {
        Some(Box::new(self.values().iter().map(|&v| v as i64)))
    }
}

impl GenericVector for Int16Array {
    fn len(&self) -> usize {
        arrow::array::Array::len(self)
    }

    fn vtype(&self) -> crate::data::VectorType {
        crate::data::VectorType::Int
    }

    fn iter(&self) -> VectorIter<'_> {
        VectorIter::Int(Box::new(self.values().iter().map(|&v| v as i64)))
    }

    fn iter_int(&self) -> Option<Box<dyn Iterator<Item = i64> + '_>> {
        Some(Box::new(self.values().iter().map(|&v| v as i64)))
    }
}

impl GenericVector for Int32Array {
    fn len(&self) -> usize {
        arrow::array::Array::len(self)
    }

    fn vtype(&self) -> crate::data::VectorType {
        crate::data::VectorType::Int
    }

    fn iter(&self) -> VectorIter<'_> {
        VectorIter::Int(Box::new(self.values().iter().map(|&v| v as i64)))
    }

    fn iter_int(&self) -> Option<Box<dyn Iterator<Item = i64> + '_>> {
        Some(Box::new(self.values().iter().map(|&v| v as i64)))
    }
}

impl GenericVector for UInt8Array {
    fn len(&self) -> usize {
        arrow::array::Array::len(self)
    }

    fn vtype(&self) -> crate::data::VectorType {
        crate::data::VectorType::Int
    }

    fn iter(&self) -> VectorIter<'_> {
        VectorIter::Int(Box::new(self.values().iter().map(|&v| v as i64)))
    }

    fn iter_int(&self) -> Option<Box<dyn Iterator<Item = i64> + '_>> {
        Some(Box::new(self.values().iter().map(|&v| v as i64)))
    }
}

impl GenericVector for UInt16Array {
    fn len(&self) -> usize {
        arrow::array::Array::len(self)
    }

    fn vtype(&self) -> crate::data::VectorType {
        crate::data::VectorType::Int
    }

    fn iter(&self) -> VectorIter<'_> {
        VectorIter::Int(Box::new(self.values().iter().map(|&v| v as i64)))
    }

    fn iter_int(&self) -> Option<Box<dyn Iterator<Item = i64> + '_>> {
        Some(Box::new(self.values().iter().map(|&v| v as i64)))
    }
}

impl GenericVector for UInt32Array {
    fn len(&self) -> usize {
        arrow::array::Array::len(self)
    }

    fn vtype(&self) -> crate::data::VectorType {
        crate::data::VectorType::Int
    }

    fn iter(&self) -> VectorIter<'_> {
        VectorIter::Int(Box::new(self.values().iter().map(|&v| v as i64)))
    }

    fn iter_int(&self) -> Option<Box<dyn Iterator<Item = i64> + '_>> {
        Some(Box::new(self.values().iter().map(|&v| v as i64)))
    }
}

impl GenericVector for UInt64Array {
    fn len(&self) -> usize {
        arrow::array::Array::len(self)
    }

    fn vtype(&self) -> crate::data::VectorType {
        crate::data::VectorType::Int
    }

    fn iter(&self) -> VectorIter<'_> {
        // Note: UInt64 values > i64::MAX will wrap when cast to i64
        VectorIter::Int(Box::new(self.values().iter().map(|&v| v as i64)))
    }

    fn iter_int(&self) -> Option<Box<dyn Iterator<Item = i64> + '_>> {
        // Note: UInt64 values > i64::MAX will wrap when cast to i64
        Some(Box::new(self.values().iter().map(|&v| v as i64)))
    }
}

impl GenericVector for Float32Array {
    fn len(&self) -> usize {
        arrow::array::Array::len(self)
    }

    fn vtype(&self) -> crate::data::VectorType {
        crate::data::VectorType::Float
    }

    fn iter(&self) -> VectorIter<'_> {
        VectorIter::Float(Box::new(self.values().iter().map(|&v| v as f64)))
    }

    fn iter_float(&self) -> Option<Box<dyn Iterator<Item = f64> + '_>> {
        Some(Box::new(self.values().iter().map(|&v| v as f64)))
    }
}

impl GenericVector for Float64Array {
    fn len(&self) -> usize {
        arrow::array::Array::len(self)
    }

    fn vtype(&self) -> crate::data::VectorType {
        crate::data::VectorType::Float
    }

    fn iter(&self) -> VectorIter<'_> {
        VectorIter::Float(Box::new(self.values().iter().copied()))
    }

    fn iter_float(&self) -> Option<Box<dyn Iterator<Item = f64> + '_>> {
        Some(Box::new(self.values().iter().copied()))
    }
}

impl GenericVector for StringArray {
    fn len(&self) -> usize {
        arrow::array::Array::len(self)
    }

    fn vtype(&self) -> crate::data::VectorType {
        crate::data::VectorType::Str
    }

    fn iter(&self) -> VectorIter<'_> {
        VectorIter::Str(Box::new(StringArrayIter {
            array: self,
            index: 0,
        }))
    }

    fn iter_str(&self) -> Option<Box<dyn Iterator<Item = &str> + '_>> {
        Some(Box::new(StringArrayIter {
            array: self,
            index: 0,
        }))
    }
}

/// Iterator for StringArray that yields &str values
pub struct StringArrayIter<'a> {
    array: &'a StringArray,
    index: usize,
}

impl<'a> Iterator for StringArrayIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < arrow::array::Array::len(self.array) {
            let value = self.array.value(self.index);
            self.index += 1;
            Some(value)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = arrow::array::Array::len(self.array) - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for StringArrayIter<'a> {
    fn len(&self) -> usize {
        arrow::array::Array::len(self.array) - self.index
    }
}

impl StrVector for StringArray {
    type Iter<'a>
        = StringArrayIter<'a>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        StringArrayIter {
            array: self,
            index: 0,
        }
    }
}

impl GenericVector for LargeStringArray {
    fn len(&self) -> usize {
        arrow::array::Array::len(self)
    }

    fn vtype(&self) -> crate::data::VectorType {
        crate::data::VectorType::Str
    }

    fn iter(&self) -> VectorIter<'_> {
        VectorIter::Str(Box::new(LargeStringArrayIter {
            array: self,
            index: 0,
        }))
    }

    fn iter_str(&self) -> Option<Box<dyn Iterator<Item = &str> + '_>> {
        Some(Box::new(LargeStringArrayIter {
            array: self,
            index: 0,
        }))
    }
}

/// Iterator for LargeStringArray that yields &str values
pub struct LargeStringArrayIter<'a> {
    array: &'a LargeStringArray,
    index: usize,
}

impl<'a> Iterator for LargeStringArrayIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < arrow::array::Array::len(self.array) {
            let value = self.array.value(self.index);
            self.index += 1;
            Some(value)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = arrow::array::Array::len(self.array) - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for LargeStringArrayIter<'a> {
    fn len(&self) -> usize {
        arrow::array::Array::len(self.array) - self.index
    }
}

impl StrVector for LargeStringArray {
    type Iter<'a>
        = LargeStringArrayIter<'a>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        LargeStringArrayIter {
            array: self,
            index: 0,
        }
    }
}

impl GenericVector for StringViewArray {
    fn len(&self) -> usize {
        arrow::array::Array::len(self)
    }

    fn vtype(&self) -> crate::data::VectorType {
        crate::data::VectorType::Str
    }

    fn iter(&self) -> VectorIter<'_> {
        VectorIter::Str(Box::new(StringViewArrayIter {
            array: self,
            index: 0,
        }))
    }

    fn iter_str(&self) -> Option<Box<dyn Iterator<Item = &str> + '_>> {
        Some(Box::new(StringViewArrayIter {
            array: self,
            index: 0,
        }))
    }
}

/// Iterator for StringViewArray that yields &str values
pub struct StringViewArrayIter<'a> {
    array: &'a StringViewArray,
    index: usize,
}

impl<'a> Iterator for StringViewArrayIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < arrow::array::Array::len(self.array) {
            let value = self.array.value(self.index);
            self.index += 1;
            Some(value)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = arrow::array::Array::len(self.array) - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for StringViewArrayIter<'a> {
    fn len(&self) -> usize {
        arrow::array::Array::len(self.array) - self.index
    }
}

impl StrVector for StringViewArray {
    type Iter<'a>
        = StringViewArrayIter<'a>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        StringViewArrayIter {
            array: self,
            index: 0,
        }
    }
}

// Dictionary array implementations for Utf8 values
impl GenericVector for DictionaryArray<Int32Type> {
    fn len(&self) -> usize {
        arrow::array::Array::len(self)
    }

    fn vtype(&self) -> crate::data::VectorType {
        // Dictionary arrays with string values are treated as strings
        crate::data::VectorType::Str
    }

    fn iter(&self) -> VectorIter<'_> {
        // Determine the value type of the dictionary
        let values = self.values();

        if values.as_any().is::<StringArray>() {
            VectorIter::Str(Box::new(DictUtf8ArrayIter {
                dict_array: self,
                values_array: values.as_any().downcast_ref::<StringArray>().unwrap(),
                index: 0,
            }))
        } else if values.as_any().is::<LargeStringArray>() {
            VectorIter::Str(Box::new(DictLargeUtf8ArrayIter {
                dict_array: self,
                values_array: values.as_any().downcast_ref::<LargeStringArray>().unwrap(),
                index: 0,
            }))
        } else if values.as_any().is::<StringViewArray>() {
            VectorIter::Str(Box::new(DictUtf8ViewArrayIter {
                dict_array: self,
                values_array: values.as_any().downcast_ref::<StringViewArray>().unwrap(),
                index: 0,
            }))
        } else {
            // Fallback - return empty iterator
            VectorIter::Str(Box::new(std::iter::empty()))
        }
    }

    fn iter_str(&self) -> Option<Box<dyn Iterator<Item = &str> + '_>> {
        // Determine the value type of the dictionary
        let values = self.values();

        if values.as_any().is::<StringArray>() {
            Some(Box::new(DictUtf8ArrayIter {
                dict_array: self,
                values_array: values.as_any().downcast_ref::<StringArray>().unwrap(),
                index: 0,
            }))
        } else if values.as_any().is::<LargeStringArray>() {
            Some(Box::new(DictLargeUtf8ArrayIter {
                dict_array: self,
                values_array: values.as_any().downcast_ref::<LargeStringArray>().unwrap(),
                index: 0,
            }))
        } else if values.as_any().is::<StringViewArray>() {
            Some(Box::new(DictUtf8ViewArrayIter {
                dict_array: self,
                values_array: values.as_any().downcast_ref::<StringViewArray>().unwrap(),
                index: 0,
            }))
        } else {
            None
        }
    }
}

/// Iterator for DictionaryArray<Int32Type> with Utf8 values
pub struct DictUtf8ArrayIter<'a> {
    dict_array: &'a DictionaryArray<Int32Type>,
    values_array: &'a StringArray,
    index: usize,
}

impl<'a> Iterator for DictUtf8ArrayIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < arrow::array::Array::len(self.dict_array) {
            let key = self.dict_array.key(self.index)?;
            self.index += 1;
            Some(self.values_array.value(key as usize))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = arrow::array::Array::len(self.dict_array) - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for DictUtf8ArrayIter<'a> {
    fn len(&self) -> usize {
        arrow::array::Array::len(self.dict_array) - self.index
    }
}

/// Iterator for DictionaryArray<Int32Type> with LargeUtf8 values
pub struct DictLargeUtf8ArrayIter<'a> {
    dict_array: &'a DictionaryArray<Int32Type>,
    values_array: &'a LargeStringArray,
    index: usize,
}

impl<'a> Iterator for DictLargeUtf8ArrayIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < arrow::array::Array::len(self.dict_array) {
            let key = self.dict_array.key(self.index)?;
            self.index += 1;
            Some(self.values_array.value(key as usize))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = arrow::array::Array::len(self.dict_array) - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for DictLargeUtf8ArrayIter<'a> {
    fn len(&self) -> usize {
        arrow::array::Array::len(self.dict_array) - self.index
    }
}

/// Iterator for DictionaryArray<Int32Type> with Utf8View values
pub struct DictUtf8ViewArrayIter<'a> {
    dict_array: &'a DictionaryArray<Int32Type>,
    values_array: &'a StringViewArray,
    index: usize,
}

impl<'a> Iterator for DictUtf8ViewArrayIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < arrow::array::Array::len(self.dict_array) {
            let key = self.dict_array.key(self.index)?;
            self.index += 1;
            Some(self.values_array.value(key as usize))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = arrow::array::Array::len(self.dict_array) - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for DictUtf8ViewArrayIter<'a> {
    fn len(&self) -> usize {
        arrow::array::Array::len(self.dict_array) - self.index
    }
}

/// Enum to handle different dictionary value types with a unified iterator interface
pub enum DictStringArrayIter<'a> {
    Utf8(DictUtf8ArrayIter<'a>),
    LargeUtf8(DictLargeUtf8ArrayIter<'a>),
    Utf8View(DictUtf8ViewArrayIter<'a>),
}

impl<'a> Iterator for DictStringArrayIter<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            DictStringArrayIter::Utf8(iter) => iter.next(),
            DictStringArrayIter::LargeUtf8(iter) => iter.next(),
            DictStringArrayIter::Utf8View(iter) => iter.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            DictStringArrayIter::Utf8(iter) => iter.size_hint(),
            DictStringArrayIter::LargeUtf8(iter) => iter.size_hint(),
            DictStringArrayIter::Utf8View(iter) => iter.size_hint(),
        }
    }
}

impl<'a> ExactSizeIterator for DictStringArrayIter<'a> {
    fn len(&self) -> usize {
        match self {
            DictStringArrayIter::Utf8(iter) => iter.len(),
            DictStringArrayIter::LargeUtf8(iter) => iter.len(),
            DictStringArrayIter::Utf8View(iter) => iter.len(),
        }
    }
}

impl StrVector for DictionaryArray<Int32Type> {
    type Iter<'a>
        = DictStringArrayIter<'a>
    where
        Self: 'a;

    fn iter(&self) -> Self::Iter<'_> {
        let values = self.values();

        if let Some(values_array) = values.as_any().downcast_ref::<StringArray>() {
            DictStringArrayIter::Utf8(DictUtf8ArrayIter {
                dict_array: self,
                values_array,
                index: 0,
            })
        } else if let Some(values_array) = values.as_any().downcast_ref::<LargeStringArray>() {
            DictStringArrayIter::LargeUtf8(DictLargeUtf8ArrayIter {
                dict_array: self,
                values_array,
                index: 0,
            })
        } else if let Some(values_array) = values.as_any().downcast_ref::<StringViewArray>() {
            DictStringArrayIter::Utf8View(DictUtf8ViewArrayIter {
                dict_array: self,
                values_array,
                index: 0,
            })
        } else {
            // This should not happen if the array is properly constructed
            // Return an empty iterator for Utf8 as fallback
            panic!("Dictionary array has unsupported value type for string iteration")
        }
    }
}

/// Implement DataSource for RecordBatch to allow direct use with plotting functions.
/// This provides zero-copy access to Arrow arrays without converting to DataFrame first.
impl crate::data::DataSource for RecordBatch {
    fn get(&self, name: &str) -> Option<&dyn crate::data::GenericVector> {
        let schema = self.schema();
        let field_index = schema.fields().iter().position(|f| f.name() == name)?;
        let column = self.column(field_index);

        // Return a reference to the array as a GenericVector
        // We need to downcast to the concrete type to get the trait object
        let data_type = schema.field(field_index).data_type();

        match data_type {
            DataType::Boolean => column
                .as_any()
                .downcast_ref::<BooleanArray>()
                .map(|arr| arr as &dyn crate::data::GenericVector),
            DataType::Int64 => column
                .as_any()
                .downcast_ref::<Int64Array>()
                .map(|arr| arr as &dyn crate::data::GenericVector),
            DataType::Int8 => column
                .as_any()
                .downcast_ref::<Int8Array>()
                .map(|arr| arr as &dyn crate::data::GenericVector),
            DataType::Int16 => column
                .as_any()
                .downcast_ref::<Int16Array>()
                .map(|arr| arr as &dyn crate::data::GenericVector),
            DataType::Int32 => column
                .as_any()
                .downcast_ref::<Int32Array>()
                .map(|arr| arr as &dyn crate::data::GenericVector),
            DataType::UInt8 => column
                .as_any()
                .downcast_ref::<UInt8Array>()
                .map(|arr| arr as &dyn crate::data::GenericVector),
            DataType::UInt16 => column
                .as_any()
                .downcast_ref::<UInt16Array>()
                .map(|arr| arr as &dyn crate::data::GenericVector),
            DataType::UInt32 => column
                .as_any()
                .downcast_ref::<UInt32Array>()
                .map(|arr| arr as &dyn crate::data::GenericVector),
            DataType::UInt64 => column
                .as_any()
                .downcast_ref::<UInt64Array>()
                .map(|arr| arr as &dyn crate::data::GenericVector),
            DataType::Float32 => column
                .as_any()
                .downcast_ref::<Float32Array>()
                .map(|arr| arr as &dyn crate::data::GenericVector),
            DataType::Float64 => column
                .as_any()
                .downcast_ref::<Float64Array>()
                .map(|arr| arr as &dyn crate::data::GenericVector),
            DataType::Utf8 => column
                .as_any()
                .downcast_ref::<StringArray>()
                .map(|arr| arr as &dyn crate::data::GenericVector),
            DataType::LargeUtf8 => column
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .map(|arr| arr as &dyn crate::data::GenericVector),
            DataType::Utf8View => column
                .as_any()
                .downcast_ref::<StringViewArray>()
                .map(|arr| arr as &dyn crate::data::GenericVector),
            DataType::Dictionary(key_type, _value_type) => {
                if matches!(key_type.as_ref(), DataType::Int32) {
                    column
                        .as_any()
                        .downcast_ref::<DictionaryArray<Int32Type>>()
                        .map(|arr| arr as &dyn crate::data::GenericVector)
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn column_names(&self) -> Vec<String> {
        self.schema()
            .fields()
            .iter()
            .map(|f| f.name().to_string())
            .collect()
    }

    fn len(&self) -> usize {
        self.num_rows()
    }

    fn clone_box(&self) -> Box<dyn crate::data::DataSource> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataSource;
    use arrow::array::{ArrayRef, Float64Array, Int64Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use arrow::record_batch::RecordBatch;
    use std::sync::Arc;

    #[test]
    fn test_convert_int8_column() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Int8, false)]));

        let int_array = Int8Array::from(vec![1_i8, 2, 3, -4, -5]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(int_array) as ArrayRef]).unwrap();

        assert_eq!(batch.len(), 5);

        let col = batch.get("x").unwrap();
        let int_iter = col.iter_int().unwrap();
        let values: Vec<i64> = int_iter.collect();
        assert_eq!(values, vec![1, 2, 3, -4, -5]);
    }

    #[test]
    fn test_convert_int16_column() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Int16, false)]));

        let int_array = Int16Array::from(vec![100_i16, 200, 300]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(int_array) as ArrayRef]).unwrap();

        assert_eq!(batch.len(), 3);

        let col = batch.get("x").unwrap();
        let int_iter = col.iter_int().unwrap();
        let values: Vec<i64> = int_iter.collect();
        assert_eq!(values, vec![100, 200, 300]);
    }

    #[test]
    fn test_convert_int32_column() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Int32, false)]));

        let int_array = Int32Array::from(vec![1000, 2000, 3000]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(int_array) as ArrayRef]).unwrap();

        assert_eq!(batch.len(), 3);

        let col = batch.get("x").unwrap();
        let int_iter = col.iter_int().unwrap();
        let values: Vec<i64> = int_iter.collect();
        assert_eq!(values, vec![1000, 2000, 3000]);
    }

    #[test]
    fn test_convert_int64_column() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Int64, false)]));

        let int_array = Int64Array::from(vec![1, 2, 3, 4, 5]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(int_array) as ArrayRef]).unwrap();

        assert_eq!(batch.len(), 5);

        let col = batch.get("x").unwrap();
        let int_iter = col.iter_int().unwrap();
        let values: Vec<i64> = int_iter.collect();
        assert_eq!(values, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_convert_uint8_column() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::UInt8, false)]));

        let int_array = UInt8Array::from(vec![10_u8, 20, 30, 255]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(int_array) as ArrayRef]).unwrap();

        assert_eq!(batch.len(), 4);

        let col = batch.get("x").unwrap();
        let int_iter = col.iter_int().unwrap();
        let values: Vec<i64> = int_iter.collect();
        assert_eq!(values, vec![10, 20, 30, 255]);
    }

    #[test]
    fn test_convert_uint16_column() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::UInt16, false)]));

        let int_array = UInt16Array::from(vec![1000_u16, 2000, 3000]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(int_array) as ArrayRef]).unwrap();

        assert_eq!(batch.len(), 3);

        let col = batch.get("x").unwrap();
        let int_iter = col.iter_int().unwrap();
        let values: Vec<i64> = int_iter.collect();
        assert_eq!(values, vec![1000, 2000, 3000]);
    }

    #[test]
    fn test_convert_uint32_column() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::UInt32, false)]));

        let int_array = UInt32Array::from(vec![100000_u32, 200000, 300000]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(int_array) as ArrayRef]).unwrap();

        assert_eq!(batch.len(), 3);

        let col = batch.get("x").unwrap();
        let int_iter = col.iter_int().unwrap();
        let values: Vec<i64> = int_iter.collect();
        assert_eq!(values, vec![100000, 200000, 300000]);
    }

    #[test]
    fn test_convert_uint64_column() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::UInt64, false)]));

        let int_array = UInt64Array::from(vec![1000000_u64, 2000000, 3000000]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(int_array) as ArrayRef]).unwrap();

        assert_eq!(batch.len(), 3);

        let col = batch.get("x").unwrap();
        let int_iter = col.iter_int().unwrap();
        let values: Vec<i64> = int_iter.collect();
        assert_eq!(values, vec![1000000, 2000000, 3000000]);
    }

    #[test]
    fn test_convert_mixed_integer_types() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("int8", DataType::Int8, false),
            Field::new("int16", DataType::Int16, false),
            Field::new("int32", DataType::Int32, false),
            Field::new("uint8", DataType::UInt8, false),
        ]));

        let int8_array = Int8Array::from(vec![1_i8, 2, 3]);
        let int16_array = Int16Array::from(vec![100_i16, 200, 300]);
        let int32_array = Int32Array::from(vec![1000, 2000, 3000]);
        let uint8_array = UInt8Array::from(vec![10_u8, 20, 30]);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(int8_array) as ArrayRef,
                Arc::new(int16_array) as ArrayRef,
                Arc::new(int32_array) as ArrayRef,
                Arc::new(uint8_array) as ArrayRef,
            ],
        )
        .unwrap();

        assert_eq!(batch.len(), 3);
        assert_eq!(batch.column_names().len(), 4);

        // Verify all columns are present and have correct values
        let int8_col = batch.get("int8").unwrap();
        let int8_values: Vec<i64> = int8_col.iter_int().unwrap().collect();
        assert_eq!(int8_values, vec![1, 2, 3]);

        let int16_col = batch.get("int16").unwrap();
        let int16_values: Vec<i64> = int16_col.iter_int().unwrap().collect();
        assert_eq!(int16_values, vec![100, 200, 300]);

        let int32_col = batch.get("int32").unwrap();
        let int32_values: Vec<i64> = int32_col.iter_int().unwrap().collect();
        assert_eq!(int32_values, vec![1000, 2000, 3000]);

        let uint8_col = batch.get("uint8").unwrap();
        let uint8_values: Vec<i64> = uint8_col.iter_int().unwrap().collect();
        assert_eq!(uint8_values, vec![10, 20, 30]);
    }

    #[test]
    fn test_convert_float32_column() {
        let schema = Arc::new(Schema::new(vec![Field::new("y", DataType::Float32, false)]));

        let float_array = Float32Array::from(vec![1.5_f32, 2.5, 3.5]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(float_array) as ArrayRef]).unwrap();

        assert_eq!(batch.len(), 3);

        let col = batch.get("y").unwrap();
        let float_iter = col.iter_float().unwrap();
        let values: Vec<f64> = float_iter.collect();
        assert_eq!(values, vec![1.5, 2.5, 3.5]);
    }

    #[test]
    fn test_convert_float64_column() {
        let schema = Arc::new(Schema::new(vec![Field::new("y", DataType::Float64, false)]));

        let float_array = Float64Array::from(vec![1.5, 2.5, 3.5]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(float_array) as ArrayRef]).unwrap();

        assert_eq!(batch.len(), 3);

        let col = batch.get("y").unwrap();
        let float_iter = col.iter_float().unwrap();
        let values: Vec<f64> = float_iter.collect();
        assert_eq!(values, vec![1.5, 2.5, 3.5]);
    }

    #[test]
    fn test_convert_string_column() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "label",
            DataType::Utf8,
            false,
        )]));

        let string_array = StringArray::from(vec!["a", "b", "c"]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(string_array) as ArrayRef]).unwrap();

        assert_eq!(batch.len(), 3);

        let col = batch.get("label").unwrap();
        let str_iter = col.iter_str().unwrap();
        let values: Vec<String> = str_iter.map(|s| s.to_string()).collect();
        assert_eq!(values, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_convert_multiple_columns() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("x", DataType::Int64, false),
            Field::new("y", DataType::Float64, false),
            Field::new("label", DataType::Utf8, false),
        ]));

        let int_array = Int64Array::from(vec![1, 2, 3]);
        let float_array = Float64Array::from(vec![1.5, 2.5, 3.5]);
        let string_array = StringArray::from(vec!["a", "b", "c"]);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(int_array) as ArrayRef,
                Arc::new(float_array) as ArrayRef,
                Arc::new(string_array) as ArrayRef,
            ],
        )
        .unwrap();

        assert_eq!(batch.len(), 3);
        assert_eq!(batch.column_names().len(), 3);

        // Verify all columns are present
        assert!(batch.get("x").is_some());
        assert!(batch.get("y").is_some());
        assert!(batch.get("label").is_some());
    }

    #[test]
    fn test_string_array_str_vector_trait() {
        // Test that StringArray implements StrVector correctly
        use crate::data::StrVector;

        let string_array = StringArray::from(vec!["hello", "world", "test"]);

        // Test GAT iterator using explicit trait call
        let values: Vec<&str> = StrVector::iter(&string_array).collect();
        assert_eq!(values, vec!["hello", "world", "test"]);

        // Test that iterator is ExactSizeIterator
        let mut iter = StrVector::iter(&string_array);
        assert_eq!(iter.len(), 3);
        iter.next();
        assert_eq!(iter.len(), 2);

        // Test iter_str() from GenericVector
        let str_iter = string_array.iter_str().unwrap();
        let values2: Vec<&str> = str_iter.collect();
        assert_eq!(values2, vec!["hello", "world", "test"]);
    }

    #[test]
    fn test_large_string_array_str_vector_trait() {
        // Test that LargeStringArray implements StrVector correctly
        use crate::data::StrVector;

        let large_string_array = LargeStringArray::from(vec!["foo", "bar", "baz"]);

        // Test GAT iterator
        let values: Vec<&str> = StrVector::iter(&large_string_array).collect();
        assert_eq!(values, vec!["foo", "bar", "baz"]);

        // Test iter_str() from GenericVector
        let str_iter = large_string_array.iter_str().unwrap();
        let values2: Vec<&str> = str_iter.collect();
        assert_eq!(values2, vec!["foo", "bar", "baz"]);
    }

    #[test]
    fn test_string_view_array_str_vector_trait() {
        // Test that StringViewArray implements StrVector correctly
        use crate::data::StrVector;

        let string_view_array = StringViewArray::from(vec!["alpha", "beta", "gamma"]);

        // Test GAT iterator
        let values: Vec<&str> = StrVector::iter(&string_view_array).collect();
        assert_eq!(values, vec!["alpha", "beta", "gamma"]);

        // Test iter_str() from GenericVector
        let str_iter = string_view_array.iter_str().unwrap();
        let values2: Vec<&str> = str_iter.collect();
        assert_eq!(values2, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn test_convert_large_utf8_column() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "label",
            DataType::LargeUtf8,
            false,
        )]));

        let large_string_array = LargeStringArray::from(vec!["x", "y", "z"]);
        let batch =
            RecordBatch::try_new(schema, vec![Arc::new(large_string_array) as ArrayRef]).unwrap();

        assert_eq!(batch.len(), 3);

        let col = batch.get("label").unwrap();
        let str_iter = col.iter_str().unwrap();
        let values: Vec<String> = str_iter.map(|s| s.to_string()).collect();
        assert_eq!(values, vec!["x", "y", "z"]);
    }

    #[test]
    fn test_convert_utf8_view_column() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "label",
            DataType::Utf8View,
            false,
        )]));

        let string_view_array = StringViewArray::from(vec!["one", "two", "three"]);
        let batch =
            RecordBatch::try_new(schema, vec![Arc::new(string_view_array) as ArrayRef]).unwrap();

        assert_eq!(batch.len(), 3);

        let col = batch.get("label").unwrap();
        let str_iter = col.iter_str().unwrap();
        let values: Vec<String> = str_iter.map(|s| s.to_string()).collect();
        assert_eq!(values, vec!["one", "two", "three"]);
    }

    #[test]
    fn test_convert_dictionary_utf8_column() {
        use arrow::array::Int32Array;

        let schema = Arc::new(Schema::new(vec![Field::new(
            "category",
            DataType::Dictionary(Box::new(DataType::Int32), Box::new(DataType::Utf8)),
            false,
        )]));

        // Create a dictionary array with repeated values
        // Keys: [0, 1, 0, 2, 1] -> Values: ["red", "green", "red", "blue", "green"]
        let keys = Int32Array::from(vec![0, 1, 0, 2, 1]);
        let values = StringArray::from(vec!["red", "green", "blue"]);
        let dict_array = DictionaryArray::<Int32Type>::try_new(keys, Arc::new(values)).unwrap();

        let batch = RecordBatch::try_new(schema, vec![Arc::new(dict_array) as ArrayRef]).unwrap();

        assert_eq!(batch.len(), 5);

        let col = batch.get("category").unwrap();
        let str_iter = col.iter_str().unwrap();
        let values_vec: Vec<String> = str_iter.map(|s| s.to_string()).collect();
        assert_eq!(values_vec, vec!["red", "green", "red", "blue", "green"]);
    }

    #[test]
    fn test_convert_dictionary_large_utf8_column() {
        use arrow::array::Int32Array;

        let schema = Arc::new(Schema::new(vec![Field::new(
            "category",
            DataType::Dictionary(Box::new(DataType::Int32), Box::new(DataType::LargeUtf8)),
            false,
        )]));

        let keys = Int32Array::from(vec![0, 1, 2]);
        let values = LargeStringArray::from(vec!["alpha", "beta", "gamma"]);
        let dict_array = DictionaryArray::<Int32Type>::try_new(keys, Arc::new(values)).unwrap();

        let batch = RecordBatch::try_new(schema, vec![Arc::new(dict_array) as ArrayRef]).unwrap();

        assert_eq!(batch.len(), 3);

        let col = batch.get("category").unwrap();
        let str_iter = col.iter_str().unwrap();
        let values_vec: Vec<String> = str_iter.map(|s| s.to_string()).collect();
        assert_eq!(values_vec, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn test_convert_dictionary_utf8_view_column() {
        use arrow::array::Int32Array;

        let schema = Arc::new(Schema::new(vec![Field::new(
            "category",
            DataType::Dictionary(Box::new(DataType::Int32), Box::new(DataType::Utf8View)),
            false,
        )]));

        let keys = Int32Array::from(vec![1, 0, 1, 2, 0]);
        let values = StringViewArray::from(vec!["foo", "bar", "baz"]);
        let dict_array = DictionaryArray::<Int32Type>::try_new(keys, Arc::new(values)).unwrap();

        let batch = RecordBatch::try_new(schema, vec![Arc::new(dict_array) as ArrayRef]).unwrap();

        assert_eq!(batch.len(), 5);

        let col = batch.get("category").unwrap();
        let str_iter = col.iter_str().unwrap();
        let values_vec: Vec<String> = str_iter.map(|s| s.to_string()).collect();
        assert_eq!(values_vec, vec!["bar", "foo", "bar", "baz", "foo"]);
    }

    #[test]
    fn test_convert_mixed_with_dictionary() {
        use arrow::array::Int32Array;

        let schema = Arc::new(Schema::new(vec![
            Field::new("x", DataType::Int64, false),
            Field::new(
                "category",
                DataType::Dictionary(Box::new(DataType::Int32), Box::new(DataType::Utf8)),
                false,
            ),
            Field::new("y", DataType::Float64, false),
        ]));

        let int_array = Int64Array::from(vec![1, 2, 3]);
        let keys = Int32Array::from(vec![0, 1, 0]);
        let values = StringArray::from(vec!["A", "B"]);
        let dict_array = DictionaryArray::<Int32Type>::try_new(keys, Arc::new(values)).unwrap();
        let float_array = Float64Array::from(vec![1.1, 2.2, 3.3]);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(int_array) as ArrayRef,
                Arc::new(dict_array) as ArrayRef,
                Arc::new(float_array) as ArrayRef,
            ],
        )
        .unwrap();

        assert_eq!(batch.len(), 3);
        assert_eq!(batch.column_names().len(), 3);

        // Verify all columns
        assert!(batch.get("x").is_some());
        assert!(batch.get("category").is_some());
        assert!(batch.get("y").is_some());

        // Check dictionary values
        let col = batch.get("category").unwrap();
        let str_iter = col.iter_str().unwrap();
        let values_vec: Vec<String> = str_iter.map(|s| s.to_string()).collect();
        assert_eq!(values_vec, vec!["A", "B", "A"]);
    }

    #[test]
    fn test_record_batch_as_datasource() {
        // Test that RecordBatch implements DataSource directly
        let schema = Arc::new(Schema::new(vec![
            Field::new("x", DataType::Int64, false),
            Field::new("y", DataType::Float64, false),
            Field::new("label", DataType::Utf8, false),
        ]));

        let int_array = Int64Array::from(vec![1, 2, 3, 4]);
        let float_array = Float64Array::from(vec![1.5, 2.5, 3.5, 4.5]);
        let string_array = StringArray::from(vec!["a", "b", "c", "d"]);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(int_array) as ArrayRef,
                Arc::new(float_array) as ArrayRef,
                Arc::new(string_array) as ArrayRef,
            ],
        )
        .unwrap();

        // Test DataSource methods
        assert_eq!(batch.len(), 4);
        assert!(!batch.is_empty());
        assert_eq!(batch.column_names().len(), 3);
        assert!(batch.column_names().contains(&"x".to_string()));
        assert!(batch.column_names().contains(&"y".to_string()));
        assert!(batch.column_names().contains(&"label".to_string()));

        // Test getting integer column
        let x_col = batch.get("x").unwrap();
        assert_eq!(x_col.len(), 4);
        let x_values: Vec<i64> = x_col.iter_int().unwrap().collect();
        assert_eq!(x_values, vec![1, 2, 3, 4]);

        // Test getting float column
        let y_col = batch.get("y").unwrap();
        assert_eq!(y_col.len(), 4);
        let y_values: Vec<f64> = y_col.iter_float().unwrap().collect();
        assert_eq!(y_values, vec![1.5, 2.5, 3.5, 4.5]);

        // Test getting string column
        let label_col = batch.get("label").unwrap();
        assert_eq!(label_col.len(), 4);
        let label_values: Vec<String> = label_col
            .iter_str()
            .unwrap()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(label_values, vec!["a", "b", "c", "d"]);

        // Test getting non-existent column
        assert!(batch.get("nonexistent").is_none());
    }

    #[test]
    fn test_convert_boolean_column() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "flag",
            DataType::Boolean,
            false,
        )]));

        let bool_array = BooleanArray::from(vec![true, false, true, false, true]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(bool_array) as ArrayRef]).unwrap();

        assert_eq!(batch.len(), 5);

        let col = batch.get("flag").unwrap();
        let bool_iter = col.iter_bool().unwrap();
        let values: Vec<bool> = bool_iter.collect();
        assert_eq!(values, vec![true, false, true, false, true]);
    }

    #[test]
    fn test_boolean_vector_iter() {
        let bool_array = BooleanArray::from(vec![true, false, true]);
        
        match GenericVector::iter(&bool_array) {
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
    fn test_mixed_with_boolean() {
        let schema = Arc::new(Schema::new(vec![
            Field::new("x", DataType::Int64, false),
            Field::new("active", DataType::Boolean, false),
            Field::new("label", DataType::Utf8, false),
        ]));

        let int_array = Int64Array::from(vec![1, 2, 3]);
        let bool_array = BooleanArray::from(vec![true, false, true]);
        let string_array = StringArray::from(vec!["a", "b", "c"]);

        let batch = RecordBatch::try_new(
            schema,
            vec![
                Arc::new(int_array) as ArrayRef,
                Arc::new(bool_array) as ArrayRef,
                Arc::new(string_array) as ArrayRef,
            ],
        )
        .unwrap();

        assert_eq!(batch.len(), 3);
        assert_eq!(batch.column_names().len(), 3);

        // Verify all columns are present
        assert!(batch.get("x").is_some());
        assert!(batch.get("active").is_some());
        assert!(batch.get("label").is_some());

        // Check boolean values
        let bool_col = batch.get("active").unwrap();
        let bool_values: Vec<bool> = bool_col.iter_bool().unwrap().collect();
        assert_eq!(bool_values, vec![true, false, true]);
    }
}
