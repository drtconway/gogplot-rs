// Example: Cumulative histograms
//
// To run this example, use:
//   cargo run --example cumulative_histogram

use gogplot::layer::{Position, Stat};
use gogplot::plot::{GeomBuilder, Plot};
use gogplot::stat::bin::BinStrategy;
use gogplot::theme::color;
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate sample data from a normal distribution
    let mut values = Vec::new();
    let n = 1000;
    
    // Simulate normal distribution using Box-Muller transform
    for i in 0..n {
        let u1 = (i as f64 + 1.0) / (n as f64 + 1.0);
        let u2 = ((i * 7 + 13) % n) as f64 / n as f64;
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        values.push(z * 2.0 + 5.0); // mean=5, sd=2
    }

    // Plot 1: Regular histogram
    println!("Creating regular histogram...");
    let mut df1 = DataFrame::new();
    df1.add_column("value", Box::new(FloatVec(values.clone())));
    
    Plot::new(Some(Box::new(df1)))
        .aes(|a| {
            a.x("value");
        })
        .geom_histogram_with(|g| g.fill(color::STEELBLUE).bins(30))
        .title("Regular Histogram")
        .save("histogram_regular.png", 800, 600)?;
    println!("Created histogram_regular.png");

    // Plot 2: Cumulative histogram (using geom method)
    println!("\nCreating cumulative histogram (via geom)...");
    let mut df2 = DataFrame::new();
    df2.add_column("value", Box::new(FloatVec(values.clone())));
    
    Plot::new(Some(Box::new(df2)))
        .aes(|a| {
            a.x("value");
        })
        .geom_histogram_with(|g| {
            g.fill(color::FORESTGREEN)
                .bins(30)
                .cumulative(true)  // Enable cumulative mode
        })
        .title("Cumulative Histogram (via geom)")
        .save("histogram_cumulative.png", 800, 600)?;
    println!("Created histogram_cumulative.png");

    // Plot 3: Cumulative histogram with explicit stat
    println!("\nCreating cumulative histogram (via stat)...");
    let mut df3 = DataFrame::new();
    df3.add_column("value", Box::new(FloatVec(values.clone())));
    
    Plot::new(Some(Box::new(df3)))
        .aes(|a| {
            a.x("value");
        })
        .geom_histogram_with(|g| {
            g.fill(color::CORAL)
                .stat(Stat::Bin(
                    BinStrategy::Count(30).cumulative(true)
                ))
        })
        .title("Cumulative Histogram (via stat)")
        .save("histogram_cumulative_stat.png", 800, 600)?;
    println!("Created histogram_cumulative_stat.png");

    // Plot 4: Grouped cumulative histograms
    println!("\nCreating grouped cumulative histograms...");
    let n_per_group = 500;
    let mut all_values = Vec::new();
    let mut all_groups = Vec::new();
    
    // Group A: mean=4, sd=1.5
    for i in 0..n_per_group {
        let u1 = (i as f64 + 1.0) / (n_per_group as f64 + 1.0);
        let u2 = ((i * 7 + 13) % n_per_group) as f64 / n_per_group as f64;
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        all_values.push(z * 1.5 + 4.0);
        all_groups.push("Group A".to_string());
    }
    
    // Group B: mean=6, sd=2
    for i in 0..n_per_group {
        let u1 = (i as f64 + 1.0) / (n_per_group as f64 + 1.0);
        let u2 = ((i * 11 + 17) % n_per_group) as f64 / n_per_group as f64;
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos();
        all_values.push(z * 2.0 + 6.0);
        all_groups.push("Group B".to_string());
    }
    
    let mut df4 = DataFrame::new();
    df4.add_column("value", Box::new(FloatVec(all_values)));
    df4.add_column("group", Box::new(StrVec(all_groups)));
    
    Plot::new(Some(Box::new(df4)))
        .aes(|a| {
            a.x("value");
            a.fill("group");
        })
        .geom_histogram_with(|g| {
            g.bins(25)
                .cumulative(true)
                .alpha(0.6)
                .position(Position::Identity)
        })
        .title("Grouped Cumulative Histograms")
        .save("histogram_cumulative_grouped.png", 800, 600)?;
    println!("Created histogram_cumulative_grouped.png");

    println!("\n✓ All cumulative histograms created successfully!");
    println!("\nGenerated plots:");
    println!("  - histogram_regular.png");
    println!("  - histogram_cumulative.png");
    println!("  - histogram_cumulative_stat.png");
    println!("  - histogram_cumulative_grouped.png");

    Ok(())
}
