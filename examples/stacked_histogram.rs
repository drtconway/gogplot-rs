use gogplot::geom::bar::GeomBar;
use gogplot::geom::IntoLayer;
use gogplot::layer::Position;
use gogplot::plot::Plot;
use gogplot::stat::bin::BinStrategy;
use gogplot::layer::Stat;
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};
use gogplot::theme::Color;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating stacked histogram example...");

    // Create sample data with three overlapping groups
    let mut values = Vec::new();
    let mut groups = Vec::new();

    // Group A
    for i in 0..100 {
        values.push(5.0 + (i as f64 % 7.0) * 0.5);
        groups.push("A");
    }

    // Group B
    for i in 0..80 {
        values.push(7.0 + (i as f64 % 6.0) * 0.6);
        groups.push("B");
    }

    // Group C
    for i in 0..60 {
        values.push(6.0 + (i as f64 % 5.0) * 0.4);
        groups.push("C");
    }

    let mut df = DataFrame::new();
    df.add_column("value", Box::new(FloatVec(values)));
    df.add_column("group", Box::new(StrVec::from(groups)));

    // Create stacked histogram
    let plot = Plot::new(Some(Box::new(df)))
        .title("Stacked Histogram")
        .aes(|a| {
            a.x("value");
            a.fill("group");
        })
        .scale_fill_manual(vec![
            Color(100, 150, 200, 255),  // Blue for group A
            Color(220, 120, 100, 255),  // Red for group B
            Color(130, 190, 130, 255),  // Green for group C
        ])
        .layer(
            GeomBar::new()
                .stat(Stat::Bin(BinStrategy::Count(15)))
                .position(Position::Stack)
                .into_layer(),
        );

    plot.save("stacked_histogram.png", 800, 600)?;
    println!("Saved stacked_histogram.png");

    Ok(())
}
