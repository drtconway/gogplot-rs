use core::panic;

use crate::PlotError;
use crate::aesthetics::{AesValue, AestheticDomain};
use crate::data::{
    DataSource, DiscreteType, GenericVector, PrimitiveType, PrimitiveValue, VectorIter,
};
use crate::error::{DataType, Result};
use crate::scale::{ContinuousScaleTrainer, DiscreteScaleTrainer};
use crate::theme::Color;

use crate::utils::data::{visit_c, visit_d};
use crate::utils::dataframe::{BoolVec, FloatVec, IntVec, StrVec};
use crate::utils::set::DiscreteSet;
use crate::visuals::{LineStyle, Shape};

/// Base trait for all scales providing common functionality.
pub trait ScaleBase: Default + Clone + Send + Sync {
    /// Train the scale on data to automatically determine the domain.
    ///
    /// This method allows the scale to learn appropriate domain bounds by examining
    /// the data. For continuous scales, this typically computes min/max values
    /// across all provided data sources. For categorical scales, this extracts
    /// unique categories. This may be called multiple times with different data
    /// to incrementally refine the domain. The domain should be taken as the union
    /// of all data seen so far.
    ///
    /// If the scale's domain is already explicitly set (e.g., via a builder),
    /// this method may be a no-op.
    ///
    /// # Arguments
    /// * `data` - A slice of data vectors to train on (e.g., for rectangles this
    ///            would include both xmin and xmax to get the full range)
    fn train<'a>(&mut self, iter: VectorIter<'a>);

    fn train_one(&mut self, value: &PrimitiveValue) {
        match value {
            PrimitiveValue::Int(x) => {
                let xs: IntVec = IntVec::from(vec![*x]);
                self.train(xs.iter());
            }
            PrimitiveValue::Float(x) => {
                let xs = FloatVec::from(vec![*x]);
                self.train(xs.iter());
            }
            PrimitiveValue::Str(x) => {
                let xs = StrVec::from(vec![x.clone()]);
                self.train(xs.iter());
            }
            PrimitiveValue::Bool(x) => {
                let xs = BoolVec::from(vec![*x]);
                self.train(xs.iter());
            }
        }
    }
}

pub trait ContinuousDomainScale: ScaleBase {
    fn domain(&self) -> Option<(f64, f64)>;

    fn set_domain(&mut self, domain: (f64, f64));

    fn limits(&self) -> (Option<f64>, Option<f64>);

    fn breaks(&self) -> &[f64];

    fn labels(&self) -> &[String];

    fn train_continuous<'a>(&mut self, iter: VectorIter<'a>) {
        let mut trainer = ContinuousScaleTrainer::new();
        visit_c(iter, &mut trainer).unwrap();
        if let Some((obs_min_value, obs_max_value)) = trainer.bounds {
            let (min_limit, max_limit) = self.limits();
            let min_value = min_limit.unwrap_or(obs_min_value);
            let max_value = max_limit.unwrap_or(obs_max_value);

            // Apply 5% expansion on each side (ggplot2 default)
            let range = max_value - min_value;
            let expansion = range * 0.05;
            let min_value = min_value - expansion;
            let max_value = max_value + expansion;

            if let Some((min_existing, max_existing)) = self.domain() {
                let min_value = min_value.min(min_existing);
                let max_value = max_value.max(max_existing);
                self.set_domain((min_value, max_value));
            } else {
                self.set_domain((min_value, max_value));
            }
            log::info!(
                "Trained continuous scale domain to ({}, {}) with 5% expansion",
                min_value,
                max_value
            );
        }
    }
}

pub trait DiscreteDomainScale: ScaleBase {
    fn categories(&self) -> &DiscreteSet;

    fn add_categories(&mut self, categories: DiscreteSet);

    fn train_discrete<'a>(&mut self, iter: VectorIter<'a>) {
        let mut trainer = DiscreteScaleTrainer::new();
        visit_d(iter, &mut trainer).unwrap();
        self.add_categories(trainer.categories);
    }

    fn len(&self) -> usize {
        self.categories().len()
    }

    fn ordinal<T: DiscreteType>(&self, value: &T) -> Option<usize> {
        self.categories().ordinal(value)
    }
}

