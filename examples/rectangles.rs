use gogplot_rs::aesthetics::{Aesthetic, AesValue};
use gogplot_rs::plot::Plot;
use gogplot_rs::theme::color;
use gogplot_rs::utils::dataframe::{DataFrame, FloatVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating rectangles example...");

    // Example 1: Simple rectangles with fill aesthetic
    {
        let mut data = DataFrame::new();
        data.add_column("xmin", Box::new(FloatVec(vec![1.0, 2.5, 4.0])));
        data.add_column("xmax", Box::new(FloatVec(vec![2.0, 3.5, 5.0])));
        data.add_column("ymin", Box::new(FloatVec(vec![1.0, 2.0, 1.5])));
        data.add_column("ymax", Box::new(FloatVec(vec![2.0, 3.0, 2.5])));

        Plot::new(Some(Box::new(data)))
            .aes(|aes| {
                aes.set(
                    Aesthetic::XBegin,
                    AesValue::Column("xmin".to_string()),
                );
                aes.set(
                    Aesthetic::XEnd,
                    AesValue::Column("xmax".to_string()),
                );
                aes.set(
                    Aesthetic::YBegin,
                    AesValue::Column("ymin".to_string()),
                );
                aes.set(
                    Aesthetic::YEnd,
                    AesValue::Column("ymax".to_string()),
                );
            })
            .geom_rect_with(|geom| geom.fill(color::STEELBLUE).alpha(0.7))
            .save("rectangle_simple.png", 600, 400)?;
        println!("Saved rectangle_simple.png");
    }

    // Example 2: Rectangles with fill and stroke
    {
        let mut data = DataFrame::new();
        data.add_column("xmin", Box::new(FloatVec(vec![0.5, 2.0, 3.5])));
        data.add_column("xmax", Box::new(FloatVec(vec![1.5, 3.0, 4.5])));
        data.add_column("ymin", Box::new(FloatVec(vec![0.5, 1.5, 2.5])));
        data.add_column("ymax", Box::new(FloatVec(vec![1.5, 2.5, 3.5])));

        Plot::new(Some(Box::new(data)))
            .aes(|aes| {
                aes.set(
                    Aesthetic::XBegin,
                    AesValue::Column("xmin".to_string()),
                );
                aes.set(
                    Aesthetic::XEnd,
                    AesValue::Column("xmax".to_string()),
                );
                aes.set(
                    Aesthetic::YBegin,
                    AesValue::Column("ymin".to_string()),
                );
                aes.set(
                    Aesthetic::YEnd,
                    AesValue::Column("ymax".to_string()),
                );
            })
            .geom_rect_with(|geom| {
                geom.fill(color::CORAL)
                    .color(color::DARKRED)
                    .alpha(0.6)
            })
            .save("rectangle_stroke.png", 600, 400)?;
        println!("Saved rectangle_stroke.png");
    }

    // Example 3: Grid of rectangles with different colors
    {
        let mut xmin_vals = Vec::new();
        let mut xmax_vals = Vec::new();
        let mut ymin_vals = Vec::new();
        let mut ymax_vals = Vec::new();

        for i in 0..5 {
            for j in 0..5 {
                xmin_vals.push(i as f64);
                xmax_vals.push((i + 1) as f64);
                ymin_vals.push(j as f64);
                ymax_vals.push((j + 1) as f64);
            }
        }

        let mut data = DataFrame::new();
        data.add_column("xmin", Box::new(FloatVec(xmin_vals)));
        data.add_column("xmax", Box::new(FloatVec(xmax_vals)));
        data.add_column("ymin", Box::new(FloatVec(ymin_vals)));
        data.add_column("ymax", Box::new(FloatVec(ymax_vals)));

        Plot::new(Some(Box::new(data)))
            .aes(|aes| {
                aes.set(
                    Aesthetic::XBegin,
                    AesValue::Column("xmin".to_string()),
                );
                aes.set(
                    Aesthetic::XEnd,
                    AesValue::Column("xmax".to_string()),
                );
                aes.set(
                    Aesthetic::YBegin,
                    AesValue::Column("ymin".to_string()),
                );
                aes.set(
                    Aesthetic::YEnd,
                    AesValue::Column("ymax".to_string()),
                );
            })
            .geom_rect_with(|geom| geom.fill(color::LIGHTBLUE).color(color::GRAY50).alpha(0.8))
            .save("rectangle_grid.png", 600, 600)?;
        println!("Saved rectangle_grid.png");
    }

    // Example 4: Overlapping semi-transparent rectangles
    {
        let mut data = DataFrame::new();
        data.add_column("xmin", Box::new(FloatVec(vec![1.0, 2.0, 1.5])));
        data.add_column("xmax", Box::new(FloatVec(vec![3.0, 4.0, 3.5])));
        data.add_column("ymin", Box::new(FloatVec(vec![1.0, 1.5, 2.0])));
        data.add_column("ymax", Box::new(FloatVec(vec![3.0, 3.5, 4.0])));

        Plot::new(Some(Box::new(data)))
            .aes(|aes| {
                aes.set(
                    Aesthetic::XBegin,
                    AesValue::Column("xmin".to_string()),
                );
                aes.set(
                    Aesthetic::XEnd,
                    AesValue::Column("xmax".to_string()),
                );
                aes.set(
                    Aesthetic::YBegin,
                    AesValue::Column("ymin".to_string()),
                );
                aes.set(
                    Aesthetic::YEnd,
                    AesValue::Column("ymax".to_string()),
                );
            })
            .geom_rect_with(|geom| {
                geom.fill(color::PURPLE).alpha(0.3)
            })
            .save("rectangle_overlap.png", 600, 400)?;
        println!("Saved rectangle_overlap.png");
    }

    println!("All rectangle examples completed!");
    Ok(())
}
