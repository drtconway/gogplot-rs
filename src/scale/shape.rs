use super::{ScaleBase, ShapeScale};
use crate::geom::point::PointShape;
use crate::data::GenericVector;
use std::collections::HashMap;

/// Discrete shape scale that maps categories to point shapes.
pub struct DiscreteShape {
    shapes: Vec<PointShape>,
    mapping: HashMap<String, usize>,
}

impl DiscreteShape {
    /// Create a new discrete shape scale with a set of shapes.
    pub fn new(shapes: Vec<PointShape>) -> Self {
        Self {
            shapes,
            mapping: HashMap::new(),
        }
    }

    /// Create a default discrete shape scale with standard shapes.
    pub fn default_shapes() -> Self {
        Self::new(vec![
            PointShape::Circle,
            PointShape::Square,
            PointShape::Triangle,
            PointShape::Diamond,
            PointShape::Cross,
            PointShape::Plus,
        ])
    }
}

impl ScaleBase for DiscreteShape {
    fn train(&mut self, data: &dyn GenericVector) {
        // Extract unique categories and assign them to shapes
        if let Some(strings) = data.as_str() {
            let mut categories: Vec<String> = Vec::new();
            for s in strings.iter() {
                if !categories.contains(s) {
                    categories.push(s.clone());
                }
            }
            
            self.mapping.clear();
            for (idx, category) in categories.iter().enumerate() {
                self.mapping.insert(category.clone(), idx % self.shapes.len());
            }
        }
    }
}

impl ShapeScale for DiscreteShape {
    fn map_to_shape(&self, category: &str) -> Option<PointShape> {
        self.mapping.get(category)
            .and_then(|&idx| self.shapes.get(idx).copied())
    }

    fn legend_breaks(&self) -> Vec<String> {
        let mut breaks: Vec<_> = self.mapping.keys().cloned().collect();
        breaks.sort();
        breaks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::dataframe::StrVec;

    #[test]
    fn test_discrete_shape_new() {
        let shapes = vec![PointShape::Circle, PointShape::Square];
        let scale = DiscreteShape::new(shapes);
        assert_eq!(scale.shapes.len(), 2);
        assert_eq!(scale.mapping.len(), 0);
    }

    #[test]
    fn test_discrete_shape_default() {
        let scale = DiscreteShape::default_shapes();
        assert_eq!(scale.shapes.len(), 6);
        assert_eq!(scale.shapes[0], PointShape::Circle);
        assert_eq!(scale.shapes[1], PointShape::Square);
        assert_eq!(scale.shapes[2], PointShape::Triangle);
        assert_eq!(scale.shapes[3], PointShape::Diamond);
        assert_eq!(scale.shapes[4], PointShape::Cross);
        assert_eq!(scale.shapes[5], PointShape::Plus);
    }

    #[test]
    fn test_discrete_shape_train() {
        let mut scale = DiscreteShape::default_shapes();
        let data = StrVec(vec![
            "cat".to_string(),
            "dog".to_string(),
            "cat".to_string(),
            "bird".to_string()
        ]);
        
        scale.train(&data);
        
        assert_eq!(scale.mapping.len(), 3); // cat, dog, bird
        assert!(scale.mapping.contains_key("cat"));
        assert!(scale.mapping.contains_key("dog"));
        assert!(scale.mapping.contains_key("bird"));
    }

    #[test]
    fn test_discrete_shape_map_category() {
        let mut scale = DiscreteShape::default_shapes();
        let data = StrVec(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
        
        scale.train(&data);
        
        let shape_a = scale.map_to_shape("A");
        let shape_b = scale.map_to_shape("B");
        let shape_c = scale.map_to_shape("C");
        let shape_missing = scale.map_to_shape("D");
        
        assert_eq!(shape_a, Some(PointShape::Circle));
        assert_eq!(shape_b, Some(PointShape::Square));
        assert_eq!(shape_c, Some(PointShape::Triangle));
        assert_eq!(shape_missing, None);
    }

    #[test]
    fn test_discrete_shape_wrap_around() {
        let mut scale = DiscreteShape::new(vec![PointShape::Circle, PointShape::Square]);
        let data = StrVec(vec![
            "A".to_string(),
            "B".to_string(),
            "C".to_string(), // Should wrap back to Circle
        ]);
        
        scale.train(&data);
        
        assert_eq!(scale.map_to_shape("A"), Some(PointShape::Circle));
        assert_eq!(scale.map_to_shape("B"), Some(PointShape::Square));
        assert_eq!(scale.map_to_shape("C"), Some(PointShape::Circle)); // Wrapped
    }

    #[test]
    fn test_discrete_shape_legend_breaks() {
        let mut scale = DiscreteShape::default_shapes();
        let data = StrVec(vec!["Z".to_string(), "A".to_string(), "M".to_string()]);
        
        scale.train(&data);
        
        let breaks = scale.legend_breaks();
        assert_eq!(breaks.len(), 3);
        assert_eq!(breaks, vec!["A", "M", "Z"]); // Should be sorted
    }

    #[test]
    fn test_discrete_shape_retrain() {
        let mut scale = DiscreteShape::default_shapes();
        
        // First training
        let data1 = StrVec(vec!["A".to_string(), "B".to_string()]);
        scale.train(&data1);
        assert_eq!(scale.mapping.len(), 2);
        
        // Second training should replace mapping
        let data2 = StrVec(vec!["X".to_string(), "Y".to_string(), "Z".to_string()]);
        scale.train(&data2);
        assert_eq!(scale.mapping.len(), 3);
        assert!(scale.mapping.contains_key("X"));
        assert!(!scale.mapping.contains_key("A"));
    }
}
