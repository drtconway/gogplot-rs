// Position adjustments for plot layers

use crate::error::PlotError;
use crate::layer::Layer;
use crate::scale::ScaleSet;

/// Apply position adjustments to layers
///
/// This function adjusts layer data for position adjustments like Stack and Dodge.
/// The adjusted data is stored in each layer's computed_data field, and the
/// aesthetic mapping is updated in computed_mapping.
pub fn apply_positions(
    layers: &mut [Layer],
    plot_data: Option<&dyn crate::data::DataSource>,
    plot_mapping: &crate::aesthetics::AesMap,
    scales: &ScaleSet,
) -> Result<(), PlotError> {

    Ok(())
}
