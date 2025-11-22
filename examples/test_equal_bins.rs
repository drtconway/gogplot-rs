// Test histogram bar widths
use gogplot::layer::Position;
use gogplot::plot::{GeomBuilder, Plot};
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Very simple case: values that clearly separate into 4 equal bins
    // Range 0-4 with 4 bins means each bin is width 1.0
    let mut df = DataFrame::new();
    df.add_column("value", Box::new(FloatVec(vec![
        0.5, 0.5,  // Bin 1: [0-1) - Group A
        1.5, 1.5,  // Bin 2: [1-2) - Group B  
        2.5, 2.5,  // Bin 3: [2-3) - Group A
        3.5, 3.5,  // Bin 4: [3-4] - Group B
    ])));
    df.add_column("group", Box::new(StrVec::from(vec![
        "A", "A", "B", "B", "A", "A", "B", "B",
    ])));

    println!("Test data: 4 bins of width 1.0 each");
    println!("  Bin 1 [0-1): 2 × A (blue)");
    println!("  Bin 2 [1-2): 2 × B (red)");
    println!("  Bin 3 [2-3): 2 × A (blue)");
    println!("  Bin 4 [3-4]: 2 × B (red)");
    println!("Expected: 4 bars of EQUAL width, alternating blue-red-blue-red");

    Plot::new(Some(Box::new(df)))
        .title("Test Equal Bin Widths - All bars should be same width")
        .aes(|a| {
            a.x("value");
            a.fill("group");
        })
        .geom_histogram_with(|layer| { layer.geom.bins(4).position(Position::Stack); })
        .save("test_equal_bins.png", 800, 600)?;

    println!("\nSaved test_equal_bins.png");
    println!("If bars are NOT equal width, there's still a bug.");

    Ok(())
}
