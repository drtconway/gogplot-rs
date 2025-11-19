// Plot structure for grammar of graphics

use crate::data::DataSource;
use crate::error::PlotError;
use crate::geom::RenderContext;
use crate::guide::Guides;
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

    /// Default aesthetic mappings for all layers
    pub default_aes: crate::aesthetics::AesMap,

    /// Layers to render
    pub layers: Vec<Layer>,

    /// Scales for coordinate and aesthetic mappings
    pub scales: ScaleSet,

    /// Visual theme
    pub theme: Theme,

    /// Guides configuration (legends, etc.)
    pub guides: Guides,

    /// Plot title
    pub title: Option<String>,
}

impl Plot {
    /// Create a new plot with optional default data
    pub fn new(data: Option<Box<dyn DataSource>>) -> Self {
        Self {
            data,
            default_aes: crate::aesthetics::AesMap::new(),
            layers: Vec::new(),
            scales: ScaleSet::new(),
            theme: Theme::default(),
            guides: Guides::default(),
            title: None,
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



    /// Set the theme (builder style)
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Set the guides configuration (builder style)
    pub fn guides(mut self, guides: Guides) -> Self {
        self.guides = guides;
        self
    }

    /// Set default aesthetic mappings for all layers (builder style)
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// let plot = Plot::new(Some(Box::new(df)))
    ///     .aes(|a| {
    ///         a.x("x_column");
    ///         a.y("y_column");
    ///         a.color("category");
    ///     });
    /// ```
    pub fn aes<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut crate::aesthetics::AesMap),
    {
        f(&mut self.default_aes);
        self
    }

    /// Add a point geom layer using default aesthetics (builder style)
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// // Use default aesthetics
    /// plot.geom_point()
    /// 
    /// // Override or add aesthetics for this layer
    /// plot.geom_point_with(|geom, aes| {
    ///     geom.size(3.0);
    ///     aes.color("species");
    /// })
    /// ```
    pub fn geom_point(self) -> Self {
        self.geom_point_with(|geom, _| geom)
    }

    /// Add a point geom layer with customization (builder style)
    pub fn geom_point_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(crate::geom::point::GeomPoint, &mut crate::aesthetics::AesMap) -> crate::geom::point::GeomPoint,
    {
        let geom = crate::geom::point::GeomPoint::default();
        let mut aes = self.default_aes.clone();
        let geom = f(geom, &mut aes);
        
        let mut layer = geom.into_layer();
        // Merge: geom defaults first, then overlay with plot aesthetics
        // This way explicit aesthetic mappings override geom defaults
        for (aesthetic, value) in aes.iter() {
            layer.mapping.set(aesthetic.clone(), value.clone());
        }
        self.layers.push(layer);
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

        // Use the common rendering code
        self.render_with_context(&mut ctx, width, height)?;

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
    pub fn save(mut self, path: impl AsRef<Path>, width: i32, height: i32) -> Result<(), PlotError> {
        // Create default scales for unmapped aesthetics
        self.create_default_scales();
        
        // Train scales on data before rendering
        self.train_scales();
        
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

    /// Helper to apply fill style to Cairo context
    fn apply_fill_style(ctx: &mut Context, fill: &crate::theme::FillStyle) {
        let alpha = (fill.color.3 as f64 / 255.0) * fill.opacity as f64;
        ctx.set_source_rgba(
            fill.color.0 as f64 / 255.0,
            fill.color.1 as f64 / 255.0,
            fill.color.2 as f64 / 255.0,
            alpha,
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

    /// Draw grid lines in the panel area
    fn draw_grid_lines(
        &self,
        ctx: &mut Context,
        plot_x0: f64,
        plot_x1: f64,
        plot_y0: f64,
        plot_y1: f64,
    ) -> Result<(), PlotError> {
        // Draw minor grid lines first (if present) so major grid lines are on top
        if let Some(ref grid_minor) = self.theme.panel.grid_minor {
            Self::apply_line_style(ctx, grid_minor);
            
            // X-axis minor grid lines
            if let Some(ref x_scale) = self.scales.x {
                let breaks = x_scale.breaks();
                // Generate minor breaks between major breaks
                for i in 0..breaks.len().saturating_sub(1) {
                    let mid = (breaks[i] + breaks[i + 1]) / 2.0;
                    if let Some(normalized) = x_scale.map_value(mid) {
                        let x = plot_x0 + normalized * (plot_x1 - plot_x0);
                        ctx.move_to(x, plot_y0);
                        ctx.line_to(x, plot_y1);
                    }
                }
            }
            
            // Y-axis minor grid lines
            if let Some(ref y_scale) = self.scales.y {
                let breaks = y_scale.breaks();
                // Generate minor breaks between major breaks
                for i in 0..breaks.len().saturating_sub(1) {
                    let mid = (breaks[i] + breaks[i + 1]) / 2.0;
                    if let Some(normalized) = y_scale.map_value(mid) {
                        let y = plot_y1 - normalized * (plot_y1 - plot_y0);
                        ctx.move_to(plot_x0, y);
                        ctx.line_to(plot_x1, y);
                    }
                }
            }
            
            ctx.stroke().ok();
        }
        
        // Draw major grid lines
        if let Some(ref grid_major) = self.theme.panel.grid_major {
            Self::apply_line_style(ctx, grid_major);
            
            // X-axis major grid lines (vertical lines at tick positions)
            if let Some(ref x_scale) = self.scales.x {
                for &break_value in x_scale.breaks() {
                    if let Some(normalized) = x_scale.map_value(break_value) {
                        let x = plot_x0 + normalized * (plot_x1 - plot_x0);
                        ctx.move_to(x, plot_y0);
                        ctx.line_to(x, plot_y1);
                    }
                }
            }
            
            // Y-axis major grid lines (horizontal lines at tick positions)
            if let Some(ref y_scale) = self.scales.y {
                for &break_value in y_scale.breaks() {
                    if let Some(normalized) = y_scale.map_value(break_value) {
                        let y = plot_y1 - normalized * (plot_y1 - plot_y0);
                        ctx.move_to(plot_x0, y);
                        ctx.line_to(plot_x1, y);
                    }
                }
            }
            
            ctx.stroke().ok();
        }
        
        Ok(())
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
        // Draw axis lines based on position
        use crate::guide::{AxisType, XAxisPosition, YAxisPosition};
        
        // X axis line
        let x_position = self.guides.x_axis.as_ref()
            .and_then(|guide| match &guide.position {
                AxisType::X(pos) => Some(pos.clone()),
                _ => None,
            })
            .unwrap_or(XAxisPosition::Bottom);
        
        if let Some(ref line_style) = self.theme.axis_x.line.line {
            Self::apply_line_style(ctx, line_style);
            match x_position {
                XAxisPosition::Bottom => {
                    ctx.move_to(x0, y1);
                    ctx.line_to(x1, y1);
                }
                XAxisPosition::Top => {
                    ctx.move_to(x0, y0);
                    ctx.line_to(x1, y0);
                }
            }
            ctx.stroke().ok();
        }

        // Y axis line
        let y_position = self.guides.y_axis.as_ref()
            .and_then(|guide| match &guide.position {
                AxisType::Y(pos) => Some(pos.clone()),
                _ => None,
            })
            .unwrap_or(YAxisPosition::Left);
        
        if let Some(ref line_style) = self.theme.axis_y.line.line {
            Self::apply_line_style(ctx, line_style);
            match y_position {
                YAxisPosition::Left => {
                    ctx.move_to(x0, y0);
                    ctx.line_to(x0, y1);
                }
                YAxisPosition::Right => {
                    ctx.move_to(x1, y0);
                    ctx.line_to(x1, y1);
                }
            }
            ctx.stroke().ok();
        }

        // Draw X axis ticks and labels
        if let Some(x_scale) = &self.scales.x {
            // Determine X axis position
            use crate::guide::{AxisType, XAxisPosition};
            let x_position = self.guides.x_axis.as_ref()
                .and_then(|guide| match &guide.position {
                    AxisType::X(pos) => Some(pos.clone()),
                    _ => None,
                })
                .unwrap_or(XAxisPosition::Bottom);
            
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
                        match x_position {
                            XAxisPosition::Bottom => {
                                ctx.move_to(x_pos, y1);
                                ctx.line_to(x_pos, y1 + tick_length);
                            }
                            XAxisPosition::Top => {
                                ctx.move_to(x_pos, y0);
                                ctx.line_to(x_pos, y0 - tick_length);
                            }
                        }
                        ctx.stroke().ok();
                    }

                    // Draw label
                    Self::apply_text_theme(ctx, &self.theme.axis_x.text.text);
                    let extents = ctx.text_extents(label).ok();
                    if let Some(ext) = extents {
                        let margin = self.theme.axis_x.text.text.margin.top as f64;
                        match x_position {
                            XAxisPosition::Bottom => {
                                ctx.move_to(x_pos - ext.width() / 2.0, y1 + tick_length + margin + ext.height());
                            }
                            XAxisPosition::Top => {
                                ctx.move_to(x_pos - ext.width() / 2.0, y0 - tick_length - margin);
                            }
                        }
                        ctx.show_text(label).ok();
                    }
                }
            }
        }

        // Draw Y axis ticks and labels
        if let Some(y_scale) = &self.scales.y {
            // Determine Y axis position
            use crate::guide::YAxisPosition;
            let y_position = self.guides.y_axis.as_ref()
                .and_then(|guide| match &guide.position {
                    crate::guide::AxisType::Y(pos) => Some(pos.clone()),
                    _ => None,
                })
                .unwrap_or(YAxisPosition::Left);
            
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
                        match y_position {
                            YAxisPosition::Left => {
                                ctx.move_to(x0 - tick_length, y_pos);
                                ctx.line_to(x0, y_pos);
                            }
                            YAxisPosition::Right => {
                                ctx.move_to(x1, y_pos);
                                ctx.line_to(x1 + tick_length, y_pos);
                            }
                        }
                        ctx.stroke().ok();
                    }

                    // Draw label
                    Self::apply_text_theme(ctx, &self.theme.axis_y.text.text);
                    let extents = ctx.text_extents(label).ok();
                    if let Some(ext) = extents {
                        let margin = self.theme.axis_y.text.text.margin.right as f64;
                        match y_position {
                            YAxisPosition::Left => {
                                ctx.move_to(x0 - tick_length - margin - ext.width(), y_pos + ext.height() / 2.0);
                            }
                            YAxisPosition::Right => {
                                ctx.move_to(x1 + tick_length + margin, y_pos + ext.height() / 2.0);
                            }
                        }
                        ctx.show_text(label).ok();
                    }
                }
            }
        }

