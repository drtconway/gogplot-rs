// Plot structure for grammar of graphics

use crate::data::DataSource;
use crate::error::PlotError;
use crate::geom::RenderContext;
use crate::layer::Layer;
use crate::scale::{ColorScale, ContinuousScale, ShapeScale};
use crate::theme::{Color, Font, FontStyle, FontWeight, LineStyle, TextTheme, Theme};
use cairo::{Context, Format, ImageSurface, PdfSurface, SvgSurface};
use std::path::Path;

/// Container for scales (x, y, color, size, etc.)
pub struct ScaleSet {
    pub x: Option<Box<dyn ContinuousScale>>,
    pub y: Option<Box<dyn ContinuousScale>>,
    pub color: Option<Box<dyn ColorScale>>,
    pub size: Option<Box<dyn ContinuousScale>>,
    pub alpha: Option<Box<dyn ContinuousScale>>,
    pub shape: Option<Box<dyn ShapeScale>>,
    // Add more as needed
}

impl ScaleSet {
    pub fn new() -> Self {
        Self {
            x: None,
            y: None,
            color: None,
            size: None,
            alpha: None,
            shape: None,
        }
    }
}

impl Default for ScaleSet {
    fn default() -> Self {
        Self::new()
    }
}

/// Main plot structure
pub struct Plot {
    /// Default data source for all layers
    pub data: Option<Box<dyn DataSource>>,

    /// Layers to render
    pub layers: Vec<Layer>,

    /// Scales for coordinate and aesthetic mappings
    pub scales: ScaleSet,

    /// Visual theme
    pub theme: Theme,

    /// Plot title
    pub title: Option<String>,

    /// X-axis label
    pub x_label: Option<String>,

    /// Y-axis label
    pub y_label: Option<String>,
}

impl Plot {
    /// Create a new plot with optional default data
    pub fn new(data: Option<Box<dyn DataSource>>) -> Self {
        Self {
            data,
            layers: Vec::new(),
            scales: ScaleSet::new(),
            theme: Theme::default(),
            title: None,
            x_label: None,
            y_label: None,
        }
    }

    /// Add a layer to the plot (builder style)
    pub fn layer(mut self, layer: Layer) -> Self {
        self.layers.push(layer);
        self
    }

    /// Set the plot title (builder style)
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the x-axis label (builder style)
    pub fn x_label(mut self, label: impl Into<String>) -> Self {
        self.x_label = Some(label.into());
        self
    }

    /// Set the y-axis label (builder style)
    pub fn y_label(mut self, label: impl Into<String>) -> Self {
        self.y_label = Some(label.into());
        self
    }

    /// Set the theme (builder style)
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Set the x scale (builder style)
    pub fn scale_x(mut self, scale: Box<dyn ContinuousScale>) -> Self {
        self.scales.x = Some(scale);
        self
    }

    /// Set the y scale (builder style)
    pub fn scale_y(mut self, scale: Box<dyn ContinuousScale>) -> Self {
        self.scales.y = Some(scale);
        self
    }

    /// Set the color scale (builder style)
    pub fn scale_color(mut self, scale: Box<dyn ColorScale>) -> Self {
        self.scales.color = Some(scale);
        self
    }

    /// Set the size scale (builder style)
    pub fn scale_size(mut self, scale: Box<dyn ContinuousScale>) -> Self {
        self.scales.size = Some(scale);
        self
    }

    /// Set the alpha scale (builder style)
    pub fn scale_alpha(mut self, scale: Box<dyn ContinuousScale>) -> Self {
        self.scales.alpha = Some(scale);
        self
    }

    /// Set the shape scale (builder style)
    pub fn scale_shape(mut self, scale: Box<dyn ShapeScale>) -> Self {
        self.scales.shape = Some(scale);
        self
    }

