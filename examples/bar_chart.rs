// Bar chart example with count stat
//
// This example demonstrates the bar geom with automatic counting of categorical data.

use gogplot_rs::aesthetics::{AesMap, AesValue, Aesthetic};
use gogplot_rs::geom::bar::GeomBar;
use gogplot_rs::layer::{Layer, Position, Stat};
use gogplot_rs::plot::Plot;
use gogplot_rs::theme::color;
use gogplot_rs::utils::dataframe::{DataFrame, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create sample data - categories with repeated values
    let categories = vec!["A", "B", "A", "C", "B", "A", "B", "C", "C", "A", "C", "B"];

    let mut df = DataFrame::new();
    df.add_column("category", Box::new(StrVec::from(categories)));

    // Create a bar geom with styling
    let geom = GeomBar::new().fill(color::STEELBLUE).width(0.7).alpha(0.9);

    // Create a layer with the geom, data, and mapping
    let mut mapping = AesMap::new();
    mapping.set(Aesthetic::X, AesValue::Column("category".to_string()));

    let layer = Layer {
        geom: Box::new(geom),
        data: Some(Box::new(df)),
        mapping,
        stat: Stat::Count, // This will count occurrences by category
        position: Position::Identity,
    };

    // Create plot and add the layer
    let mut plot = Plot::new(None);
    plot.layers.push(layer);

    let plot = plot.title("Bar Chart - Category Counts");

    // Save the plot
    plot.save("bar_chart.png", 800, 600)?;

    println!("Bar chart saved to bar_chart.png");
    println!("Expected counts: A=4, B=4, C=4");

    Ok(())
}