        // Draw X axis title
        if let Some(x_axis) = &self.guides.x_axis {
            if let Some(x_label) = &x_axis.title {
                use crate::guide::{AxisType, XAxisPosition};
                let x_position = match &x_axis.position {
                    AxisType::X(pos) => pos.clone(),
                    _ => XAxisPosition::Bottom,
                };
                
                Self::apply_text_theme(ctx, &self.theme.axis_x.text.title);
                let extents = ctx.text_extents(x_label).ok();
                if let Some(ext) = extents {
                    let x_center = (x0 + x1) / 2.0;
                    let tick_length = self.theme.axis_x.line.tick_length as f64;
                    let label_margin = self.theme.axis_x.text.text.margin.top as f64;
                    let typical_label_height = self.theme.axis_x.text.text.font.size as f64;
                    let title_margin = self.theme.axis_x.text.title.margin.top as f64;
                    
                    match x_position {
                        XAxisPosition::Bottom => {
                            // Position below: axis line + ticks + tick label margin + typical label height + title margin
                            let y_offset = y1 + tick_length + label_margin + typical_label_height + title_margin + ext.height();
                            ctx.move_to(x_center - ext.width() / 2.0, y_offset);
                        }
                        XAxisPosition::Top => {
                            // Position above: axis line + ticks + tick label margin + title margin
                            let y_offset = y0 - tick_length - label_margin - typical_label_height - title_margin;
                            ctx.move_to(x_center - ext.width() / 2.0, y_offset);
                        }
                    }
                    ctx.show_text(x_label).ok();
                }
            }
        }

