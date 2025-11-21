// Example demonstrating grouped and stacked histograms
// This shows what we want to achieve with multiple data series

use gogplot::plot::{GeomBuilder, Plot};
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate sample data with two groups
    let mut values = Vec::new();
    let mut groups = Vec::new();
    
    // Group A: centered around 5
    for i in 0..500 {
        let u1 = (i as f64 + 1.0) / 501.0;
        let u2 = ((i * 7 + 13) % 500) as f64 / 500.0;
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        values.push(z * 1.5 + 5.0);
        groups.push("A".to_string());
    }
    
    // Group B: centered around 7
    for i in 0..500 {
        let u1 = (i as f64 + 1.0) / 501.0;
        let u2 = ((i * 11 + 17) % 500) as f64 / 500.0;
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        values.push(z * 1.5 + 7.0);
        groups.push("B".to_string());
    }

    let mut df = DataFrame::new();
    df.add_column("value", Box::new(FloatVec(values)));
    df.add_column("group", Box::new(StrVec(groups)));

    // Example 1: Overlapping histograms with transparency
    Plot::new(Some(Box::new(df)))
        .aes(|a| {
            a.x("value");
            a.fill("group");  // Fill color by group
        })
        .geom_histogram_with(|g| g.bins(20).alpha(0.5))
        .title("Overlapping Histograms (position = identity)")
        .save("histogram_overlapping.png", 800, 600)?;

    // Example 2: Stacked histogram
    // TODO: This would require Position::Stack to be implemented
    // Plot::new(Some(Box::new(df.clone())))
    //     .aes(|a| {
    //         a.x("value");
    //         a.fill("group");
    //     })
    //     .geom_histogram_with(|g| g.bins(20).position(Position::Stack))
    //     .title("Stacked Histogram")
    //     .save("histogram_stacked.png", 800, 600)?;

    // Example 3: Dodged histogram (side-by-side bars)
    // TODO: This would require Position::Dodge to be implemented
    // Plot::new(Some(Box::new(df)))
    //     .aes(|a| {
    //         a.x("value");
    //         a.fill("group");
    //     })
    //     .geom_histogram_with(|g| g.bins(20).position(Position::Dodge))
    //     .title("Dodged Histogram (side-by-side)")
    //     .save("histogram_dodged.png", 800, 600)?;

    println!("Grouped histogram examples saved!");

    Ok(())
}
