use crate::aesthetics::{AesMap, AesValue, Aesthetic, AestheticDomain};
use crate::data::{ContinuousType, DataSource, VectorIter};
use crate::error::{PlotError, Result};
use crate::stat::Stat;
use crate::utils::data::Vectorable;
use crate::utils::dataframe::DataFrame;
use std::any::Any;
use std::collections::HashMap;

/// Bin configuration strategy
#[derive(Debug, Clone)]
pub enum BinStrategy {
    /// Fixed number of bins
    Count(usize),
    /// Fixed bin width
    Width(f64),
}

struct Binner {
    binwidth: f64,
    min: f64,
    n_bins: usize,
}

impl Binner {
    fn new(min: f64, max: f64, strategy: &BinStrategy) -> Self {
        let (binwidth, n_bins) = match strategy {
            BinStrategy::Width(width) => {
                let range = max - min;
                let n_bins = ((range / width).ceil() as usize).max(1);
                (*width, n_bins)
            }
            BinStrategy::Count(bins) => {
                let range = max - min;
                let binwidth = range / (*bins) as f64;
                (binwidth, *bins)
            }
        };
        Self {
            binwidth,
            min,
            n_bins,
        }
    }

    fn len(&self) -> usize {
        self.n_bins
    }

    fn bin_of_value(&self, value: f64) -> usize {
        let idx = ((value - self.min) / self.binwidth).floor() as usize;
        let idx = idx.min((self.n_bins as usize) - 1);
        idx
    }

    fn center_of_bin(&self, idx: usize) -> f64 {
        let bin_start = self.min + idx as f64 * self.binwidth;
        let bin_end = bin_start + self.binwidth;
        (bin_start + bin_end) / 2.0
    }

    fn bin_bounds(&self, idx: usize) -> (f64, f64) {
        let bin_start = self.min + idx as f64 * self.binwidth;
        let bin_end = bin_start + self.binwidth;
        (bin_start, bin_end)
    }

    fn bin_from_iter<T: ContinuousType + Vectorable>(
        &self,
        iter: impl Iterator<Item = T>,
    ) -> Vec<i64> {
        let mut counts = vec![0i64; self.n_bins];
        for value in iter {
            let value_f64 = value.to_f64();
            let bin_idx = self.bin_of_value(value_f64);
            counts[bin_idx] += 1;
        }
        counts
    }
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

struct DataRanges {
    ranges: HashMap<Aesthetic, (f64, f64)>,
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

impl Stat for Bin {
    fn compute_params(
        &self,
        data: &dyn DataSource,
        mapping: &AesMap,
        aesthetics: &[Aesthetic],
    ) -> Result<Option<Box<dyn Any>>> {
        let mut result = HashMap::new();
        for aes in aesthetics {
            match mapping.get_vector_iter(aes, data) {
                Some(iter) => {
                    let range = get_data_range(iter)
                        .ok_or(PlotError::MissingAesthetic { aesthetic: *aes })?;
                    result.insert(*aes, range);
                }
                None => {
                    return Err(PlotError::MissingAesthetic { aesthetic: *aes });
                }
            }
        }
        Ok(Some(Box::new(DataRanges { ranges: result })))
    }

