use crate::{
    data::{DataSource, VectorIter},
    error::PlotError,
    theme::{Color, color::BLACK},
    utils::Either,
    visuals::Shape,
};

pub enum Property {
    Float(FloatProperty),
    String(StringProperty),
    Color(ColorProperty),
    Shape(ShapeProperty),
}

    #[derive(Debug, Clone)]
pub enum PropertyValue {
    Int(i64),
    Float(f64),
    String(String),
    Color(Color),
    Shape(Shape),    
}

#[derive(Debug, Clone)]
pub enum PropertyVector {
    Int(Vec<i64>),
    Float(Vec<f64>),
    String(Vec<String>),
    Color(Vec<Color>),
    Shape(Vec<Shape>),
}

impl PropertyVector {
    pub fn to_color(self) -> PropertyVector {
        match self {
            PropertyVector::Int(v) => {
                let colors: Vec<Color> = v.into_iter().map(|c| Color::from(c)).collect();
                PropertyVector::Color(colors)
            }
            PropertyVector::Color(_) => self.clone(),
            _ => panic!("Cannot convert to Color PropertyVector"),
        }
    }

    pub fn to_shape(self) -> PropertyVector {
        match self {
            PropertyVector::Int(v) => {
                log::info!("Converting int vector to shapes: {:?}", v);
                let shapes: Vec<Shape> = v.into_iter().map(|s| {
                    let shape = Shape::from(s);
                    log::info!("  {} -> {:?}", s, shape);
                    shape
                }).collect();
                PropertyVector::Shape(shapes)
            }
            PropertyVector::Shape(_) => self.clone(),
            _ => panic!("Cannot convert to Shape PropertyVector"),
        }
    }

    pub fn as_floats(self) -> Vec<f64> {
        match self {
            PropertyVector::Float(v) => v,
            _ => panic!("Not a Float PropertyVector"),
        }
    }

    pub fn as_strings(self) -> Vec<String> {
        match self {
            PropertyVector::String(v) => v,
            _ => panic!("Not a String PropertyVector"),
        }
    }

    pub fn as_colors(self) -> Vec<Color> {
        match self.to_color() {
            PropertyVector::Color(v) => v,
            _ => panic!("Not a Color PropertyVector"),
        }
    }

    pub fn as_shapes(self) -> Vec<Shape> {
        match self.to_shape() {
            PropertyVector::Shape(v) => v,
            _ => panic!("Not a Shape PropertyVector"),
        }
    }
}

impl<'a> From<VectorIter<'a>> for PropertyVector {
    fn from(iter: VectorIter<'a>) -> Self {
        match iter {
            VectorIter::Int(iter) => PropertyVector::Int(iter.collect()),
            VectorIter::Float(iter) => PropertyVector::Float(iter.collect()),
            VectorIter::Str(iter) => PropertyVector::String(iter.map(|s| s.to_string()).collect()),
            VectorIter::Bool(_) => panic!("invalid property vector"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct FloatProperty {
    pub value: Either<f64, String>,
}

impl FloatProperty {
    pub fn new() -> Self {
        Self {
            value: Either::Left(1.0),
        }
    }

    /// Set the float property
    pub fn value(&mut self, value: f64) -> &mut Self {
        self.value = Either::Left(value);
        self
    }

    /// Get a float iterator
    pub fn iter<'a>(
        &'a self,
        data: &'a dyn DataSource,
    ) -> Result<Box<dyn Iterator<Item = f64> + 'a>, PlotError> {
        match &self.value {
            Either::Left(value) => Ok(Box::new(std::iter::repeat(*value))),
            Either::Right(column) => {
                let v = data
                    .get(&column)
                    .ok_or(crate::error::PlotError::MissingColumn {
                        column: column.clone(),
                    })?;
                Ok(Box::new(crate::utils::data::make_float_iter(v.iter())))
            }
        }
    }
}

impl Into<FloatProperty> for f64 {
    fn into(self) -> FloatProperty {
        let mut prop = FloatProperty::new();
        prop.value(self);
        prop
    }
}

impl Into<FloatProperty> for &str {
    fn into(self) -> FloatProperty {
        let mut prop = FloatProperty::new();
        prop.value = Either::Right(self.to_string());
        prop
    }
}

impl Into<FloatProperty> for String {
    fn into(self) -> FloatProperty {
        let mut prop = FloatProperty::new();
        prop.value = Either::Right(self);
        prop
    }
}

#[derive(Debug, Clone)]
pub struct StringProperty {
    pub value: Either<String, String>,
}

impl StringProperty {
    pub fn new() -> Self {
        Self {
            value: Either::Left(String::new()),
        }
    }

    /// Set the string property
    pub fn value(&mut self, value: String) -> &mut Self {
        self.value = Either::Left(value);
        self
    }

    /// Get a string iterator
    pub fn iter<'a>(
        &'a self,
        data: &'a dyn DataSource,
    ) -> Result<Box<dyn Iterator<Item = String> + 'a>, PlotError> {
        match &self.value {
            Either::Left(value) => Ok(Box::new(std::iter::repeat(value.clone()))),
            Either::Right(column) => {
                let v = data
                    .get(&column)
                    .ok_or(crate::error::PlotError::MissingColumn {
                        column: column.clone(),
                    })?;
                Ok(Box::new(crate::utils::data::make_string_iter(v.iter())))
            }
        }
    }
}

