use crate::data::{DataSource, DiscreteValue, GenericVector, PrimitiveValue, VectorIter, VectorValue};
use crate::error::PlotError;
use crate::scale::ScaleType;
use crate::theme::Color;
use crate::visuals::Shape;
use std::collections::HashMap;
use std::sync::Arc;

pub mod builder;
pub mod values;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AestheticDomain {
    Continuous,
    Discrete,
}

pub enum AestheticPropertyType {
    Int,
    Float,
    String,
    Color,
    Shape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AestheticProperty {
    Color,
    Fill,
    Size,
    Shape,
    Alpha,
    Linetype,
    X,
    Y,
    XMin,
    XMax,
    YMin,
    YMax,
    XBegin,
    XEnd,
    YBegin,
    YEnd,
    XIntercept,
    YIntercept,
    Lower,
    Middle,
    Upper,
    XOffset,
    YOffset,
    Width,
    Height,
    Label,
}

impl AestheticProperty {
    pub fn property_type(&self) -> AestheticPropertyType {
        match self {
            AestheticProperty::Color => AestheticPropertyType::Int,
            AestheticProperty::Fill => AestheticPropertyType::Int,
            AestheticProperty::Size => AestheticPropertyType::Float,
            AestheticProperty::Shape => AestheticPropertyType::Int,
            AestheticProperty::Alpha => AestheticPropertyType::Float,
            AestheticProperty::Linetype => AestheticPropertyType::String,
            AestheticProperty::X => AestheticPropertyType::Float,
            AestheticProperty::Y => AestheticPropertyType::Float,
            AestheticProperty::XMin => AestheticPropertyType::Float,
            AestheticProperty::XMax => AestheticPropertyType::Float,
            AestheticProperty::YMin => AestheticPropertyType::Float,
            AestheticProperty::YMax => AestheticPropertyType::Float,
            AestheticProperty::XBegin => AestheticPropertyType::Float,
            AestheticProperty::XEnd => AestheticPropertyType::Float,
            AestheticProperty::YBegin => AestheticPropertyType::Float,
            AestheticProperty::YEnd => AestheticPropertyType::Float,
            AestheticProperty::XIntercept => AestheticPropertyType::Float,
            AestheticProperty::YIntercept => AestheticPropertyType::Float,
            AestheticProperty::Lower => AestheticPropertyType::Float,
            AestheticProperty::Middle => AestheticPropertyType::Float,
            AestheticProperty::Upper => AestheticPropertyType::Float,
            AestheticProperty::XOffset => AestheticPropertyType::Float,
            AestheticProperty::YOffset => AestheticPropertyType::Float,
            AestheticProperty::Width => AestheticPropertyType::Float,
            AestheticProperty::Height => AestheticPropertyType::Float,
            AestheticProperty::Label => AestheticPropertyType::String,
        }
    }

    pub fn aesthetics(&self) -> &[Aesthetic] {
        use AestheticDomain::*;
        match self {
            AestheticProperty::Color => &[Aesthetic::Color(Continuous), Aesthetic::Color(Discrete)],
            AestheticProperty::Fill => &[Aesthetic::Fill(Continuous), Aesthetic::Fill(Discrete)],
            AestheticProperty::Size => &[Aesthetic::Size(Continuous), Aesthetic::Size(Discrete)],
            AestheticProperty::Shape => &[Aesthetic::Shape],
            AestheticProperty::Alpha => &[Aesthetic::Alpha(Continuous), Aesthetic::Alpha(Discrete)],
            AestheticProperty::Linetype => &[Aesthetic::Linetype],
            AestheticProperty::X => &[Aesthetic::X(Continuous), Aesthetic::X(Discrete)],
            AestheticProperty::Y => &[Aesthetic::Y(Continuous), Aesthetic::Y(Discrete)],
            AestheticProperty::XMin => &[Aesthetic::Xmin(Continuous), Aesthetic::Xmin(Discrete)],
            AestheticProperty::XMax => &[Aesthetic::Xmax(Continuous), Aesthetic::Xmax(Discrete)],
            AestheticProperty::YMin => &[Aesthetic::Ymin(Continuous), Aesthetic::Ymin(Discrete)],
            AestheticProperty::YMax => &[Aesthetic::Ymax(Continuous), Aesthetic::Ymax(Discrete)],
            AestheticProperty::XBegin => &[Aesthetic::XBegin],
            AestheticProperty::XEnd => &[Aesthetic::XEnd],
            AestheticProperty::YBegin => &[Aesthetic::YBegin],
            AestheticProperty::YEnd => &[Aesthetic::YEnd],
            AestheticProperty::XIntercept => &[Aesthetic::XIntercept],
            AestheticProperty::YIntercept => &[Aesthetic::YIntercept],
            AestheticProperty::Lower => &[Aesthetic::Lower],
            AestheticProperty::Middle => &[Aesthetic::Middle],
            AestheticProperty::Upper => &[Aesthetic::Upper],
            AestheticProperty::XOffset => &[Aesthetic::XOffset],
            AestheticProperty::YOffset => &[Aesthetic::YOffset],
            AestheticProperty::Width => &[Aesthetic::Width],
            AestheticProperty::Height => &[Aesthetic::Height],
            AestheticProperty::Label => &[Aesthetic::Label],
        }
    }