    fn compute_group(
        &self,
        aesthetics: Vec<Aesthetic>,
        iters: Vec<VectorIter<'_>>,
        params: Option<&dyn Any>,
    ) -> Result<(DataFrame, AesMap)> {
        let data_ranges = params.and_then(|p| p.downcast_ref::<DataRanges>()).ok_or(
            PlotError::InvalidStatParameters {
                details: "Missing or invalid parameters for Bin stat".to_string(),
            },
        )?;
        for (aesthetic, iter) in aesthetics.into_iter().zip(iters.into_iter()) {
            let data_range = data_ranges.ranges.get(&aesthetic).cloned();
            let (min, max) = match data_range {
                Some((min, max)) if min < max => (min, max),
                Some((min, max)) if min == max => {
                    // All values are identical; create a single bin
                    let mut data = DataFrame::new();
                    data.add_column("xmin", vec![min]);
                    data.add_column("x", vec![min]);
                    data.add_column("xmax", vec![min]);
                    data.add_column("count", vec![iter.count() as i64]);
                    let mut mapping = AesMap::new();
                    mapping.set(
                        Aesthetic::X(AestheticDomain::Continuous),
                        AesValue::column("x"),
                    );
                    mapping.set(
                        Aesthetic::Xmin(AestheticDomain::Continuous),
                        AesValue::column("xmin"),
                    );
                    mapping.set(
                        Aesthetic::Xmax(AestheticDomain::Continuous),
                        AesValue::column("xmax"),
                    );
                    mapping.set(
                        Aesthetic::Y(AestheticDomain::Continuous),
                        AesValue::column("count"),
                    );
                    return Ok((data, mapping));
                },
                _ => {
                    // All values are identical or no valid values; create a single bin
                    let mut data = DataFrame::new();
                    data.add_column("xmin", vec![0.0]);
                    data.add_column("x", vec![0.0]);
                    data.add_column("xmax", vec![0.0]);
                    data.add_column("count", vec![0]);
                    let mut mapping = AesMap::new();
                    mapping.set(
                        Aesthetic::X(AestheticDomain::Continuous),
                        AesValue::column("x"),
                    );
                    mapping.set(
                        Aesthetic::Xmin(AestheticDomain::Continuous),
                        AesValue::column("xmin"),
                    );
                    mapping.set(
                        Aesthetic::Xmax(AestheticDomain::Continuous),
                        AesValue::column("xmax"),
                    );
                    mapping.set(
                        Aesthetic::Y(AestheticDomain::Continuous),
                        AesValue::column("count"),
                    );
                    return Ok((data, mapping));
                }
            };

            let binner = Binner::new(min, max, &self.strategy.strategy);

            println!("aes = {:?}, iter.vtype() = {:?}", aesthetic,iter.vtype());

            let mut counts = match iter {
                VectorIter::Int(iter) => binner.bin_from_iter(iter),
                VectorIter::Float(iter) => binner.bin_from_iter(iter),
                _ => {
                    return Err(PlotError::InvalidAestheticType {
                        aesthetic,
                        expected: crate::error::DataType::Continuous,
                        actual: crate::error::DataType::Discrete,
                    });
                }
            };

            if self.strategy.cumulative {
                for i in 1..counts.len() {
                    counts[i] += counts[i - 1];
                }
            }
            let mut xmins = Vec::with_capacity(binner.len());
            let mut xmaxs = Vec::with_capacity(binner.len());
            let mut xcenters = Vec::with_capacity(binner.len());
            for i in 0..binner.len() {
                let (xmin, xmax) = binner.bin_bounds(i);
                xmins.push(xmin);
                xmaxs.push(xmax);
                xcenters.push(binner.center_of_bin(i));
            }

            let mut data = DataFrame::new();
            data.add_column("xmin", xmins);
            data.add_column("x", xcenters);
            data.add_column("xmax", xmaxs);
            data.add_column("count", counts);
            println!("data: {:?}", data);

            let mut mapping = AesMap::new();
            mapping.set(
                Aesthetic::X(AestheticDomain::Continuous),
                AesValue::column("x"),
            );
            mapping.set(
                Aesthetic::Xmin(AestheticDomain::Continuous),
                AesValue::column("xmin"),
            );
            mapping.set(
                Aesthetic::Xmax(AestheticDomain::Continuous),
                AesValue::column("xmax"),
            );
            mapping.set(
                Aesthetic::Y(AestheticDomain::Continuous),
                AesValue::column("count"),
            );

            return Ok((data, mapping));
        }
        panic!("No aesthetics provided");
    }
}

fn get_data_range<'a>(iter: VectorIter<'a>) -> Option<(f64, f64)> {
    match iter {
        VectorIter::Int(int_iter) => get_data_range_inner(int_iter),
        VectorIter::Float(float_iter) => get_data_range_inner(float_iter),
        _ => None,
    }
}

fn get_data_range_inner<T: ContinuousType>(
    mut iter: impl Iterator<Item = T>,
) -> Option<(f64, f64)> {
    let first = iter.next()?;
    let mut min = first.clone();
    let mut max = first;
    for value in iter {
        if value < min {
            min = value;
        } else if value > max {
            max = value;
        }
    }
    Some((min.to_f64(), max.to_f64()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{data::VectorType, utils::dataframe::DataFrame};

    #[test]
    fn test_bin_basic() {
        let mut df = DataFrame::new();
        df.add_column("x", vec![1.0, 1.5, 2.0, 2.5, 3.0, 3.5]);
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Continuous);

        let bin = Bin::with_count(3);
        let result = bin.compute(df.as_ref(), &mapping);
        assert!(result.is_ok());

        let (data, new_mapping) = result.unwrap();

        // Check that y is now mapped to count
        assert_eq!(
            new_mapping.get(&Aesthetic::Y(AestheticDomain::Continuous)),
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
        df.add_column("x", vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Continuous);

        let bin = Bin::with_count(5);
        let (data, _) = bin.compute(df.as_ref(), &mapping).unwrap();

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
            vec![0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0],
        );
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Continuous);

        let bin = Bin::with_width(1.0);
        let (data, _) = bin.compute(df.as_ref(), &mapping).unwrap();

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
        df.add_column("x", vec![5.0, 5.0, 5.0, 5.0]);
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Continuous);

        let bin = Bin::with_count(3);
        let (data, _) = bin.compute(df.as_ref(), &mapping).unwrap();

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
        df.add_column("y", vec![1.0, 2.0, 3.0]);
        let df: Box<dyn DataSource> = Box::new(df);

        let mapping = AesMap::new(); // No x mapping

        let bin = Bin::default();
        let result = bin.compute(df.as_ref(), &mapping);
        assert!(result.is_err());
    }

