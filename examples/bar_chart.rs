// Bar chart example with count stat
//
// This example demonstrates the bar geom with automatic counting of categorical data.

use gogplot_rs::plot::{GeomBuilder, Plot};
use gogplot_rs::theme::color;
use gogplot_rs::utils::dataframe::{DataFrame, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create sample data - categories with repeated values
    let categories = vec!["A", "B", "A", "C", "B", "A", "B", "C", "C", "A", "C", "B"];

    let mut df = DataFrame::new();
    df.add_column("category", Box::new(StrVec::from(categories)));

    let plot = Plot::new(Some(Box::new(df)))
        .title("Bar Chart - Category Counts")
        .aes(|a| a.x("category"))
        .geom_bar_with(|geom| geom.fill(color::STEELBLUE).width(0.7).alpha(0.9))
        .y_scale_with(|scale| scale.set_lower_bound(0.0));

    // Save the plot
    plot.save("bar_chart.png", 800, 600)?;

    println!("Bar chart saved to bar_chart.png");
    println!("Expected counts: A=4, B=4, C=4");

    Ok(())
}
