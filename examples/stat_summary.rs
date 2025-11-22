use gogplot::aesthetics::Aesthetic;
use gogplot::layer::Stat;
use gogplot::plot::GeomBuilder;
use gogplot::prelude::Plot;
use gogplot::theme::color;
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create sample data with multiple groups
    let x_vals = vec![1.0, 2.0, 3.0, 4.0, 5.0, 1.0, 2.0, 3.0, 4.0, 5.0];
    let y_vals = vec![2.3, 3.1, 4.8, 5.2, 6.1, 2.1, 3.5, 4.2, 5.8, 6.5];
    let group = vec!["A", "A", "A", "A", "A", "B", "B", "B", "B", "B"];

    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(x_vals)));
    df.add_column("y", Box::new(FloatVec(y_vals)));
    df.add_column("group", Box::new(StrVec::from(group)));

    // Create plot with scatter points and reference lines at summary statistics
    let plot = Plot::new(Some(Box::new(df)))
        .aes(|a| {
            a.x("x");
            a.y("y");
            a.color("group");
        })
        .title("Scatter Plot with Summary Statistics Reference Lines")
        // Scatter points
        .geom_point()
        // Horizontal line at mean of y
        .geom_hline_with(|layer| {
            layer.aes.yintercept("mean");
            layer.stat = Stat::Summary(vec![Aesthetic::Y]);
            layer.geom.color(color::RED).size(2.0).linetype("--");
        })
        // Horizontal line at median of y
        .geom_hline_with(|layer| {
            layer.aes.yintercept("median");
            layer.stat = Stat::Summary(vec![Aesthetic::Y]);
            layer.geom.color(color::BLUE).size(2.0).linetype("-.");
        })
        // Vertical line at mean of x
        .geom_vline_with(|layer| {
            layer.aes.xintercept("mean");
            layer.stat = Stat::Summary(vec![Aesthetic::X]);
            layer.geom.color(color::GREEN).size(1.5).linetype(":");
        });

    // Save the plot
    plot.save("stat_summary.png", 800, 600)?;

    println!("Plot saved to stat_summary.png");
    println!("The plot shows:");
    println!("  - Red dashed line: mean of y");
    println!("  - Blue dash-dot line: median of y");
    println!("  - Green dotted line: mean of x");

    Ok(())
}
