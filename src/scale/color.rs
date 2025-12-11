use crate::data::GenericVector;
use crate::theme::{Color, color};
use crate::utils::set::DiscreteSet;
use crate::visuals::palette::{discrete_palette, okabe_ito_palette};

/// Discrete color scale that maps categories to colors.
pub struct DiscreteColorScale {
    palette: Vec<Color>,
    elements: DiscreteSet,
}

impl DiscreteColorScale {
    /// Create a new discrete color scale with a palette.
    pub fn new() -> Self {
        Self {
            palette: okabe_ito_palette(),
            elements: DiscreteSet::new(),
        }
    }

    /// Set a custom color palette.
    pub fn set_palette(&mut self, palette: Vec<Color>) {
        self.palette = palette;
    }
}

impl Default for DiscreteColorScale {
    fn default() -> Self {
        Self::new()
    }
}

impl super::traits::ScaleBase for DiscreteColorScale {
    fn scale_type(&self) -> super::ScaleType {
        super::ScaleType::Categorical
    }

    fn train(&mut self, data: &[&dyn GenericVector]) {
        for vec in data {
            if let Some(ints) = vec.iter_int() {
                for v in ints {
                    self.elements.add(&v);
                }
            } else if let Some(strs) = vec.iter_str() {
                for v in strs {
                    self.elements.add(&v);
                }
            } else if let Some(bools) = vec.iter_bool() {
                for v in bools {
                    self.elements.add(&v);
                }
            }
        }
        self.elements.build();

        let n = self.elements.len();
        if n > self.palette.len() {
            self.palette = discrete_palette(n);
        }
    }
}

impl super::traits::DiscreteColorScale for DiscreteColorScale {
    fn map_value<T: crate::data::DiscreteType>(&self, value: &T) -> Option<Color> {
        let ordinal = self.elements.ordinal(value)?;
        Some(self.palette[ordinal])
    }
}

/// Continuous color scale that maps numeric values to a gradient.
pub struct ContinuousColorScale {
    domain: (f64, f64),
    colors: Vec<Color>,
}

impl ContinuousColorScale {
    /// Create a new continuous color gradient with a list of colors.
    /// Colors are interpolated evenly across the domain.
    pub fn new(domain: (f64, f64), colors: Vec<Color>) -> Self {
        assert!(!colors.is_empty(), "Must provide at least one color");
        Self { domain, colors }
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
                color::LIGHTBLUE3, // dark blue
                color::BLACK,      // black
            ],
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
        let t = scaled - segment as f64;

        // Interpolate between the two colors in this segment
        let Color(r1, g1, b1, a1) = self.colors[segment];
        let Color(r2, g2, b2, a2) = self.colors[segment + 1];

        let r = (r1 as f64 + t * (r2 as f64 - r1 as f64)) as u8;
        let g = (g1 as f64 + t * (g2 as f64 - g1 as f64)) as u8;
        let b = (b1 as f64 + t * (b2 as f64 - b1 as f64)) as u8;
        let a = (a1 as f64 + t * (a2 as f64 - a1 as f64)) as u8;

        Color(r, g, b, a)
    }
}

impl Default for ContinuousColorScale {
    fn default() -> Self {
        Self::default_gradient((0.0, 1.0))
    }
}

impl super::traits::ScaleBase for ContinuousColorScale {
    fn scale_type(&self) -> super::ScaleType {
        super::ScaleType::Continuous
    }

    fn train(&mut self, data: &[&dyn GenericVector]) {
        use crate::data::compute_min_max;

        if let Some((min, max)) = compute_min_max(data) {
            self.domain = (min, max);
        }
    }
}

impl super::traits::ContinuousColorScale for ContinuousColorScale {
    fn domain(&self) -> Option<(f64, f64)> {
        Some(self.domain)
    }

    fn map_value<T: crate::data::ContinuousType>(&self, value: &T) -> Option<Color> {
        let v = value.to_primitive();
        if v < self.domain.0 || v > self.domain.1 {
            return None;
        }
        let t = (v - self.domain.0) / (self.domain.1 - self.domain.0);
        Some(self.interpolate_color(t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::dataframe::StrVec;

    #[test]
    fn test_discrete_color_default_palette() {
        let scale = DiscreteColorScale::default_palette();
        assert_eq!(scale.palette.len(), 8);
    }

    #[test]
    fn test_discrete_color_train() {
        let mut scale = DiscreteColorScale::default_palette();
        let data = StrVec(vec![
            "A".to_string(),
            "B".to_string(),
            "A".to_string(),
            "C".to_string(),
        ]);

        scale.train(&[&data]);

        assert_eq!(scale.mapping.len(), 3); // A, B, C
        assert!(scale.mapping.contains_key("A"));
        assert!(scale.mapping.contains_key("B"));
        assert!(scale.mapping.contains_key("C"));
    }

    #[test]
    fn test_discrete_color_map_category() {
        let mut scale = DiscreteColorScale::default_palette();
        let data = StrVec(vec![
            "red".to_string(),
            "blue".to_string(),
            "green".to_string(),
        ]);

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
        let mut scale = DiscreteColorScale::default_palette();
        let data = StrVec(vec!["Z".to_string(), "A".to_string(), "M".to_string()]);

        scale.train(&[&data]);

        let breaks = scale.legend_breaks();
        assert_eq!(breaks.len(), 3);
        assert_eq!(breaks, vec!["A", "M", "Z"]); // Should be sorted
    }

    #[test]
    fn test_continuous_color_new() {
        let scale = ContinuousColorScale::new(
            (0.0, 100.0),
            vec![Color::rgb(0, 0, 255), Color::rgb(255, 0, 0)],
        );
        assert_eq!(scale.domain, (0.0, 100.0));
        assert_eq!(scale.colors.len(), 2);
    }

    #[test]
    fn test_continuous_color_default_gradient() {
        let scale = ContinuousColorScale::default_gradient((0.0, 1.0));
        assert_eq!(scale.domain, (0.0, 1.0));
        assert_eq!(scale.colors.len(), 2);
        assert_eq!(scale.colors[0], Color::rgb(154, 192, 205)); // lightblue3
        assert_eq!(scale.colors[1], Color::rgb(0, 0, 0)); // black
    }

    #[test]
    fn test_continuous_color_interpolate() {
        let scale = ContinuousColorScale::new(
            (0.0, 100.0),
            vec![Color::rgb(0, 0, 0), Color::rgb(100, 100, 100)],
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
        let scale = ContinuousColorScale::new(
            (0.0, 100.0),
            vec![Color::rgb(0, 0, 255), Color::rgb(255, 0, 0)],
        );

        assert!(scale.map_continuous_to_color(-10.0).is_none());
        assert!(scale.map_continuous_to_color(110.0).is_none());
    }

    #[test]
    fn test_continuous_color_legend_breaks() {
        let scale = ContinuousColorScale::default_gradient((0.0, 100.0));
        let breaks = scale.legend_breaks();

        assert_eq!(breaks.len(), 3);
        assert_eq!(breaks[0], "0.00");
        assert_eq!(breaks[1], "50.00");
        assert_eq!(breaks[2], "100.00");
    }

    #[test]
    fn test_discrete_color_set_palette() {
        let mut scale = DiscreteColorScale::default_palette();
        let custom_palette = vec![Color(255, 255, 255, 255), Color(0, 0, 0, 255)];

        scale.set_palette(custom_palette.clone());
        assert_eq!(scale.palette, custom_palette);
    }
}
