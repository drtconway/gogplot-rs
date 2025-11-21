use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::data::{DataSource, StackedDataSource};
use crate::error::Result;
use crate::stat::StatTransform;
use crate::utils::dataframe::{DataFrame, FloatVec, IntVec};

/// Bin configuration strategy
#[derive(Debug, Clone)]
pub enum BinStrategy {
    /// Fixed number of bins
    Count(usize),
    /// Fixed bin width
    Width(f64),
}

impl Default for BinStrategy {
    fn default() -> Self {
        BinStrategy::Count(30)
    }
}

/// Bin statistical transformation
///
/// Divides the range of x values into equally-spaced bins and counts the number of
/// observations in each bin. Produces new columns for bin centers and counts.
///
/// # Example
///
/// For continuous data [1.0, 1.5, 2.0, 2.5, 3.0, 3.5] with 3 bins:
/// - Bins: [1.0-1.83), [1.83-2.67), [2.67-3.5]
/// - Centers: [1.42, 2.25, 3.08]
/// - Counts: [2, 2, 2]
pub struct Bin {
    pub strategy: BinStrategy,
}

impl Bin {
    /// Create a new Bin stat with the specified number of bins
    pub fn with_count(bins: usize) -> Self {
        Self {
            strategy: BinStrategy::Count(bins),
        }
    }

    /// Create a new Bin stat with a specific bin width
    pub fn with_width(binwidth: f64) -> Self {
        Self {
            strategy: BinStrategy::Width(binwidth),
        }
    }
}

impl Default for Bin {
    fn default() -> Self {
        Self::with_count(30)
    }
}

