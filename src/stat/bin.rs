use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::data::DataSource;
use crate::error::{DataType, Result};
use crate::stat::StatTransform;
use crate::utils::dataframe::{DataFrame, FloatVec, IntVec, StrVec};
use crate::utils::grouping::{get_grouping_columns, create_composite_keys, group_by_key, split_composite_key};
use std::collections::HashMap;

/// Bin configuration strategy
#[derive(Debug, Clone)]
pub enum BinStrategy {
    /// Fixed number of bins
    Count(usize),
    /// Fixed bin width
    Width(f64),
}

impl BinStrategy {
    /// Make this binning strategy cumulative
    pub fn cumulative(self, cumulative: bool) -> CumulativeBinStrategy {
        CumulativeBinStrategy {
            strategy: self,
            cumulative,
        }
    }
}

/// Bin strategy with optional cumulative flag
#[derive(Debug, Clone)]
pub struct CumulativeBinStrategy {
    pub strategy: BinStrategy,
    pub cumulative: bool,
}

impl CumulativeBinStrategy {
    /// Create a new non-cumulative bin strategy
    pub fn new(strategy: BinStrategy) -> Self {
        Self {
            strategy,
            cumulative: false,
        }
    }

    /// Enable or disable cumulative mode
    pub fn cumulative(mut self, cumulative: bool) -> Self {
        self.cumulative = cumulative;
        self
    }
}

impl From<BinStrategy> for CumulativeBinStrategy {
    fn from(strategy: BinStrategy) -> Self {
        Self::new(strategy)
    }
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
///
/// For cumulative mode, counts are accumulated:
/// - Cumulative Counts: [2, 4, 6]
pub struct Bin {
    pub strategy: CumulativeBinStrategy,
}

impl Bin {
    /// Create a new Bin stat with the specified number of bins
    pub fn with_count(bins: usize) -> Self {
        Self {
            strategy: BinStrategy::Count(bins).into(),
        }
    }

    /// Create a new Bin stat with a specific bin width
    pub fn with_width(binwidth: f64) -> Self {
        Self {
            strategy: BinStrategy::Width(binwidth).into(),
        }
    }

