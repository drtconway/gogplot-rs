pub mod dataframe;
pub mod data;
pub mod grouping;
pub mod mtcars;
pub mod sp500;

#[cfg(feature = "arrow")]
pub mod datafusion;

#[cfg(feature = "polars")]
pub mod polars;
