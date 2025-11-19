use gogplot_rs::plot::Plot;
use gogplot_rs::theme::color;
use gogplot_rs::utils::dataframe::{DataFrame, FloatVec};
use gogplot_rs::visuals::Shape;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating shape examples...");

    // Circle (default)
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0])));
    df.add_column("y", Box::new(FloatVec(vec![2.0, 3.5, 2.5, 4.0, 3.0, 4.5])));

    let plot = Plot::new(Some(Box::new(df)))
        .title("Circle Shape")
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point_with(|geom, _aes| {
            geom.size(8.0)
                .color(color::BLUE)
                .shape(Shape::Circle)
        });
    plot.save("shape_circle.png", 800, 600)?;

    // Square
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0])));
    df.add_column("y", Box::new(FloatVec(vec![2.0, 3.5, 2.5, 4.0, 3.0, 4.5])));

    let plot = Plot::new(Some(Box::new(df)))
        .title("Square Shape")
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point_with(|geom, _aes| {
            geom.size(8.0)
                .color(color::RED)
                .shape(Shape::Square)
        });
    plot.save("shape_square.png", 800, 600)?;

    // Triangle
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0])));
    df.add_column("y", Box::new(FloatVec(vec![2.0, 3.5, 2.5, 4.0, 3.0, 4.5])));

    let plot = Plot::new(Some(Box::new(df)))
        .title("Triangle Shape")
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point_with(|geom, _aes| {
            geom.size(8.0)
                .color(color::GREEN)
                .shape(Shape::Triangle)
        });
    plot.save("shape_triangle.png", 800, 600)?;

    // Diamond
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0])));
    df.add_column("y", Box::new(FloatVec(vec![2.0, 3.5, 2.5, 4.0, 3.0, 4.5])));

    let plot = Plot::new(Some(Box::new(df)))
        .title("Diamond Shape")
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point_with(|geom, _aes| {
            geom.size(8.0)
                .color(color::PURPLE)
                .shape(Shape::Diamond)
        });
    plot.save("shape_diamond.png", 800, 600)?;

    // Cross
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0])));
    df.add_column("y", Box::new(FloatVec(vec![2.0, 3.5, 2.5, 4.0, 3.0, 4.5])));

    let plot = Plot::new(Some(Box::new(df)))
        .title("Cross Shape")
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point_with(|geom, _aes| {
            geom.size(8.0)
                .color(color::ORANGE)
                .shape(Shape::Cross)
        });
    plot.save("shape_cross.png", 800, 600)?;

    // Plus
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0])));
    df.add_column("y", Box::new(FloatVec(vec![2.0, 3.5, 2.5, 4.0, 3.0, 4.5])));

    let plot = Plot::new(Some(Box::new(df)))
        .title("Plus Shape")
        .aes(|a| {
            a.x("x");
            a.y("y");
        })
        .geom_point_with(|geom, _aes| {
            geom.size(8.0)
                .color(color::CYAN)
                .shape(Shape::Plus)
        });
    plot.save("shape_plus.png", 800, 600)?;

    println!("Saved shape_circle.png, shape_square.png, shape_triangle.png, shape_diamond.png, shape_cross.png, shape_plus.png");
    
    Ok(())
}
