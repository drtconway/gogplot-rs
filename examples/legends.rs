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

    // Create a color legend manually
    let mut color_legend = LegendGuide::new();
    color_legend.title = Some("Species".to_string());
    color_legend.position = LegendPosition::Right;
    color_legend.entries.push(LegendEntry {
        label: "setosa".to_string(),
        color: Some(Color(230, 97, 0, 255)),
        shape: Some(Shape::Circle),
        size: Some(5.0),
    });
    color_legend.entries.push(LegendEntry {
        label: "versicolor".to_string(),
        color: Some(Color(0, 158, 115, 255)),
        shape: Some(Shape::Circle),
        size: Some(5.0),
    });
    color_legend.entries.push(LegendEntry {
        label: "virginica".to_string(),
        color: Some(Color(204, 121, 167, 255)),
        shape: Some(Shape::Circle),
        size: Some(5.0),
    });

    // Create guides with color legend
    let guides = Guides {
        x_axis: None,
        y_axis: None,
        color: Some(color_legend),
        shape: None,
        size: None,
        alpha: None,
    };

    // Create plot with legend using simplified API
    let plot = Plot::new(Some(Box::new(df)))
        .title("Scatter Plot with Legend")
        .guides(guides
            .x_axis(AxisGuide::x().title("X Values"))
            .y_axis(AxisGuide::y().title("Y Values")))
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point_with(|geom, _| {
            geom.size(6.0).color(Color(230, 97, 0, 255))
        });

    // Save to a file - margins are automatically adjusted for legends
    plot.save("legends.png", 800, 600)?;

    println!("Plot saved to legends.png");
    
    Ok(())
}