    /// Create a new Bin stat from a cumulative strategy
    pub fn with_strategy(strategy: CumulativeBinStrategy) -> Self {
        Self { strategy }
    }
}

impl Default for Bin {
    fn default() -> Self {
        Self::with_count(30)
    }
}

impl Bin {
    /// Apply grouped binning - bin each group separately using the same bin boundaries.
    ///
    /// When multiple grouping aesthetics are mapped (e.g., both Fill and Shape), this creates
    /// groups based on the intersection of all grouping columns. For example:
    /// - Fill="A", Shape="Circle" → Group "A__Circle"
    /// - Fill="A", Shape="Square" → Group "A__Square"
    /// - Fill="B", Shape="Circle" → Group "B__Circle"
    ///
    /// All groups use the same bin boundaries (computed from the combined data range),
    /// but are counted separately. This matches ggplot2 behavior.
    fn apply_grouped(
        &self,
        data: Box<dyn DataSource>,
        mapping: &AesMap,
        x_col_name: &str,
        group_cols: &[(Aesthetic, String)],
    ) -> Result<Option<(Box<dyn DataSource>, AesMap)>> {
        // Get x column
        let x_col = data.get(x_col_name).ok_or_else(|| {
            crate::error::PlotError::missing_column(x_col_name)
        })?;

        // Create composite group keys using grouping utilities
        let composite_keys = create_composite_keys(data.as_ref(), group_cols);

        // Convert x to float values
        let x_values: Vec<f64> = if let Some(int_iter) = x_col.iter_int() {
            int_iter.map(|v| v as f64).collect()
        } else if let Some(float_iter) = x_col.iter_float() {
            float_iter.filter(|v| v.is_finite()).collect()
        } else {
            return Err(crate::error::PlotError::invalid_column_type(
                x_col_name,
                "numeric (int or float)",
            ));
        };

        if x_values.is_empty() {
            return Err(crate::error::PlotError::no_valid_data(
                "no valid numeric values for binning"
            ));
        }

        // Calculate global bin boundaries based on all data
        let min_val = x_values.iter().copied().fold(f64::INFINITY, f64::min);
        let max_val = x_values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let range = max_val - min_val;

        if range == 0.0 {
            // All values the same - single bin per group
            let mut groups_map: HashMap<String, i64> = HashMap::new();
            for key in &composite_keys {
                *groups_map.entry(key.clone()).or_insert(0) += 1;
            }

            let mut x_centers = Vec::new();
            let mut counts = Vec::new();
            let mut xmins = Vec::new();
            let mut xmaxs = Vec::new();
            let mut group_keys = Vec::new();

            for (group, count) in groups_map {
                x_centers.push(min_val);
                counts.push(count);
                xmins.push(min_val - 0.5);
                xmaxs.push(min_val + 0.5);
                group_keys.push(group);
            }

            let mut computed = DataFrame::new();
            computed.add_column("x", Box::new(FloatVec(x_centers)));
            computed.add_column("count", Box::new(IntVec(counts)));
            computed.add_column("xmin", Box::new(FloatVec(xmins)));
            computed.add_column("xmax", Box::new(FloatVec(xmaxs)));
            
            // Add columns for each grouping aesthetic
            for (aesthetic, col_name) in group_cols {
                let values: Vec<String> = group_keys
                    .iter()
                    .map(|key| {
                        let key_parts = split_composite_key(key, group_cols);
                        key_parts
                            .iter()
                            .find(|(aes, _)| aes == aesthetic)
                            .map(|(_, val)| val.clone())
                            .unwrap_or_default()
                    })
                    .collect();
                computed.add_column(col_name, Box::new(StrVec(values)));
            }

            let mut new_mapping = mapping.clone();
            new_mapping.set(Aesthetic::X, AesValue::column("x"));
            new_mapping.set(Aesthetic::Y, AesValue::column("count"));

            return Ok(Some((Box::new(computed), new_mapping)));
        }

        // Determine bin width based on strategy
        let binwidth = match &self.strategy.strategy {
            BinStrategy::Width(width) => *width,
            BinStrategy::Count(bins) => range / *bins as f64,
        };

        let n_bins = ((range / binwidth).ceil() as usize).max(1);
        let adjusted_range = binwidth * n_bins as f64;
        let offset = (adjusted_range - range) / 2.0;
        let bin_min = min_val - offset;

        // Group data by composite key using utility
        let groups_data = group_by_key(&x_values, &composite_keys);

        // Bin each group separately using the same boundaries
        let mut all_x_centers = Vec::new();
        let mut all_counts = Vec::new();
        let mut all_xmins = Vec::new();
        let mut all_xmaxs = Vec::new();
        let mut all_group_keys = Vec::new();

        for (group_key, group_x_values) in groups_data {
            // Count values in each bin for this group
            let mut counts = vec![0i64; n_bins];
            for &value in &group_x_values {
                let bin_idx = ((value - bin_min) / binwidth).floor() as usize;
                let bin_idx = bin_idx.min(n_bins - 1);
                counts[bin_idx] += 1;
            }

            // If cumulative mode, accumulate counts
            if self.strategy.cumulative {
                for i in 1..n_bins {
                    counts[i] += counts[i - 1];
                }
            }

            // Generate output rows for this group (include all bins, even empty ones)
            // This is important for position adjustments like Stack/Dodge
            for i in 0..n_bins {
                let bin_start = bin_min + i as f64 * binwidth;
                let bin_end = bin_start + binwidth;
                all_x_centers.push((bin_start + bin_end) / 2.0);
                all_counts.push(counts[i]);
                all_xmins.push(bin_start);
                all_xmaxs.push(bin_end);
                all_group_keys.push(group_key.clone());
            }
        }

        // Create computed DataFrame
        let mut computed = DataFrame::new();
        computed.add_column("x", Box::new(FloatVec(all_x_centers)));
        computed.add_column("count", Box::new(IntVec(all_counts)));
        computed.add_column("xmin", Box::new(FloatVec(all_xmins)));
        computed.add_column("xmax", Box::new(FloatVec(all_xmaxs)));
        
        // Add columns for each grouping aesthetic by splitting composite keys
        for (aesthetic, col_name) in group_cols {
            let values: Vec<String> = all_group_keys
                .iter()
                .map(|key| {
                    let key_parts = split_composite_key(key, group_cols);
                    key_parts
                        .iter()
                        .find(|(aes, _)| aes == aesthetic)
                        .map(|(_, val)| val.clone())
                        .unwrap_or_default()
                })
                .collect();
            computed.add_column(col_name, Box::new(StrVec(values)));
        }

        // Update mapping
        let mut new_mapping = mapping.clone();
        new_mapping.set(Aesthetic::X, AesValue::column("x"));
        new_mapping.set(Aesthetic::Y, AesValue::column("count"));

        Ok(Some((Box::new(computed), new_mapping)))
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
            crate::error::PlotError::missing_stat_input("Bin", Aesthetic::X)
        })?;

