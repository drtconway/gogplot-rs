use std::collections::HashMap;

use super::Position;
use crate::aesthetics::{AesMap, AesValue, Aesthetic, AestheticDomain};
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
    ) -> Result<Option<AesMap>, PlotError> {
        if !mapping.contains(Aesthetic::Group) {
            // No grouping aesthetic, cannot dodge
            return Err(PlotError::MissingAesthetic {
                aesthetic: Aesthetic::Group,
            });
        }
        
        // Check for discrete X - this is what we have before scale application
        let x_discrete_aes = if mapping.contains(Aesthetic::X(AestheticDomain::Discrete)) {
            Aesthetic::X(AestheticDomain::Discrete)
        } else {
            return Err(PlotError::MissingAesthetic {
                aesthetic: Aesthetic::X(AestheticDomain::Discrete),
            });
        };

        // Get the discrete X values - we'll create continuous offsets from these
        let x_discrete_iter = mapping
            .get_vector_iter(&x_discrete_aes, data.as_ref())
            .unwrap();

        let group_values = mapping
            .get_vector_iter(&Aesthetic::Group, data.as_ref())
            .unwrap();

        let mut dodger = GroupDodger::new(self.width, self.padding);

        // Process both x and group values together
        dodger.process(x_discrete_iter, group_values)?;

        let x_offsets = dodger.x_offsets;
        let widths = dodger.widths;

        log::debug!("dodge - x_offsets: {:?}", x_offsets);
        log::debug!("dodge - widths: {:?}", widths);

        // Create new mapping keeping X(Discrete) and adding XOffset and Width
        let mut new_mapping = AesMap::new();
        
        log::debug!("dodge - input mapping aesthetics: {:?}", mapping.aesthetics().collect::<Vec<_>>());
        
        for (aes, aes_value) in mapping.iter() {
            log::debug!("dodge - processing aesthetic {:?}", aes);
            new_mapping.set(*aes, aes_value.clone());
        }
        
        // Add dodge offset and width as new aesthetics
        new_mapping.set(Aesthetic::XOffset, AesValue::vector(x_offsets, None));
        new_mapping.set(Aesthetic::Width, AesValue::vector(widths, None));
        
        log::debug!("dodge - output mapping aesthetics: {:?}", new_mapping.aesthetics().collect::<Vec<_>>());

        Ok(Some(new_mapping))
    }
}

struct GroupDodger {
    x_offsets: Vec<f64>,
    widths: Vec<f64>,
    width: Option<f64>,
    padding: f64,
}

impl GroupDodger {
    fn new(width: Option<f64>, padding: f64) -> Self {
        Self {
            x_offsets: Vec::new(),
            widths: Vec::new(),
            width,
            padding,
        }
    }

    fn process(
        &mut self,
        x_iter: crate::data::VectorIter<'_>,
        group_iter: crate::data::VectorIter<'_>,
    ) -> Result<(), PlotError> {
        use crate::data::DiscreteValue;

        // Collect x and group values into discrete form
        let x_values: Vec<DiscreteValue> = x_iter.to_discrete_iter().collect();

        let group_values: Vec<DiscreteValue> = group_iter.to_discrete_iter().collect();

        // For each unique x value, find unique groups and assign indices
        let mut groups_per_x: HashMap<DiscreteValue, Vec<DiscreteValue>> = HashMap::new();
        for (x_val, group_val) in x_values.iter().zip(group_values.iter()) {
            groups_per_x
                .entry(x_val.clone())
                .or_insert_with(Vec::new)
                .push(group_val.clone());
        }

        // Find all unique groups across ALL x positions (for global group indexing)
        let mut all_groups: Vec<DiscreteValue> = Vec::new();
        for groups in groups_per_x.values() {
            for group in groups {
                if !all_groups.contains(group) {
                    all_groups.push(group.clone());
                }
            }
        }
        all_groups.sort();
        all_groups.dedup();
        
        // Create global group index mapping
        let mut global_group_indices: HashMap<DiscreteValue, usize> = HashMap::new();
        for (idx, group) in all_groups.iter().enumerate() {
            global_group_indices.insert(group.clone(), idx);
        }
        let max_groups = all_groups.len();

        // Calculate dodge width (default 0.9)
        let dodge_width = self.width.unwrap_or(0.9);

        // Use max_groups to ensure consistent bar width across all x positions
        let bar_width = dodge_width / (max_groups as f64);
        let padding_width = bar_width * self.padding;
        let effective_bar_width = bar_width - padding_width;

        // Calculate offsets and widths for each (x, group) pair
        self.x_offsets = Vec::with_capacity(x_values.len());
        self.widths = Vec::with_capacity(x_values.len());
        
        for (_x_val, group_val) in x_values.iter().zip(group_values.iter()) {
            let group_idx = global_group_indices.get(group_val).unwrap_or(&0);

            // Center the group of bars around x=0 (will be added to discrete position later)
            let start_offset = -dodge_width / 2.0;
            let bar_offset = start_offset + (*group_idx as f64 + 0.5) * bar_width;

            self.x_offsets.push(bar_offset);
            self.widths.push(effective_bar_width);
        }

        Ok(())
    }
}
