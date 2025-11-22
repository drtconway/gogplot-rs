use gogplot::aesthetics::Aesthetic;
use gogplot::layer::Stat;
use gogplot::plot::GeomBuilder;
use gogplot::prelude::Plot;
use gogplot::theme::color;
use gogplot::utils::dataframe::{DataFrame, FloatVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create sample data
    let x_vals: Vec<f64> = (1..=20).map(|i| i as f64).collect();
    let y_vals = vec![
        2.3, 3.1, 4.8, 5.2, 6.1, 2.1, 3.5, 4.2, 5.8, 6.5, 
        3.8, 4.1, 5.3, 4.9, 5.5, 3.2, 4.7, 5.1, 6.2, 5.9
    ];

    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(x_vals)));
    df.add_column("y", Box::new(FloatVec(y_vals)));

    // Simple example - just show horizontal reference lines at summary statistics
    let plot = Plot::new(Some(Box::new(df)))
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .title("Simple Summary Statistics Demo")
        // Scatter points
        .geom_point()
        // Add reference line at mean with stat computation
        .geom_hline_with(|layer| {
            layer.aes.yintercept("mean");
            layer.stat = Stat::Summary(vec![Aesthetic::Y]);
            layer.geom.color(color::RED).size(2.0);
        })
        // Add reference line at median
        .geom_hline_with(|layer| {
            layer.aes.yintercept("median");
            layer.stat = Stat::Summary(vec![Aesthetic::Y]);
            layer.geom.color(color::BLUE).size(2.0).linetype("--");
        });

    // Save the plot
    plot.save("simple_summary.png", 800, 600)?;

    println!("Plot saved to simple_summary.png");
    println!("Red line: mean = calculated from data");
    println!("Blue dashed line: median = calculated from data");

    Ok(())
}
