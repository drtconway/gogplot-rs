use crate::data::{DiscreteType, VectorIter};
use crate::scale::traits::DiscreteDomainScale;
use crate::utils::set::DiscreteSet;
use crate::visuals::Shape;

/// Discrete shape scale that maps categories to point shapes.
#[derive(Debug, Clone)]
pub struct ShapeScale {
    shapes: Vec<Shape>,
    elements: DiscreteSet,
}

impl ShapeScale {
    /// Create a new discrete shape scale with a set of shapes.
    pub fn new(shapes: Vec<Shape>) -> Self {
        Self {
            shapes,
            elements: DiscreteSet::new(),
        }
    }

    /// Create a default discrete shape scale with standard shapes.
    pub fn default_shapes() -> Self {
        Self::new(vec![
            Shape::Circle,
            Shape::Square,
            Shape::Triangle,
            Shape::Diamond,
            Shape::Cross,
            Shape::Plus,
        ])
    }
}

impl Default for ShapeScale {
    fn default() -> Self {
        Self::default_shapes()
    }
}

impl super::traits::ScaleBase for ShapeScale {
    fn train<'a>(&mut self, iter: VectorIter<'a>) {
        self.train_discrete(iter);
    }
}

impl super::traits::DiscreteDomainScale for ShapeScale {
    fn categories(&self) -> &DiscreteSet {
        &self.elements
    }

    fn add_categories(&mut self, categories: DiscreteSet) {
        self.elements.union(&categories);
    }
}

impl super::traits::ShapeRangeScale for ShapeScale {
    fn map_value<T: DiscreteType>(&self, value: &T) -> Option<Shape> {
        let ordinal = self.elements.ordinal(value)?;
        let shape = self.shapes[ordinal % self.shapes.len()];
        Some(shape)
    }
}

impl super::traits::ShapeScale for ShapeScale {
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discrete_shape_new() {
        let shapes = vec![Shape::Circle, Shape::Square];
        let scale = ShapeScale::new(shapes);
        assert_eq!(scale.shapes.len(), 2);
    }

}
