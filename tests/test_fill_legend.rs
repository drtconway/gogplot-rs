// Simple test for fill legend
use gogplot::aesthetics::Aesthetic;
use gogplot::plot::{GeomBuilder, Plot};
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing fill legend generation...");

    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0])));
    df.add_column("y", Box::new(FloatVec(vec![1.0, 2.0, 3.0])));
    df.add_column("category", Box::new(StrVec::from(vec!["A", "B", "C"])));

    Plot::new(Some(Box::new(df)))
        .title("Test Fill Legend")
        .aes(|a| {
            a.set_to_column(Aesthetic::X, "x");
            a.set_to_column(Aesthetic::Y, "y");
            a.set_to_column(Aesthetic::Fill, "category");
        })
        .geom_point()
        .save("test_fill_legend.png", 600, 400)?;
    
    println!("Saved test_fill_legend.png");
    Ok(())
}
