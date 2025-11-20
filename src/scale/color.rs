use super::{ScaleBase, ColorScale};
use crate::theme::{Color, color};
use crate::data::GenericVector;
use std::collections::{HashMap, HashSet};

/// Discrete color scale that maps categories to colors.
pub struct DiscreteColor {
    palette: Vec<Color>,
    mapping: HashMap<String, usize>,
}

impl DiscreteColor {
    /// Create a new discrete color scale with a palette.
    pub fn new(palette: Vec<Color>) -> Self {
        Self {
            palette,
            mapping: HashMap::new(),
        }
    }

    /// Create a default discrete color scale with a colorblind-friendly palette.
    pub fn default_palette() -> Self {
        Self::new(vec![
            Color(228, 26, 28, 255),   // red
            Color(55, 126, 184, 255),  // blue
            Color(77, 175, 74, 255),   // green
            Color(152, 78, 163, 255),  // purple
            Color(255, 127, 0, 255),   // orange
            Color(255, 255, 51, 255),  // yellow
            Color(166, 86, 40, 255),   // brown
            Color(247, 129, 191, 255), // pink
        ])
    }

    /// Set a custom color palette.
    pub fn set_palette(&mut self, palette: Vec<Color>) {
        self.palette = palette;
    }

}

impl ScaleBase for DiscreteColor {
    fn train(&mut self, data: &[&dyn GenericVector]) {
        // Extract unique categories from all data vectors and assign them to palette colors
        let mut categories: Vec<String> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        
        for vec in data {
            if let Some(strings) = vec.as_str() {
                for s in strings.iter() {
                    if !seen.contains(s) {
                        categories.push(s.clone());
                        seen.insert(s.clone());
                    }
                }
            }
        }
        
        self.mapping.clear();
        for (idx, category) in categories.iter().enumerate() {
            self.mapping.insert(category.clone(), idx % self.palette.len());
        }
    }
}

impl ColorScale for DiscreteColor {
    fn map_discrete_to_color(&self, category: &str) -> Option<Color> {
        self.mapping.get(category)
            .and_then(|&idx| self.palette.get(idx).copied())
    }

    fn legend_breaks(&self) -> Vec<String> {
        let mut breaks: Vec<_> = self.mapping.keys().cloned().collect();
        breaks.sort();
        breaks
    }
}

/// Continuous color scale that maps numeric values to a gradient.
pub struct ContinuousColor {
    domain: (f64, f64),
    colors: Vec<Color>,
}

impl ContinuousColor {
    /// Create a new continuous color gradient with a list of colors.
    /// Colors are interpolated evenly across the domain.
    pub fn new(domain: (f64, f64), colors: Vec<Color>) -> Self {
        assert!(!colors.is_empty(), "Must provide at least one color");
        Self {
            domain,
            colors,
        }
    }

    /// Create a two-color gradient (for backwards compatibility).
    pub fn gradient(domain: (f64, f64), low_color: Color, high_color: Color) -> Self {
        Self::new(domain, vec![low_color, high_color])
    }

    /// Create a default blue to black gradient (like ggplot2).
    pub fn default_gradient(domain: (f64, f64)) -> Self {
        Self::new(
            domain,
            vec![
                color::LIGHTBLUE3,   // dark blue
                color::BLACK,     // black
            ]
        )
    }

    /// Interpolate between colors in the palette.
    fn interpolate_color(&self, t: f64) -> Color {
        let t = t.clamp(0.0, 1.0);
        
        if self.colors.len() == 1 {
            return self.colors[0];
        }
        
        // Determine which segment of the color palette we're in
        let segment_count = self.colors.len() - 1;
        let scaled = t * segment_count as f64;
        let segment = (scaled.floor() as usize).min(segment_count - 1);
        let local_t = scaled - segment as f64;
        
        // Interpolate between the two colors in this segment
        let Color(r1, g1, b1, a1) = self.colors[segment];
        let Color(r2, g2, b2, a2) = self.colors[segment + 1];
        
        let r = (r1 as f64 + local_t * (r2 as f64 - r1 as f64)) as u8;
        let g = (g1 as f64 + local_t * (g2 as f64 - g1 as f64)) as u8;
        let b = (b1 as f64 + local_t * (b2 as f64 - b1 as f64)) as u8;
        let a = (a1 as f64 + local_t * (a2 as f64 - a1 as f64)) as u8;
        
        Color(r, g, b, a)
    }
}

impl ScaleBase for ContinuousColor {
    fn train(&mut self, data: &[&dyn GenericVector]) {
        use crate::data::compute_min_max;
        
        if let Some((min, max)) = compute_min_max(data) {
            self.domain = (min, max);
        }
    }
}

impl ColorScale for ContinuousColor {
    fn map_continuous_to_color(&self, value: f64) -> Option<Color> {
        let (d0, d1) = self.domain;
        
        // Check bounds
        if value < d0.min(d1) || value > d0.max(d1) {
            return None;
        }
        
        // Normalize to [0, 1]
        let t = (value - d0) / (d1 - d0);
        Some(self.interpolate_color(t))
    }

