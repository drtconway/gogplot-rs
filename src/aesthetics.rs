use crate::data::PrimitiveValue;
use crate::scale::ScaleType;
use std::collections::HashMap;

// Supported aesthetics
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Aesthetic {
    X,
    Y,
    Xmin,
    Xmax,
    Ymin,
    Ymax,
    Lower,   // Q1 (first quartile) for boxplots
    Middle,  // Median for boxplots
    Upper,   // Q3 (third quartile) for boxplots
    Color,
    Fill,
    Alpha,
    Size,
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
            Aesthetic::Color
                | Aesthetic::Fill
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
            Aesthetic::X | Aesthetic::XBegin | Aesthetic::XEnd | Aesthetic::Xmin | Aesthetic::Xmax | Aesthetic::XIntercept
        )
    }

    /// Returns true if this aesthetic relates to the y-axis position.
    /// Used for training y-scales on all relevant data.
    pub fn is_y_like(&self) -> bool {
        matches!(
            self,
            Aesthetic::Y | Aesthetic::YBegin | Aesthetic::YEnd | Aesthetic::Ymin | Aesthetic::Ymax | Aesthetic::YIntercept
                | Aesthetic::Lower | Aesthetic::Middle | Aesthetic::Upper
        )
    }

    /// A printable name for the aesthetic.
    pub fn to_str(&self) -> &'static str {
        match self {
            Aesthetic::X => "x",
            Aesthetic::Y => "y",
            Aesthetic::Xmin => "xmin",
            Aesthetic::Xmax => "xmax",
            Aesthetic::Ymin => "ymin",
            Aesthetic::Ymax => "ymax",
            Aesthetic::Lower => "lower",
            Aesthetic::Middle => "middle",
            Aesthetic::Upper => "upper",
            Aesthetic::Color => "color",
            Aesthetic::Fill => "fill",
            Aesthetic::Alpha => "alpha",
            Aesthetic::Size => "size",
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
}

// AesValue is a type that can be mapped to an aesthetic
// It can be a column name, a constant value, or a computed value
// Each can optionally carry a hint about whether it should be treated as continuous or categorical
#[derive(Debug, Clone, PartialEq)]
pub enum AesValue {
    /// Column name from data with optional scale type hint
    Column { 
        name: String, 
        hint: Option<ScaleType> 
    },
    /// Fixed value with optional scale type hint
    Constant { 
        value: PrimitiveValue, 
        hint: Option<ScaleType> 
    },
}

impl AesValue {
    /// Create a Column variant from a string-like value with no type hint
    pub fn column(name: impl Into<String>) -> Self {
        AesValue::Column { 
            name: name.into(), 
            hint: None 
        }
    }

    /// Create a Column variant that should be treated as continuous
    pub fn continuous_column(name: impl Into<String>) -> Self {
        AesValue::Column { 
            name: name.into(), 
            hint: Some(ScaleType::Continuous) 
        }
    }

    /// Create a Column variant that should be treated as categorical
    /// Use this when you want to treat a numeric column as categorical
    pub fn categorical_column(name: impl Into<String>) -> Self {
        AesValue::Column { 
            name: name.into(), 
            hint: Some(ScaleType::Categorical) 
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
            hint: None 
        }
    }

    /// Create a Constant variant that should be treated as continuous
    pub fn continuous_constant(value: impl Into<PrimitiveValue>) -> Self {
        AesValue::Constant { 
            value: value.into(), 
            hint: Some(ScaleType::Continuous) 
        }
    }

