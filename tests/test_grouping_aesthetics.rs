// Test that different grouping aesthetics trigger grouped binning
use gogplot::aesthetics::Aesthetic;
use gogplot::plot::{GeomBuilder, Plot};
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn create_test_data() -> DataFrame {
    let mut df = DataFrame::new();
    df.add_column(
        "value",
        Box::new(FloatVec(vec![
            1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5, // Group A
            2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5, 6.0, 6.5, // Group B
        ])),
    );
    df.add_column(
        "category",
        Box::new(StrVec::from(vec![
            "A", "A", "A", "A", "A", "A", "A", "A", "A", "A",
            "B", "B", "B", "B", "B", "B", "B", "B", "B", "B",
        ])),
    );
    df
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing grouped binning with different aesthetics...");

    // Test 1: Grouping by Fill aesthetic
    Plot::new(Some(Box::new(create_test_data())))
        .aes(|a| {
            a.set_to_column(Aesthetic::X, "value");
            a.set_to_column(Aesthetic::Fill, "category");
        })
        .geom_histogram_with(|g| g.bins(5).alpha(0.6))
        .title("Grouped by Fill Aesthetic")
        .save("grouped_by_fill.png", 800, 600)?;
    println!("✓ Saved grouped_by_fill.png");

    // Test 2: Grouping by Color aesthetic
    Plot::new(Some(Box::new(create_test_data())))
        .aes(|a| {
            a.set_to_column(Aesthetic::X, "value");
            a.set_to_column(Aesthetic::Color, "category");
        })
        .geom_histogram_with(|g| g.bins(5))
        .title("Grouped by Color Aesthetic")
        .save("grouped_by_color.png", 800, 600)?;
    println!("✓ Saved grouped_by_color.png");

    // Test 3: Grouping by explicit Group aesthetic
    Plot::new(Some(Box::new(create_test_data())))
        .aes(|a| {
            a.set_to_column(Aesthetic::X, "value");
            a.set_to_column(Aesthetic::Group, "category");
            a.set_to_column(Aesthetic::Fill, "category");
        })
        .geom_histogram_with(|g| g.bins(5).alpha(0.7))
        .title("Grouped by Group + Fill Aesthetics")
        .save("grouped_by_group.png", 800, 600)?;
    println!("✓ Saved grouped_by_group.png");

    println!("\nAll grouping aesthetic tests completed!");
    Ok(())
}