        // Draw Y axis title (rotated)
        if let Some(y_axis) = &self.guides.y_axis {
            if let Some(y_label) = &y_axis.title {
                use crate::guide::{AxisType, YAxisPosition};
                let y_position = match &y_axis.position {
                    AxisType::Y(pos) => pos.clone(),
                    _ => YAxisPosition::Left,
                };
                
                ctx.save().ok();
                Self::apply_text_theme(ctx, &self.theme.axis_y.text.title);
                let y_center = (y0 + y1) / 2.0;
                let extents = ctx.text_extents(y_label).ok();
                if let Some(ext) = extents {
                    let tick_length = self.theme.axis_y.line.tick_length as f64;
                    let label_margin = self.theme.axis_y.text.text.margin.right as f64;
                    // Estimate max label width (rough approximation based on font size * typical digits)
                    let typical_label_width = self.theme.axis_y.text.text.font.size as f64 * 2.5;
                    let title_margin = self.theme.axis_y.text.title.margin.right as f64;
                    let title_height = ext.height();
                    
                    match y_position {
                        YAxisPosition::Left => {
                            // Position to left of: axis line + ticks + tick label margin + typical max label width + title margin
                            let x_offset = x0 - tick_length - label_margin - typical_label_width - title_margin - title_height;
                            ctx.move_to(x_offset, y_center + ext.width() / 2.0);
                            ctx.rotate(-std::f64::consts::PI / 2.0);
                        }
                        YAxisPosition::Right => {
                            // Position to right of: axis line + ticks + tick label margin + typical max label width + title margin
                            let x_offset = x1 + tick_length + label_margin + typical_label_width + title_margin + title_height;
                            ctx.move_to(x_offset, y_center - ext.width() / 2.0);
                            ctx.rotate(std::f64::consts::PI / 2.0);
                        }
                    }
                    ctx.show_text(y_label).ok();
                }
                ctx.restore().ok();
            }
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

    /// Draw legends
    fn draw_legends(
        &self,
        ctx: &mut Context,
        _plot_x0: f64,
        plot_x1: f64,
        plot_y0: f64,
        _plot_y1: f64,
        width: i32,
        _height: i32,
    ) -> Result<(), PlotError> {
        use crate::guide::LegendPosition;

        // Generate automatic legends from scales if aesthetics are mapped
        let guides = self.generate_automatic_legends();

        // Collect all legends to draw
        let mut legends = Vec::new();
        
        // Add color legend if present
        if let Some(ref color_guide) = guides.color {
            legends.push(color_guide);
        }
        
        // Add shape legend if present
        if let Some(ref shape_guide) = guides.shape {
            legends.push(shape_guide);
        }
        
        // Add size legend if present
        if let Some(ref size_guide) = guides.size {
            legends.push(size_guide);
        }
        
        if legends.is_empty() {
            return Ok(());
        }

        // For now, render legends on the right side
        // Position legend between plot panel and right edge
        let legend_margin = 10.0;
        let legend_x = plot_x1 + legend_margin;
        let available_width = width as f64 - legend_x - legend_margin;
        
        // If there's not enough space, don't draw legends
        if available_width < 80.0 {
            return Ok(());
        }
        
        let mut legend_y = plot_y0;
        
        for legend in legends {
            if matches!(legend.position, LegendPosition::None) {
                continue;
            }
            
            // Draw legend background
            Self::apply_fill_style(ctx, &self.theme.legend.background);
            let legend_width = 120.0;
            let item_height = 20.0;
            let padding = 10.0;
            let title_height = if legend.title.is_some() { 20.0 } else { 0.0 };
            let legend_height = title_height + (legend.entries.len() as f64 * item_height) + padding * 2.0;
            
            ctx.rectangle(legend_x, legend_y, legend_width, legend_height);
            ctx.fill().ok();
            
            // Draw legend border
            Self::apply_line_style(ctx, &self.theme.legend.border);
            ctx.rectangle(legend_x, legend_y, legend_width, legend_height);
            ctx.stroke().ok();
            
            let mut item_y = legend_y + padding;
            
            // Draw title if present
            if let Some(ref title) = legend.title {
                Self::apply_font(ctx, &self.theme.legend.text_font);
                Self::apply_color(ctx, &self.theme.legend.text_color);
                ctx.set_font_size(self.theme.legend.text_font.size as f64);
                ctx.move_to(legend_x + padding, item_y + 12.0);
                ctx.show_text(title).ok();
                item_y += title_height;
            }
            
            // Draw legend entries
            for entry in &legend.entries {
                let symbol_x = legend_x + padding;
                let symbol_y = item_y + item_height / 2.0;
                
                // Draw symbol
                if let Some(color) = entry.color {
                    Self::apply_color(ctx, &color);
                    let size = entry.size.unwrap_or(5.0);
                    
                    if let Some(shape) = entry.shape {
                        Self::draw_shape(ctx, symbol_x + 10.0, symbol_y, size, shape);
                    } else {
                        // Default to circle
                        ctx.arc(symbol_x + 10.0, symbol_y, size, 0.0, 2.0 * std::f64::consts::PI);
                        ctx.fill().ok();
                    }
                }
                
                // Draw label
                Self::apply_font(ctx, &self.theme.legend.text_font);
                Self::apply_color(ctx, &self.theme.legend.text_color);
                ctx.move_to(symbol_x + 25.0, symbol_y + 4.0);
                ctx.show_text(&entry.label).ok();
                
                item_y += item_height;
            }
            
            legend_y += legend_height + 10.0;
        }
        
        Ok(())
    }

    /// Helper to draw a shape at a position
    fn draw_shape(ctx: &mut Context, x: f64, y: f64, size: f64, shape: crate::guide::Shape) {
        use crate::guide::Shape;
        
        match shape {
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
                let w = size * 0.3;
                ctx.move_to(x - size, y - w);
                ctx.line_to(x - w, y - w);
                ctx.line_to(x - w, y - size);
                ctx.line_to(x + w, y - size);
                ctx.line_to(x + w, y - w);
                ctx.line_to(x + size, y - w);
                ctx.line_to(x + size, y + w);
                ctx.line_to(x + w, y + w);
                ctx.line_to(x + w, y + size);
                ctx.line_to(x - w, y + size);
                ctx.line_to(x - w, y + w);
                ctx.line_to(x - size, y + w);
                ctx.close_path();
                ctx.fill().ok();
            }
            Shape::Plus => {
                ctx.set_line_width(size * 0.4);
                ctx.move_to(x - size, y);
                ctx.line_to(x + size, y);
                ctx.stroke().ok();
                ctx.move_to(x, y - size);
                ctx.line_to(x, y + size);
                ctx.stroke().ok();
            }
        }
    }

