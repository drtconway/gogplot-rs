use gogplot::plot::{GeomBuilder, Plot};
use gogplot::utils::dataframe::{DataFrame, FloatVec, IntVec};
use gogplot::utils::sp500::sp500;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data = sp500();
    
    // Get the close prices
    let close_col = data.get("close").unwrap();
    let close_values: Vec<f64> = close_col.iter_float().unwrap().collect();
    let n = close_values.len();
    
    // Create a new dataframe with an index for x-axis
    let mut df = DataFrame::new();
    df.add_column("index", Box::new(IntVec((0..n as i64).collect())));
    df.add_column("close", Box::new(FloatVec(close_values)));
    
    Plot::new(Some(Box::new(df)))
        .title("S&P 500 Closing Price with Linear Trend")
        .aes(|a| {
            a.x("index");
            a.y("close");
        })
        .geom_line_with(|layer| {
            layer.geom.color(gogplot::theme::color::DARKORANGE).size(1.5);
        })
        .geom_smooth()
        .save("sp500_smooth.png", 1200, 600)?;
    
    println!("Saved sp500_smooth.png");
    println!("The plot shows {} days of S&P 500 closing prices with a linear regression trend line.", n);
    
    Ok(())
}
