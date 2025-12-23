// Plot structure for grammar of graphics

mod export;
mod geom_builder;
mod group_inference;
mod layer_geom;
mod positions;
mod render;
mod stats;

use crate::aesthetics::{AesMap, AestheticProperty};
use crate::data::DataSource;
use crate::error::PlotError;
use crate::aesthetics::builder::AesMapBuilder;
use crate::guide::{AxisGuide, Guides};
use crate::layer::{Layer, LayerBuilder};
use crate::scale::ScaleSet;
use crate::theme::Theme;
use cairo::ImageSurface;
use std::ops::Add;
use std::path::Path;

pub use geom_builder::GeomBuilder;
pub use layer_geom::LayerGeom;

pub struct PlotBuilder<'a> {
    data: &'a Box<dyn DataSource>,
    mapping: AesMap,
    layers: Vec<Box<dyn LayerBuilder>>,
    guides: Guides,
    theme: Theme,
    title: Option<String>,
}

impl<'a> PlotBuilder<'a> {
    pub fn aes(self, closure: impl FnOnce(&mut AesMapBuilder)) -> Self {
        let empty = AesMap::new();
        let mut builder = AesMapBuilder::new();
        closure(&mut builder);
        Self {
            data: self.data,
            mapping: builder.build(&empty),
            layers: self.layers,
            guides: self.guides,
            theme: self.theme,
            title: self.title,
        }
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

    pub fn build(self) -> Result<Plot<'a>, PlotError> {
        let mut layers: Vec<Layer> = self
            .layers
            .into_iter()
            .map(|builder| builder.build(&self.mapping))
            .collect();

        let mut scales = ScaleSet::default();

        // Step 1: Apply stat transformations to each layer
        for layer in &mut layers {
            layer.apply_stat(&self.data, &self.mapping)?;
        }

        // Step 2: Apply position adjustments across layers
        positions::apply_positions(
            &mut layers,
            Some(self.data.as_ref()),
            &self.mapping,
            &scales,
        )?;

        // Step 3: Train scales on all layer data
        for layer in &mut layers {
            layer.train_scales(&mut scales, &self.data, &self.mapping)?;
        }

        // Step 4: Apply scales to convert data to visual coordinates
        for layer in &mut layers {
            layer.apply_scales(&scales, self.data, &self.mapping)?;
        }

        let mut required_scales = Vec::new();
        for layer in &layers {
            let geom = layer.geom.as_ref();
            required_scales.extend(geom.required_scales());
        }
        required_scales.sort();
        required_scales.dedup();
        log::info!("Required scales: {:?}", required_scales);

        scales.x_continuous.compute_breaks(5);
        scales.y_continuous.compute_breaks(5);

        // Populate default axis and legend labels from aesthetic mappings
        let mut guides = self.guides;
        
        // Set default X axis label if not already set by user
        if guides.x_axis.is_none() {
            if let Some(x_label) = self.mapping.get_label(AestheticProperty::X) {
                guides.x_axis = Some(AxisGuide::x().title(x_label));
            }
        }
        
        // Set default Y axis label if not already set by user
        if guides.y_axis.is_none() {
            if let Some(y_label) = self.mapping.get_label(AestheticProperty::Y) {
                guides.y_axis = Some(AxisGuide::y().title(y_label));
            }
        }

        Ok(Plot {
            data: self.data,
            mapping: self.mapping,
            layers,
            scales,
            theme: self.theme,
            guides,
            title: self.title,
        })
    }
}

pub fn plot<'a>(data: &'a Box<dyn DataSource>) -> PlotBuilder<'a> {
    PlotBuilder {
        data,
        mapping: AesMap::new(),
        layers: Vec::new(),
        guides: Guides::default(),
        theme: Theme::default(),
        title: None,
    }
}

impl<'a> Add<AesMapBuilder> for PlotBuilder<'a> {
    type Output = Self;

    fn add(mut self, rhs: AesMapBuilder) -> Self::Output {
        self.mapping = rhs.build(&self.mapping);
        self
    }
}

impl<'a, L: LayerBuilder + 'static> Add<L> for PlotBuilder<'a> {
    type Output = Self;

    fn add(mut self, rhs: L) -> Self::Output {
        self.layers.push(Box::new(rhs));
        self
    }
}

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
    pub fn save(&self, path: impl AsRef<Path>, width: i32, height: i32) -> Result<(), PlotError> {
        export::save(
            path,
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
}
