// Geom builder methods for Plot

use crate::aesthetics::AesMap;
use crate::data::DataSource;
use crate::geom::IntoLayer;
use crate::layer::Layer;

/// Trait for adding geom layers to a plot
pub trait GeomBuilder {
    /// Get mutable access to layers
    fn layers_mut(&mut self) -> &mut Vec<Layer>;

    /// Get access to default aesthetics
    fn default_aes(&self) -> &AesMap;

    /// Get mutable access to plot data
    fn data_mut(&mut self) -> &mut Option<Box<dyn DataSource>>;

    /// Merge default aesthetics into a layer
    fn merge_default_aesthetics(&self, layer: &mut Layer) {
        for (aesthetic, value) in self.default_aes().iter() {
            if layer.mapping.get(aesthetic).is_none() {
                layer.mapping.set(*aesthetic, value.clone());
            }
        }
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
    fn geom_point(self) -> Self
    where
        Self: Sized,
    {
        self.geom_point_with(|_layer| {})
    }

    /// Add a point geom layer with customization (builder style)
    fn geom_point_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut crate::plot::LayerGeom<crate::geom::point::GeomPoint>),
        Self: Sized,
    {
        let geom = crate::geom::point::GeomPoint::default();
        let mut layer_geom = crate::plot::LayerGeom::new(geom);
        f(&mut layer_geom);

        let (geom, layer_aes, stat) = layer_geom.into_parts();
        let mut layer = geom.into_layer();
        // Only override the stat if it's not Identity (preserve geom defaults)
        if !matches!(stat, crate::layer::Stat::Identity) {
            layer.stat = stat;
        }
        // Merge layer-specific aesthetics
        for (aesthetic, value) in layer_aes.iter() {
            layer.mapping.set(*aesthetic, value.clone());
        }
        self.merge_default_aesthetics(&mut layer);
        self.layers_mut().push(layer);
        self
    }

