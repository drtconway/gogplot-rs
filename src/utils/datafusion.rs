use crate::data::{FloatVector, GenericVector, IntVector, StrVector};
use crate::utils::dataframe::{DataFrame, FloatVec, IntVec, StrVec};
use arrow::array::{
    Array, ArrayRef, DictionaryArray, Float64Array, Int64Array, LargeStringArray, StringArray,
    StringViewArray,
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

/// Convert a DataFusion RecordBatch into a gogplot DataFrame.
///
/// This function supports the following Arrow data types:
/// - Int64 -> IntVec
/// - Float64 -> FloatVec  
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