    pub fn perpendicular(&self) -> AestheticProperty {
        match self {
            AestheticProperty::X => AestheticProperty::Y,
            AestheticProperty::Y => AestheticProperty::X,
            AestheticProperty::XMin => AestheticProperty::YMin,
            AestheticProperty::YMin => AestheticProperty::XMin,
            AestheticProperty::XMax => AestheticProperty::YMax,
            AestheticProperty::YMax => AestheticProperty::XMax,
            AestheticProperty::XBegin => AestheticProperty::YBegin,
            AestheticProperty::YBegin => AestheticProperty::XBegin,
            AestheticProperty::XEnd => AestheticProperty::YEnd,
            AestheticProperty::YEnd => AestheticProperty::XEnd,
            AestheticProperty::XIntercept => AestheticProperty::YIntercept,
            AestheticProperty::YIntercept => AestheticProperty::XIntercept,
            AestheticProperty::XOffset => AestheticProperty::YOffset,
            AestheticProperty::YOffset => AestheticProperty::XOffset,
            AestheticProperty::Width => AestheticProperty::Height,
            AestheticProperty::Height => AestheticProperty::Width,
            other => *other,
        }
    }

    pub fn to_str(&self) -> &'static str {
        match self {
            AestheticProperty::Color => "color",
            AestheticProperty::Fill => "fill",
            AestheticProperty::Size => "size",
            AestheticProperty::Shape => "shape",
            AestheticProperty::Alpha => "alpha",
            AestheticProperty::Linetype => "linetype",
            AestheticProperty::X => "x",
            AestheticProperty::Y => "y",
            AestheticProperty::XMin => "xmin",
            AestheticProperty::XMax => "xmax",
            AestheticProperty::YMin => "ymin",
            AestheticProperty::YMax => "ymax",
            AestheticProperty::XBegin => "xbegin",
            AestheticProperty::XEnd => "xend",
            AestheticProperty::YBegin => "ybegin",
            AestheticProperty::YEnd => "yend",
            AestheticProperty::XIntercept => "xintercept",
            AestheticProperty::YIntercept => "yintercept",
            AestheticProperty::Lower => "lower",
            AestheticProperty::Middle => "middle",
            AestheticProperty::Upper => "upper",
            AestheticProperty::XOffset => "xoffset",
            AestheticProperty::YOffset => "yoffset",
            AestheticProperty::Width => "width",
            AestheticProperty::Height => "height",
            AestheticProperty::Label => "label",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PrimaryAesthetic {
    X(AestheticDomain),
    Y(AestheticDomain),
    Color(AestheticDomain),
    Fill(AestheticDomain),
    Size(AestheticDomain),
    Alpha(AestheticDomain),
    Shape,
    Linetype,
}

impl TryFrom<Aesthetic> for PrimaryAesthetic {
    type Error = PlotError;

    fn try_from(aes: Aesthetic) -> Result<Self, Self::Error> {
        match aes {
            Aesthetic::X(kind) => Ok(PrimaryAesthetic::X(kind)),
            Aesthetic::Y(kind) => Ok(PrimaryAesthetic::Y(kind)),
            Aesthetic::Color(kind) => Ok(PrimaryAesthetic::Color(kind)),
            Aesthetic::Fill(kind) => Ok(PrimaryAesthetic::Fill(kind)),
            Aesthetic::Shape => Ok(PrimaryAesthetic::Shape),
            Aesthetic::Size(kind) => Ok(PrimaryAesthetic::Size(kind)),
            Aesthetic::Alpha(kind) => Ok(PrimaryAesthetic::Alpha(kind)),
            Aesthetic::Linetype => Ok(PrimaryAesthetic::Linetype),
            _ => Err(PlotError::InvalidAestheticConversion { from: aes }),
        }
    }
}

// Supported aesthetics
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Aesthetic {
    X(AestheticDomain),
    Y(AestheticDomain),
    Xmin(AestheticDomain),
    Xmax(AestheticDomain),
    Ymin(AestheticDomain),
    Ymax(AestheticDomain),
    Lower,  // Q1 (first quartile) for boxplots
    Middle, // Median for boxplots
    Upper,  // Q3 (third quartile) for boxplots
    Color(AestheticDomain),
    Fill(AestheticDomain),
    Alpha(AestheticDomain),
    Size(AestheticDomain),
    Shape,
    Linetype,
    Group,
    XBegin,
    XEnd,
    YBegin,
    YEnd,
    XIntercept,
    YIntercept,
    XOffset,
    YOffset,
    Width,
    Height,
    Label,
}

impl Aesthetic {
    /// Returns true if this aesthetic creates groups when mapped to categorical data.
    /// Grouping aesthetics are used to split data into subsets for operations like
    /// binning, smoothing, or statistical transformations.
    pub fn is_grouping(&self) -> bool {
        matches!(
            self,
            Aesthetic::Color(AestheticDomain::Discrete)
                | Aesthetic::Fill(AestheticDomain::Discrete)
                | Aesthetic::Shape
                | Aesthetic::Linetype
                | Aesthetic::Group
        )
    }

    /// Returns true if this aesthetic relates to the x-axis position.
    /// Used for training x-scales on all relevant data.
    pub fn is_x_like(&self) -> bool {
        matches!(
            self,
            Aesthetic::X(_)
                | Aesthetic::XBegin
                | Aesthetic::XEnd
                | Aesthetic::Xmin(_)
                | Aesthetic::Xmax(_)
                | Aesthetic::XIntercept
        )
    }

    /// Returns true if this aesthetic relates to the y-axis position.
    /// Used for training y-scales on all relevant data.
    pub fn is_y_like(&self) -> bool {
        matches!(
            self,
            Aesthetic::Y(_)
                | Aesthetic::YBegin
                | Aesthetic::YEnd
                | Aesthetic::Ymin(_)
                | Aesthetic::Ymax(_)
                | Aesthetic::YIntercept
                | Aesthetic::Lower
                | Aesthetic::Middle
                | Aesthetic::Upper
        )
    }

    /// Returns the AestheticDomain for the aesthetic,
    /// which may be explicit or implied.
    pub fn domain(&self) -> AestheticDomain {
        match self {
            Aesthetic::X(kind)
            | Aesthetic::Y(kind)
            | Aesthetic::Xmin(kind)
            | Aesthetic::Xmax(kind)
            | Aesthetic::Ymin(kind)
            | Aesthetic::Ymax(kind)
            | Aesthetic::Color(kind)
            | Aesthetic::Fill(kind)
            | Aesthetic::Alpha(kind)
            | Aesthetic::Size(kind) => *kind,
            Aesthetic::Shape | Aesthetic::Linetype | Aesthetic::Group => AestheticDomain::Discrete,
            Aesthetic::XBegin
            | Aesthetic::XEnd
            | Aesthetic::XIntercept
            | Aesthetic::YBegin
            | Aesthetic::YEnd
            | Aesthetic::YIntercept
            | Aesthetic::XOffset
            | Aesthetic::YOffset
            | Aesthetic::Width
            | Aesthetic::Height
            | Aesthetic::Lower
            | Aesthetic::Middle
            | Aesthetic::Upper
            | Aesthetic::Label => AestheticDomain::Continuous,
        }
    }

    /// A printable name for the aesthetic.
    pub fn to_str(&self) -> &'static str {
        match self {
            Aesthetic::X(_) => "x",
            Aesthetic::Y(_) => "y",
            Aesthetic::Xmin(_) => "xmin",
            Aesthetic::Xmax(_) => "xmax",
            Aesthetic::Ymin(_) => "ymin",
            Aesthetic::Ymax(_) => "ymax",
            Aesthetic::Lower => "lower",
            Aesthetic::Middle => "middle",
            Aesthetic::Upper => "upper",
            Aesthetic::Color(_) => "color",
            Aesthetic::Fill(_) => "fill",
            Aesthetic::Alpha(_) => "alpha",
            Aesthetic::Size(_) => "size",
            Aesthetic::Shape => "shape",
            Aesthetic::Linetype => "linetype",
            Aesthetic::Group => "group",
            Aesthetic::XBegin => "xbegin",
            Aesthetic::XEnd => "xend",
            Aesthetic::YBegin => "ybegin",
            Aesthetic::YEnd => "yend",
            Aesthetic::XIntercept => "xintercept",
            Aesthetic::YIntercept => "yintercept",
            Aesthetic::XOffset => "xoffset",
            Aesthetic::YOffset => "yoffset",
            Aesthetic::Width => "width",
            Aesthetic::Height => "height",
            Aesthetic::Label => "label",
        }
    }

    pub fn is_continuous(&self) -> bool {
        match self.domain() {
            AestheticDomain::Continuous => true,
            AestheticDomain::Discrete => false,
        }
    }

    pub fn is_discrete(&self) -> bool {
        match self.domain() {
            AestheticDomain::Continuous => false,
            AestheticDomain::Discrete => true,
        }
    }

    /// Extract the aesthetic property (without domain information)
    /// Returns None for positional aesthetics that don't map to properties
    pub fn to_property(&self) -> Option<AestheticProperty> {
        match self {
            Aesthetic::Color(_) => Some(AestheticProperty::Color),
            Aesthetic::Fill(_) => Some(AestheticProperty::Fill),
            Aesthetic::Size(_) => Some(AestheticProperty::Size),
            Aesthetic::Alpha(_) => Some(AestheticProperty::Alpha),
            Aesthetic::Shape => Some(AestheticProperty::Shape),
            Aesthetic::Linetype => Some(AestheticProperty::Linetype),
            Aesthetic::X(_) => Some(AestheticProperty::X),
            Aesthetic::Y(_) => Some(AestheticProperty::Y),
            Aesthetic::Lower => Some(AestheticProperty::Lower),
            Aesthetic::Middle => Some(AestheticProperty::Middle),
            Aesthetic::Upper => Some(AestheticProperty::Upper),
            Aesthetic::Xmin(_) => Some(AestheticProperty::XMin),
            Aesthetic::Xmax(_) => Some(AestheticProperty::XMax),
            Aesthetic::Ymin(_) => Some(AestheticProperty::YMin),
            Aesthetic::Ymax(_) => Some(AestheticProperty::YMax),
            Aesthetic::XBegin => Some(AestheticProperty::XBegin),
            Aesthetic::XEnd => Some(AestheticProperty::XEnd),
            Aesthetic::YBegin => Some(AestheticProperty::YBegin),
            Aesthetic::YEnd => Some(AestheticProperty::YEnd),
            Aesthetic::XIntercept => Some(AestheticProperty::XIntercept),
            Aesthetic::YIntercept => Some(AestheticProperty::YIntercept),
            Aesthetic::XOffset => Some(AestheticProperty::XOffset),
            Aesthetic::YOffset => Some(AestheticProperty::YOffset),
            Aesthetic::Width => Some(AestheticProperty::Width),
            Aesthetic::Height => Some(AestheticProperty::Height),
            Aesthetic::Label => Some(AestheticProperty::Label),
            // Group doesn't have a corresponding property
            Aesthetic::Group => None,
        }
    }
}

impl From<(AestheticProperty, AestheticDomain)> for Aesthetic {
    fn from(value: (AestheticProperty, AestheticDomain)) -> Self {
        match value.0 {
            AestheticProperty::X => Aesthetic::X(value.1),
            AestheticProperty::Y => Aesthetic::Y(value.1),
            AestheticProperty::XMin => Aesthetic::Xmin(value.1),
            AestheticProperty::XMax => Aesthetic::Xmax(value.1),
            AestheticProperty::YMin => Aesthetic::Ymin(value.1),
            AestheticProperty::YMax => Aesthetic::Ymax(value.1),
            AestheticProperty::XBegin => Aesthetic::XBegin,
            AestheticProperty::XEnd => Aesthetic::XEnd,
            AestheticProperty::YBegin => Aesthetic::YBegin,
            AestheticProperty::YEnd => Aesthetic::YEnd,
            AestheticProperty::XIntercept => Aesthetic::XIntercept,
            AestheticProperty::YIntercept => Aesthetic::YIntercept,
            AestheticProperty::Lower => Aesthetic::Lower,
            AestheticProperty::Middle => Aesthetic::Middle,
            AestheticProperty::Upper => Aesthetic::Upper,
            AestheticProperty::XOffset => Aesthetic::XOffset,
            AestheticProperty::YOffset => Aesthetic::YOffset,
            AestheticProperty::Width => Aesthetic::Width,
            AestheticProperty::Height => Aesthetic::Height,
            AestheticProperty::Color => Aesthetic::Color(value.1),
            AestheticProperty::Fill => Aesthetic::Fill(value.1),
            AestheticProperty::Size => Aesthetic::Size(value.1),
            AestheticProperty::Alpha => Aesthetic::Alpha(value.1),
            AestheticProperty::Shape => Aesthetic::Shape,
            AestheticProperty::Linetype => Aesthetic::Linetype,
            AestheticProperty::Label => Aesthetic::Label,
        }
    }
}

// AesValue is a type that can be mapped to an aesthetic
// It can be a column name, a constant value, or a computed value
// Each can optionally carry a hint about whether it should be treated as continuous or categorical
#[derive(Clone)]
pub enum AesValue {
    /// Column name from data with optional scale type hint
    Column {
        name: String,
        hint: Option<ScaleType>,
        /// Original column name before any disambiguation (e.g., "x" instead of "x_fill_1")
        /// Used for legend titles and other user-facing labels
        original_name: Option<String>,
    },
    /// Fixed value with optional scale type hint
    Constant {
        value: PrimitiveValue,
        hint: Option<ScaleType>,
    },
    /// Materialized vector of values (result of scale/stat/position transformation)
    Vector {
        values: Arc<VectorValue>,
        /// Original column name before transformation (for legend titles)
        original_name: Option<String>,
    },
}

impl std::fmt::Debug for AesValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AesValue::Column {
                name,
                hint,
                original_name,
            } => f
                .debug_struct("Column")
                .field("name", name)
                .field("hint", hint)
                .field("original_name", original_name)
                .finish(),
            AesValue::Constant { value, hint } => f
                .debug_struct("Constant")
                .field("value", value)
                .field("hint", hint)
                .finish(),
            AesValue::Vector {
                values: _,
                original_name,
            } => f
                .debug_struct("Vector")
                .field("original_name", original_name)
                .field("values", &"<GenericVector>")
                .finish(),
        }
    }
}

