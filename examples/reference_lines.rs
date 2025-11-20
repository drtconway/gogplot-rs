use gogplot_rs::plot::{GeomBuilder, Plot};
use gogplot_rs::theme::color;
use gogplot_rs::utils::dataframe::{DataFrame, FloatVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating reference lines example...");

    // Create sample scatter data
    let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let y = vec![2.3, 3.1, 2.8, 4.5, 3.9, 5.2, 4.7, 6.1, 5.5, 6.8];

    // Example 1: Horizontal reference line at y=4.0
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(x.clone())));
    df.add_column("y", Box::new(FloatVec(y.clone())));
    
    let plot = Plot::new(Some(Box::new(df)))
        .title("Horizontal Reference Line")
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point_with(|geom| geom.size(4.0).color(color::BLUE))
        .geom_hline_with(4.0, |geom| geom.color(color::RED).size(2.0).linetype("-"));
    plot.save("reference_hline.png", 800, 600)?;
    println!("Saved reference_hline.png");

    // Example 2: Vertical reference line at x=5.0
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(x.clone())));
    df.add_column("y", Box::new(FloatVec(y.clone())));
    
    let plot = Plot::new(Some(Box::new(df)))
        .title("Vertical Reference Line")
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point_with(|geom| geom.size(4.0).color(color::BLUE))
        .geom_vline_with(5.0, |geom| geom.color(color::GREEN).size(2.0).linetype("-"));
    plot.save("reference_vline.png", 800, 600)?;
    println!("Saved reference_vline.png");

    // Example 3: Multiple reference lines with different styles
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(x.clone())));
    df.add_column("y", Box::new(FloatVec(y.clone())));
    
    let plot = Plot::new(Some(Box::new(df)))
        .title("Multiple Reference Lines")
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point_with(|geom| geom.size(4.0).color(color::BLUE))
        .geom_hline_with(3.0, |geom| geom.color(color::BLUE).size(1.5).linetype(".").alpha(0.6))
        .geom_hline_with(5.0, |geom| geom.color(color::GREEN).size(1.5).linetype(".").alpha(0.6))
        .geom_vline_with(5.5, |geom| geom.color(color::RED).size(2.0).linetype("-.").alpha(0.8));
    plot.save("reference_multiple.png", 800, 600)?;
    println!("Saved reference_multiple.png");

    // Example 4: Marking mean and median
    let mean_x = 5.5;
    let mean_y = 4.49;
    
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(x)));
    df.add_column("y", Box::new(FloatVec(y)));
    
    let plot = Plot::new(Some(Box::new(df)))
        .title("Reference Lines for Mean")
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point_with(|geom| geom.size(4.0).color(color::BLUE))
        .geom_hline_with(mean_y, |geom| {
            geom.color(color::RED)
                .size(2.0)
                .linetype("- -")
                .alpha(0.7)
        })
        .geom_vline_with(mean_x, |geom| {
            geom.color(color::RED)
                .size(2.0)
                .linetype("- -")
                .alpha(0.7)
        });
    plot.save("reference_mean.png", 800, 600)?;
    println!("Saved reference_mean.png");

    Ok(())
}
