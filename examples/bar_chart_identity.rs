// Bar chart example with Identity stat
//
// This example demonstrates using the bar geom with pre-computed values
// (no automatic counting), using the Identity stat.

use gogplot_rs::aesthetics::{Aesthetic, AesMap, AesValue};
use gogplot_rs::geom::bar::GeomBar;
use gogplot_rs::layer::{Layer, Stat, Position};
use gogplot_rs::utils::dataframe::{DataFrame, FloatVec, StrVec};
use gogplot_rs::plot::Plot;
use gogplot_rs::theme::color;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create data with pre-computed values (e.g., summary statistics)
    let categories = vec!["Q1", "Q2", "Q3", "Q4"];
    let sales = vec![42.5, 55.2, 48.7, 63.1];
    
    let mut df = DataFrame::new();
    df.add_column("quarter", Box::new(StrVec::from(categories)));
    df.add_column("sales", Box::new(FloatVec(sales)));
    
    // Create a bar geom with Identity stat (no counting)
    let geom = GeomBar::new()
        .fill(color::STEELBLUE)
        .width(0.7)
        .alpha(0.9)
        .stat(Stat::Identity);  // Use values as-is, don't count
    
    // Create a layer with explicit mappings
    let mut mapping = AesMap::new();
    mapping.set(Aesthetic::X, AesValue::Column("quarter".to_string()));
    mapping.set(Aesthetic::Y, AesValue::Column("sales".to_string()));
    
    let layer = Layer {
        geom: Box::new(geom),
        data: Some(Box::new(df)),
        mapping,
        stat: Stat::Identity,  // Pre-computed values
        position: Position::Identity,
    };
    
    // Create plot and add the layer
    let mut plot = Plot::new(None);
    plot.layers.push(layer);
    
    // Bar charts should have y-axis starting at 0 for accurate visual comparison
    use gogplot_rs::scale::continuous::Builder;
    let y_scale = Builder::new().set_lower_bound(0.0).linear()?;
    
    let plot = plot
        .title("Quarterly Sales (Identity Stat)")
        .scale_y(Box::new(y_scale));
    
    // Save the plot
    plot.save("bar_chart_identity.png", 800, 600)?;
    
    println!("Bar chart saved to bar_chart_identity.png");
    println!("Values: Q1=42.5, Q2=55.2, Q3=48.7, Q4=63.1");
    
    Ok(())
}
