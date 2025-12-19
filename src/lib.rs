pub mod aesthetics;
pub mod data;
pub mod error;
pub mod geom;
pub mod guide;
pub mod layer;
pub mod plot;
pub mod position;
pub mod prelude;
pub mod scale;
pub mod stat;
pub mod theme;
pub mod utils;
pub mod visuals;

pub use plot::Plot;
pub use error::PlotError;

pub fn plot<'a>(data: &'a Box<dyn data::DataSource>) -> Plot<'a> {
    Plot::new(data)
}