impl PartialEq for AesValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (
                AesValue::Column {
                    name: n1,
                    hint: h1,
                    original_name: o1,
                },
                AesValue::Column {
                    name: n2,
                    hint: h2,
                    original_name: o2,
                },
            ) => n1 == n2 && h1 == h2 && o1 == o2,
            (
                AesValue::Constant {
                    value: v1,
                    hint: h1,
                },
                AesValue::Constant {
                    value: v2,
                    hint: h2,
                },
            ) => v1 == v2 && h1 == h2,
            (
                AesValue::Vector {
                    original_name: o1, ..
                },
                AesValue::Vector {
                    original_name: o2, ..
                },
            ) => {
                // Note: We can't compare GenericVector values, so just compare metadata
                o1 == o2
            }
            _ => false,
        }
    }
}

impl AesValue {
    /// Create a Column variant from a string-like value with no type hint
    pub fn column(name: impl Into<String>) -> Self {
        AesValue::Column {
            name: name.into(),
            hint: None,
            original_name: None,
        }
    }

    /// Create a Column variant that should be treated as continuous
    pub fn continuous_column(name: impl Into<String>) -> Self {
        AesValue::Column {
            name: name.into(),
            hint: Some(ScaleType::Continuous),
            original_name: None,
        }
    }

