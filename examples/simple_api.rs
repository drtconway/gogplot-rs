use gogplot_rs::plot::Plot;
use gogplot_rs::utils::dataframe::{DataFrame, FloatVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create some sample data
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 5.0])));
    df.add_column("y", Box::new(FloatVec(vec![2.0, 4.0, 3.0, 5.0, 6.0])));

    // Simple, clean API similar to ggplot2:
    // - Set data once
    // - Define aesthetic mappings once
    // - Add multiple geoms that inherit the mappings
    // - Scales and labels are automatic
    let plot = Plot::new(Some(Box::new(df)))
        .title("Simple API Demo")
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point();  // Uses x and y mappings from .aes()

    plot.save("simple_api.png", 800, 600)?;
    
    println!("Plot saved to simple_api.png");
    println!("Created with just data + aes + geom!");
    
    Ok(())
}
