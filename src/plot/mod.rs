// Plot structure for grammar of graphics

mod export;
mod geom_builder;
mod group_inference;
mod layer_geom;
mod positions;
mod render;
mod scale_application;
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

pub use geom_builder::GeomBuilder;
pub use layer_geom::LayerGeom;
pub use scale_set::ScaleSet;

/// Main plot structure
pub struct Plot {
    /// Default data source for all layers
    pub data: Option<Box<dyn DataSource>>,

    /// Default aesthetic mappings for all layers
    pub mapping: crate::aesthetics::AesMap,

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
            mapping: crate::aesthetics::AesMap::new(),
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
        f(&mut self.mapping);
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

    /// Set the fill scale (builder style)
    pub fn scale_fill(mut self, scale: Box<dyn ColorScale>) -> Self {
        self.scales.fill = Some(scale);
        self
    }

    /// Set a continuous fill scale with custom gradient colors (builder style)
    pub fn scale_fill_continuous(mut self, colors: Vec<Color>) -> Self {
        use crate::scale::color::ContinuousColor;
        self.scales.fill = Some(Box::new(ContinuousColor::new((0.0, 1.0), colors)));
        self
    }

    /// Set a discrete fill scale with custom colors (builder style)
    pub fn scale_fill_manual(mut self, colors: Vec<Color>) -> Self {
        use crate::scale::color::DiscreteColor;
        self.scales.fill = Some(Box::new(DiscreteColor::new(colors)));
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
        // NEW PIPELINE: stat → geom.setup_data() → train_scales → infer_group → apply_scales → position → render

        // Steps 1-2: Apply stat transformations and geom.setup_data() for each layer
        for layer in &mut self.layers {
            // Step 1: Apply stat transformation to this layer
            stats::apply_stat_to_layer(layer, self.data.as_deref(), &self.mapping)?;

            // Step 2: Call geom.setup_data() to add required columns (e.g., xmin/xmax for bars)
            let plot_data = self.data.as_ref().map(|d| d.as_ref());
            let layer_data = layer.get_data(&plot_data);

            let layer_mapping = layer.get_mapping(&self.mapping);

            if let Some(data_ref) = layer_data {
                // Call setup_data
                let (new_data, new_mapping) = layer.geom.setup_data(data_ref, layer_mapping)?;
                
                if let Some(data) = new_data {
                    layer.computed_data = Some(data);
                }
                if let Some(mapping) = new_mapping {
                    layer.computed_mapping = Some(mapping);
                }
            }
        }

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

        // Step 3: Create default scales for unmapped aesthetics
        self.scales.create_defaults(
            &self.layers,
            self.data.as_ref().map(|d| d.as_ref()),
            &self.mapping,
            &mut x_axis_title,
            &mut y_axis_title,
        );

        // Step 4: Train scales (after geom.setup_data() so scales see all columns)
        self.scales
            .train(&self.layers, self.data.as_ref().map(|d| d.as_ref()), &self.mapping);

        // Step 5: Establish grouping for layers that need it
        use crate::plot::group_inference::establish_grouping;
        for layer in &mut self.layers {
            let layer_data = layer
                .computed_data
                .as_ref()
                .or(layer.data.as_ref())
                .or(self.data.as_ref());
            let layer_mapping = layer.computed_mapping.as_ref().unwrap_or(&self.mapping);

            if let Some(data_ref) = layer_data {
                let (maybe_new_data, maybe_new_mapping) =
                    establish_grouping(data_ref.as_ref(), layer_mapping);
                if let Some(new_data) = maybe_new_data {
                    layer.computed_data = Some(new_data);
                }
                if let Some(new_mapping) = maybe_new_mapping {
                    layer.computed_mapping = Some(new_mapping);
                }
            }
        }

        // Step 6: Apply scales to normalize data to [0,1]
        use crate::plot::scale_application::apply_scales;
        for layer in &mut self.layers {
            let layer_data = layer
                .computed_data
                .as_ref()
                .or(layer.data.as_ref())
                .or(self.data.as_ref());

            if let Some(data_ref) = layer_data {
                let mapping = layer.get_mapping(&self.mapping);

                // Apply scales and get normalized data
                let (normalized_df, normalized_mapping) =
                    apply_scales(data_ref.as_ref(), mapping, &self.scales)?;
                layer.computed_data = Some(Box::new(normalized_df));
                layer.computed_mapping = Some(normalized_mapping);
            }
        }

        // Step 7: Apply position adjustments (dodge/stack) on normalized [0,1] data
        positions::apply_positions(
            &mut self.layers,
            self.data.as_deref(),
            &self.mapping,
            &self.scales,
        )?;

        // Update axis guides with titles if they were auto-generated
        if x_axis_title.is_some() && self.guides.x_axis.is_none() {
            self.guides.x_axis = Some(AxisGuide::x().title(x_axis_title.unwrap()));
        }
        if y_axis_title.is_some() && self.guides.y_axis.is_none() {
            self.guides.y_axis = Some(AxisGuide::y().title(y_axis_title.unwrap()));
        }

        // Step 8: Render with pre-normalized and position-adjusted data
        export::save(
            path,
            &self.layers,
            &self.scales,
            &self.theme,
            &self.guides,
            self.title.as_ref(),
            self.data.as_ref().map(|d| d.as_ref()),
            &self.mapping,
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
        &self.mapping
    }

    fn data_mut(&mut self) -> &mut Option<Box<dyn DataSource>> {
        &mut self.data
    }
}
