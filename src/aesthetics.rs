use crate::data::{self, DataSource, DiscreteValue, GenericVector, PrimitiveValue, VectorIter};
use crate::error::PlotError;
use crate::scale::ScaleType;
use crate::utils::dataframe::{BoolVec, FloatVec, IntVec, StrVec};
use crate::visuals::Shape;
use core::panic;
use std::collections::HashMap;

pub mod builder;

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
            Aesthetic::X(_) | Aesthetic::Xmin(_) | Aesthetic::Xmax(_) 
            | Aesthetic::XBegin | Aesthetic::XEnd | Aesthetic::XIntercept => Some(AestheticProperty::X),
            Aesthetic::Y(_) | Aesthetic::Ymin(_) | Aesthetic::Ymax(_)
            | Aesthetic::YBegin | Aesthetic::YEnd | Aesthetic::YIntercept
            | Aesthetic::Lower | Aesthetic::Middle | Aesthetic::Upper => Some(AestheticProperty::Y),
            // Group and Label don't have corresponding properties
            Aesthetic::Group | Aesthetic::Label => None,
        }
    }
}

impl From<(AestheticProperty, AestheticDomain)> for Aesthetic {
    fn from(value: (AestheticProperty, AestheticDomain)) -> Self {
        match value.0 {
            AestheticProperty::X => Aesthetic::X(value.1),
            AestheticProperty::Y => Aesthetic::Y(value.1),
            AestheticProperty::Color => Aesthetic::Color(value.1),
            AestheticProperty::Fill => Aesthetic::Fill(value.1),
            AestheticProperty::Size => Aesthetic::Size(value.1),
            AestheticProperty::Alpha => Aesthetic::Alpha(value.1),
            AestheticProperty::Shape => Aesthetic::Shape,
            AestheticProperty::Linetype => Aesthetic::Linetype,
        }
    }
}