    /// Render the plot to an ImageSurface
    ///
    /// # Arguments
    ///
    /// * `width` - Width of the plot in pixels
    /// * `height` - Height of the plot in pixels
    ///
    /// # Returns
    ///
    /// An ImageSurface containing the rendered plot, or an error
    pub fn render(&self, width: i32, height: i32) -> Result<ImageSurface, PlotError> {
        // Create the surface
        let surface = ImageSurface::create(Format::ARgb32, width, height)
            .map_err(|e| PlotError::ThemeError(format!("Failed to create surface: {}", e)))?;

        let mut ctx = Context::new(&surface)
            .map_err(|e| PlotError::ThemeError(format!("Failed to create context: {}", e)))?;

        // Fill background
        ctx.set_source_rgb(1.0, 1.0, 1.0);
        ctx.paint()
            .map_err(|e| PlotError::ThemeError(format!("Failed to paint background: {}", e)))?;

        // Define plot area (leaving margins for axes and labels)
        let margin_left = 60.0;
        let margin_right = 20.0;
        let margin_top = 40.0;
        let margin_bottom = 60.0;

        let plot_x0 = margin_left;
        let plot_x1 = width as f64 - margin_right;
        let plot_y0 = margin_top;
        let plot_y1 = height as f64 - margin_bottom;

        // Draw axes before rendering layers
        self.draw_axes(&mut ctx, plot_x0, plot_x1, plot_y0, plot_y1)?;

        // Render each layer
        for layer in &self.layers {
            // Get the data source for this layer (layer data or plot default data)
            let data: &dyn DataSource = match &layer.data {
                Some(d) => d.as_ref(),
                None => match &self.data {
                    Some(d) => d.as_ref(),
                    None => return Err(PlotError::MissingAesthetic("No data source".to_string())),
                },
            };

            // Create render context
            let mut render_ctx = RenderContext::new(
                &mut ctx,
                data,
                &layer.mapping,
                &self.scales,
                (plot_x0, plot_x1),
                (plot_y1, plot_y0), // Y is inverted in screen coordinates
            );

            // Render the geom
            layer.geom.render(&mut render_ctx)?;
        }

        Ok(surface)
    }

    /// Save the plot to a file
    ///
    /// The output format is determined by the file extension:
    /// - `.png` - PNG image
    /// - `.svg` - SVG vector graphic
    /// - `.pdf` - PDF document
    ///
    /// # Arguments
    ///
    /// * `path` - Output file path
    /// * `width` - Width in pixels (for PNG) or points (for SVG/PDF)
    /// * `height` - Height in pixels (for PNG) or points (for SVG/PDF)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// plot.save("output.png", 800, 600)?;
    /// ```
    pub fn save(&self, path: impl AsRef<Path>, width: i32, height: i32) -> Result<(), PlotError> {
        let path = path.as_ref();
        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| PlotError::ThemeError("Invalid file path".to_string()))?;

        match extension.to_lowercase().as_str() {
            "png" => {
                let surface = self.render(width, height)?;
                let mut file = std::fs::File::create(path)
                    .map_err(|e| PlotError::ThemeError(format!("Failed to create file: {}", e)))?;
                surface
                    .write_to_png(&mut file)
                    .map_err(|e| PlotError::ThemeError(format!("Failed to write PNG: {}", e)))?;
            }
            "svg" => {
                let surface =
                    SvgSurface::new(width as f64, height as f64, Some(path)).map_err(|e| {
                        PlotError::ThemeError(format!("Failed to create SVG surface: {}", e))
                    })?;

                let mut ctx = Context::new(&surface).map_err(|e| {
                    PlotError::ThemeError(format!("Failed to create context: {}", e))
                })?;
                self.render_with_context(&mut ctx, width, height)?;
                surface.finish();
            }
            "pdf" => {
                let surface = PdfSurface::new(width as f64, height as f64, path).map_err(|e| {
                    PlotError::ThemeError(format!("Failed to create PDF surface: {}", e))
                })?;

                let mut ctx = Context::new(&surface).map_err(|e| {
                    PlotError::ThemeError(format!("Failed to create context: {}", e))
                })?;
                self.render_with_context(&mut ctx, width, height)?;
                surface.finish();
            }
            _ => {
                return Err(PlotError::ThemeError(format!(
                    "Unsupported file format: {}",
                    extension
                )));
            }
        }

