// Scale management for plots

use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::data::{ColumnDataType, DataSource};
use crate::error::PlotError;
use crate::geom::Geom;
use crate::layer::Layer;
use crate::scale::{ColorScale, ContinuousScale, ScaleType, ShapeScale};

/// Container for scales (x, y, color, size, etc.)
pub struct ScaleSet {
    pub x: Option<Box<dyn ContinuousScale>>,
    pub y: Option<Box<dyn ContinuousScale>>,
    pub color: Option<Box<dyn ColorScale>>,
    pub fill: Option<Box<dyn ColorScale>>,
    pub size: Option<Box<dyn ContinuousScale>>,
    pub alpha: Option<Box<dyn ContinuousScale>>,
    pub shape: Option<Box<dyn ShapeScale>>,
    // Add more as needed
}

/// Validates and determines the appropriate scale type for an aesthetic.
///
/// This function checks for conflicts between:
/// - User's explicit type hint (from categorical_column, continuous_column, etc.)
/// - Geom's scale type requirement (from aesthetic_scale_type method)
/// - Data column type (numeric vs string)
///
/// Returns the determined scale type or an error if there's a conflict.
fn validate_and_determine_scale_type(
    aesthetic: Aesthetic,
    aes_value: &AesValue,
    geom: &dyn Geom,
    data: Option<&dyn DataSource>,
) -> Result<ScaleType, PlotError> {
    let user_hint = aes_value.user_hint();
    let geom_requirement = geom.aesthetic_scale_type(aesthetic);
    
    // Get column data type if this is a column mapping
    let column_data_type = if let Some(col_name) = aes_value.as_column_name() {
        data.and_then(|d| {
            // Get the column and inspect its type
            d.get(col_name).map(|col| col.vtype().to_column_data_type())
        })
    } else {
        None
    };
    
    // Validate: string columns MUST use categorical scales
    if let Some(ColumnDataType::String) = column_data_type {
        if matches!(user_hint, Some(ScaleType::Continuous)) {
            return Err(PlotError::string_column_requires_categorical(
                aesthetic,
                aes_value.as_column_name().unwrap(),
            ));
        }
        // String columns override geom requirements - they must be categorical
        return Ok(ScaleType::Categorical);
    }
    
    // Check for conflicts between user hint and geom requirement
    match (user_hint, geom_requirement) {
        // No user hint - use geom requirement
        (None, req) => Ok(req),
        
        // User hint matches geom requirement
        (Some(ScaleType::Continuous), ScaleType::Continuous) => Ok(ScaleType::Continuous),
        (Some(ScaleType::Categorical), ScaleType::Categorical) => Ok(ScaleType::Categorical),
        (Some(ScaleType::Either), ScaleType::Either) => Ok(ScaleType::Either),
        
        // Geom accepts either - use user hint
        (Some(hint), ScaleType::Either) => Ok(hint),
        
        // User wants either - use geom requirement
        (Some(ScaleType::Either), req) => Ok(req),
        
        // Conflict: user wants continuous, geom requires categorical
        (Some(ScaleType::Continuous), ScaleType::Categorical) => {
            Err(PlotError::aesthetic_type_mismatch(
                aesthetic,
                "continuous",
                "categorical",
                "This geom requires categorical scales for this aesthetic",
            ))
        }
        
        // Conflict: user wants categorical, geom requires continuous
        (Some(ScaleType::Categorical), ScaleType::Continuous) => {
            Err(PlotError::aesthetic_type_mismatch(
                aesthetic,
                "categorical",
                "continuous",
                "This geom requires continuous scales for this aesthetic",
            ))
        }
    }
}

impl ScaleSet {
    pub fn new() -> Self {
        Self {
            x: None,
            y: None,
            color: None,
            fill: None,
            size: None,
            alpha: None,
            shape: None,
        }
    }

