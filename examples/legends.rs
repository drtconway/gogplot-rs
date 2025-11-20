use gogplot_rs::plot::Plot;
use gogplot_rs::utils::dataframe::{DataFrame, FloatVec};
use gogplot_rs::guide::{AxisGuide, LegendGuide, LegendEntry, LegendPosition, Guides};
use gogplot_rs::theme::Color;
use gogplot_rs::visuals::Shape;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating legends example...");

    // Sample data for scatter plot
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0])));
    df.add_column("y", Box::new(FloatVec(vec![2.1, 3.9, 3.2, 5.8, 6.5, 5.2, 7.8, 8.1])));

    // Create a color legend using builder API
    let color_legend = LegendGuide::new()
        .title("Species")
        .position(LegendPosition::Right)
        .entry(LegendEntry::new("setosa")
            .color(Color::rgb(230, 97, 0))
            .shape(Shape::Circle)
            .size(5.0))
        .entry(LegendEntry::new("versicolor")
            .color(Color::rgb(0, 158, 115))
            .shape(Shape::Circle)
            .size(5.0))
        .entry(LegendEntry::new("virginica")
            .color(Color::rgb(204, 121, 167))
            .shape(Shape::Circle)
            .size(5.0));

    // Create guides with color legend
    let guides = Guides::new()
        .color(color_legend);

    // Create plot with legend using fluent API
    let plot = Plot::new(Some(Box::new(df)))
        .title("Scatter Plot with Legend")
        .guides(guides
            .x_axis(AxisGuide::x().title("X Values"))
            .y_axis(AxisGuide::y().title("Y Values")))
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point_with(|geom| {
            geom.size(6.0).color(Color::rgb(230, 97, 0))
        });

    // Save to a file - margins are automatically adjusted for legends
    plot.save("legends.png", 800, 600)?;

    println!("Plot saved to legends.png");
    
    Ok(())
}
