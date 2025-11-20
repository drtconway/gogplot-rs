use gogplot_rs::plot::{GeomBuilder, Plot};
use gogplot_rs::guide::{AxisGuide, Guides};
use gogplot_rs::utils::dataframe::{DataFrame, FloatVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating axis position examples...");

    // Example 1: Default positions (bottom X, left Y)
    let mut df1 = DataFrame::new();
    df1.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 5.0])));
    df1.add_column("y", Box::new(FloatVec(vec![2.1, 3.9, 3.2, 5.8, 6.5])));
    
    let plot1 = Plot::new(Some(Box::new(df1)))
        .title("Default Axes (Bottom X, Left Y)")
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point();

    plot1.save("axis_default.png", 800, 600)?;
    println!("Saved axis_default.png");

    // Example 2: Top X axis
    let mut df2 = DataFrame::new();
    df2.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 5.0])));
    df2.add_column("y", Box::new(FloatVec(vec![2.1, 3.9, 3.2, 5.8, 6.5])));
    
    let plot2 = Plot::new(Some(Box::new(df2)))
        .title("Top X Axis")
        .guides(Guides::new()
            .x_axis(AxisGuide::x().top()))
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point();

    plot2.save("axis_top_x.png", 800, 600)?;
    println!("Saved axis_top_x.png");

    // Example 3: Right Y axis
    let mut df3 = DataFrame::new();
    df3.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 5.0])));
    df3.add_column("y", Box::new(FloatVec(vec![2.1, 3.9, 3.2, 5.8, 6.5])));
    
    let plot3 = Plot::new(Some(Box::new(df3)))
        .title("Right Y Axis")
        .guides(Guides::new()
            .y_axis(AxisGuide::y().right()))
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point();

    plot3.save("axis_right_y.png", 800, 600)?;
    println!("Saved axis_right_y.png");

    // Example 4: Both axes on opposite sides (top X, right Y)
    let mut df4 = DataFrame::new();
    df4.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 5.0])));
    df4.add_column("y", Box::new(FloatVec(vec![2.1, 3.9, 3.2, 5.8, 6.5])));
    
    let plot4 = Plot::new(Some(Box::new(df4)))
        .title("Top X and Right Y Axes")
        .guides(Guides::new()
            .x_axis(AxisGuide::x().top())
            .y_axis(AxisGuide::y().right()))
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point();

    plot4.save("axis_top_right.png", 800, 600)?;
    println!("Saved axis_top_right.png");

    Ok(())
}
