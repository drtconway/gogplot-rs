use super::{Geom, RenderContext};
use crate::aesthetics::{Aesthetic, AesMap, AesValue};
use crate::data::PrimitiveValue;
use crate::error::PlotError;
use crate::layer::{Layer, Stat, Position};

/// Point shape options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointShape {
    Circle,
    Square,
    Triangle,
    Diamond,
    Cross,
    Plus,
}

/// GeomPoint renders points/scatterplot
pub struct GeomPoint {
    /// Default point size (if not mapped)
    pub size: Option<AesValue>,

    /// Default point color (if not mapped)
    pub color: Option<AesValue>,

    /// Default point shape (if not mapped)
    pub shape: Option<AesValue>,

    /// Default alpha/opacity (if not mapped)
    pub alpha: Option<AesValue>,
}

impl GeomPoint {
    /// Create a new point geom with default settings
    pub fn new() -> Self {
        Self {
            size: None,
            color: None,
            shape: None,
            alpha: None,
        }
    }

    /// Set the default point size
    pub fn size(mut self, size: f64) -> Self {
        self.size = Some(AesValue::Constant(PrimitiveValue::Float(size)));
        self
    }

    /// Set the default point color
    pub fn color(mut self, color: crate::theme::Color) -> Self {
        let rgba = ((color.0 as i64) << 24) | ((color.1 as i64) << 16) | ((color.2 as i64) << 8) | (color.3 as i64);
        self.color = Some(AesValue::Constant(PrimitiveValue::Int(rgba)));
        self
    }

    /// Set the default point shape
    pub fn shape(mut self, shape: PointShape) -> Self {
        self.shape = Some(AesValue::Constant(PrimitiveValue::Int(shape as i64)));
        self
    }

    /// Set the default alpha/opacity
    pub fn alpha(mut self, alpha: f64) -> Self {
        self.alpha = Some(AesValue::Constant(PrimitiveValue::Float(alpha.clamp(0.0, 1.0))));
        self
    }

    /// Create a Layer with this geom and default aesthetics
    pub fn into_layer(self) -> Layer {
        let mut mapping = AesMap::new();
        
        // Set default aesthetics from geom settings if provided
        if let Some(color) = &self.color {
            mapping.set(Aesthetic::Color, color.clone());
        }
        if let Some(alpha) = &self.alpha {
            mapping.set(Aesthetic::Alpha, alpha.clone());
        }
        if let Some(size) = &self.size {
            mapping.set(Aesthetic::Size, size.clone());
        }
        if let Some(shape) = &self.shape {
            mapping.set(Aesthetic::Shape, shape.clone());
        }
        
        Layer {
            geom: Box::new(self),
            data: None,
            mapping,
            stat: Stat::Identity,
            position: Position::Identity,
        }
    }
}

impl Default for GeomPoint {
    fn default() -> Self {
        Self::new()
    }
}

impl Geom for GeomPoint {
    fn required_aesthetics(&self) -> &[Aesthetic] {
        &[Aesthetic::X, Aesthetic::Y]
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        // Get all aesthetic iterators (constants use lazy repeat iterators)
        let x_normalized = ctx.get_aesthetic_values(Aesthetic::X, ctx.scales.x.as_ref())?;
        let y_normalized = ctx.get_aesthetic_values(Aesthetic::Y, ctx.scales.y.as_ref())?;
        let colors = ctx.get_color_values()?;
        let alphas = ctx.get_aesthetic_values(Aesthetic::Alpha, None)?;
        let sizes = ctx.get_aesthetic_values(Aesthetic::Size, None)?;
        let shapes = ctx.get_shape_values()?;

        // Zip all iterators together
        let iter = x_normalized
            .zip(y_normalized)
            .zip(colors)
            .zip(alphas)
            .zip(sizes)
            .zip(shapes);

        for (((((x_norm, y_norm), color), alpha), size), shape) in iter {
            let x_visual = ctx.map_x(x_norm);
            let y_visual = ctx.map_y(y_norm);

            // Set drawing properties for this point
            ctx.set_color_alpha(&color, alpha);

            // Draw the point based on shape
            match shape {
                PointShape::Circle => {
                    ctx.cairo.arc(
                        x_visual,
                        y_visual,
                        size,
                        0.0,
                        2.0 * std::f64::consts::PI,
                    );
                    ctx.cairo.fill().ok();
                }
                PointShape::Square => {
                    let half_size = size;
                    ctx.cairo.rectangle(
                        x_visual - half_size,
                        y_visual - half_size,
                        2.0 * half_size,
                        2.0 * half_size,
                    );
                    ctx.cairo.fill().ok();
                }
                // Other shapes would be implemented here
                _ => {
                    // Default to circle for unimplemented shapes
                    ctx.cairo.arc(
                        x_visual,
                        y_visual,
                        size,
                        0.0,
                        2.0 * std::f64::consts::PI,
                    );
                    ctx.cairo.fill().ok();
                }
            }
        }

        Ok(())
    }
}