    /// Create default scales for aesthetics that don't have scales but are mapped to columns
    fn create_default_scales(&mut self) {
        use crate::aesthetics::{Aesthetic, AesValue};
        use crate::scale::continuous::Builder;
        use crate::scale::color::DiscreteColor;
        use crate::scale::shape::DiscreteShape;
        
        // Find the first layer that maps each aesthetic to determine default scales
        for layer in &self.layers {
            // X scale
            if self.scales.x.is_none() {
                if let Some(AesValue::Column(col_name)) = layer.mapping.get(&Aesthetic::X) {
                    // Create default linear scale
                    if let Ok(scale) = Builder::new().linear() {
                        self.scales.x = Some(Box::new(scale));
                        // Set default axis title if not already set
                        if self.guides.x_axis.is_none() {
                            self.guides.x_axis = Some(crate::guide::AxisGuide::x().title(col_name.clone()));
                        }
                    }
                }
            }
            
            // Y scale
            if self.scales.y.is_none() {
                if let Some(AesValue::Column(col_name)) = layer.mapping.get(&Aesthetic::Y) {
                    // Create default linear scale
                    if let Ok(scale) = Builder::new().linear() {
                        self.scales.y = Some(Box::new(scale));
                        // Set default axis title if not already set
                        if self.guides.y_axis.is_none() {
                            self.guides.y_axis = Some(crate::guide::AxisGuide::y().title(col_name.clone()));
                        }
                    }
                }
            }
            
            // Color scale
            if self.scales.color.is_none() {
                if let Some(AesValue::Column(_)) = layer.mapping.get(&Aesthetic::Color) {
                    // Create default discrete color scale
                    self.scales.color = Some(Box::new(DiscreteColor::default_palette()));
                }
            }
            
            // Shape scale
            if self.scales.shape.is_none() {
                if let Some(AesValue::Column(_)) = layer.mapping.get(&Aesthetic::Shape) {
                    // Create default discrete shape scale
                    self.scales.shape = Some(Box::new(DiscreteShape::default_shapes()));
                }
            }
            
            // Size scale
            if self.scales.size.is_none() {
                if let Some(AesValue::Column(_)) = layer.mapping.get(&Aesthetic::Size) {
                    // Create default linear scale for size
                    if let Ok(scale) = Builder::new().linear() {
                        self.scales.size = Some(Box::new(scale));
                    }
                }
            }
            
            // Alpha scale
            if self.scales.alpha.is_none() {
                if let Some(AesValue::Column(_)) = layer.mapping.get(&Aesthetic::Alpha) {
                    // Create default linear scale for alpha
                    if let Ok(scale) = Builder::new().linear() {
                        self.scales.alpha = Some(Box::new(scale));
                    }
                }
            }
        }
    }

