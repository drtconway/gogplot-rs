use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::data::{DataSource, VectorType};
use crate::error::{DataType, PlotError};
use crate::plot::ScaleSet;
use crate::scale::ContinuousScale;
use crate::theme::Color;
use cairo::Context;

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

    /// Data source for the layer
    pub data: &'a dyn DataSource,

    /// Aesthetic mappings
    pub mapping: &'a AesMap,

    /// Scales for transforming data to visual space
    pub scales: &'a ScaleSet,

    /// X viewport range (min, max) in device coordinates
    pub x_range: (f64, f64),

    /// Y viewport range (min, max) in device coordinates
    pub y_range: (f64, f64),
}

impl<'a> RenderContext<'a> {
    pub fn new(
        cairo: &'a mut Context,
        data: &'a dyn DataSource,
        mapping: &'a AesMap,
        scales: &'a ScaleSet,
        x_range: (f64, f64),
        y_range: (f64, f64),
    ) -> Self {
        Self {
            cairo,
            data,
            mapping,
            scales,
            x_range,
            y_range,
        }
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

    /// Get float values for an aesthetic as an iterator
    /// Constants are replicated to match data length, columns read from data
    pub fn get_aesthetic_values(
        &self,
        aesthetic: Aesthetic,
        scale: Option<&dyn ContinuousScale>,
    ) -> Result<AestheticValues<'a>, PlotError> {
        use crate::aesthetics::AesValue;
        use crate::data::PrimitiveValue;

        let mapping = self.mapping.get(&aesthetic);
        let n = self.data.len();

        match mapping {
            Some(AesValue::Column(col_name)) => {
                // Get data from column
                let vec = self
                    .data
                    .get(col_name.as_str())
                    .ok_or_else(|| PlotError::missing_column(col_name.as_str()))?;

                // Convert to f64 and optionally apply scale
                match vec.vtype() {
                    VectorType::Float => {
                        let floats = vec.iter_float().ok_or_else(|| {
                            PlotError::InvalidAestheticType {
                                aesthetic,
                                expected: DataType::Vector(VectorType::Float),
                                actual: DataType::Custom("unknown".to_string()),
                            }
                        })?;

                        if let Some(scale) = scale {
                            // Need to collect for scale application
                            let values: Vec<f64> =
                                floats.filter_map(|v| scale.map_value(v)).collect();
                            Ok(AestheticValues::Owned(values))
                        } else {
                            // Collect to owned since FloatRef expects std::slice::Iter
                            let values: Vec<f64> = floats.collect();
                            Ok(AestheticValues::Owned(values))
                        }
                    }
                    VectorType::Int => {
                        let ints = vec.iter_int().ok_or_else(|| {
                            PlotError::InvalidAestheticType {
                                aesthetic,
                                expected: DataType::Vector(VectorType::Int),
                                actual: DataType::Custom("unknown".to_string()),
                            }
                        })?;

                        // Need to convert ints to f64, so collect
                        let values: Vec<f64> = ints.map(|x| x as f64).collect();

                        if let Some(scale) = scale {
                            let scaled: Vec<f64> = values
                                .into_iter()
                                .filter_map(|v| scale.map_value(v))
                                .collect();
                            Ok(AestheticValues::Owned(scaled))
                        } else {
                            Ok(AestheticValues::Owned(values))
                        }
                    }
                    VectorType::Str => {
                        // Try to use scale's categorical mapping
                        if let Some(scale) = scale {
                            let strs = vec.iter_str().ok_or_else(|| {
                                PlotError::InvalidAestheticType {
                                    aesthetic,
                                    expected: DataType::Vector(VectorType::Str),
                                    actual: DataType::Custom("unknown".to_string()),
                                }
                            })?;

                            let strs_vec: Vec<&str> = strs.collect();
                            let mapped: Vec<f64> =
                                strs_vec.iter().filter_map(|s| scale.map_category(s)).collect();

                            if mapped.len() == strs_vec.len() {
                                Ok(AestheticValues::Owned(mapped))
                            } else {
                                Err(PlotError::InvalidAestheticType {
                                    aesthetic,
                                    expected: DataType::Custom("all values mapped by scale".to_string()),
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
                }
            }
            Some(AesValue::CategoricalColumn(col_name)) => {
                // Treat numeric column as categorical by converting to strings
                let vec = self
                    .data
                    .get(col_name.as_str())
                    .ok_or_else(|| PlotError::missing_column(col_name.as_str()))?;

                // Require a scale for categorical mapping
                let scale = scale.ok_or_else(|| PlotError::InvalidAestheticType {
                    aesthetic,
                    expected: DataType::CategoricalScale,
                    actual: DataType::Custom("no scale provided for categorical column".to_string()),
                })?;

                match vec.vtype() {
                    VectorType::Float => {
                        let floats = vec.iter_float().ok_or_else(|| {
                            PlotError::InvalidAestheticType {
                                aesthetic,
                                expected: DataType::Vector(VectorType::Float),
                                actual: DataType::Custom("unknown".to_string()),
                            }
                        })?;

                        // Convert floats to strings and use categorical mapping
                        let strings: Vec<String> = floats.map(|f| f.to_string()).collect();
                        let mapped: Vec<f64> = strings
                            .iter()
                            .filter_map(|s| scale.map_category(s.as_str()))
                            .collect();

                        if mapped.len() == strings.len() {
                            Ok(AestheticValues::Owned(mapped))
                        } else {
                            Err(PlotError::InvalidAestheticType {
                                aesthetic,
                                expected: DataType::Custom("all values mapped by scale".to_string()),
                                actual: DataType::Custom("some values not mapped".to_string()),
                            })
                        }
                    }
                    VectorType::Int => {
                        let ints = vec.iter_int().ok_or_else(|| {
                            PlotError::InvalidAestheticType {
                                aesthetic,
                                expected: DataType::Vector(VectorType::Int),
                                actual: DataType::Custom("unknown".to_string()),
                            }
                        })?;

                        // Convert ints to strings and use categorical mapping
                        let strings: Vec<String> = ints.map(|i| i.to_string()).collect();
                        let mapped: Vec<f64> = strings
                            .iter()
                            .filter_map(|s| scale.map_category(s.as_str()))
                            .collect();

                        if mapped.len() == strings.len() {
                            Ok(AestheticValues::Owned(mapped))
                        } else {
                            Err(PlotError::InvalidAestheticType {
                                aesthetic,
                                expected: DataType::Custom("all values mapped by scale".to_string()),
                                actual: DataType::Custom("some values not mapped".to_string()),
                            })
                        }
                    }
                    VectorType::Str => {
                        // Already strings, use categorical mapping
                        let strs = vec.iter_str().ok_or_else(|| {
                            PlotError::InvalidAestheticType {
                                aesthetic,
                                expected: DataType::Vector(VectorType::Str),
                                actual: DataType::Custom("unknown".to_string()),
                            }
                        })?;

                        let strs_vec: Vec<&str> = strs.collect();
                        let mapped: Vec<f64> = strs_vec
                            .iter()
                            .filter_map(|s| scale.map_category(s))
                            .collect();

                        if mapped.len() == strs_vec.len() {
                            Ok(AestheticValues::Owned(mapped))
                        } else {
                            Err(PlotError::InvalidAestheticType {
                                aesthetic,
                                expected: DataType::Custom("all values mapped by scale".to_string()),
                                actual: DataType::Custom("some values not mapped".to_string()),
                            })
                        }
                    }
                }
            }
            Some(AesValue::Constant(prim)) => {
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
                };
                Ok(AestheticValues::Constant(value, n))
            }
            None => {
                // Use default value replicated to match data length
                let default_value = match aesthetic {
                    Aesthetic::Size => 2.0,
                    Aesthetic::Alpha => 1.0,
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

        let n = self.data.len();
        let mapping = self.mapping.get(&Aesthetic::Color);
        let color_scale = self.scales.color.as_ref();

        match (mapping, color_scale) {
            // Column mapped with scale
            (Some(AesValue::Column(col_name)), Some(scale)) => {
                let vec = self
                    .data
                    .get(col_name.as_str())
                    .ok_or_else(|| PlotError::missing_column(col_name.as_str()))?;

                match vec.vtype() {
                    VectorType::Float | VectorType::Int => {
                        // Continuous color scale
                        let values: Vec<f64> = match vec.vtype() {
                            VectorType::Float => vec
                                .iter_float()
                                .ok_or_else(|| {
                                    PlotError::InvalidAestheticType {
                                        aesthetic: Aesthetic::Color,
                                        expected: DataType::Vector(VectorType::Float),
                                        actual: DataType::Custom("unknown".to_string()),
                                    }
                                })?
                                .collect(),
                            VectorType::Int => vec
                                .iter_int()
                                .ok_or_else(|| {
                                    PlotError::InvalidAestheticType {
                                        aesthetic: Aesthetic::Color,
                                        expected: DataType::Vector(VectorType::Int),
                                        actual: DataType::Custom("unknown".to_string()),
                                    }
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
                        let strings = vec.iter_str().ok_or_else(|| {
                            PlotError::InvalidAestheticType {
                                aesthetic: Aesthetic::Color,
                                expected: DataType::Vector(VectorType::Str),
                                actual: DataType::Custom("unknown".to_string()),
                            }
                        })?;

                        let colors: Vec<Color> = strings
                            .filter_map(|s| scale.map_discrete_to_color(s))
                            .collect();
                        Ok(ColorValues::Mapped(colors))
                    }
                }
            }
            // CategoricalColumn - treat numeric data as discrete categories
            (Some(AesValue::CategoricalColumn(col_name)), Some(scale)) => {
                let vec = self
                    .data
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
                };

                let colors: Vec<Color> = strings
                    .iter()
                    .filter_map(|s| scale.map_discrete_to_color(s.as_str()))
                    .collect();
                Ok(ColorValues::Mapped(colors))
            }
            // Constant color
            (Some(AesValue::Constant(PrimitiveValue::Int(rgba))), _) => {
                let r = ((rgba >> 24) & 0xFF) as u8;
                let g = ((rgba >> 16) & 0xFF) as u8;
                let b = ((rgba >> 8) & 0xFF) as u8;
                let a = (rgba & 0xFF) as u8;
                Ok(ColorValues::Constant(Color(r, g, b, a), n))
            }
            (Some(AesValue::Constant(_)), _) => Err(PlotError::InvalidAestheticType {
                aesthetic: Aesthetic::Color,
                expected: DataType::RgbaConstant,
                actual: DataType::Custom("other constant".to_string()),
            }),
            // Column mapped but no scale
            (Some(AesValue::Column(_) | AesValue::CategoricalColumn(_)), None) => Err(PlotError::InvalidAestheticType {
                aesthetic: Aesthetic::Color,
                expected: DataType::Custom("color scale for column mapping".to_string()),
                actual: DataType::Custom("no scale provided".to_string()),
            }),
            // No mapping, use default
            (None, _) => Ok(ColorValues::Constant(Color(0, 0, 0, 255), n)),
        }
    }

    /// Get fill color values for the Fill aesthetic as an iterator
    /// Handles both constants and scale-mapped colors
    pub fn get_fill_color_values(&self) -> Result<ColorValues, PlotError> {
        use crate::aesthetics::AesValue;
        use crate::data::PrimitiveValue;

        let n = self.data.len();
        let mapping = self.mapping.get(&Aesthetic::Fill);
        let color_scale = self.scales.fill.as_ref();

        match (mapping, color_scale) {
            // Column mapped with scale (continuous)
            (Some(AesValue::Column(col_name)), Some(scale)) => {
                let vec = self
                    .data
                    .get(col_name.as_str())
                    .ok_or_else(|| PlotError::missing_column(col_name.as_str()))?;

                match vec.vtype() {
                    VectorType::Float | VectorType::Int => {
                        // Continuous color scale
                        let values: Vec<f64> = match vec.vtype() {
                            VectorType::Float => vec
                                .iter_float()
                                .ok_or_else(|| {
                                    PlotError::InvalidAestheticType {
                                        aesthetic: Aesthetic::Fill,
                                        expected: DataType::Vector(VectorType::Float),
                                        actual: DataType::Custom("unknown".to_string()),
                                    }
                                })?
                                .collect(),
                            VectorType::Int => vec
                                .iter_int()
                                .ok_or_else(|| {
                                    PlotError::InvalidAestheticType {
                                        aesthetic: Aesthetic::Fill,
                                        expected: DataType::Vector(VectorType::Int),
                                        actual: DataType::Custom("unknown".to_string()),
                                    }
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
                        let strings = vec.iter_str().ok_or_else(|| {
                            PlotError::InvalidAestheticType {
                                aesthetic: Aesthetic::Fill,
                                expected: DataType::Vector(VectorType::Str),
                                actual: DataType::Custom("unknown".to_string()),
                            }
                        })?;

                        let colors: Vec<Color> = strings
                            .filter_map(|s| scale.map_discrete_to_color(s))
                            .collect();
                        Ok(ColorValues::Mapped(colors))
                    }
                }
            }
            // CategoricalColumn - treat numeric data as discrete categories
            (Some(AesValue::CategoricalColumn(col_name)), Some(scale)) => {
                let vec = self
                    .data
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
                };

                let colors: Vec<Color> = strings
                    .iter()
                    .filter_map(|s| scale.map_discrete_to_color(s.as_str()))
                    .collect();
                Ok(ColorValues::Mapped(colors))
            }
            // Constant color
            (Some(AesValue::Constant(PrimitiveValue::Int(rgba))), _) => {
                let r = ((rgba >> 24) & 0xFF) as u8;
                let g = ((rgba >> 16) & 0xFF) as u8;
                let b = ((rgba >> 8) & 0xFF) as u8;
                let a = (rgba & 0xFF) as u8;
                Ok(ColorValues::Constant(Color(r, g, b, a), n))
            }
            (Some(AesValue::Constant(_)), _) => Err(PlotError::InvalidAestheticType {
                aesthetic: Aesthetic::Fill,
                expected: DataType::RgbaConstant,
                actual: DataType::Custom("other constant".to_string()),
            }),
            // Column mapped but no scale
            (Some(AesValue::Column(_) | AesValue::CategoricalColumn(_)), None) => Err(PlotError::InvalidAestheticType {
                aesthetic: Aesthetic::Fill,
                expected: DataType::Custom("fill scale for column mapping".to_string()),
                actual: DataType::Custom("no scale provided".to_string()),
            }),
            // No mapping, use default gray
            (None, _) => Ok(ColorValues::Constant(Color(128, 128, 128, 255), n)),
        }
    }

    /// Get shape values for the Shape aesthetic as an iterator
    /// Handles both constants and scale-mapped shapes
    pub fn get_shape_values(&self) -> Result<ShapeValues, PlotError> {
        use crate::aesthetics::AesValue;
        use crate::data::PrimitiveValue;
        use crate::visuals::Shape;

        let n = self.data.len();
        let mapping = self.mapping.get(&Aesthetic::Shape);
        let shape_scale = self.scales.shape.as_ref();

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
            // Column mapped with scale
            (Some(AesValue::Column(col_name) | AesValue::CategoricalColumn(col_name)), Some(scale)) => {
                let vec = self
                    .data
                    .get(col_name.as_str())
                    .ok_or_else(|| PlotError::missing_column(col_name.as_str()))?;

                if let Some(strings) = vec.iter_str() {
                    let shapes: Vec<Shape> = strings
                        .filter_map(|s| scale.map_to_shape(s))
                        .collect();
                    Ok(ShapeValues::Mapped(shapes))
                } else {
                    Err(PlotError::InvalidAestheticType {
                        aesthetic: Aesthetic::Shape,
                        expected: DataType::Custom("string (categorical)".to_string()),
                        actual: DataType::Numeric,
                    })
                }
            }
            // Constant shape
            (Some(AesValue::Constant(PrimitiveValue::Int(v))), _) => {
                Ok(ShapeValues::Constant(int_to_shape(*v), n))
            }
            (Some(AesValue::Constant(_)), _) => Err(PlotError::InvalidAestheticType {
                aesthetic: Aesthetic::Shape,
                expected: DataType::Constant(VectorType::Int),
                actual: DataType::Custom("other constant".to_string()),
            }),
            // Column mapped but no scale
            (Some(AesValue::Column(_) | AesValue::CategoricalColumn(_)), None) => Err(PlotError::InvalidAestheticType {
                aesthetic: Aesthetic::Shape,
                expected: DataType::Custom("shape scale for column mapping".to_string()),
                actual: DataType::Custom("no scale provided".to_string()),
            }),
            // No mapping, use default
            (None, _) => Ok(ShapeValues::Constant(Shape::Circle, n)),
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
            .mapping
            .get(&aesthetic)
            .ok_or(PlotError::MissingAesthetic { aesthetic })?;

        // Extract column name
        let col_name = match mapping {
            AesValue::Column(name) | AesValue::CategoricalColumn(name) => name.as_str(),
            AesValue::Constant(_) => {
                return Err(PlotError::InvalidAestheticType {
                    aesthetic,
                    expected: DataType::ColumnMapping,
                    actual: DataType::Custom("constant".to_string()),
                });
            }
        };

        // Get the data vector
        let vec = self
            .data
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
                ints.map(|v| v as f64).collect()
            }
            VectorType::Str => {
                return Err(PlotError::InvalidAestheticType {
                    aesthetic,
                    expected: DataType::Numeric,
                    actual: DataType::Vector(VectorType::Str),
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
}