    fn legend_breaks(&self) -> Vec<String> {
        // Generate some representative values
        let (d0, d1) = self.domain;
        vec![
            format!("{:.2}", d0),
            format!("{:.2}", (d0 + d1) / 2.0),
            format!("{:.2}", d1),
        ]
    }
    
    fn is_continuous(&self) -> bool {
        true
    }
    
    fn get_continuous_domain(&self) -> Option<(f64, f64)> {
        Some(self.domain)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::dataframe::StrVec;

    #[test]
    fn test_discrete_color_new() {
        let palette = vec![Color(255, 0, 0, 255), Color(0, 255, 0, 255)];
        let scale = DiscreteColor::new(palette.clone());
        assert_eq!(scale.palette.len(), 2);
        assert_eq!(scale.mapping.len(), 0);
    }

    #[test]
    fn test_discrete_color_default_palette() {
        let scale = DiscreteColor::default_palette();
        assert_eq!(scale.palette.len(), 8);
    }

    #[test]
    fn test_discrete_color_train() {
        let mut scale = DiscreteColor::default_palette();
        let data = StrVec(vec!["A".to_string(), "B".to_string(), "A".to_string(), "C".to_string()]);
        
        scale.train(&[&data]);
        
        assert_eq!(scale.mapping.len(), 3); // A, B, C
        assert!(scale.mapping.contains_key("A"));
        assert!(scale.mapping.contains_key("B"));
        assert!(scale.mapping.contains_key("C"));
    }

    #[test]
    fn test_discrete_color_map_category() {
        let mut scale = DiscreteColor::default_palette();
        let data = StrVec(vec!["red".to_string(), "blue".to_string(), "green".to_string()]);
        
        scale.train(&[&data]);
        
        let color_red = scale.map_discrete_to_color("red");
        let color_blue = scale.map_discrete_to_color("blue");
        let color_missing = scale.map_discrete_to_color("yellow");
        
        assert!(color_red.is_some());
        assert!(color_blue.is_some());
        assert!(color_missing.is_none());
    }

    #[test]
    fn test_discrete_color_legend_breaks() {
        let mut scale = DiscreteColor::default_palette();
        let data = StrVec(vec!["Z".to_string(), "A".to_string(), "M".to_string()]);
        
        scale.train(&[&data]);
        
        let breaks = scale.legend_breaks();
        assert_eq!(breaks.len(), 3);
        assert_eq!(breaks, vec!["A", "M", "Z"]); // Should be sorted
    }

    #[test]
    fn test_continuous_color_new() {
        let scale = ContinuousColor::new(
            (0.0, 100.0),
            vec![Color::rgb(0, 0, 255), Color::rgb(255, 0, 0)]
        );
        assert_eq!(scale.domain, (0.0, 100.0));
        assert_eq!(scale.colors.len(), 2);
    }

    #[test]
    fn test_continuous_color_default_gradient() {
        let scale = ContinuousColor::default_gradient((0.0, 1.0));
        assert_eq!(scale.domain, (0.0, 1.0));
        assert_eq!(scale.colors.len(), 2);
        assert_eq!(scale.colors[0], Color::rgb(154, 192, 205)); // lightblue3
        assert_eq!(scale.colors[1], Color::rgb(0, 0, 0));   // black
    }

    #[test]
    fn test_continuous_color_interpolate() {
        let scale = ContinuousColor::new(
            (0.0, 100.0),
            vec![Color::rgb(0, 0, 0), Color::rgb(100, 100, 100)]
        );
        
        // Test interpolation at midpoint
        let mid_color = scale.map_continuous_to_color(50.0).unwrap();
        assert_eq!(mid_color, Color::rgb(50, 50, 50));
        
        // Test at endpoints
        let low_color = scale.map_continuous_to_color(0.0).unwrap();
        assert_eq!(low_color, Color::rgb(0, 0, 0));
        
        let high_color = scale.map_continuous_to_color(100.0).unwrap();
        assert_eq!(high_color, Color::rgb(100, 100, 100));
    }

    #[test]
    fn test_continuous_color_out_of_bounds() {
        let scale = ContinuousColor::new(
            (0.0, 100.0),
            vec![Color::rgb(0, 0, 255), Color::rgb(255, 0, 0)]
        );
        
        assert!(scale.map_continuous_to_color(-10.0).is_none());
        assert!(scale.map_continuous_to_color(110.0).is_none());
    }

    #[test]
    fn test_continuous_color_legend_breaks() {
        let scale = ContinuousColor::default_gradient((0.0, 100.0));
        let breaks = scale.legend_breaks();
        
        assert_eq!(breaks.len(), 3);
        assert_eq!(breaks[0], "0.00");
        assert_eq!(breaks[1], "50.00");
        assert_eq!(breaks[2], "100.00");
    }

    #[test]
    fn test_discrete_color_set_palette() {
        let mut scale = DiscreteColor::default_palette();
        let custom_palette = vec![Color(255, 255, 255, 255), Color(0, 0, 0, 255)];
        
        scale.set_palette(custom_palette.clone());
        assert_eq!(scale.palette, custom_palette);
    }
}