impl Into<StringProperty> for &str {
    fn into(self) -> StringProperty {
        let mut prop = StringProperty::new();
        prop.value(self.to_string());
        prop
    }
}

impl Into<StringProperty> for String {
    fn into(self) -> StringProperty {
        let mut prop = StringProperty::new();
        prop.value(self);
        prop
    }
}

#[derive(Debug, Clone)]
pub struct ColorProperty {
    pub color: Either<Color, String>,
}

impl ColorProperty {
    pub fn new() -> Self {
        Self {
            color: Either::Left(BLACK),
        }
    }

    /// Set the color property
    pub fn color(&mut self, color: Color) -> &mut Self {
        self.color = Either::Left(color);
        self
    }

    /// Get a color iterator
    pub fn iter<'a>(
        &'a self,
        data: &'a dyn DataSource,
    ) -> Result<Box<dyn Iterator<Item = Color> + 'a>, PlotError> {
        match &self.color {
            Either::Left(color) => Ok(Box::new(std::iter::repeat(*color))),
            Either::Right(column) => {
                let v = data
                    .get(&column)
                    .ok_or(crate::error::PlotError::MissingColumn {
                        column: column.clone(),
                    })?;
                Ok(Box::new(crate::utils::data::make_color_iter(v.iter())))
            }
        }
    }
}

impl Into<ColorProperty> for Color {
    fn into(self) -> ColorProperty {
        let mut prop = ColorProperty::new();
        prop.color(self);
        prop
    }
}

impl Into<ColorProperty> for &str {
    fn into(self) -> ColorProperty {
        let mut prop = ColorProperty::new();
        prop.color = Either::Right(self.to_string());
        prop
    }
}

impl Into<ColorProperty> for String {
    fn into(self) -> ColorProperty {
        let mut prop = ColorProperty::new();
        prop.color = Either::Right(self);
        prop
    }
}

#[derive(Debug, Clone)]
pub struct ShapeProperty {
    pub shape: Either<Shape, String>,
}

impl Into<ShapeProperty> for Shape {
    fn into(self) -> ShapeProperty {
        let mut prop = ShapeProperty::new();
        prop.shape(self);
        prop
    }
}

impl ShapeProperty {
    pub fn new() -> Self {
        Self {
            shape: Either::Left(Shape::Circle),
        }
    }

    /// Set the shape property
    pub fn shape(&mut self, shape: Shape) -> &mut Self {
        self.shape = Either::Left(shape);
        self
    }

    /// Get a shape iterator
    pub fn iter<'a>(
        &'a self,
        data: &'a dyn DataSource,
    ) -> Result<Box<dyn Iterator<Item = Shape> + 'a>, PlotError> {
        match &self.shape {
            Either::Left(shape) => Ok(Box::new(std::iter::repeat(*shape))),
            Either::Right(column) => {
                let v = data
                    .get(&column)
                    .ok_or(crate::error::PlotError::MissingColumn {
                        column: column.clone(),
                    })?;
                Ok(Box::new(crate::utils::data::make_shape_iter(v.iter())))
            }
        }
    }
}