    /// Create a Column variant that should be treated as categorical
    /// Use this when you want to treat a numeric column as categorical
    pub fn categorical_column(name: impl Into<String>) -> Self {
        AesValue::Column {
            name: name.into(),
            hint: Some(ScaleType::Categorical),
            original_name: None,
        }
    }

    /// Legacy alias for categorical_column
    pub fn categorical(name: impl Into<String>) -> Self {
        Self::categorical_column(name)
    }

    /// Create a Constant variant with no type hint
    pub fn constant(value: impl Into<PrimitiveValue>) -> Self {
        AesValue::Constant {
            value: value.into(),
            hint: None,
        }
    }

    /// Create a Constant variant that should be treated as continuous
    pub fn continuous_constant(value: impl Into<PrimitiveValue>) -> Self {
        AesValue::Constant {
            value: value.into(),
            hint: Some(ScaleType::Continuous),
        }
    }

    /// Create a Constant variant that should be treated as categorical
    pub fn categorical_constant(value: impl Into<PrimitiveValue>) -> Self {
        AesValue::Constant {
            value: value.into(),
            hint: Some(ScaleType::Categorical),
        }
    }

    /// Create a Vector variant from materialized values
    pub fn vector(values: impl Into<VectorValue>, original_name: Option<String>) -> Self {
        AesValue::Vector {
            values: Arc::new(values.into()),
            original_name,
        }
    }

