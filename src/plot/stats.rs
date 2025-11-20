// Statistical transformations for plot layers

use crate::error::PlotError;
use crate::layer::{Layer, Stat};
use crate::stat::StatTransform;
use crate::stat::count::Count;

/// Apply statistical transformations to layers
///
/// This function transforms layers that have stats other than Identity.
/// The transformed data is stored in each layer's data field, and the
/// aesthetic mapping is updated to reflect the transformation.
///
/// Note: Currently, this only works for layers that have their own data.
/// Layers that rely on plot-level data cannot be transformed yet.
pub fn apply_stats(layers: &mut [Layer]) -> Result<(), PlotError> {
    // We need to transform each layer that has a non-Identity stat
    // We'll process layers in reverse order so we can swap data out
    let num_layers = layers.len();
    for i in 0..num_layers {
        // Check if this layer needs transformation
        let needs_transform = !matches!(layers[i].stat, Stat::Identity);

        if !needs_transform {
            continue;
        }

        // If layer doesn't have data, it can't be transformed
        // (stat transformations need owned data)
        if layers[i].data.is_none() {
            continue;
        }

        // Take ownership of the layer's data
        let data = layers[i].data.take().unwrap();

        // Apply the stat transformation
        let stat_result = match &layers[i].stat {
            Stat::Count => Count.apply(data, &layers[i].mapping)?,
            Stat::Identity => {
                // Put the data back and continue
                layers[i].data = Some(data);
                continue;
            }
            Stat::Bin | Stat::Smooth => {
                // Not implemented yet - put data back
                layers[i].data = Some(data);
                continue;
            }
        };

        // If transformation succeeded, store the result
        if let Some((transformed_data, new_mapping)) = stat_result {
            layers[i].data = Some(transformed_data);

            // Replace the layer's mapping with the new mapping from the stat
            // The stat knows best what the transformed data looks like
            layers[i].mapping = new_mapping;
        } else {
            // No transformation needed - data is already back in place from match arm
        }
    }

    Ok(())
}