    /// Train all scales on the data
    fn train_scales(&mut self) {
        use crate::aesthetics::{Aesthetic, AesValue};
        
        for layer in &self.layers {
            let data: &dyn DataSource = match &layer.data {
                Some(d) => d.as_ref(),
                None => match &self.data {
                    Some(d) => d.as_ref(),
                    None => continue,
                },
            };
            
            // Train x scale
            if let Some(AesValue::Column(col_name)) = layer.mapping.get(&Aesthetic::X) {
                if let Some(ref mut scale) = self.scales.x {
                    if let Some(vec) = data.get(col_name) {
                        scale.train(vec);
                    }
                }
            }
            
            // Train y scale
            if let Some(AesValue::Column(col_name)) = layer.mapping.get(&Aesthetic::Y) {
                if let Some(ref mut scale) = self.scales.y {
                    if let Some(vec) = data.get(col_name) {
                        scale.train(vec);
                    }
                }
            }
            
            // Train color scale
            if let Some(AesValue::Column(col_name)) = layer.mapping.get(&Aesthetic::Color) {
                if let Some(ref mut scale) = self.scales.color {
                    if let Some(vec) = data.get(col_name) {
                        scale.train(vec);
                    }
                }
            }
            
            // Train shape scale
            if let Some(AesValue::Column(col_name)) = layer.mapping.get(&Aesthetic::Shape) {
                if let Some(ref mut scale) = self.scales.shape {
                    if let Some(vec) = data.get(col_name) {
                        scale.train(vec);
                    }
                }
            }
            
            // Train size scale
            if let Some(AesValue::Column(col_name)) = layer.mapping.get(&Aesthetic::Size) {
                if let Some(ref mut scale) = self.scales.size {
                    if let Some(vec) = data.get(col_name) {
                        scale.train(vec);
                    }
                }
            }
            
            // Train alpha scale
            if let Some(AesValue::Column(col_name)) = layer.mapping.get(&Aesthetic::Alpha) {
                if let Some(ref mut scale) = self.scales.alpha {
                    if let Some(vec) = data.get(col_name) {
                        scale.train(vec);
                    }
                }
            }
        }
    }

