// Example: Line plot with points overlay
// Demonstrates combining geom_line and geom_point to show both trend and individual data points

use gogplot::guide::{AxisGuide, Guides};
use gogplot::plot::{GeomBuilder, Plot};
use gogplot::theme::color;
use gogplot::utils::dataframe::{DataFrame, FloatVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sample data: time series measurements
    let time = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let temperature = vec![20.0, 21.5, 23.0, 22.0, 24.5, 26.0, 25.0, 23.5, 22.0, 20.5, 19.0];

    let mut df = DataFrame::new();
    df.add_column("time", Box::new(FloatVec(time)));
    df.add_column("temperature", Box::new(FloatVec(temperature)));

    // Create plot with both line and points
    let plot = Plot::new(Some(Box::new(df)))
        .aes(|a| {
            a.x("time");
            a.y("temperature");
        })
        .geom_line_with(|geom| geom.size(2.0).color(color::BLUE))
        .geom_point_with(|geom| geom.size(5.0).color(color::FIREBRICK))
        .title("Temperature Over Time")
        .guides(
            Guides::new()
                .x_axis(AxisGuide::x().title("Time (hours)"))
                .y_axis(AxisGuide::y().title("Temperature (°C)"))
        );

    plot.save("line_with_points.png", 800, 600)?;
    println!("Created line_with_points.png");

    Ok(())
}
