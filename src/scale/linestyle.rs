use crate::data::{DiscreteType, VectorIter};
use crate::scale::traits::DiscreteDomainScale;
use crate::utils::set::DiscreteSet;
use crate::utils::DashPatterns;
use crate::visuals::LineStyle;

/// Discrete linestyle scale that maps categories to line styles.
#[derive(Debug, Clone)]
pub struct LineStyleScale {
    linestyles: Vec<LineStyle>,
    elements: DiscreteSet,
}

impl LineStyleScale {
    /// Create a new discrete linestyle scale with a set of linestyles.
    pub fn new(linestyles: Vec<LineStyle>) -> Self {
        Self {
            linestyles,
            elements: DiscreteSet::new(),
        }
    }

    /// Create a default discrete linestyle scale with distinct patterns.
    /// Uses the DashPatterns iterator to generate non-redundant patterns.
    pub fn default_linestyles() -> Self {
        let patterns: Vec<LineStyle> = DashPatterns::new()
            .take(6)
            .map(|pattern| LineStyle::from(pattern))
            .collect();
        Self::new(patterns)
    }
}

impl Default for LineStyleScale {
    fn default() -> Self {
        Self::default_linestyles()
    }
}

impl super::traits::ScaleBase for LineStyleScale {
    fn train<'a>(&mut self, iter: VectorIter<'a>) {
        self.train_discrete(iter);
    }
}

impl super::traits::DiscreteDomainScale for LineStyleScale {
    fn categories(&self) -> &DiscreteSet {
        &self.elements
    }

    fn add_categories(&mut self, categories: DiscreteSet) {
        self.elements.union(&categories);
    }
}

impl super::traits::LineStyleRangeScale for LineStyleScale {
    fn map_value<T: DiscreteType>(&self, value: &T) -> Option<LineStyle> {
        let ordinal = self.elements.ordinal(value)?;
        let linestyle = self.linestyles[ordinal % self.linestyles.len()].clone();
        Some(linestyle)
    }
}

impl super::traits::LineStyleScale for LineStyleScale {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discrete_linestyle_new() {
        let linestyles = vec![LineStyle::Solid, LineStyle::from("-")];
        let scale = LineStyleScale::new(linestyles);
        assert_eq!(scale.linestyles.len(), 2);
    }

    #[test]
    fn test_default_linestyles() {
        let scale = LineStyleScale::default_linestyles();
        assert_eq!(scale.linestyles.len(), 6);
        // Ensure all patterns are distinct
        for i in 0..scale.linestyles.len() {
            for j in (i + 1)..scale.linestyles.len() {
                assert_ne!(
                    scale.linestyles[i], scale.linestyles[j],
                    "Linestyles at positions {} and {} should be different",
                    i, j
                );
            }
        }
    }
}