    /// Extract the column name from Column variants
    /// Returns None for Constant values
    pub fn as_column_name(&self) -> Option<&str> {
        match self {
            AesValue::Column { name, .. } => Some(name.as_str()),
            AesValue::Constant { .. } => None,
            AesValue::Vector { .. } => None,
        }
    }

    /// Extract the original column name (before disambiguation) from Column variants
    /// Falls back to the current name if no original name was stored
    /// Returns None for Constant values
    pub fn as_original_column_name(&self) -> Option<&str> {
        match self {
            AesValue::Column {
                name,
                original_name,
                ..
            } => Some(original_name.as_ref().unwrap_or(name).as_str()),
            AesValue::Constant { .. } => None,
            AesValue::Vector { original_name, .. } => original_name.as_ref().map(|s| s.as_str()),
        }
    }

    /// Get the constant value if this is a Constant variant
    pub fn as_constant(&self) -> Option<&PrimitiveValue> {
        match self {
            AesValue::Constant { value, .. } => Some(value),
            _ => None,
        }
    }

    /// Get the vector if this is a Vector variant
    pub fn as_vector(&self) -> Option<&dyn GenericVector> {
        match self {
            AesValue::Vector { values, .. } => Some(values.as_ref()),
            _ => None,
        }
    }

    fn as_vector_iter<'a>(&'a self, data: &'a dyn DataSource) -> Option<VectorIter<'a>> {
        match self {
            AesValue::Column { name, .. } => {
                let column = data.get(name.as_str())?;
                Some(column.iter())
            }
            AesValue::Constant { value, .. } => {
                let n = data.len();
                match value {
                    PrimitiveValue::Int(i) => {
                        Some(VectorIter::Int(Box::new(std::iter::repeat(*i).take(n))))
                    }
                    PrimitiveValue::Float(f) => {
                        Some(VectorIter::Float(Box::new(std::iter::repeat(*f).take(n))))
                    }
                    PrimitiveValue::Str(s) => Some(VectorIter::Str(Box::new(
                        std::iter::repeat(s.as_str()).take(n),
                    ))),
                    PrimitiveValue::Bool(b) => {
                        Some(VectorIter::Bool(Box::new(std::iter::repeat(*b).take(n))))
                    }
                }
            }
            AesValue::Vector { values, .. } => Some(values.iter()),
        }
    }

