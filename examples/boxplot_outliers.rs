use gogplot::plot::{GeomBuilder, Plot};
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create test data with clear outliers
    // Group A: [1, 2, 3, 4, 5] - no outliers
    // Group B: [10, 11, 12, 13, 14, 30] - 30 is an outlier (way above Q3 + 1.5*IQR)
    // Group C: [20, 21, 22, 23, 24, 0] - 0 is an outlier (way below Q1 - 1.5*IQR)
    
    let mut df = DataFrame::new();
    df.add_column("group", Box::new(StrVec(vec![
        "A".to_string(), "A".to_string(), "A".to_string(), "A".to_string(), "A".to_string(),
        "B".to_string(), "B".to_string(), "B".to_string(), "B".to_string(), "B".to_string(), "B".to_string(),
        "C".to_string(), "C".to_string(), "C".to_string(), "C".to_string(), "C".to_string(), "C".to_string(),
    ])));
    df.add_column("value", Box::new(FloatVec(vec![
        1.0, 2.0, 3.0, 4.0, 5.0,
        10.0, 11.0, 12.0, 13.0, 14.0, 30.0,  // 30 should be an outlier
        20.0, 21.0, 22.0, 23.0, 24.0, 0.0,   // 0 should be an outlier
    ])));
    
    Plot::new(Some(Box::new(df)))
        .title("Boxplot with Outliers Test")
        .aes(|a| {
            a.x("group");
            a.y("value");
            a.fill("group");
        })
        .geom_boxplot()
        .save("boxplot_outliers_test.png", 800, 600)?;
    
    println!("Saved boxplot_outliers_test.png");
    println!("\nExpected outliers:");
    println!("  Group A: none");
    println!("  Group B: 30 (upper outlier)");
    println!("  Group C: 0 (lower outlier)");
    
    Ok(())
}
