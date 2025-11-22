// Legend examples: automatic, manual, and themed legends
//
// This example demonstrates three approaches to creating legends:
// 1. Automatic legend generation from aesthetic mappings
// 2. Manual legend creation using the builder API
// 3. Dark themed legend

use gogplot::guide::{AxisGuide, Guides, LegendEntry, LegendGuide, LegendPosition};
use gogplot::plot::{GeomBuilder, Plot};
use gogplot::theme::{Color, Theme};
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};
use gogplot::visuals::Shape;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Automatic legend from color mapping
    automatic_legend()?;
    
    // Example 2: Manual legend with builder API
    manual_legend()?;
    
    // Example 3: Dark themed legend
    dark_themed_legend()?;
    
    Ok(())
}

fn automatic_legend() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating automatic legend example...");

    // Sample data with species categories
    let mut df = DataFrame::new();
    df.add_column(
        "x",
        Box::new(FloatVec(vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 1.5, 2.5,
        ])),
    );
    df.add_column(
        "y",
        Box::new(FloatVec(vec![
            2.1, 3.9, 3.2, 5.8, 6.5, 5.2, 7.8, 8.1, 2.5, 4.2,
        ])),
    );
    df.add_column(
        "species",
        Box::new(StrVec::from(vec![
            "setosa",
            "setosa",
            "versicolor",
            "versicolor",
            "virginica",
            "virginica",
            "setosa",
            "versicolor",
            "virginica",
            "setosa",
        ])),
    );

    // Create plot - everything is automatic!
    let plot = Plot::new(Some(Box::new(df)))
        .title("Automatic Legend from Color Mapping")
        .aes(|a| {
            a.x("x");
            a.y("y");
            a.color("species"); // Map color to species column
        })
        .geom_point_with(|layer| { layer.geom.size(6.0); });

    plot.save("automatic_legend.png", 800, 600)?;
    println!("Plot saved to automatic_legend.png");
    println!("Scales, labels, and legend were all automatically generated!");

    Ok(())
}

fn manual_legend() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating manual legend example...");

    // Sample data for scatter plot
    let mut df = DataFrame::new();
    df.add_column(
        "x",
        Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0])),
    );
    df.add_column(
        "y",
        Box::new(FloatVec(vec![2.1, 3.9, 3.2, 5.8, 6.5, 5.2, 7.8, 8.1])),
    );

    // Create a color legend using builder API
    let color_legend = LegendGuide::new()
        .title("Species")
        .position(LegendPosition::Right)
        .entry(
            LegendEntry::new("setosa")
                .color(Color::rgb(230, 97, 0))
                .shape(Shape::Circle)
                .size(5.0),
        )
        .entry(
            LegendEntry::new("versicolor")
                .color(Color::rgb(0, 158, 115))
                .shape(Shape::Circle)
                .size(5.0),
        )
        .entry(
            LegendEntry::new("virginica")
                .color(Color::rgb(204, 121, 167))
                .shape(Shape::Circle)
                .size(5.0),
        );

    // Create guides with color legend
    let guides = Guides::new().color(color_legend);

    // Create plot with legend using fluent API
    let plot = Plot::new(Some(Box::new(df)))
        .title("Manual Legend with Builder API")
        .guides(
            guides
                .x_axis(AxisGuide::x().title("X Values"))
                .y_axis(AxisGuide::y().title("Y Values")),
        )
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point_with(|layer| { layer.geom.size(6.0).color(Color::rgb(230, 97, 0)); });

    plot.save("manual_legend.png", 800, 600)?;
    println!("Plot saved to manual_legend.png");

    Ok(())
}

fn dark_themed_legend() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating dark theme with legend example...");

    // Sample data for scatter plot
    let mut df = DataFrame::new();
    df.add_column(
        "x",
        Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0])),
    );
    df.add_column(
        "y",
        Box::new(FloatVec(vec![2.1, 3.9, 3.2, 5.8, 6.5, 5.2, 7.8, 8.1])),
    );

    // Create a color legend using builder API
    let color_legend = LegendGuide::new()
        .title("Species")
        .position(LegendPosition::Right)
        .entry(
            LegendEntry::new("setosa")
                .color(Color::rgb(230, 97, 0))
                .shape(Shape::Circle)
                .size(5.0),
        )
        .entry(
            LegendEntry::new("versicolor")
                .color(Color::rgb(0, 158, 115))
                .shape(Shape::Circle)
                .size(5.0),
        )
        .entry(
            LegendEntry::new("virginica")
                .color(Color::rgb(204, 121, 167))
                .shape(Shape::Circle)
                .size(5.0),
        );

    // Create guides with color legend
    let guides = Guides::new().color(color_legend);

    // Create plot with dark theme and legend
    let plot = Plot::new(Some(Box::new(df)))
        .title("Dark Theme with Legend")
        .theme(Theme::dark())
        .guides(
            guides
                .x_axis(AxisGuide::x().title("X Values"))
                .y_axis(AxisGuide::y().title("Y Values")),
        )
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point_with(|layer| { layer.geom.size(6.0).color(Color::rgb(230, 97, 0)); });

    plot.save("dark_legend.png", 800, 600)?;
    println!("Plot saved to dark_legend.png");

    Ok(())
}
