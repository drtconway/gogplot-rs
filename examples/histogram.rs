use gogplot::plot::{GeomBuilder, Plot};
use gogplot::theme::color;
use gogplot::utils::dataframe::{DataFrame, FloatVec};

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

    // Print data statistics
    let min_val = values.iter().copied().fold(f64::INFINITY, f64::min);
    let max_val = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let range = max_val - min_val;
    
    println!("Data statistics:");
    println!("  Min: {:.2}", min_val);
    println!("  Max: {:.2}", max_val);
    println!("  Range: {:.2}", range);
    println!("  Expected bins with binwidth=0.5: {:.0}", (range / 0.5).ceil());
    println!();

    // Create a basic histogram
    let mut df1 = DataFrame::new();
    df1.add_column("value", Box::new(FloatVec(values.clone())));
    
    Plot::new(Some(Box::new(df1)))
        .aes(|a| {
            a.x("value");
        })
        .geom_histogram_with(|g| g.fill(color::STEELBLUE).bins(30))
        .title("Basic Histogram (30 bins)")
        .save("histogram_basic.png", 800, 600)?;

    // Histogram with custom bin count
    let mut df2 = DataFrame::new();
    df2.add_column("value", Box::new(FloatVec(values.clone())));
    
    Plot::new(Some(Box::new(df2)))
        .aes(|a| {
            a.x("value");
        })
        .geom_histogram_with(|g| g.fill(color::CORAL).bins(50).alpha(0.7))
        .title("Histogram with 50 Bins")
        .save("histogram_50bins.png", 800, 600)?;

    // Histogram with specific bin width
    let mut df3 = DataFrame::new();
    df3.add_column("value", Box::new(FloatVec(values.clone())));
    
    Plot::new(Some(Box::new(df3)))
        .aes(|a| {
            a.x("value");
        })
        .geom_histogram_with(|g| g.fill(color::FORESTGREEN).binwidth(0.5).alpha(0.8))
        .title("Histogram (binwidth = 0.5)")
        .save("histogram_binwidth.png", 800, 600)?;

    // Histogram with border
    let mut df4 = DataFrame::new();
    df4.add_column("value", Box::new(FloatVec(values)));
    
    Plot::new(Some(Box::new(df4)))
        .aes(|a| {
            a.x("value");
        })
        .geom_histogram_with(|g| {
            g.fill(color::LIGHTBLUE)
                .color(color::DARKBLUE)
                .bins(25)
                .alpha(0.9)
        })
        .title("Histogram with Border (25 bins)")
        .save("histogram_border.png", 800, 600)?;

    println!("Histograms saved successfully!");
    println!("- histogram_basic.png");
    println!("- histogram_50bins.png");
    println!("- histogram_binwidth.png");
    println!("- histogram_border.png");

    Ok(())
}
