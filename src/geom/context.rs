use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::data::{DataSource, VectorType};
use crate::error::{DataType, PlotError};
use crate::layer::Layer;
use crate::plot::ScaleSet;
use crate::scale::{ContinuousScale, ScaleType};
use crate::theme::{self, Color};
use cairo::Context;
use ordered_float::OrderedFloat;

/// Enum to hold aesthetic values that can be either borrowed or owned
pub enum AestheticValues<'a> {
    /// Borrowed float slice iterator
    FloatRef(std::slice::Iter<'a, f64>),
    /// Owned vector (needed for type conversions or scale applications)
    Owned(Vec<f64>),
    /// Constant value repeated n times
    Constant(f64, usize),
}

impl<'a> Iterator for AestheticValues<'a> {
    type Item = f64;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            AestheticValues::FloatRef(iter) => iter.next().copied(),
            AestheticValues::Owned(vec) => {
                if vec.is_empty() {
                    None
                } else {
                    Some(vec.remove(0))
                }
            }
            AestheticValues::Constant(value, n) => {
                if *n > 0 {
                    *n -= 1;
                    Some(*value)
                } else {
                    None
                }
            }
        }
    }
}

/// Enum to hold color values
pub enum ColorValues {
    /// Constant color repeated n times
    Constant(Color, usize),
    /// Mapped colors from data
    Mapped(Vec<Color>),
}

impl Iterator for ColorValues {
    type Item = Color;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ColorValues::Constant(color, n) => {
                if *n > 0 {
                    *n -= 1;
                    Some(*color)
                } else {
                    None
                }
            }
            ColorValues::Mapped(vec) => {
                if vec.is_empty() {
                    None
                } else {
                    Some(vec.remove(0))
                }
            }
        }
    }
}

/// Enum to hold shape values
pub enum ShapeValues {
    /// Constant shape repeated n times
    Constant(crate::visuals::Shape, usize),
    /// Mapped shapes from data
    Mapped(Vec<crate::visuals::Shape>),
}

impl Iterator for ShapeValues {
    type Item = crate::visuals::Shape;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ShapeValues::Constant(shape, n) => {
                if *n > 0 {
                    *n -= 1;
                    Some(*shape)
                } else {
                    None
                }
            }
            ShapeValues::Mapped(vec) => {
                if vec.is_empty() {
                    None
                } else {
                    Some(vec.remove(0))
                }
            }
        }
    }
}

/// Encapsulates all the context needed for rendering a geom
pub struct RenderContext<'a> {
    /// Cairo drawing context
    pub cairo: &'a mut Context,

    /// The layer being rendered (contains data, mapping, stat, position, etc.)
    pub layer: &'a Layer,

    /// Plot-level data (fallback if layer has no data)
    pub plot_data: Option<&'a dyn DataSource>,

    /// Plot-level aesthetic mapping (fallback if layer has no mapping)
    pub plot_mapping: &'a AesMap,

    /// Scales for transforming data to visual space
    pub scales: &'a ScaleSet,

    /// Theme for styling
    pub theme: &'a theme::Theme,

    /// X viewport range (min, max) in device coordinates
    pub x_range: (f64, f64),

    /// Y viewport range (min, max) in device coordinates
    pub y_range: (f64, f64),
}

impl<'a> RenderContext<'a> {
    pub fn new(
        cairo: &'a mut Context,
        layer: &'a Layer,
        plot_data: Option<&'a dyn DataSource>,
        plot_mapping: &'a AesMap,
        scales: &'a ScaleSet,
        theme: &'a theme::Theme,
        x_range: (f64, f64),
        y_range: (f64, f64),
    ) -> Self {
        Self {
            cairo,
            layer,
            plot_data,
            plot_mapping,
            scales,
            theme,
            x_range,
            y_range,
        }
    }

    /// Get the active data source (computed data if available, otherwise layer or plot-level data)
    pub fn data(&self) -> &dyn DataSource {
        self.layer
            .computed_data
            .as_ref()
            .map(|d| d.as_ref())
            .or_else(|| self.layer.data.as_ref().map(|d| d.as_ref()))
            .or(self.plot_data)
            .expect("Layer must have computed_data, layer data, or plot-level data")
    }

    /// Get the active aesthetic mapping (computed if available, otherwise original)
    pub fn mapping(&self) -> &AesMap {
        self.layer.get_mapping(self.plot_mapping)
    }

    /// Get the original layer data (useful for drawing outliers, raw points, etc.)
    /// Returns None if the layer has no original data
    pub fn original_data(&self) -> Option<&dyn DataSource> {
        self.layer.data.as_ref().map(|d| d.as_ref())
    }

    /// Map normalized [0, 1] x-coordinate to viewport coordinate
    pub fn map_x(&self, normalized: f64) -> f64 {
        let (x0, x1) = self.x_range;
        x0 + normalized * (x1 - x0)
    }

