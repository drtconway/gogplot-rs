use gogplot::prelude::*;
use gogplot::theme::{color, Color};
use gogplot::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating comprehensive label examples...");
    
    example_label_positioning()?;
    example_label_styling()?;
    
    println!("All comprehensive label examples completed!");
    Ok(())
}

fn example_label_positioning() -> Result<(), Box<dyn std::error::Error>> {
    // Demonstrate different label positions using hjust and vjust
    let x_vals = vec![2.0, 2.0, 2.0, 2.0, 2.0];
    let y_vals = vec![5.0, 4.0, 3.0, 2.0, 1.0];
    let labels = vec!["left (0.0)", "center (0.5)", "right (1.0)", "above (vjust=-0.2)", "below (vjust=1.2)"];
    
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(x_vals)));
    df.add_column("y", Box::new(FloatVec(y_vals)));
    df.add_column("label", Box::new(StrVec(labels.iter().map(|s| s.to_string()).collect())));

    let plot = Plot::new(Some(Box::new(df.clone())))
        .title("Label Positioning with hjust and vjust")
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point_with(|l| {
            l.geom.size(4.0).color(color::RED);
        });

    // Left-aligned label
    let mut df1 = DataFrame::new();
    df1.add_column("x", Box::new(FloatVec(vec![2.0])));
    df1.add_column("y", Box::new(FloatVec(vec![5.0])));
    df1.add_column("label", Box::new(StrVec(vec![labels[0].to_string()])));
    
    let plot = plot.geom_label_with(move |l| {
        l.data(Box::new(df1));
        l.aes(|a| {
            a.x("x");
            a.y("y");
            a.label("label");
        });
        l.geom.hjust(0.0).vjust(0.5).fill(Color::rgb(255, 240, 245));
    });

    // Center-aligned label
    let mut df2 = DataFrame::new();
    df2.add_column("x", Box::new(FloatVec(vec![2.0])));
    df2.add_column("y", Box::new(FloatVec(vec![4.0])));
    df2.add_column("label", Box::new(StrVec(vec![labels[1].to_string()])));
    
    let plot = plot.geom_label_with(|l| {
        l.data(Box::new(df2));
        l.aes(|a| {
            a.x("x");
            a.y("y");
            a.label("label");
        });
        l.geom.hjust(0.5).vjust(0.5).fill(Color::rgb(240, 248, 255));
    });

    // Right-aligned label
    let mut df3 = DataFrame::new();
    df3.add_column("x", Box::new(FloatVec(vec![2.0])));
    df3.add_column("y", Box::new(FloatVec(vec![3.0])));
    df3.add_column("label", Box::new(StrVec(vec![labels[2].to_string()])));
    
    let plot = plot.geom_label_with(move |l| {
        l.data(Box::new(df3));
        l.aes(|a| {
            a.x("x");
            a.y("y");
            a.label("label");
        });
        l.geom.hjust(1.0).vjust(0.5).fill(Color::rgb(240, 255, 240));
    });

    // Above point
    let mut df4 = DataFrame::new();
    df4.add_column("x", Box::new(FloatVec(vec![2.0])));
    df4.add_column("y", Box::new(FloatVec(vec![2.0])));
    df4.add_column("label", Box::new(StrVec(vec![labels[3].to_string()])));
    
    let plot = plot.geom_label_with(move |l| {
        l.data(Box::new(df4));
        l.aes(|a| {
            a.x("x");
            a.y("y");
            a.label("label");
        });
        l.geom.hjust(0.5).vjust(1.2).fill(Color::rgb(255, 255, 224));
    });

    // Below point
    let mut df5 = DataFrame::new();
    df5.add_column("x", Box::new(FloatVec(vec![2.0])));
    df5.add_column("y", Box::new(FloatVec(vec![1.0])));
    df5.add_column("label", Box::new(StrVec(vec![labels[4].to_string()])));
    
    let plot = plot.geom_label_with(move |l| {
        l.data(Box::new(df5));
        l.aes(|a| {
            a.x("x");
            a.y("y");
            a.label("label");
        });
        l.geom.hjust(0.5).vjust(-0.2).fill(Color::rgb(255, 240, 245));
    });

    plot.save("labels_positioning.png", 800, 600)?;
    println!("Saved labels_positioning.png");
    Ok(())
}

fn example_label_styling() -> Result<(), Box<dyn std::error::Error>> {
    // Demonstrate different label styles
    let plot = Plot::new(None)
        .title("Label Styling Options");

    // Default style
    let mut df1 = DataFrame::new();
    df1.add_column("x", Box::new(FloatVec(vec![1.0])));
    df1.add_column("y", Box::new(FloatVec(vec![3.0])));
    df1.add_column("label", Box::new(StrVec(vec!["Default".to_string()])));
    
    let plot = plot.geom_label_with(move |l| {
        l.data(Box::new(df1));
        l.aes(|a| {
            a.x("x");
            a.y("y");
            a.label("label");
        });
        // Use default settings
    });

    // Rounded corners
    let mut df2 = DataFrame::new();
    df2.add_column("x", Box::new(FloatVec(vec![2.0])));
    df2.add_column("y", Box::new(FloatVec(vec![3.0])));
    df2.add_column("label", Box::new(StrVec(vec!["Rounded".to_string()])));
    
    let plot = plot.geom_label_with(move |l| {
        l.data(Box::new(df2));
        l.aes(|a| {
            a.x("x");
            a.y("y");
            a.label("label");
        });
        l.geom.radius(8.0).fill(Color::rgb(173, 216, 230));
    });

    // Large padding
    let mut df3 = DataFrame::new();
    df3.add_column("x", Box::new(FloatVec(vec![3.0])));
    df3.add_column("y", Box::new(FloatVec(vec![3.0])));
    df3.add_column("label", Box::new(StrVec(vec!["Large Padding".to_string()])));
    
    let plot = plot.geom_label_with(move |l| {
        l.data(Box::new(df3));
        l.aes(|a| {
            a.x("x");
            a.y("y");
            a.label("label");
        });
        l.geom.padding(8.0).fill(Color::rgb(255, 218, 185));
    });

    // Colored background and text
    let mut df4 = DataFrame::new();
    df4.add_column("x", Box::new(FloatVec(vec![4.0])));
    df4.add_column("y", Box::new(FloatVec(vec![3.0])));
    df4.add_column("label", Box::new(StrVec(vec!["Colored".to_string()])));
    
    let plot = plot.geom_label_with(move |l| {
        l.data(Box::new(df4));
        l.aes(|a| {
            a.x("x");
            a.y("y");
            a.label("label");
        });
        l.geom
            .fill(Color::rgb(255, 99, 71))  // Tomato
            .color(Color::rgb(255, 255, 255))  // White text
            .radius(4.0)
            .size(12.0);
    });

    plot.save("labels_styling.png", 800, 600)?;
    println!("Saved labels_styling.png");
    Ok(())
}
