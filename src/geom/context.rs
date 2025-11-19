use crate::aesthetics::{AesMap, AesValue, Aesthetic};
use crate::data::{DataSource, VectorType};
use crate::error::PlotError;
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
        scale: Option<&Box<dyn ContinuousScale>>,
    ) -> Result<AestheticValues<'a>, PlotError> {
        use crate::aesthetics::AesValue;
        use crate::data::PrimitiveValue;

        let mapping = self.mapping.get(&aesthetic);
        let n = self.data.len();

        match mapping {
            Some(AesValue::Column(col_name)) => {
                // Get data from column
                let vec = self.data.get(col_name.as_str())
                    .ok_or_else(|| PlotError::MissingAesthetic(format!("column '{}'", col_name)))?;
                
                // Convert to f64 and optionally apply scale
                match vec.vtype() {
                    VectorType::Float => {
                        let floats = vec.as_float()
                            .ok_or_else(|| PlotError::InvalidAestheticType("expected float".to_string()))?;
                        
                        if let Some(scale) = scale {
                            // Need to collect for scale application
                            let values: Vec<f64> = floats.iter()
                                .filter_map(|&v| scale.map_value(v))
                                .collect();
                            Ok(AestheticValues::Owned(values))
                        } else {
                            Ok(AestheticValues::FloatRef(floats.iter()))
                        }
                    }
                    VectorType::Int => {
                        let ints = vec.as_int()
                            .ok_or_else(|| PlotError::InvalidAestheticType("expected int".to_string()))?;
                        
                        // Need to convert ints to f64, so collect
                        let values: Vec<f64> = ints.iter().map(|&x| x as f64).collect();
                        
                        if let Some(scale) = scale {
                            let scaled: Vec<f64> = values.into_iter()
                                .filter_map(|v| scale.map_value(v))
                                .collect();
                            Ok(AestheticValues::Owned(scaled))
                        } else {
                            Ok(AestheticValues::Owned(values))
                        }
                    }
                    VectorType::Str => {
                        return Err(PlotError::InvalidAestheticType(
                            "cannot convert string to numeric".to_string(),
                        ))
                    }
                }
            }
            Some(AesValue::Constant(prim)) => {
                // Replicate constant value to match data length
                let value = match prim {
                    PrimitiveValue::Float(v) => *v,
                    PrimitiveValue::Int(v) => *v as f64,
                    PrimitiveValue::Str(_) => {
                        return Err(PlotError::InvalidAestheticType(
                            format!("Aesthetic {:?} has string constant, expected numeric", aesthetic)
                        ))
                    }
                };
                Ok(AestheticValues::Constant(value, n))
            }
            None => {
                // Use default value replicated to match data length
                let default_value = match aesthetic {
                    Aesthetic::Size => 2.0,
                    Aesthetic::Alpha => 1.0,
                    _ => return Err(PlotError::MissingAesthetic(format!("{:?}", aesthetic)))
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
                let vec = self.data.get(col_name.as_str())
                    .ok_or_else(|| PlotError::MissingAesthetic(format!("column '{}'", col_name)))?;
                
                match vec.vtype() {
                    VectorType::Float | VectorType::Int => {
                        // Continuous color scale
                        let values: Vec<f64> = match vec.vtype() {
                            VectorType::Float => {
                                vec.as_float()
                                    .ok_or_else(|| PlotError::InvalidAestheticType("expected float".to_string()))?
                                    .iter().copied().collect()
                            }
                            VectorType::Int => {
                                vec.as_int()
                                    .ok_or_else(|| PlotError::InvalidAestheticType("expected int".to_string()))?
                                    .iter().map(|&x| x as f64).collect()
                            }
                            _ => unreachable!()
                        };
                        
                        let colors: Vec<Color> = values.iter()
                            .filter_map(|&v| scale.map_continuous_to_color(v))
                            .collect();
                        Ok(ColorValues::Mapped(colors))
                    }
                    VectorType::Str => {
                        // Discrete color scale
                        let strings = vec.as_str()
                            .ok_or_else(|| PlotError::InvalidAestheticType("expected string".to_string()))?;
                        
                        let colors: Vec<Color> = strings.iter()
                            .filter_map(|s| scale.map_discrete_to_color(s))
                            .collect();
                        Ok(ColorValues::Mapped(colors))
                    }
                }
            }
            // Constant color
            (Some(AesValue::Constant(PrimitiveValue::Int(rgba))), _) => {
                let r = ((rgba >> 24) & 0xFF) as u8;
                let g = ((rgba >> 16) & 0xFF) as u8;
                let b = ((rgba >> 8) & 0xFF) as u8;
                let a = (rgba & 0xFF) as u8;
                Ok(ColorValues::Constant(Color(r, g, b, a), n))
            }
            (Some(AesValue::Constant(_)), _) => {
                Err(PlotError::InvalidAestheticType(
                    "Color constant must be RGBA int".to_string()
                ))
            }
            // Column mapped but no scale
            (Some(AesValue::Column(_)), None) => {
                Err(PlotError::InvalidAestheticType(
                    "Color mapped from column requires a color scale".to_string()
                ))
            }
            // No mapping, use default
            (None, _) => {
                Ok(ColorValues::Constant(Color(0, 0, 0, 255), n))
            }
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
            (Some(AesValue::Column(col_name)), Some(scale)) => {
                let vec = self.data.get(col_name.as_str())
                    .ok_or_else(|| PlotError::MissingAesthetic(format!("column '{}'", col_name)))?;
                
                if let Some(strings) = vec.as_str() {
                    let shapes: Vec<Shape> = strings.iter()
                        .filter_map(|s| scale.map_to_shape(s))
                        .collect();
                    Ok(ShapeValues::Mapped(shapes))
                } else {
                    Err(PlotError::InvalidAestheticType(
                        "Shape mapping requires categorical (string) data".to_string()
                    ))
                }
            }
            // Constant shape
            (Some(AesValue::Constant(PrimitiveValue::Int(v))), _) => {
                Ok(ShapeValues::Constant(int_to_shape(*v), n))
            }
            (Some(AesValue::Constant(_)), _) => {
                Err(PlotError::InvalidAestheticType(
                    "Shape constant must be int".to_string()
                ))
            }
            // Column mapped but no scale
            (Some(AesValue::Column(_)), None) => {
                Err(PlotError::InvalidAestheticType(
                    "Shape mapped from column requires a shape scale".to_string()
                ))
            }
            // No mapping, use default
            (None, _) => {
                Ok(ShapeValues::Constant(Shape::Circle, n))
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
        scale: Option<&Box<dyn ContinuousScale>>,
    ) -> Result<Vec<f64>, PlotError> {
        // Get the aesthetic mapping
        let mapping = self
            .mapping
            .get(&aesthetic)
            .ok_or_else(|| PlotError::MissingAesthetic(format!("{:?}", aesthetic)))?;

        // Extract column name
        let col_name = match mapping {
            AesValue::Column(name) => name.as_str(),
            AesValue::Constant(_) => {
                return Err(PlotError::InvalidAestheticType(
                    "constant aesthetics not yet supported".to_string(),
                ))
            }
        };

        // Get the data vector
        let vec = self
            .data
            .get(col_name)
            .ok_or_else(|| PlotError::MissingAesthetic(format!("column '{}'", col_name)))?;

        // Convert to f64 based on type
        let values: Vec<f64> = match vec.vtype() {
            VectorType::Float => {
                let floats = vec
                    .as_float()
                    .ok_or_else(|| PlotError::InvalidAestheticType("expected float".to_string()))?;
                floats.iter().copied().collect()
            }
            VectorType::Int => {
                let ints = vec
                    .as_int()
                    .ok_or_else(|| PlotError::InvalidAestheticType("expected int".to_string()))?;
                ints.iter().map(|&x| x as f64).collect()
            }
            VectorType::Str => {
                return Err(PlotError::InvalidAestheticType(
                    "cannot convert string to numeric".to_string(),
                ))
            }
        };

        // Apply scale if present
        if let Some(scale) = scale {
            Ok(values
                .iter()
                .filter_map(|&v| scale.map_value(v))
                .collect())
        } else {
            Ok(values)
        }
    }
}