pub trait ContinuousRangeScale: ScaleBase {
    /// Map a value from the data domain to normalized [0, 1] coordinates.
    ///
    /// # Arguments
    /// * `value` - A value in the data domain
    ///
    /// # Returns
    /// * `Some(normalized_value)` - The corresponding value in [0, 1] range
    /// * `None` - If the value is outside the scale's domain bounds (will be filtered out)
    fn map_value<T: PrimitiveType>(&self, value: &T) -> Option<f64>;

    fn map_primitive_value(&self, value: &PrimitiveValue) -> Option<f64> {
        match value {
            PrimitiveValue::Int(v) => self.map_value(v),
            PrimitiveValue::Float(v) => self.map_value(v),
            PrimitiveValue::Str(v) => self.map_value(v),
            PrimitiveValue::Bool(v) => self.map_value(v),
        }
    }

    fn map_vector_iter<'a>(&self, iter: VectorIter<'a>) -> Vec<f64> {
        let mut mapped_values: Vec<f64> = Vec::new();
        match iter {
            VectorIter::Int(iterator) => {
                for v in iterator {
                    // Preserve row alignment by pushing NaN for unmappable values
                    mapped_values.push(self.map_value(&v).unwrap_or(f64::NAN));
                }
            }
            VectorIter::Float(iterator) => {
                for v in iterator {
                    // Preserve row alignment by pushing NaN for unmappable values
                    mapped_values.push(self.map_value(&v).unwrap_or(f64::NAN));
                }
            }
            VectorIter::Str(iterator) => {
                for v in iterator {
                    // Preserve row alignment by pushing NaN for unmappable values
                    mapped_values.push(self.map_value(&v.to_string()).unwrap_or(f64::NAN));
                }
            }
            VectorIter::Bool(iterator) => {
                for v in iterator {
                    // Preserve row alignment by pushing NaN for unmappable values
                    mapped_values.push(self.map_value(&v).unwrap_or(f64::NAN));
                }
            }
        }
        mapped_values
    }

    fn map_aesthetic_value(&self, value: &AesValue, data: &dyn DataSource) -> Result<AesValue> {
        match value {
            AesValue::Column { name } => {
                let column = DataSource::get(data, name).ok_or(PlotError::MissingColumn {
                    column: name.to_string(),
                })?;
                let values = self.map_vector_iter(column.iter());
                return Ok(AesValue::vector(values, Some(name.clone())));
            }
            AesValue::Constant { value } => {
                let mapped =
                    self.map_primitive_value(value)
                        .ok_or(PlotError::AestheticDomainMismatch {
                            expected: AestheticDomain::Continuous,
                            actual: DataType::from(value),
                        })?;
                Ok(AesValue::Constant {
                    value: PrimitiveValue::Float(mapped),
                })
            }
            AesValue::Vector { values, name } => {
                let mapped_values = self.map_vector_iter(values.iter());
                Ok(AesValue::vector(mapped_values, name.clone()))
            }
        }
    }
}

pub trait ColorRangeScale: ScaleBase {
    /// Map a value from the data domain to a color.
    ///
    /// # Arguments
    /// * `value` - A value in the data domain
    ///
    /// # Returns
    /// * `Some(color)` - The corresponding color
    /// * `None` - If the value is outside the scale's domain bounds (will be filtered out)
    fn map_value<T: PrimitiveType>(&self, value: &T) -> Option<Color>;

    fn map_primitive_value(&self, value: &PrimitiveValue) -> Option<Color> {
        match value {
            PrimitiveValue::Int(v) => self.map_value(v),
            PrimitiveValue::Float(v) => self.map_value(v),
            PrimitiveValue::Str(v) => self.map_value(v),
            PrimitiveValue::Bool(v) => self.map_value(v),
        }
    }

