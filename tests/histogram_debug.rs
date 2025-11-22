use gogplot::plot::{GeomBuilder, Plot};
use gogplot::theme::color;
use gogplot::utils::dataframe::{DataFrame, FloatVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Simple test data with known range: 0 to 10
    let values = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    
    println!("Data range: 0.0 to 10.0");
    println!("Range: 10.0");
    
    // Test 1: binwidth = 1.0 should give us 10 bins
    println!("\n=== Test 1: binwidth = 1.0 ===");
    println!("Expected bins: ~10");
    let mut df1 = DataFrame::new();
    df1.add_column("value", Box::new(FloatVec(values.clone())));
    
    Plot::new(Some(Box::new(df1)))
        .aes(|a| {
            a.x("value");
        })
        .geom_histogram_with(|layer| { layer.geom.fill(color::STEELBLUE).binwidth(1.0); })
        .title("binwidth = 1.0 (expect ~10 bins)")
        .save("histogram_debug_bw1.png", 800, 600)?;

    // Test 2: binwidth = 2.0 should give us 5 bins
    println!("\n=== Test 2: binwidth = 2.0 ===");
    println!("Expected bins: ~5");
    let mut df2 = DataFrame::new();
    df2.add_column("value", Box::new(FloatVec(values.clone())));
    
    Plot::new(Some(Box::new(df2)))
        .aes(|a| {
            a.x("value");
        })
        .geom_histogram_with(|layer| { layer.geom.fill(color::CORAL).binwidth(2.0); })
        .title("binwidth = 2.0 (expect ~5 bins)")
        .save("histogram_debug_bw2.png", 800, 600)?;

    // Test 3: binwidth = 5.0 should give us 2 bins
    println!("\n=== Test 3: binwidth = 5.0 ===");
    println!("Expected bins: ~2");
    let mut df3 = DataFrame::new();
    df3.add_column("value", Box::new(FloatVec(values.clone())));
    
    Plot::new(Some(Box::new(df3)))
        .aes(|a| {
            a.x("value");
        })
        .geom_histogram_with(|layer| { layer.geom.fill(color::FORESTGREEN).binwidth(5.0); })
        .title("binwidth = 5.0 (expect ~2 bins)")
        .save("histogram_debug_bw5.png", 800, 600)?;

    // Test 4: bins = 5 should give us exactly 5 bins
    println!("\n=== Test 4: bins = 5 ===");
    println!("Expected bins: 5");
    let mut df4 = DataFrame::new();
    df4.add_column("value", Box::new(FloatVec(values)));
    
    Plot::new(Some(Box::new(df4)))
        .aes(|a| {
            a.x("value");
        })
        .geom_histogram_with(|layer| { layer.geom.fill(color::PURPLE).bins(5); })
        .title("bins = 5 (expect exactly 5 bins)")
        .save("histogram_debug_bins5.png", 800, 600)?;

    println!("\nDebug histograms saved!");

    Ok(())
}