    fn as_int_vector_iter<'a>(
        &'a self,
        data: &'a dyn DataSource,
    ) -> Option<Box<dyn Iterator<Item = i64> + 'a>> {
        match self.as_vector_iter(data)? {
            VectorIter::Int(iter) => Some(iter),
            _ => None,
        }
    }

    fn as_float_vector_iter<'a>(
        &'a self,
        data: &'a dyn DataSource,
    ) -> Option<Box<dyn Iterator<Item = f64> + 'a>> {
        match self.as_vector_iter(data)? {
            VectorIter::Float(iter) => Some(iter),
            VectorIter::Int(iter) => Some(Box::new(iter.map(|v| v as f64))),
            _ => None,
        }
    }

    fn as_str_vector_iter<'a>(
        &'a self,
        data: &'a dyn DataSource,
    ) -> Option<Box<dyn Iterator<Item = String> + 'a>> {
        match self.as_vector_iter(data)? {
            VectorIter::Str(iter) => Some(Box::new(iter.map(|s| s.to_string()))),
            _ => None,
        }
    }

    fn as_discrete_vector_iter<'a>(
        &'a self,
        data: &'a dyn DataSource,
    ) -> Option<Box<dyn Iterator<Item = DiscreteValue> + 'a>> {
        match self.as_vector_iter(data)? {
            VectorIter::Int(iter) => Some(Box::new(iter.map(DiscreteValue::Int))),
            VectorIter::Str(iter) => {
                Some(Box::new(iter.map(|s| DiscreteValue::Str(s.to_string()))))
            }
            VectorIter::Bool(iter) => Some(Box::new(iter.map(DiscreteValue::Bool))),
            _ => None,
        }
    }

    fn as_color_vector_iter<'a>(
        &'a self,
        data: &'a dyn DataSource,
    ) -> Option<Box<dyn Iterator<Item = Color> + 'a>> {
        match self.as_int_vector_iter(data)? {
            iter => Some(Box::new(iter.map(|i| Color::from(i)))),
        }
    }

    fn as_shape_vector_iter<'a>(
        &'a self,
        data: &'a dyn DataSource,
    ) -> Option<Box<dyn Iterator<Item = Shape> + 'a>> {
        match self.as_int_vector_iter(data)? {
            iter => Some(Box::new(iter.map(|i| Shape::from(i)))),
        }
    }
}

pub mod constant;

// The mapping structure
#[derive(Clone, Debug)]
pub struct AesMap {
    map: HashMap<Aesthetic, AesValue>,
}

impl Default for AesMap {
    fn default() -> Self {
        Self::new()
    }
}