    /// Generate legends automatically from scales when aesthetics are mapped
    fn generate_automatic_legends(&self) -> Guides {
        use crate::aesthetics::{Aesthetic, AesValue};
        use crate::guide::{LegendGuide, LegendEntry, LegendPosition, Shape};
        
        let mut guides = self.guides.clone();
        
        // Check if any layer maps Color aesthetic to a column
        let has_color_mapping = self.layers.iter().any(|layer| {
            matches!(layer.mapping.get(&Aesthetic::Color), Some(AesValue::Column(_)))
        });
        
        // Check if any layer maps Shape aesthetic to a column
        let has_shape_mapping = self.layers.iter().any(|layer| {
            matches!(layer.mapping.get(&Aesthetic::Shape), Some(AesValue::Column(_)))
        });
        
        // Generate color legend if Color is mapped and we have a color scale
        if has_color_mapping && self.scales.color.is_some() && guides.color.is_none() {
            if let Some(ref color_scale) = self.scales.color {
                let breaks = color_scale.legend_breaks();
                if !breaks.is_empty() {
                    let mut legend = LegendGuide::new();
                    legend.position = LegendPosition::Right;
                    
                    // Get the column name for the title
                    for layer in &self.layers {
                        if let Some(AesValue::Column(col_name)) = layer.mapping.get(&Aesthetic::Color) {
                            legend.title = Some(col_name.clone());
                            break;
                        }
                    }
                    
                    // Generate entries from breaks
                    for category in breaks {
                        if let Some(color) = color_scale.map_discrete_to_color(&category) {
                            legend.entries.push(LegendEntry {
                                label: category.clone(),
                                color: Some(color),
                                shape: Some(Shape::Circle),
                                size: Some(5.0),
                            });
                        }
                    }
                    
                    guides.color = Some(legend);
                }
            }
        }
        
        // Generate shape legend if Shape is mapped and we have a shape scale
        if has_shape_mapping && self.scales.shape.is_some() && guides.shape.is_none() {
            if let Some(ref shape_scale) = self.scales.shape {
                let breaks = shape_scale.legend_breaks();
                if !breaks.is_empty() {
                    let mut legend = LegendGuide::new();
                    legend.position = LegendPosition::Right;
                    
                    // Get the column name for the title
                    for layer in &self.layers {
                        if let Some(AesValue::Column(col_name)) = layer.mapping.get(&Aesthetic::Shape) {
                            legend.title = Some(col_name.clone());
                            break;
                        }
                    }
                    
                    // Generate entries from breaks
                    for category in breaks {
                        if let Some(point_shape) = shape_scale.map_to_shape(&category) {
                            let shape = match point_shape {
                                crate::geom::point::PointShape::Circle => Shape::Circle,
                                crate::geom::point::PointShape::Square => Shape::Square,
                                crate::geom::point::PointShape::Triangle => Shape::Triangle,
                                crate::geom::point::PointShape::Diamond => Shape::Diamond,
                                crate::geom::point::PointShape::Cross => Shape::Cross,
                                crate::geom::point::PointShape::Plus => Shape::Plus,
                            };
                            
                            legend.entries.push(LegendEntry {
                                label: category.clone(),
                                color: Some(Color(100, 100, 100, 255)), // Default gray
                                shape: Some(shape),
                                size: Some(5.0),
                            });
                        }
                    }
                    
                    guides.shape = Some(legend);
                }
            }
        }
        
        guides
    }

