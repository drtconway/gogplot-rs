use crate::aesthetics::AesValue;
use crate::data::{DataSource, DiscreteType, PrimitiveType, PrimitiveValue, VectorIter};
use crate::scale::{ContinuousScaleTrainer, DiscreteScaleTrainer};
use crate::theme::Color;

use crate::utils::data::{visit_c, visit_d};
use crate::utils::dataframe::{DataFrame, FloatVec, IntVec};
use crate::utils::set::DiscreteSet;
use crate::visuals::Shape;

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
            if let Some((min_existing, max_existing)) = self.domain() {
                let min_value = min_value.min(min_existing);
                let max_value = max_value.max(max_existing);
                self.set_domain((min_value, max_value));
            } else {
                self.set_domain((min_value, max_value));
            }
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

    fn map_aesthetic_value(&self, value: &AesValue, data: &DataFrame, new_data: &mut DataFrame) -> Option<AesValue> {
        match value {
            AesValue::Column { name, hint, original_name } => {
                let column = DataSource::get(&data, name)?;
                let values: Vec<f64> = column.iter().filter_map(|v| {
                    match v {
                        PrimitiveValue::Int(v) => self.map_value(v),
                        PrimitiveValue::Float(v) => self.map_value(v),
                        PrimitiveValue::Str(v) => self.map_value(v),
                        PrimitiveValue::Bool(v) => self.map_value(v),
                    }
                }).collect();

                new_data.add_column(name.clone(), FloatVec::from(values));
                Some(AesValue::Column {
                    name: name.clone(),
                    hint: hint.clone(),
                    original_name: original_name.clone(),
                })
            },
            AesValue::Constant { value, hint } =>  {
                let mapped = self.map_primitive_value(value)?;
                Some(AesValue::Constant {
                    value: PrimitiveValue::Float(mapped),
                    hint: hint.clone(),
                })
            },
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

    fn map_aesthetic_value(&self, value: &AesValue, data: &DataFrame, new_data: &mut DataFrame) -> Option<AesValue> {
        match value {
            AesValue::Column { name, hint, original_name } => {
                let column = DataSource::get(&data, name)?;
                let values: Vec<i64> = column.iter().filter_map(|v| {
                    let color = match v {
                        PrimitiveValue::Int(v) => self.map_value(v),
                        PrimitiveValue::Float(v) => self.map_value(v),
                        PrimitiveValue::Str(v) => self.map_value(v),
                        PrimitiveValue::Bool(v) => self.map_value(v),
                    };
                    color.map(|c| i64::from(c))
                }).collect();

                new_data.add_column(name.clone(), IntVec::from(values));
                Some(AesValue::Column {
                    name: name.clone(),
                    hint: hint.clone(),
                    original_name: original_name.clone(),
                })
            },
            AesValue::Constant { value, hint } =>  {
                let mapped = self.map_primitive_value(value)?;
                Some(AesValue::Constant {
                    value: PrimitiveValue::Float(mapped),
                    hint: hint.clone(),
                })
            },
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
    fn map_value<T: PrimitiveType>(&self, value: &T) -> Option<Shape>;
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
