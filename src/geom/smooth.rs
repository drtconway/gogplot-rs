use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::Result;
use crate::layer::Stat;

/// GeomSmooth renders fitted curves with confidence intervals
///
/// This geom combines a line (the fitted curve) and a ribbon (the confidence interval).
/// It's typically used with Stat::Smooth which computes the fitted values.
///
/// # Required Aesthetics
///
/// When using Stat::Smooth (default):
/// - X: Predictor variable
/// - Y: Response variable
///
/// The stat computes: x, y (fitted), ymin (lower CI), ymax (upper CI)
///
/// When using Stat::Identity, these are required:
/// - X: x values
/// - Y: fitted y values
/// - Ymin: lower confidence bound
/// - Ymax: upper confidence bound
///
/// # Optional Aesthetics
///
/// - Color: Line color (default: blue)
/// - Fill: Ribbon fill color (default: gray with alpha)
/// - Alpha: Transparency for ribbon (default: 0.4)
/// - Size: Line width (default: 1.0)
///
/// # Examples
///
/// ```rust,ignore
/// // Basic smooth with linear regression
/// Plot::new(data)
///     .aes(|a| { a.x("x"); a.y("y"); })
///     .geom_point()
///     .geom_smooth()
///     .save("smooth.png", 800, 600)?;
///
/// // Smooth per group with different colors
/// Plot::new(data)
///     .aes(|a| { 
///         a.x("x"); 
///         a.y("y"); 
///         a.color("group");
///     })
///     .geom_point()
///     .geom_smooth()
///     .save("smooth_grouped.png", 800, 600)?;
///
/// // Smooth without confidence interval
/// Plot::new(data)
///     .aes(|a| { a.x("x"); a.y("y"); })
///     .geom_smooth_with(|layer| {
///         layer.geom.se(false);
///     })
///     .save("smooth_no_se.png", 800, 600)?;
/// ```
pub struct GeomSmooth {
    /// Default line color (if not mapped)
    pub color: Option<AesValue>,

    /// Default fill color for confidence ribbon (if not mapped)
    pub fill: Option<AesValue>,

    /// Default alpha/opacity for ribbon
    pub alpha: Option<AesValue>,

    /// Default line width
    pub size: Option<AesValue>,

    /// Whether to show confidence interval
    pub se: bool,

    /// Stat to use (default is Smooth)
    pub stat: Stat,

    /// Position adjustment (typically Identity)
    pub position: crate::layer::Position,
}

impl GeomSmooth {
    /// Create a new smooth geom with default settings
    pub fn new() -> Self {
        Self {
            color: Some(AesValue::Constant(PrimitiveValue::Int(
                crate::theme::Color::rgb(0, 114, 178).into(), // Blue
            ))),
            fill: Some(AesValue::Constant(PrimitiveValue::Int(
                crate::theme::Color::rgb(128, 128, 128).into(), // Gray
            ))),
            alpha: Some(AesValue::Constant(PrimitiveValue::Float(0.4))),
            size: Some(AesValue::Constant(PrimitiveValue::Float(1.0))),
            se: true,
            stat: Stat::Smooth {
                method: crate::stat::smooth::Method::Lm,
                level: 0.95,
                n: 80,
            },
            position: crate::layer::Position::Identity,
        }
    }

    /// Set the line color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        self.color = Some(AesValue::Constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the ribbon fill color
    pub fn fill(&mut self, color: crate::theme::Color) -> &mut Self {
        self.fill = Some(AesValue::Constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the ribbon alpha/opacity
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = Some(AesValue::Constant(PrimitiveValue::Float(
            alpha.clamp(0.0, 1.0),
        )));
        self
    }

    /// Set the line width
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size = Some(AesValue::Constant(PrimitiveValue::Float(size.max(0.0))));
        self
    }

    /// Set whether to show confidence interval
    pub fn se(&mut self, se: bool) -> &mut Self {
        self.se = se;
        self
    }

    /// Set the smoothing method
    pub fn method(&mut self, method: crate::stat::smooth::Method) -> &mut Self {
        if let Stat::Smooth { method: m, .. } = &mut self.stat {
            *m = method;
        }
        self
    }

    /// Set the stat to use
    pub fn stat(&mut self, stat: Stat) -> &mut Self {
        self.stat = stat;
        self
    }

    /// Set the position adjustment
    pub fn position(&mut self, position: crate::layer::Position) -> &mut Self {
        self.position = position;
        self
    }
}

impl Default for GeomSmooth {
    fn default() -> Self {
        Self::new()
    }
}

