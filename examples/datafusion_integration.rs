// Example: DataFusion integration
// Demonstrates converting DataFusion RecordBatch to gogplot DataFrame
//
// To run this example, use:
//   cargo run --example datafusion_integration --features datafusion

#[cfg(feature = "datafusion")]
use datafusion::arrow::array::{Float64Array, StringArray};
#[cfg(feature = "datafusion")]
use datafusion::arrow::datatypes::{DataType, Field, Schema};
#[cfg(feature = "datafusion")]
use datafusion::arrow::record_batch::RecordBatch;
#[cfg(feature = "datafusion")]
use gogplot::plot::{GeomBuilder, Plot};
#[cfg(feature = "datafusion")]
use gogplot::utils::datafusion::record_batch_to_dataframe;
#[cfg(feature = "datafusion")]
use std::sync::Arc;

#[cfg(feature = "datafusion")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a sample Arrow RecordBatch
    // This could come from reading a CSV, Parquet file, or SQL query
    let schema = Arc::new(Schema::new(vec![
        Field::new("x", DataType::Float64, false),
        Field::new("y", DataType::Float64, false),
        Field::new("category", DataType::Utf8, false),
    ]));

    let x_array = Float64Array::from(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0]);
    let y_array = Float64Array::from(vec![2.5, 3.8, 3.2, 4.5, 5.2, 4.8, 6.1, 5.9]);
    let cat_array = StringArray::from(vec!["A", "B", "A", "B", "A", "B", "A", "B"]);

    let batch = RecordBatch::try_new(
        schema,
        vec![
            Arc::new(x_array),
            Arc::new(y_array),
            Arc::new(cat_array),
        ],
    )?;

    // Convert the RecordBatch to a gogplot DataFrame
    let df = record_batch_to_dataframe(&batch)?;

    // Create a plot using the converted data
    let plot = Plot::new(Some(Box::new(df)))
        .aes(|a| {
            a.x("x");
            a.y("y");
            a.color("category");
        })
        .geom_point_with(|geom| geom.size(6.0))
        .title("DataFusion Integration Example");

    plot.save("datafusion_example.png", 800, 600)?;
    println!("Created datafusion_example.png");

    Ok(())
}

#[cfg(not(feature = "datafusion"))]
fn main() {
    println!("This example requires the 'datafusion' feature.");
    println!("Run with: cargo run --example datafusion_integration --features datafusion");
}
