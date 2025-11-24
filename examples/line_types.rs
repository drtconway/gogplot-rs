use gogplot::prelude::*;
use gogplot::theme::color;
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating line types example...");
    
    // Create data with multiple series
    let x_vals = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let y1 = vec![1.0, 1.0, 1.0, 1.0, 1.0];  // Solid
    let y2 = vec![2.0, 2.0, 2.0, 2.0, 2.0];  // Dashed
    let y3 = vec![3.0, 3.0, 3.0, 3.0, 3.0];  // Dotted
    let y4 = vec![4.0, 4.0, 4.0, 4.0, 4.0];  // DashDot
    let y5 = vec![5.0, 5.0, 5.0, 5.0, 5.0];  // LongDash

    // Flatten for long format
    let x_all: Vec<f64> = x_vals.iter().cycle().take(25).copied().collect();
    let y_all: Vec<f64> = y1.iter()
        .chain(y2.iter())
        .chain(y3.iter())
        .chain(y4.iter())
        .chain(y5.iter())
        .copied()
        .collect();
    let groups = vec!["Solid"; 5]
        .into_iter()
        .chain(vec!["Dashed"; 5])
        .chain(vec!["Dotted"; 5])
        .chain(vec!["DashDot"; 5])
        .chain(vec!["LongDash"; 5])
        .collect::<Vec<_>>();

    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(x_all)));
    df.add_column("y", Box::new(FloatVec(y_all)));
    df.add_column("linetype", Box::new(StrVec::from(groups)));

    let plot = Plot::new(Some(Box::new(df)))
        .title("LineType Examples")
        .aes(|a| {
            a.x("x");
            a.y("y");
            a.group("linetype");
            a.linetype("linetype");
        })
        .geom_line_with(|layer| {
            layer.geom.size(2.0).color(color::STEELBLUE);
        });

    plot.save("line_types_demo.png", 800, 600)?;
    println!("Saved line_types_demo.png");
    Ok(())
}
