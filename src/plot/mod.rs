// Plot structure for grammar of graphics

mod export;
mod geom_builder;
mod group_inference;
mod layer_geom;
mod positions;
mod render;
mod scale_application;
mod stats;

use crate::aesthetics::AesMap;
use crate::data::DataSource;
use crate::error::PlotError;
use crate::guide::{AxisGuide, Guides};
use crate::layer::Layer;
use crate::scale::ScaleSet;
use crate::theme::Theme;
use cairo::ImageSurface;
use std::path::Path;

pub use geom_builder::GeomBuilder;
pub use layer_geom::LayerGeom;

/// Main plot structure
pub struct Plot<'a> {
    /// Default data source for all layers
    pub data: &'a Box<dyn DataSource>,

    /// Default aesthetic mappings for all layers
    pub mapping: AesMap,

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

impl<'a> Plot<'a> {
    /// Create a new plot with optional default data
    pub fn new(data: &'a Box<dyn DataSource>) -> Self {
        Self {
            data,
            mapping: AesMap::new(),
            layers: Vec::new(),
            scales: ScaleSet::default(),
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
        f(&mut self.mapping);
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
            Some(self.data.as_ref()),
            &self.mapping,
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

        Ok(())
    }
}
