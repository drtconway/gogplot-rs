// Statistical transformations for plot layers

use crate::error::PlotError;
use crate::layer::{Layer, Stat};
use crate::stat::bin::Bin;
use crate::stat::count::Count;
use crate::stat::density::Density;
use crate::stat::summary::Summary;
use crate::stat::StatTransform;

/// Apply statistical transformations to layers
///
/// This function transforms layers that have stats other than Identity.
/// The transformed data is stored in computed_data, and the
/// aesthetic mapping is updated in computed_mapping.
///
/// Layers without their own data will use plot_data if available.
/// Layers without their own mappings will inherit from plot_default_aes.
pub fn apply_stats(
    layers: &mut [Layer],
    plot_data: Option<&dyn crate::data::DataSource>,
    plot_default_aes: &crate::aesthetics::AesMap,
) -> Result<(), PlotError> {
    // We need to transform each layer that has a non-Identity stat
    // We'll process layers in reverse order so we can swap data out
    let num_layers = layers.len();
    for i in 0..num_layers {
        // Check if this layer needs transformation
        let needs_transform = !matches!(layers[i].stat, Stat::Identity);

        if !needs_transform {
            continue;
        }

        // Get data source - either layer-specific or plot-level
        let data = if let Some(layer_data) = layers[i].data.take() {
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

        // Apply the stat transformation
        match &layers[i].stat {
            Stat::Count => {
                let stat_result = Count.apply(data, &merged_mapping)?;
                if let Some((transformed_data, new_mapping)) = stat_result {
                    layers[i].computed_data = Some(transformed_data);
                    layers[i].computed_mapping = Some(new_mapping);
                }
            }
            Stat::Bin(strategy) => {
                let bin_stat = Bin {
                    strategy: strategy.clone(),
                };
                let stat_result = bin_stat.apply(data, &merged_mapping)?;
                if let Some((transformed_data, new_mapping)) = stat_result {
                    layers[i].computed_data = Some(transformed_data);
                    layers[i].computed_mapping = Some(new_mapping);
                }
            }
            Stat::Density { adjust, n } => {
                let density_stat = Density::new().adjust(*adjust).n(*n);
                let stat_result = density_stat.apply(data, &merged_mapping)?;
                if let Some((transformed_data, new_mapping)) = stat_result {
                    layers[i].computed_data = Some(transformed_data);
                    layers[i].computed_mapping = Some(new_mapping);
                }
            }
            Stat::Summary(aesthetics) => {
                let summary_stat = Summary {
                    aesthetics: aesthetics.clone(),
                };
                let stat_result = summary_stat.apply(data, &merged_mapping)?;
                if let Some((transformed_data, new_mapping)) = stat_result {
                    layers[i].computed_data = Some(transformed_data);
                    layers[i].computed_mapping = Some(new_mapping);
                }
            }
            Stat::Identity => {
                // Put the data back and continue
                layers[i].data = Some(data);
            }
            Stat::Smooth => {
                // Not implemented yet - put data back
                layers[i].data = Some(data);
            }
        }
    }

    Ok(())
}