        Ok(())
    }

    /// Helper to apply color to Cairo context
    fn apply_color(ctx: &mut Context, color: &Color) {
        ctx.set_source_rgba(
            color.0 as f64 / 255.0,
            color.1 as f64 / 255.0,
            color.2 as f64 / 255.0,
            color.3 as f64 / 255.0,
        );
    }

    /// Helper to apply font to Cairo context
    fn apply_font(ctx: &mut Context, font: &Font) {
        let slant = match font.style {
            FontStyle::Normal => cairo::FontSlant::Normal,
            FontStyle::Italic => cairo::FontSlant::Italic,
            FontStyle::Oblique => cairo::FontSlant::Oblique,
        };
        let weight = match font.weight {
            FontWeight::Normal => cairo::FontWeight::Normal,
            FontWeight::Bold => cairo::FontWeight::Bold,
            FontWeight::Light => cairo::FontWeight::Normal, // Cairo doesn't have Light
        };
        ctx.select_font_face(&font.family, slant, weight);
        ctx.set_font_size(font.size as f64);
    }

    /// Helper to apply line style to Cairo context
    fn apply_line_style(ctx: &mut Context, line: &LineStyle) {
        Self::apply_color(ctx, &line.color);
        ctx.set_line_width(line.width as f64);
        if let Some(ref dash) = line.dash {
            let dash_f64: Vec<f64> = dash.iter().map(|&d| d as f64).collect();
            ctx.set_dash(&dash_f64, 0.0);
        } else {
            ctx.set_dash(&[], 0.0);
        }
    }

    /// Helper to apply text theme to Cairo context
    fn apply_text_theme(ctx: &mut Context, text: &TextTheme) {
        Self::apply_font(ctx, &text.font);
        Self::apply_color(ctx, &text.color);
    }

    /// Draw axes with tick marks and labels
    fn draw_axes(
        &self,
        ctx: &mut Context,
        x0: f64,
        x1: f64,
        y0: f64,
        y1: f64,
    ) -> Result<(), PlotError> {
        // X axis
        if let Some(ref line_style) = self.theme.axis_x.line.line {
            Self::apply_line_style(ctx, line_style);
            ctx.move_to(x0, y1);
            ctx.line_to(x1, y1);
            ctx.stroke().ok();
        }

        // Y axis
        if let Some(ref line_style) = self.theme.axis_y.line.line {
            Self::apply_line_style(ctx, line_style);
            ctx.move_to(x0, y0);
            ctx.line_to(x0, y1);
            ctx.stroke().ok();
        }

        // X axis (bottom)
        ctx.move_to(x0, y1);
        ctx.line_to(x1, y1);
        ctx.stroke().ok();

        // Y axis (left)
        ctx.move_to(x0, y0);
        ctx.line_to(x0, y1);
        ctx.stroke().ok();

        // Draw X axis ticks and labels
        if let Some(x_scale) = &self.scales.x {
            let breaks = x_scale.breaks();
            let labels = x_scale.labels();
            let tick_length = self.theme.axis_x.line.tick_length as f64;

            Self::apply_text_theme(ctx, &self.theme.axis_x.text.text);

            for (value, label) in breaks.iter().zip(labels.iter()) {
                if let Some(normalized) = x_scale.map_value(*value) {
                    let x_pos = x0 + normalized * (x1 - x0);

                    // Draw tick mark
                    if let Some(ref tick_style) = self.theme.axis_x.line.ticks {
                        Self::apply_line_style(ctx, tick_style);
                        ctx.move_to(x_pos, y1);
                        ctx.line_to(x_pos, y1 + tick_length);
                        ctx.stroke().ok();
                    }

                    // Draw label
                    Self::apply_text_theme(ctx, &self.theme.axis_x.text.text);
                    let extents = ctx.text_extents(label).ok();
                    if let Some(ext) = extents {
                        let margin = self.theme.axis_x.text.text.margin.top as f64;
                        ctx.move_to(x_pos - ext.width() / 2.0, y1 + tick_length + margin + ext.height());
                        ctx.show_text(label).ok();
                    }
                }
            }
        }

        // Draw Y axis ticks and labels
        if let Some(y_scale) = &self.scales.y {
            let breaks = y_scale.breaks();
            let labels = y_scale.labels();
            let tick_length = self.theme.axis_y.line.tick_length as f64;

            Self::apply_text_theme(ctx, &self.theme.axis_y.text.text);

            for (value, label) in breaks.iter().zip(labels.iter()) {
                if let Some(normalized) = y_scale.map_value(*value) {
                    let y_pos = y1 - normalized * (y1 - y0); // Y is inverted

                    // Draw tick mark
                    if let Some(ref tick_style) = self.theme.axis_y.line.ticks {
                        Self::apply_line_style(ctx, tick_style);
                        ctx.move_to(x0 - tick_length, y_pos);
                        ctx.line_to(x0, y_pos);
                        ctx.stroke().ok();
                    }

                    // Draw label
                    Self::apply_text_theme(ctx, &self.theme.axis_y.text.text);
                    let extents = ctx.text_extents(label).ok();
                    if let Some(ext) = extents {
                        let margin = self.theme.axis_y.text.text.margin.right as f64;
                        ctx.move_to(x0 - tick_length - margin - ext.width(), y_pos + ext.height() / 2.0);
                        ctx.show_text(label).ok();
                    }
                }
            }
        }

        // Draw X axis title
        if let Some(x_label) = &self.x_label {
            Self::apply_text_theme(ctx, &self.theme.axis_x.text.title);
            let extents = ctx.text_extents(x_label).ok();
            if let Some(ext) = extents {
                let x_center = (x0 + x1) / 2.0;
                // Position below: axis line + ticks + tick label margin + typical label height + title margin
                let tick_length = self.theme.axis_x.line.tick_length as f64;
                let label_margin = self.theme.axis_x.text.text.margin.top as f64;
                let typical_label_height = self.theme.axis_x.text.text.font.size as f64;
                let title_margin = self.theme.axis_x.text.title.margin.top as f64;
                let y_offset = y1 + tick_length + label_margin + typical_label_height + title_margin + ext.height();
                ctx.move_to(x_center - ext.width() / 2.0, y_offset);
                ctx.show_text(x_label).ok();
            }
        }

        // Draw Y axis title (rotated)
        if let Some(y_label) = &self.y_label {
            ctx.save().ok();
            Self::apply_text_theme(ctx, &self.theme.axis_y.text.title);
            let y_center = (y0 + y1) / 2.0;
            let extents = ctx.text_extents(y_label).ok();
            if let Some(ext) = extents {
                // Position to left of: axis line + ticks + tick label margin + typical max label width + title margin
                let tick_length = self.theme.axis_y.line.tick_length as f64;
                let label_margin = self.theme.axis_y.text.text.margin.right as f64;
                // Estimate max label width (rough approximation based on font size * typical digits)
                let typical_label_width = self.theme.axis_y.text.text.font.size as f64 * 2.5;
                let title_margin = self.theme.axis_y.text.title.margin.right as f64;
                let title_height = ext.height();
                let x_offset = x0 - tick_length - label_margin - typical_label_width - title_margin - title_height;
                ctx.move_to(x_offset, y_center + ext.width() / 2.0);
                ctx.rotate(-std::f64::consts::PI / 2.0);
                ctx.show_text(y_label).ok();
            }
            ctx.restore().ok();
        }

        // Draw plot title
        if let Some(title) = &self.title {
            Self::apply_text_theme(ctx, &self.theme.plot_title.text);
            let extents = ctx.text_extents(title).ok();
            if let Some(ext) = extents {
                let x_center = (x0 + x1) / 2.0;
                let y_offset = self.theme.plot_title.text.margin.top as f64;
                ctx.move_to(x_center - ext.width() / 2.0, y_offset + ext.height());
                ctx.show_text(title).ok();
            }
        }

        Ok(())
    }

    /// Helper method to render using an existing Cairo context
    fn render_with_context(
        &self,
        ctx: &mut Context,
        width: i32,
        height: i32,
    ) -> Result<(), PlotError> {
        // Fill background
        Self::apply_color(ctx, &self.theme.background.fill.color);
        ctx.paint()
            .map_err(|e| PlotError::ThemeError(format!("Failed to paint background: {}", e)))?;

        // Define plot area using theme margins
        let margin_left = self.theme.plot_margin.left as f64;
        let margin_right = self.theme.plot_margin.right as f64;
        let margin_top = self.theme.plot_margin.top as f64;
        let margin_bottom = self.theme.plot_margin.bottom as f64;

        let plot_x0 = margin_left;
        let plot_x1 = width as f64 - margin_right;
        let plot_y0 = margin_top;
        let plot_y1 = height as f64 - margin_bottom;

        // Draw panel background
        if let Some(ref panel_bg) = self.theme.panel.background {
            Self::apply_color(ctx, &panel_bg.color);
            ctx.rectangle(plot_x0, plot_y0, plot_x1 - plot_x0, plot_y1 - plot_y0);
            ctx.fill().ok();
        }

        // Draw panel border
        if let Some(ref border) = self.theme.panel.border {
            Self::apply_line_style(ctx, border);
            ctx.rectangle(plot_x0, plot_y0, plot_x1 - plot_x0, plot_y1 - plot_y0);
            ctx.stroke().ok();
        }

        // Draw axes before rendering layers
        self.draw_axes(ctx, plot_x0, plot_x1, plot_y0, plot_y1)?;

        // Render each layer
        for layer in &self.layers {
            let data: &dyn DataSource = match &layer.data {
                Some(d) => d.as_ref(),
                None => match &self.data {
                    Some(d) => d.as_ref(),
                    None => return Err(PlotError::MissingAesthetic("No data source".to_string())),
                },
            };

            let mut render_ctx = RenderContext::new(
                ctx,
                data,
                &layer.mapping,
                &self.scales,
                (plot_x0, plot_x1),
                (plot_y1, plot_y0),
            );

            layer.geom.render(&mut render_ctx)?;
        }

        Ok(())
    }
}

impl Default for Plot {
    fn default() -> Self {
        Self::new(None)
    }
}
