use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::PlotError;
use crate::visuals::Shape;

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
    /// Create a new point geom with default settings from theme
    pub fn new() -> Self {
        Self {
            size: None,
            color: None,
            shape: None,
            alpha: None,
        }
    }

    /// Set the default point size
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size = Some(AesValue::constant(PrimitiveValue::Float(size)));
        self
    }

    /// Set the default point color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        self.color = Some(AesValue::constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the default point shape
    pub fn shape(&mut self, shape: Shape) -> &mut Self {
        self.shape = Some(AesValue::constant(PrimitiveValue::Int(shape as i64)));
        self
    }

    /// Set the default alpha/opacity
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = Some(AesValue::constant(PrimitiveValue::Float(
            alpha.clamp(0.0, 1.0),
        )));
        self
    }
}

impl Default for GeomPoint {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoLayer for GeomPoint {
    fn default_aesthetics(&self) -> Vec<(Aesthetic, AesValue)> {
        let mut defaults = Vec::new();

        if let Some(color) = &self.color {
            defaults.push((Aesthetic::Color, color.clone()));
        }
        
        if let Some(alpha) = &self.alpha {
            defaults.push((Aesthetic::Alpha, alpha.clone()));
        }
        
        if let Some(size) = &self.size {
            defaults.push((Aesthetic::Size, size.clone()));
        }
        
        if let Some(shape) = &self.shape {
            defaults.push((Aesthetic::Shape, shape.clone()));
        }

        defaults
    }
}

impl Geom for GeomPoint {
    fn required_aesthetics(&self) -> &[Aesthetic] {
        &[Aesthetic::X, Aesthetic::Y]
    }

    fn setup_data(
        &self,
        _data: &dyn crate::data::DataSource,
        _mapping: &crate::aesthetics::AesMap,
    ) -> Result<(Option<Box<dyn crate::data::DataSource>>, Option<crate::aesthetics::AesMap>), PlotError> {
        // Geom doesn't need to add any columns
        Ok((None, None))
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        // Get all aesthetic iterators (constants use lazy repeat iterators)
        let x_normalized = ctx.get_x_aesthetic_values(Aesthetic::X)?;
        let y_normalized = ctx.get_y_aesthetic_values(Aesthetic::Y)?;
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

            // Draw the shape
            shape.draw(ctx.cairo, x_visual, y_visual, size);
        }

        Ok(())
    }
}
