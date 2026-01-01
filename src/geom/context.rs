use crate::theme;
use cairo::Context;

/// Encapsulates the rendering context needed by geoms
/// Layer handles data/mapping/grouping, geom just renders with this context
pub struct RenderContext<'a> {
    /// Cairo drawing context
    pub cairo: &'a mut Context,

    /// Theme for styling
    pub theme: &'a theme::Theme,

    /// X viewport range (min, max) in device coordinates
    pub x_range: (f64, f64),

    /// Y viewport range (min, max) in device coordinates
    pub y_range: (f64, f64),
}

impl<'a> RenderContext<'a> {
    pub fn new(
        cairo: &'a mut Context,
        theme: &'a theme::Theme,
        x_range: (f64, f64),
        y_range: (f64, f64),
    ) -> Self {
        Self {
            cairo,
            theme,
            x_range,
            y_range,
        }
    }

    /// Map normalized [0, 1] x-coordinate to viewport coordinate
    pub fn map_x(&self, normalized: f64) -> f64 {
        let (x0, x1) = self.x_range;
        x0 + normalized * (x1 - x0)
    }

    /// Map normalized [0, 1] y-coordinate to viewport coordinate
    pub fn map_y(&self, normalized: f64) -> f64 {
        let (y0, y1) = self.y_range;
        y0 + normalized * (y1 - y0)
    }

}
