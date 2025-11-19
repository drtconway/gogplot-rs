use gogplot_rs::aesthetics::{Aesthetic, AesValue};
use gogplot_rs::geom::point::GeomPoint;
use gogplot_rs::plot::Plot;
use gogplot_rs::scale::continuous::Builder;
use gogplot_rs::scale::color::DiscreteColor;
use gogplot_rs::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating automatic legend example...");

    // Sample data with species categories
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 1.5, 2.5])));
    df.add_column("y", Box::new(FloatVec(vec![2.1, 3.9, 3.2, 5.8, 6.5, 5.2, 7.8, 8.1, 2.5, 4.2])));
    df.add_column("species", Box::new(StrVec::from(vec![
        "setosa", "setosa", "versicolor", "versicolor",
        "virginica", "virginica", "setosa", "versicolor",
        "virginica", "setosa",
    ])));

    // Create scales
    let x_scale = Builder::new()
        .limits((0.0, 9.0))
        .linear()?;
    
    let y_scale = Builder::new()
        .limits((0.0, 10.0))
        .linear()?;

    // Create a discrete color scale
    let color_scale = DiscreteColor::default_palette();

    // Create a point geom
    let geom = GeomPoint::default()
        .size(6.0)
        .color(100, 100, 100, 255); // Default color, will be overridden by scale
    
    // Create a layer and map x, y, and color aesthetics to data columns
    let mut layer = geom.into_layer();
    layer.mapping.set(Aesthetic::X, AesValue::Column("x".to_string()));
    layer.mapping.set(Aesthetic::Y, AesValue::Column("y".to_string()));
    layer.mapping.set(Aesthetic::Color, AesValue::Column("species".to_string())); // Map color to species

    // Create plot - legend should be generated automatically!
    let plot = Plot::new(Some(Box::new(df)))
        .title("Automatic Legend from Color Mapping")
        .x_label("X Values")
        .y_label("Y Values")
        .scale_x(Box::new(x_scale))
        .scale_y(Box::new(y_scale))
        .scale_color(Box::new(color_scale)) // Provide the color scale
        .layer(layer);

    // Save to a file
    plot.save("automatic_legend.png", 800, 600)?;

    println!("Plot saved to automatic_legend.png");
    println!("Legend should be automatically generated from the 'species' column mapping!");
    
    Ok(())
}
