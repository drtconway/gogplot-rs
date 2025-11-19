use std::collections::HashMap;
use crate::data::PrimitiveValue;

// Supported aesthetics
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Aesthetic {
    X,
    Y,
    Color,
    Alpha,
    Size,
    Shape,
    XBegin,
    XEnd,
    YBegin,
    YEnd,
}

// AesValue is a type that can be mapped to an aesthetic
// It can be a column name, a constant value, or a computed value
#[derive(Debug, Clone, PartialEq)]
pub enum AesValue {
    Column(String),           // Column name from data
    Constant(PrimitiveValue), // Fixed value
    // Computed?
}

// The mapping structure
#[derive(Clone)]
pub struct AesMap {
    map: HashMap<Aesthetic, AesValue>,
}

impl AesMap {
    pub fn new() -> Self {
        Self { map: HashMap::new() }
    }

    pub fn set(&mut self, aes: Aesthetic, value: AesValue) {
        self.map.insert(aes, value);
    }

    pub fn get(&self, aes: &Aesthetic) -> Option<&AesValue> {
        self.map.get(aes)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Aesthetic, &AesValue)> {
        self.map.iter()
    }

    // Convenience methods for column mappings
    pub fn x(&mut self, column: impl Into<String>) {
        self.set(Aesthetic::X, AesValue::Column(column.into()));
    }
    pub fn y(&mut self, column: impl Into<String>) {
        self.set(Aesthetic::Y, AesValue::Column(column.into()));
    }
    pub fn color(&mut self, column: impl Into<String>) {
        self.set(Aesthetic::Color, AesValue::Column(column.into()));
    }
    pub fn alpha(&mut self, column: impl Into<String>) {
        self.set(Aesthetic::Alpha, AesValue::Column(column.into()));
    }
    pub fn size(&mut self, column: impl Into<String>) {
        self.set(Aesthetic::Size, AesValue::Column(column.into()));
    }
    pub fn shape(&mut self, column: impl Into<String>) {
        self.set(Aesthetic::Shape, AesValue::Column(column.into()));
    }

    // Convenience methods for constant value mappings
    pub fn const_color(&mut self, r: u8, g: u8, b: u8, a: u8) {
        // Store as a combined RGBA integer
        let rgba = ((r as i64) << 24) | ((g as i64) << 16) | ((b as i64) << 8) | (a as i64);
        self.set(Aesthetic::Color, AesValue::Constant(PrimitiveValue::Int(rgba)));
    }
    
    pub fn const_alpha(&mut self, alpha: f64) {
        self.set(Aesthetic::Alpha, AesValue::Constant(PrimitiveValue::Float(alpha)));
    }
    
    pub fn const_size(&mut self, size: f64) {
        self.set(Aesthetic::Size, AesValue::Constant(PrimitiveValue::Float(size)));
    }
    
    pub fn const_shape(&mut self, shape: i64) {
        self.set(Aesthetic::Shape, AesValue::Constant(PrimitiveValue::Int(shape)));
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
        
        assert_eq!(aes.get(&Aesthetic::X), Some(&AesValue::Column("col_x".to_string())));
        assert_eq!(aes.get(&Aesthetic::Y), Some(&AesValue::Column("col_y".to_string())));
        assert_eq!(aes.get(&Aesthetic::Color), Some(&AesValue::Column("group".to_string())));
    }
}