        // Only support column mappings for now
        let x_col_name = match x_mapping {
            AesValue::Column(name) => name,
            _ => {
                return Err(crate::error::PlotError::InvalidAestheticType {
                    aesthetic: Aesthetic::X,
                    expected: DataType::ColumnMapping,
                    actual: DataType::Custom("constant".to_string()),
                });
            }
        };

        // Get grouping columns using utility
        let group_col_names = get_grouping_columns(mapping);

        // If we have any grouping aesthetics, use grouped binning
        if !group_col_names.is_empty() {
            return self.apply_grouped(data, mapping, x_col_name.as_str(), &group_col_names);
        }

        // Otherwise, use ungrouped binning (original logic)
        // Get the x column from data
        let x_col = data.get(x_col_name.as_str()).ok_or_else(|| {
            crate::error::PlotError::missing_column(x_col_name.as_str())
        })?;

        // Convert to float values
        let x_values: Vec<f64> = if let Some(int_iter) = x_col.iter_int() {
            int_iter.map(|v| v as f64).collect()
        } else if let Some(float_iter) = x_col.iter_float() {
            float_iter.filter(|v| v.is_finite()).collect()
        } else {
            return Err(crate::error::PlotError::invalid_column_type(
                x_col_name.as_str(),
                "numeric (int or float)",
            ));
        };