impl AesMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn set(&mut self, aes: Aesthetic, value: AesValue) {
        self.map.insert(aes, value);
    }

    /// Convenience method to set an aesthetic to a column mapping
    pub fn set_to_column(&mut self, aes: Aesthetic, column: impl Into<String>) {
        self.set(aes, AesValue::column(column));
    }

    pub fn get(&self, aes: &Aesthetic) -> Option<&AesValue> {
        self.map.get(aes)
    }

    pub fn remove(&mut self, aes: &Aesthetic) {
        self.map.remove(aes);
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Aesthetic, &AesValue)> {
        self.map.iter()
    }

    pub fn aesthetics(&self) -> impl Iterator<Item = &Aesthetic> {
        self.map.keys()
    }

    pub fn contains(&self, aes: Aesthetic) -> bool {
        self.map.contains_key(&aes)
    }

    /// Merge another AesMap into this one
    /// Values from `other` override values in `self`
    pub fn merge(&mut self, other: &AesMap) {
        for (aes, value) in other.iter() {
            self.set(*aes, value.clone());
        }
    }

    /// Get the label/title for an aesthetic based on its mapped column name
    /// Returns the original column name if available, otherwise the current name
    /// Returns None if the aesthetic is not mapped or is a constant
    pub fn get_label(&self, property: AestheticProperty) -> Option<String> {
        // Find any aesthetic with this property
        for (aes, value) in self.iter() {
            let matches = match (property, aes) {
                (AestheticProperty::X, Aesthetic::X(_)) => true,
                (AestheticProperty::Y, Aesthetic::Y(_)) => true,
                (AestheticProperty::Color, Aesthetic::Color(_)) => true,
                (AestheticProperty::Fill, Aesthetic::Fill(_)) => true,
                (AestheticProperty::Size, Aesthetic::Size(_)) => true,
                (AestheticProperty::Alpha, Aesthetic::Alpha(_)) => true,
                (AestheticProperty::Shape, Aesthetic::Shape) => true,
                (AestheticProperty::Linetype, Aesthetic::Linetype) => true,
                _ => false,
            };

            if matches {
                return value.as_original_column_name().map(|s| s.to_string());
            }
        }
        None
    }

    // Convenience methods for column mappings
    pub fn x(&mut self, column: impl Into<String>, kind: AestheticDomain) {
        self.set_to_column(Aesthetic::X(kind), column);
    }
    pub fn y(&mut self, column: impl Into<String>, kind: AestheticDomain) {
        self.set_to_column(Aesthetic::Y(kind), column);
    }
    pub fn alpha(&mut self, column: impl Into<String>, kind: AestheticDomain) {
        self.set_to_column(Aesthetic::Alpha(kind), column);
    }
    pub fn size(&mut self, column: impl Into<String>, kind: AestheticDomain) {
        self.set_to_column(Aesthetic::Size(kind), column);
    }
    pub fn shape(&mut self, column: impl Into<String>) {
        self.set_to_column(Aesthetic::Shape, column);
    }
    pub fn group(&mut self, column: impl Into<String>) {
        self.set_to_column(Aesthetic::Group, column);
    }
    pub fn linetype(&mut self, column: impl Into<String>) {
        self.set_to_column(Aesthetic::Linetype, column);
    }
    pub fn yintercept(&mut self, column: impl Into<String>) {
        self.set_to_column(Aesthetic::YIntercept, column);
    }
    pub fn xintercept(&mut self, column: impl Into<String>) {
        self.set_to_column(Aesthetic::XIntercept, column);
    }
    pub fn label(&mut self, column: impl Into<String>) {
        self.set_to_column(Aesthetic::Label, column);
    }
    pub fn ymin(&mut self, column: impl Into<String>, kind: AestheticDomain) {
        self.set_to_column(Aesthetic::Ymin(kind), column);
    }
    pub fn ymax(&mut self, column: impl Into<String>, kind: AestheticDomain) {
        self.set_to_column(Aesthetic::Ymax(kind), column);
    }

    // Convenience methods for categorical column mappings
    // Use these when you want to treat a numeric column as categorical
    pub fn x_categorical(&mut self, column: impl Into<String>) {
        self.set(
            Aesthetic::X(AestheticDomain::Discrete),
            AesValue::categorical_column(column),
        );
    }
    pub fn y_categorical(&mut self, column: impl Into<String>) {
        self.set(
            Aesthetic::Y(AestheticDomain::Discrete),
            AesValue::categorical_column(column),
        );
    }
    pub fn shape_categorical(&mut self, column: impl Into<String>) {
        self.set(Aesthetic::Shape, AesValue::categorical_column(column));
    }
    pub fn group_categorical(&mut self, column: impl Into<String>) {
        self.set(Aesthetic::Group, AesValue::categorical_column(column));
    }
    pub fn linetype_categorical(&mut self, column: impl Into<String>) {
        self.set(Aesthetic::Linetype, AesValue::categorical_column(column));
    }

    // Convenience methods for continuous column mappings
    // Use these when you want to explicitly mark a column as continuous
    pub fn x_continuous(&mut self, column: impl Into<String>) {
        self.set(
            Aesthetic::X(AestheticDomain::Continuous),
            AesValue::continuous_column(column),
        );
    }
    pub fn y_continuous(&mut self, column: impl Into<String>) {
        self.set(
            Aesthetic::Y(AestheticDomain::Continuous),
            AesValue::continuous_column(column),
        );
    }
    pub fn size_continuous(&mut self, column: impl Into<String>) {
        self.set(
            Aesthetic::Size(AestheticDomain::Continuous),
            AesValue::continuous_column(column),
        );
    }
    pub fn alpha_continuous(&mut self, column: impl Into<String>) {
        self.set(
            Aesthetic::Alpha(AestheticDomain::Continuous),
            AesValue::continuous_column(column),
        );
    }

    pub fn const_alpha(&mut self, alpha: f64) {
        self.set(
            Aesthetic::Alpha(AestheticDomain::Continuous),
            AesValue::constant(PrimitiveValue::Float(alpha)),
        );
    }

    pub fn const_size(&mut self, size: f64) {
        self.set(
            Aesthetic::Size(AestheticDomain::Continuous),
            AesValue::constant(PrimitiveValue::Float(size)),
        );
    }

    pub fn const_shape(&mut self, shape: i64) {
        self.set(
            Aesthetic::Shape,
            AesValue::constant(PrimitiveValue::Int(shape)),
        );
    }

    pub fn const_linetype(&mut self, pattern: impl Into<String>) {
        self.set(
            Aesthetic::Linetype,
            AesValue::constant(PrimitiveValue::Str(pattern.into())),
        );
    }

    pub fn yintercept_const(&mut self, value: f64) {
        self.set(
            Aesthetic::YIntercept,
            AesValue::constant(PrimitiveValue::Float(value)),
        );
    }

    pub fn xintercept_const(&mut self, value: f64) {
        self.set(
            Aesthetic::XIntercept,
            AesValue::constant(PrimitiveValue::Float(value)),
        );
    }

    pub fn get_vector_iter<'a>(
        &'a self,
        aes: &'_ Aesthetic,
        data: &'a dyn DataSource,
    ) -> Option<VectorIter<'a>> {
        self.get(aes)?.as_vector_iter(data)
    }

    pub fn get_iter_float<'a>(
        &'a self,
        aes: &Aesthetic,
        data: &'a dyn DataSource,
    ) -> Option<Box<dyn Iterator<Item = f64> + 'a>> {
        self.get(aes)?.as_float_vector_iter(data)
    }

    pub fn get_iter_int<'a>(
        &'a self,
        aes: &Aesthetic,
        data: &'a dyn DataSource,
    ) -> Option<Box<dyn Iterator<Item = i64> + 'a>> {
        self.get(aes)?.as_int_vector_iter(data)
    }

    pub fn get_iter_string<'a>(
        &'a self,
        aes: &Aesthetic,
        data: &'a dyn DataSource,
    ) -> Option<Box<dyn Iterator<Item = String> + 'a>> {
        self.get(aes)?.as_str_vector_iter(data)
    }

    pub fn get_iter_discrete<'a>(
        &'a self,
        aes: &Aesthetic,
        data: &'a dyn DataSource,
    ) -> Option<Box<dyn Iterator<Item = DiscreteValue> + 'a>> {
        self.get(aes)?.as_discrete_vector_iter(data)
    }

    pub fn get_iter_color<'a>(
        &'a self,
        aes: &Aesthetic,
        data: &'a dyn DataSource,
    ) -> Option<Box<dyn Iterator<Item = Color> + 'a>> {
        self.get(aes)?.as_color_vector_iter(data)
    }

    pub fn get_iter_shape<'a>(
        &'a self,
        aes: &Aesthetic,
        data: &'a dyn DataSource,
    ) -> Option<Box<dyn Iterator<Item = Shape> + 'a>> {
        self.get(aes)?.as_shape_vector_iter(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_x_like() {
        // X-like aesthetics
        assert!(Aesthetic::X(AestheticDomain::Continuous).is_x_like());
        assert!(Aesthetic::XBegin.is_x_like());
        assert!(Aesthetic::XEnd.is_x_like());
        assert!(Aesthetic::Xmin(AestheticDomain::Continuous).is_x_like());
        assert!(Aesthetic::Xmax(AestheticDomain::Continuous).is_x_like());
        assert!(Aesthetic::XIntercept.is_x_like());

        // Non-X-like aesthetics
        assert!(!Aesthetic::Y(AestheticDomain::Continuous).is_x_like());
    }

    #[test]
    fn test_is_y_like() {
        // Y-like aesthetics
        assert!(Aesthetic::Y(AestheticDomain::Continuous).is_y_like());
        assert!(Aesthetic::YBegin.is_y_like());
        assert!(Aesthetic::YEnd.is_y_like());
        assert!(Aesthetic::Ymin(AestheticDomain::Continuous).is_y_like());
        assert!(Aesthetic::Ymax(AestheticDomain::Continuous).is_y_like());
        assert!(Aesthetic::YIntercept.is_y_like());

        // Non-Y-like aesthetics
        assert!(!Aesthetic::X(AestheticDomain::Continuous).is_y_like());
        assert!(!Aesthetic::Xmin(AestheticDomain::Continuous).is_y_like());
        assert!(!Aesthetic::Xmax(AestheticDomain::Continuous).is_y_like());
    }
}
