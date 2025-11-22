// Debug stacked histogram to understand the data flow

use gogplot::geom::bar::GeomBar;
use gogplot::geom::IntoLayer;
use gogplot::layer::Position;
use gogplot::plot::Plot;
use gogplot::stat::bin::BinStrategy;
use gogplot::layer::Stat;
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};
use gogplot::theme::Color;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating debug stacked histogram with simple data...");

    // Simpler example: 6 values, 2 groups, 3 bins
    // Values: 1, 2 in bin 1; 3, 4 in bin 2; 5, 6 in bin 3
    let mut df = DataFrame::new();
    df.add_column("value", Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0])));
    df.add_column("group", Box::new(StrVec::from(vec!["A", "B", "A", "B", "A", "B"])));

    println!("Input data:");
    println!("  value: [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]");
    println!("  group: [A, B, A, B, A, B]");
    println!("Expected with 3 bins:");
    println!("  Bin 1 (1-2.33): A=1, B=1 -> stacked height=2");
    println!("  Bin 2 (2.33-3.67): A=1, B=1 -> stacked height=2");
    println!("  Bin 3 (3.67-5): A=1, B=1 -> stacked height=2");

    Plot::new(Some(Box::new(df)))
        .title("Debug Stacked Histogram (Simple)")
        .aes(|a| {
            a.x("value");
            a.fill("group");
        })
        .scale_fill_manual(vec![
            Color(100, 150, 200, 255),  // Blue for A
            Color(220, 120, 100, 255),  // Red for B
        ])
        .layer(
            GeomBar::new()
                .stat(Stat::Bin(BinStrategy::Count(3).into()))
                .position(Position::Stack)
                .into_layer(),
        )
        .save("debug_stacked_histogram.png", 800, 600)?;

    println!("Saved debug_stacked_histogram.png");
    println!("Each of 3 bins should show blue (A) on bottom, red (B) on top");

    Ok(())
}