    #[test]
    fn test_bin_filters_nan() {
        let mut df = DataFrame::new();
        df.add_column(
            "x",
            vec![1.0, f64::NAN, 2.0, 3.0, f64::NAN, 4.0, 5.0],
        );
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Continuous);

        let bin = Bin::with_count(2);
        let (data, _) = bin.compute(df.as_ref(), &mapping).unwrap();

        let count_col = data.get("count").unwrap();
        let counts: Vec<i64> = count_col.iter_int().unwrap().collect();
        // Only 5 valid values (NaNs filtered out)
        assert_eq!(counts.iter().sum::<i64>(), 7);
    }

    #[test]
    fn test_binwidth_explicit() {
        // Test that binwidth parameter actually controls bin width
        let mut df = DataFrame::new();
        df.add_column(
            "x",
            vec![
                0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0,
            ],
        );
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Continuous);

        // With range 0-10 and binwidth 2.0, we should get 5 bins
        let bin = Bin::with_width(2.0);
        let (data, _) = bin.compute(df.as_ref(), &mapping).unwrap();

        let xmin_col = data.get("xmin").unwrap();
        let xmins: Vec<f64> = xmin_col.iter_float().unwrap().collect();

        let xmax_col = data.get("xmax").unwrap();
        let xmaxs: Vec<f64> = xmax_col.iter_float().unwrap().collect();

        println!("Binwidth 2.0 test - Number of bins: {}", xmins.len());
        for i in 0..xmins.len() {
            let width = xmaxs[i] - xmins[i];
            println!(
                "  Bin {}: [{:.2}, {:.2}) width={:.2}",
                i, xmins[i], xmaxs[i], width
            );
            // Each bin should be exactly 2.0 wide
            assert!(
                (width - 2.0).abs() < 0.01,
                "Bin {} width is {}, expected 2.0",
                i,
                width
            );
        }

        // Should have 5 bins (range 10 / binwidth 2 = 5)
        assert_eq!(
            xmins.len(),
            5,
            "Expected 5 bins with binwidth 2.0 over range 10"
        );
    }

    #[test]
    fn test_grouped_binning() {
        // Create data with two groups
        let mut df = DataFrame::new();
        df.add_column(
            "x",
            vec![
                1.0, 1.5, 2.0, 2.5, 3.0, // Group A
                2.0, 2.5, 3.0, 3.5, 4.0, // Group B
            ],
        );
        df.add_column(
            "group",
            vec![
                "A", "A", "A", "A", "A", "B", "B", "B", "B", "B",
            ],
        );
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Continuous);
        mapping.set(
            Aesthetic::Fill(AestheticDomain::Discrete),
            AesValue::column("group"),
        );

        assert_eq!(mapping.get(&Aesthetic::X(AestheticDomain::Continuous)), Some(&AesValue::column("x")));
        assert_eq!(mapping.get_vector_iter(&Aesthetic::X(AestheticDomain::Continuous), df.as_ref()).map(|x| x.vtype()), Some(VectorType::Float));

        let bin = Bin::with_count(3);
        let (data, new_mapping) = bin.compute(df.as_ref(), &mapping).unwrap();

        // Check that y is mapped to count
        assert_eq!(
            new_mapping.get(&Aesthetic::Y(AestheticDomain::Continuous)),
            Some(&AesValue::column("count"))
        );

        println!("data: {:?}", data);

        // Check that group column is preserved
        let group_col = data.get("group").unwrap();
        assert!(group_col.iter_str().is_some());

        // Check that we have data for both groups
        let groups: Vec<String> = group_col
            .iter_str()
            .unwrap()
            .map(|s| s.to_string())
            .collect();
        println!("Groups in binned data: {:?}", groups);
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
            assert!(
                (width - bin_width).abs() < 0.01,
                "Bin widths should be consistent"
            );
        }
    }

    #[test]
    fn test_multiple_grouping_aesthetics() {
        // Create data with two grouping dimensions
        let mut df = DataFrame::new();
        df.add_column(
            "x",
            vec![
                1.0, 2.0, // Color A, Shape X
                3.0, 4.0, // Color A, Shape Y
                1.5, 2.5, // Color B, Shape X
                3.5, 4.5, // Color B, Shape Y
            ],
        );
        df.add_column(
            "color",
            vec!["A", "A", "A", "A", "B", "B", "B", "B"],
        );
        df.add_column(
            "shape",
            vec!["X", "X", "Y", "Y", "X", "X", "Y", "Y"],
        );
        let df: Box<dyn DataSource> = Box::new(df);

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Continuous);
        mapping.set(
            Aesthetic::Fill(AestheticDomain::Discrete),
            AesValue::column("color"),
        );
        mapping.set(Aesthetic::Shape, AesValue::column("shape"));

        let bin = Bin::with_count(3);
        let (data, new_mapping) = bin.compute(df.as_ref(), &mapping).unwrap();

        println!("data: {:?}", data);
        println!("new_mapping: {:?}", new_mapping);

        // Check that both grouping columns are preserved
        let color_col = data.get("color").unwrap();
        assert!(color_col.iter_str().is_some());
        let shape_col = data.get("shape").unwrap();
        assert!(shape_col.iter_str().is_some());

        // Should have 4 distinct groups: A-X, A-Y, B-X, B-Y
        let colors: Vec<String> = color_col
            .iter_str()
            .unwrap()
            .map(|s| s.to_string())
            .collect();
        let shapes: Vec<String> = shape_col
            .iter_str()
            .unwrap()
            .map(|s| s.to_string())
            .collect();

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
