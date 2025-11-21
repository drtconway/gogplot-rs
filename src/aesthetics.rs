use crate::data::PrimitiveValue;
use std::collections::HashMap;

// Supported aesthetics
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Aesthetic {
    X,
    Y,
    Ymin,
    Ymax,
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
}

// AesValue is a type that can be mapped to an aesthetic
// It can be a column name, a constant value, or a computed value
#[derive(Debug, Clone, PartialEq)]
pub enum AesValue {
    Column(String), // Column name from data
    Constant(PrimitiveValue), // Fixed value
                    // Computed?
}

impl AesValue {
    /// Create a Column variant from a string-like value
    pub fn column(name: impl Into<String>) -> Self {
        AesValue::Column(name.into())
    }
}

// The mapping structure
#[derive(Clone)]
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

    // Convenience methods for constant value mappings
    pub fn const_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        use crate::theme::Color;
        let rgba = Color(r, g, b, a).into();
        self.set(
            Aesthetic::Color,
            AesValue::Constant(PrimitiveValue::Int(rgba)),
        );
    }

    pub fn const_fill(&mut self, r: u8, g: u8, b: u8, a: u8) {
        use crate::theme::Color;
        let rgba = Color(r, g, b, a).into();
        self.set(
            Aesthetic::Fill,
            AesValue::Constant(PrimitiveValue::Int(rgba)),
        );
    }

    pub fn const_alpha(&mut self, alpha: f64) {
        self.set(
            Aesthetic::Alpha,
            AesValue::Constant(PrimitiveValue::Float(alpha)),
        );
    }

    pub fn const_size(&mut self, size: f64) {
        self.set(
            Aesthetic::Size,
            AesValue::Constant(PrimitiveValue::Float(size)),
        );
    }

    pub fn const_shape(&mut self, shape: i64) {
        self.set(
            Aesthetic::Shape,
            AesValue::Constant(PrimitiveValue::Int(shape)),
        );
    }

    pub fn const_linetype(&mut self, pattern: impl Into<String>) {
        self.set(
            Aesthetic::Linetype,
            AesValue::Constant(PrimitiveValue::Str(pattern.into())),
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
}