    /// Map normalized [0, 1] y-coordinate to viewport coordinate
    pub fn map_y(&self, normalized: f64) -> f64 {
        let (y0, y1) = self.y_range;
        y0 + normalized * (y1 - y0)
    }

    /// Get float values for an x-like aesthetic (X, Xmin, Xmax, Xintercept)
    /// In the new pipeline, data is pre-normalized to [0,1], so NO scaling is applied here.
    /// The values returned are already in normalized [0,1] coordinates.
    pub fn get_x_aesthetic_values(
        &self,
        aesthetic: Aesthetic,
    ) -> Result<AestheticValues<'a>, PlotError> {
        // Data is pre-normalized, don't apply scale again
        self.get_aesthetic_values(aesthetic, None)
    }

    /// Get float values for a y-like aesthetic (Y, Ymin, Ymax, Yintercept)
    /// In the new pipeline, data is pre-normalized to [0,1], so NO scaling is applied here.
    /// The values returned are already in normalized [0,1] coordinates.
    pub fn get_y_aesthetic_values(
        &self,
        aesthetic: Aesthetic,
    ) -> Result<AestheticValues<'a>, PlotError> {
        // Data is pre-normalized, don't apply scale again
        self.get_aesthetic_values(aesthetic, None)
    }

    /// Get float values for an aesthetic without any scale transformation
    pub fn get_unscaled_aesthetic_values(
        &self,
        aesthetic: Aesthetic,
    ) -> Result<AestheticValues<'a>, PlotError> {
        self.get_aesthetic_values(aesthetic, None)
    }

    /// Get float values for an aesthetic as an iterator
    /// Constants are replicated to match data length, columns read from data
    pub fn get_aesthetic_values(
        &self,
        aesthetic: Aesthetic,
        scale: Option<&dyn ContinuousScale>,
    ) -> Result<AestheticValues<'a>, PlotError> {
        use crate::aesthetics::AesValue;
        use crate::data::PrimitiveValue;

        let mapping = self.mapping().get(&aesthetic);
        let n = self.data().len();

        match mapping {
            Some(AesValue::Column{ name: col_name, hint: None | Some(ScaleType::Continuous | ScaleType::Either) }) => {
                // Get data from column
                let vec = self
                    .data()
                    .get(col_name.as_str())
                    .ok_or_else(|| PlotError::missing_column(col_name.as_str()))?;

                // Convert to f64 and optionally apply scale
                match vec.vtype() {
                    VectorType::Float => {
                        let floats =
                            vec.iter_float()
                                .ok_or_else(|| PlotError::InvalidAestheticType {
                                    aesthetic,
                                    expected: DataType::Vector(VectorType::Float),
                                    actual: DataType::Custom("unknown".to_string()),
                                })?;

                        if let Some(scale) = scale {
                            // Check if scale is categorical - if so, treat floats as categories
                            use crate::scale::ScaleType;
                            if scale.scale_type() == ScaleType::Categorical {
                                // Convert floats to strings and use categorical mapping
                                let strings: Vec<String> = floats.map(|f| f.to_string()).collect();
                                let mapped: Vec<f64> = strings
                                    .iter()
                                    .filter_map(|s| scale.map_category(s.as_str(), aesthetic))
                                    .collect();
                                Ok(AestheticValues::Owned(mapped))
                            } else {
                                // Continuous scale: apply map_value
                                let values: Vec<f64> =
                                    floats.filter_map(|v| scale.map_value(v)).collect();
                                Ok(AestheticValues::Owned(values))
                            }
                        } else {
                            // Collect to owned since FloatRef expects std::slice::Iter
                            let values: Vec<f64> = floats.collect();
                            Ok(AestheticValues::Owned(values))
                        }
                    }
                    VectorType::Int => {
                        let ints =
                            vec.iter_int()
                                .ok_or_else(|| PlotError::InvalidAestheticType {
                                    aesthetic,
                                    expected: DataType::Vector(VectorType::Int),
                                    actual: DataType::Custom("unknown".to_string()),
                                })?;

                        if let Some(scale) = scale {
                            // Check if scale is categorical - if so, treat integers as categories
                            use crate::scale::ScaleType;
                            if scale.scale_type() == ScaleType::Categorical {
                                // Convert integers to strings and use categorical mapping
                                let strings: Vec<String> = ints.map(|i| i.to_string()).collect();
                                let mapped: Vec<f64> = strings
                                    .iter()
                                    .filter_map(|s| scale.map_category(s.as_str(), aesthetic))
                                    .collect();
                                Ok(AestheticValues::Owned(mapped))
                            } else {
                                // Continuous scale: convert to f64 and map
                                let values: Vec<f64> = ints.map(|x| x as f64).collect();
                                let scaled: Vec<f64> = values
                                    .into_iter()
                                    .filter_map(|v| scale.map_value(v))
                                    .collect();
                                Ok(AestheticValues::Owned(scaled))
                            }
                        } else {
                            // No scale: just convert to f64
                            let values: Vec<f64> = ints.map(|x| x as f64).collect();
                            Ok(AestheticValues::Owned(values))
                        }
                    }
                    VectorType::Str => {
                        // Try to use scale's categorical mapping
                        if let Some(scale) = scale {
                            let strs =
                                vec.iter_str()
                                    .ok_or_else(|| PlotError::InvalidAestheticType {
                                        aesthetic,
                                        expected: DataType::Vector(VectorType::Str),
                                        actual: DataType::Custom("unknown".to_string()),
                                    })?;

                            let strs_vec: Vec<&str> = strs.collect();
                            let mapped: Vec<f64> = strs_vec
                                .iter()
                                .filter_map(|s| scale.map_category(s, aesthetic))
                                .collect();

                            if mapped.len() == strs_vec.len() {
                                Ok(AestheticValues::Owned(mapped))
                            } else {
                                Err(PlotError::InvalidAestheticType {
                                    aesthetic,
                                    expected: DataType::Custom(
                                        "all values mapped by scale".to_string(),
                                    ),
                                    actual: DataType::Custom("some values not mapped".to_string()),
                                })
                            }
                        } else {
                            Err(PlotError::InvalidAestheticType {
                                aesthetic,
                                expected: DataType::CategoricalScale,
                                actual: DataType::Custom("no scale provided".to_string()),
                            })
                        }
                    }
                    VectorType::Bool => {
                        return Err(PlotError::InvalidAestheticType {
                            aesthetic,
                            expected: DataType::Vector(VectorType::Float),
                            actual: DataType::Vector(VectorType::Bool),
                        });
                    }
                }
            }
            Some(AesValue::Column{ name: col_name, hint: Some(ScaleType::Categorical) }) => {
                // Treat numeric column as categorical by converting to strings
                let vec = self
                    .data()
                    .get(col_name.as_str())
                    .ok_or_else(|| PlotError::missing_column(col_name.as_str()))?;

                // Require a scale for categorical mapping
                let scale = scale.ok_or_else(|| PlotError::InvalidAestheticType {
                    aesthetic,
                    expected: DataType::CategoricalScale,
                    actual: DataType::Custom(
                        "no scale provided for categorical column".to_string(),
                    ),
                })?;

                match vec.vtype() {
                    VectorType::Float => {
                        let floats =
                            vec.iter_float()
                                .ok_or_else(|| PlotError::InvalidAestheticType {
                                    aesthetic,
                                    expected: DataType::Vector(VectorType::Float),
                                    actual: DataType::Custom("unknown".to_string()),
                                })?;

                        // Convert floats to strings and use categorical mapping
                        let strings: Vec<String> = floats.map(|f| f.to_string()).collect();
                        let mapped: Vec<f64> = strings
                            .iter()
                            .filter_map(|s| scale.map_category(s.as_str(), aesthetic))
                            .collect();

                        if mapped.len() == strings.len() {
                            Ok(AestheticValues::Owned(mapped))
                        } else {
                            Err(PlotError::InvalidAestheticType {
                                aesthetic,
                                expected: DataType::Custom(
                                    "all values mapped by scale".to_string(),
                                ),
                                actual: DataType::Custom("some values not mapped".to_string()),
                            })
                        }
                    }
                    VectorType::Int => {
                        let ints =
                            vec.iter_int()
                                .ok_or_else(|| PlotError::InvalidAestheticType {
                                    aesthetic,
                                    expected: DataType::Vector(VectorType::Int),
                                    actual: DataType::Custom("unknown".to_string()),
                                })?;

                        // Convert ints to strings and use categorical mapping
                        let strings: Vec<String> = ints.map(|i| i.to_string()).collect();
                        let mapped: Vec<f64> = strings
                            .iter()
                            .filter_map(|s| scale.map_category(s.as_str(), aesthetic))
                            .collect();

                        if mapped.len() == strings.len() {
                            Ok(AestheticValues::Owned(mapped))
                        } else {
                            Err(PlotError::InvalidAestheticType {
                                aesthetic,
                                expected: DataType::Custom(
                                    "all values mapped by scale".to_string(),
                                ),
                                actual: DataType::Custom("some values not mapped".to_string()),
                            })
                        }
                    }
                    VectorType::Str => {
                        // Already strings, use categorical mapping
                        let strs =
                            vec.iter_str()
                                .ok_or_else(|| PlotError::InvalidAestheticType {
                                    aesthetic,
                                    expected: DataType::Vector(VectorType::Str),
                                    actual: DataType::Custom("unknown".to_string()),
                                })?;

                        let strs_vec: Vec<&str> = strs.collect();
                        let mapped: Vec<f64> = strs_vec
                            .iter()
                            .filter_map(|s| scale.map_category(s, aesthetic))
                            .collect();

                        if mapped.len() == strs_vec.len() {
                            Ok(AestheticValues::Owned(mapped))
                        } else {
                            Err(PlotError::InvalidAestheticType {
                                aesthetic,
                                expected: DataType::Custom(
                                    "all values mapped by scale".to_string(),
                                ),
                                actual: DataType::Custom("some values not mapped".to_string()),
                            })
                        }
                    }
                    VectorType::Bool => {
                        let bools =
                            vec.iter_bool()
                                .ok_or_else(|| PlotError::InvalidAestheticType {
                                    aesthetic,
                                    expected: DataType::Vector(VectorType::Bool),
                                    actual: DataType::Custom("unknown".to_string()),
                                })?;

                        // Convert bools to strings and use categorical mapping
                        let strings: Vec<String> = bools.map(|b| b.to_string()).collect();
                        let mapped: Vec<f64> = strings
                            .iter()
                            .filter_map(|s| scale.map_category(s.as_str(), aesthetic))
                            .collect();

                        if mapped.len() == strings.len() {
                            Ok(AestheticValues::Owned(mapped))
                        } else {
                            Err(PlotError::InvalidAestheticType {
                                aesthetic,
                                expected: DataType::Custom(
                                    "all values mapped by scale".to_string(),
                                ),
                                actual: DataType::Custom("some values not mapped".to_string()),
                            })
                        }
                    }
                }
            }
            Some(AesValue::Constant{ value: prim, hint: _ }) => {
                // Replicate constant value to match data length
                let value = match prim {
                    PrimitiveValue::Float(v) => *v,
                    PrimitiveValue::Int(v) => *v as f64,
                    PrimitiveValue::Str(_) => {
                        return Err(PlotError::InvalidAestheticType {
                            aesthetic,
                            expected: DataType::Custom("numeric constant".to_string()),
                            actual: DataType::Constant(VectorType::Str),
                        });
                    }
                    PrimitiveValue::Bool(_) => {
                        return Err(PlotError::InvalidAestheticType {
                            aesthetic,
                            expected: DataType::Custom("numeric constant".to_string()),
                            actual: DataType::Constant(VectorType::Bool),
                        });
                    }
                };
                Ok(AestheticValues::Constant(value, n))
            }
            None => {
                // Use theme default value replicated to match data length
                let default_value = match aesthetic {
                    Aesthetic::Size => self.theme.geom_point.size,
                    Aesthetic::Alpha => self.theme.geom_point.alpha,
                    _ => return Err(PlotError::MissingAesthetic { aesthetic }),
                };
                Ok(AestheticValues::Constant(default_value, n))
            }
        }
    }

    /// Get color values for the Color aesthetic as an iterator
    /// Handles both constants and scale-mapped colors
    pub fn get_color_values(&self) -> Result<ColorValues, PlotError> {
        use crate::aesthetics::AesValue;
        use crate::data::PrimitiveValue;

        let n = self.data().len();
        let mapping = self.mapping().get(&Aesthetic::Color);
        let color_scale = self
            .layer
            .computed_scales
            .as_ref()
            .and_then(|s| s.color.as_ref())
            .or_else(|| self.scales.color.as_ref());

        match (mapping, color_scale) {
            // Column mapped with continuous scale
            (
                Some(AesValue::Column {
                    name: col_name,
                    hint: None | Some(ScaleType::Continuous | ScaleType::Either),
                }),
                Some(scale),
            ) => {
                let vec = self
                    .data()
                    .get(col_name.as_str())
                    .ok_or_else(|| PlotError::missing_column(col_name.as_str()))?;

                match vec.vtype() {
                    VectorType::Float | VectorType::Int => {
                        // Continuous color scale
                        let values: Vec<f64> = match vec.vtype() {
                            VectorType::Float => vec
                                .iter_float()
                                .ok_or_else(|| PlotError::InvalidAestheticType {
                                    aesthetic: Aesthetic::Color,
                                    expected: DataType::Vector(VectorType::Float),
                                    actual: DataType::Custom("unknown".to_string()),
                                })?
                                .collect(),
                            VectorType::Int => vec
                                .iter_int()
                                .ok_or_else(|| PlotError::InvalidAestheticType {
                                    aesthetic: Aesthetic::Color,
                                    expected: DataType::Vector(VectorType::Int),
                                    actual: DataType::Custom("unknown".to_string()),
                                })?
                                .map(|x| x as f64)
                                .collect(),
                            _ => unreachable!(),
                        };

                        let colors: Vec<Color> = values
                            .iter()
                            .filter_map(|&v| scale.map_continuous_to_color(v))
                            .collect();
                        Ok(ColorValues::Mapped(colors))
                    }
                    VectorType::Str => {
                        // Discrete color scale
                        let strings =
                            vec.iter_str()
                                .ok_or_else(|| PlotError::InvalidAestheticType {
                                    aesthetic: Aesthetic::Color,
                                    expected: DataType::Vector(VectorType::Str),
                                    actual: DataType::Custom("unknown".to_string()),
                                })?;

                        let colors: Vec<Color> = strings
                            .filter_map(|s| scale.map_discrete_to_color(s))
                            .collect();
                        Ok(ColorValues::Mapped(colors))
                    }
                    VectorType::Bool => {
                        return Err(PlotError::InvalidAestheticType {
                            aesthetic: Aesthetic::Color,
                            expected: DataType::Vector(VectorType::Float),
                            actual: DataType::Vector(VectorType::Bool),
                        });
                    }
                }
            }
            // CategoricalColumn - treat numeric data as discrete categories
            (
                Some(AesValue::Column {
                    name: col_name,
                    hint: Some(ScaleType::Categorical),
                }),
                Some(scale),
            ) => {
                let vec = self
                    .data()
                    .get(col_name.as_str())
                    .ok_or_else(|| PlotError::missing_column(col_name.as_str()))?;

                // Convert any data type to strings for discrete color mapping
                let strings: Vec<String> = match vec.vtype() {
                    VectorType::Float => vec
                        .iter_float()
                        .ok_or_else(|| PlotError::InvalidAestheticType {
                            aesthetic: Aesthetic::Color,
                            expected: DataType::Vector(VectorType::Float),
                            actual: DataType::Custom("unknown".to_string()),
                        })?
                        .map(|f| f.to_string())
                        .collect(),
                    VectorType::Int => vec
                        .iter_int()
                        .ok_or_else(|| PlotError::InvalidAestheticType {
                            aesthetic: Aesthetic::Color,
                            expected: DataType::Vector(VectorType::Int),
                            actual: DataType::Custom("unknown".to_string()),
                        })?
                        .map(|i| i.to_string())
                        .collect(),
                    VectorType::Str => vec
                        .iter_str()
                        .ok_or_else(|| PlotError::InvalidAestheticType {
                            aesthetic: Aesthetic::Color,
                            expected: DataType::Vector(VectorType::Str),
                            actual: DataType::Custom("unknown".to_string()),
                        })?
                        .map(|s| s.to_string())
                        .collect(),
                    VectorType::Bool => vec
                        .iter_bool()
                        .ok_or_else(|| PlotError::InvalidAestheticType {
                            aesthetic: Aesthetic::Color,
                            expected: DataType::Vector(VectorType::Bool),
                            actual: DataType::Custom("unknown".to_string()),
                        })?
                        .map(|b| b.to_string())
                        .collect(),
                };

                let colors: Vec<Color> = strings
                    .iter()
                    .filter_map(|s| scale.map_discrete_to_color(s.as_str()))
                    .collect();
                Ok(ColorValues::Mapped(colors))
            }
            // Constant color (with categorical hint or no hint)
            (
                Some(AesValue::Constant {
                    value: PrimitiveValue::Int(rgba),
                    hint: None | Some(ScaleType::Categorical),
                }),
                _,
            ) => Ok(ColorValues::Constant(Color::from(*rgba), n)),
            // Other constant types are not valid for colors
            (Some(AesValue::Constant { value: _, hint: _ }), _) => {
                Err(PlotError::InvalidAestheticType {
                    aesthetic: Aesthetic::Color,
                    expected: DataType::RgbaConstant,
                    actual: DataType::Custom("non-integer constant".to_string()),
                })
            }
            // Column mapped but no scale
            (Some(AesValue::Column { name: _, hint: _ }), None) => {
                Err(PlotError::InvalidAestheticType {
                    aesthetic: Aesthetic::Color,
                    expected: DataType::Custom("color scale for column mapping".to_string()),
                    actual: DataType::Custom("no scale provided".to_string()),
                })
            }
            // No mapping, use theme default
            (None, _) => Ok(ColorValues::Constant(self.theme.geom_point.color, n)),
        }
    }

    /// Get fill color values for the Fill aesthetic as an iterator
    /// Handles both constants and scale-mapped colors
    pub fn get_fill_color_values(&self) -> Result<ColorValues, PlotError> {
        use crate::aesthetics::AesValue;
        use crate::data::PrimitiveValue;

        let n = self.data().len();
        let mapping = self.mapping().get(&Aesthetic::Fill);
        let color_scale = self
            .layer
            .computed_scales
            .as_ref()
            .and_then(|s| s.fill.as_ref())
            .or_else(|| self.scales.fill.as_ref());

        match (mapping, color_scale) {
            // Column mapped with scale (continuous)
            (
                Some(AesValue::Column {
                    name: col_name,
                    hint: None | Some(ScaleType::Continuous | ScaleType::Either),
                }),
                Some(scale),
            ) => {
                let vec = self
                    .data()
                    .get(col_name.as_str())
                    .ok_or_else(|| PlotError::missing_column(col_name.as_str()))?;

                match vec.vtype() {
                    VectorType::Float | VectorType::Int => {
                        // Continuous color scale
                        let values: Vec<f64> = match vec.vtype() {
                            VectorType::Float => vec
                                .iter_float()
                                .ok_or_else(|| PlotError::InvalidAestheticType {
                                    aesthetic: Aesthetic::Fill,
                                    expected: DataType::Vector(VectorType::Float),
                                    actual: DataType::Custom("unknown".to_string()),
                                })?
                                .collect(),
                            VectorType::Int => vec
                                .iter_int()
                                .ok_or_else(|| PlotError::InvalidAestheticType {
                                    aesthetic: Aesthetic::Fill,
                                    expected: DataType::Vector(VectorType::Int),
                                    actual: DataType::Custom("unknown".to_string()),
                                })?
                                .map(|x| x as f64)
                                .collect(),
                            _ => unreachable!(),
                        };

                        let colors: Vec<Color> = values
                            .iter()
                            .filter_map(|&v| {
                                let result = scale.map_continuous_to_color(v);
                                if result.is_none() {}
                                result
                            })
                            .collect();
                        Ok(ColorValues::Mapped(colors))
                    }
                    VectorType::Str => {
                        // Discrete color scale
                        let strings =
                            vec.iter_str()
                                .ok_or_else(|| PlotError::InvalidAestheticType {
                                    aesthetic: Aesthetic::Fill,
                                    expected: DataType::Vector(VectorType::Str),
                                    actual: DataType::Custom("unknown".to_string()),
                                })?;

                        let colors: Vec<Color> = strings
                            .filter_map(|s| scale.map_discrete_to_color(s))
                            .collect();
                        Ok(ColorValues::Mapped(colors))
                    }
                    VectorType::Bool => {
                        return Err(PlotError::InvalidAestheticType {
                            aesthetic: Aesthetic::Fill,
                            expected: DataType::Vector(VectorType::Float),
                            actual: DataType::Vector(VectorType::Bool),
                        });
                    }
                }
            }
            // CategoricalColumn - treat numeric data as discrete categories
            (
                Some(AesValue::Column {
                    name: col_name,
                    hint: Some(ScaleType::Categorical),
                }),
                Some(scale),
            ) => {
                let vec = self
                    .data()
                    .get(col_name.as_str())
                    .ok_or_else(|| PlotError::missing_column(col_name.as_str()))?;

                // Convert any data type to strings for discrete color mapping
                let strings: Vec<String> = match vec.vtype() {
                    VectorType::Float => vec
                        .iter_float()
                        .ok_or_else(|| PlotError::InvalidAestheticType {
                            aesthetic: Aesthetic::Fill,
                            expected: DataType::Vector(VectorType::Float),
                            actual: DataType::Custom("unknown".to_string()),
                        })?
                        .map(|f| f.to_string())
                        .collect(),
                    VectorType::Int => vec
                        .iter_int()
                        .ok_or_else(|| PlotError::InvalidAestheticType {
                            aesthetic: Aesthetic::Fill,
                            expected: DataType::Vector(VectorType::Int),
                            actual: DataType::Custom("unknown".to_string()),
                        })?
                        .map(|i| i.to_string())
                        .collect(),
                    VectorType::Str => vec
                        .iter_str()
                        .ok_or_else(|| PlotError::InvalidAestheticType {
                            aesthetic: Aesthetic::Fill,
                            expected: DataType::Vector(VectorType::Str),
                            actual: DataType::Custom("unknown".to_string()),
                        })?
                        .map(|s| s.to_string())
                        .collect(),
                    VectorType::Bool => vec
                        .iter_bool()
                        .ok_or_else(|| PlotError::InvalidAestheticType {
                            aesthetic: Aesthetic::Fill,
                            expected: DataType::Vector(VectorType::Bool),
                            actual: DataType::Custom("unknown".to_string()),
                        })?
                        .map(|b| b.to_string())
                        .collect(),
                };

                let colors: Vec<Color> = strings
                    .iter()
                    .filter_map(|s| scale.map_discrete_to_color(s.as_str()))
                    .collect();
                Ok(ColorValues::Mapped(colors))
            }
            // Constant color (with categorical hint or no hint)
            (
                Some(AesValue::Constant {
                    value: PrimitiveValue::Int(rgba),
                    hint: None | Some(ScaleType::Categorical),
                }),
                _,
            ) => Ok(ColorValues::Constant(Color::from(*rgba), n)),
            // Other constant types are not valid for fill
            (Some(AesValue::Constant { value: _, hint: _ }), _) => {
                Err(PlotError::InvalidAestheticType {
                    aesthetic: Aesthetic::Fill,
                    expected: DataType::RgbaConstant,
                    actual: DataType::Custom("non-integer constant".to_string()),
                })
            }
            // Column mapped but no scale
            (Some(AesValue::Column { name: _, hint: _ }), None) => {
                Err(PlotError::InvalidAestheticType {
                    aesthetic: Aesthetic::Fill,
                    expected: DataType::Custom("fill scale for column mapping".to_string()),
                    actual: DataType::Custom("no scale provided".to_string()),
                })
            }
            // No mapping, use theme default
            (None, _) => Ok(ColorValues::Constant(self.theme.geom_rect.fill, n)),
        }
    }

    /// Get shape values for the Shape aesthetic as an iterator
    /// Handles both constants and scale-mapped shapes
    pub fn get_shape_values(&self) -> Result<ShapeValues, PlotError> {
        use crate::aesthetics::AesValue;
        use crate::data::PrimitiveValue;
        use crate::visuals::Shape;

        let n = self.data().len();
        let mapping = self.mapping().get(&Aesthetic::Shape);
        let shape_scale = self
            .layer
            .computed_scales
            .as_ref()
            .and_then(|s| s.shape.as_ref())
            .or_else(|| self.scales.shape.as_ref());

        let int_to_shape = |v: i64| match v {
            0 => Shape::Circle,
            1 => Shape::Square,
            2 => Shape::Triangle,
            3 => Shape::Diamond,
            4 => Shape::Cross,
            5 => Shape::Plus,
            _ => Shape::Circle,
        };

        match (mapping, shape_scale) {
            // Column mapped with scale (categorical or unspecified)
            (
                Some(AesValue::Column {
                    name: col_name,
                    hint: None | Some(ScaleType::Categorical | ScaleType::Either),
                }),
                Some(scale),
            ) => {
                let vec = self
                    .data()
                    .get(col_name.as_str())
                    .ok_or_else(|| PlotError::missing_column(col_name.as_str()))?;

                if let Some(strings) = vec.iter_str() {
                    let shapes: Vec<Shape> =
                        strings.filter_map(|s| scale.map_to_shape(s)).collect();
                    Ok(ShapeValues::Mapped(shapes))
                } else {
                    Err(PlotError::InvalidAestheticType {
                        aesthetic: Aesthetic::Shape,
                        expected: DataType::Custom("string (categorical)".to_string()),
                        actual: DataType::Numeric,
                    })
                }
            }
            // Continuous hint for shapes is invalid
            (
                Some(AesValue::Column {
                    hint: Some(ScaleType::Continuous),
                    ..
                }),
                Some(_),
            ) => Err(PlotError::InvalidAestheticType {
                aesthetic: Aesthetic::Shape,
                expected: DataType::Custom("categorical for shapes".to_string()),
                actual: DataType::Custom("continuous hint provided".to_string()),
            }),
            // Constant shape
            (Some(AesValue::Constant{ value: PrimitiveValue::Int(v), hint: _ }), _) => {
                Ok(ShapeValues::Constant(int_to_shape(*v), n))
            }
            (Some(AesValue::Constant{ value: _, hint: _ }), _) => Err(PlotError::InvalidAestheticType {
                aesthetic: Aesthetic::Shape,
                expected: DataType::Constant(VectorType::Int),
                actual: DataType::Custom("other constant".to_string()),
            }),
            // Column mapped but no scale
            (Some(AesValue::Column { name: _, hint: _ }), None) => {
                Err(PlotError::InvalidAestheticType {
                    aesthetic: Aesthetic::Shape,
                    expected: DataType::Custom("shape scale for column mapping".to_string()),
                    actual: DataType::Custom("no scale provided".to_string()),
                })
            }
            // No mapping, use theme default
            (None, _) => {
                let shape = int_to_shape(self.theme.geom_point.shape);
                Ok(ShapeValues::Constant(shape, n))
            }
        }
    }

    /// Set the source color from a Color struct
    pub fn set_color(&mut self, color: &Color) {
        let Color(r, g, b, a) = color;
        self.cairo.set_source_rgba(
            *r as f64 / 255.0,
            *g as f64 / 255.0,
            *b as f64 / 255.0,
            *a as f64 / 255.0,
        );
    }

    /// Set the source color with an additional alpha multiplier
    pub fn set_color_alpha(&mut self, color: &Color, alpha: f64) {
        let Color(r, g, b, a) = color;
        self.cairo.set_source_rgba(
            *r as f64 / 255.0,
            *g as f64 / 255.0,
            *b as f64 / 255.0,
            (*a as f64 / 255.0) * alpha.clamp(0.0, 1.0),
        );
    }

    /// Set the source color from RGB values (0-255)
    pub fn set_rgb(&mut self, r: u8, g: u8, b: u8) {
        self.cairo
            .set_source_rgb(r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0);
    }

    /// Set the source color from RGBA values (0-255)
    pub fn set_rgba(&mut self, r: u8, g: u8, b: u8, a: u8) {
        self.cairo.set_source_rgba(
            r as f64 / 255.0,
            g as f64 / 255.0,
            b as f64 / 255.0,
            a as f64 / 255.0,
        );
    }

    /// Get scaled float values for an aesthetic
    /// Takes a column from data, optionally applies a scale, and returns an iterator of f64 values
    pub fn get_scaled_values(
        &self,
        aesthetic: Aesthetic,
        scale: Option<&dyn ContinuousScale>,
    ) -> Result<Vec<f64>, PlotError> {
        // Get the aesthetic mapping
        let mapping = self
            .mapping()
            .get(&aesthetic)
            .ok_or(PlotError::MissingAesthetic { aesthetic })?;

        // Extract column name
        let col_name = match mapping {
            AesValue::Column { name, hint: _ } => name.as_str(),
            AesValue::Constant{ value: _, hint: _ } => {
                return Err(PlotError::InvalidAestheticType {
                    aesthetic,
                    expected: DataType::ColumnMapping,
                    actual: DataType::Custom("constant".to_string()),
                });
            }
        };

        // Get the data vector
        let vec = self
            .data()
            .get(col_name)
            .ok_or_else(|| PlotError::missing_column(col_name))?;

        // Convert to f64 based on type
        let values: Vec<f64> = match vec.vtype() {
            VectorType::Float => {
                let floats = vec
                    .iter_float()
                    .ok_or_else(|| PlotError::InvalidAestheticType {
                        aesthetic,
                        expected: DataType::Vector(VectorType::Float),
                        actual: DataType::Custom("unknown".to_string()),
                    })?;
                floats.collect()
            }
            VectorType::Int => {
                let ints = vec
                    .iter_int()
                    .ok_or_else(|| PlotError::InvalidAestheticType {
                        aesthetic,
                        expected: DataType::Vector(VectorType::Int),
                        actual: DataType::Custom("unknown".to_string()),
                    })?;
                ints.map(|i| i as f64).collect()
            }
            VectorType::Str => {
                return Err(PlotError::InvalidAestheticType {
                    aesthetic,
                    expected: DataType::Vector(VectorType::Float),
                    actual: DataType::Vector(VectorType::Str),
                });
            }
            VectorType::Bool => {
                return Err(PlotError::InvalidAestheticType {
                    aesthetic,
                    expected: DataType::Vector(VectorType::Float),
                    actual: DataType::Vector(VectorType::Bool),
                });
            }
        };

        // Apply scale if present
        if let Some(scale) = scale {
            Ok(values.iter().filter_map(|&v| scale.map_value(v)).collect())
        } else {
            Ok(values)
        }
    }

    /// Get label values as strings from the Label aesthetic
    /// Handles string, int, and float columns, converting them to strings
    pub fn get_label_values(&self) -> Result<Vec<String>, PlotError> {
        use crate::data::PrimitiveValue;

        let label_mapping =
            self.mapping()
                .get(&Aesthetic::Label)
                .ok_or_else(|| PlotError::MissingAesthetic {
                    aesthetic: Aesthetic::Label,
                })?;

        match label_mapping {
            AesValue::Column{ name: col_name, hint: _ } => {
                let col =
                    self.data()
                        .get(col_name.as_str())
                        .ok_or_else(|| PlotError::MissingColumn {
                            column: col_name.clone(),
                        })?;

                // Convert column to strings using iterators
                if let Some(str_iter) = col.iter_str() {
                    Ok(str_iter.map(|s| s.to_string()).collect())
                } else if let Some(int_iter) = col.iter_int() {
                    Ok(int_iter.map(|v| v.to_string()).collect())
                } else if let Some(float_iter) = col.iter_float() {
                    Ok(float_iter.map(|v| v.to_string()).collect())
                } else {
                    Err(PlotError::invalid_column_type(
                        col_name,
                        "string, int, or float",
                    ))
                }
            }
            AesValue::Constant{ value: prim, hint: _ } => {
                let n_rows = self.data().len();
                let label_str = match prim {
                    PrimitiveValue::Str(s) => s.clone(),
                    PrimitiveValue::Int(i) => i.to_string(),
                    PrimitiveValue::Float(f) => f.to_string(),
                    PrimitiveValue::Bool(b) => b.to_string(),
                };
                Ok(vec![label_str; n_rows])
            }
        }
    }
}

pub(crate) fn compute_min_spacing(aesthetic_values: AestheticValues<'_>, width: f64) -> f64 {
    let mut x_set: Vec<OrderedFloat<f64>> = aesthetic_values
        .filter(|x| x.is_finite())
        .map(|x| OrderedFloat(x))
        .collect();
    x_set.sort();
    x_set.dedup();
    let x_set: Vec<f64> = x_set.into_iter().map(|of| of.0).collect();

    if x_set.len() > 1 {
        // Find minimum spacing between consecutive x values
        let mut min_spacing = f64::MAX;
        for i in 1..x_set.len() {
            let spacing = x_set[i] - x_set[i - 1];
            if spacing < min_spacing {
                min_spacing = spacing;
            }
        }
        min_spacing * width / 2.0
    } else {
        0.05 // Single bar fallback half-width
    }
}