// AesValue is a type that can be mapped to an aesthetic
// It can be a column name, a constant value, or a computed value
// Each can optionally carry a hint about whether it should be treated as continuous or categorical
#[derive(Debug, Clone, PartialEq)]
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

    /// Extract the column name from Column variants
    /// Returns None for Constant values
    pub fn as_column_name(&self) -> Option<&str> {
        match self {
            AesValue::Column { name, .. } => Some(name.as_str()),
            AesValue::Constant { .. } => None,
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
        }
    }

    /// Get the user's scale type hint if one was provided
    pub fn user_hint(&self) -> Option<ScaleType> {
        match self {
            AesValue::Column { hint, .. } => *hint,
            AesValue::Constant { hint, .. } => *hint,
        }
    }

    /// Returns true if this value has an explicit categorical hint
    pub fn is_categorical(&self) -> bool {
        self.user_hint() == Some(ScaleType::Categorical)
    }

    /// Returns true if this value has an explicit continuous hint
    pub fn is_continuous(&self) -> bool {
        self.user_hint() == Some(ScaleType::Continuous)
    }

    /// Get the constant value if this is a Constant variant
    pub fn as_constant(&self) -> Option<&PrimitiveValue> {
        match self {
            AesValue::Constant { value, .. } => Some(value),
            _ => None,
        }
    }

    pub fn duplicate(
        &self,
        data: &dyn DataSource,
    ) -> std::result::Result<(AesValue, Option<(String, Box<dyn GenericVector>)>), PlotError> {
        match self {
            AesValue::Column {
                name,
                hint,
                original_name,
            } => {
                let column = data.get(name.as_str()).unwrap();
                let cloned_column: Box<dyn GenericVector> = match column.iter() {
                    data::VectorIter::Int(iter) => {
                        let vec: Vec<i64> = iter.collect();
                        Box::new(IntVec(vec))
                    }
                    data::VectorIter::Float(iter) => {
                        let vec: Vec<f64> = iter.collect();
                        Box::new(FloatVec(vec))
                    }
                    data::VectorIter::Str(iter) => {
                        let vec: Vec<String> = iter.map(|s| s.to_string()).collect();
                        Box::new(StrVec(vec))
                    }
                    data::VectorIter::Bool(iter) => {
                        let vec: Vec<bool> = iter.collect();
                        Box::new(BoolVec(vec))
                    }
                };
                Ok((
                    AesValue::Column {
                        name: name.clone(),
                        hint: *hint,
                        original_name: original_name.clone(),
                    },
                    Some((name.clone(), cloned_column)),
                ))
            }
            AesValue::Constant { value, hint } => Ok((
                AesValue::Constant {
                    value: value.clone(),
                    hint: *hint,
                },
                None,
            )),
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

    pub fn get_iter<'a>(
        &self,
        aes: &Aesthetic,
        data: &'a dyn DataSource,
    ) -> Option<Box<dyn Iterator<Item = PrimitiveValue> + 'a>> {
        match self.get(aes) {
            Some(value) => {
                match value {
                    AesValue::Column { name, .. } => {
                        // Look up the column in the data source
                        let column = data.get(name.as_str())?;
                        // Return an iterator over the column's values as PrimitiveValue
                        let iter: Box<dyn Iterator<Item = PrimitiveValue> + 'a> = match column
                            .iter()
                        {
                            data::VectorIter::Int(iter) => Box::new(iter.map(PrimitiveValue::Int)),
                            data::VectorIter::Float(iter) => {
                                Box::new(iter.map(PrimitiveValue::Float))
                            }
                            data::VectorIter::Str(iter) => {
                                Box::new(iter.map(|s| PrimitiveValue::Str(s.to_string())))
                            }
                            data::VectorIter::Bool(iter) => {
                                Box::new(iter.map(PrimitiveValue::Bool))
                            }
                        };
                        Some(iter)
                    }
                    AesValue::Constant { value, .. } => {
                        let n = data.len();
                        Some(Box::new(std::iter::repeat(value.clone()).take(n))
                            as Box<dyn Iterator<Item = PrimitiveValue> + 'a>)
                    }
                }
            }
            _ => None,
        }
    }

    pub fn get_vector_iter<'a>(
        &'a self,
        aes: &'a Aesthetic,
        data: &'a dyn DataSource,
    ) -> Option<VectorIter<'a>> {
        match self.get(aes) {
            Some(value) => match value {
                AesValue::Column { name, .. } => {
                    let column = data.get(name.as_str())?;
                    Some(column.iter())
                }
                AesValue::Constant { value, .. } => match value {
                    PrimitiveValue::Int(i) => {
                        let n = data.len();
                        Some(VectorIter::Int(Box::new(std::iter::repeat(*i).take(n))))
                    }
                    PrimitiveValue::Float(f) => {
                        let n = data.len();
                        Some(VectorIter::Float(Box::new(std::iter::repeat(*f).take(n))))
                    }
                    PrimitiveValue::Str(s) => {
                        let n = data.len();
                        Some(VectorIter::Str(Box::new(
                            std::iter::repeat(s.as_str()).take(n),
                        )))
                    }
                    PrimitiveValue::Bool(b) => {
                        let n = data.len();
                        Some(VectorIter::Bool(Box::new(std::iter::repeat(*b).take(n))))
                    }
                },
            },
            None => None,
        }
    }

    pub fn get_iter_float<'a>(
        &self,
        aes: &Aesthetic,
        data: &'a dyn DataSource,
    ) -> Option<Box<dyn Iterator<Item = f64> + 'a>> {
        match self.get(aes) {
            Some(value) => {
                match value {
                    AesValue::Column { name, .. } => {
                        // Look up the column in the data source
                        let column = data.get(name.as_str())?;
                        // Return an iterator over the column's values as f64
                        if let Some(iter) = column.iter_float() {
                            return Some(iter);
                        }
                        if let Some(iter) = column.iter_int() {
                            // Convert int to float
                            let float_iter = iter.map(|v| v as f64);
                            return Some(Box::new(float_iter) as Box<dyn Iterator<Item = f64> + 'a>);
                        }
                    }
                    AesValue::Constant { value, .. } => {
                        if let PrimitiveValue::Float(f) = value {
                            let n = data.len();
                            return Some(Box::new(std::iter::repeat(*f).take(n))
                                as Box<dyn Iterator<Item = f64> + 'a>);
                        }
                        if let PrimitiveValue::Int(i) = value {
                            let f = *i as f64;
                            let n = data.len();
                            return Some(Box::new(std::iter::repeat(f).take(n))
                                as Box<dyn Iterator<Item = f64> + 'a>);
                        }
                    }
                }
            }
            None => {}
        }
        None
    }

    pub fn get_iter_int<'a>(
        &self,
        aes: &Aesthetic,
        data: &'a dyn DataSource,
    ) -> Option<Box<dyn Iterator<Item = i64> + 'a>> {
        match self.get(aes) {
            Some(value) => {
                match value {
                    AesValue::Column { name, .. } => {
                        // Look up the column in the data source
                        let column = data.get(name.as_str())?;
                        if let Some(iter) = column.iter_int() {
                            return Some(Box::new(iter) as Box<dyn Iterator<Item = i64> + 'a>);
                        }
                    }
                    AesValue::Constant { value, .. } => {
                        if let PrimitiveValue::Int(i) = value {
                            let n = data.len();
                            return Some(Box::new(std::iter::repeat(*i).take(n))
                                as Box<dyn Iterator<Item = i64> + 'a>);
                        }
                    }
                }
            }
            None => {}
        }
        None
    }

    pub fn get_iter_string<'a>(
        &self,
        aes: &Aesthetic,
        data: &'a dyn DataSource,
    ) -> Option<Box<dyn Iterator<Item = String> + 'a>> {
        match self.get(aes) {
            Some(value) => {
                match value {
                    AesValue::Column { name, .. } => {
                        // Look up the column in the data source
                        let column = data.get(name.as_str())?;
                        match column.iter() {
                            data::VectorIter::Str(iter) => {
                                let string_iter = iter.map(|s| s.to_string());
                                return Some(Box::new(string_iter)
                                    as Box<dyn Iterator<Item = String> + 'a>);
                            }
                            _ => {
                                panic!("Only string columns can be used as string values");
                            }
                        }
                    }
                    AesValue::Constant { value, .. } => match value {
                        PrimitiveValue::Str(s) => {
                            let n = data.len();
                            return Some(Box::new(
                                std::iter::repeat(s.to_string()).take(n),
                            )
                                as Box<dyn Iterator<Item = String> + 'a>);
                        }
                        _ => {
                            panic!("Only string constants can be used as string values");
                        }
                    },
                }
            }
            None => {}
        }
        None
    }

    pub fn get_iter_discrete<'a>(
        &self,
        aes: &Aesthetic,
        data: &'a dyn DataSource,
    ) -> Option<Box<dyn Iterator<Item = DiscreteValue> + 'a>> {
        match self.get(aes) {
            Some(value) => {
                match value {
                    AesValue::Column { name, .. } => {
                        // Look up the column in the data source
                        let column = data.get(name.as_str())?;
                        match column.iter() {
                            data::VectorIter::Int(iter) => {
                                let discrete_iter = iter.map(DiscreteValue::Int);
                                return Some(Box::new(discrete_iter)
                                    as Box<dyn Iterator<Item = DiscreteValue> + 'a>);
                            }
                            data::VectorIter::Float(_) => {
                                panic!("Float columns cannot be used as discrete values");
                            }
                            data::VectorIter::Str(iter) => {
                                let discrete_iter = iter.map(|s| DiscreteValue::Str(s.to_string()));
                                return Some(Box::new(discrete_iter)
                                    as Box<dyn Iterator<Item = DiscreteValue> + 'a>);
                            }
                            data::VectorIter::Bool(iter) => {
                                let discrete_iter = iter.map(DiscreteValue::Bool);
                                return Some(Box::new(discrete_iter)
                                    as Box<dyn Iterator<Item = DiscreteValue> + 'a>);
                            }
                        }
                    }
                    AesValue::Constant { value, .. } => match value {
                        PrimitiveValue::Int(i) => {
                            let n = data.len();
                            return Some(
                                Box::new(std::iter::repeat(DiscreteValue::Int(*i)).take(n))
                                    as Box<dyn Iterator<Item = DiscreteValue> + 'a>,
                            );
                        }
                        PrimitiveValue::Str(s) => {
                            let n = data.len();
                            return Some(Box::new(
                                std::iter::repeat(DiscreteValue::Str(s.to_string())).take(n),
                            )
                                as Box<dyn Iterator<Item = DiscreteValue> + 'a>);
                        }
                        PrimitiveValue::Bool(b) => {
                            let n = data.len();
                            return Some(
                                Box::new(std::iter::repeat(DiscreteValue::Bool(*b)).take(n))
                                    as Box<dyn Iterator<Item = DiscreteValue> + 'a>,
                            );
                        }
                        PrimitiveValue::Float(_) => {
                            panic!("Float constants cannot be used as discrete values");
                        }
                    },
                }
            }
            None => {}
        }
        None
    }

    pub fn get_iter_color<'a>(
        &self,
        aes: &Aesthetic,
        data: &'a dyn DataSource,
    ) -> Option<Box<dyn Iterator<Item = crate::theme::Color> + 'a>> {
        match self.get(aes) {
            Some(value) => match value {
                AesValue::Column { name, .. } => {
                    let column = data.get(name.as_str())?;
                    let iter: Box<dyn Iterator<Item = crate::theme::Color> + 'a> =
                        match column.iter() {
                            data::VectorIter::Int(iter) => {
                                Box::new(iter.map(crate::theme::Color::from))
                            }
                            data::VectorIter::Float(_) => {
                                panic!("Float columns cannot be mapped to colors")
                            }
                            data::VectorIter::Str(_) => {
                                panic!("String columns cannot be mapped to colors")
                            }
                            data::VectorIter::Bool(_) => {
                                panic!("Boolean columns cannot be mapped to colors")
                            }
                        };
                    Some(iter)
                }
                AesValue::Constant { value, .. } => match value {
                    PrimitiveValue::Int(i) => {
                        let n = data.len();
                        Some(
                            Box::new(std::iter::repeat(crate::theme::Color::from(*i)).take(n))
                                as Box<dyn Iterator<Item = crate::theme::Color> + 'a>,
                        )
                    }
                    PrimitiveValue::Float(_) => {
                        panic!("Float constants cannot be used as colors");
                    }
                    PrimitiveValue::Str(_) => {
                        panic!("String constants cannot be used as colors");
                    }
                    PrimitiveValue::Bool(_) => {
                        panic!("Boolean constants cannot be used as colors");
                    }
                },
            },
            None => None,
        }
    }

    pub fn get_iter_shape<'a>(
        &self,
        aes: &Aesthetic,
        data: &'a dyn DataSource,
    ) -> Option<Box<dyn Iterator<Item = Shape> + 'a>> {
        match self.get(aes) {
            Some(value) => match value {
                AesValue::Column { name, .. } => {
                    let column = data.get(name.as_str())?;
                    let iter: Box<dyn Iterator<Item = Shape> + 'a> = match column.iter() {
                        data::VectorIter::Int(iter) => Box::new(iter.map(|v| Shape::from(v))),
                        data::VectorIter::Float(_) => {
                            panic!("Float columns cannot be mapped to shapes")
                        }
                        data::VectorIter::Str(_) => {
                            panic!("String columns cannot be mapped to shapes")
                        }
                        data::VectorIter::Bool(_) => {
                            panic!("Boolean columns cannot be mapped to shapes")
                        }
                    };
                    Some(iter)
                }
                AesValue::Constant { value, .. } => match value {
                    PrimitiveValue::Int(i) => {
                        let n = data.len();
                        Some(Box::new(std::iter::repeat(Shape::from(*i)).take(n))
                            as Box<dyn Iterator<Item = Shape> + 'a>)
                    }
                    _ => {
                        panic!("Only integer constants can be used as shapes");
                    }
                },
            },
            None => None,
        }
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
