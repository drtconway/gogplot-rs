use gogplot::layer::Position;
use gogplot::plot::{GeomBuilder, Plot};
use gogplot::utils::mtcars::mtcars;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a basic boxplot of mpg by number of cylinders
    Plot::new(Some(mtcars()))
        .title("MPG Distribution by Cylinder Count")
        .aes(|a| {
            a.x("cyl");
            a.y("mpg");
        })
        .geom_boxplot()
        .save("boxplot_basic.png", 800, 600)?;

    println!("Saved boxplot_basic.png");

    // Create a boxplot with fill mapped to cylinder
    Plot::new(Some(mtcars()))
        .title("MPG by Cylinders (Colored)")
        .aes(|a| {
            a.x("cyl");
            a.y("mpg");
            a.fill("cyl");
        })
        .geom_boxplot_with(|layer| {
            layer.geom.width(0.6).alpha(0.8);
        })
        .save("boxplot_filled.png", 800, 600)?;

    println!("Saved boxplot_filled.png");

    // Create a boxplot with multiple groups (transmission type) using dodge
    Plot::new(Some(mtcars()))
        .title("MPG by Cylinders and Transmission (Dodged)")
        .aes(|a| {
            a.x("cyl");
            a.y("mpg");
            a.fill("am");
        })
        .geom_boxplot_with(|layer| {
            layer.geom.width(0.7).position(Position::Dodge);
        })
        .save("boxplot_grouped.png", 800, 600)?;

    println!("Saved boxplot_grouped.png");

    println!("\nAll boxplots saved successfully!");

    Ok(())
}
