// Example: Dodged bar charts with categorical data

use gogplot::layer::{Position, Stat};
use gogplot::plot::{GeomBuilder, Plot};
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sample data: sales by category and region
    let categories = vec!["A", "B", "C", "D"];
    let regions = vec!["North", "South", "North", "South", "North", "South", "North", "South"];
    let sales = vec![23.0, 19.0, 34.0, 28.0, 15.0, 22.0, 41.0, 35.0];
    
    // Expand categories to match
    let expanded_categories: Vec<&str> = categories
        .iter()
        .flat_map(|&cat| vec![cat, cat])
        .collect();

    // Create dataframe
    let mut df = DataFrame::new();
    df.add_column("category", Box::new(StrVec::from(expanded_categories)));
    df.add_column("region", Box::new(StrVec::from(regions.clone())));
    df.add_column("sales", Box::new(FloatVec(sales.clone())));

    // Create plot with dodged bars
    let plot = Plot::new(Some(Box::new(df.clone())))
        .aes(|a| {
            a.x("category");
            a.y("sales");
            a.fill("region");
        })
        .geom_bar_with(|layer| {
            layer.geom.stat(Stat::Identity).position(Position::Dodge);
        })
        .title("Dodged Bars")
        .y_scale_with(|scale| scale.set_lower_bound(0.0));

    plot.save("dodged_bars.png", 800, 600)?;
    println!("Created dodged_bars.png");

    // Also create a stacked version for comparison
    let plot2 = Plot::new(Some(Box::new(df.clone())))
        .aes(|a| {
            a.x("category");
            a.y("sales");
            a.fill("region");
        })
        .geom_bar_with(|layer| {
            layer.geom.stat(Stat::Identity).position(Position::Stack);
        })
        .title("Stacked Bars")
        .y_scale_with(|scale| scale.set_lower_bound(0.0));

    plot2.save("stacked_bars.png", 800, 600)?;
    println!("Created stacked_bars.png");

    // And an identity version (overlapping)
    let plot3 = Plot::new(Some(Box::new(df)))
        .aes(|a| {
            a.x("category");
            a.y("sales");
            a.fill("region");
        })
        .geom_bar_with(|layer| {
            layer.geom.stat(Stat::Identity).position(Position::Identity);
        })
        .title("Identity (Overlapping) Bars")
        .y_scale_with(|scale| scale.set_lower_bound(0.0));

    plot3.save("identity_bars.png", 800, 600)?;
    println!("Created identity_bars.png");

    Ok(())
}
