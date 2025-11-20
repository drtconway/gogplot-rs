use gogplot_rs::plot::{GeomBuilder, Plot};
use gogplot_rs::theme::color;
use gogplot_rs::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating line style examples...");

    // Create sample data with different line styles
    let x = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];

    // Different line patterns
    let y1 = vec![1.0, 2.0, 1.5, 3.0, 2.5, 4.0, 3.5, 5.0, 4.5, 6.0];
    let y2 = vec![2.0, 2.5, 2.0, 3.5, 3.0, 4.5, 4.0, 5.5, 5.0, 6.5];
    let y3 = vec![3.0, 3.0, 2.5, 4.0, 3.5, 5.0, 4.5, 6.0, 5.5, 7.0];
    let y4 = vec![4.0, 3.5, 3.0, 4.5, 4.0, 5.5, 5.0, 6.5, 6.0, 7.5];

    // Concatenate all data for a single dataframe
    let mut all_x = Vec::new();
    let mut all_y = Vec::new();
    let mut all_style = Vec::new();

    for i in 0..x.len() {
        all_x.push(x[i]);
        all_y.push(y1[i]);
        all_style.push("solid".to_string());

        all_x.push(x[i]);
        all_y.push(y2[i]);
        all_style.push("-".to_string());

        all_x.push(x[i]);
        all_y.push(y3[i]);
        all_style.push(".".to_string());

        all_x.push(x[i]);
        all_y.push(y4[i]);
        all_style.push("-.".to_string());
    }

    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(all_x)));
    df.add_column("y", Box::new(FloatVec(all_y)));
    df.add_column("style", Box::new(StrVec(all_style)));

    // Create plot with different line styles
    let plot = Plot::new(Some(Box::new(df)))
        .title("Line Styles Example")
        .aes(|a| {
            a.x("x");
            a.y("y");
            a.linetype("style");
            a.group("style");
        })
        .geom_line_with(|geom| geom.color(color::BLUE).size(2.0));

    plot.save("line_styles_mapped.png", 800, 600)?;
    println!("Saved line_styles_mapped.png");

    // Create individual examples for each style
    create_style_example("-", "Dashed")?;
    create_style_example(".", "Dotted")?;
    create_style_example("-.", "Dash-Dot")?;
    create_style_example("- -", "Dashed with Long Gaps")?;
    create_style_example(". .", "Dotted with Long Gaps")?;
    create_style_example("- . ", "Complex Pattern")?;

    Ok(())
}

fn create_style_example(pattern: &str, title: &str) -> Result<(), Box<dyn std::error::Error>> {
    let x = vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
    let y = vec![1.0, 3.0, 2.0, 4.0, 3.0, 5.0, 4.0, 6.0, 5.0, 7.0, 6.0];

    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(x)));
    df.add_column("y", Box::new(FloatVec(y)));

    let plot = Plot::new(Some(Box::new(df)))
        .title(title)
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_line_with(|geom| geom.color(color::RED).size(2.5).linetype(pattern));

    let filename = format!(
        "line_style_{}.png",
        pattern
            .chars()
            .map(|c| match c {
                '-' => "dash",
                '.' => "dot",
                ' ' => "gap",
                _ => "x",
            })
            .collect::<Vec<_>>()
            .join("_")
    );

    plot.save(&filename, 600, 400)?;
    println!("Saved {}", filename);

    Ok(())
}
