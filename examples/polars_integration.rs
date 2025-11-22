// Example: Polars integration
// Demonstrates using Polars DataFrame directly as a DataSource
//
// To run this example, use:
//   cargo run --example polars_integration --features polars

#[cfg(feature = "polars")]
mod example {
    use gogplot::plot::{GeomBuilder, Plot};
    use polars::prelude::*;

    pub fn main() -> Result<(), Box<dyn std::error::Error>> {
        // Create a sample Polars DataFrame
        let df = df! {
            "x" => &[1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0],
            "y" => &[2.5, 3.8, 3.2, 4.5, 5.2, 4.8, 6.1, 5.9],
            "category" => &["A", "B", "A", "B", "A", "B", "A", "B"]
        }?;

        // Use the DataFrame directly as a DataSource - no conversion needed!
        // Polars DataFrame implements DataSource, providing direct access to columns
        let plot = Plot::new(Some(Box::new(df)))
            .aes(|a| {
                a.x("x");
                a.y("y");
                a.color("category");
            })
            .geom_point_with(|layer| { layer.geom.size(6.0); })
            .title("Polars Integration Example");

        plot.save("polars_example.png", 800, 600)?;
        println!("Created polars_example.png");

        Ok(())
    }
}

#[cfg(feature = "polars")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    example::main()
}

#[cfg(not(feature = "polars"))]
fn main() {
    println!("This example requires the 'polars' feature.");
    println!("Run with: cargo run --example polars_integration --features polars");
}
