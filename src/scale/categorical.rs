use std::collections::HashMap;

use super::{ContinuousScale, ScaleBase};

pub struct Builder {
    drop: bool,
    breaks: Option<Vec<String>>,
    labels: Option<Vec<String>>,
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

impl Builder {
    pub fn new() -> Self {
        Self {
            drop: true,
            breaks: None,
            labels: None,
        }
    }

    pub fn drop_unused(mut self) -> Self {
        self.drop = true;
        self
    }

    pub fn keep_unused(mut self) -> Self {
        self.drop = false;
        self
    }

    pub fn breaks(mut self, breaks: Vec<String>) -> Self {
        self.breaks = Some(breaks);
        self
    }

    pub fn labels(mut self, labels: Vec<String>) -> Self {
        self.labels = Some(labels);
        self
    }

    pub fn build(self, categories: Vec<String>, range: (f64, f64)) -> Catagorical {
        let n = categories.len() as f64;
        let step = (range.1 - range.0) / n;

        let mapping: HashMap<String, f64> = categories
            .into_iter()
            .enumerate()
            .map(|(i, cat)| (cat, range.0 + i as f64 * step + step / 2.0))
            .collect();

        Catagorical::new(mapping)
    }
}

pub struct Catagorical {
    pub(crate) mapping: HashMap<String, f64>,
    breaks: Vec<f64>,
    labels: Vec<String>,
}

impl Catagorical {
    pub fn new(mapping: HashMap<String, f64>) -> Self {
        let mut items: Vec<_> = mapping.iter().collect();
        items.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());

        let breaks = items.iter().map(|(_, v)| **v).collect();
        let labels = items.iter().map(|(k, _)| (*k).to_string()).collect();

        Self {
            mapping,
            breaks,
            labels,
        }
    }

    pub fn map_category(&self, data: &str) -> f64 {
        *self.mapping.get(data).unwrap_or(&0.0)
    }
}

impl ScaleBase for Catagorical {
    fn train(&mut self, data: &[&dyn crate::data::GenericVector]) {
        // Extract unique categories from string data
        use std::collections::HashSet;

        let mut all_categories = HashSet::new();

        for vec in data {
            if let Some(str_iter) = vec.iter_str() {
                // String data - use directly
                for s in str_iter {
                    all_categories.insert(s.to_string());
                }
            } else if let Some(int_iter) = vec.iter_int() {
                // Integer data - convert to strings for categorical use
                for i in int_iter {
                    all_categories.insert(i.to_string());
                }
            } else if let Some(float_iter) = vec.iter_float() {
                // Float data - convert to strings for categorical use
                for f in float_iter {
                    all_categories.insert(f.to_string());
                }
            }
        }

        // If we found categories and the mapping is empty, initialize it
        if !all_categories.is_empty() && self.mapping.is_empty() {
            // Sort categories alphabetically for consistent ordering
            let mut categories: Vec<String> = all_categories.into_iter().collect();
            categories.sort();

            // Assign positions evenly spaced in [0, 1]
            let n = categories.len() as f64;
            for (i, cat) in categories.iter().enumerate() {
                // Position categories at 0.5/n, 1.5/n, 2.5/n, ... (centered in their bins)
                let pos = (i as f64 + 0.5) / n;
                self.mapping.insert(cat.clone(), pos);
            }

            // Update breaks and labels
            let mut items: Vec<_> = self.mapping.iter().collect();
            items.sort_by(|a, b| a.1.partial_cmp(b.1).unwrap());

            self.breaks = items.iter().map(|(_, v)| **v).collect();
            self.labels = items.iter().map(|(k, _)| (*k).to_string()).collect();
        }
    }
}

impl ContinuousScale for Catagorical {
    fn map_value(&self, value: f64) -> Option<f64> {
        // For categorical scales, this maps a numeric position to itself
        // Always return Some since categorical scales don't have domain bounds
        Some(value)
    }

    fn map_category(&self, category: &str) -> Option<f64> {
        // Map category string to numeric position
        self.mapping.get(category).copied()
    }

    fn inverse(&self, value: f64) -> f64 {
        // Inverse of identity
        value
    }

    fn breaks(&self) -> &[f64] {
        &self.breaks
    }

    fn labels(&self) -> &[String] {
        &self.labels
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default() {
        let builder = Builder::new();
        assert!(builder.drop);
        assert!(builder.breaks.is_none());
        assert!(builder.labels.is_none());
    }

    #[test]
    fn test_builder_drop_unused() {
        let builder = Builder::new().drop_unused();
        assert!(builder.drop);
    }

    #[test]
    fn test_builder_keep_unused() {
        let builder = Builder::new().keep_unused();
        assert!(!builder.drop);
    }

    #[test]
    fn test_builder_build() {
        let categories = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let scale = Builder::new().build(categories, (0.0, 1.0));

        assert_eq!(scale.mapping.len(), 3);
        assert_eq!(scale.breaks.len(), 3);
        assert_eq!(scale.labels.len(), 3);
    }

    #[test]
    fn test_categorical_new() {
        let mut mapping = HashMap::new();
        mapping.insert("low".to_string(), 0.25);
        mapping.insert("medium".to_string(), 0.5);
        mapping.insert("high".to_string(), 0.75);

        let scale = Catagorical::new(mapping);

        assert_eq!(scale.breaks.len(), 3);
        assert_eq!(scale.labels.len(), 3);
        // Should be sorted by value
        assert_eq!(scale.labels[0], "low");
        assert_eq!(scale.labels[1], "medium");
        assert_eq!(scale.labels[2], "high");
    }

    #[test]
    fn test_categorical_map_category() {
        let categories = vec!["A".to_string(), "B".to_string(), "C".to_string()];
        let scale = Builder::new().build(categories, (0.0, 3.0));

        let pos_a = scale.map_category("A");
        let pos_b = scale.map_category("B");
        let pos_c = scale.map_category("C");

        assert!(pos_a < pos_b);
        assert!(pos_b < pos_c);
        assert!(pos_a >= 0.0 && pos_a <= 3.0);
        assert!(pos_c >= 0.0 && pos_c <= 3.0);
    }

    #[test]
    fn test_categorical_map_missing() {
        let categories = vec!["A".to_string(), "B".to_string()];
        let scale = Builder::new().build(categories, (0.0, 1.0));

        let missing = scale.map_category("Z");
        assert_eq!(missing, 0.0); // Default value
    }

    #[test]
    fn test_categorical_continuous_scale_impl() {
        let categories = vec!["X".to_string(), "Y".to_string()];
        let scale = Builder::new().build(categories, (0.0, 1.0));

        // Test ContinuousScale trait methods
        assert_eq!(scale.map_value(0.5), Some(0.5)); // Identity mapping
        assert_eq!(scale.inverse(0.3), 0.3); // Identity inverse
        assert_eq!(scale.breaks().len(), 2);
        assert_eq!(scale.labels().len(), 2);
    }

    #[test]
    fn test_categorical_evenly_spaced() {
        let categories = vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(),
            "D".to_string(),
        ];
        let scale = Builder::new().build(categories, (0.0, 4.0));

        let pos_a = scale.map_category("A");
        let pos_b = scale.map_category("B");
        let pos_c = scale.map_category("C");
        let pos_d = scale.map_category("D");

        // Should be evenly spaced (step = 1.0, centered at 0.5, 1.5, 2.5, 3.5)
        assert!((pos_a - 0.5).abs() < 0.01);
        assert!((pos_b - 1.5).abs() < 0.01);
        assert!((pos_c - 2.5).abs() < 0.01);
        assert!((pos_d - 3.5).abs() < 0.01);
    }
}
