use gogplot::prelude::*;
use gogplot::theme::{color, Theme};
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn main() {
    println!("Creating text theme examples...");
    
    example_default_theme().unwrap();
    example_custom_theme().unwrap();
    
    println!("All text theme examples completed!");
}

fn example_default_theme() -> Result<(), Box<dyn std::error::Error>> {
    // Create data
    let x_vals = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let y_vals = vec![2.0, 3.0, 4.0, 5.0, 4.5];
    let labels = vec!["A", "B", "C", "D", "E"];

    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(x_vals)));
    df.add_column("y", Box::new(FloatVec(y_vals)));
    df.add_column("label", Box::new(StrVec::from(labels)));

    // Create plot with default theme (text defaults from theme)
    let plot = Plot::new(Some(Box::new(df)))
        .title("Text with Default Theme")
        .aes(|a| {
            a.x("x");
            a.y("y");
            a.label("label");
        })
        .geom_point()
        .geom_text();  // Uses theme defaults: size=11, color=black, alpha=1.0

    plot.save("text_theme_default.png", 800, 600)?;
    println!("Saved text_theme_default.png");
    Ok(())
}

fn example_custom_theme() -> Result<(), Box<dyn std::error::Error>> {
    // Create data
    let x_vals = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let y_vals = vec![2.0, 3.0, 4.0, 5.0, 4.5];
    let labels = vec!["A", "B", "C", "D", "E"];

    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(x_vals)));
    df.add_column("y", Box::new(FloatVec(y_vals)));
    df.add_column("label", Box::new(StrVec::from(labels)));

    // Create plot with custom theme
    let mut theme = Theme::default();
    theme.geom_text.size = 16.0;  // Larger text
    theme.geom_text.color = color::BLUE;
    theme.geom_text.alpha = 0.9;
    
    let plot = Plot::new(Some(Box::new(df)))
        .title("Text with Custom Theme (size=16, color=blue, alpha=0.9)")
        .theme(theme)
        .aes(|a| {
            a.x("x");
            a.y("y");
            a.label("label");
        })
        .geom_point_with(|layer| {
            layer.geom.size(5.0).color(color::STEELBLUE);
        })
        .geom_text();  // Uses custom theme

    plot.save("text_theme_custom.png", 800, 600)?;
    println!("Saved text_theme_custom.png");
    Ok(())
}
