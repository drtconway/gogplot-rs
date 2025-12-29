use std::collections::HashMap;

use super::Position;
use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain};
use crate::data::DataSource;
use crate::error::PlotError;

/// Dodge position adjustment
///
/// Places bars side-by-side at the same x position, making each bar narrower
/// to fit multiple groups without overlapping.
pub struct Dodge {
    /// Total width for all bars at one x position (None = auto-detect)
    pub width: Option<f64>,
    /// Padding between dodged bars as a fraction of bar width (default 0.1)
    pub padding: f64,
}

impl Default for Dodge {
    fn default() -> Self {
        Self {
            width: None,
            padding: 0.1, // 10% padding between bars within a cluster
        }
    }
}

impl Position for Dodge {
    fn apply(
        &self,
        data: &Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<Option<(Box<dyn DataSource>, AesMap)>, PlotError> {
        if !mapping.contains(Aesthetic::Group) {
            // No grouping aesthetic, cannot dodge
            return Err(PlotError::MissingAesthetic {
                aesthetic: Aesthetic::Group,
            });
        }
        if !mapping.contains(Aesthetic::X(AestheticDomain::Discrete)) {
            // No discrete x aesthetic, cannot dodge
            return Err(PlotError::MissingAesthetic {
                aesthetic: Aesthetic::X(AestheticDomain::Discrete),
            });
        }

        let mut x_like_data: HashMap<Aesthetic, Vec<f64>> = HashMap::new();
        for aes in mapping.aesthetics() {
            if aes.is_x_like() && aes.is_continuous() {
                let x_like_values = mapping.get_iter_float(aes, data.as_ref()).unwrap();
                let x_like_values = x_like_values.collect();
                x_like_data.insert(*aes, x_like_values);
            }
        }
        todo!()
    }
}
