use gogplot_rs::aesthetics::{Aesthetic, AesValue};
use gogplot_rs::geom::point::GeomPoint;
use gogplot_rs::plot::Plot;
use gogplot_rs::scale::continuous::Builder;
use gogplot_rs::theme::Theme;
use gogplot_rs::utils::dataframe::{DataFrame, FloatVec, IntVec};

fn create_sample_plot(theme: Theme, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create some sample data
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(IntVec(vec![1, 2, 3, 4, 5, 6, 7, 8])));
    df.add_column("y", Box::new(FloatVec(vec![2.0, 4.0, 3.0, 5.0, 6.0, 5.5, 7.0, 8.0])));

    // Create scales
    let x_scale = Builder::new()
        .limits((0.0, 9.0))
        .linear()?;
    
    let y_scale = Builder::new()
        .limits((0.0, 9.0))
        .linear()?;

    // Create a point geom
    let geom = GeomPoint::default()
        .size(6.0)
        .color(50, 100, 200, 255);
    
    // Create a layer and map aesthetics
    let mut layer = geom.into_layer();
    layer.mapping.set(Aesthetic::X, AesValue::Column("x".to_string()));
    layer.mapping.set(Aesthetic::Y, AesValue::Column("y".to_string()));

    // Create plot with the provided theme
    let plot = Plot::new(Some(Box::new(df)))
        .title("Theme Demo Plot")
        .x_label("X Values")
        .y_label("Y Values")
        .scale_x(Box::new(x_scale))
        .scale_y(Box::new(y_scale))
        .theme(theme)
        .layer(layer);

    // Save to a file
    plot.save(filename, 800, 600)?;
    println!("Plot saved to {}", filename);
    
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Default theme (ggplot2-like with gray background and white grid)
    create_sample_plot(Theme::default(), "theme_default.png")?;
    
    // Minimal theme (white background, no grid)
    create_sample_plot(Theme::minimal(), "theme_minimal.png")?;
    
    // Classic theme (white background with black border, no grid)
    create_sample_plot(Theme::classic(), "theme_classic.png")?;
    
    // Dark theme
    create_sample_plot(Theme::dark(), "theme_dark.png")?;
    
    Ok(())
}