    /// Calculate the required width for legends
    fn calculate_legend_width(&self) -> f64 {
        use crate::guide::LegendPosition;
        
        let mut total_width = 0.0;
        let legend_width = 120.0; // Base legend width
        let legend_spacing = 10.0;
        
        // Generate automatic legends to get accurate count
        let guides = self.generate_automatic_legends();
        
        // Check if we have any legends to display
        let mut legend_count = 0;
        
        if let Some(ref legend) = guides.color {
            if !matches!(legend.position, LegendPosition::None) {
                legend_count += 1;
            }
        }
        
        if let Some(ref legend) = guides.shape {
            if !matches!(legend.position, LegendPosition::None) {
                legend_count += 1;
            }
        }
        
        if let Some(ref legend) = guides.size {
            if !matches!(legend.position, LegendPosition::None) {
                legend_count += 1;
            }
        }
        
        if legend_count > 0 {
            // Add margin for legend placement (10px before legend, legend width, 10px after)
            total_width = legend_spacing + legend_width + legend_spacing;
        }
        
        total_width
    }

    /// Helper method to render using an existing Cairo context
    fn render_with_context(
        &self,
        ctx: &mut Context,
        width: i32,
        height: i32,
    ) -> Result<(), PlotError> {
        // Fill background
        Self::apply_fill_style(ctx, &self.theme.background.fill);
        ctx.paint()
            .map_err(|e| PlotError::ThemeError(format!("Failed to paint background: {}", e)))?;

        // Calculate required legend width and adjust right margin
        let legend_width = self.calculate_legend_width();
        
        // Determine axis positions
        use crate::guide::{AxisType, XAxisPosition, YAxisPosition};
        
        let x_position = self.guides.x_axis.as_ref()
            .and_then(|guide| match &guide.position {
                AxisType::X(pos) => Some(pos.clone()),
                _ => None,
            })
            .unwrap_or(XAxisPosition::Bottom);
        
        let y_position = self.guides.y_axis.as_ref()
            .and_then(|guide| match &guide.position {
                AxisType::Y(pos) => Some(pos.clone()),
                _ => None,
            })
            .unwrap_or(YAxisPosition::Left);
        
        // Get base margins from theme
        let theme_margin_left = self.theme.plot_margin.left as f64;
        let theme_margin_right = self.theme.plot_margin.right as f64;
        let theme_margin_top = self.theme.plot_margin.top as f64;
        let theme_margin_bottom = self.theme.plot_margin.bottom as f64;
        
        // Adjust margins based on axis positions
        // When axis moves to opposite side, use theme's opposite margin
        let margin_left = match y_position {
            YAxisPosition::Left => theme_margin_left,
            YAxisPosition::Right => theme_margin_right,
        };
        
        let mut margin_right = match y_position {
            YAxisPosition::Left => theme_margin_right,
            YAxisPosition::Right => theme_margin_left,
        };
        
        let margin_top = match x_position {
            XAxisPosition::Top => theme_margin_bottom,
            XAxisPosition::Bottom => theme_margin_top,
        };
        
        let margin_bottom = match x_position {
            XAxisPosition::Bottom => theme_margin_bottom,
            XAxisPosition::Top => theme_margin_top,
        };
        
        // Increase right margin if legends are present
        if legend_width > 0.0 {
            margin_right = f64::max(margin_right, legend_width);
        }

        let plot_x0 = margin_left;
        let plot_x1 = width as f64 - margin_right;
        let plot_y0 = margin_top;
        let plot_y1 = height as f64 - margin_bottom;

        // Draw panel background
        if let Some(ref panel_bg) = self.theme.panel.background {
            Self::apply_fill_style(ctx, panel_bg);
            ctx.rectangle(plot_x0, plot_y0, plot_x1 - plot_x0, plot_y1 - plot_y0);
            ctx.fill().ok();
        }

        // Draw grid lines (before border so border is on top)
        self.draw_grid_lines(ctx, plot_x0, plot_x1, plot_y0, plot_y1)?;

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

        // Draw legends
        self.draw_legends(ctx, plot_x0, plot_x1, plot_y0, plot_y1, width, height)?;

        Ok(())
    }
}

impl Default for Plot {
    fn default() -> Self {
        Self::new(None)
    }
}
