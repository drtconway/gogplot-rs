use gogplot::prelude::*;
use gogplot::theme::Color;
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating label examples...");
    
    example_basic_labels()?;
    example_text_vs_labels()?;
    example_custom_labels()?;
    
    println!("All label examples completed!");
    Ok(())
}

fn example_basic_labels() -> Result<(), Box<dyn std::error::Error>> {
    // Create data with labels
    let x_vals = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let y_vals = vec![2.0, 3.5, 3.0, 4.5, 4.0];
    let labels = vec!["A", "B", "C", "D", "E"];

    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(x_vals)));
    df.add_column("y", Box::new(FloatVec(y_vals)));
    df.add_column("label", Box::new(StrVec(labels.iter().map(|s| s.to_string()).collect())));

    let plot = Plot::new(Some(Box::new(df)))
        .title("Basic Labels with Background Boxes")
        .aes(|a| {
            a.x("x");
            a.y("y");
            a.label("label");
        })
        .geom_point()
        .geom_label();

    plot.save("labels_basic.png", 800, 600)?;
    println!("Saved labels_basic.png");
    Ok(())
}

fn example_text_vs_labels() -> Result<(), Box<dyn std::error::Error>> {
    // Compare text and labels - text version
    let x_vals = vec![1.0, 2.0, 3.0];
    let labels = vec!["Plain", "Text", "Example"];

    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(x_vals.clone())));
    df.add_column("y", Box::new(FloatVec(vec![3.0, 3.0, 3.0])));
    df.add_column("label", Box::new(StrVec(labels.iter().map(|s| s.to_string()).collect())));

    let plot = Plot::new(Some(Box::new(df)))
        .title("Plain Text (no background)")
        .aes(|a| {
            a.x("x");
            a.y("y");
            a.label("label");
        })
        .geom_text_with(|l| {
            l.geom.size(14.0);
        });

    plot.save("labels_text_plain.png", 800, 600)?;

    // Labels version with background
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(x_vals)));
    df.add_column("y", Box::new(FloatVec(vec![3.0, 3.0, 3.0])));
    df.add_column("label", Box::new(StrVec(labels.iter().map(|s| s.to_string()).collect())));

    let plot = Plot::new(Some(Box::new(df)))
        .title("Labels (with background box)")
        .aes(|a| {
            a.x("x");
            a.y("y");
            a.label("label");
        })
        .geom_label_with(|l| {
            l.geom.size(14.0);
        });

    plot.save("labels_with_background.png", 800, 600)?;
    println!("Saved text vs label comparison");
    Ok(())
}

fn example_custom_labels() -> Result<(), Box<dyn std::error::Error>> {
    // Create data
    let x_vals = vec![1.0, 2.0, 3.0, 4.0];
    let y_vals = vec![2.5, 3.5, 2.0, 4.0];
    let labels = vec!["Red", "Blue", "Green", "Orange"];

    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(x_vals)));
    df.add_column("y", Box::new(FloatVec(y_vals)));
    df.add_column("label", Box::new(StrVec(labels.iter().map(|s| s.to_string()).collect())));

    let plot = Plot::new(Some(Box::new(df)))
        .title("Customized Labels")
        .aes(|a| {
            a.x("x");
            a.y("y");
            a.label("label");
        })
        .geom_point_with(|l| {
            l.geom.size(6.0);
        })
        .geom_label_with(|l| {
            l.geom
                .size(11.0)
                .fill(Color::rgb(255, 250, 205))  // Light yellow
                .padding(4.0)
                .radius(4.0)
                .vjust(-0.5);  // Position labels above points
        });

    plot.save("labels_custom.png", 800, 600)?;
    println!("Saved labels_custom.png");
    Ok(())
}
