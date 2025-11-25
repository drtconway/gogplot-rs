use gogplot::layer::{Position, Stat};
use gogplot::prelude::*;
use gogplot::theme::color;
use gogplot::utils::dataframe::{DataFrame, FloatVec, IntVec};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    example_basic()?;
    example_styled()?;
    example_with_bars()?;
    example_with_bars_and_dodge()?;
    Ok(())
}

fn example_basic() -> Result<(), Box<dyn std::error::Error>> {
    // Sample data with individual points
    let mut df_points = DataFrame::new();
    df_points.add_column("x", Box::new(FloatVec(vec![1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 3.0, 3.0, 3.0])));
    df_points.add_column("y", Box::new(FloatVec(vec![10.0, 12.0, 11.0, 15.0, 18.0, 16.0, 8.0, 9.0, 10.0])));
    
    // Summary data with means and standard errors
    let mut df_summary = DataFrame::new();
    df_summary.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0])));
    df_summary.add_column("y", Box::new(FloatVec(vec![11.0, 16.33, 9.0])));
    df_summary.add_column("ymin", Box::new(FloatVec(vec![10.423, 15.448, 8.423])));
    df_summary.add_column("ymax", Box::new(FloatVec(vec![11.577, 17.212, 9.577])));
    
    let plot = Plot::new(Some(Box::new(df_points)))
        .title("Basic Error Bars")
        .aes(|a| { a.x("x"); a.y("y"); })
        .geom_point_with(|layer| {
            layer.geom.alpha(0.3);
        })
        .geom_errorbar_with(|layer| {
            layer
                .data(Box::new(df_summary.clone()))
                .aes(|a| { a.x("x"); a.ymin("ymin"); a.ymax("ymax"); });
        })
        .geom_point_with(|layer| {
            layer
                .data(Box::new(df_summary.clone()))
                .aes(|a| { a.x("x"); a.y("y"); })
                .geom.size(3.0);
        });
    
    plot.save("errorbars_basic.png", 800, 600)?;
    println!("Created errorbars_basic.png");
    Ok(())
}

fn example_styled() -> Result<(), Box<dyn std::error::Error>> {
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(FloatVec(vec![1.0, 2.0, 3.0])));
    df.add_column("y", Box::new(FloatVec(vec![11.0, 16.33, 9.0])));
    df.add_column("ymin", Box::new(FloatVec(vec![10.423, 15.448, 8.423])));
    df.add_column("ymax", Box::new(FloatVec(vec![11.577, 17.212, 9.577])));
    
    let plot = Plot::new(Some(Box::new(df.clone())))
        .title("Styled Error Bars")
        .aes(|a| { a.x("x"); a.y("y"); a.ymin("ymin"); a.ymax("ymax"); })
        .geom_errorbar_with(|layer| {
            layer.geom
                .width(0.3)
                .color(color::STEELBLUE)
                .size(1.5);
        })
        .geom_point_with(|layer| {
            layer.geom
                .size(4.0)
                .color(color::STEELBLUE);
        });
    
    plot.save("errorbars_styled.png", 800, 600)?;
    println!("Created errorbars_styled.png");
    Ok(())
}

fn example_with_bars() -> Result<(), Box<dyn std::error::Error>> {
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(IntVec(vec![1, 2, 3])));
    df.add_column("y", Box::new(FloatVec(vec![11.0, 16.33, 9.0])));
    df.add_column("ymin", Box::new(FloatVec(vec![10.423, 15.448, 8.423])));
    df.add_column("ymax", Box::new(FloatVec(vec![11.577, 17.212, 9.577])));
    
    let plot = Plot::new(Some(Box::new(df.clone())))
        .title("Bar Chart with Error Bars")
        .aes(|a| { a.x_categorical("x"); a.y("y"); })
        .geom_bar_with(|layer| {
            layer.stat = Stat::Identity;
            layer.geom
                .fill(color::LIGHTGRAY)
                .color(color::BLACK);
        })
        .geom_errorbar_with(|layer| {
            layer
                .aes(|a| { a.ymin("ymin"); a.ymax("ymax"); })
                .geom.width(0.25);
        });
    
    plot.save("errorbars_with_bars.png", 800, 600)?;
    println!("Created errorbars_with_bars.png");
    Ok(())
}

fn example_with_bars_and_dodge() -> Result<(), Box<dyn std::error::Error>> {
    let mut df = DataFrame::new();
    df.add_column("x", Box::new(IntVec(vec![1, 1, 2, 3, 3])));
    df.add_column("group", Box::new(IntVec(vec![1, 2, 1, 1, 2])));
    df.add_column("y", Box::new(FloatVec(vec![11.0, 16.33, 9.0, 12.0, 15.0])));
    df.add_column("ymin", Box::new(FloatVec(vec![10.423, 15.448, 8.423, 11.423, 14.423])));
    df.add_column("ymax", Box::new(FloatVec(vec![11.577, 17.212, 9.577, 12.577, 15.577])));
    
    let plot = Plot::new(Some(Box::new(df.clone())))
        .title("Bar Chart with Error Bars")
        .aes(|a| { a.x_categorical("x"); a.y("y"); a.fill_categorical("group");})
        .geom_bar_with(|layer| {
            layer.stat = Stat::Identity;
            layer.geom
                .position(Position::Dodge)
                .color(color::BLACK);
        })
        .geom_errorbar_with(|layer| {
            layer
                .aes(|a| { a.ymin("ymin"); a.ymax("ymax"); })
                .geom.width(0.25);
        });
    
    plot.save("errorbars_with_bars_and_dodge.png", 800, 600)?;
    println!("Created errorbars_with_bars_and_dodge.png");
    Ok(())
}
