// Position adjustments for plot layers

use crate::error::PlotError;
use crate::layer::{Layer, Position};
use crate::position::dodge::Dodge;
use crate::position::stack::Stack;
use crate::position::PositionAdjust;

/// Apply position adjustments to layers
///
/// This function adjusts layer data for position adjustments like Stack and Dodge.
/// The adjusted data is stored in each layer's computed_data field, and the
/// aesthetic mapping is updated in computed_mapping.
pub fn apply_positions(
    layers: &mut [Layer],
    plot_data: Option<&dyn crate::data::DataSource>,
    plot_default_aes: &crate::aesthetics::AesMap,
    scales: &crate::plot::scale_set::ScaleSet,
) -> Result<(), PlotError> {
    let num_layers = layers.len();
    for i in 0..num_layers {
        // Check if this layer needs position adjustment
        let needs_adjustment = !matches!(layers[i].position, Position::Identity);

        if !needs_adjustment {
            continue;
        }

        // Get data source - prefer computed_data (from stats), then layer data, then plot data
        let data = if let Some(computed_data) = layers[i].computed_data.take() {
            computed_data
        } else if let Some(layer_data) = layers[i].data.take() {
            layer_data
        } else if let Some(plot_data) = plot_data {
            // Clone plot-level data for this layer
            plot_data.clone_box()
        } else {
            continue;
        };

        // Merge plot-level aesthetics with layer-specific mappings
        // Layer mappings take precedence over plot defaults
        let mut merged_mapping = plot_default_aes.clone();
        for (aes, value) in layers[i].mapping.iter() {
            merged_mapping.set(*aes, value.clone());
        }
        
        // Use computed_mapping if stat transformation created one
        let mapping = if let Some(ref computed_mapping) = layers[i].computed_mapping {
            computed_mapping
        } else {
            &merged_mapping
        };

        // Apply the position adjustment
        match &layers[i].position {
            Position::Stack => {
                let position_result = Stack.apply(data, mapping, scales)?;
                if let Some((adjusted_data, new_mapping, new_scales)) = position_result {
                    layers[i].computed_data = Some(adjusted_data);
                    layers[i].computed_mapping = Some(new_mapping);
                    layers[i].computed_scales = new_scales;
                } else {
                    // Position adjustment returned None, meaning no change needed
                    // Data was consumed, layer has no computed_data now
                }
            }
            Position::Dodge => {
                let dodge = Dodge::default();
                let position_result = dodge.apply(data, mapping, scales)?;
                if let Some((adjusted_data, new_mapping, new_scales)) = position_result {
                    layers[i].computed_data = Some(adjusted_data);
                    layers[i].computed_mapping = Some(new_mapping);
                    layers[i].computed_scales = new_scales;
                } else {
                    // Position adjustment returned None, meaning no change needed
                    // Data was consumed, layer has no computed_data now
                }
            }
            Position::Identity => {
                // No adjustment needed, restore data
                layers[i].computed_data = Some(data);
            }
            _ => {
                // Not implemented yet, restore data
                layers[i].computed_data = Some(data);
            }
        }
    }

    Ok(())
}
