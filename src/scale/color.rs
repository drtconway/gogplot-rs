use crate::data::{PrimitiveType, VectorIter};
use crate::scale::traits::{ContinuousDomainScale, DiscreteDomainScale};
use crate::theme::{Color, color};
use crate::utils::set::DiscreteSet;
use crate::visuals::palette::okabe_ito_palette;


/// Discrete color scale that maps categories to colors.
#[derive(Debug, Clone)]
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
    fn train<'a>(&mut self, iter: VectorIter<'a>) {
        self.train_discrete(iter);
    }
}

impl super::traits::DiscreteDomainScale for DiscreteColorScale {
    fn categories(&self) -> &DiscreteSet {
        &self.elements
    }

    fn add_categories(&mut self, categories: DiscreteSet) {
        self.elements.union(&categories);
    }
}

impl super::traits::ColorRangeScale for DiscreteColorScale {
    fn map_value<T: PrimitiveType>(&self, value: &T) -> Option<Color> {
        let ordinal = match value.to_primitive() {
            crate::data::PrimitiveValue::Int(x) => self.elements.ordinal(&x),
            crate::data::PrimitiveValue::Float(_) => None,
            crate::data::PrimitiveValue::Str(x) => self.elements.ordinal(&x),
            crate::data::PrimitiveValue::Bool(x) => self.elements.ordinal(&x),
        }?;
        Some(self.palette[ordinal])
    }
}

/// Continuous color scale that maps numeric values to a gradient.
#[derive(Debug, Clone)]
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
    fn train<'a>(&mut self, iter: VectorIter<'a>) {
        self.train_continuous(iter);
    }
}

impl super::traits::ContinuousDomainScale for ContinuousColorScale {
    fn domain(&self) -> Option<(f64, f64)> {
        Some(self.domain)
    }

    fn set_domain(&mut self, domain: (f64, f64)) {
        self.domain = domain;
    }

    fn limits(&self) -> (Option<f64>, Option<f64>) {
        (None, None)
    }

    fn breaks(&self) -> &[f64] {
        &[]
    }

    fn labels(&self) -> &[String] {
        &[]
    }
}

impl super::traits::ColorRangeScale for ContinuousColorScale {
    fn map_value<T: crate::data::PrimitiveType>(&self, value: &T) -> Option<Color> {
        let v = match value.to_primitive() {
            crate::data::PrimitiveValue::Int(x) => Some(x as f64),
            crate::data::PrimitiveValue::Float(x) => Some(x),
            crate::data::PrimitiveValue::Str(_) => None,
            crate::data::PrimitiveValue::Bool(_) => None,
        }?;
        let (min_domain, max_domain) = self.domain;
        if v < min_domain || v > max_domain {
            return None;
        }
        let t = (v - min_domain) / (max_domain - min_domain);
        Some(self.interpolate_color(t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discrete_color_default_palette() {
        let scale = DiscreteColorScale::default();
        assert_eq!(scale.palette.len(), 8);
    }

}
