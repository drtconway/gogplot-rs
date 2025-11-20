// Plot structure for grammar of graphics

mod export;
mod geom_builder;
mod render;
mod scale_set;
mod stats;

use crate::data::DataSource;
use crate::error::PlotError;
use crate::guide::{AxisGuide, Guides};
use crate::layer::Layer;
use crate::scale::{ColorScale, ContinuousScale, ShapeScale};
use crate::theme::{Color, Theme};
use cairo::ImageSurface;
use std::path::Path;

pub use scale_set::ScaleSet;
pub use geom_builder::GeomBuilder;

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
        render::render(
            &self.layers,
            &self.scales,
            &self.theme,
            &self.guides,
            self.title.as_ref(),
            self.data.as_ref().map(|d| d.as_ref()),
            width,
            height,
        )
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
    pub fn save(
        mut self,
        path: impl AsRef<Path>,
        width: i32,
        height: i32,
    ) -> Result<(), PlotError> {
        // Apply stat transformations to layers
        stats::apply_stats(&mut self.layers)?;

        // Create axis titles from mapped columns if not already set
        let mut x_axis_title = self
            .guides
            .x_axis
            .as_ref()
            .and_then(|axis| axis.title.clone());
        let mut y_axis_title = self
            .guides
            .y_axis
            .as_ref()
            .and_then(|axis| axis.title.clone());

        // Create default scales for unmapped aesthetics
        self.scales.create_defaults(
            &self.layers,
            self.data.as_ref().map(|d| d.as_ref()),
            &mut x_axis_title,
            &mut y_axis_title,
        );

        // Update axis guides with titles if they were auto-generated
        if x_axis_title.is_some() && self.guides.x_axis.is_none() {
            self.guides.x_axis = Some(AxisGuide::x().title(x_axis_title.unwrap()));
        }
        if y_axis_title.is_some() && self.guides.y_axis.is_none() {
            self.guides.y_axis = Some(AxisGuide::y().title(y_axis_title.unwrap()));
        }

        // Train scales on data before rendering
        self.scales
            .train(&self.layers, self.data.as_ref().map(|d| d.as_ref()));

        export::save(
            path,
            &self.layers,
            &self.scales,
            &self.theme,
            &self.guides,
            self.title.as_ref(),
            self.data.as_ref().map(|d| d.as_ref()),
            width,
            height,
        )
    }
}

impl Default for Plot {
    fn default() -> Self {
        Self::new(None)
    }
}

// Implement GeomBuilder trait for Plot to add all geom_* methods
impl GeomBuilder for Plot {
    fn layers_mut(&mut self) -> &mut Vec<Layer> {
        &mut self.layers
    }

    fn default_aes(&self) -> &crate::aesthetics::AesMap {
        &self.default_aes
    }

    fn data_mut(&mut self) -> &mut Option<Box<dyn DataSource>> {
        &mut self.data
    }
}
