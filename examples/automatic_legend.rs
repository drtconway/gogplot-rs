use gogplot_rs::plot::{GeomBuilder, Plot};
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

    // Create plot - everything is automatic!
    let plot = Plot::new(Some(Box::new(df)))
        .title("Automatic Legend from Color Mapping")
        .aes(|a| {
            a.x("x");
            a.y("y");
            a.color("species"); // Map color to species column
        })
        .geom_point_with(|geom| geom.size(6.0));

    // Save to a file
    plot.save("automatic_legend.png", 800, 600)?;

    println!("Plot saved to automatic_legend.png");
    println!("Scales, labels, and legend were all automatically generated!");
    
    Ok(())
}