    /// Create a Constant variant that should be treated as categorical
    pub fn categorical_constant(value: impl Into<PrimitiveValue>) -> Self {
        AesValue::Constant { 
            value: value.into(), 
            hint: Some(ScaleType::Categorical) 
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
}

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

    pub fn iter(&self) -> impl Iterator<Item = (&Aesthetic, &AesValue)> {
        self.map.iter()
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

    // Convenience methods for column mappings
    pub fn x(&mut self, column: impl Into<String>) {
        self.set_to_column(Aesthetic::X, column);
    }
    pub fn y(&mut self, column: impl Into<String>) {
        self.set_to_column(Aesthetic::Y, column);
    }
    pub fn color(&mut self, column: impl Into<String>) {
        self.set_to_column(Aesthetic::Color, column);
    }
    pub fn fill(&mut self, column: impl Into<String>) {
        self.set_to_column(Aesthetic::Fill, column);
    }
    pub fn alpha(&mut self, column: impl Into<String>) {
        self.set_to_column(Aesthetic::Alpha, column);
    }
    pub fn size(&mut self, column: impl Into<String>) {
        self.set_to_column(Aesthetic::Size, column);
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
    pub fn ymin(&mut self, column: impl Into<String>) {
        self.set_to_column(Aesthetic::Ymin, column);
    }
    pub fn ymax(&mut self, column: impl Into<String>) {
        self.set_to_column(Aesthetic::Ymax, column);
    }

    // Convenience methods for categorical column mappings
    // Use these when you want to treat a numeric column as categorical
    pub fn x_categorical(&mut self, column: impl Into<String>) {
        self.set(Aesthetic::X, AesValue::categorical_column(column));
    }
    pub fn y_categorical(&mut self, column: impl Into<String>) {
        self.set(Aesthetic::Y, AesValue::categorical_column(column));
    }
    pub fn color_categorical(&mut self, column: impl Into<String>) {
        self.set(Aesthetic::Color, AesValue::categorical_column(column));
    }
    pub fn fill_categorical(&mut self, column: impl Into<String>) {
        self.set(Aesthetic::Fill, AesValue::categorical_column(column));
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
        self.set(Aesthetic::X, AesValue::continuous_column(column));
    }
    pub fn y_continuous(&mut self, column: impl Into<String>) {
        self.set(Aesthetic::Y, AesValue::continuous_column(column));
    }
    pub fn color_continuous(&mut self, column: impl Into<String>) {
        self.set(Aesthetic::Color, AesValue::continuous_column(column));
    }
    pub fn fill_continuous(&mut self, column: impl Into<String>) {
        self.set(Aesthetic::Fill, AesValue::continuous_column(column));
    }
    pub fn size_continuous(&mut self, column: impl Into<String>) {
        self.set(Aesthetic::Size, AesValue::continuous_column(column));
    }
    pub fn alpha_continuous(&mut self, column: impl Into<String>) {
        self.set(Aesthetic::Alpha, AesValue::continuous_column(column));
    }

    // Convenience methods for constant value mappings
    pub fn const_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        use crate::theme::Color;
        let rgba = Color(r, g, b, a).into();
        self.set(
            Aesthetic::Color,
            AesValue::constant(PrimitiveValue::Int(rgba)),
        );
    }

    pub fn const_fill(&mut self, r: u8, g: u8, b: u8, a: u8) {
        use crate::theme::Color;
        let rgba = Color(r, g, b, a).into();
        self.set(
            Aesthetic::Fill,
            AesValue::constant(PrimitiveValue::Int(rgba)),
        );
    }

    pub fn const_alpha(&mut self, alpha: f64) {
        self.set(
            Aesthetic::Alpha,
            AesValue::constant(PrimitiveValue::Float(alpha)),
        );
    }

    pub fn const_size(&mut self, size: f64) {
        self.set(
            Aesthetic::Size,
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_map() {
        let mut aes = AesMap::new();
        aes.x("col_x");
        aes.y("col_y");
        aes.color("group");

        assert_eq!(aes.get(&Aesthetic::X), Some(&AesValue::column("col_x")));
        assert_eq!(aes.get(&Aesthetic::Y), Some(&AesValue::column("col_y")));
        assert_eq!(aes.get(&Aesthetic::Color), Some(&AesValue::column("group")));
    }

    #[test]
    fn test_is_grouping() {
        // Grouping aesthetics
        assert!(Aesthetic::Color.is_grouping());
        assert!(Aesthetic::Fill.is_grouping());
        assert!(Aesthetic::Shape.is_grouping());
        assert!(Aesthetic::Linetype.is_grouping());
        assert!(Aesthetic::Group.is_grouping());

        // Non-grouping aesthetics
        assert!(!Aesthetic::X.is_grouping());
        assert!(!Aesthetic::Y.is_grouping());
        assert!(!Aesthetic::Alpha.is_grouping());
        assert!(!Aesthetic::Size.is_grouping());
        assert!(!Aesthetic::XBegin.is_grouping());
        assert!(!Aesthetic::XEnd.is_grouping());
        assert!(!Aesthetic::YBegin.is_grouping());
        assert!(!Aesthetic::YEnd.is_grouping());
    }

    #[test]
    fn test_is_x_like() {
        // X-like aesthetics
        assert!(Aesthetic::X.is_x_like());
        assert!(Aesthetic::XBegin.is_x_like());
        assert!(Aesthetic::XEnd.is_x_like());
        assert!(Aesthetic::Xmin.is_x_like());
        assert!(Aesthetic::Xmax.is_x_like());
        assert!(Aesthetic::XIntercept.is_x_like());

        // Non-X-like aesthetics
        assert!(!Aesthetic::Y.is_x_like());
        assert!(!Aesthetic::Color.is_x_like());
        assert!(!Aesthetic::Fill.is_x_like());
    }

    #[test]
    fn test_is_y_like() {
        // Y-like aesthetics
        assert!(Aesthetic::Y.is_y_like());
        assert!(Aesthetic::YBegin.is_y_like());
        assert!(Aesthetic::YEnd.is_y_like());
        assert!(Aesthetic::Ymin.is_y_like());
        assert!(Aesthetic::Ymax.is_y_like());
        assert!(Aesthetic::YIntercept.is_y_like());

        // Non-Y-like aesthetics
        assert!(!Aesthetic::X.is_y_like());
        assert!(!Aesthetic::Xmin.is_y_like());
        assert!(!Aesthetic::Xmax.is_y_like());
        assert!(!Aesthetic::Color.is_y_like());
        assert!(!Aesthetic::Fill.is_y_like());
    }
}