        if x_values.is_empty() {
            return Err(crate::error::PlotError::no_valid_data(
                "no valid numeric values for binning"
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

            let mut new_mapping = mapping.clone();
            new_mapping.set(Aesthetic::X, AesValue::column("x"));
            new_mapping.set(Aesthetic::Y, AesValue::column("count"));

            return Ok(Some((Box::new(computed), new_mapping)));
        }

        // Determine bin width based on strategy
        let binwidth = match &self.strategy.strategy {
            BinStrategy::Width(width) => *width,
            BinStrategy::Count(bins) => range / *bins as f64,
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

        // If cumulative mode, accumulate counts
        if self.strategy.cumulative {
            for i in 1..n_bins {
                counts[i] += counts[i - 1];
            }
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

        // Update mapping
        let mut new_mapping = mapping.clone();
        new_mapping.set(Aesthetic::X, AesValue::column("x"));
        new_mapping.set(Aesthetic::Y, AesValue::column("count"));

        Ok(Some((Box::new(computed), new_mapping)))
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
        let counts: Vec<i64> = count_col.iter_int().unwrap().collect();
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
        let counts: Vec<i64> = count_col.iter_int().unwrap().collect();
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
        let xmins: Vec<f64> = xmin_col.iter_float().unwrap().collect();
        
        let xmax_col = data.get("xmax").unwrap();
        let xmaxs: Vec<f64> = xmax_col.iter_float().unwrap().collect();

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
        let centers: Vec<f64> = x_col.iter_float().unwrap().collect();
        assert_eq!(centers.len(), 1);
        assert_eq!(centers[0], 5.0);

        let count_col = data.get("count").unwrap();
        let counts: Vec<i64> = count_col.iter_int().unwrap().collect();
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
        let counts: Vec<i64> = count_col.iter_int().unwrap().collect();
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
        let xmins: Vec<f64> = xmin_col.iter_float().unwrap().collect();
        
        let xmax_col = data.get("xmax").unwrap();
        let xmaxs: Vec<f64> = xmax_col.iter_float().unwrap().collect();

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

    #[test]
    fn test_grouped_binning() {
        use crate::utils::dataframe::StrVec;
        
        // Create data with two groups
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(FloatVec(vec![
            1.0, 1.5, 2.0, 2.5, 3.0,  // Group A
            2.0, 2.5, 3.0, 3.5, 4.0,  // Group B
        ])));
        df.add_column("group", Box::new(StrVec(vec![
            "A".to_string(), "A".to_string(), "A".to_string(), "A".to_string(), "A".to_string(),
            "B".to_string(), "B".to_string(), "B".to_string(), "B".to_string(), "B".to_string(),
        ])));

        let mut mapping = AesMap::new();
        mapping.x("x");
        mapping.set(Aesthetic::Fill, AesValue::column("group"));

        let bin = Bin::with_count(3);
        let (data, new_mapping) = bin.apply(Box::new(df), &mapping).unwrap().unwrap();

        // Check that y is mapped to count
        assert_eq!(
            new_mapping.get(&Aesthetic::Y),
            Some(&AesValue::column("count"))
        );

        // Check that group column is preserved
        let group_col = data.get("group").unwrap();
        assert!(group_col.iter_str().is_some());

        // Check that we have data for both groups
        let groups: Vec<String> = group_col.iter_str().unwrap().map(|s| s.to_string()).collect();
        assert!(groups.contains(&"A".to_string()));
        assert!(groups.contains(&"B".to_string()));

        // Verify counts sum to original data size
        let count_col = data.get("count").unwrap();
        let counts: Vec<i64> = count_col.iter_int().unwrap().collect();
        assert_eq!(counts.iter().sum::<i64>(), 10);

        // Verify all bins use the same boundaries (by checking xmin/xmax are consistent)
        let xmin_col = data.get("xmin").unwrap();
        let xmins: Vec<f64> = xmin_col.iter_float().unwrap().collect();
        let xmax_col = data.get("xmax").unwrap();
        let xmaxs: Vec<f64> = xmax_col.iter_float().unwrap().collect();

        // All bins should have the same width
        let bin_width = xmaxs[0] - xmins[0];
        for i in 0..xmins.len() {
            let width = xmaxs[i] - xmins[i];
            assert!((width - bin_width).abs() < 0.01, "Bin widths should be consistent");
        }
    }

    #[test]
    fn test_multiple_grouping_aesthetics() {
        use crate::utils::dataframe::StrVec;
        
        // Create data with two grouping dimensions
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(FloatVec(vec![
            1.0, 2.0,  // Color A, Shape X
            3.0, 4.0,  // Color A, Shape Y
            1.5, 2.5,  // Color B, Shape X
            3.5, 4.5,  // Color B, Shape Y
        ])));
        df.add_column("color", Box::new(StrVec(vec![
            "A".to_string(), "A".to_string(),
            "A".to_string(), "A".to_string(),
            "B".to_string(), "B".to_string(),
            "B".to_string(), "B".to_string(),
        ])));
        df.add_column("shape", Box::new(StrVec(vec![
            "X".to_string(), "X".to_string(),
            "Y".to_string(), "Y".to_string(),
            "X".to_string(), "X".to_string(),
            "Y".to_string(), "Y".to_string(),
        ])));

        let mut mapping = AesMap::new();
        mapping.x("x");
        mapping.set(Aesthetic::Fill, AesValue::column("color"));
        mapping.set(Aesthetic::Shape, AesValue::column("shape"));

        let bin = Bin::with_count(3);
        let (data, _) = bin.apply(Box::new(df), &mapping).unwrap().unwrap();

        // Check that both grouping columns are preserved
        let color_col = data.get("color").unwrap();
        assert!(color_col.iter_str().is_some());
        let shape_col = data.get("shape").unwrap();
        assert!(shape_col.iter_str().is_some());

        // Should have 4 distinct groups: A-X, A-Y, B-X, B-Y
        let colors: Vec<String> = color_col.iter_str().unwrap().map(|s| s.to_string()).collect();
        let shapes: Vec<String> = shape_col.iter_str().unwrap().map(|s| s.to_string()).collect();
        
        let mut groups = std::collections::HashSet::new();
        for i in 0..colors.len() {
            groups.insert(format!("{}-{}", colors[i], shapes[i]));
        }
        
        // We expect up to 4 groups (some bins may be empty for some groups)
        assert!(groups.len() <= 4);
        assert!(groups.len() >= 2); // At least some groups should be present

        // Verify counts sum to original data size
        let count_col = data.get("count").unwrap();
        let counts: Vec<i64> = count_col.iter_int().unwrap().collect();
        assert_eq!(counts.iter().sum::<i64>(), 8);
    }
}