    /// Create default scales for aesthetics that don't have scales but are mapped to columns
    pub fn create_defaults(
        &mut self,
        layers: &[Layer],
        default_data: Option<&dyn DataSource>,
        default_mapping: &AesMap,
        x_axis_title: &mut Option<String>,
        y_axis_title: &mut Option<String>,
    ) {
        use crate::scale::color::DiscreteColor;
        use crate::scale::continuous::Continuous;
        use crate::scale::shape::DiscreteShape;
        use crate::scale::ScaleType;

        // Find the first layer that maps each aesthetic to determine default scales
        for layer in layers {
            let layer_mapping = layer.get_mapping(default_mapping);
            // X scale
            if self.x.is_none() {
                // Check both computed_mapping (from stats/positions) and original mapping
                let mapping = layer.computed_mapping.as_ref().unwrap_or(&layer_mapping);
                
                // Find the first x-like aesthetic that's mapped to a column
                let col_info = mapping.iter().find_map(|(aes, value)| {
                    if aes.is_x_like() {
                        if value.as_column_name().is_some() {
                            return Some((*aes, value));
                        }
                    }
                    None
                });

                if let Some((x_aesthetic, aes_value)) = col_info {
                    // Get the appropriate data source
                    let data = if let Some(ref computed) = layer.computed_data {
                        Some(computed.as_ref())
                    } else {
                        match &layer.data {
                            Some(d) => Some(d.as_ref()),
                            None => default_data,
                        }
                    };

                    // Use validation function to determine scale type
                    match validate_and_determine_scale_type(
                        x_aesthetic,
                        aes_value,
                        layer.geom.as_ref(),
                        data,
                    ) {
                        Ok(ScaleType::Categorical) => {
                            // Create categorical scale - we'll train it later
                            use crate::scale::categorical::Catagorical;
                            use std::collections::HashMap;
                            self.x = Some(Box::new(Catagorical::new(HashMap::new())));
                        }
                        Ok(ScaleType::Continuous | ScaleType::Either) => {
                            // Create default linear scale
                            if let Ok(scale) = Continuous::new().linear() {
                                self.x = Some(Box::new(scale));
                            }
                        }
                        Err(_) => {
                            // Validation failed - skip creating a scale
                            // The error will be reported elsewhere
                        }
                    }

                    // Set default axis title if not already set
                    if x_axis_title.is_none() {
                        if let Some(col_name) = aes_value.as_column_name() {
                            *x_axis_title = Some(col_name.to_string());
                        }
                    }
                }
            }

            // Y scale
            if self.y.is_none() {
                // Check both computed_mapping (from stats/positions) and original mapping
                let mapping = layer.computed_mapping.as_ref().unwrap_or(&layer_mapping);
                
                // Find the first y-like aesthetic that's mapped to a column
                let col_info = mapping.iter().find_map(|(aes, value)| {
                    if aes.is_y_like() {
                        if value.as_column_name().is_some() {
                            return Some((*aes, value));
                        }
                    }
                    None
                });

                if let Some((y_aesthetic, aes_value)) = col_info {
                    // Get the appropriate data source
                    let data = if let Some(ref computed) = layer.computed_data {
                        Some(computed.as_ref())
                    } else {
                        match &layer.data {
                            Some(d) => Some(d.as_ref()),
                            None => default_data,
                        }
                    };

                    // Use validation function to determine scale type
                    match validate_and_determine_scale_type(
                        y_aesthetic,
                        aes_value,
                        layer.geom.as_ref(),
                        data,
                    ) {
                        Ok(ScaleType::Categorical) => {
                            // Create categorical scale - we'll train it later
                            use crate::scale::categorical::Catagorical;
                            use std::collections::HashMap;
                            self.y = Some(Box::new(Catagorical::new(HashMap::new())));
                        }
                        Ok(ScaleType::Continuous | ScaleType::Either) => {
                            // Create default linear scale
                            if let Ok(scale) = Continuous::new().linear() {
                                self.y = Some(Box::new(scale));
                            }
                        }
                        Err(_) => {
                            // Validation failed - skip creating a scale
                            // The error will be reported elsewhere
                        }
                    }

                    // Set default axis title if not already set
                    if y_axis_title.is_none() {
                        if let Some(col_name) = aes_value.as_column_name() {
                            *y_axis_title = Some(col_name.to_string());
                        }
                    }
                }
            }

            // Color scale
            if self.color.is_none() {
                if let Some(aes_value) = layer_mapping.get(&Aesthetic::Color) {
                    if aes_value.as_column_name().is_some() {
                        let data = match &layer.data {
                            Some(d) => Some(d.as_ref()),
                            None => default_data,
                        };
                        
                        // Use validation to determine if we need categorical or continuous
                        match validate_and_determine_scale_type(
                            Aesthetic::Color,
                            aes_value,
                            layer.geom.as_ref(),
                            data,
                        ) {
                            Ok(ScaleType::Categorical) => {
                                self.color = Some(Box::new(DiscreteColor::default_palette()));
                            }
                            Ok(ScaleType::Continuous | ScaleType::Either) => {
                                // Could create a continuous color scale here if needed
                                // For now, default to categorical for color
                                self.color = Some(Box::new(DiscreteColor::default_palette()));
                            }
                            Err(_) => {}
                        }
                    }
                }
            }

            // Fill scale
            if self.fill.is_none() {
                if let Some(aes_value) = layer_mapping.get(&Aesthetic::Fill) {
                    if aes_value.as_column_name().is_some() {
                        let data = match &layer.data {
                            Some(d) => Some(d.as_ref()),
                            None => default_data,
                        };
                        
                        // Use validation to determine if we need categorical or continuous
                        match validate_and_determine_scale_type(
                            Aesthetic::Fill,
                            aes_value,
                            layer.geom.as_ref(),
                            data,
                        ) {
                            Ok(ScaleType::Categorical) => {
                                self.fill = Some(Box::new(DiscreteColor::default_palette()));
                            }
                            Ok(ScaleType::Continuous | ScaleType::Either) => {
                                // Could create a continuous color scale here if needed
                                // For now, default to categorical for fill
                                self.fill = Some(Box::new(DiscreteColor::default_palette()));
                            }
                            Err(_) => {}
                        }
                    }
                }
            }

            // Shape scale
            if self.shape.is_none() {
                if matches!(layer_mapping.get(&Aesthetic::Shape), Some(AesValue::Column { name: _, hint: Some(ScaleType::Categorical) , ..})) {
                    // Create default discrete shape scale
                    self.shape = Some(Box::new(DiscreteShape::default_shapes()));
                }
            }

            // Size scale
            if self.size.is_none() {
                if matches!(layer_mapping.get(&Aesthetic::Size), Some(AesValue::Column { .. })) {
                    // Create default linear scale for size
                    if let Ok(scale) = Continuous::new().linear() {
                        self.size = Some(Box::new(scale));
                    }
                }
            }

            // Alpha scale
            if self.alpha.is_none() {
                if matches!(layer_mapping.get(&Aesthetic::Alpha), Some(AesValue::Column { .. })) {
                    // Create default linear scale for alpha
                    if let Ok(scale) = Continuous::new().linear() {
                        self.alpha = Some(Box::new(scale));
                    }
                }
            }
        }
    }

