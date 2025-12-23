use gogplot::{
    aesthetics::builder::{XContininuousAesBuilder, YContininuousAesBuilder}, error::to_io_error, geom::point::geom_point, plot::plot, theme::color, utils::mtcars::mtcars
};

fn basic_points() -> std::io::Result<()> {
    let data = mtcars();

    let builder = plot(&data).aes(|a| {
        a.x("wt");
        a.y("mpg");
    }) + geom_point().size(3.0);

    let p = builder.build().map_err(to_io_error)?;
    p.save("basic_points.png", 800, 600).map_err(to_io_error)?;
    Ok(())
}

fn basic_points_with_color() -> std::io::Result<()> {
    let data = mtcars();

    let builder = plot(&data).aes(|a| {
        a.x("wt");
        a.y("mpg");
    }) + geom_point().size(5.0).color(color::BLUEVIOLET);

    let p = builder.build().map_err(to_io_error)?;
    p.save("basic_points_with_color.png", 800, 600).map_err(to_io_error)?;
    Ok(())
}

fn basic_points_with_color_and_size() -> std::io::Result<()> {
    let data = mtcars();

    let builder = plot(&data).aes(|a| {
        a.x("wt");
        a.y("mpg");
    }) + geom_point().aes(|a| {
        a.color_continuous("hp");
        a.size_discrete("cyl");
    });

    let p = builder.build().map_err(to_io_error)?;
    p.save("basic_points_with_color_and_size.png", 800, 600).map_err(to_io_error)?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    env_logger::builder()
    .filter_level(log::LevelFilter::Info)
    .init();

    basic_points()?;

    basic_points_with_color()?;

    basic_points_with_color_and_size()?;
    
    Ok(())
}
