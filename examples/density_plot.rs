use gogplot_rs::plot::{GeomBuilder, Plot};
use gogplot_rs::theme::color;
use gogplot_rs::utils::dataframe::{DataFrame, FloatVec};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("Creating density plot examples...");
    
    // Example 1: Simple density plot using geom_density
    example_basic_density()?;
    
    // Example 2: Comparing bandwidth adjustment
    example_bandwidth_adjustment()?;
    
    println!("All density examples completed!");
    Ok(())
}

fn example_basic_density() -> Result<(), Box<dyn Error>> {
    // Generate some sample data
    let values: Vec<f64> = vec![
        -2.5, -2.0, -1.8, -1.5, -1.2, -1.0, -0.8, -0.5, -0.3, -0.1,
        0.0, 0.1, 0.3, 0.5, 0.8, 1.0, 1.2, 1.5, 1.8, 2.0, 2.5,
        -1.9, -1.3, -0.9, -0.4, 0.2, 0.6, 0.9, 1.3, 1.9,
        -1.7, -1.1, -0.6, 0.4, 1.1, 1.7,
    ];
    
    // Create dataframe with the data
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(values)));
    
    let plot = Plot::new(Some(Box::new(df)))
        .title("Kernel Density Estimate")
        .aes(|a| {
            a.x("x");
        })
        .geom_density_with(|geom| geom.size(2.0).color(color::BLUE));
    
    plot.save("density_basic.png", 800, 600)?;
    
    println!("Saved density_basic.png");
    Ok(())
}

fn example_bandwidth_adjustment() -> Result<(), Box<dyn Error>> {
    // Generate data
    let values: Vec<f64> = vec![
        -2.5, -2.0, -1.8, -1.5, -1.2, -1.0, -0.8, -0.5, -0.3, -0.1,
        0.0, 0.1, 0.3, 0.5, 0.8, 1.0, 1.2, 1.5, 1.8, 2.0, 2.5,
        -1.9, -1.3, -0.9, -0.4, 0.2, 0.6, 0.9, 1.3, 1.9,
        -1.7, -1.1, -0.6, 0.4, 1.1, 1.7,
    ];
    
    // Create dataframe
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(values)));
    
    let plot = Plot::new(Some(Box::new(df)))
        .title("Density with adjust = 0.5 (Narrower Bandwidth)")
        .aes(|a| {
            a.x("x");
        })
        .geom_density_with(|geom| {
            geom.size(2.0)
                .color(color::RED)
                .adjust(0.5)
        });
    
    plot.save("density_bandwidth.png", 800, 600)?;
    
    println!("Saved density_bandwidth.png");
    Ok(())
}