    fn map_vector_iter<'a>(&self, iter: VectorIter<'a>) -> Vec<Color> {
        let mut colors: Vec<Color> = Vec::new();
        match iter {
            VectorIter::Int(iterator) => {
                for v in iterator {
                    if let Some(color) = self.map_value(&v) {
                        colors.push(color);
                    }
                }
            }
            VectorIter::Float(iterator) => {
                for v in iterator {
                    if let Some(color) = self.map_value(&v) {
                        colors.push(color);
                    }
                }
            }
            VectorIter::Str(iterator) => {
                for v in iterator {
                    if let Some(color) = self.map_value(&v.to_string()) {
                        colors.push(color);
                    }
                }
            }
            VectorIter::Bool(iterator) => {
                for v in iterator {
                    if let Some(color) = self.map_value(&v) {
                        colors.push(color);
                    }
                }
            }
        }
        colors
    }

    fn map_aesthetic_value(&self, value: &AesValue, data: &dyn DataSource) -> Result<AesValue> {
        match value {
            AesValue::Column { name } => {
                let column = DataSource::get(data, name).ok_or(PlotError::MissingColumn {
                    column: name.clone(),
                })?;
                let colors: Vec<Color> = self.map_vector_iter(column.iter());
                let color_values: Vec<i64> = colors.iter().map(|c| i64::from(*c)).collect();
                Ok(AesValue::vector(color_values, Some(name.clone())))
            }
            AesValue::Constant { value } => {
                let color = self.map_primitive_value(value).ok_or(
                    PlotError::InvalidPrimitiveValueMapping {
                        value: value.clone(),
                    },
                )?;
                Ok(AesValue::Constant {
                    value: PrimitiveValue::Int(i64::from(color)),
                })
            }
            AesValue::Vector {
                values,
                name: original_name,
            } => {
                let colors = self.map_vector_iter(values.iter());
                let color_values: Vec<i64> = colors.iter().map(|c| i64::from(*c)).collect();
                Ok(AesValue::vector(color_values, original_name.clone()))
            }
        }
    }
}

pub trait ShapeRangeScale: ScaleBase {
    /// Map a value from the data domain to a shape.
    ///
    /// # Arguments
    /// * `value` - A value in the data domain
    ///
    /// # Returns
    /// * `Some(shape)` - The corresponding shape
    /// * `None` - If the value is outside the scale's domain bounds (will be filtered out)
    fn map_value<T: DiscreteType>(&self, value: &T) -> Option<Shape>;

    fn map_primitive_value(&self, value: &PrimitiveValue) -> Option<Shape> {
        match value {
            PrimitiveValue::Int(v) => self.map_value(v),
            PrimitiveValue::Float(_) => None,
            PrimitiveValue::Str(v) => self.map_value(v),
            PrimitiveValue::Bool(v) => self.map_value(v),
        }
    }

    fn map_vector_iter<'a>(&self, iter: VectorIter<'a>) -> Vec<Shape> {
        let mut shapes: Vec<Shape> = Vec::new();
        match iter {
            VectorIter::Int(iterator) => {
                for v in iterator {
                    if let Some(shape) = self.map_value(&v) {
                        shapes.push(shape);
                    }
                }
            }
            VectorIter::Float(_) => {
                panic!("Shape scale cannot map float values");
            }
            VectorIter::Str(iterator) => {
                for v in iterator {
                    if let Some(shape) = self.map_value(&v.to_string()) {
                        shapes.push(shape);
                    }
                }
            }
            VectorIter::Bool(iterator) => {
                for v in iterator {
                    if let Some(shape) = self.map_value(&v) {
                        shapes.push(shape);
                    }
                }
            }
        }
        shapes
    }

    fn map_aesthetic_value(&self, value: &AesValue, data: &dyn DataSource) -> Result<AesValue> {
        match value {
            AesValue::Column { name } => {
                let column = DataSource::get(data, name).ok_or(PlotError::MissingColumn {
                    column: name.to_string(),
                })?;
                let shapes: Vec<Shape> = self.map_vector_iter(column.iter());
                let shape_values: Vec<i64> = shapes.iter().map(|s| i64::from(*s)).collect();
                Ok(AesValue::vector(shape_values, Some(name.clone())))
            }
            AesValue::Constant { value } => {
                let shape = self.map_primitive_value(value).ok_or(
                    PlotError::InvalidPrimitiveValueMapping {
                        value: value.clone(),
                    },
                )?;
                Ok(AesValue::Constant {
                    value: PrimitiveValue::Int(i64::from(shape)),
                })
            }
            AesValue::Vector {
                values,
                name,
            } => {
                let shapes = self.map_vector_iter(values.iter());
                let shape_values: Vec<i64> = shapes.iter().map(|s| i64::from(*s)).collect();
                Ok(AesValue::vector(shape_values, name.clone()))
            }
        }
    }
}