    /// Train all scales on the data from layers
    pub fn train(&mut self, layers: &[Layer], default_data: Option<&dyn DataSource>, default_mapping: &AesMap) {
        // Train scales on data (including computed stat data)
        for layer in layers {
            // Use computed data if available, otherwise use original data
            let data: &dyn DataSource = if let Some(ref computed) = layer.computed_data {
                computed.as_ref()
            } else {
                match &layer.data {
                    Some(d) => d.as_ref(),
                    None => match default_data {
                        Some(d) => d,
                        None => continue,
                    },
                }
            };

            // Use computed mapping if available, otherwise use original mapping
            let mapping = layer.get_mapping(default_mapping);

            // Collect all x-related vectors (X, XBegin, XEnd, Xmin, Xmax, etc.)
            let mut x_vecs = Vec::new();
            for (aes, aes_value) in mapping.iter() {
                if aes.is_x_like() {
                    if let Some(col_name) = aes_value.as_column_name() {
                        if let Some(vec) = data.get(col_name) {
                            x_vecs.push(vec);
                        }
                    }
                }
            }

            // Train x scale on all x-related data
            if !x_vecs.is_empty() {
                if let Some(ref mut scale) = self.x {
                    scale.train(&x_vecs);
                }
            }

            // Collect all y-related vectors (Y, YBegin, YEnd, Ymin, Ymax, etc.)
            let mut y_vecs = Vec::new();
            for (aes, aes_value) in mapping.iter() {
                if aes.is_y_like() {
                    if let Some(col_name) = aes_value.as_column_name() {
                        if let Some(vec) = data.get(col_name) {
                            y_vecs.push(vec);
                        }
                    }
                }
            }

            // Train y scale on all y-related data
            if !y_vecs.is_empty() {
                if let Some(ref mut scale) = self.y {
                    scale.train(&y_vecs);
                }
            }

            // Train color scale
            if let Some(col_name) = mapping.get(&Aesthetic::Color).and_then(|v| v.as_column_name()) {
                if let Some(ref mut scale) = self.color {
                    if let Some(vec) = data.get(col_name) {
                        scale.train(&[vec]);
                    }
                }
            }

            // Train fill scale
            if let Some(col_name) = mapping.get(&Aesthetic::Fill).and_then(|v| v.as_column_name()) {
                if let Some(ref mut scale) = self.fill {
                    if let Some(vec) = data.get(col_name) {
                        scale.train(&[vec]);
                    }
                }
            }

            // Train shape scale
            if let Some(col_name) = mapping.get(&Aesthetic::Shape).and_then(|v| v.as_column_name()) {
                if let Some(ref mut scale) = self.shape {
                    if let Some(vec) = data.get(col_name) {
                        scale.train(&[vec]);
                    }
                }
            }

            // Train size scale
            if let Some(col_name) = mapping.get(&Aesthetic::Size).and_then(|v| v.as_column_name()) {
                if let Some(ref mut scale) = self.size {
                    if let Some(vec) = data.get(col_name) {
                        scale.train(&[vec]);
                    }
                }
            }

            // Train alpha scale
            if let Some(col_name) = mapping.get(&Aesthetic::Alpha).and_then(|v| v.as_column_name()) {
                if let Some(ref mut scale) = self.alpha {
                    if let Some(vec) = data.get(col_name) {
                        scale.train(&[vec]);
                    }
                }
            }
        }
    }
}

impl Default for ScaleSet {
    fn default() -> Self {
        Self::new()
    }
}
