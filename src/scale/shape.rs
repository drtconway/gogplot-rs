use crate::data::{DiscreteType, GenericVector, VectorIter};
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
    use crate::utils::dataframe::StrVec;

    #[test]
    fn test_discrete_shape_new() {
        let shapes = vec![Shape::Circle, Shape::Square];
        let scale = ShapeScale::new(shapes);
        assert_eq!(scale.shapes.len(), 2);
        assert_eq!(scale.mapping.len(), 0);
    }

    #[test]
    fn test_discrete_shape_default() {
        let scale = ShapeScale::default_shapes();
        assert_eq!(scale.shapes.len(), 6);
        assert_eq!(scale.shapes[0], Shape::Circle);
        assert_eq!(scale.shapes[1], Shape::Square);
        assert_eq!(scale.shapes[2], Shape::Triangle);
        assert_eq!(scale.shapes[3], Shape::Diamond);
        assert_eq!(scale.shapes[4], Shape::Cross);
        assert_eq!(scale.shapes[5], Shape::Plus);
    }

    #[test]
    fn test_discrete_shape_train() {
        let mut scale = ShapeScale::default_shapes();
        let data = StrVec(vec![
            "cat".to_string(),
            "dog".to_string(),
            "cat".to_string(),
            "bird".to_string(),
        ]);

        scale.train(&[&data]);

        assert_eq!(scale.mapping.len(), 3); // cat, dog, bird
        assert!(scale.mapping.contains_key("cat"));
        assert!(scale.mapping.contains_key("dog"));
        assert!(scale.mapping.contains_key("bird"));
    }

    #[test]
    fn test_discrete_shape_map_category() {
        let mut scale = ShapeScale::default_shapes();
        let data = StrVec(vec!["A".to_string(), "B".to_string(), "C".to_string()]);

        scale.train(&[&data]);

        let shape_a = scale.map_to_shape("A");
        let shape_b = scale.map_to_shape("B");
        let shape_c = scale.map_to_shape("C");
        let shape_missing = scale.map_to_shape("D");

        assert_eq!(shape_a, Some(Shape::Circle));
        assert_eq!(shape_b, Some(Shape::Square));
        assert_eq!(shape_c, Some(Shape::Triangle));
        assert_eq!(shape_missing, None);
    }

    #[test]
    fn test_discrete_shape_wrap_around() {
        let mut scale = ShapeScale::new(vec![Shape::Circle, Shape::Square]);
        let data = StrVec(vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(), // Should wrap back to Circle
        ]);

        scale.train(&[&data]);

        assert_eq!(scale.map_to_shape("A"), Some(Shape::Circle));
        assert_eq!(scale.map_to_shape("B"), Some(Shape::Square));
        assert_eq!(scale.map_to_shape("C"), Some(Shape::Circle)); // Wrapped
    }

    #[test]
    fn test_discrete_shape_legend_breaks() {
        let mut scale = ShapeScale::default_shapes();
        let data = StrVec(vec!["Z".to_string(), "A".to_string(), "M".to_string()]);

        scale.train(&[&data]);

        let breaks = scale.legend_breaks();
        assert_eq!(breaks.len(), 3);
        assert_eq!(breaks, vec!["A", "M", "Z"]); // Should be sorted
    }

    #[test]
    fn test_discrete_shape_retrain() {
        let mut scale = ShapeScale::default_shapes();

        // First training
        let data1 = StrVec(vec!["A".to_string(), "B".to_string()]);
        scale.train(&[&data1]);
        assert_eq!(scale.mapping.len(), 2);

        // Second training should replace mapping
        let data2 = StrVec(vec!["X".to_string(), "Y".to_string(), "Z".to_string()]);
        scale.train(&[&data2]);
        assert_eq!(scale.mapping.len(), 3);
        assert!(scale.mapping.contains_key("X"));
        assert!(!scale.mapping.contains_key("A"));
    }
}
