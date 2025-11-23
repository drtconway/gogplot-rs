use gogplot::prelude::*;
use gogplot::theme::color;
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Creating text label examples...");

    // Example 1: Basic text labels
    example_basic_text()?;

    // Example 2: Text labels with adjustments
    example_text_adjustments()?;

    // Example 3: Rotated text
    example_rotated_text()?;

    println!("All text label examples completed!");
    Ok(())
}

fn example_basic_text() -> Result<(), Box<dyn Error>> {
    // Create sample data
    let x_vals = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let y_vals = vec![2.0, 4.0, 3.0, 5.0, 4.5];
    let labels = vec!["A", "B", "C", "D", "E"];

    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(x_vals.clone())));
    df.add_column("y", Box::new(FloatVec(y_vals.clone())));
    df.add_column("label", Box::new(StrVec::from(labels)));

    let plot = Plot::new(Some(Box::new(df)))
        .title("Text Labels - Basic")
        .aes(|a| {
            a.x("x");
            a.y("y");
            a.label("label");
        })
        .geom_point_with(|layer| {
            layer.geom.size(5.0).color(color::STEELBLUE);
        })
        .geom_text_with(|layer| {
            layer.geom.size(14.0).color(color::BLACK);
        });

    plot.save("text_basic.png", 800, 600)?;
    println!("Saved text_basic.png");
    Ok(())
}

fn example_text_adjustments() -> Result<(), Box<dyn Error>> {
    // Create sample data
    let x_vals = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let y_vals = vec![2.0, 4.0, 3.0, 5.0, 4.5];
    let labels = vec!["Left", "Right", "Top", "Bottom", "Center"];

    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(x_vals.clone())));
    df.add_column("y", Box::new(FloatVec(y_vals.clone())));
    df.add_column("label", Box::new(StrVec::from(labels)));

    let plot = Plot::new(Some(Box::new(df)))
        .title("Text Labels - With Adjustments")
        .aes(|a| {
            a.x("x");
            a.y("y");
            a.label("label");
        })
        .geom_point_with(|layer| {
            layer.geom.size(5.0).color(color::RED);
        })
        .geom_text_with(|layer| {
            layer.geom.size(12.0)
                .color(color::DARKBLUE)
                .hjust(0.0)  // Left-aligned
                .vjust(1.0); // Top
        });

    plot.save("text_adjustments.png", 800, 600)?;
    println!("Saved text_adjustments.png");
    Ok(())
}

fn example_rotated_text() -> Result<(), Box<dyn Error>> {
    // Create sample data
    let x_vals = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let y_vals = vec![2.0, 4.0, 3.0, 5.0, 4.5];
    let labels = vec!["Rotated", "Text", "Labels", "At", "45°"];

    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(x_vals.clone())));
    df.add_column("y", Box::new(FloatVec(y_vals.clone())));
    df.add_column("label", Box::new(StrVec::from(labels)));

    let plot = Plot::new(Some(Box::new(df)))
        .title("Text Labels - Rotated 45 Degrees")
        .aes(|a| {
            a.x("x");
            a.y("y");
            a.label("label");
        })
        .geom_point_with(|layer| {
            layer.geom.size(5.0).color(color::GREEN);
        })
        .geom_text_with(|layer| {
            layer.geom.size(12.0)
                .color(color::BLACK)
                .angle(45.0);
        });

    plot.save("text_rotated.png", 800, 600)?;
    println!("Saved text_rotated.png");
    Ok(())
}
