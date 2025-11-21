// Test histogram binning with multiple grouping aesthetics
use gogplot::aesthetics::Aesthetic;
use gogplot::plot::{GeomBuilder, Plot};
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing binning with multiple grouping aesthetics...");

    // Create data with two grouping dimensions
    let mut df = DataFrame::new();
    
    // Values: mix of ranges for different groups
    let mut values = Vec::new();
    let mut colors = Vec::new();
    let mut shapes = Vec::new();
    
    // Color=Red, Shape=Circle: values around 2-4
    for i in 0..50 {
        values.push(2.0 + (i as f64) / 25.0);
        colors.push("Red");
        shapes.push("Circle");
    }
    
    // Color=Red, Shape=Square: values around 4-6
    for i in 0..50 {
        values.push(4.0 + (i as f64) / 25.0);
        colors.push("Red");
        shapes.push("Square");
    }
    
    // Color=Blue, Shape=Circle: values around 3-5
    for i in 0..50 {
        values.push(3.0 + (i as f64) / 25.0);
        colors.push("Blue");
        shapes.push("Circle");
    }
    
    // Color=Blue, Shape=Square: values around 5-7
    for i in 0..50 {
        values.push(5.0 + (i as f64) / 25.0);
        colors.push("Blue");
        shapes.push("Square");
    }
    
    df.add_column("value", Box::new(FloatVec(values)));
    df.add_column("color_group", Box::new(StrVec::from(colors)));
    df.add_column("shape_group", Box::new(StrVec::from(shapes)));

    // Test: Both Color and Shape as grouping aesthetics
    // This should create 4 distinct groups: Red-Circle, Red-Square, Blue-Circle, Blue-Square
    Plot::new(Some(Box::new(df)))
        .aes(|a| {
            a.set_to_column(Aesthetic::X, "value");
            a.set_to_column(Aesthetic::Fill, "color_group");
            a.set_to_column(Aesthetic::Shape, "shape_group");
        })
        .geom_histogram_with(|g| g.bins(10).alpha(0.6))
        .title("Histogram with Multiple Grouping Aesthetics (Fill + Shape)")
        .save("histogram_multiple_groups.png", 900, 600)?;

    println!("✓ Saved histogram_multiple_groups.png");
    println!("\nExpected 4 groups:");
    println!("  - Red + Circle");
    println!("  - Red + Square");
    println!("  - Blue + Circle");
    println!("  - Blue + Square");
    println!("\nEach group should be binned separately but using the same bin boundaries.");

    Ok(())
}
