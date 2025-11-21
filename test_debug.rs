use gogplot::geom::bar::GeomBar;
use gogplot::geom::IntoLayer;
use gogplot::layer::Position;
use gogplot::plot::Plot;
use gogplot::stat::bin::BinStrategy;
use gogplot::layer::Stat;
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};
use gogplot::theme::Color;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating stacked histogram with debug...");

    let mut values = Vec::new();
    let mut groups = Vec::new();
    for i in 0..10 {
        values.push(5.0 + (i as f64));
        groups.push("A");
    }
    for i in 0..10 {
        values.push(7.0 + (i as f64));
        groups.push("B");
    }

    let mut df = DataFrame::new();
    df.add_column("value", Box::new(FloatVec(values)));
    df.add_column("group", Box::new(StrVec::from(groups)));

    println!("Input data: {} rows", df.len());
    println!("Columns: {:?}", df.column_names());

    let plot = Plot::new(Some(Box::new(df)))
        .title("Debug Stacked Histogram")
        .aes(|a| {
            a.x("value");
            a.fill("group");
        })
        .scale_fill_manual(vec![
            Color(100, 150, 200, 255),
            Color(220, 120, 100, 255),
        ])
        .layer(
            GeomBar::new()
                .stat(Stat::Bin(BinStrategy::Count(5)))
                .position(Position::Stack)
                .into_layer(),
        );

    plot.save("test_debug.png", 800, 600)?;
    println!("Saved test_debug.png");

    Ok(())
}
