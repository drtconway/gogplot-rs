use gogplot_rs::aesthetics::Aesthetic;
use gogplot_rs::plot::Plot;
use gogplot_rs::theme::color;
use gogplot_rs::utils::dataframe::{DataFrame, FloatVec, StrVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating segment examples...");

    // Example 1: Simple segments
    {
        let mut data = DataFrame::new();
        data.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0])));
        data.add_column("y", Box::new(FloatVec(vec![1.0, 2.0, 1.5, 3.0])));
        data.add_column("xend", Box::new(FloatVec(vec![1.5, 2.5, 3.5, 4.5])));
        data.add_column("yend", Box::new(FloatVec(vec![2.0, 1.0, 2.5, 1.5])));

        Plot::new(Some(Box::new(data)))
            .aes(|aes| {
                aes.set_to_column(Aesthetic::X, "x");
                aes.set_to_column(Aesthetic::Y, "y");
                aes.set_to_column(Aesthetic::XEnd, "xend");
                aes.set_to_column(Aesthetic::YEnd, "yend");
            })
            .geom_segment_with(|geom| geom.color(color::STEELBLUE).size(2.0))
            .save("segment_simple.png", 600, 400)?;
        println!("Saved segment_simple.png");
    }

    // Example 2: Segments with varying colors
    {
        let mut data = DataFrame::new();
        data.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 5.0])));
        data.add_column("y", Box::new(FloatVec(vec![2.0, 3.0, 2.5, 4.0, 3.5])));
        data.add_column("xend", Box::new(FloatVec(vec![2.0, 3.0, 4.0, 5.0, 6.0])));
        data.add_column("yend", Box::new(FloatVec(vec![3.0, 2.5, 4.0, 3.5, 5.0])));
        data.add_column("group", Box::new(StrVec::from(vec!["A", "B", "A", "C", "B"])));

        Plot::new(Some(Box::new(data)))
            .aes(|aes| {
                aes.set_to_column(Aesthetic::X, "x");
                aes.set_to_column(Aesthetic::Y, "y");
                aes.set_to_column(Aesthetic::XEnd, "xend");
                aes.set_to_column(Aesthetic::YEnd, "yend");
                aes.set_to_column(Aesthetic::Color, "group");
            })
            .geom_segment_with(|geom| geom.size(3.0))
            .save("segment_colored.png", 600, 400)?;
        println!("Saved segment_colored.png");
    }

    // Example 3: Arrow-like pattern with segments
    {
        let center_x = 3.0;
        let center_y = 3.0;
        let n_arrows = 8;
        let mut x = Vec::new();
        let mut y = Vec::new();
        let mut xend = Vec::new();
        let mut yend = Vec::new();

        for i in 0..n_arrows {
            let angle = (i as f64) * 2.0 * std::f64::consts::PI / (n_arrows as f64);
            x.push(center_x);
            y.push(center_y);
            xend.push(center_x + angle.cos() * 1.5);
            yend.push(center_y + angle.sin() * 1.5);
        }

        let mut data = DataFrame::new();
        data.add_column("x", Box::new(FloatVec(x)));
        data.add_column("y", Box::new(FloatVec(y)));
        data.add_column("xend", Box::new(FloatVec(xend)));
        data.add_column("yend", Box::new(FloatVec(yend)));

        Plot::new(Some(Box::new(data)))
            .aes(|aes| {
                aes.set_to_column(Aesthetic::X, "x");
                aes.set_to_column(Aesthetic::Y, "y");
                aes.set_to_column(Aesthetic::XEnd, "xend");
                aes.set_to_column(Aesthetic::YEnd, "yend");
            })
            .geom_segment_with(|geom| geom.color(color::CORAL).size(3.0).alpha(0.7))
            .save("segment_radial.png", 600, 400)?;
        println!("Saved segment_radial.png");
    }

    // Example 4: Grid pattern with segments
    {
        let mut x = Vec::new();
        let mut y = Vec::new();
        let mut xend = Vec::new();
        let mut yend = Vec::new();

        // Horizontal segments
        for i in 0..5 {
            let y_pos = (i as f64) * 0.5 + 1.0;
            x.push(1.0);
            y.push(y_pos);
            xend.push(4.0);
            yend.push(y_pos);
        }

        // Vertical segments
        for i in 0..7 {
            let x_pos = (i as f64) * 0.5 + 1.0;
            x.push(x_pos);
            y.push(1.0);
            xend.push(x_pos);
            yend.push(3.0);
        }

        let mut data = DataFrame::new();
        data.add_column("x", Box::new(FloatVec(x)));
        data.add_column("y", Box::new(FloatVec(y)));
        data.add_column("xend", Box::new(FloatVec(xend)));
        data.add_column("yend", Box::new(FloatVec(yend)));

        Plot::new(Some(Box::new(data)))
            .aes(|aes| {
                aes.set_to_column(Aesthetic::X, "x");
                aes.set_to_column(Aesthetic::Y, "y");
                aes.set_to_column(Aesthetic::XEnd, "xend");
                aes.set_to_column(Aesthetic::YEnd, "yend");
            })
            .geom_segment_with(|geom| geom.color(color::GRAY).size(1.0))
            .save("segment_grid.png", 600, 400)?;
        println!("Saved segment_grid.png");
    }

    // Example 5: Different line styles
    {
        let mut data = DataFrame::new();
        data.add_column("x", Box::new(FloatVec(vec![1.0, 1.0, 1.0, 1.0, 1.0])));
        data.add_column("y", Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 5.0])));
        data.add_column("xend", Box::new(FloatVec(vec![5.0, 5.0, 5.0, 5.0, 5.0])));
        data.add_column("yend", Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 5.0])));
        data.add_column("style", Box::new(StrVec::from(vec!["-", ".", "-.", "- -", ". ."])));

        Plot::new(Some(Box::new(data)))
            .aes(|aes| {
                aes.set_to_column(Aesthetic::X, "x");
                aes.set_to_column(Aesthetic::Y, "y");
                aes.set_to_column(Aesthetic::XEnd, "xend");
                aes.set_to_column(Aesthetic::YEnd, "yend");
                aes.set_to_column(Aesthetic::Linetype, "style");
            })
            .geom_segment_with(|geom| geom.color(color::NAVY).size(2.0))
            .save("segment_linestyles.png", 600, 400)?;
        println!("Saved segment_linestyles.png");
    }

    println!("All segment examples completed!");
    Ok(())
}