/// Scales that map to continuous [0, 1] normalized coordinates.
///
/// Used for position (x, y), size, alpha, and other continuous aesthetics.
/// These scales transform data values to normalized [0, 1] space, which
/// the rendering layer then maps to actual viewport coordinates.
pub trait ContinuousPositionalScale: ContinuousDomainScale + ContinuousRangeScale {}

pub trait DiscretePositionalScale: DiscreteDomainScale + ContinuousRangeScale {}

/// Scales that map data values to colors.
///
/// Can handle both continuous domains (gradients) and discrete domains (palettes).
/// The implementation determines whether it accepts continuous or categorical input.
pub trait ContinuousColorScale: ContinuousDomainScale + ColorRangeScale {}

pub trait DiscreteColorScale: DiscreteDomainScale + ColorRangeScale {}

/// Scales that map data values to point shapes.
///
/// Typically used for discrete/categorical data where each category
/// gets a distinct shape.
pub trait ShapeScale: DiscreteDomainScale + ShapeRangeScale {}

pub trait LineStyleRangeScale: ScaleBase {
    /// Map a value from the data domain to a linestyle.
    ///
    /// # Arguments
    /// * `value` - A value in the data domain
    ///
    /// # Returns
    /// * `Some(linestyle)` - The corresponding linestyle
    /// * `None` - If the value is outside the scale's domain bounds (will be filtered out)
    fn map_value<T: DiscreteType>(&self, value: &T) -> Option<LineStyle>;

    fn map_primitive_value(&self, value: &PrimitiveValue) -> Option<LineStyle> {
        match value {
            PrimitiveValue::Int(v) => self.map_value(v),
            PrimitiveValue::Float(_) => None,
            PrimitiveValue::Str(v) => self.map_value(v),
            PrimitiveValue::Bool(v) => self.map_value(v),
        }
    }

    fn map_vector_iter<'a>(&self, iter: VectorIter<'a>) -> Vec<LineStyle> {
        let mut linestyles: Vec<LineStyle> = Vec::new();
        match iter {
            VectorIter::Int(iterator) => {
                for v in iterator {
                    if let Some(linestyle) = self.map_value(&v) {
                        linestyles.push(linestyle);
                    }
                }
            }
            VectorIter::Float(_) => {
                panic!("LineStyle scale cannot map float values");
            }
            VectorIter::Str(iterator) => {
                for v in iterator {
                    if let Some(linestyle) = self.map_value(&v.to_string()) {
                        linestyles.push(linestyle);
                    }
                }
            }
            VectorIter::Bool(iterator) => {
                for v in iterator {
                    if let Some(linestyle) = self.map_value(&v) {
                        linestyles.push(linestyle);
                    }
                }
            }
        }
        linestyles
    }

    fn map_aesthetic_value(&self, value: &AesValue, data: &dyn DataSource) -> Result<AesValue> {
        match value {
            AesValue::Column { name } => {
                let column = DataSource::get(data, name).ok_or(PlotError::MissingColumn {
                    column: name.to_string(),
                })?;
                let linestyles: Vec<LineStyle> = self.map_vector_iter(column.iter());
                // Convert to String representation for storage
                let linestyle_strings: Vec<String> = linestyles.iter()
                    .map(|ls| format!("{:?}", ls))
                    .collect();
                Ok(AesValue::vector(linestyle_strings, Some(name.clone())))
            }
            AesValue::Constant { value } => {
                let linestyle = self.map_primitive_value(value).ok_or(
                    PlotError::InvalidPrimitiveValueMapping {
                        value: value.clone(),
                    },
                )?;
                Ok(AesValue::Constant {
                    value: PrimitiveValue::Str(format!("{:?}", linestyle)),
                })
            }
            AesValue::Vector {
                values,
                name,
            } => {
                let linestyles = self.map_vector_iter(values.iter());
                let linestyle_strings: Vec<String> = linestyles.iter()
                    .map(|ls| format!("{:?}", ls))
                    .collect();
                Ok(AesValue::vector(linestyle_strings, name.clone()))
            }
        }
    }
}

/// Scales that map data values to line styles.
///
/// Typically used for discrete/categorical data where each category
/// gets a distinct line pattern.
pub trait LineStyleScale: DiscreteDomainScale + LineStyleRangeScale {}
