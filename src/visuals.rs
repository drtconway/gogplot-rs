// Visual elements and symbols shared across geoms and guides

use cairo::Context;

/// Line style patterns for line geoms
#[derive(Clone, Debug, PartialEq)]
pub enum LineStyle {
    /// Solid line (no dashing)
    Solid,
    /// Custom dash pattern specified as array of on/off lengths
    Custom(Vec<f64>),
}

impl Default for LineStyle {
    fn default() -> Self {
        LineStyle::Solid
    }
}

impl From<&str> for LineStyle {
    /// Create a LineStyle from a pattern string.
    ///
    /// Pattern characters:
    /// - `-` : dash (5 units on, 2 units off)
    /// - `.` : dot (1 unit on, 2 units off)
    /// - ` ` : long gap (5 units off)
    ///
    /// The pattern repeats. Examples:
    /// - `"-"` : dashed line
    /// - `"."` : dotted line
    /// - `"-."` : dash-dot pattern
    /// - `"- -"` : dash with long gaps
    /// - `". ."` : dots with long gaps
    fn from(pattern: &str) -> Self {
        if pattern.is_empty() {
            return LineStyle::Solid;
        }

        let mut dashes = Vec::new();

        for ch in pattern.chars() {
            match ch {
                '-' => {
                    dashes.push(5.0); // dash on
                    dashes.push(2.0); // gap after dash
                }
                '.' => {
                    dashes.push(1.0); // dot on
                    dashes.push(2.0); // gap after dot
                }
                ' ' => {
                    dashes.push(5.0); // long gap
                }
                _ => {} // ignore other characters
            }
        }

        if dashes.is_empty() {
            LineStyle::Solid
        } else {
            LineStyle::Custom(dashes)
        }
    }
}

impl LineStyle {
    /// Apply this line style to a Cairo context
    pub fn apply(&self, ctx: &mut Context) {
        match self {
            LineStyle::Solid => {
                ctx.set_dash(&[], 0.0);
            }
            LineStyle::Custom(dashes) => {
                ctx.set_dash(dashes, 0.0);
            }
        }
    }
}

/// Shape types for points and legend symbols
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Circle,
    Square,
    Triangle,
    Diamond,
    Cross,
    Plus,
}

impl Shape {
    /// Draw this shape at the given position with the given size
    ///
    /// # Arguments
    /// * `ctx` - Cairo drawing context
    /// * `x` - X coordinate of center
    /// * `y` - Y coordinate of center
    /// * `size` - Size (radius) of the shape
    pub fn draw(&self, ctx: &mut Context, x: f64, y: f64, size: f64) {
        match self {
            Shape::Circle => {
                ctx.arc(x, y, size, 0.0, 2.0 * std::f64::consts::PI);
                ctx.fill().ok();
            }
            Shape::Square => {
                ctx.rectangle(x - size, y - size, size * 2.0, size * 2.0);
                ctx.fill().ok();
            }
            Shape::Triangle => {
                let h = size * 1.732; // sqrt(3)
                ctx.move_to(x, y - h * 0.577);
                ctx.line_to(x - size, y + h * 0.289);
                ctx.line_to(x + size, y + h * 0.289);
                ctx.close_path();
                ctx.fill().ok();
            }
            Shape::Diamond => {
                ctx.move_to(x, y - size);
                ctx.line_to(x + size, y);
                ctx.line_to(x, y + size);
                ctx.line_to(x - size, y);
                ctx.close_path();
                ctx.fill().ok();
            }
            Shape::Cross => {
                let width = size * 0.3;
                ctx.set_line_width(width);
                ctx.move_to(x - size, y - size);
                ctx.line_to(x + size, y + size);
                ctx.stroke().ok();
                ctx.move_to(x - size, y + size);
                ctx.line_to(x + size, y - size);
                ctx.stroke().ok();
            }
            Shape::Plus => {
                let width = size * 0.3;
                ctx.set_line_width(width);
                ctx.move_to(x - size, y);
                ctx.line_to(x + size, y);
                ctx.stroke().ok();
                ctx.move_to(x, y - size);
                ctx.line_to(x, y + size);
                ctx.stroke().ok();
            }
        }
    }
}
