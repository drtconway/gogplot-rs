use crate::data::{FloatVector, GenericVector, IntVector, StrVector};
use crate::utils::dataframe::{DataFrame, FloatVec, IntVec, StrVec};
use arrow::array::{
    Array, ArrayRef, DictionaryArray, Float32Array, Float64Array, Int16Array,
    Int32Array, Int64Array, Int8Array, LargeStringArray, StringArray, StringViewArray,
    UInt16Array, UInt32Array, UInt64Array, UInt8Array,
};
use arrow::datatypes::{DataType, Int32Type};
use datafusion::arrow::record_batch::RecordBatch;

impl GenericVector for Int64Array {
    fn len(&self) -> usize {
        arrow::array::Array::len(self)
    }

    fn vtype(&self) -> crate::data::VectorType {
        crate::data::VectorType::Int
    }

    fn iter_int(&self) -> Option<Box<dyn Iterator<Item = &i64> + '_>> {
        Some(Box::new(self.values().iter()))
    }
}

impl IntVector for Int64Array {
    type Iter<'a> = std::slice::Iter<'a, i64> where Self: 'a;
    
    fn iter(&self) -> Self::Iter<'_> {
        // Arrow Int64Array stores data in a buffer. We can get a slice of the values
        // if there are no nulls. For now, we'll use values() which gives us a slice.
        // Note: This will include values at null positions if any exist.
        // The caller should check for nulls separately if needed.
        self.values().iter()
    }
}

impl GenericVector for Float64Array {
    fn len(&self) -> usize {
        arrow::array::Array::len(self)
    }

    fn vtype(&self) -> crate::data::VectorType {
        crate::data::VectorType::Float
    }

    fn iter_float(&self) -> Option<Box<dyn Iterator<Item = &f64> + '_>> {
        Some(Box::new(self.values().iter()))
    }
}

impl FloatVector for Float64Array {
    type Iter<'a> = std::slice::Iter<'a, f64> where Self: 'a;
    
    fn iter(&self) -> Self::Iter<'_> {
        self.values().iter()
    }
}

impl GenericVector for StringArray {
    fn len(&self) -> usize {
        arrow::array::Array::len(self)
    }

