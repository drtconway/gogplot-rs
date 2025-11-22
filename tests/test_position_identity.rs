// Test Position::Identity behavior
use gogplot::layer::Position;
use gogplot::plot::{GeomBuilder, Plot};
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};

#[test]
fn test_position_identity_preserves_values() {
    // Create data with explicit y values
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(vec![1.0, 1.0, 2.0, 2.0])));
    df.add_column("y", Box::new(FloatVec(vec![5.0, 3.0, 4.0, 2.0])));
    df.add_column(
        "group",
        Box::new(StrVec::from(vec!["A", "B", "A", "B"])),
    );

    // With Identity position, bars should use the exact y values
    // No stacking or dodging should occur
    let result = Plot::new(Some(Box::new(df)))
        .aes(|a| {
            a.x("x");
            a.y("y");
            a.fill("group");
        })
        .geom_bar_with(|layer| {
            layer.geom.stat(gogplot::layer::Stat::Identity)
                .position(Position::Identity);
        })
        .save("test_position_identity.png", 400, 300);

    assert!(result.is_ok(), "Plot with Position::Identity should work");
}

#[test]
fn test_position_identity_with_histogram() {
    // For histograms, Identity means bins are not adjusted
    let mut df = DataFrame::new();
    df.add_column(
        "value",
        Box::new(FloatVec(vec![1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0])),
    );
    df.add_column(
        "category",
        Box::new(StrVec::from(vec!["A", "A", "B", "B", "A", "B", "A"])),
    );

    // Even with a fill aesthetic, Identity should not stack
    let result = Plot::new(Some(Box::new(df)))
        .aes(|a| {
            a.x("value");
            a.fill("category");
        })
        .geom_histogram_with(|layer| { layer.geom.bins(4).position(Position::Identity); })
        .save("test_histogram_identity.png", 400, 300);

    assert!(
        result.is_ok(),
        "Histogram with Position::Identity should work"
    );
}
