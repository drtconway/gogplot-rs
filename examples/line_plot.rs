use gogplot::plot::{GeomBuilder, Plot};
use gogplot::theme::color;
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating line plot example...");

    // Simple line plot
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 5.0])));
    df.add_column("y", Box::new(FloatVec(vec![2.0, 4.0, 3.0, 5.0, 6.0])));

    let plot = Plot::new(Some(Box::new(df)))
        .title("Simple Line Plot")
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_line_with(|layer| { layer.geom.size(2.0).color(color::BLUE); });
    plot.save("line_simple.png", 800, 600)?;

    // Grouped line plot
    let mut df = DataFrame::new();
    df.add_column(
        "x",
        Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 1.0, 2.0, 3.0, 4.0])),
    );
    df.add_column(
        "y",
        Box::new(FloatVec(vec![2.0, 3.0, 2.5, 4.0, 1.5, 2.0, 3.5, 3.0])),
    );
    df.add_column(
        "group",
        Box::new(StrVec::from(vec!["A", "A", "A", "A", "B", "B", "B", "B"])),
    );

    let plot = Plot::new(Some(Box::new(df)))
        .title("Grouped Line Plot")
        .aes(|a| {
            a.x("x");
            a.y("y");
            a.color("group");
            a.group("group");
        })
        .geom_line_with(|layer| { layer.geom.size(2.0); });
    plot.save("line_grouped.png", 800, 600)?;

    println!("Saved line_simple.png and line_grouped.png");

    Ok(())
}
