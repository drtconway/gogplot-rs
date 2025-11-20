// Scale management for plots

use crate::aesthetics::{Aesthetic, AesValue};
use crate::data::DataSource;
use crate::layer::Layer;
use crate::scale::{ColorScale, ContinuousScale, ShapeScale};

/// Container for scales (x, y, color, size, etc.)
pub struct ScaleSet {
    pub x: Option<Box<dyn ContinuousScale>>,
    pub y: Option<Box<dyn ContinuousScale>>,
    pub color: Option<Box<dyn ColorScale>>,
    pub size: Option<Box<dyn ContinuousScale>>,
    pub alpha: Option<Box<dyn ContinuousScale>>,
    pub shape: Option<Box<dyn ShapeScale>>,
    // Add more as needed
}

impl ScaleSet {
    pub fn new() -> Self {
        Self {
            x: None,
            y: None,
            color: None,
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
        x_axis_title: &mut Option<String>,
        y_axis_title: &mut Option<String>,
    ) {
        use crate::scale::color::DiscreteColor;
        use crate::scale::continuous::Continuous;
        use crate::scale::shape::DiscreteShape;

        // Find the first layer that maps each aesthetic to determine default scales
        for layer in layers {
            // X scale
            if self.x.is_none() {
                let col_name = layer
                    .mapping
                    .get(&Aesthetic::X)
                    .or_else(|| layer.mapping.get(&Aesthetic::XBegin))
                    .or_else(|| layer.mapping.get(&Aesthetic::XEnd));

                if let Some(AesValue::Column(col_name)) = col_name {
                    // Check if this column is categorical (string type)
                    let data = match &layer.data {
                        Some(d) => Some(d.as_ref()),
                        None => default_data,
                    };

                    let is_categorical = data
                        .and_then(|d| d.get(col_name))
                        .map(|col| col.as_str().is_some())
                        .unwrap_or(false);

                    if is_categorical {
                        // Create categorical scale - we'll train it later
                        use crate::scale::categorical::Catagorical;
                        use std::collections::HashMap;
                        self.x = Some(Box::new(Catagorical::new(HashMap::new())));
                    } else {
                        // Create default linear scale
                        if let Ok(scale) = Continuous::new().linear() {
                            self.x = Some(Box::new(scale));
                        }
                    }

                    // Set default axis title if not already set
                    if x_axis_title.is_none() {
                        *x_axis_title = Some(col_name.clone());
                    }
                }
            }

            // Y scale
            if self.y.is_none() {
                let col_name = layer
                    .mapping
                    .get(&Aesthetic::Y)
                    .or_else(|| layer.mapping.get(&Aesthetic::YBegin))
                    .or_else(|| layer.mapping.get(&Aesthetic::YEnd));

                if let Some(AesValue::Column(col_name)) = col_name {
                    // Check if this column is categorical (string type)
                    let data = match &layer.data {
                        Some(d) => Some(d.as_ref()),
                        None => default_data,
                    };

                    let is_categorical = data
                        .and_then(|d| d.get(col_name))
                        .map(|col| col.as_str().is_some())
                        .unwrap_or(false);

                    if is_categorical {
                        // Create categorical scale - we'll train it later
                        use crate::scale::categorical::Catagorical;
                        use std::collections::HashMap;
                        self.y = Some(Box::new(Catagorical::new(HashMap::new())));
                    } else {
                        // Create default linear scale
                        if let Ok(scale) = Continuous::new().linear() {
                            self.y = Some(Box::new(scale));
                        }
                    }

                    // Set default axis title if not already set
                    if y_axis_title.is_none() {
                        *y_axis_title = Some(col_name.clone());
                    }
                }
            }

            // Color scale
            if self.color.is_none() {
                if let Some(AesValue::Column(_)) = layer.mapping.get(&Aesthetic::Color) {
                    // Create default discrete color scale
                    self.color = Some(Box::new(DiscreteColor::default_palette()));
                }
            }

            // Shape scale
            if self.shape.is_none() {
                if let Some(AesValue::Column(_)) = layer.mapping.get(&Aesthetic::Shape) {
                    // Create default discrete shape scale
                    self.shape = Some(Box::new(DiscreteShape::default_shapes()));
                }
            }

            // Size scale
            if self.size.is_none() {
                if let Some(AesValue::Column(_)) = layer.mapping.get(&Aesthetic::Size) {
                    // Create default linear scale for size
                    if let Ok(scale) = Continuous::new().linear() {
                        self.size = Some(Box::new(scale));
                    }
                }
            }

            // Alpha scale
            if self.alpha.is_none() {
                if let Some(AesValue::Column(_)) = layer.mapping.get(&Aesthetic::Alpha) {
                    // Create default linear scale for alpha
                    if let Ok(scale) = Continuous::new().linear() {
                        self.alpha = Some(Box::new(scale));
                    }
                }
            }
        }
    }

    /// Train all scales on the data from layers
    pub fn train(&mut self, layers: &[Layer], default_data: Option<&dyn DataSource>) {
        // Train scales on data (including computed stat data)
        for layer in layers {
            // Use computed data if available, otherwise use original data
            let data: &dyn DataSource = if let Some(ref computed) = layer.computed_data {
                computed as &dyn DataSource
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
            let mapping = layer.computed_mapping.as_ref().unwrap_or(&layer.mapping);

            // Collect all x-related vectors (X, XBegin, XEnd)
            let mut x_vecs = Vec::new();
            if let Some(AesValue::Column(col_name)) = mapping.get(&Aesthetic::X) {
                if let Some(vec) = data.get(col_name) {
                    x_vecs.push(vec);
                }
            }
            if let Some(AesValue::Column(col_name)) = mapping.get(&Aesthetic::XBegin) {
                if let Some(vec) = data.get(col_name) {
                    x_vecs.push(vec);
                }
            }
            if let Some(AesValue::Column(col_name)) = mapping.get(&Aesthetic::XEnd) {
                if let Some(vec) = data.get(col_name) {
                    x_vecs.push(vec);
                }
            }

            // Train x scale on all x-related data
            if !x_vecs.is_empty() {
                if let Some(ref mut scale) = self.x {
                    scale.train(&x_vecs);
                }
            }

            // Collect all y-related vectors (Y, YBegin, YEnd)
            let mut y_vecs = Vec::new();
            if let Some(AesValue::Column(col_name)) = mapping.get(&Aesthetic::Y) {
                if let Some(vec) = data.get(col_name) {
                    y_vecs.push(vec);
                }
            }
            if let Some(AesValue::Column(col_name)) = mapping.get(&Aesthetic::YBegin) {
                if let Some(vec) = data.get(col_name) {
                    y_vecs.push(vec);
                }
            }
            if let Some(AesValue::Column(col_name)) = mapping.get(&Aesthetic::YEnd) {
                if let Some(vec) = data.get(col_name) {
                    y_vecs.push(vec);
                }
            }

            // Train y scale on all y-related data
            if !y_vecs.is_empty() {
                if let Some(ref mut scale) = self.y {
                    scale.train(&y_vecs);
                }
            }

            // Train color scale
            if let Some(AesValue::Column(col_name)) = mapping.get(&Aesthetic::Color) {
                if let Some(ref mut scale) = self.color {
                    if let Some(vec) = data.get(col_name) {
                        scale.train(&[vec]);
                    }
                }
            }

            // Train shape scale
            if let Some(AesValue::Column(col_name)) = mapping.get(&Aesthetic::Shape) {
                if let Some(ref mut scale) = self.shape {
                    if let Some(vec) = data.get(col_name) {
                        scale.train(&[vec]);
                    }
                }
            }

            // Train size scale
            if let Some(AesValue::Column(col_name)) = mapping.get(&Aesthetic::Size) {
                if let Some(ref mut scale) = self.size {
                    if let Some(vec) = data.get(col_name) {
                        scale.train(&[vec]);
                    }
                }
            }

            // Train alpha scale
            if let Some(AesValue::Column(col_name)) = mapping.get(&Aesthetic::Alpha) {
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
