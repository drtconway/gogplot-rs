// Example: Dodged histograms with grouped data

use gogplot::layer::Position;
use gogplot::plot::{GeomBuilder, Plot};
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Sample data: measurements from two different groups
    let group_a_values = vec![
        1.2, 1.5, 1.8, 2.1, 2.3, 2.5, 2.7, 2.9, 3.1, 3.3,
        1.4, 1.6, 1.9, 2.2, 2.4, 2.6, 2.8, 3.0, 3.2, 3.4,
    ];
    let group_b_values = vec![
        1.8, 2.1, 2.4, 2.7, 2.9, 3.1, 3.3, 3.5, 3.7, 3.9,
        2.0, 2.3, 2.6, 2.8, 3.0, 3.2, 3.4, 3.6, 3.8, 4.0,
    ];

    // Combine into single dataset with group labels
    let mut values = Vec::new();
    let mut groups = Vec::new();
    
    for val in group_a_values {
        values.push(val);
        groups.push("Group A");
    }
    for val in group_b_values {
        values.push(val);
        groups.push("Group B");
    }

    let mut df = DataFrame::new();
    df.add_column("value", Box::new(FloatVec(values)));
    df.add_column("group", Box::new(StrVec::from(groups)));

    // Create dodged histogram
    let plot1 = Plot::new(Some(Box::new(df.clone())))
        .aes(|a| {
            a.x("value");
            a.fill("group");
        })
        .geom_histogram_with(|geom| geom.bins(8).position(Position::Dodge))
        .title("Dodged Histogram")
        .y_scale_with(|scale| scale.set_lower_bound(0.0));

    plot1.save("dodged_histogram.png", 800, 600)?;
    println!("Created dodged_histogram.png");

    // Create stacked histogram for comparison
    let plot2 = Plot::new(Some(Box::new(df.clone())))
        .aes(|a| {
            a.x("value");
            a.fill("group");
        })
        .geom_histogram_with(|geom| geom.bins(8).position(Position::Stack))
        .title("Stacked Histogram")
        .y_scale_with(|scale| scale.set_lower_bound(0.0));

    plot2.save("stacked_histogram.png", 800, 600)?;
    println!("Created stacked_histogram.png");

    // Create identity (overlapping) histogram for comparison
    let plot3 = Plot::new(Some(Box::new(df)))
        .aes(|a| {
            a.x("value");
            a.fill("group");
        })
        .geom_histogram_with(|geom| geom.bins(8).position(Position::Identity).alpha(0.6))
        .title("Overlapping Histogram")
        .y_scale_with(|scale| scale.set_lower_bound(0.0));

    plot3.save("overlapping_histogram.png", 800, 600)?;
    println!("Created overlapping_histogram.png");

    Ok(())
}
