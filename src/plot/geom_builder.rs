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
                layer.mapping.set(aesthetic.clone(), value.clone());
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
        self.geom_point_with(|geom| geom)
    }

    /// Add a point geom layer with customization (builder style)
    fn geom_point_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(crate::geom::point::GeomPoint) -> crate::geom::point::GeomPoint,
        Self: Sized,
    {
        let geom = crate::geom::point::GeomPoint::default();
        let geom = f(geom);

        let mut layer = geom.into_layer();
        self.merge_default_aesthetics(&mut layer);
        self.layers_mut().push(layer);
        self
    }

    /// Add a line geom layer with customization (builder style)
    fn geom_line_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(crate::geom::line::GeomLine) -> crate::geom::line::GeomLine,
        Self: Sized,
    {
        let geom = crate::geom::line::GeomLine::default();
        let geom = f(geom);

        let mut layer = geom.into_layer();
        self.merge_default_aesthetics(&mut layer);
        self.layers_mut().push(layer);
        self
    }

    /// Add a density geom layer with customization (builder style)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// plot.geom_density_with(|geom| {
    ///     geom.color(color::BLUE)
    ///         .size(2.0)
    ///         .adjust(0.5)
    /// })
    /// ```
    fn geom_density_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(crate::geom::density::GeomDensity) -> crate::geom::density::GeomDensity,
        Self: Sized,
    {
        let geom = crate::geom::density::GeomDensity::default();
        let geom = f(geom);

        let mut layer = geom.into_layer();
        self.merge_default_aesthetics(&mut layer);
        self.layers_mut().push(layer);
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
    fn geom_hline_with<F>(mut self, yintercept: f64, f: F) -> Self
    where
        F: FnOnce(crate::geom::hline::GeomHLine) -> crate::geom::hline::GeomHLine,
        Self: Sized,
    {
        let geom = crate::geom::hline::GeomHLine::new(yintercept);
        let geom = f(geom);

        let mut layer = geom.into_layer();
        self.merge_default_aesthetics(&mut layer);
        self.layers_mut().push(layer);
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
    fn geom_vline_with<F>(mut self, xintercept: f64, f: F) -> Self
    where
        F: FnOnce(crate::geom::vline::GeomVLine) -> crate::geom::vline::GeomVLine,
        Self: Sized,
    {
        let geom = crate::geom::vline::GeomVLine::new(xintercept);
        let geom = f(geom);

        let mut layer = geom.into_layer();
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
    fn geom_rect_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(crate::geom::rect::GeomRect) -> crate::geom::rect::GeomRect,
        Self: Sized,
    {
        let geom = crate::geom::rect::GeomRect::new();
        let geom = f(geom);

        let mut layer = geom.into_layer();
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
    fn geom_segment_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(crate::geom::segment::GeomSegment) -> crate::geom::segment::GeomSegment,
        Self: Sized,
    {
        let geom = crate::geom::segment::GeomSegment::new();
        let geom = f(geom);

        let mut layer = geom.into_layer();
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
    fn geom_bar_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(crate::geom::bar::GeomBar) -> crate::geom::bar::GeomBar,
        Self: Sized,
    {
        let geom = crate::geom::bar::GeomBar::new();
        let geom = f(geom);

        let mut layer = geom.into_layer();
        self.merge_default_aesthetics(&mut layer);

        // If layer needs stat transformation and doesn't have data, take plot data
        // Stats need owned data to transform
        if !matches!(layer.stat, crate::layer::Stat::Identity) && layer.data.is_none() {
            layer.data = self.data_mut().take();
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
        self.geom_histogram_with(|geom| geom)
    }

    /// Add a histogram geom layer with customization (builder style)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// plot.geom_histogram_with(|geom| {
    ///     geom.fill(color::STEELBLUE)
    ///         .bins(20)
    ///         .alpha(0.8)
    /// })
    /// ```
    fn geom_histogram_with<F>(mut self, f: F) -> Self
    where
        F: FnOnce(crate::geom::histogram::GeomHistogram) -> crate::geom::histogram::GeomHistogram,
        Self: Sized,
    {
        let geom = crate::geom::histogram::GeomHistogram::new();
        let geom = f(geom);

        let mut layer = geom.into_layer();
        self.merge_default_aesthetics(&mut layer);

        // If layer needs stat transformation and doesn't have data, take plot data
        // Stats need owned data to transform
        if !matches!(layer.stat, crate::layer::Stat::Identity) && layer.data.is_none() {
            layer.data = self.data_mut().take();
        }

        self.layers_mut().push(layer);
        self
    }
}