impl StatTransform for Bin {
    fn apply(
        &self,
        data: Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<Option<(Box<dyn DataSource>, AesMap)>> {
        // Get the x aesthetic - this is required for binning
        let x_mapping = mapping.get(&Aesthetic::X).ok_or_else(|| {
            crate::error::PlotError::Generic("Bin stat requires X aesthetic".to_string())
        })?;

        // Only support column mappings for now
        let x_col_name = match x_mapping {
            AesValue::Column(name) => name,
            _ => {
                return Err(crate::error::PlotError::Generic(
                    "Bin stat requires X to be mapped to a column".to_string(),
                ));
            }
        };

        // Get the x column from data
        let x_col = data.get(x_col_name.as_str()).ok_or_else(|| {
            crate::error::PlotError::Generic(format!("Column '{}' not found in data", x_col_name))
        })?;

        // Convert to float values
        let x_values: Vec<f64> = if let Some(int_vec) = x_col.as_int() {
            int_vec.iter().map(|&v| v as f64).collect()
        } else if let Some(float_vec) = x_col.as_float() {
            float_vec.iter().copied().filter(|v| v.is_finite()).collect()
        } else {
            return Err(crate::error::PlotError::Generic(
                "Bin stat requires numeric X values".to_string(),
            ));
        };

        if x_values.is_empty() {
            return Err(crate::error::PlotError::Generic(
                "No valid numeric values for binning".to_string(),
            ));
        }

        // Calculate bin parameters
        let min_val = x_values
            .iter()
            .copied()
            .fold(f64::INFINITY, f64::min);
        let max_val = x_values
            .iter()
            .copied()
            .fold(f64::NEG_INFINITY, f64::max);

        let range = max_val - min_val;
        if range == 0.0 {
            // All values are the same - create a single bin
            let mut computed = DataFrame::new();
            computed.add_column("x", Box::new(FloatVec(vec![min_val])));
            computed.add_column("count", Box::new(IntVec(vec![x_values.len() as i64])));
            computed.add_column("xmin", Box::new(FloatVec(vec![min_val - 0.5])));
            computed.add_column("xmax", Box::new(FloatVec(vec![min_val + 0.5])));

            let stacked = StackedDataSource::two_layer(Box::new(computed), data);
            let mut new_mapping = mapping.clone();
            new_mapping.set(Aesthetic::X, AesValue::column("x"));
            new_mapping.set(Aesthetic::Y, AesValue::column("count"));

            return Ok(Some((Box::new(stacked), new_mapping)));
        }

        // Determine bin width based on strategy
        let binwidth = match self.strategy {
            BinStrategy::Width(width) => width,
            BinStrategy::Count(bins) => range / bins as f64,
        };

        // Determine actual number of bins needed
        let n_bins = ((range / binwidth).ceil() as usize).max(1);

        // Expand range slightly to ensure max value is included
        let adjusted_range = binwidth * n_bins as f64;
        let offset = (adjusted_range - range) / 2.0;
        let bin_min = min_val - offset;

        // Count values in each bin
        let mut counts = vec![0i64; n_bins];
        for &value in &x_values {
            let bin_idx = ((value - bin_min) / binwidth).floor() as usize;
            let bin_idx = bin_idx.min(n_bins - 1); // Ensure last bin includes max value
            counts[bin_idx] += 1;
        }

        // Generate bin centers, min, and max
        let mut centers = Vec::with_capacity(n_bins);
        let mut xmins = Vec::with_capacity(n_bins);
        let mut xmaxs = Vec::with_capacity(n_bins);

        for i in 0..n_bins {
            let bin_start = bin_min + i as f64 * binwidth;
            let bin_end = bin_start + binwidth;
            centers.push((bin_start + bin_end) / 2.0);
            xmins.push(bin_start);
            xmaxs.push(bin_end);
        }

        // Create computed DataFrame
        let mut computed = DataFrame::new();
        computed.add_column("x", Box::new(FloatVec(centers)));
        computed.add_column("count", Box::new(IntVec(counts)));
        computed.add_column("xmin", Box::new(FloatVec(xmins)));
        computed.add_column("xmax", Box::new(FloatVec(xmaxs)));

        // Create stacked data source
        let stacked = StackedDataSource::two_layer(Box::new(computed), data);

        // Update mapping
        let mut new_mapping = mapping.clone();
        new_mapping.set(Aesthetic::X, AesValue::column("x"));
        new_mapping.set(Aesthetic::Y, AesValue::column("count"));

        Ok(Some((Box::new(stacked), new_mapping)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::dataframe::DataFrame;

    #[test]
    fn test_bin_basic() {
        let mut df = DataFrame::new();
        df.add_column(
            "x",
            Box::new(FloatVec(vec![1.0, 1.5, 2.0, 2.5, 3.0, 3.5])),
        );

        let mut mapping = AesMap::new();
        mapping.x("x");

        let bin = Bin::with_count(3);
        let result = bin.apply(Box::new(df), &mapping);
        assert!(result.is_ok());

        let (data, new_mapping) = result.unwrap().unwrap();

        // Check that y is now mapped to count
        assert_eq!(
            new_mapping.get(&Aesthetic::Y),
            Some(&AesValue::column("count"))
        );

        // Check that we have the right number of bins
        let count_col = data.get("count").unwrap();
        let counts: Vec<i64> = count_col.as_int().unwrap().iter().copied().collect();
        assert_eq!(counts.len(), 3);
        assert_eq!(counts.iter().sum::<i64>(), 6); // Total should equal input size
    }

    #[test]
    fn test_bin_with_integers() {
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(IntVec(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10])));

        let mut mapping = AesMap::new();
        mapping.x("x");

        let bin = Bin::with_count(5);
        let (data, _) = bin.apply(Box::new(df), &mapping).unwrap().unwrap();

        let count_col = data.get("count").unwrap();
        let counts: Vec<i64> = count_col.as_int().unwrap().iter().copied().collect();
        assert_eq!(counts.len(), 5);
        assert_eq!(counts.iter().sum::<i64>(), 10);
    }

    #[test]
    fn test_bin_with_width() {
        let mut df = DataFrame::new();
        df.add_column(
            "x",
            Box::new(FloatVec(vec![0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0])),
        );

        let mut mapping = AesMap::new();
        mapping.x("x");

        let bin = Bin::with_width(1.0);
        let (data, _) = bin.apply(Box::new(df), &mapping).unwrap().unwrap();

        // With binwidth=1.0 and range 0-4, we expect bins like:
        // [0-1), [1-2), [2-3), [3-4]
        let xmin_col = data.get("xmin").unwrap();
        let xmins: Vec<f64> = xmin_col.as_float().unwrap().iter().copied().collect();
        
        let xmax_col = data.get("xmax").unwrap();
        let xmaxs: Vec<f64> = xmax_col.as_float().unwrap().iter().copied().collect();

        // Verify bins are approximately 1.0 wide
        for i in 0..xmins.len() {
            assert!((xmaxs[i] - xmins[i] - 1.0).abs() < 0.01);
        }
    }

    #[test]
    fn test_bin_single_value() {
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(FloatVec(vec![5.0, 5.0, 5.0, 5.0])));

        let mut mapping = AesMap::new();
        mapping.x("x");

        let bin = Bin::with_count(3);
        let (data, _) = bin.apply(Box::new(df), &mapping).unwrap().unwrap();

        let x_col = data.get("x").unwrap();
        let centers: Vec<f64> = x_col.as_float().unwrap().iter().copied().collect();
        assert_eq!(centers.len(), 1);
        assert_eq!(centers[0], 5.0);

        let count_col = data.get("count").unwrap();
        let counts: Vec<i64> = count_col.as_int().unwrap().iter().copied().collect();
        assert_eq!(counts, vec![4]);
    }

    #[test]
    fn test_bin_requires_x() {
        let mut df = DataFrame::new();
        df.add_column("y", Box::new(FloatVec(vec![1.0, 2.0, 3.0])));

        let mapping = AesMap::new(); // No x mapping

        let bin = Bin::default();
        let result = bin.apply(Box::new(df), &mapping);
        assert!(result.is_err());
    }

    #[test]
    fn test_bin_filters_nan() {
        let mut df = DataFrame::new();
        df.add_column(
            "x",
            Box::new(FloatVec(vec![
                1.0,
                f64::NAN,
                2.0,
                3.0,
                f64::NAN,
                4.0,
                5.0,
            ])),
        );

        let mut mapping = AesMap::new();
        mapping.x("x");

        let bin = Bin::with_count(2);
        let (data, _) = bin.apply(Box::new(df), &mapping).unwrap().unwrap();

        let count_col = data.get("count").unwrap();
        let counts: Vec<i64> = count_col.as_int().unwrap().iter().copied().collect();
        // Only 5 valid values (NaNs filtered out)
        assert_eq!(counts.iter().sum::<i64>(), 5);
    }

    #[test]
    fn test_binwidth_explicit() {
        // Test that binwidth parameter actually controls bin width
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(FloatVec(vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0])));

        let mut mapping = AesMap::new();
        mapping.x("x");

        // With range 0-10 and binwidth 2.0, we should get 5 bins
        let bin = Bin::with_width(2.0);
        let (data, _) = bin.apply(Box::new(df), &mapping).unwrap().unwrap();

        let xmin_col = data.get("xmin").unwrap();
        let xmins: Vec<f64> = xmin_col.as_float().unwrap().iter().copied().collect();
        
        let xmax_col = data.get("xmax").unwrap();
        let xmaxs: Vec<f64> = xmax_col.as_float().unwrap().iter().copied().collect();

        println!("Binwidth 2.0 test - Number of bins: {}", xmins.len());
        for i in 0..xmins.len() {
            let width = xmaxs[i] - xmins[i];
            println!("  Bin {}: [{:.2}, {:.2}) width={:.2}", i, xmins[i], xmaxs[i], width);
            // Each bin should be exactly 2.0 wide
            assert!((width - 2.0).abs() < 0.01, "Bin {} width is {}, expected 2.0", i, width);
        }
        
        // Should have 5 bins (range 10 / binwidth 2 = 5)
        assert_eq!(xmins.len(), 5, "Expected 5 bins with binwidth 2.0 over range 10");
    }
}
