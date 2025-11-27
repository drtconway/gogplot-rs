// Categorical scatter plot example
//
// This example demonstrates handling categorical variables on x and y axes
// with geom_point to isolate categorical variable issues from bar chart specifics.

use gogplot::plot::{GeomBuilder, Plot};
use gogplot::theme::color;
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Categorical X, Numeric Y
    println!("Creating categorical_x_numeric_y.png...");
    let categories_x = vec!["A", "B", "C", "A", "B", "C", "A", "B", "C"];
    let values_y = vec![1.5, 2.3, 3.1, 2.0, 2.8, 3.5, 1.8, 2.5, 3.0];

    let mut df1 = DataFrame::new();
    df1.add_column("category", Box::new(StrVec::from(categories_x)));
    df1.add_column("value", Box::new(FloatVec(values_y)));

    Plot::new(Some(Box::new(df1)))
        .title("Categorical X, Numeric Y")
        .aes(|a| {
            a.x("category");
            a.y("value");
        })
        .geom_point_with(|layer| {
            layer.geom.color(color::STEELBLUE).size(4.0);
        })
        .save("categorical_x_numeric_y.png", 800, 600)?;
    println!("Saved categorical_x_numeric_y.png");

    // Example 2: Numeric X, Categorical Y
    println!("\nCreating numeric_x_categorical_y.png...");
    let values_x = vec![1.5, 2.3, 3.1, 2.0, 2.8, 3.5, 1.8, 2.5, 3.0];
    let categories_y = vec![
        "Low", "Med", "High", "Low", "Med", "High", "Low", "Med", "High",
    ];

    let mut df2 = DataFrame::new();
    df2.add_column("value", Box::new(FloatVec(values_x)));
    df2.add_column("category", Box::new(StrVec::from(categories_y)));

    Plot::new(Some(Box::new(df2)))
        .title("Numeric X, Categorical Y")
        .aes(|a| {
            a.x("value");
            a.y("category");
        })
        .geom_point_with(|layer| {
            layer.geom.color(color::CORAL).size(4.0);
        })
        .save("numeric_x_categorical_y.png", 800, 600)?;
    println!("Saved numeric_x_categorical_y.png");

    Ok(())
}
