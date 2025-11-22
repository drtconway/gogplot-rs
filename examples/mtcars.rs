// Example: Using the built-in mtcars dataset
//
// To run this example, use:
//   cargo run --example mtcars

use gogplot::plot::{GeomBuilder, Plot};
use gogplot::utils::mtcars::mtcars;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = mtcars();

    // Plot 1: MPG vs Weight - scatter plot
    println!("Creating scatter plot: MPG vs Weight...");
    let plot1 = Plot::new(Some(data.clone_box()))
        .aes(|a| {
            a.x("wt");
            a.y("mpg");
        })
        .geom_point_with(|geom| geom.size(5.0))
        .title("Fuel Efficiency vs Weight");

    plot1.save("mtcars_scatter.png", 800, 600)?;
    println!("Created mtcars_scatter.png");

    // Plot 2: MPG vs Weight colored by cylinder count (using categorical mapping)
    println!("\nCreating colored scatter plot: MPG vs Weight by Cylinders...");
    let plot2 = Plot::new(Some(data.clone_box()))
        .aes(|a| {
            a.x("wt");
            a.y("mpg");
            a.color_categorical("cyl"); // Treat numeric cyl as categorical
        })
        .geom_point_with(|geom| geom.size(6.0))
        .title("Fuel Efficiency vs Weight by Cylinder Count");

    plot2.save("mtcars_colored.png", 800, 600)?;
    println!("Created mtcars_colored.png");

    // Plot 3: Horsepower vs Displacement
    println!("\nCreating scatter plot: Horsepower vs Displacement...");
    let plot3 = Plot::new(Some(data.clone_box()))
        .aes(|a| {
            a.x("disp");
            a.y("hp");
        })
        .geom_point_with(|geom| geom.size(5.0))
        .title("Horsepower vs Engine Displacement");

    plot3.save("mtcars_hp_disp.png", 800, 600)?;
    println!("Created mtcars_hp_disp.png");

    println!("\n✓ All plots created successfully!");
    println!("\nGenerated plots:");
    println!("  - mtcars_scatter.png");
    println!("  - mtcars_colored.png");
    println!("  - mtcars_hp_disp.png");

    Ok(())
}
