// Position adjustments for plot layers

use crate::error::PlotError;
use crate::layer::{Layer, Position};
use crate::position::stack::Stack;
use crate::position::PositionAdjust;

/// Apply position adjustments to layers
///
/// This function adjusts layer data for position adjustments like Stack and Dodge.
/// The adjusted data is stored in each layer's computed_data field, and the
/// aesthetic mapping is updated in computed_mapping.
pub fn apply_positions(layers: &mut [Layer]) -> Result<(), PlotError> {
    let num_layers = layers.len();
    for i in 0..num_layers {
        // Check if this layer needs position adjustment
        let needs_adjustment = !matches!(layers[i].position, Position::Identity);

        if !needs_adjustment {
            continue;
        }

        // Get the data to adjust - take ownership of computed_data
        let data = if let Some(computed_data) = layers[i].computed_data.take() {
            computed_data
        } else {
            // Position adjustments only work on computed data (from stats)
            // If there's no computed data, nothing to adjust
            continue;
        };

        // Get the mapping - either computed_mapping (from stat) or original mapping
        let mapping = if let Some(ref computed_mapping) = layers[i].computed_mapping {
            computed_mapping
        } else {
            &layers[i].mapping
        };

        // Apply the position adjustment
        match &layers[i].position {
            Position::Stack => {
                let position_result = Stack.apply(data, mapping)?;
                if let Some((adjusted_data, new_mapping)) = position_result {
                    layers[i].computed_data = Some(adjusted_data);
                    layers[i].computed_mapping = Some(new_mapping);
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
