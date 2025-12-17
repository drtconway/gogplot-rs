use crate::{
    aesthetics::{AesMap, Aesthetic},
    data::DataSource,
    error::PlotError,
    theme::{Color, color::BLACK},
    utils::Either,
    visuals::Shape,
};

pub struct FloatProperty {
    pub value: Either<f64, Aesthetic>,
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
        mapping: &'a AesMap,
    ) -> Result<Box<dyn Iterator<Item = f64> + 'a>, PlotError> {
        match &self.value {
            Either::Left(value) => Ok(Box::new(std::iter::repeat(*value))),
            Either::Right(aesthetic) => {
                let iter = data.aesthetic_value_iter(mapping, *aesthetic).ok_or(
                    crate::error::PlotError::MissingAesthetic {
                        aesthetic: *aesthetic,
                    },
                )?;
                Ok(Box::new(crate::utils::data::make_float_iter(iter)))
            }
        }
    }
}

pub struct ColorProperty {
    pub color: Either<Color, Aesthetic>,
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
        mapping: &'a AesMap,
    ) -> Result<Box<dyn Iterator<Item = Color> + 'a>, PlotError> {
        match &self.color {
            Either::Left(color) => Ok(Box::new(std::iter::repeat(*color))),
            Either::Right(aesthetic) => {
                let iter = data.aesthetic_value_iter(mapping, *aesthetic).ok_or(
                    crate::error::PlotError::MissingAesthetic {
                        aesthetic: *aesthetic,
                    },
                )?;
                Ok(Box::new(crate::utils::data::make_color_iter(iter)))
            }
        }
    }
}

pub struct ShapeProperty {
    pub shape: Either<Shape, Aesthetic>,
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
        mapping: &'a AesMap,
    ) -> Result<Box<dyn Iterator<Item = Shape> + 'a>, PlotError> {
        match &self.shape {
            Either::Left(shape) => Ok(Box::new(std::iter::repeat(*shape))),
            Either::Right(aesthetic) => {
                let iter = data.aesthetic_value_iter(mapping, *aesthetic).ok_or(
                    crate::error::PlotError::MissingAesthetic {
                        aesthetic: *aesthetic,
                    },
                )?;
                Ok(Box::new(crate::utils::data::make_shape_iter(iter)))
            }
        }
    }
}
