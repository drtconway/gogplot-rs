

use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::PlotError;
use crate::geom::context::AestheticValues;

/// GeomBar renders bars from y=0 to y=value
/// By default, it uses Stat::Count to count occurrences at each x position
pub struct GeomBar {
    /// Default fill color (if not mapped)
    pub fill: Option<AesValue>,

    /// Default stroke color (if not mapped)
    pub color: Option<AesValue>,

    /// Default alpha/opacity (if not mapped)
    pub alpha: Option<AesValue>,

    /// Bar width (as a proportion of the spacing between x values)
    pub width: f64,
}

impl GeomBar {
    /// Create a new bar geom with default settings
    pub fn new() -> Self {
        Self {
            fill: None,
            color: None,
            alpha: None,
            width: 0.9,
        }
    }

    /// Set the default fill color
    pub fn fill(&mut self, color: crate::theme::Color) -> &mut Self {
        self.fill = Some(AesValue::constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the default stroke color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        self.color = Some(AesValue::constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the default alpha/opacity
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = Some(AesValue::constant(PrimitiveValue::Float(
            alpha.clamp(0.0, 1.0),
        )));
        self
    }

    /// Set the bar width (as a proportion of spacing, typically 0.0-1.0)
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width;
        self
    }
}

impl Default for GeomBar {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoLayer for GeomBar {
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
            mapping: Some(mapping),
            stat,
            position,
        }
    }
}

impl Geom for GeomBar {
    fn train_scales(&self, _scales: &mut crate::scale::ScaleSet) {
        
    }

    fn apply_scales(&mut self, scales: &crate::scale::ScaleSet) {
        
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        // NEW SIMPLIFIED RENDER: Data is pre-normalized to [0,1] and position-adjusted
        // setup_data() has already created xmin/xmax columns
        // apply_scales() has already normalized everything to [0,1]
        // position adjustments (dodge/stack) have already been applied
        
        let mapping = ctx.mapping();
        
        // Get normalized [0,1] coordinates - all data is already scaled
        let xmin_vals = ctx.get_x_aesthetic_values(Aesthetic::Xmin)?;
        let xmax_vals = ctx.get_x_aesthetic_values(Aesthetic::Xmax)?;
        
        // For Y, check if we have Ymin/Ymax (from stack) or just Y
        let has_y_range = mapping.contains(Aesthetic::Ymin) && mapping.contains(Aesthetic::Ymax);
        
        let ymin_vals = if has_y_range {
            ctx.get_y_aesthetic_values(Aesthetic::Ymin)?
        } else {
            // Y baseline is at 0, which is already normalized by apply_scales
            // Get the normalized y=0 position
            let y_zero_norm = if let Some(y_scale) = ctx.scales.y.as_deref() {
                y_scale.map_value(0.0).unwrap_or(0.0)
            } else {
                0.0
            };
            // Create iterator that repeats y_zero for each bar
            let n = ctx.data().len();
            AestheticValues::Owned(vec![y_zero_norm; n])
        };
        
        let ymax_vals = if has_y_range {
            ctx.get_y_aesthetic_values(Aesthetic::Ymax)?
        } else {
            ctx.get_y_aesthetic_values(Aesthetic::Y)?
        };
        
        // Get styling aesthetics
        let fills = ctx.get_fill_color_values()?;
        let colors = ctx.get_color_values()?;
        let alphas = ctx.get_unscaled_aesthetic_values(Aesthetic::Alpha)?;
        
        // Render each bar
        for (((((xmin, xmax), ymin), ymax), fill), (color, alpha)) in 
            xmin_vals.zip(xmax_vals).zip(ymin_vals).zip(ymax_vals)
                .zip(fills).zip(colors.zip(alphas))
        {
            // Map normalized [0,1] coordinates to device coordinates
            let x_left = ctx.map_x(xmin);
            let x_right = ctx.map_x(xmax);
            let y_top = ctx.map_y(ymax);
            let y_bottom = ctx.map_y(ymin);
            
            let x = x_left.min(x_right);
            let y = y_top.min(y_bottom);
            let width = (x_right - x_left).abs();
            let height = (y_bottom - y_top).abs();
            
            // Fill the bar
            ctx.set_color_alpha(&fill, alpha);
            ctx.cairo.rectangle(x, y, width, height);
            ctx.cairo.fill().ok();
            
            // Stroke the bar if a stroke color is defined
            if self.color.is_some() {
                ctx.set_color_alpha(&color, alpha);
                ctx.cairo.rectangle(x, y, width, height);
                ctx.cairo.stroke().ok();
            }
        }
        
        Ok(())
    }
}
