// Example: Analyzing mtcars dataset with Polars
// Demonstrates loading CSV data with Polars and creating various plots
//
// To run this example, use:
//   cargo run --example mtcars_polars --features polars

#[cfg(feature = "polars")]
mod example {
    use gogplot::layer::Stat;
    use gogplot::plot::{GeomBuilder, Plot};
    use polars::prelude::*;
    use std::path::Path;

    pub fn main() -> Result<(), Box<dyn std::error::Error>> {
        // Load the mtcars dataset using Polars
        let csv_path = Path::new("examples/mtcars.csv");
        let df = CsvReadOptions::default()
            .with_has_header(true)
            .try_into_reader_with_file_path(Some(csv_path.into()))?
            .finish()?;

        println!("Loaded mtcars dataset with {} rows", df.height());
        println!("Columns: {:?}", df.get_column_names());

        // Plot 1: MPG vs Weight - scatter plot
        println!("\nCreating scatter plot: MPG vs Weight...");
        let plot1 = Plot::new(Some(Box::new(df.clone())))
            .aes(|a| {
                a.x("wt");
                a.y("mpg");
            })
            .geom_point_with(|layer| { layer.geom.size(5.0); })
            .title("Fuel Efficiency vs Weight");

        plot1.save("mtcars_mpg_vs_weight.png", 800, 600)?;
        println!("Created mtcars_mpg_vs_weight.png");

        // Plot 2: MPG vs Weight colored by cylinder count
        println!("\nCreating colored scatter plot: MPG vs Weight by Cylinders...");
        
        let plot2 = Plot::new(Some(Box::new(df.clone())))
            .aes(|a| {
                a.x("wt");
                a.y("mpg");
                a.color_categorical("cyl"); // Treat numeric cyl as categorical
            })
            .geom_point_with(|layer| { layer.geom.size(6.0); })
            .title("Fuel Efficiency vs Weight by Cylinder Count");

        plot2.save("mtcars_mpg_vs_weight_by_cyl.png", 800, 600)?;
        println!("Created mtcars_mpg_vs_weight_by_cyl.png");

        // Plot 3: Horsepower vs Displacement
        println!("\nCreating scatter plot: Horsepower vs Displacement...");
        let plot3 = Plot::new(Some(Box::new(df.clone())))
            .aes(|a| {
                a.x("disp");
                a.y("hp");
            })
            .geom_point_with(|layer| { layer.geom.size(5.0); })
            .title("Horsepower vs Engine Displacement");

        plot3.save("mtcars_hp_vs_disp.png", 800, 600)?;
        println!("Created mtcars_hp_vs_disp.png");

        // Plot 4: Bar chart of average MPG by cylinder count
        println!("\nCreating bar chart: Average MPG by Cylinder Count...");
        
        // Calculate average MPG by cylinder count
        let mpg_by_cyl = df
            .clone()
            .lazy()
            .group_by([col("cyl")])
            .agg([col("mpg").mean().alias("avg_mpg")])
            .sort(["cyl"], Default::default())
            .collect()?;

        let plot4 = Plot::new(Some(Box::new(mpg_by_cyl)))
            .aes(|a| {
                a.x_categorical("cyl"); // Treat numeric cyl as categorical for x-axis
                a.y("avg_mpg");
            })
            .geom_bar_with(|layer| { layer.geom.stat(Stat::Identity); })
            .title("Average Fuel Efficiency by Cylinder Count");

        plot4.save("mtcars_mpg_by_cyl.png", 800, 600)?;
        println!("Created mtcars_mpg_by_cyl.png");

        // Plot 5: MPG distribution by transmission type (am: 0=automatic, 1=manual)
        println!("\nCreating scatter plot: MPG by Transmission Type...");
        
        // Create a new dataframe with transmission as a string label
        let df_with_trans = df
            .clone()
            .lazy()
            .with_column(
                when(col("am").eq(lit(0)))
                    .then(lit("Automatic"))
                    .otherwise(lit("Manual"))
                    .alias("transmission")
            )
            .collect()?;
        
        let plot5 = Plot::new(Some(Box::new(df_with_trans)))
            .aes(|a| {
                a.x("transmission");
                a.y("mpg");
                a.color("transmission");
            })
            .geom_point_with(|layer| { layer.geom.size(6.0); })
            .title("Fuel Efficiency by Transmission Type");

        plot5.save("mtcars_mpg_by_transmission.png", 800, 600)?;
        println!("Created mtcars_mpg_by_transmission.png");

        println!("\n✓ All plots created successfully!");
        println!("\nGenerated plots:");
        println!("  - mtcars_mpg_vs_weight.png");
        println!("  - mtcars_mpg_vs_weight_by_cyl.png");
        println!("  - mtcars_hp_vs_disp.png");
        println!("  - mtcars_mpg_by_cyl.png");
        println!("  - mtcars_mpg_by_transmission.png");

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
    println!("Run with: cargo run --example mtcars_polars --features polars");
}