    /// Add a line geom layer with customization (builder style)
    fn geom_line_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut crate::plot::LayerGeom<crate::geom::line::GeomLine>),
        Self: Sized,
    {
        let geom = crate::geom::line::GeomLine::default();
        let mut layer_geom = crate::plot::LayerGeom::new(geom);
        f(&mut layer_geom);

        let (geom, layer_aes, stat) = layer_geom.into_parts();
        let mut layer = geom.into_layer();
        // Only override the stat if it's not Identity (preserve geom defaults)
        if !matches!(stat, crate::layer::Stat::Identity) {
            layer.stat = stat;
        }
        // Merge layer-specific aesthetics
        for (aesthetic, value) in layer_aes.iter() {
            layer.mapping.set(*aesthetic, value.clone());
        }
        self.merge_default_aesthetics(&mut layer);
        self.layers_mut().push(layer);
        self
    }

    /// Add a density geom layer with customization (builder style)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// plot.geom_density_with(|layer| {
    ///     layer.geom.color(color::BLUE)
    ///         .size(2.0)
    ///         .adjust(0.5);
    /// })
    /// ```
    fn geom_density_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut crate::plot::LayerGeom<crate::geom::density::GeomDensity>),
        Self: Sized,
    {
        let geom = crate::geom::density::GeomDensity::default();
        let mut layer_geom = crate::plot::LayerGeom::new(geom);
        f(&mut layer_geom);

        let (geom, layer_aes, stat) = layer_geom.into_parts();
        let mut layer = geom.into_layer();
        // Only override the stat if it's not Identity (preserve geom defaults)
        if !matches!(stat, crate::layer::Stat::Identity) {
            layer.stat = stat;
        }
        // Merge layer-specific aesthetics
        for (aesthetic, value) in layer_aes.iter() {
            layer.mapping.set(*aesthetic, value.clone());
        }
        self.merge_default_aesthetics(&mut layer);
        self.layers_mut().push(layer);
        self
    }

    /// Add a horizontal line geom layer with customization (builder style)
    ///
    /// The y-intercept should be specified via the aesthetic mapping using `.aes()`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // With a computed column
    /// plot.geom_hline_with(|layer| {
    ///     layer.aes(|a| a.yintercept("mean"));
    ///     layer.geom.color(color::RED).size(2.0);
    ///     layer.stat(Stat::Summary(vec![Aesthetic::Y]));
    /// })
    /// ```
    fn geom_hline_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut crate::plot::LayerGeom<crate::geom::hline::GeomHLine>),
        Self: Sized,
    {
        let geom = crate::geom::hline::GeomHLine::new();
        let mut layer_geom = crate::plot::LayerGeom::new(geom);
        f(&mut layer_geom);

        let (geom, layer_aes, stat) = layer_geom.into_parts();
        let mut layer = geom.into_layer();
        // Only override the stat if it's not Identity (preserve geom defaults)
        if !matches!(stat, crate::layer::Stat::Identity) {
            layer.stat = stat;
        }
        // Merge layer-specific aesthetics
        for (aesthetic, value) in layer_aes.iter() {
            layer.mapping.set(*aesthetic, value.clone());
        }
        self.merge_default_aesthetics(&mut layer);
        self.layers_mut().push(layer);
        self
    }

    /// Add a vertical line geom layer with customization (builder style)
    ///
    /// The x-intercept should be specified via the aesthetic mapping using `.aes()`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // With a computed column
    /// plot.geom_vline_with(|layer| {
    ///     layer.aes(|a| a.xintercept("mean"));
    ///     layer.geom.color(color::BLUE).size(2.0);
    ///     layer.stat(Stat::Summary(vec![Aesthetic::X]));
    /// })
    /// ```
    fn geom_vline_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut crate::plot::LayerGeom<crate::geom::vline::GeomVLine>),
        Self: Sized,
    {
        let geom = crate::geom::vline::GeomVLine::new();
        let mut layer_geom = crate::plot::LayerGeom::new(geom);
        f(&mut layer_geom);

        let (geom, layer_aes, stat) = layer_geom.into_parts();
        let mut layer = geom.into_layer();
        // Only override the stat if it's not Identity (preserve geom defaults)
        if !matches!(stat, crate::layer::Stat::Identity) {
            layer.stat = stat;
        }
        // Merge layer-specific aesthetics
        for (aesthetic, value) in layer_aes.iter() {
            layer.mapping.set(*aesthetic, value.clone());
        }
        self.merge_default_aesthetics(&mut layer);
        self.layers_mut().push(layer);
        self
    }

    /// Add a rectangle geom layer
    ///
    /// # Examples
    ///
    /// ```ignore
    /// plot.geom_rect()
    /// ```
    fn geom_rect(self) -> Self
    where
        Self: Sized,
    {
        self.geom_rect_with(|_layer| {})
    }

    /// Add a rectangle geom layer with customization (builder style)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// plot.geom_rect_with(|layer| {
    ///     layer.geom.fill(color::RED)
    ///         .color(color::BLACK)
    ///         .alpha(0.5);
    /// })
    /// ```
    fn geom_rect_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut crate::plot::LayerGeom<crate::geom::rect::GeomRect>),
        Self: Sized,
    {
        let geom = crate::geom::rect::GeomRect::new();
        let mut layer_geom = crate::plot::LayerGeom::new(geom);
        f(&mut layer_geom);

        let (geom, layer_aes, stat) = layer_geom.into_parts();
        let mut layer = geom.into_layer();
        // Only override the stat if it's not Identity (preserve geom defaults)
        if !matches!(stat, crate::layer::Stat::Identity) {
            layer.stat = stat;
        }
        // Merge layer-specific aesthetics
        for (aesthetic, value) in layer_aes.iter() {
            layer.mapping.set(*aesthetic, value.clone());
        }
        self.merge_default_aesthetics(&mut layer);
        self.layers_mut().push(layer);
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
    fn geom_segment(self) -> Self
    where
        Self: Sized,
    {
        self.geom_segment_with(|_layer| {})
    }

    /// Add a line segment geom layer with customization (builder style)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// plot.geom_segment_with(|layer| {
    ///     layer.geom.color(color::BLUE)
    ///         .size(2.0)
    ///         .alpha(0.8);
    /// })
    /// ```
    fn geom_segment_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut crate::plot::LayerGeom<crate::geom::segment::GeomSegment>),
        Self: Sized,
    {
        let geom = crate::geom::segment::GeomSegment::new();
        let mut layer_geom = crate::plot::LayerGeom::new(geom);
        f(&mut layer_geom);

        let (geom, layer_aes, stat) = layer_geom.into_parts();
        let mut layer = geom.into_layer();
        // Only override the stat if it's not Identity (preserve geom defaults)
        if !matches!(stat, crate::layer::Stat::Identity) {
            layer.stat = stat;
        }
        // Merge layer-specific aesthetics
        for (aesthetic, value) in layer_aes.iter() {
            layer.mapping.set(*aesthetic, value.clone());
        }
        self.merge_default_aesthetics(&mut layer);
        self.layers_mut().push(layer);
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
    fn geom_bar(self) -> Self
    where
        Self: Sized,
    {
        self.geom_bar_with(|_layer| {})
    }

    /// Add a bar geom layer with customization (builder style)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// plot.geom_bar_with(|layer| {
    ///     layer.geom.fill(color::BLUE)
    ///         .width(0.8)
    ///         .alpha(0.9);
    /// })
    /// ```
    fn geom_bar_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut crate::plot::LayerGeom<crate::geom::bar::GeomBar>),
        Self: Sized,
    {
        let geom = crate::geom::bar::GeomBar::new();
        let mut layer_geom = crate::plot::LayerGeom::new(geom);
        f(&mut layer_geom);

        let (geom, layer_aes, stat) = layer_geom.into_parts();
        let mut layer = geom.into_layer();
        // Only override the stat if it's not Identity (preserve geom defaults)
        if !matches!(stat, crate::layer::Stat::Identity) {
            layer.stat = stat;
        }
        // Merge layer-specific aesthetics
        for (aesthetic, value) in layer_aes.iter() {
            layer.mapping.set(*aesthetic, value.clone());
        }
        self.merge_default_aesthetics(&mut layer);

        // If layer needs stat transformation and doesn't have data, clone plot data
        // Stats need owned data to transform
        if !matches!(layer.stat, crate::layer::Stat::Identity) && layer.data.is_none() {
            if let Some(data) = self.data_mut() {
                layer.data = Some(data.clone_box());
            }
        }

        self.layers_mut().push(layer);
        self
    }

    /// Add a histogram geom layer (builder style)
    ///
    /// By default, uses Stat::Bin to bin continuous data and count observations.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// plot.geom_histogram()
    /// ```
    fn geom_histogram(self) -> Self
    where
        Self: Sized,
    {
        self.geom_histogram_with(|_layer| {})
    }

    /// Add a histogram geom layer with customization (builder style)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// plot.geom_histogram_with(|layer| {
    ///     layer.geom.fill(color::STEELBLUE)
    ///         .bins(20)
    ///         .alpha(0.8);
    /// })
    /// ```
    fn geom_histogram_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut crate::plot::LayerGeom<crate::geom::histogram::GeomHistogram>),
        Self: Sized,
    {
        let geom = crate::geom::histogram::GeomHistogram::new();
        let mut layer_geom = crate::plot::LayerGeom::new(geom);
        f(&mut layer_geom);

        let (geom, layer_aes, stat) = layer_geom.into_parts();
        let mut layer = geom.into_layer();
        // Only override the stat if it's not Identity (preserve geom defaults)
        if !matches!(stat, crate::layer::Stat::Identity) {
            layer.stat = stat;
        }
        // Merge layer-specific aesthetics
        for (aesthetic, value) in layer_aes.iter() {
            layer.mapping.set(*aesthetic, value.clone());
        }
        self.merge_default_aesthetics(&mut layer);

        // If layer needs stat transformation and doesn't have data, clone plot data
        // Stats need owned data to transform
        if !matches!(layer.stat, crate::layer::Stat::Identity) && layer.data.is_none() {
            if let Some(data) = self.data_mut() {
                layer.data = Some(data.clone_box());
            }
        }

        self.layers_mut().push(layer);
        self
    }

    /// Add a boxplot geom layer (builder style)
    ///
    /// By default, uses Stat::Boxplot to compute five-number summary statistics.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// plot.geom_boxplot()
    ///     .aes(|a| a.x("category").y("value"))
    /// ```
    fn geom_boxplot(self) -> Self
    where
        Self: Sized,
    {
        self.geom_boxplot_with(|_layer| {})
    }

    /// Add a boxplot geom layer with customization (builder style)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// plot.geom_boxplot_with(|layer| {
    ///     layer.geom.fill(color::STEELBLUE)
    ///         .width(0.5)
    ///         .alpha(0.8);
    /// })
    /// ```
    fn geom_boxplot_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut crate::plot::LayerGeom<crate::geom::boxplot::GeomBoxplot>),
        Self: Sized,
    {
        let geom = crate::geom::boxplot::GeomBoxplot::new();
        let mut layer_geom = crate::plot::LayerGeom::new(geom);
        f(&mut layer_geom);

        let (geom, layer_aes, stat) = layer_geom.into_parts();
        let mut layer = geom.into_layer();
        // Only override the stat if it's not Identity (preserve geom defaults)
        if !matches!(stat, crate::layer::Stat::Identity) {
            layer.stat = stat;
        }
        // Merge layer-specific aesthetics
        for (aesthetic, value) in layer_aes.iter() {
            layer.mapping.set(*aesthetic, value.clone());
        }
        self.merge_default_aesthetics(&mut layer);

        // If layer needs stat transformation and doesn't have data, clone plot data
        // Stats need owned data to transform
        if !matches!(layer.stat, crate::layer::Stat::Identity) && layer.data.is_none() {
            if let Some(data) = self.data_mut() {
                layer.data = Some(data.clone_box());
            }
        }

        self.layers_mut().push(layer);
        self
    }

    /// Add a text geom layer using default aesthetics
    ///
    /// # Examples
    ///
    /// ```ignore
    /// plot.geom_text()
    /// ```
    fn geom_text(self) -> Self
    where
        Self: Sized,
    {
        self.geom_text_with(|_layer| {})
    }

    /// Add a text geom layer with customization (builder style)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// plot.geom_text_with(|layer| {
    ///     layer.geom.size(12.0)
    ///         .color(color::BLACK)
    ///         .hjust(0.5)
    ///         .vjust(0.5);
    /// })
    /// ```
    fn geom_text_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut crate::plot::LayerGeom<crate::geom::text::GeomText>),
        Self: Sized,
    {
        let geom = crate::geom::text::GeomText::new();
        let mut layer_geom = crate::plot::LayerGeom::new(geom);
        f(&mut layer_geom);

        let (geom, layer_aes, stat) = layer_geom.into_parts();
        let mut layer = geom.into_layer();
        // Only override the stat if it's not Identity (preserve geom defaults)
        if !matches!(stat, crate::layer::Stat::Identity) {
            layer.stat = stat;
        }
        // Merge layer-specific aesthetics
        for (aesthetic, value) in layer_aes.iter() {
            layer.mapping.set(*aesthetic, value.clone());
        }
        self.merge_default_aesthetics(&mut layer);
        self.layers_mut().push(layer);
        self
    }

    /// Add a smooth geom layer with trend line and confidence interval
    ///
    /// # Example
    /// ```rust,ignore
    /// plot.geom_smooth()
    /// ```
    fn geom_smooth(self) -> Self
    where
        Self: Sized,
    {
        self.geom_smooth_with(|_layer| {})
    }

    /// Add a smooth geom layer with customization
    ///
    /// # Example
    /// ```rust,ignore
    /// plot.geom_smooth_with(|layer| {
    ///     layer.geom.se(false);  // Hide confidence interval
    ///     layer.geom.color(Color::rgb(255, 0, 0));  // Red line
    /// })
    /// ```
    fn geom_smooth_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(&mut crate::plot::LayerGeom<crate::geom::smooth::GeomSmooth>),
        Self: Sized,
    {
        let geom = crate::geom::smooth::GeomSmooth::new();
        let mut layer_geom = crate::plot::LayerGeom::new(geom);
        f(&mut layer_geom);

        let (geom, layer_aes, stat) = layer_geom.into_parts();
        let mut layer = geom.into_layer();
        // Only override the stat if it's not Identity (preserve geom defaults)
        if !matches!(stat, crate::layer::Stat::Identity) {
            layer.stat = stat;
        }
        // Merge layer-specific aesthetics
        for (aesthetic, value) in layer_aes.iter() {
            layer.mapping.set(*aesthetic, value.clone());
        }
        self.merge_default_aesthetics(&mut layer);

        // If layer needs stat transformation and doesn't have data, clone plot data
        // Stats need owned data to transform
        if !matches!(layer.stat, crate::layer::Stat::Identity) && layer.data.is_none() {
            if let Some(data) = self.data_mut() {
                layer.data = Some(data.clone_box());
            }
        }

        self.layers_mut().push(layer);
        self
    }
}
