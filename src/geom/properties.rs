use crate::{
    data::VectorIter,
    theme::Color,
    visuals::Shape,
};

pub enum Property {
    Float(f64),
    String(String),
    Color(Color),
    Shape(Shape),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    Int(i64),
    Float(f64),
    String(String),
    Color(Color),
    Shape(Shape),
}

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyVector {
    Int(Vec<i64>),
    Float(Vec<f64>),
    String(Vec<String>),
    Color(Vec<Color>),
    Shape(Vec<Shape>),
}

impl PropertyVector {
    pub fn len(&self) -> usize {
        match self {
            PropertyVector::Int(v) => v.len(),
            PropertyVector::Float(v) => v.len(),
            PropertyVector::String(v) => v.len(),
            PropertyVector::Color(v) => v.len(),
            PropertyVector::Shape(v) => v.len(),
        }
    }
    
    pub fn to_color(self) -> PropertyVector {
        match self {
            PropertyVector::Int(v) => {
                let colors: Vec<Color> = v.into_iter().map(|c| Color::from(c)).collect();
                PropertyVector::Color(colors)
            }
            PropertyVector::Color(_) => self.clone(),
            _ => panic!("Cannot convert to Color PropertyVector"),
        }
    }

    pub fn to_shape(self) -> PropertyVector {
        match self {
            PropertyVector::Int(v) => {
                let shapes: Vec<Shape> = v.into_iter().map(|s| Shape::from(s)).collect();
                PropertyVector::Shape(shapes)
            }
            PropertyVector::Shape(_) => self.clone(),
            _ => panic!("Cannot convert to Shape PropertyVector"),
        }
    }

    pub fn as_floats(self) -> Vec<f64> {
        match self {
            PropertyVector::Float(v) => v,
            _ => panic!("Not a Float PropertyVector"),
        }
    }

    pub fn as_strings(self) -> Vec<String> {
        match self {
            PropertyVector::String(v) => v,
            _ => panic!("Not a String PropertyVector"),
        }
    }

    pub fn as_colors(self) -> Vec<Color> {
        match self.to_color() {
            PropertyVector::Color(v) => v,
            _ => panic!("Not a Color PropertyVector"),
        }
    }

    pub fn as_shapes(self) -> Vec<Shape> {
        match self.to_shape() {
            PropertyVector::Shape(v) => v,
            _ => panic!("Not a Shape PropertyVector"),
        }
    }
}

impl<'a> From<VectorIter<'a>> for PropertyVector {
    fn from(iter: VectorIter<'a>) -> Self {
        match iter {
            VectorIter::Int(iter) => PropertyVector::Int(iter.collect()),
            VectorIter::Float(iter) => PropertyVector::Float(iter.collect()),
            VectorIter::Str(iter) => PropertyVector::String(iter.map(|s| s.to_string()).collect()),
            VectorIter::Bool(_) => panic!("invalid property vector"),
        }
    }
}