    fn vtype(&self) -> crate::data::VectorType {
        crate::data::VectorType::Str
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
    type Iter<'a> = StringArrayIter<'a> where Self: 'a;
    
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
    type Iter<'a> = LargeStringArrayIter<'a> where Self: 'a;
    
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
    type Iter<'a> = StringViewArrayIter<'a> where Self: 'a;
    
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
    type Iter<'a> = DictStringArrayIter<'a> where Self: 'a;
    
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

/// Convert a DataFusion RecordBatch into a gogplot DataFrame.
///
/// This function supports the following Arrow data types:
/// - Int8, Int16, Int32, Int64 -> IntVec
/// - UInt8, UInt16, UInt32, UInt64 -> IntVec (cast to i64)
/// - Float32, Float64 -> FloatVec (cast to f64)
/// - Utf8 (String) -> StrVec
/// - LargeUtf8 (LargeString) -> StrVec
/// - Utf8View (StringView) -> StrVec
/// - Dictionary(Int32, Utf8/LargeUtf8/Utf8View) -> StrVec (dictionary-encoded strings)
///
/// # Errors
///
/// Returns an error if:
/// - The RecordBatch contains unsupported column types
/// - Column data cannot be downcast to the expected array type
///
/// # Examples
///
/// ```ignore
/// use datafusion::prelude::*;
/// use gogplot::utils::datafusion::record_batch_to_dataframe;
///
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let ctx = SessionContext::new();
/// let df = ctx.read_csv("data.csv", CsvReadOptions::new()).await?;
/// let batches = df.collect().await?;
/// 
/// for batch in batches {
///     let gg_df = record_batch_to_dataframe(&batch)?;
///     // Use gg_df with gogplot...
/// }
/// # Ok(())
/// # }
/// ```
pub fn record_batch_to_dataframe(
    batch: &RecordBatch,
) -> Result<DataFrame, Box<dyn std::error::Error>> {
    let mut df = DataFrame::new();

    for (field, column) in batch.schema().fields().iter().zip(batch.columns()) {
        let name = field.name();
        let col_data = convert_array(column, field.data_type())?;
        df.add_column(name, col_data);
    }

    Ok(df)
}

fn convert_array(
    array: &ArrayRef,
    data_type: &DataType,
) -> Result<Box<dyn crate::data::GenericVector>, Box<dyn std::error::Error>> {
    match data_type {
        DataType::Int8 => {
            let arr = array
                .as_any()
                .downcast_ref::<Int8Array>()
                .ok_or("Failed to downcast to Int8Array")?;

            let mut values = Vec::with_capacity(arr.len());
            for i in 0..arr.len() {
                if arr.is_null(i) {
                    return Err("Null values not supported in Int8 columns".into());
                }
                values.push(arr.value(i) as i64);
            }
            Ok(Box::new(IntVec(values)))
        }
        DataType::Int16 => {
            let arr = array
                .as_any()
                .downcast_ref::<Int16Array>()
                .ok_or("Failed to downcast to Int16Array")?;

            let mut values = Vec::with_capacity(arr.len());
            for i in 0..arr.len() {
                if arr.is_null(i) {
                    return Err("Null values not supported in Int16 columns".into());
                }
                values.push(arr.value(i) as i64);
            }
            Ok(Box::new(IntVec(values)))
        }
        DataType::Int32 => {
            let arr = array
                .as_any()
                .downcast_ref::<Int32Array>()
                .ok_or("Failed to downcast to Int32Array")?;

            let mut values = Vec::with_capacity(arr.len());
            for i in 0..arr.len() {
                if arr.is_null(i) {
                    return Err("Null values not supported in Int32 columns".into());
                }
                values.push(arr.value(i) as i64);
            }
            Ok(Box::new(IntVec(values)))
        }
        DataType::Int64 => {
            let arr = array
                .as_any()
                .downcast_ref::<Int64Array>()
                .ok_or("Failed to downcast to Int64Array")?;

            let mut values = Vec::with_capacity(arr.len());
            for i in 0..arr.len() {
                if arr.is_null(i) {
                    return Err("Null values not supported in Int64 columns".into());
                }
                values.push(arr.value(i));
            }
            Ok(Box::new(IntVec(values)))
        }
        DataType::UInt8 => {
            let arr = array
                .as_any()
                .downcast_ref::<UInt8Array>()
                .ok_or("Failed to downcast to UInt8Array")?;

            let mut values = Vec::with_capacity(arr.len());
            for i in 0..arr.len() {
                if arr.is_null(i) {
                    return Err("Null values not supported in UInt8 columns".into());
                }
                values.push(arr.value(i) as i64);
            }
            Ok(Box::new(IntVec(values)))
        }
        DataType::UInt16 => {
            let arr = array
                .as_any()
                .downcast_ref::<UInt16Array>()
                .ok_or("Failed to downcast to UInt16Array")?;

            let mut values = Vec::with_capacity(arr.len());
            for i in 0..arr.len() {
                if arr.is_null(i) {
                    return Err("Null values not supported in UInt16 columns".into());
                }
                values.push(arr.value(i) as i64);
            }
            Ok(Box::new(IntVec(values)))
        }
        DataType::UInt32 => {
            let arr = array
                .as_any()
                .downcast_ref::<UInt32Array>()
                .ok_or("Failed to downcast to UInt32Array")?;

            let mut values = Vec::with_capacity(arr.len());
            for i in 0..arr.len() {
                if arr.is_null(i) {
                    return Err("Null values not supported in UInt32 columns".into());
                }
                values.push(arr.value(i) as i64);
            }
            Ok(Box::new(IntVec(values)))
        }
        DataType::UInt64 => {
            let arr = array
                .as_any()
                .downcast_ref::<UInt64Array>()
                .ok_or("Failed to downcast to UInt64Array")?;

            let mut values = Vec::with_capacity(arr.len());
            for i in 0..arr.len() {
                if arr.is_null(i) {
                    return Err("Null values not supported in UInt64 columns".into());
                }
                // Note: UInt64 values > i64::MAX will wrap when cast to i64
                values.push(arr.value(i) as i64);
            }
            Ok(Box::new(IntVec(values)))
        }
        DataType::Float32 => {
            let arr = array
                .as_any()
                .downcast_ref::<Float32Array>()
                .ok_or("Failed to downcast to Float32Array")?;

            let mut values = Vec::with_capacity(arr.len());
            for i in 0..arr.len() {
                if arr.is_null(i) {
                    return Err("Null values not supported in Float32 columns".into());
                }
                values.push(arr.value(i) as f64);
            }
            Ok(Box::new(FloatVec(values)))
        }
        DataType::Float64 => {
            let arr = array
                .as_any()
                .downcast_ref::<Float64Array>()
                .ok_or("Failed to downcast to Float64Array")?;

            let mut values = Vec::with_capacity(arr.len());
            for i in 0..arr.len() {
                if arr.is_null(i) {
                    return Err("Null values not supported in Float64 columns".into());
                }
                values.push(arr.value(i));
            }
            Ok(Box::new(FloatVec(values)))
        }
        DataType::Utf8 => {
            let arr = array
                .as_any()
                .downcast_ref::<StringArray>()
                .ok_or("Failed to downcast to StringArray")?;

            let len = arrow::array::Array::len(arr);
            let mut values = Vec::with_capacity(len);
            for i in 0..len {
                if arr.is_null(i) {
                    return Err("Null values not supported in String columns".into());
                }
                values.push(arr.value(i).to_string());
            }
            Ok(Box::new(StrVec(values)))
        }
        DataType::LargeUtf8 => {
            let arr = array
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .ok_or("Failed to downcast to LargeStringArray")?;

            let len = arrow::array::Array::len(arr);
            let mut values = Vec::with_capacity(len);
            for i in 0..len {
                if arr.is_null(i) {
                    return Err("Null values not supported in LargeString columns".into());
                }
                values.push(arr.value(i).to_string());
            }
            Ok(Box::new(StrVec(values)))
        }
        DataType::Utf8View => {
            let arr = array
                .as_any()
                .downcast_ref::<StringViewArray>()
                .ok_or("Failed to downcast to StringViewArray")?;

            let len = arrow::array::Array::len(arr);
            let mut values = Vec::with_capacity(len);
            for i in 0..len {
                if arr.is_null(i) {
                    return Err("Null values not supported in StringView columns".into());
                }
                values.push(arr.value(i).to_string());
            }
            Ok(Box::new(StrVec(values)))
        }
        DataType::Dictionary(key_type, value_type) => {
            // Handle dictionary-encoded columns
            // Most common case is Int32 keys with String values
            match (key_type.as_ref(), value_type.as_ref()) {
                (DataType::Int32, DataType::Utf8) => {
                    let dict_array = array
                        .as_any()
                        .downcast_ref::<DictionaryArray<Int32Type>>()
                        .ok_or("Failed to downcast to DictionaryArray<Int32>")?;

                    let values_array = dict_array
                        .values()
                        .as_any()
                        .downcast_ref::<StringArray>()
                        .ok_or("Failed to downcast dictionary values to StringArray")?;

                    let len = arrow::array::Array::len(dict_array);
                    let mut values = Vec::with_capacity(len);
                    for i in 0..len {
                        if dict_array.is_null(i) {
                            return Err("Null values not supported in Dictionary columns".into());
                        }
                        let key = dict_array.key(i).ok_or("Failed to get dictionary key")?;
                        let value = values_array.value(key as usize);
                        values.push(value.to_string());
                    }
                    Ok(Box::new(StrVec(values)))
                }
                (DataType::Int32, DataType::LargeUtf8) => {
                    let dict_array = array
                        .as_any()
                        .downcast_ref::<DictionaryArray<Int32Type>>()
                        .ok_or("Failed to downcast to DictionaryArray<Int32>")?;

                    let values_array = dict_array
                        .values()
                        .as_any()
                        .downcast_ref::<LargeStringArray>()
                        .ok_or("Failed to downcast dictionary values to LargeStringArray")?;

                    let len = arrow::array::Array::len(dict_array);
                    let mut values = Vec::with_capacity(len);
                    for i in 0..len {
                        if dict_array.is_null(i) {
                            return Err("Null values not supported in Dictionary columns".into());
                        }
                        let key = dict_array.key(i).ok_or("Failed to get dictionary key")?;
                        let value = values_array.value(key as usize);
                        values.push(value.to_string());
                    }
                    Ok(Box::new(StrVec(values)))
                }
                (DataType::Int32, DataType::Utf8View) => {
                    let dict_array = array
                        .as_any()
                        .downcast_ref::<DictionaryArray<Int32Type>>()
                        .ok_or("Failed to downcast to DictionaryArray<Int32>")?;

                    let values_array = dict_array
                        .values()
                        .as_any()
                        .downcast_ref::<StringViewArray>()
                        .ok_or("Failed to downcast dictionary values to StringViewArray")?;

                    let len = arrow::array::Array::len(dict_array);
                    let mut values = Vec::with_capacity(len);
                    for i in 0..len {
                        if dict_array.is_null(i) {
                            return Err("Null values not supported in Dictionary columns".into());
                        }
                        let key = dict_array.key(i).ok_or("Failed to get dictionary key")?;
                        let value = values_array.value(key as usize);
                        values.push(value.to_string());
                    }
                    Ok(Box::new(StrVec(values)))
                }
                _ => Err(format!(
                    "Unsupported dictionary type: key={:?}, value={:?}",
                    key_type, value_type
                )
                .into()),
            }
        }
        other => Err(format!("Unsupported data type: {:?}", other).into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::DataSource;
    use arrow::array::{Float64Array, Int64Array, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use datafusion::arrow::record_batch::RecordBatch;
    use std::sync::Arc;

    #[test]
    fn test_convert_int8_column() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Int8, false)]));

        let int_array = Int8Array::from(vec![1_i8, 2, 3, -4, -5]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(int_array) as ArrayRef]).unwrap();

