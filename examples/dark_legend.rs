use gogplot_rs::aesthetics::{Aesthetic, AesValue};
use gogplot_rs::geom::point::GeomPoint;
use gogplot_rs::plot::Plot;
use gogplot_rs::scale::continuous::Builder;
use gogplot_rs::theme::Theme;
use gogplot_rs::utils::dataframe::{DataFrame, FloatVec};
use gogplot_rs::guide::{LegendGuide, LegendEntry, LegendPosition, Guides, Shape};
use gogplot_rs::theme::Color;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating dark theme with legend example...");

    // Sample data for scatter plot
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0])));
    df.add_column("y", Box::new(FloatVec(vec![2.1, 3.9, 3.2, 5.8, 6.5, 5.2, 7.8, 8.1])));

    // Create scales
    let x_scale = Builder::new()
        .limits((0.0, 9.0))
        .linear()?;
    
    let y_scale = Builder::new()
        .limits((0.0, 10.0))
        .linear()?;

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
        color: Some(color_legend),
        shape: None,
        size: None,
        alpha: None,
    };

    // Create a point geom
    let geom = GeomPoint::default()
        .size(6.0)
        .color(230, 97, 0, 255); // Orange
    
    // Create a layer and map x, y aesthetics to data columns
    let mut layer = geom.into_layer();
    layer.mapping.set(Aesthetic::X, AesValue::Column("x".to_string()));
    layer.mapping.set(Aesthetic::Y, AesValue::Column("y".to_string()));

    // Create plot with dark theme and legend
    let plot = Plot::new(Some(Box::new(df)))
        .title("Dark Theme with Legend")
        .x_label("X Values")
        .y_label("Y Values")
        .scale_x(Box::new(x_scale))
        .scale_y(Box::new(y_scale))
        .theme(Theme::dark())
        .guides(guides)
        .layer(layer);

    // Save to a file
    plot.save("dark_legend.png", 800, 600)?;

    println!("Plot saved to dark_legend.png");
    
    Ok(())
}
