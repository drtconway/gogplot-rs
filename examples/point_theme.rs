use gogplot::prelude::*;
use gogplot::theme::{color, Theme};
use gogplot::utils::dataframe::{DataFrame, FloatVec};
use gogplot::visuals::Shape;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating point theme examples...");
    
    example_default_theme()?;
    example_custom_theme()?;
    
    println!("All point theme examples completed!");
    Ok(())
}

fn example_default_theme() -> Result<(), Box<dyn std::error::Error>> {
    // Create data
    let x_vals = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let y_vals = vec![2.0, 3.0, 4.0, 5.0, 4.5];

    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(x_vals)));
    df.add_column("y", Box::new(FloatVec(y_vals)));

    // Create plot with default theme (point defaults from theme)
    let plot = Plot::new(Some(Box::new(df)))
        .title("Points with Default Theme")
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point();  // Uses theme defaults: size=3, color=black, alpha=1.0, shape=circle

    plot.save("point_theme_default.png", 800, 600)?;
    println!("Saved point_theme_default.png");
    Ok(())
}

fn example_custom_theme() -> Result<(), Box<dyn std::error::Error>> {
    // Create data
    let x_vals = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let y_vals = vec![2.0, 3.0, 4.0, 5.0, 4.5];

    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(x_vals)));
    df.add_column("y", Box::new(FloatVec(y_vals)));

    // Create plot with custom theme
    let mut theme = Theme::default();
    theme.geom_point.size = 8.0;  // Larger points
    theme.geom_point.color = color::STEELBLUE;
    theme.geom_point.alpha = 0.7;
    theme.geom_point.shape = Shape::Triangle as i64;
    
    let plot = Plot::new(Some(Box::new(df)))
        .title("Points with Custom Theme (size=8, color=steelblue, alpha=0.7, shape=triangle)")
        .theme(theme)
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point();  // Uses custom theme

    plot.save("point_theme_custom.png", 800, 600)?;
    println!("Saved point_theme_custom.png");
    Ok(())
}
