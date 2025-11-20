// Plot structure for grammar of graphics

use crate::data::DataSource;
use crate::error::PlotError;
use crate::geom::{IntoLayer, RenderContext};
use crate::guide::Guides;
use crate::layer::Layer;
use crate::scale::{ColorScale, ContinuousScale, ShapeScale};
use crate::stat::count::Count;
use crate::stat::StatTransform;
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
    /// // Customize the point geom
    /// plot.geom_point_with(|geom| {
    ///     geom.size(3.0).color(color::BLUE)
    /// })
    /// ```
    pub fn geom_point(self) -> Self {
        self.geom_point_with(|geom| geom)
    }

    /// Add a point geom layer with customization (builder style)
    pub fn geom_point_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(crate::geom::point::GeomPoint) -> crate::geom::point::GeomPoint,
    {
        let geom = crate::geom::point::GeomPoint::default();
        let geom = f(geom);
        
        let mut layer = geom.into_layer();
        for (aesthetic, value) in self.default_aes.iter() {
            if !layer.mapping.get(aesthetic).is_some() {
                layer.mapping.set(aesthetic.clone(), value.clone());
            }
        }
        self.layers.push(layer);
        self
    }

    /// Add a line geom layer with customization (builder style)
    pub fn geom_line_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(crate::geom::line::GeomLine) -> crate::geom::line::GeomLine,
    {
        let geom = crate::geom::line::GeomLine::default();
        let geom = f(geom);
        
        let mut layer = geom.into_layer();
        for (aesthetic, value) in self.default_aes.iter() {
            if !layer.mapping.get(aesthetic).is_some() {
                layer.mapping.set(aesthetic.clone(), value.clone());
            }
        }
        self.layers.push(layer);
        self
    }

    /// Add a horizontal line geom layer with customization (builder style)
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// plot.geom_hline_with(4.0, |geom| {
    ///     geom.color(color::RED)
    ///         .size(2.0)
    ///         .linetype("-")
    /// })
    /// ```
    pub fn geom_hline_with<F>(mut self, yintercept: f64, f: F) -> Self
    where
        F: FnOnce(crate::geom::hline::GeomHLine) -> crate::geom::hline::GeomHLine,
    {
        let geom = crate::geom::hline::GeomHLine::new(yintercept);
        let geom = f(geom);
        
        let mut layer = geom.into_layer();
        for (aesthetic, value) in self.default_aes.iter() {
            if !layer.mapping.get(aesthetic).is_some() {
                layer.mapping.set(aesthetic.clone(), value.clone());
            }
        }
        self.layers.push(layer);
        self
    }

    /// Add a vertical line geom layer with customization (builder style)
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// plot.geom_vline_with(5.0, |geom| {
    ///     geom.color(color::BLUE)
    ///         .size(2.0)
    ///         .linetype(".")
    /// })
    /// ```
    pub fn geom_vline_with<F>(mut self, xintercept: f64, f: F) -> Self
    where
        F: FnOnce(crate::geom::vline::GeomVLine) -> crate::geom::vline::GeomVLine,
    {
        let geom = crate::geom::vline::GeomVLine::new(xintercept);
        let geom = f(geom);
        
        let mut layer = geom.into_layer();
        for (aesthetic, value) in self.default_aes.iter() {
            if !layer.mapping.get(aesthetic).is_some() {
                layer.mapping.set(aesthetic.clone(), value.clone());
            }
        }
        self.layers.push(layer);
        self
    }

    /// Add a rectangle geom layer
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// plot.geom_rect()
    /// ```
    pub fn geom_rect(self) -> Self {
        self.geom_rect_with(|geom| geom)
    }

    /// Add a rectangle geom layer with customization (builder style)
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// plot.geom_rect_with(|geom| {
    ///     geom.fill(color::RED)
    ///         .color(color::BLACK)
    ///         .alpha(0.5)
    /// })
    /// ```
    pub fn geom_rect_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(crate::geom::rect::GeomRect) -> crate::geom::rect::GeomRect,
    {
        let geom = crate::geom::rect::GeomRect::new();
        let geom = f(geom);
        
        let mut layer = geom.into_layer();
        for (aesthetic, value) in self.default_aes.iter() {
            if !layer.mapping.get(aesthetic).is_some() {
                layer.mapping.set(aesthetic.clone(), value.clone());
            }
        }
        self.layers.push(layer);
        self
    }

    /// Add a line segment geom layer using default aesthetics
    /// 
    /// Draws line segments from (x, y) to (xend, yend).
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// plot.geom_segment()
    /// ```
    pub fn geom_segment(self) -> Self {
        self.geom_segment_with(|geom| geom)
    }

    /// Add a line segment geom layer with customization (builder style)
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// plot.geom_segment_with(|geom| {
    ///     geom.color(color::BLUE)
    ///         .size(2.0)
    ///         .alpha(0.8)
    /// })
    /// ```
    pub fn geom_segment_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(crate::geom::segment::GeomSegment) -> crate::geom::segment::GeomSegment,
    {
        let geom = crate::geom::segment::GeomSegment::new();
        let geom = f(geom);
        
        let mut layer = geom.into_layer();
        for (aesthetic, value) in self.default_aes.iter() {
            if !layer.mapping.get(aesthetic).is_some() {
                layer.mapping.set(aesthetic.clone(), value.clone());
            }
        }
        self.layers.push(layer);
        self
    }

    /// Add a bar geom layer (builder style)
    /// 
    /// By default, uses Stat::Count to count occurrences at each x position.
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// plot.geom_bar()
    /// ```
    pub fn geom_bar(self) -> Self {
        self.geom_bar_with(|geom| geom)
    }

    /// Add a bar geom layer with customization (builder style)
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// plot.geom_bar_with(|geom| {
    ///     geom.fill(color::BLUE)
    ///         .width(0.8)
    ///         .alpha(0.9)
    /// })
    /// ```
    pub fn geom_bar_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(crate::geom::bar::GeomBar) -> crate::geom::bar::GeomBar,
    {
        let geom = crate::geom::bar::GeomBar::new();
        let geom = f(geom);
        
        let mut layer = geom.into_layer();
        for (aesthetic, value) in self.default_aes.iter() {
            if !layer.mapping.get(aesthetic).is_some() {
                layer.mapping.set(aesthetic.clone(), value.clone());
            }
        }
        
        // If layer needs stat transformation and doesn't have data, take plot data
        // Stats need owned data to transform
        if !matches!(layer.stat, crate::layer::Stat::Identity) && layer.data.is_none() {
            layer.data = self.data.take();
        }
        
        self.layers.push(layer);
        self
    }

    /// Set the x scale (builder style)
    pub fn scale_x(mut self, scale: Box<dyn ContinuousScale>) -> Self {
        self.scales.x = Some(scale);
        self
    }

    /// Customize the x scale using a builder function
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// plot.x_scale_with(|scale| scale.set_lower_bound(0.0))
    /// ```
    pub fn x_scale_with<F>(self, f: F) -> Self
    where
        F: FnOnce(crate::scale::continuous::Continuous) -> crate::scale::continuous::Continuous,
    {
        let builder = crate::scale::continuous::Continuous::new();
        let builder = f(builder);
        // Assume linear scale for now; could be extended
        if let Ok(scale) = builder.linear() {
            self.scale_x(Box::new(scale))
        } else {
            self
        }
    }

    /// Set the y scale (builder style)
    pub fn scale_y(mut self, scale: Box<dyn ContinuousScale>) -> Self {
        self.scales.y = Some(scale);
        self
    }

    /// Customize the y scale using a builder function
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// plot.y_scale_with(|scale| scale.set_lower_bound(0.0))
    /// ```
    pub fn y_scale_with<F>(self, f: F) -> Self
    where
        F: FnOnce(crate::scale::continuous::Continuous) -> crate::scale::continuous::Continuous,
    {
        let builder = crate::scale::continuous::Continuous::new();
        let builder = f(builder);
        // Assume linear scale for now; could be extended
        if let Ok(scale) = builder.linear() {
            self.scale_y(Box::new(scale))
        } else {
            self
        }
    }

    /// Set the color scale (builder style)
    pub fn scale_color(mut self, scale: Box<dyn ColorScale>) -> Self {
        self.scales.color = Some(scale);
        self
    }

    /// Set a continuous color scale with custom gradient colors (builder style)
    /// 
    /// # Examples
    /// 
    /// ```ignore
    /// // Use default blue-to-black gradient
    /// plot.scale_color_continuous(vec![
    ///     Color::rgb(0, 0, 139),   // dark blue
    ///     Color::rgb(0, 0, 0),     // black
    /// ])
    /// 
    /// // Use a custom three-color gradient
    /// plot.scale_color_continuous(vec![
    ///     Color::rgb(0, 0, 255),   // blue
    ///     Color::rgb(255, 255, 0), // yellow
    ///     Color::rgb(255, 0, 0),   // red
    /// ])
    /// ```
    pub fn scale_color_continuous(mut self, colors: Vec<Color>) -> Self {
        use crate::scale::color::ContinuousColor;
        self.scales.color = Some(Box::new(ContinuousColor::new((0.0, 1.0), colors)));
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

    /// Apply statistical transformations to layers
    /// 
    /// This method transforms layers that have stats other than Identity.
    /// The transformed data is stored in each layer's data field, and the
    /// aesthetic mapping is updated to reflect the transformation.
    /// 
    /// Note: Currently, this only works for layers that have their own data.
    /// Layers that rely on plot-level data cannot be transformed yet.
    fn apply_stats(&mut self) -> Result<(), PlotError> {
        use crate::layer::Stat;
        
        // We need to transform each layer that has a non-Identity stat
        // We'll process layers in reverse order so we can swap data out
        let num_layers = self.layers.len();
        for i in 0..num_layers {
            // Check if this layer needs transformation
            let needs_transform = !matches!(self.layers[i].stat, Stat::Identity);
            
            if !needs_transform {
                continue;
            }

            // If layer doesn't have data, it can't be transformed
            // (stat transformations need owned data)
            if self.layers[i].data.is_none() {
                continue;
            }

            // Take ownership of the layer's data
            let data = self.layers[i].data.take().unwrap();
            
            // Apply the stat transformation
            let stat_result = match &self.layers[i].stat {
                Stat::Count => Count.apply(data, &self.layers[i].mapping)?,
                Stat::Identity => {
                    // Put the data back and continue
                    self.layers[i].data = Some(data);
                    continue;
                }
                Stat::Bin | Stat::Smooth => {
                    // Not implemented yet - put data back
                    self.layers[i].data = Some(data);
                    continue;
                }
            };

            // If transformation succeeded, store the result
            if let Some((transformed_data, new_mapping)) = stat_result {
                self.layers[i].data = Some(transformed_data);
                
                // Replace the layer's mapping with the new mapping from the stat
                // The stat knows best what the transformed data looks like
                self.layers[i].mapping = new_mapping;
            } else {
                // No transformation needed - data is already back in place from match arm
            }
        }

        Ok(())
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
        // Apply stat transformations to layers
        self.apply_stats()?;
        
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
            
            use crate::guide::LegendType;
            
            // Draw legend background
            Self::apply_fill_style(ctx, &self.theme.legend.background);
            let legend_width = 120.0;
            let item_height = 20.0;
            let padding = 10.0;
            let title_height = if legend.title.is_some() { 20.0 } else { 0.0 };
            
            let legend_height = match &legend.legend_type {
                LegendType::Discrete => {
                    title_height + (legend.entries.len() as f64 * item_height) + padding * 2.0
                }
                LegendType::ColorBar { .. } => {
                    title_height + 150.0 + padding * 2.0  // Fixed height for color bar
                }
            };
            
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
            
            // Draw legend content based on type
            match &legend.legend_type {
                LegendType::Discrete => {
                    // Draw discrete legend entries
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
                }
                LegendType::ColorBar { domain, colors } => {
                    // Draw continuous color bar
                    let bar_x = legend_x + padding + 10.0;
                    let bar_width = 20.0;
                    let bar_height = 120.0;
                    let bar_y = item_y;
                    
                    // Draw color gradient
                    let n_segments = colors.len().max(2) - 1;
                    let segment_height = bar_height / n_segments as f64;
                    
                    for i in 0..n_segments {
                        let y = bar_y + i as f64 * segment_height;
                        
                        if i < colors.len() - 1 {
                            // Create gradient pattern for this segment
                            let color1 = colors[i];
                            let color2 = colors[i + 1];
                            
                            // For simplicity, just fill with interpolated colors
                            let steps = 10;
                            for j in 0..steps {
                                let t = j as f64 / steps as f64;
                                let r = (color1.0 as f64 * (1.0 - t) + color2.0 as f64 * t) as u8;
                                let g = (color1.1 as f64 * (1.0 - t) + color2.1 as f64 * t) as u8;
                                let b = (color1.2 as f64 * (1.0 - t) + color2.2 as f64 * t) as u8;
                                let a = (color1.3 as f64 * (1.0 - t) + color2.3 as f64 * t) as u8;
                                
                                Self::apply_color(ctx, &Color(r, g, b, a));
                                ctx.rectangle(
                                    bar_x,
                                    y + j as f64 * segment_height / steps as f64,
                                    bar_width,
                                    segment_height / steps as f64 + 0.5
                                );
                                ctx.fill().ok();
                            }
                        }
                    }
                    
                    // Draw border around color bar
                    Self::apply_line_style(ctx, &self.theme.legend.border);
                    ctx.rectangle(bar_x, bar_y, bar_width, bar_height);
                    ctx.stroke().ok();
                    
                    // Draw tick marks and labels with 5 evenly spaced breaks
                    let label_x = bar_x + bar_width + 5.0;
                    let num_breaks = 5;
                    
                    Self::apply_font(ctx, &self.theme.legend.text_font);
                    Self::apply_color(ctx, &self.theme.legend.text_color);
                    ctx.set_font_size(9.0);
                    
                    for i in 0..num_breaks {
                        let t = i as f64 / (num_breaks - 1) as f64;
                        let value = domain.0 + t * (domain.1 - domain.0);
                        // Calculate position on the bar
                        let t = (value - domain.0) / (domain.1 - domain.0);
                        let tick_y = bar_y + bar_height - t * bar_height;
                        
                        // Draw tick mark
                        ctx.move_to(bar_x + bar_width, tick_y);
                        ctx.line_to(bar_x + bar_width + 3.0, tick_y);
                        ctx.stroke().ok();
                        
                        // Draw label - format intelligently
                        ctx.move_to(label_x + 3.0, tick_y + 3.0);
                        let label = if value.abs() < 1e-10 {
                            // Treat very small values as zero
                            "0".to_string()
                        } else if (value - value.round()).abs() < 0.01 {
                            // Value is close to an integer - show it as an integer
                            format!("{}", value.round() as i64)
                        } else if value.abs() < 0.01 || value.abs() > 10000.0 {
                            format!("{:.2e}", value)
                        } else {
                            format!("{:.2}", value)
                        };
                        ctx.show_text(&label).ok();
                    }
                }
            }
            
            legend_y += legend_height + 10.0;
        }
        
        Ok(())
    }

    /// Helper to draw a shape at a position
    fn draw_shape(ctx: &mut Context, x: f64, y: f64, size: f64, shape: crate::visuals::Shape) {
        shape.draw(ctx, x, y, size);
    }

    /// Create default scales for aesthetics that don't have scales but are mapped to columns
    fn create_default_scales(&mut self) {
        use crate::aesthetics::{Aesthetic, AesValue};
        use crate::scale::continuous::Continuous;
        use crate::scale::color::DiscreteColor;
        use crate::scale::shape::DiscreteShape;
        
        // Find the first layer that maps each aesthetic to determine default scales
        for layer in &self.layers {
            // X scale
            if self.scales.x.is_none() {
                let col_name = layer.mapping.get(&Aesthetic::X)
                    .or_else(|| layer.mapping.get(&Aesthetic::XBegin))
                    .or_else(|| layer.mapping.get(&Aesthetic::XEnd));
                    
                if let Some(AesValue::Column(col_name)) = col_name {
                    // Check if this column is categorical (string type)
                    let data = match &layer.data {
                        Some(d) => Some(d.as_ref()),
                        None => self.data.as_ref().map(|d| d.as_ref()),
                    };
                    
                    let is_categorical = data
                        .and_then(|d| d.get(col_name))
                        .map(|col| col.as_str().is_some())
                        .unwrap_or(false);
                    
                    if is_categorical {
                        // Create categorical scale - we'll train it later
                        use crate::scale::categorical::Catagorical;
                        use std::collections::HashMap;
                        self.scales.x = Some(Box::new(Catagorical::new(HashMap::new())));
                    } else {
                        // Create default linear scale
                        if let Ok(scale) = Continuous::new().linear() {
                            self.scales.x = Some(Box::new(scale));
                        }
                    }
                    
                    // Set default axis title if not already set
                    if self.guides.x_axis.is_none() {
                        self.guides.x_axis = Some(crate::guide::AxisGuide::x().title(col_name.clone()));
                    }
                }
            }
            
            // Y scale
            if self.scales.y.is_none() {
                let col_name = layer.mapping.get(&Aesthetic::Y)
                    .or_else(|| layer.mapping.get(&Aesthetic::YBegin))
                    .or_else(|| layer.mapping.get(&Aesthetic::YEnd));
                    
                if let Some(AesValue::Column(col_name)) = col_name {
                    // Check if this column is categorical (string type)
                    let data = match &layer.data {
                        Some(d) => Some(d.as_ref()),
                        None => self.data.as_ref().map(|d| d.as_ref()),
                    };
                    
                    let is_categorical = data
                        .and_then(|d| d.get(col_name))
                        .map(|col| col.as_str().is_some())
                        .unwrap_or(false);
                    
                    if is_categorical {
                        // Create categorical scale - we'll train it later
                        use crate::scale::categorical::Catagorical;
                        use std::collections::HashMap;
                        self.scales.y = Some(Box::new(Catagorical::new(HashMap::new())));
                    } else {
                        // Create default linear scale
                        if let Ok(scale) = Continuous::new().linear() {
                            self.scales.y = Some(Box::new(scale));
                        }
                    }
                    
                    // Set default axis title if not already set
                    if self.guides.y_axis.is_none() {
                        self.guides.y_axis = Some(crate::guide::AxisGuide::y().title(col_name.clone()));
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
                    if let Ok(scale) = Continuous::new().linear() {
                        self.scales.size = Some(Box::new(scale));
                    }
                }
            }
            
            // Alpha scale
            if self.scales.alpha.is_none() {
                if let Some(AesValue::Column(_)) = layer.mapping.get(&Aesthetic::Alpha) {
                    // Create default linear scale for alpha
                    if let Ok(scale) = Continuous::new().linear() {
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
            
            // Collect all x-related vectors (X, XBegin, XEnd)
            let mut x_vecs = Vec::new();
            if let Some(AesValue::Column(col_name)) = layer.mapping.get(&Aesthetic::X) {
                if let Some(vec) = data.get(col_name) {
                    x_vecs.push(vec);
                }
            }
            if let Some(AesValue::Column(col_name)) = layer.mapping.get(&Aesthetic::XBegin) {
                if let Some(vec) = data.get(col_name) {
                    x_vecs.push(vec);
                }
            }
            if let Some(AesValue::Column(col_name)) = layer.mapping.get(&Aesthetic::XEnd) {
                if let Some(vec) = data.get(col_name) {
                    x_vecs.push(vec);
                }
            }
            
            // Train x scale on all x-related data
            if !x_vecs.is_empty() {
                if let Some(ref mut scale) = self.scales.x {
                    scale.train(&x_vecs);
                }
            }
            
            // Collect all y-related vectors (Y, YBegin, YEnd)
            let mut y_vecs = Vec::new();
            if let Some(AesValue::Column(col_name)) = layer.mapping.get(&Aesthetic::Y) {
                if let Some(vec) = data.get(col_name) {
                    y_vecs.push(vec);
                }
            }
            if let Some(AesValue::Column(col_name)) = layer.mapping.get(&Aesthetic::YBegin) {
                if let Some(vec) = data.get(col_name) {
                    y_vecs.push(vec);
                }
            }
            if let Some(AesValue::Column(col_name)) = layer.mapping.get(&Aesthetic::YEnd) {
                if let Some(vec) = data.get(col_name) {
                    y_vecs.push(vec);
                }
            }
            
            // Train y scale on all y-related data
            if !y_vecs.is_empty() {
                if let Some(ref mut scale) = self.scales.y {
                    scale.train(&y_vecs);
                }
            }
            
            // Train color scale
            if let Some(AesValue::Column(col_name)) = layer.mapping.get(&Aesthetic::Color) {
                if let Some(ref mut scale) = self.scales.color {
                    if let Some(vec) = data.get(col_name) {
                        scale.train(&[vec]);
                    }
                }
            }
            
            // Train shape scale
            if let Some(AesValue::Column(col_name)) = layer.mapping.get(&Aesthetic::Shape) {
                if let Some(ref mut scale) = self.scales.shape {
                    if let Some(vec) = data.get(col_name) {
                        scale.train(&[vec]);
                    }
                }
            }
            
            // Train size scale
            if let Some(AesValue::Column(col_name)) = layer.mapping.get(&Aesthetic::Size) {
                if let Some(ref mut scale) = self.scales.size {
                    if let Some(vec) = data.get(col_name) {
                        scale.train(&[vec]);
                    }
                }
            }
            
            // Train alpha scale
            if let Some(AesValue::Column(col_name)) = layer.mapping.get(&Aesthetic::Alpha) {
                if let Some(ref mut scale) = self.scales.alpha {
                    if let Some(vec) = data.get(col_name) {
                        scale.train(&[vec]);
                    }
                }
            }
        }
    }

    /// Generate legends automatically from scales when aesthetics are mapped
    fn generate_automatic_legends(&self) -> Guides {
        use crate::aesthetics::{Aesthetic, AesValue};
        use crate::guide::{LegendGuide, LegendEntry, LegendPosition};
        use crate::visuals::Shape;
        
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
                use crate::guide::LegendType;
                
                let mut legend = LegendGuide::new();
                legend.position = LegendPosition::Right;
                
                // Get the column name for the title
                for layer in &self.layers {
                    if let Some(AesValue::Column(col_name)) = layer.mapping.get(&Aesthetic::Color) {
                        legend.title = Some(col_name.clone());
                        break;
                    }
                }
                
                if color_scale.is_continuous() {
                    // Create a continuous color bar
                    if let Some(domain) = color_scale.get_continuous_domain() {
                        // Sample colors across the domain
                        let n_samples = 50;
                        let mut colors = Vec::new();
                        for i in 0..=n_samples {
                            let t = i as f64 / n_samples as f64;
                            let value = domain.0 + t * (domain.1 - domain.0);
                            if let Some(color) = color_scale.map_continuous_to_color(value) {
                                colors.push(color);
                            }
                        }
                        
                        legend.legend_type = LegendType::ColorBar { domain, colors };
                        guides.color = Some(legend);
                    }
                } else {
                    // Create discrete legend entries
                    let breaks = color_scale.legend_breaks();
                    if !breaks.is_empty() {
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
                        if let Some(shape) = shape_scale.map_to_shape(&category) {
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
