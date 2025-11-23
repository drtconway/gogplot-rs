use gogplot::plot::{GeomBuilder, Plot};
use gogplot::utils::dataframe::{DataFrame, FloatVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create simple linear data with some deterministic noise
    let mut x_vals = Vec::new();
    let mut y_vals = Vec::new();
    
    for i in 0..50 {
        let x = i as f64 * 0.2;
        // Add deterministic noise using sine function
        let noise = (i as f64 * 0.5).sin() * 2.0;
        let y = 2.0 + 3.0 * x + noise;  // y = 2 + 3x + noise
        x_vals.push(x);
        y_vals.push(y);
    }
    
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(x_vals)));
    df.add_column("y", Box::new(FloatVec(y_vals)));
    
    // Create plot with smooth line only (points commented out for now)
    Plot::new(Some(Box::new(df)))
        .title("Linear Smooth Example")
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        // .geom_point()
        .geom_smooth()
        .save("smooth_example.png", 800, 600)?;
    
    println!("Saved smooth_example.png");
    
    Ok(())
}
