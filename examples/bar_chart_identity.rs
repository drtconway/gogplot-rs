// Bar chart example with Identity stat
//
// This example demonstrates using the bar geom with pre-computed values
// (no automatic counting), using the Identity stat.

use gogplot_rs::layer::Stat;
use gogplot_rs::plot::Plot;
use gogplot_rs::theme::color;
use gogplot_rs::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create data with pre-computed values (e.g., summary statistics)
    let categories = vec!["Q1", "Q2", "Q3", "Q4"];
    let sales = vec![42.5, 55.2, 48.7, 63.1];
    
    let mut df = DataFrame::new();
    df.add_column("quarter", Box::new(StrVec::from(categories)));
    df.add_column("sales", Box::new(FloatVec(sales)));
    
    // Bar charts should have y-axis starting at 0 for accurate visual comparison
    use gogplot_rs::scale::continuous::Builder;
    let y_scale = Builder::new().set_lower_bound(0.0).linear()?;
    
    let plot = Plot::new(Some(Box::new(df)))
        .aes(|a| {
            a.x("quarter");
            a.y("sales");
        })
        .geom_bar_with(|geom| {
            geom.stat(Stat::Identity)  // Use values as-is, don't count
                .fill(color::STEELBLUE)
                .width(0.7)
                .alpha(0.9)
        })
        .title("Quarterly Sales (Identity Stat)")
        .scale_y(Box::new(y_scale));
    
    // Save the plot
    plot.save("bar_chart_identity.png", 800, 600)?;
    
    println!("Bar chart saved to bar_chart_identity.png");
    println!("Values: Q1=42.5, Q2=55.2, Q3=48.7, Q4=63.1");
    
    Ok(())
}
