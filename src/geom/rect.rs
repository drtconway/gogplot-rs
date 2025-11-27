use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::PlotError;

/// GeomRect renders rectangles defined by xmin, xmax, ymin, ymax
pub struct GeomRect {
    /// Default fill color (if not mapped)
    pub fill: Option<AesValue>,

    /// Default stroke color (if not mapped)
    pub color: Option<AesValue>,

    /// Default alpha/opacity (if not mapped)
    pub alpha: Option<AesValue>,
}

impl GeomRect {
    /// Create a new rect geom with default settings
    pub fn new() -> Self {
        Self {
            fill: None,
            color: None,
            alpha: None,
        }
    }

    /// Set the default fill color
    pub fn fill(&mut self, color: crate::theme::Color) -> &mut Self {
        let rgba = color.into();
        self.fill = Some(AesValue::constant(PrimitiveValue::Int(rgba)));
        self
    }

    /// Set the default stroke color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        let rgba = color.into();
        self.color = Some(AesValue::constant(PrimitiveValue::Int(rgba)));
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

impl Default for GeomRect {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoLayer for GeomRect {
    fn default_aesthetics(&self) -> Vec<(Aesthetic, AesValue)> {
        let mut defaults = Vec::new();

        if let Some(fill) = &self.fill {
            defaults.push((Aesthetic::Fill, fill.clone()));
        }
        if let Some(color) = &self.color {
            defaults.push((Aesthetic::Color, color.clone()));
        }
        if let Some(alpha) = &self.alpha {
            defaults.push((Aesthetic::Alpha, alpha.clone()));
        }

        defaults
    }
}

impl Geom for GeomRect {
    fn required_aesthetics(&self) -> &[Aesthetic] {
        &[
            Aesthetic::XBegin,
            Aesthetic::XEnd,
            Aesthetic::YBegin,
            Aesthetic::YEnd,
        ]
    }

    fn setup_data(
        &self,
        _data: &dyn crate::data::DataSource,
        _mapping: &crate::aesthetics::AesMap,
    ) -> Result<(Option<Box<dyn crate::data::DataSource>>, Option<crate::aesthetics::AesMap>), PlotError> {
        // Rect geom doesn't need to add any columns - it uses XBegin, XEnd, YBegin, YEnd directly
        Ok((None, None))
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        // Get all aesthetic iterators
        let x_begin_normalized =
            ctx.get_aesthetic_values(Aesthetic::XBegin, ctx.scales.x.as_deref())?;
        let x_end_normalized = ctx.get_aesthetic_values(Aesthetic::XEnd, ctx.scales.x.as_deref())?;
        let y_begin_normalized =
            ctx.get_aesthetic_values(Aesthetic::YBegin, ctx.scales.y.as_deref())?;
        let y_end_normalized = ctx.get_aesthetic_values(Aesthetic::YEnd, ctx.scales.y.as_deref())?;
        let fills = ctx.get_fill_color_values()?;
        let colors = ctx.get_color_values()?;
        let alphas = ctx.get_aesthetic_values(Aesthetic::Alpha, None)?;

        // Zip all iterators together
        let iter = x_begin_normalized
            .zip(x_end_normalized)
            .zip(y_begin_normalized)
            .zip(y_end_normalized)
            .zip(fills)
            .zip(colors)
            .zip(alphas);

        for ((((((x_begin, x_end), y_begin), y_end), fill), color), alpha) in iter {
            let x1 = ctx.map_x(x_begin);
            let x2 = ctx.map_x(x_end);
            let y1 = ctx.map_y(y_begin);
            let y2 = ctx.map_y(y_end);

            let width = (x2 - x1).abs();
            let height = (y2 - y1).abs();
            let x = x1.min(x2);
            let y = y1.min(y2);

            // Fill the rectangle
            ctx.set_color_alpha(&fill, alpha);
            ctx.cairo.rectangle(x, y, width, height);
            ctx.cairo.fill().ok();

            // Stroke the rectangle if a stroke color is defined
            // Only stroke if the color is different from fill or explicitly set
            if self.color.is_some() {
                ctx.set_color_alpha(&color, alpha);
                ctx.cairo.rectangle(x, y, width, height);
                ctx.cairo.stroke().ok();
            }
        }

        Ok(())
    }
}
