// Test fill scales with rectangles
use gogplot::aesthetics::Aesthetic;
use gogplot::plot::{GeomBuilder, Plot};
use gogplot::theme::Color;
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing fill scales with rectangles...");

    // Test 1: Categorical fill with default palette
    let mut df1 = DataFrame::new();
    df1.add_column("xmin", Box::new(FloatVec(vec![0.0, 1.0, 2.0, 0.0, 1.0, 2.0])));
    df1.add_column("xmax", Box::new(FloatVec(vec![0.9, 1.9, 2.9, 0.9, 1.9, 2.9])));
    df1.add_column("ymin", Box::new(FloatVec(vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0])));
    df1.add_column("ymax", Box::new(FloatVec(vec![0.9, 0.9, 0.9, 1.9, 1.9, 1.9])));
    df1.add_column("category", Box::new(StrVec::from(vec!["A", "B", "C", "A", "B", "C"])));

    Plot::new(Some(Box::new(df1)))
        .title("Rectangles with Categorical Fill (Default Palette)")
        .aes(|a| {
            a.set_to_column(Aesthetic::XBegin, "xmin");
            a.set_to_column(Aesthetic::XEnd, "xmax");
            a.set_to_column(Aesthetic::YBegin, "ymin");
            a.set_to_column(Aesthetic::YEnd, "ymax");
            a.set_to_column(Aesthetic::Fill, "category");
        })
        .geom_rect()
        .save("rect_fill_default.png", 600, 600)?;
    println!("Saved rect_fill_default.png");

    // Test 2: Categorical fill with manual colors
    let mut df2 = DataFrame::new();
    df2.add_column("xmin", Box::new(FloatVec(vec![0.0, 1.0, 2.0, 0.0, 1.0, 2.0])));
    df2.add_column("xmax", Box::new(FloatVec(vec![0.9, 1.9, 2.9, 0.9, 1.9, 2.9])));
    df2.add_column("ymin", Box::new(FloatVec(vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0])));
    df2.add_column("ymax", Box::new(FloatVec(vec![0.9, 0.9, 0.9, 1.9, 1.9, 1.9])));
    df2.add_column("category", Box::new(StrVec::from(vec!["A", "B", "C", "A", "B", "C"])));

    Plot::new(Some(Box::new(df2)))
        .title("Rectangles with Manual Fill Colors")
        .aes(|a| {
            a.set_to_column(Aesthetic::XBegin, "xmin");
            a.set_to_column(Aesthetic::XEnd, "xmax");
            a.set_to_column(Aesthetic::YBegin, "ymin");
            a.set_to_column(Aesthetic::YEnd, "ymax");
            a.set_to_column(Aesthetic::Fill, "category");
        })
        .scale_fill_manual(vec![
            Color::rgb(230, 97, 0),   // orange
            Color::rgb(0, 158, 115),  // teal
            Color::rgb(204, 121, 167), // pink
        ])
        .geom_rect()
        .save("rect_fill_manual.png", 600, 600)?;
    println!("Saved rect_fill_manual.png");

    // Test 3: Continuous fill scale
    let mut df3 = DataFrame::new();
    df3.add_column("xmin", Box::new(FloatVec(vec![0.0, 1.0, 2.0, 3.0, 4.0])));
    df3.add_column("xmax", Box::new(FloatVec(vec![0.9, 1.9, 2.9, 3.9, 4.9])));
    df3.add_column("ymin", Box::new(FloatVec(vec![0.0; 5])));
    df3.add_column("ymax", Box::new(FloatVec(vec![0.9; 5])));
    df3.add_column("value", Box::new(FloatVec(vec![0.0, 1.0, 2.0, 3.0, 4.0])));

    Plot::new(Some(Box::new(df3)))
        .title("Rectangles with Continuous Fill (Blue to Red)")
        .aes(|a| {
            a.set_to_column(Aesthetic::XBegin, "xmin");
            a.set_to_column(Aesthetic::XEnd, "xmax");
            a.set_to_column(Aesthetic::YBegin, "ymin");
            a.set_to_column(Aesthetic::YEnd, "ymax");
            a.set_to_column(Aesthetic::Fill, "value");
        })
        .scale_fill_continuous(vec![
            Color::rgb(0, 0, 255),   // blue
            Color::rgb(255, 0, 0),   // red
        ])
        .geom_rect()
        .save("rect_fill_continuous.png", 600, 300)?;
    println!("Saved rect_fill_continuous.png");

    println!("\nAll fill scale tests completed!");
    Ok(())
}