impl Geom for GeomSmooth {
    fn required_aesthetics(&self) -> &[Aesthetic] {
        &[Aesthetic::X, Aesthetic::Y]
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<()> {
        // Get aesthetic values
        let x_values = ctx.get_x_aesthetic_values(Aesthetic::X)?;
        let y_values = ctx.get_y_aesthetic_values(Aesthetic::Y)?;
        
        // Collect into vectors for easier processing
        let x_vec: Vec<f64> = x_values.collect();
        let y_vec: Vec<f64> = y_values.collect();
        
        if x_vec.is_empty() {
            return Ok(());
        }

        // Get styling aesthetics
        let colors = ctx.get_color_values()?;
        let fills = ctx.get_fill_color_values()?;
        let alphas = ctx.get_unscaled_aesthetic_values(Aesthetic::Alpha)?;
        let sizes = ctx.get_unscaled_aesthetic_values(Aesthetic::Size)?;
        
        let colors_vec: Vec<_> = colors.collect();
        let fills_vec: Vec<_> = fills.collect();
        let alphas_vec: Vec<_> = alphas.collect();
        let sizes_vec: Vec<_> = sizes.collect();
        
        // Get first values for styling (all points should have same style for smooth)
        let color = colors_vec.first().copied().unwrap_or(crate::theme::Color::rgb(0, 114, 178));
        let fill = fills_vec.first().copied().unwrap_or(crate::theme::Color::rgb(128, 128, 128));
        let alpha = alphas_vec.first().copied().unwrap_or(0.4);
        let size = sizes_vec.first().copied().unwrap_or(1.0);

        // Draw confidence ribbon if se is enabled
        if self.se {
            let ymin_values = ctx.get_y_aesthetic_values(Aesthetic::Ymin)?;
            let ymax_values = ctx.get_y_aesthetic_values(Aesthetic::Ymax)?;
            
            let ymin_vec: Vec<f64> = ymin_values.collect();
            let ymax_vec: Vec<f64> = ymax_values.collect();
            
            if ymin_vec.len() == x_vec.len() && ymax_vec.len() == x_vec.len() {
                // Draw ribbon as a filled polygon
                ctx.set_color_alpha(&fill, alpha);
                
                // Start path at first point on lower edge
                let x0_visual = ctx.map_x(x_vec[0]);
                let ymin0_visual = ctx.map_y(ymin_vec[0]);
                ctx.cairo.move_to(x0_visual, ymin0_visual);
                
                // Draw lower edge (left to right)
                for i in 1..x_vec.len() {
                    let x_visual = ctx.map_x(x_vec[i]);
                    let ymin_visual = ctx.map_y(ymin_vec[i]);
                    ctx.cairo.line_to(x_visual, ymin_visual);
                }
                
                // Draw upper edge (right to left)
                for i in (0..x_vec.len()).rev() {
                    let x_visual = ctx.map_x(x_vec[i]);
                    let ymax_visual = ctx.map_y(ymax_vec[i]);
                    ctx.cairo.line_to(x_visual, ymax_visual);
                }
                
                // Close path and fill
                ctx.cairo.close_path();
                ctx.cairo.fill().ok();
            }
        }

        // Draw the line on top of the ribbon
        ctx.cairo.set_line_width(size);
        ctx.set_color_alpha(&color, 1.0);  // Line is always opaque
        
        if !x_vec.is_empty() {
            let x0_visual = ctx.map_x(x_vec[0]);
            let y0_visual = ctx.map_y(y_vec[0]);
            ctx.cairo.move_to(x0_visual, y0_visual);
            
            for i in 1..x_vec.len() {
                let x_visual = ctx.map_x(x_vec[i]);
                let y_visual = ctx.map_y(y_vec[i]);
                ctx.cairo.line_to(x_visual, y_visual);
            }
            
            ctx.cairo.stroke().ok();
        }

        Ok(())
    }
}

impl IntoLayer for GeomSmooth {
    fn default_aesthetics(&self) -> Vec<(Aesthetic, AesValue)> {
        let mut defaults = Vec::new();
        
        if let Some(color) = &self.color {
            defaults.push((Aesthetic::Color, color.clone()));
        }
        if let Some(fill) = &self.fill {
            defaults.push((Aesthetic::Fill, fill.clone()));
        }
        if let Some(alpha) = &self.alpha {
            defaults.push((Aesthetic::Alpha, alpha.clone()));
        }
        if let Some(size) = &self.size {
            defaults.push((Aesthetic::Size, size.clone()));
        }
        
        defaults
    }

    fn into_layer(self) -> crate::layer::Layer
    where
        Self: Geom + 'static,
    {
        let mut mapping = crate::aesthetics::AesMap::new();

        // Set default aesthetics from geom settings if provided
        for (aesthetic, value) in self.default_aesthetics() {
            mapping.set(aesthetic, value);
        }

        // Get stat and position before consuming self
        let stat = self.stat.clone();
        let position = self.position.clone();

        crate::layer::Layer {
            geom: Box::new(self),
            data: None,
            mapping,
            stat,
            position,
            computed_data: None,
            computed_mapping: None,
            computed_scales: None,
        }
    }
}

impl Clone for GeomSmooth {
    fn clone(&self) -> Self {
        Self {
            color: self.color.clone(),
            fill: self.fill.clone(),
            alpha: self.alpha.clone(),
            size: self.size.clone(),
            se: self.se,
            stat: self.stat.clone(),
            position: self.position.clone(),
        }
    }
}
