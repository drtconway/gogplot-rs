use gogplot_rs::aesthetics::{Aesthetic, AesValue};
use gogplot_rs::geom::point::GeomPoint;
use gogplot_rs::plot::Plot;
use gogplot_rs::utils::dataframe::{DataFrame, FloatVec, IntVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create some sample data
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(IntVec(vec![1, 2, 3, 4, 5])));
    df.add_column("y", Box::new(FloatVec(vec![2.0, 4.0, 3.0, 5.0, 6.0])));

    // Create a point geom with blue color and size 5
    let geom = GeomPoint::default()
        .size(5.0)
        .color(0, 0, 255, 255); // Blue (RGBA)
    
    // Create a layer and map x, y aesthetics to data columns
    let mut layer = geom.into_layer();
    layer.mapping.set(Aesthetic::X, AesValue::Column("x".to_string()));
    layer.mapping.set(Aesthetic::Y, AesValue::Column("y".to_string()));

    // Create a plot - scales and labels are automatically generated from column names
    let plot = Plot::new(Some(Box::new(df)))
        .title("Simple Scatter Plot")
        .layer(layer);

    // Save to a file
    plot.save("scatter_plot.png", 800, 600)?;
    
    println!("Plot saved to scatter_plot.png");
    
    Ok(())
}
