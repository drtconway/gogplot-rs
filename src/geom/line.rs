use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::{DataType, PlotError};

/// GeomLine renders lines connecting points
pub struct GeomLine {
    /// Default line color (if not mapped)
    pub color: Option<AesValue>,

    /// Default line width (if not mapped)
    pub size: Option<AesValue>,

    /// Default alpha/opacity (if not mapped)
    pub alpha: Option<AesValue>,

    /// Default line style pattern (if not mapped)
    pub linetype: Option<AesValue>,
}

impl GeomLine {
    /// Create a new line geom with default settings
    pub fn new() -> Self {
        Self {
            color: None,
            size: None,
            alpha: None,
            linetype: None,
        }
    }

    /// Set the default line color
    pub fn color(mut self, color: crate::theme::Color) -> Self {
        let rgba = color.into();
        self.color = Some(AesValue::Constant(PrimitiveValue::Int(rgba)));
        self
    }

    /// Set the default line width
    pub fn size(mut self, size: f64) -> Self {
        self.size = Some(AesValue::Constant(PrimitiveValue::Float(size)));
        self
    }

    /// Set the default alpha/opacity
    pub fn alpha(mut self, alpha: f64) -> Self {
        self.alpha = Some(AesValue::Constant(PrimitiveValue::Float(
            alpha.clamp(0.0, 1.0),
        )));
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
    pub fn linetype(mut self, pattern: impl Into<String>) -> Self {
        self.linetype = Some(AesValue::Constant(PrimitiveValue::Str(pattern.into())));
        self
    }
}

impl Default for GeomLine {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoLayer for GeomLine {
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

impl Geom for GeomLine {
    fn required_aesthetics(&self) -> &[Aesthetic] {
        &[Aesthetic::X, Aesthetic::Y]
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        use crate::data::VectorType;

        // Get x and y values
        let x_normalized = ctx.get_aesthetic_values(Aesthetic::X, ctx.scales.x.as_deref())?;
        let y_normalized = ctx.get_aesthetic_values(Aesthetic::Y, ctx.scales.y.as_deref())?;

        // Collect into vectors for sorting
        let x_vals: Vec<f64> = x_normalized.collect();
        let y_vals: Vec<f64> = y_normalized.collect();

        // Check if we have a group aesthetic
        let has_group = ctx.mapping.get(&Aesthetic::Group).is_some();

        if has_group {
            // Get group values
            let group_col = match ctx.mapping.get(&Aesthetic::Group) {
                Some(AesValue::Column(col)) => col,
                _ => {
                    return Err(PlotError::MissingAesthetic {
                        aesthetic: Aesthetic::Group,
                    });
                }
            };

            let group_vec = ctx
                .data
                .get(group_col.as_str())
                .ok_or_else(|| PlotError::missing_column(group_col))?;

            // Group strings together
            let groups = match group_vec.vtype() {
                VectorType::Str => group_vec.iter_str().ok_or_else(|| {
                    PlotError::InvalidAestheticType {
                        aesthetic: Aesthetic::Group,
                        expected: DataType::Vector(VectorType::Str),
                        actual: DataType::Custom("unknown".to_string()),
                    }
                })?,
                _ => {
                    return Err(PlotError::InvalidAestheticType {
                        aesthetic: Aesthetic::Group,
                        expected: DataType::Vector(VectorType::Str),
                        actual: DataType::Custom("non-string".to_string()),
                    });
                }
            };

            // Organize points by group
            use std::collections::HashMap;
            let mut grouped_points: HashMap<String, Vec<(f64, f64, usize)>> = HashMap::new();

            for (i, group) in groups.enumerate() {
                grouped_points
                    .entry(group.to_string())
                    .or_default()
                    .push((x_vals[i], y_vals[i], i));
            }

            // Draw each group separately
            for (_group_name, mut points) in grouped_points {
                // Sort by x value
                points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

                self.draw_line_segment(ctx, &points)?;
            }
        } else {
            // No grouping - create a single sorted line
            let mut points: Vec<(f64, f64, usize)> = x_vals
                .iter()
                .zip(y_vals.iter())
                .enumerate()
                .map(|(i, (&x, &y))| (x, y, i))
                .collect();

            // Sort by x value
            points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

            self.draw_line_segment(ctx, &points)?;
        }

        Ok(())
    }
}

impl GeomLine {
    /// Draw a connected line through a sequence of points
    fn draw_line_segment(
        &self,
        ctx: &mut RenderContext,
        points: &[(f64, f64, usize)],
    ) -> Result<(), PlotError> {
        if points.is_empty() {
            return Ok(());
        }

        // Get color and alpha for the line (use first point's values)
        let colors = ctx.get_color_values()?;
        let alphas = ctx.get_aesthetic_values(Aesthetic::Alpha, None)?;
        let sizes = ctx.get_aesthetic_values(Aesthetic::Size, None)?;

        let colors_vec: Vec<_> = colors.collect();
        let alphas_vec: Vec<_> = alphas.collect();
        let sizes_vec: Vec<_> = sizes.collect();

        // Get linetype values if mapped
        let linetype_pattern =
            if let Some(AesValue::Column(col)) = ctx.mapping.get(&Aesthetic::Linetype) {
                // Get the string value from the data
                let linetype_vec = ctx
                    .data
                    .get(col.as_str())
                    .ok_or_else(|| PlotError::missing_column(col))?;
                if let Some(mut strs) = linetype_vec.iter_str() {
                    let idx = points[0].2;
                    Some(
                        strs
                            .nth(idx)
                            .map(|s| s.to_string())
                            .unwrap_or_default(),
                    )
                } else {
                    None
                }
            } else if let Some(AesValue::Constant(PrimitiveValue::Str(pattern))) =
                ctx.mapping.get(&Aesthetic::Linetype)
            {
                Some(pattern.clone())
            } else {
                None
            };

        // Use the first point's color/alpha/size for the entire line
        let idx = points[0].2;
        let color = &colors_vec[idx];
        let alpha = alphas_vec[idx];
        let size = sizes_vec[idx];

        // Set drawing properties
        ctx.set_color_alpha(color, alpha);
        ctx.cairo.set_line_width(size);

        // Apply line style
        use crate::visuals::LineStyle;
        if let Some(pattern) = linetype_pattern {
            let style = LineStyle::from(pattern.as_str());
            style.apply(ctx.cairo);
        } else {
            LineStyle::default().apply(ctx.cairo);
        }

        // Start path at first point
        let (x0, y0, _) = points[0];
        let x0_visual = ctx.map_x(x0);
        let y0_visual = ctx.map_y(y0);
        ctx.cairo.move_to(x0_visual, y0_visual);

        // Draw lines to subsequent points
        for &(x, y, _) in &points[1..] {
            let x_visual = ctx.map_x(x);
            let y_visual = ctx.map_y(y);
            ctx.cairo.line_to(x_visual, y_visual);
        }

        ctx.cairo.stroke().ok();

        Ok(())
    }
}
