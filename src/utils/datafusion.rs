use crate::utils::dataframe::{DataFrame, FloatVec, IntVec, StrVec};
use arrow::array::{Array, ArrayRef, Float64Array, Int64Array, StringArray};
use arrow::datatypes::DataType;
use datafusion::arrow::record_batch::RecordBatch;

/// Convert a DataFusion RecordBatch into a gogplot DataFrame.
///
/// This function supports the following Arrow data types:
/// - Int64 -> IntVec
/// - Float64 -> FloatVec  
/// - Utf8 (String) -> StrVec
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

            let mut values = Vec::with_capacity(arr.len());
            for i in 0..arr.len() {
                if arr.is_null(i) {
                    return Err("Null values not supported in String columns".into());
                }
                values.push(arr.value(i).to_string());
            }
            Ok(Box::new(StrVec(values)))
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
        let int_vec = col.as_int().unwrap();
        let values: Vec<i64> = int_vec.iter().copied().collect();
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
        let float_vec = col.as_float().unwrap();
        let values: Vec<f64> = float_vec.iter().copied().collect();
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
        let str_vec = col.as_str().unwrap();
        let values: Vec<String> = str_vec.iter().cloned().collect();
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
}
