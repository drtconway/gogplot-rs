// Statistical transformations for plot layers

use crate::error::PlotError;
use crate::layer::{Layer, Stat};
use crate::stat::bin::Bin;
use crate::stat::boxplot::Boxplot;
use crate::stat::count::Count;
use crate::stat::density::Density;
use crate::stat::smooth::Smooth;
use crate::stat::summary::Summary;
use crate::stat::Stat;

/// Apply statistical transformation to a single layer
///
/// This function transforms a layer that has a stat other than Identity.
/// The transformed data is stored in computed_data, and the
/// aesthetic mapping is updated in computed_mapping.
///
/// If the layer has no data, plot_data will be used if available.
/// Layer mappings are merged with plot_default_aes (layer takes precedence).
pub fn apply_stat_to_layer(
    layer: &mut Layer,
    plot_data: Option<&dyn crate::data::DataSource>,
    plot_mapping: &crate::aesthetics::AesMap,
) -> Result<(), PlotError> {
    // Check if this layer needs transformation
    let needs_transform = !matches!(layer.stat, Stat::Identity | Stat::None);

    if !needs_transform {
        return Ok(());
    }

    // Get data source - either layer-specific or plot-level
    let data = if let Some(layer_data) = layer.data.take() {
        layer_data
    } else if let Some(plot_data) = plot_data {
        // Clone plot-level data for this layer
        plot_data.clone_box()
    } else {
        return Ok(());
    };

    // Merge plot-level aesthetics with layer-specific mappings
    // Layer mappings take precedence over plot defaults
    let mut merged_mapping = plot_mapping.clone();
    for (aes, value) in layer.get_mapping(plot_mapping).iter() {
        merged_mapping.set(*aes, value.clone());
    }

    // Apply the stat transformation
    match &layer.stat {
        Stat::None => {
            // This should never happen - None should be resolved during layer construction
            panic!("Stat::None should be resolved before stat application");
        }
        Stat::Count => {
            let stat_result = Count.apply(data, &merged_mapping)?;
            if let Some((transformed_data, new_mapping)) = stat_result {
                layer.computed_data = Some(transformed_data);
                layer.computed_mapping = Some(new_mapping);
            }
        }
        Stat::Bin(strategy) => {
            let bin_stat = Bin {
                strategy: strategy.clone(),
            };
            let stat_result = bin_stat.apply(data, &merged_mapping)?;
            if let Some((transformed_data, new_mapping)) = stat_result {
                layer.computed_data = Some(transformed_data);
                layer.computed_mapping = Some(new_mapping);
            }
        }
        Stat::Density { adjust, n } => {
            let density_stat = Density::new().adjust(*adjust).n(*n);
            let stat_result = density_stat.apply(data, &merged_mapping)?;
            if let Some((transformed_data, new_mapping)) = stat_result {
                layer.computed_data = Some(transformed_data);
                layer.computed_mapping = Some(new_mapping);
            }
        }
        Stat::Boxplot { coef } => {
            let boxplot_stat = Boxplot::new().with_coef(*coef);
            let stat_result = boxplot_stat.apply(data, &merged_mapping)?;
            if let Some((transformed_data, new_mapping)) = stat_result {
                eprintln!("Boxplot stat produced {} rows", transformed_data.len());
                layer.computed_data = Some(transformed_data);
                layer.computed_mapping = Some(new_mapping);
            } else {
                eprintln!("Boxplot stat returned None");
            }
        }
        Stat::Summary(aesthetics) => {
            let summary_stat = Summary {
                aesthetics: aesthetics.clone(),
            };
            let stat_result = summary_stat.apply(data, &merged_mapping)?;
            if let Some((transformed_data, new_mapping)) = stat_result {
                layer.computed_data = Some(transformed_data);
                layer.computed_mapping = Some(new_mapping);
            }
        }
        Stat::Smooth { method, level, n, span } => {
            let smooth_stat = Smooth::new()
                .method(*method)
                .level(*level)
                .n(*n)
                .span(*span);
            let stat_result = smooth_stat.apply(data, &merged_mapping)?;
            if let Some((transformed_data, new_mapping)) = stat_result {
                layer.computed_data = Some(transformed_data);
                layer.computed_mapping = Some(new_mapping);
            }
        }
        Stat::Identity => {
            // Put the data back and continue
            layer.data = Some(data);
        }
    }

    Ok(())
}
