use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::PlotError;

/// Geometry for drawing line segments.
///
/// Draws line segments from (x, y) to (xend, yend). Each segment can have
/// its own color, alpha, and size (line width).
///
/// # Required Aesthetics
///
/// - `X`: Starting x coordinate
/// - `Y`: Starting y coordinate  
/// - `XEnd`: Ending x coordinate
/// - `YEnd`: Ending y coordinate
///
/// # Optional Aesthetics
///
/// - `Color`: Line color (can be constant or mapped to data)
/// - `Alpha`: Line transparency (0.0 = transparent, 1.0 = opaque)
/// - `Size`: Line width in pixels
/// - `Linetype`: Line style pattern (e.g., "-", ".", "-.", etc.)
pub struct GeomSegment {
    /// Default color (if not mapped)
    pub color: Option<AesValue>,

    /// Default alpha/opacity (if not mapped)
    pub alpha: Option<AesValue>,

    /// Default line width (if not mapped)
    pub size: Option<AesValue>,

    /// Default line style pattern (if not mapped)
    pub linetype: Option<AesValue>,
}

impl GeomSegment {
    /// Create a new segment geom with default settings
    pub fn new() -> Self {
        Self {
            color: None,
            alpha: None,
            size: None,
            linetype: None,
        }
    }

    /// Set a constant color for all segments
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        let rgba = color.into();
        self.color = Some(AesValue::constant(PrimitiveValue::Int(rgba)));
        self
    }

    /// Set a constant alpha (transparency) for all segments
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = Some(AesValue::constant(PrimitiveValue::Float(alpha)));
        self
    }

    /// Set a constant size (line width) for all segments
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size = Some(AesValue::constant(PrimitiveValue::Float(size)));
        self
    }

    /// Set the default line style pattern
    ///
    /// Pattern characters:
    /// - `-` : dash
    /// - `.` : dot
    /// - ` ` : long gap
    ///
    /// Examples: `"-"`, `"."`, `"-."`, `"- -"`, `". ."`
    pub fn linetype(&mut self, pattern: impl Into<String>) -> &mut Self {
        self.linetype = Some(AesValue::constant(PrimitiveValue::Str(pattern.into())));
        self
    }
}

impl Default for GeomSegment {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoLayer for GeomSegment {
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
        if let Some(linetype) = &self.linetype {
            defaults.push((Aesthetic::Linetype, linetype.clone()));
        }

        defaults
    }
}

impl Geom for GeomSegment {
    fn train_scales(&self, _scales: &mut crate::scale::ScaleSet) {
        
    }

    fn apply_scales(&mut self, scales: &crate::scale::ScaleSet) {
        
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        use crate::visuals::LineStyle;

        // Get all aesthetic iterators
        let x_normalized = ctx.get_x_aesthetic_values(Aesthetic::X)?;
        let y_normalized = ctx.get_y_aesthetic_values(Aesthetic::Y)?;
        let xend_normalized = ctx.get_x_aesthetic_values(Aesthetic::XEnd)?;
        let yend_normalized = ctx.get_y_aesthetic_values(Aesthetic::YEnd)?;
        let colors = ctx.get_color_values()?;
        let alphas = ctx.get_unscaled_aesthetic_values(Aesthetic::Alpha)?;
        let sizes = ctx.get_unscaled_aesthetic_values(Aesthetic::Size)?;

        // Get constant linetype if set
        let constant_linetype = if let Some(AesValue::Constant { value: PrimitiveValue::Str(pattern), .. }) =
            ctx.mapping().get(&Aesthetic::Linetype)
        {
            Some(pattern.clone())
        } else {
            None
        };

        // Collect linetype column values if mapped
        let linetype_vec =
            if let Some(AesValue::Column { name: col, .. }) = ctx.mapping().get(&Aesthetic::Linetype) {
                let vec = ctx
                    .data()
                    .get(col.as_str())
                    .ok_or_else(|| PlotError::missing_column(col))?;
                vec.iter_str()
                    .ok_or_else(|| PlotError::invalid_column_type(col, "string"))?
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            } else {
                Vec::new()
            };

        // Zip all iterators together
        let iter = x_normalized
            .zip(y_normalized)
            .zip(xend_normalized)
            .zip(yend_normalized)
            .zip(colors)
            .zip(alphas)
            .zip(sizes);

        for (i, ((((((x1_norm, y1_norm), x2_norm), y2_norm), color), alpha), size)) in
            iter.enumerate()
        {
            let x1 = ctx.map_x(x1_norm);
            let y1 = ctx.map_y(y1_norm);
            let x2 = ctx.map_x(x2_norm);
            let y2 = ctx.map_y(y2_norm);

            // Set drawing properties
            ctx.set_color_alpha(&color, alpha);
            ctx.cairo.set_line_width(size);

            // Apply line style
            let pattern = if !linetype_vec.is_empty() {
                linetype_vec.get(i).map(|s| s.as_str())
            } else {
                constant_linetype.as_deref()
            };

            if let Some(p) = pattern {
                let style = LineStyle::from(p);
                style.apply(ctx.cairo);
            } else {
                LineStyle::default().apply(ctx.cairo);
            }

            // Draw the segment
            ctx.cairo.move_to(x1, y1);
            ctx.cairo.line_to(x2, y2);
            ctx.cairo.stroke().ok();
        }

        Ok(())
    }
}
