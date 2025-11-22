// Demonstrate Position::Identity vs Position::Stack
//
// This example shows how Position::Identity preserves original values
// without any adjustment, compared to Position::Stack which stacks values.

use gogplot::layer::{Position, Stat};
use gogplot::plot::{GeomBuilder, Plot};
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating position comparison examples...");

    // Create sample data with explicit y values
    let mut df1 = DataFrame::new();
    df1.add_column(
        "quarter",
        Box::new(StrVec::from(vec!["Q1", "Q1", "Q2", "Q2", "Q3", "Q3"])),
    );
    df1.add_column(
        "sales",
        Box::new(FloatVec(vec![10.0, 15.0, 12.0, 18.0, 14.0, 20.0])),
    );
    df1.add_column(
        "region",
        Box::new(StrVec::from(vec!["North", "South", "North", "South", "North", "South"])),
    );

    // Example 1: Position::Identity - bars at original y values (will overlap)
    Plot::new(Some(Box::new(df1.clone())))
        .title("Position::Identity - Bars Overlap at Original Values")
        .aes(|a| {
            a.x("quarter");
            a.y("sales");
            a.fill("region");
        })
        .geom_bar_with(|layer| {
            layer.geom.stat(Stat::Identity)
                .position(Position::Identity)
                .alpha(0.7); // Make semi-transparent to see overlap
        })
        .y_scale_with(|scale| scale.set_lower_bound(0.0))
        .save("position_identity.png", 800, 600)?;

    println!("Saved position_identity.png - bars overlap at original y values");

    // Example 2: Position::Stack - bars stacked on top of each other
    Plot::new(Some(Box::new(df1)))
        .title("Position::Stack - Bars Stacked on Top of Each Other")
        .aes(|a| {
            a.x("quarter");
            a.y("sales");
            a.fill("region");
        })
        .geom_bar_with(|layer| { layer.geom.stat(Stat::Identity).position(Position::Stack); })
        .y_scale_with(|scale| scale.set_lower_bound(0.0))
        .save("position_stack.png", 800, 600)?;

    println!("Saved position_stack.png - bars stacked to show total");

    // Example 3: Histogram with Position::Identity (groups overlap)
    let mut df2 = DataFrame::new();
    df2.add_column(
        "value",
        Box::new(FloatVec(vec![
            1.0, 1.2, 1.5, 1.8, 2.0, 2.2, 2.5, 2.8, 3.0, 3.2, 3.5, 3.8, 4.0, 4.2, 4.5, 4.8,
        ])),
    );
    df2.add_column(
        "group",
        Box::new(StrVec::from(vec![
            "A", "A", "A", "A", "B", "B", "B", "B", "A", "A", "A", "A", "B", "B", "B", "B",
        ])),
    );

    Plot::new(Some(Box::new(df2.clone())))
        .title("Histogram with Position::Identity - Groups Overlap")
        .aes(|a| {
            a.x("value");
            a.fill("group");
        })
        .geom_histogram_with(|layer| {
            layer.geom.bins(4)
                .position(Position::Identity)
                .alpha(0.6); // Semi-transparent to see overlap
        })
        .save("histogram_identity.png", 800, 600)?;

    println!("Saved histogram_identity.png - overlapping histograms");

    // Example 4: Histogram with Position::Stack
    Plot::new(Some(Box::new(df2)))
        .title("Histogram with Position::Stack - Groups Stacked")
        .aes(|a| {
            a.x("value");
            a.fill("group");
        })
        .geom_histogram_with(|layer| { layer.geom.bins(4).position(Position::Stack); })
        .save("histogram_stack.png", 800, 600)?;

    println!("Saved histogram_stack.png - stacked histograms");

    println!("\nComparison complete!");
    println!("- Position::Identity: Values unchanged, bars/bins overlap");
    println!("- Position::Stack: Values adjusted to stack on top of each other");

    Ok(())
}
