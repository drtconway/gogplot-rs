use gogplot_rs::plot::{GeomBuilder, Plot};
use gogplot_rs::theme::{Color, color};
use gogplot_rs::utils::dataframe::{DataFrame, FloatVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating continuous color scale examples...");

    // Example 1: Default blue-to-black gradient (ggplot2-style)
    {
        let mut data = DataFrame::new();
        let x: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let y: Vec<f64> = x
            .iter()
            .map(|&x| (x / 5.0).sin() * 10.0 + x / 2.0)
            .collect();
        let z: Vec<f64> = x.iter().map(|&x| x).collect(); // Color by x value

        data.add_column("x", Box::new(FloatVec(x)));
        data.add_column("y", Box::new(FloatVec(y)));
        data.add_column("z", Box::new(FloatVec(z)));

        Plot::new(Some(Box::new(data)))
            .title("Default Continuous Color Scale (Blue to Black)")
            .aes(|aes| {
                aes.x("x");
                aes.y("y");
                aes.color("z"); // Map color to z values
            })
            .scale_color_continuous(vec![
                color::LIGHTBLUE2, // dark blue
                color::BLACK,      // black
            ])
            .geom_point_with(|geom| geom.size(8.0))
            .save("continuous_color_default.png", 800, 600)?;
        println!("Saved continuous_color_default.png");
    }

    // Example 2: Custom two-color gradient (blue to red)
    {
        let mut data = DataFrame::new();
        let x: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let y: Vec<f64> = x
            .iter()
            .map(|&x| (x / 5.0).cos() * 10.0 + x / 2.0)
            .collect();
        let z: Vec<f64> = x.iter().map(|&x| x).collect();

        data.add_column("x", Box::new(FloatVec(x)));
        data.add_column("y", Box::new(FloatVec(y)));
        data.add_column("z", Box::new(FloatVec(z)));

        Plot::new(Some(Box::new(data)))
            .title("Custom Two-Color Gradient (Blue to Red)")
            .aes(|aes| {
                aes.x("x");
                aes.y("y");
                aes.color("z");
            })
            .scale_color_continuous(vec![
                Color::rgb(0, 0, 255), // blue
                Color::rgb(255, 0, 0), // red
            ])
            .geom_point_with(|geom| geom.size(8.0))
            .save("continuous_color_two_colors.png", 800, 600)?;
        println!("Saved continuous_color_two_colors.png");
    }

    // Example 3: Multi-color gradient (viridis-inspired)
    {
        let mut data = DataFrame::new();
        let mut x = Vec::new();
        let mut y = Vec::new();
        let mut z = Vec::new();

        for i in 0..100 {
            for j in 0..100 {
                if (i + j) % 3 == 0 {
                    // Sparse grid for visibility
                    x.push(i as f64);
                    y.push(j as f64);
                    z.push((i * i + j * j) as f64);
                }
            }
        }

        data.add_column("x", Box::new(FloatVec(x)));
        data.add_column("y", Box::new(FloatVec(y)));
        data.add_column("z", Box::new(FloatVec(z)));

        Plot::new(Some(Box::new(data)))
            .title("Multi-Color Gradient (Viridis-Inspired)")
            .aes(|aes| {
                aes.x("x");
                aes.y("y");
                aes.color("z");
            })
            .scale_color_continuous(vec![
                Color::rgb(68, 1, 84),    // dark purple
                Color::rgb(59, 82, 139),  // blue-purple
                Color::rgb(33, 145, 140), // teal
                Color::rgb(94, 201, 98),  // green
                Color::rgb(253, 231, 37), // yellow
            ])
            .geom_point_with(|geom| geom.size(4.0))
            .save("continuous_color_viridis.png", 800, 800)?;
        println!("Saved continuous_color_viridis.png");
    }

    // Example 4: Diverging color scale (blue-white-red)
    {
        let mut data = DataFrame::new();
        let x: Vec<f64> = (0..50).map(|i| i as f64).collect();
        let y: Vec<f64> = x.iter().map(|&x| (x / 3.0).sin() * 15.0).collect();
        let z: Vec<f64> = y.clone(); // Color by y value (centered around 0)

        data.add_column("x", Box::new(FloatVec(x)));
        data.add_column("y", Box::new(FloatVec(y)));
        data.add_column("z", Box::new(FloatVec(z)));

        Plot::new(Some(Box::new(data)))
            .title("Diverging Color Scale (Blue-White-Red)")
            .aes(|aes| {
                aes.x("x");
                aes.y("y");
                aes.color("z");
            })
            .scale_color_continuous(vec![
                Color::rgb(0, 0, 255),     // blue
                Color::rgb(255, 255, 255), // white
                Color::rgb(255, 0, 0),     // red
            ])
            .geom_point_with(|geom| geom.size(10.0))
            .save("continuous_color_diverging.png", 800, 600)?;
        println!("Saved continuous_color_diverging.png");
    }

    println!("All continuous color examples completed!");
    Ok(())
}
