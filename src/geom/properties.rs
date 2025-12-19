use crate::{
    aesthetics::AesMap,
    data::DataSource,
    error::PlotError,
    theme::{Color, color::BLACK},
    utils::Either,
    visuals::Shape,
};

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
        _mapping: &'a AesMap,
    ) -> Result<Box<dyn Iterator<Item = f64> + 'a>, PlotError> {
        match &self.value {
            Either::Left(value) => Ok(Box::new(std::iter::repeat(*value))),
            Either::Right(column) => {
                let v = data.get(&column)
                    .ok_or(crate::error::PlotError::MissingColumn { column: column.clone() })?;
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
        _mapping: &'a AesMap,
    ) -> Result<Box<dyn Iterator<Item = Color> + 'a>, PlotError> {
        match &self.color {
            Either::Left(color) => Ok(Box::new(std::iter::repeat(*color))),
            Either::Right(column) => {
                let v =data.get(&column)
                    .ok_or(crate::error::PlotError::MissingColumn { column: column.clone() })?;
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

pub struct ShapeProperty {
    pub shape: Either<Shape, String>,
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
        _mapping: &'a AesMap,
    ) -> Result<Box<dyn Iterator<Item = Shape> + 'a>, PlotError> {
        match &self.shape {
            Either::Left(shape) => Ok(Box::new(std::iter::repeat(*shape))),
            Either::Right(column) => {
                let v = data.get(&column)
                    .ok_or(crate::error::PlotError::MissingColumn { column: column.clone() })?;
                Ok(Box::new(crate::utils::data::make_shape_iter(v.iter())))
            }
        }
    }
}
