use gogplot::plot::{GeomBuilder, Plot};
use gogplot::theme::color;
use gogplot::utils::dataframe::{DataFrame, FloatVec, IntVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create some sample data
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(IntVec(vec![1, 2, 3, 4, 5])));
    df.add_column("y", Box::new(FloatVec(vec![2.0, 4.0, 3.0, 5.0, 6.0])));

    // Create a plot with aesthetics and geom - everything is automatic!
    let plot = Plot::new(Some(Box::new(df)))
        .title("Simple Scatter Plot")
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point_with(|geom| geom.size(5.0).color(color::BLUE));

    // Save to a file
    plot.save("scatter_plot.png", 800, 600)?;

    println!("Plot saved to scatter_plot.png");

    Ok(())
}