        let df = record_batch_to_dataframe(&batch).unwrap();
        assert_eq!(df.len(), 5);

        let col = df.get("x").unwrap();
        let int_iter = col.iter_int().unwrap();
        let values: Vec<i64> = int_iter.copied().collect();
        assert_eq!(values, vec![1, 2, 3, -4, -5]);
    }

    #[test]
    fn test_convert_int16_column() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Int16, false)]));

        let int_array = Int16Array::from(vec![100_i16, 200, 300]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(int_array) as ArrayRef]).unwrap();

        let df = record_batch_to_dataframe(&batch).unwrap();
        assert_eq!(df.len(), 3);

        let col = df.get("x").unwrap();
        let int_iter = col.iter_int().unwrap();
        let values: Vec<i64> = int_iter.copied().collect();
        assert_eq!(values, vec![100, 200, 300]);
    }

    #[test]
    fn test_convert_int32_column() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Int32, false)]));

        let int_array = Int32Array::from(vec![1000, 2000, 3000]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(int_array) as ArrayRef]).unwrap();

        let df = record_batch_to_dataframe(&batch).unwrap();
        assert_eq!(df.len(), 3);

        let col = df.get("x").unwrap();
        let int_iter = col.iter_int().unwrap();
        let values: Vec<i64> = int_iter.copied().collect();
        assert_eq!(values, vec![1000, 2000, 3000]);
    }

    #[test]
    fn test_convert_int64_column() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::Int64, false)]));

        let int_array = Int64Array::from(vec![1, 2, 3, 4, 5]);
        let batch =
            RecordBatch::try_new(schema, vec![Arc::new(int_array) as ArrayRef]).unwrap();

        let df = record_batch_to_dataframe(&batch).unwrap();
        assert_eq!(df.len(), 5);

        let col = df.get("x").unwrap();
        let int_iter = col.iter_int().unwrap();
        let values: Vec<i64> = int_iter.copied().collect();
        assert_eq!(values, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_convert_uint8_column() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::UInt8, false)]));

        let int_array = UInt8Array::from(vec![10_u8, 20, 30, 255]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(int_array) as ArrayRef]).unwrap();

        let df = record_batch_to_dataframe(&batch).unwrap();
        assert_eq!(df.len(), 4);

        let col = df.get("x").unwrap();
        let int_iter = col.iter_int().unwrap();
        let values: Vec<i64> = int_iter.copied().collect();
        assert_eq!(values, vec![10, 20, 30, 255]);
    }

    #[test]
    fn test_convert_uint16_column() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::UInt16, false)]));

        let int_array = UInt16Array::from(vec![1000_u16, 2000, 3000]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(int_array) as ArrayRef]).unwrap();

        let df = record_batch_to_dataframe(&batch).unwrap();
        assert_eq!(df.len(), 3);

        let col = df.get("x").unwrap();
        let int_iter = col.iter_int().unwrap();
        let values: Vec<i64> = int_iter.copied().collect();
        assert_eq!(values, vec![1000, 2000, 3000]);
    }

    #[test]
    fn test_convert_uint32_column() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::UInt32, false)]));

        let int_array = UInt32Array::from(vec![100000_u32, 200000, 300000]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(int_array) as ArrayRef]).unwrap();

        let df = record_batch_to_dataframe(&batch).unwrap();
        assert_eq!(df.len(), 3);

        let col = df.get("x").unwrap();
        let int_iter = col.iter_int().unwrap();
        let values: Vec<i64> = int_iter.copied().collect();
        assert_eq!(values, vec![100000, 200000, 300000]);
    }

    #[test]
    fn test_convert_uint64_column() {
        let schema = Arc::new(Schema::new(vec![Field::new("x", DataType::UInt64, false)]));

        let int_array = UInt64Array::from(vec![1000000_u64, 2000000, 3000000]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(int_array) as ArrayRef]).unwrap();

        let df = record_batch_to_dataframe(&batch).unwrap();
        assert_eq!(df.len(), 3);

        let col = df.get("x").unwrap();
        let int_iter = col.iter_int().unwrap();
        let values: Vec<i64> = int_iter.copied().collect();
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

        let df = record_batch_to_dataframe(&batch).unwrap();
        assert_eq!(df.len(), 3);
        assert_eq!(df.column_names().len(), 4);

        // Verify all columns are present and have correct values
        let int8_col = df.get("int8").unwrap();
        let int8_values: Vec<i64> = int8_col.iter_int().unwrap().copied().collect();
        assert_eq!(int8_values, vec![1, 2, 3]);

        let int16_col = df.get("int16").unwrap();
        let int16_values: Vec<i64> = int16_col.iter_int().unwrap().copied().collect();
        assert_eq!(int16_values, vec![100, 200, 300]);

        let int32_col = df.get("int32").unwrap();
        let int32_values: Vec<i64> = int32_col.iter_int().unwrap().copied().collect();
        assert_eq!(int32_values, vec![1000, 2000, 3000]);

        let uint8_col = df.get("uint8").unwrap();
        let uint8_values: Vec<i64> = uint8_col.iter_int().unwrap().copied().collect();
        assert_eq!(uint8_values, vec![10, 20, 30]);
    }

    #[test]
    fn test_convert_float32_column() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "y",
            DataType::Float32,
            false,
        )]));

        let float_array = Float32Array::from(vec![1.5_f32, 2.5, 3.5]);
        let batch = RecordBatch::try_new(schema, vec![Arc::new(float_array) as ArrayRef]).unwrap();

        let df = record_batch_to_dataframe(&batch).unwrap();
        assert_eq!(df.len(), 3);

        let col = df.get("y").unwrap();
        let float_iter = col.iter_float().unwrap();
        let values: Vec<f64> = float_iter.copied().collect();
        assert_eq!(values, vec![1.5, 2.5, 3.5]);
    }

    #[test]
    fn test_convert_float64_column() {
        let schema = Arc::new(Schema::new(vec![Field::new(
            "y",
            DataType::Float64,
            false,
        )]));

        let float_array = Float64Array::from(vec![1.5, 2.5, 3.5]);
        let batch =
            RecordBatch::try_new(schema, vec![Arc::new(float_array) as ArrayRef]).unwrap();

        let df = record_batch_to_dataframe(&batch).unwrap();
        assert_eq!(df.len(), 3);

        let col = df.get("y").unwrap();
        let float_iter = col.iter_float().unwrap();
        let values: Vec<f64> = float_iter.copied().collect();
        assert_eq!(values, vec![1.5, 2.5, 3.5]);
    }

    #[test]
    fn test_convert_string_column() {
        let schema = Arc::new(Schema::new(vec![Field::new("label", DataType::Utf8, false)]));

        let string_array = StringArray::from(vec!["a", "b", "c"]);
        let batch =
            RecordBatch::try_new(schema, vec![Arc::new(string_array) as ArrayRef]).unwrap();

        let df = record_batch_to_dataframe(&batch).unwrap();
        assert_eq!(df.len(), 3);

        let col = df.get("label").unwrap();
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

        let df = record_batch_to_dataframe(&batch).unwrap();
        assert_eq!(df.len(), 3);
        assert_eq!(df.column_names().len(), 3);

        // Verify all columns are present
        assert!(df.get("x").is_some());
        assert!(df.get("y").is_some());
        assert!(df.get("label").is_some());
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
        let batch = RecordBatch::try_new(schema, vec![Arc::new(large_string_array) as ArrayRef])
            .unwrap();

        let df = record_batch_to_dataframe(&batch).unwrap();
        assert_eq!(df.len(), 3);

        let col = df.get("label").unwrap();
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
        let batch = RecordBatch::try_new(schema, vec![Arc::new(string_view_array) as ArrayRef])
            .unwrap();

        let df = record_batch_to_dataframe(&batch).unwrap();
        assert_eq!(df.len(), 3);

        let col = df.get("label").unwrap();
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

        let df = record_batch_to_dataframe(&batch).unwrap();
        assert_eq!(df.len(), 5);

        let col = df.get("category").unwrap();
        let str_iter = col.iter_str().unwrap();
        let values_vec: Vec<String> = str_iter.map(|s| s.to_string()).collect();
        assert_eq!(
            values_vec,
            vec!["red", "green", "red", "blue", "green"]
        );
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

        let df = record_batch_to_dataframe(&batch).unwrap();
        assert_eq!(df.len(), 3);

        let col = df.get("category").unwrap();
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

        let df = record_batch_to_dataframe(&batch).unwrap();
        assert_eq!(df.len(), 5);

        let col = df.get("category").unwrap();
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

        let df = record_batch_to_dataframe(&batch).unwrap();
        assert_eq!(df.len(), 3);
        assert_eq!(df.column_names().len(), 3);

        // Verify all columns
        assert!(df.get("x").is_some());
        assert!(df.get("category").is_some());
        assert!(df.get("y").is_some());

        // Check dictionary values
        let col = df.get("category").unwrap();
        let str_iter = col.iter_str().unwrap();
        let values_vec: Vec<String> = str_iter.map(|s| s.to_string()).collect();
        assert_eq!(values_vec, vec!["A", "B", "A"]);
    }
}
