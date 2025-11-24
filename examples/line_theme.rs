use gogplot::prelude::*;
use gogplot::theme::{color, Theme};
use gogplot::utils::dataframe::{DataFrame, FloatVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating line theme examples...");
    
    example_default_theme()?;
    example_custom_theme()?;
    
    println!("All line theme examples completed!");
    Ok(())
}

fn example_default_theme() -> Result<(), Box<dyn std::error::Error>> {
    // Create data
    let x_vals = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let y_vals = vec![2.0, 3.0, 4.0, 5.0, 4.5];

    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(x_vals)));
    df.add_column("y", Box::new(FloatVec(y_vals)));

    // Create plot with default theme
    let plot = Plot::new(Some(Box::new(df)))
        .title("Line with Default Theme")
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_line();  // Uses theme defaults: size=1.0, color=black, alpha=1.0, linetype="-"

    plot.save("line_theme_default.png", 800, 600)?;
    println!("Saved line_theme_default.png");
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
    theme.geom_line.size = 3.0;  // Thicker line
    theme.geom_line.color = color::STEELBLUE;
    theme.geom_line.alpha = 0.8;
    theme.geom_line.linestyle = "-.".into();  // Dash-dot pattern
    
    let plot = Plot::new(Some(Box::new(df)))
        .title("Line with Custom Theme (size=3, color=steelblue, alpha=0.8, linestyle=dash-dot)")
        .theme(theme)
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_line();  // Uses custom theme

    plot.save("line_theme_custom.png", 800, 600)?;
    println!("Saved line_theme_custom.png");
    Ok(())
}
