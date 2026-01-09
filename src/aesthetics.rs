use crate::data::{DataSource, DiscreteValue, GenericVector, PrimitiveValue, VectorIter, VectorValue};
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

// AesValue is a type that can be mapped to an aesthetic
// It can be a column name, a constant value, or a computed value
// Each can optionally carry a hint about whether it should be treated as continuous or categorical
#[derive(Clone, PartialEq)]
pub enum AesValue {
    /// Column name from data with optional scale type hint
    Column {
        name: String,
    },
    /// Fixed value with optional scale type hint
    Constant {
        value: PrimitiveValue,
    },
    /// Materialized vector of values (result of scale/stat/position transformation)
    Vector {
        values: Arc<VectorValue>,
        /// Original column name before transformation (for legend titles)
        name: Option<String>,
    },
}

impl std::fmt::Debug for AesValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AesValue::Column {
                name,
            } => f
                .debug_struct("Column")
                .field("name", name)
                .finish(),
            AesValue::Constant { value } => f
                .debug_struct("Constant")
                .field("value", value)
                .finish(),
            AesValue::Vector {
                values: _,
                name: original_name,
            } => f
                .debug_struct("Vector")
                .field("original_name", original_name)
                .field("values", &"<GenericVector>")
                .finish(),
        }
    }
}

impl AesValue {
    /// Create a Column variant from a string-like value with no type hint
    pub fn column(name: impl Into<String>) -> Self {
        AesValue::Column {
            name: name.into(),
        }
    }

    /// Create a Vector variant from materialized values
    pub fn vector(values: impl Into<VectorValue>, original_name: Option<String>) -> Self {
        AesValue::Vector {
            values: Arc::new(values.into()),
            name: original_name,
        }
    }

    /// Extract the original column name (before disambiguation) from Column variants
    /// Falls back to the current name if no original name was stored
    /// Returns None for Constant values
    pub fn as_original_column_name(&self) -> Option<&str> {
        match self {
            AesValue::Column {
                name,
            } => Some(name.as_str()),
            AesValue::Constant { .. } => None,
            AesValue::Vector { name: original_name, .. } => original_name.as_ref().map(|s| s.as_str()),
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
                Some(Box::new(iter.map(|s| DiscreteValue::from(s))))
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
