use gogplot::prelude::*;
use gogplot::theme::{color, Color, Theme};
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};
use gogplot::visuals::Shape;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing theme defaults across different geoms...");
    
    test_point_defaults()?;
    test_bar_defaults()?;
    test_mixed_geoms()?;
    
    println!("All theme default tests completed!");
    Ok(())
}

fn test_point_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0])));
    df.add_column("y", Box::new(FloatVec(vec![2.0, 3.0, 2.5])));

    // Test 1: Default theme (should use theme.geom_point.*)
    let plot = Plot::new(Some(Box::new(df.clone())))
        .title("Points: Default Theme")
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point();

    plot.save("theme_defaults_point_default.png", 600, 400)?;
    
    // Test 2: Custom theme (should use custom values)
    let mut theme = Theme::default();
    theme.geom_point.size = 10.0;
    theme.geom_point.color = color::STEELBLUE;
    theme.geom_point.alpha = 0.6;
    
    let plot = Plot::new(Some(Box::new(df)))
        .title("Points: Custom Theme (size=10, color=steelblue, alpha=0.6)")
        .theme(theme)
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point();

    plot.save("theme_defaults_point_custom.png", 600, 400)?;
    println!("Point theme defaults: ✓");
    Ok(())
}

fn test_bar_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let mut df = DataFrame::new();
    df.add_column("category", Box::new(StrVec(vec!["A".to_string(), "B".to_string(), "C".to_string()])));
    df.add_column("value", Box::new(FloatVec(vec![10.0, 15.0, 12.0])));

    // Test 1: Default theme (should use theme.geom_rect.fill)
    let plot = Plot::new(Some(Box::new(df.clone())))
        .title("Bars: Default Theme (gray fill)")
        .aes(|a| {
            a.x("category");
            a.y("value");
        })
        .geom_bar();

    plot.save("theme_defaults_bar_default.png", 600, 400)?;
    
    // Test 2: Custom theme (should use custom fill)
    let mut theme = Theme::default();
    theme.geom_rect.fill = color::CORAL;
    theme.geom_rect.alpha = 0.8;
    
    let plot = Plot::new(Some(Box::new(df)))
        .title("Bars: Custom Theme (fill=coral, alpha=0.8)")
        .theme(theme)
        .aes(|a| {
            a.x("category");
            a.y("value");
        })
        .geom_bar();

    plot.save("theme_defaults_bar_custom.png", 600, 400)?;
    println!("Bar theme defaults: ✓");
    Ok(())
}

fn test_mixed_geoms() -> Result<(), Box<dyn std::error::Error>> {
    // Create two separate plots to show different geom theme defaults
    let mut df1 = DataFrame::new();
    df1.add_column("category", Box::new(StrVec(vec!["A".to_string(), "B".to_string(), "C".to_string()])));
    df1.add_column("value", Box::new(FloatVec(vec![10.0, 15.0, 12.0])));

    let mut df2 = DataFrame::new();
    df2.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0])));
    df2.add_column("y", Box::new(FloatVec(vec![11.0, 14.0, 13.0])));

    // Custom theme with different defaults for each geom type
    let mut theme = Theme::default();
    theme.geom_rect.fill = Color::rgb(100, 149, 237);  // Cornflower blue
    theme.geom_rect.alpha = 0.7;
    theme.geom_point.color = Color::rgb(220, 20, 60);   // Crimson
    theme.geom_point.size = 8.0;
    theme.geom_point.shape = Shape::Diamond as i64;
    
    // Bar plot using rect theme
    let plot1 = Plot::new(Some(Box::new(df1)))
        .title("Bars Use geom_rect Theme")
        .theme(theme.clone())
        .aes(|a| {
            a.x("category");
            a.y("value");
        })
        .geom_bar();

    plot1.save("theme_defaults_mixed_bars.png", 600, 400)?;

    // Point plot using point theme
    let plot2 = Plot::new(Some(Box::new(df2)))
        .title("Points Use geom_point Theme")
        .theme(theme)
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point();

    plot2.save("theme_defaults_mixed_points.png", 600, 400)?;
    println!("Mixed geom theme defaults: ✓");
    Ok(())
}
