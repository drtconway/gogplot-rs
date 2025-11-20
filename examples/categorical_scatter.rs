// Categorical scatter plot example
//
// This example demonstrates handling categorical variables on x and y axes
// with geom_point to isolate categorical variable issues from bar chart specifics.

use gogplot_rs::aesthetics::{Aesthetic, AesMap, AesValue};
use gogplot_rs::geom::point::GeomPoint;
use gogplot_rs::layer::{Layer, Stat, Position};
use gogplot_rs::utils::dataframe::{DataFrame, FloatVec, StrVec};
use gogplot_rs::plot::Plot;
use gogplot_rs::theme::color;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Example 1: Categorical X, Numeric Y
    println!("Creating categorical_x_numeric_y.png...");
    let categories_x = vec!["A", "B", "C", "A", "B", "C", "A", "B", "C"];
    let values_y = vec![1.5, 2.3, 3.1, 2.0, 2.8, 3.5, 1.8, 2.5, 3.0];
    
    let mut df1 = DataFrame::new();
    df1.add_column("category", Box::new(StrVec::from(categories_x)));
    df1.add_column("value", Box::new(FloatVec(values_y)));
    
    let mut mapping1 = AesMap::new();
    mapping1.set(Aesthetic::X, AesValue::column("category"));
    mapping1.set(Aesthetic::Y, AesValue::column("value"));
    
    let geom1 = GeomPoint::new()
        .color(color::STEELBLUE)
        .size(4.0);
    
    let layer1 = Layer {
        geom: Box::new(geom1),
        data: Some(Box::new(df1)),
        mapping: mapping1,
        stat: Stat::Identity,
        position: Position::Identity,
        computed_data: None,
        computed_mapping: None,
    };
    
    let mut plot1 = Plot::new(None);
    plot1.layers.push(layer1);
    let plot1 = plot1.title("Categorical X, Numeric Y");
    
    plot1.save("categorical_x_numeric_y.png", 800, 600)?;
    println!("Saved categorical_x_numeric_y.png");
    
    // Example 2: Numeric X, Categorical Y
    println!("\nCreating numeric_x_categorical_y.png...");
    let values_x = vec![1.5, 2.3, 3.1, 2.0, 2.8, 3.5, 1.8, 2.5, 3.0];
    let categories_y = vec!["Low", "Med", "High", "Low", "Med", "High", "Low", "Med", "High"];
    
    let mut df2 = DataFrame::new();
    df2.add_column("value", Box::new(FloatVec(values_x)));
    df2.add_column("category", Box::new(StrVec::from(categories_y)));
    
    let mut mapping2 = AesMap::new();
    mapping2.set(Aesthetic::X, AesValue::column("value"));
    mapping2.set(Aesthetic::Y, AesValue::column("category"));
    
    let geom2 = GeomPoint::new()
        .color(color::CORAL)
        .size(4.0);
    
    let layer2 = Layer {
        geom: Box::new(geom2),
        data: Some(Box::new(df2)),
        mapping: mapping2,
        stat: Stat::Identity,
        position: Position::Identity,
        computed_data: None,
        computed_mapping: None,
    };
    
    let mut plot2 = Plot::new(None);
    plot2.layers.push(layer2);
    let plot2 = plot2.title("Numeric X, Categorical Y");
    
    plot2.save("numeric_x_categorical_y.png", 800, 600)?;
    println!("Saved numeric_x_categorical_y.png");
    
    Ok(())
}
