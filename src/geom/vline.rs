use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::PlotError;
use crate::layer::{Position, Stat};

/// GeomVLine renders vertical reference lines at specified x-intercepts
///
/// The x-intercept is specified via the XIntercept aesthetic mapping.
pub struct GeomVLine {
    /// Default line color
    pub color: Option<AesValue>,

    /// Default line width
    pub size: Option<AesValue>,

    /// Default alpha/opacity
    pub alpha: Option<AesValue>,

    /// Default line style pattern
    pub linetype: Option<AesValue>,

    /// The stat to use (default is Identity)
    pub stat: Stat,

    /// The position adjustment (default is Identity)
    pub position: Position,
}

impl GeomVLine {
    /// Create a new vertical line geom
    ///
    /// X-intercept should be specified via aesthetic mapping:
    /// - Constant: `.aes(|a| a.xintercept_const(value))`
    /// - Column: `.aes(|a| a.xintercept("column_name"))`
    pub fn new() -> Self {
        Self {
            color: None,
            size: None,
            alpha: None,
            linetype: None,
            stat: Stat::Identity,
            position: Position::Identity,
        }
    }

    /// Set the line color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        let rgba = color.into();
        self.color = Some(AesValue::Constant(PrimitiveValue::Int(rgba)));
        self
    }

    /// Set the line width
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size = Some(AesValue::Constant(PrimitiveValue::Float(size)));
        self
    }

    /// Set the alpha/opacity
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = Some(AesValue::Constant(PrimitiveValue::Float(
            alpha.clamp(0.0, 1.0),
        )));
        self
    }

    /// Set the line style pattern
    pub fn linetype(&mut self, pattern: impl Into<String>) -> &mut Self {
        self.linetype = Some(AesValue::Constant(PrimitiveValue::Str(pattern.into())));
        self
    }

    /// Set the stat to use (default is Identity)
    pub fn stat(&mut self, stat: Stat) -> &mut Self {
        self.stat = stat;
        self
    }

    /// Set the position adjustment (default is Identity)
    pub fn position(&mut self, position: Position) -> &mut Self {
        self.position = position;
        self
    }
}

impl IntoLayer for GeomVLine {
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

impl Geom for GeomVLine {
    fn required_aesthetics(&self) -> &[Aesthetic] {
        &[Aesthetic::XIntercept]
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        use crate::visuals::LineStyle;
        
        // Get x-intercept values (scaled)
        let x_values = ctx.get_x_aesthetic_values(Aesthetic::XIntercept)?;
        
        // Get visual properties
        let colors = ctx.get_color_values()?;
        let alphas = ctx.get_unscaled_aesthetic_values(Aesthetic::Alpha)?;
        let sizes = ctx.get_unscaled_aesthetic_values(Aesthetic::Size)?;
        
        // Get linetype if specified
        let linetype_pattern = if let Some(AesValue::Constant(PrimitiveValue::Str(pattern))) =
            ctx.mapping().get(&Aesthetic::Linetype)
        {
            Some(pattern.clone())
        } else {
            None
        };
        
        // Apply line style once
        if let Some(pattern) = &linetype_pattern {
            let style = LineStyle::from(pattern.as_str());
            style.apply(ctx.cairo);
        } else {
            LineStyle::default().apply(ctx.cairo);
        }
        
        // Draw vertical line(s) across the full height of the plot
        let (y0, y1) = ctx.y_range;
        
        for (((x_normalized, color), alpha), size) in x_values
            .zip(colors)
            .zip(alphas)
            .zip(sizes)
        {
            let x_visual = ctx.map_x(x_normalized);
            
            // Set drawing properties for this line
            ctx.set_color_alpha(&color, alpha);
            ctx.cairo.set_line_width(size);
            
            // Draw line from bottom to top edge of plot area
            ctx.cairo.move_to(x_visual, y0);
            ctx.cairo.line_to(x_visual, y1);
            ctx.cairo.stroke().ok();
        }

        Ok(())
    }
}
