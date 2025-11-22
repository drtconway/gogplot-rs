/// The mtcars dataset - Motor Trend Car Road Tests
///
/// The data was extracted from the 1974 Motor Trend US magazine, and comprises
/// fuel consumption and 10 aspects of automobile design and performance for 32
/// automobiles (1973–74 models).
///
/// ## Format
///
/// A data frame with 32 observations on 11 (numeric) variables:
///
/// - `model`: Model name
/// - `mpg`: Miles/(US) gallon
/// - `cyl`: Number of cylinders
/// - `disp`: Displacement (cu.in.)
/// - `hp`: Gross horsepower
/// - `drat`: Rear axle ratio
/// - `wt`: Weight (1000 lbs)
/// - `qsec`: 1/4 mile time
/// - `vs`: Engine (0 = V-shaped, 1 = straight)
/// - `am`: Transmission (0 = automatic, 1 = manual)
/// - `gear`: Number of forward gears
/// - `carb`: Number of carburetors
///
/// ## Source
///
/// Henderson and Velleman (1981), Building multiple regression models interactively.
/// Biometrics, 37, 391–411.

use crate::data::DataSource;
use crate::utils::dataframe::{DataFrame, FloatVec, IntVec, StrVec};

/// Load the mtcars dataset
///
/// Returns a DataFrame containing the classic Motor Trend Car Road Tests dataset.
///
/// # Example
///
/// ```
/// use gogplot::utils::mtcars::mtcars;
/// use gogplot::plot::Plot;
///
/// let data = mtcars();
/// let plot = Plot::new(Some(data))
///     .aes(|a| {
///         a.x("wt");
///         a.y("mpg");
///     })
///     .geom_point();
/// ```
pub fn mtcars() -> Box<dyn DataSource> {
    let mut df = DataFrame::new();

    // Model names
    df.add_column(
        "model", Box::new(StrVec::from(
        vec![
            "Mazda RX4",
            "Mazda RX4 Wag",
            "Datsun 710",
            "Hornet 4 Drive",
            "Hornet Sportabout",
            "Valiant",
            "Duster 360",
            "Merc 240D",
            "Merc 230",
            "Merc 280",
            "Merc 280C",
            "Merc 450SE",
            "Merc 450SL",
            "Merc 450SLC",
            "Cadillac Fleetwood",
            "Lincoln Continental",
            "Chrysler Imperial",
            "Fiat 128",
            "Honda Civic",
            "Toyota Corolla",
            "Toyota Corona",
            "Dodge Challenger",
            "AMC Javelin",
            "Camaro Z28",
            "Pontiac Firebird",
            "Fiat X1-9",
            "Porsche 914-2",
            "Lotus Europa",
            "Ford Pantera L",
            "Ferrari Dino",
            "Maserati Bora",
            "Volvo 142E",
        ])),
    );

    // MPG - Miles per gallon
    df.add_column(
        "mpg", Box::new(FloatVec(vec![
            21.0, 21.0, 22.8, 21.4, 18.7, 18.1, 14.3, 24.4, 22.8, 19.2, 17.8, 16.4, 17.3, 15.2,
            10.4, 10.4, 14.7, 32.4, 30.4, 33.9, 21.5, 15.5, 15.2, 13.3, 19.2, 27.3, 26.0, 30.4,
            15.8, 19.7, 15.0, 21.4,
        ])),
    );

    // Cylinders
    df.add_column(
        "cyl",
        Box::new(IntVec(vec![
            6, 6, 4, 6, 8, 6, 8, 4, 4, 6, 6, 8, 8, 8, 8, 8, 8, 4, 4, 4, 4, 8, 8, 8, 8, 4, 4, 4, 8,
            6, 8, 4,
        ])),
    );

    // Displacement (cubic inches)
    df.add_column(
        "disp",
        Box::new(FloatVec(vec![
            160.0, 160.0, 108.0, 258.0, 360.0, 225.0, 360.0, 146.7, 140.8, 167.6, 167.6, 275.8,
            275.8, 275.8, 472.0, 460.0, 440.0, 78.7, 75.7, 71.1, 120.1, 318.0, 304.0, 350.0,
            400.0, 79.0, 120.3, 95.1, 351.0, 145.0, 301.0, 121.0,
        ])),
    );

    // Horsepower
    df.add_column(
        "hp",
        Box::new(IntVec(vec![
            110, 110, 93, 110, 175, 105, 245, 62, 95, 123, 123, 180, 180, 180, 205, 215, 230, 66,
            52, 65, 97, 150, 150, 245, 175, 66, 91, 113, 264, 175, 335, 109,
        ])),
    );

    // Rear axle ratio
    df.add_column(
        "drat",
        Box::new(FloatVec(vec![
            3.90, 3.90, 3.85, 3.08, 3.15, 2.76, 3.21, 3.69, 3.92, 3.92, 3.92, 3.07, 3.07, 3.07,
            2.93, 3.00, 3.23, 4.08, 4.93, 4.22, 3.70, 2.76, 3.15, 3.73, 3.08, 4.08, 4.43, 3.77,
            4.22, 3.62, 3.54, 4.11,
        ])),
    );

    // Weight (1000 lbs)
    df.add_column(
        "wt",
        Box::new(FloatVec(vec![
            2.620, 2.875, 2.320, 3.215, 3.440, 3.460, 3.570, 3.190, 3.150, 3.440, 3.440, 4.070,
            3.730, 3.780, 5.250, 5.424, 5.345, 2.200, 1.615, 1.835, 2.465, 3.520, 3.435, 3.840,
            3.845, 1.935, 2.140, 1.513, 3.170, 2.770, 3.570, 2.780,
        ])),
    );

    // 1/4 mile time
    df.add_column(
        "qsec",
        Box::new(FloatVec(vec![
            16.46, 17.02, 18.61, 19.44, 17.02, 20.22, 15.84, 20.00, 22.90, 18.30, 18.90, 17.40,
            17.60, 18.00, 17.98, 17.82, 17.42, 19.47, 18.52, 19.90, 20.01, 16.87, 17.30, 15.41,
            17.05, 18.90, 16.70, 16.90, 14.50, 15.50, 14.60, 18.60,
        ])),
    );

    // Engine shape (0 = V-shaped, 1 = straight)
    df.add_column(
        "vs",
        Box::new(IntVec(vec![
            0, 0, 1, 1, 0, 1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 1, 0,
            0, 0, 1,
        ])),
    );

    // Transmission (0 = automatic, 1 = manual)
    df.add_column(
        "am",
        Box::new(IntVec(vec![
            1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 1, 1, 1,
            1, 1, 1,
        ])),
    );

    // Number of forward gears
    df.add_column(
        "gear",
        Box::new(IntVec(vec![
            4, 4, 4, 3, 3, 3, 3, 4, 4, 4, 4, 3, 3, 3, 3, 3, 3, 4, 4, 4, 3, 3, 3, 3, 3, 4, 5, 5, 5,
            5, 5, 4,
        ])),
    );

    // Number of carburetors
    df.add_column(
        "carb",
        Box::new(IntVec(vec![
            4, 4, 1, 1, 2, 1, 4, 2, 2, 4, 4, 3, 3, 3, 4, 4, 4, 1, 2, 1, 1, 2, 2, 4, 2, 1, 2, 2, 4,
            6, 8, 2,
        ])),
    );

    Box::new(df)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mtcars_dimensions() {
        let data = mtcars();
        assert_eq!(data.len(), 32);
        
        // Check that key columns exist
        assert!(data.get("model").is_some());
        assert!(data.get("mpg").is_some());
        assert!(data.get("cyl").is_some());
        assert!(data.get("wt").is_some());
    }

    #[test]
    fn test_mtcars_column_types() {
        let data = mtcars();
        
        // Model should be strings
        let model = data.get("model").unwrap();
        assert!(model.iter_str().is_some());
        
        // MPG should be floats
        let mpg = data.get("mpg").unwrap();
        assert!(mpg.iter_float().is_some());
        
        // Cylinders should be integers
        let cyl = data.get("cyl").unwrap();
        assert!(cyl.iter_int().is_some());
    }

    #[test]
    fn test_mtcars_values() {
        let data = mtcars();
        
        // Check first row values
        let mpg = data.get("mpg").unwrap();
        let mpg_values: Vec<f64> = mpg.iter_float().unwrap().collect();
        assert_eq!(mpg_values[0], 21.0); // Mazda RX4
        
        let cyl = data.get("cyl").unwrap();
        let cyl_values: Vec<i64> = cyl.iter_int().unwrap().collect();
        assert_eq!(cyl_values[0], 6); // Mazda RX4 has 6 cylinders
    }
}
