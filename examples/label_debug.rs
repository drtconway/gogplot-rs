use gogplot::prelude::*;
use gogplot::theme::Color;
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing label box sizing...");
    
    // Test with various padding values
    let mut plot = Plot::new(None)
        .title("Label Box Sizing Test");

    for (i, (pad, label)) in [(0.0, "No Padding"), (2.0, "Padding=2"), (5.0, "Padding=5"), (10.0, "Padding=10")].iter().enumerate() {
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(FloatVec(vec![2.0])));
        df.add_column("y", Box::new(FloatVec(vec![4.0 - i as f64])));
        df.add_column("label", Box::new(StrVec(vec![label.to_string()])));
        
        plot = plot.geom_label_with(|l| {
            l.data(Box::new(df));
            l.aes(|a| {
                a.x("x");
                a.y("y");
                a.label("label");
            });
            l.geom.padding(*pad).radius(0.0); // No rounding to see exact box
        });
    }

    plot.save("label_debug.png", 800, 600)?;
    println!("Saved label_debug.png");
    Ok(())
}
