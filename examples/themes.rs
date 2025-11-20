use gogplot_rs::plot::{GeomBuilder, Plot};
use gogplot_rs::theme::{Theme, Color};
use gogplot_rs::utils::dataframe::{DataFrame, FloatVec, IntVec};

fn create_sample_plot(theme: Theme, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create some sample data
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(IntVec(vec![1, 2, 3, 4, 5, 6, 7, 8])));
    df.add_column("y", Box::new(FloatVec(vec![2.0, 4.0, 3.0, 5.0, 6.0, 5.5, 7.0, 8.0])));

    // Create plot with the provided theme using simplified API
    let plot = Plot::new(Some(Box::new(df)))
        .title("Theme Demo Plot")
        .theme(theme)
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point_with(|geom| geom.size(6.0).color(Color::rgb(50, 100, 200)));

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
