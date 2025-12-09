use crate::aesthetics::{AesMap, AesValue, Aesthetic, AestheticDomain};
use crate::data::{ContinuousType, DataSource, DiscreteType, PrimitiveType, VectorIter};
use crate::error::Result;
use crate::stat::StatTransform;
use crate::utils::data::{ContinuousVectorVisitor, DiscreteContinuousVisitor2, Vectorable, visit_c, visit2_dc};
use crate::utils::dataframe::{BoolVec, DataFrame, FloatVec, IntVec, StrVec};
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

impl StatTransform for Bin {
    fn apply(
        &self,
        data: Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<Option<(Box<dyn DataSource>, AesMap)>> {
        // Get the x aesthetic - this is required for binning
        let x_mapping = mapping
            .get_vector_iter(&Aesthetic::X(AestheticDomain::Continuous), data.as_ref())
            .ok_or_else(|| {
                crate::error::PlotError::missing_stat_input(
                    "Bin",
                    Aesthetic::X(AestheticDomain::Continuous),
                )
            })?;

        let (x_min, x_max) = get_data_range(x_mapping).ok_or_else(|| {
            crate::error::PlotError::no_valid_data("no valid numeric values for binning")
        })?;
        let binner = Binner::new(x_min, x_max, &self.strategy.strategy);

        if let Some(group_iter) = mapping.get_vector_iter(&Aesthetic::Group, data.as_ref()) {
            let x_values = mapping
                .get_vector_iter(&Aesthetic::X(AestheticDomain::Continuous), data.as_ref())
                .unwrap();

            let mut binner = GroupedValueBinner::new(binner, self.strategy.cumulative);
            visit2_dc(group_iter, x_values, &mut binner)?;

            Ok(Some((Box::new(binner.data), binner.mapping)))
        } else {
            let x_values = mapping
                .get_vector_iter(&Aesthetic::X(AestheticDomain::Continuous), data.as_ref())
                .unwrap();


            let mut binner = UngroupedValueBinner::new(binner, self.strategy.cumulative);
            visit_c(x_values, &mut binner)?;

            Ok(Some((Box::new(binner.data), binner.mapping)))
        }
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

fn bin_ungrouped_data<T: ContinuousType>(
    iterator: impl Iterator<Item = T>,
    binner: &Binner,
    cumulative: bool,
) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<i64>) {
    let mut counts = vec![0i64; binner.len()];

    for value in iterator {
        let value_f64 = value.to_f64();
        let bin_idx = binner.bin_of_value(value_f64);
        counts[bin_idx] += 1;
    }

    // If cumulative, accumulate counts
    if cumulative {
        for i in 1..binner.len() {
            counts[i] += counts[i - 1];
        }
    }

    // Generate bin centers
    let mut mins = Vec::with_capacity(binner.len());
    let mut centers = Vec::with_capacity(binner.len());
    let mut maxs = Vec::with_capacity(binner.len());
    for i in 0..binner.len() {
        let (min, max) = binner.bin_bounds(i);
        mins.push(min);
        centers.push(binner.center_of_bin(i));
        maxs.push(max);
    }

    (mins, centers, maxs, counts)
}

struct UngroupedValueBinner {
    binner: Binner,
    cumulative: bool,
    data: DataFrame,
    mapping: AesMap,
}

impl UngroupedValueBinner {
    fn new(binner: Binner, cumulative: bool) -> Self {
        Self {
            binner,
            cumulative,
            data: DataFrame::new(),
            mapping: AesMap::new(),
        }
    }
}

impl ContinuousVectorVisitor for UngroupedValueBinner {
    fn visit<T: Vectorable + ContinuousType>(&mut self, values: impl Iterator<Item = T>) {
        let mut counts = vec![0i64; self.binner.len()];

        for value in values {
            let value_f64 = value.to_f64();
            let bin_idx = self.binner.bin_of_value(value_f64);
            counts[bin_idx] += 1;
        }

        // If cumulative, accumulate counts
        if self.cumulative {
            for i in 1..self.binner.len() {
                counts[i] += counts[i - 1];
            }
        }

        // Generate bin centers
        let mut mins = Vec::with_capacity(self.binner.len());
        let mut centers = Vec::with_capacity(self.binner.len());
        let mut maxs = Vec::with_capacity(self.binner.len());
        for i in 0..self.binner.len() {
            let (min, max) = self.binner.bin_bounds(i);
            mins.push(min);
            centers.push(self.binner.center_of_bin(i));
            maxs.push(max);
        }

        self.data.add_column("xmin", Box::new(FloatVec(mins)));
        self.data.add_column("x", Box::new(FloatVec(centers)));
        self.data.add_column("xmax", Box::new(FloatVec(maxs)));
        self.data.add_column("count", Box::new(IntVec(counts)));
        self.mapping.set(
            Aesthetic::X(AestheticDomain::Continuous),
            AesValue::column("x"),
        );
        self.mapping.set(
            Aesthetic::Xmin(AestheticDomain::Continuous),
            AesValue::column("xmin"),
        );
        self.mapping.set(
            Aesthetic::Xmax(AestheticDomain::Continuous),
            AesValue::column("xmax"),
        );
        self.mapping.set(
            Aesthetic::Y(AestheticDomain::Continuous),
            AesValue::column("count"),
        );
    }
}


struct GroupedValueBinner {
    binner: Binner,
    cumulative: bool,
    data: DataFrame,
    mapping: AesMap,
}

impl GroupedValueBinner {
    fn new(binner: Binner, cumulative: bool) -> Self {
        Self {
            binner,
            cumulative,
            data: DataFrame::new(),
            mapping: AesMap::new(),
        }
    }
}

impl DiscreteContinuousVisitor2 for GroupedValueBinner {
    fn visit<G: Vectorable + DiscreteType, T: Vectorable + ContinuousType>(
        &mut self,
        group_values: impl Iterator<Item = G>,
        x_values: impl Iterator<Item = T>,
    ) {
        let mut groups: HashMap<G::Sortable, Vec<i64>> = HashMap::new();
        for (group, x) in group_values.zip(x_values) {
            let value_f64 = x.to_f64();
            let bin_idx = self.binner.bin_of_value(value_f64);
            let entry = groups
                .entry(group.to_sortable())
                .or_insert_with(|| vec![0i64; self.binner.len()]);
            entry[bin_idx] += 1;
        }

        let mut pairs = groups.into_iter().collect::<Vec<_>>();
        pairs.sort_by_key(|(group, _)| group.clone());

        let n = pairs.len() * self.binner.len();
        let mut mins = Vec::with_capacity(n);
        let mut centers = Vec::with_capacity(n);
        let mut maxs = Vec::with_capacity(n);
        let mut counts = Vec::with_capacity(n);
        let mut group_values = Vec::with_capacity(n);
        for (group, group_counts) in pairs.into_iter() {
            let group = G::from_sortable(group);
            let mut group_counts = group_counts;
            for i in 0..self.binner.len() {
                if self.cumulative && i > 0 {
                    group_counts[i] += group_counts[i - 1];
                }
                let count = group_counts[i];
                counts.push(count);
                let (min, max) = self.binner.bin_bounds(i);
                mins.push(min);
                centers.push(self.binner.center_of_bin(i));
                maxs.push(max);
                group_values.push(group.clone());
            }
        }

        self.data.add_column("xmin", Box::new(FloatVec(mins)));
        self.data.add_column("x", Box::new(FloatVec(centers)));
        self.data.add_column("xmax", Box::new(FloatVec(maxs)));
        self.data.add_column("count", Box::new(IntVec(counts)));
        self.data.add_column("group", G::make_vector(group_values));

        self.mapping.set(
            Aesthetic::X(AestheticDomain::Continuous),
            AesValue::column("x"),
        );
        self.mapping.set(
            Aesthetic::Xmin(AestheticDomain::Continuous),
            AesValue::column("xmin"),
        );
        self.mapping.set(
            Aesthetic::Xmax(AestheticDomain::Continuous),
            AesValue::column("xmax"),
        );
        self.mapping.set(
            Aesthetic::Y(AestheticDomain::Continuous),
            AesValue::column("count"),
        );
        self.mapping.set(
            Aesthetic::Group,
            AesValue::column("group"),
        );
    }
}

fn bin_grouped_data<'a, T: ContinuousType>(
    x_values: impl Iterator<Item = T>,
    group_values: VectorIter<'a>,
    binner: &Binner,
    cumulative: bool,
    data: &mut DataFrame,
    mapping: &mut AesMap,
) {
    match group_values {
        VectorIter::Int(iterator) => {
            let group_values =
                bin_grouped_data_inner(x_values, iterator, binner, cumulative, data, mapping);
            data.add_column("group", Box::new(IntVec(group_values)));
            mapping.set(Aesthetic::Group, AesValue::column("group"));
        }
        VectorIter::Float(iterator) => {
            let group_values =
                bin_grouped_data_inner(x_values, iterator, binner, cumulative, data, mapping);
            data.add_column("group", Box::new(FloatVec(group_values)));
            mapping.set(Aesthetic::Group, AesValue::column("group"));
        }
        VectorIter::Str(iterator) => {
            let group_values =
                bin_grouped_data_inner(x_values, iterator.map(|s| s.to_string()), binner, cumulative, data, mapping);
            data.add_column("group", Box::new(StrVec(group_values)));
            mapping.set(Aesthetic::Group, AesValue::column("group"));
        }
        VectorIter::Bool(iterator) => {
            let group_values =
                bin_grouped_data_inner(x_values, iterator, binner, cumulative, data, mapping);
            data.add_column("group", Box::new(BoolVec(group_values)));
            mapping.set(Aesthetic::Group, AesValue::column("group"));
        }
    }
}

fn bin_grouped_data_inner<T: ContinuousType, G: PrimitiveType>(
    x_values: impl Iterator<Item = T>,
    group_values: impl Iterator<Item = G>,
    binner: &Binner,
    cumulative: bool,
    data: &mut DataFrame,
    mapping: &mut AesMap,
) -> Vec<G> {
    let mut groups: HashMap<G::Sortable, Vec<i64>> = HashMap::new();
    for (x, group) in x_values.zip(group_values) {
        let value_f64 = x.to_f64();
        let bin_idx = binner.bin_of_value(value_f64);
        let entry = groups
            .entry(group.to_sortable())
            .or_insert_with(|| vec![0i64; binner.len()]);
        entry[bin_idx] += 1;
    }

    let mut pairs = groups.into_iter().collect::<Vec<_>>();
    pairs.sort_by_key(|(group, _)| group.clone());

    let n = pairs.len() * binner.len();
    let mut mins = Vec::with_capacity(n);
    let mut centers = Vec::with_capacity(n);
    let mut maxs = Vec::with_capacity(n);
    let mut counts = Vec::with_capacity(n);
    let mut group_values = Vec::with_capacity(n);

    for (group, group_counts) in pairs.into_iter() {
        let group = G::from_sortable(group);
        let mut group_counts = group_counts;
        for i in 0..binner.len() {
            if cumulative && i > 0 {
                group_counts[i] += group_counts[i - 1];
            }
            let count = group_counts[i];
            counts.push(count);
            let (min, max) = binner.bin_bounds(i);
            mins.push(min);
            centers.push(binner.center_of_bin(i));
            maxs.push(max);
            group_values.push(group.clone());
        }
    }

    data.add_column("xmin", Box::new(FloatVec(mins)));
    data.add_column("x", Box::new(FloatVec(centers)));
    data.add_column("xmax", Box::new(FloatVec(maxs)));
    data.add_column("count", Box::new(IntVec(counts)));
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
    group_values
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::dataframe::DataFrame;

    #[test]
    fn test_bin_basic() {
        let mut df = DataFrame::new();
        df.add_column("x", Box::new(FloatVec(vec![1.0, 1.5, 2.0, 2.5, 3.0, 3.5])));

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Continuous);

        let bin = Bin::with_count(3);
        let result = bin.apply(Box::new(df), &mapping);
        assert!(result.is_ok());

        let (data, new_mapping) = result.unwrap().unwrap();

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
        df.add_column("x", Box::new(IntVec(vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10])));

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Continuous);

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
        mapping.x("x", AestheticDomain::Continuous);

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
        mapping.x("x", AestheticDomain::Continuous);

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
            Box::new(FloatVec(vec![1.0, f64::NAN, 2.0, 3.0, f64::NAN, 4.0, 5.0])),
        );

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Continuous);

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
        df.add_column(
            "x",
            Box::new(FloatVec(vec![
                0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0,
            ])),
        );

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Continuous);

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
        use crate::utils::dataframe::StrVec;

        // Create data with two groups
        let mut df = DataFrame::new();
        df.add_column(
            "x",
            Box::new(FloatVec(vec![
                1.0, 1.5, 2.0, 2.5, 3.0, // Group A
                2.0, 2.5, 3.0, 3.5, 4.0, // Group B
            ])),
        );
        df.add_column(
            "group",
            Box::new(StrVec(vec![
                "A".to_string(),
                "A".to_string(),
                "A".to_string(),
                "A".to_string(),
                "A".to_string(),
                "B".to_string(),
                "B".to_string(),
                "B".to_string(),
                "B".to_string(),
                "B".to_string(),
            ])),
        );

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Continuous);
        mapping.set(Aesthetic::Fill, AesValue::column("group"));

        let bin = Bin::with_count(3);
        let (data, new_mapping) = bin.apply(Box::new(df), &mapping).unwrap().unwrap();

        // Check that y is mapped to count
        assert_eq!(
            new_mapping.get(&Aesthetic::Y(AestheticDomain::Continuous)),
            Some(&AesValue::column("count"))
        );

        // Check that group column is preserved
        let group_col = data.get("group").unwrap();
        assert!(group_col.iter_str().is_some());

        // Check that we have data for both groups
        let groups: Vec<String> = group_col
            .iter_str()
            .unwrap()
            .map(|s| s.to_string())
            .collect();
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
        use crate::utils::dataframe::StrVec;

        // Create data with two grouping dimensions
        let mut df = DataFrame::new();
        df.add_column(
            "x",
            Box::new(FloatVec(vec![
                1.0, 2.0, // Color A, Shape X
                3.0, 4.0, // Color A, Shape Y
                1.5, 2.5, // Color B, Shape X
                3.5, 4.5, // Color B, Shape Y
            ])),
        );
        df.add_column(
            "color",
            Box::new(StrVec(vec![
                "A".to_string(),
                "A".to_string(),
                "A".to_string(),
                "A".to_string(),
                "B".to_string(),
                "B".to_string(),
                "B".to_string(),
                "B".to_string(),
            ])),
        );
        df.add_column(
            "shape",
            Box::new(StrVec(vec![
                "X".to_string(),
                "X".to_string(),
                "Y".to_string(),
                "Y".to_string(),
                "X".to_string(),
                "X".to_string(),
                "Y".to_string(),
                "Y".to_string(),
            ])),
        );

        let mut mapping = AesMap::new();
        mapping.x("x", AestheticDomain::Continuous);
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
