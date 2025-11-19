// Plot structure for grammar of graphics

use crate::data::DataSource;
use crate::layer::Layer;
use crate::scale::{ContinuousScale, ColorScale, ShapeScale};
use crate::theme::Theme;
use crate::error::PlotError;
use crate::geom::RenderContext;
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
        let extension = path.extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| PlotError::ThemeError("Invalid file path".to_string()))?;
        
        match extension.to_lowercase().as_str() {
            "png" => {
                let surface = self.render(width, height)?;
                let mut file = std::fs::File::create(path)
                    .map_err(|e| PlotError::ThemeError(format!("Failed to create file: {}", e)))?;
                surface.write_to_png(&mut file)
                    .map_err(|e| PlotError::ThemeError(format!("Failed to write PNG: {}", e)))?;
            }
            "svg" => {
                let surface = SvgSurface::new(width as f64, height as f64, Some(path))
                    .map_err(|e| PlotError::ThemeError(format!("Failed to create SVG surface: {}", e)))?;
                
                let mut ctx = Context::new(&surface)
                    .map_err(|e| PlotError::ThemeError(format!("Failed to create context: {}", e)))?;
                self.render_with_context(&mut ctx, width, height)?;
                surface.finish();
            }
            "pdf" => {
                let surface = PdfSurface::new(width as f64, height as f64, path)
                    .map_err(|e| PlotError::ThemeError(format!("Failed to create PDF surface: {}", e)))?;
                
                let mut ctx = Context::new(&surface)
                    .map_err(|e| PlotError::ThemeError(format!("Failed to create context: {}", e)))?;
                self.render_with_context(&mut ctx, width, height)?;
                surface.finish();
            }
            _ => {
                return Err(PlotError::ThemeError(
                    format!("Unsupported file format: {}", extension)
                ));
            }
        }
        
        Ok(())
    }
    
    /// Helper method to render using an existing Cairo context
    fn render_with_context(&self, ctx: &mut Context, width: i32, height: i32) -> Result<(), PlotError> {
        // Fill background
        ctx.set_source_rgb(1.0, 1.0, 1.0);
        ctx.paint()
            .map_err(|e| PlotError::ThemeError(format!("Failed to paint background: {}", e)))?;
        
        // Define plot area
        let margin_left = 60.0;
        let margin_right = 20.0;
        let margin_top = 40.0;
        let margin_bottom = 60.0;
        
        let plot_x0 = margin_left;
        let plot_x1 = width as f64 - margin_right;
        let plot_y0 = margin_top;
        let plot_y1 = height as f64 - margin_bottom;
        
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
