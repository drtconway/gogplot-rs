use super::{Geom, RenderContext};
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

impl Geom for GeomSegment {
    fn train_scales(&self, _scales: &mut crate::scale::ScaleSet) {
        
    }

    fn apply_scales(&mut self, scales: &crate::scale::ScaleSet) {
        
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        Ok(())
    }
}
