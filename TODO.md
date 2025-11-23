# TODO: gogplot-rs Features

## High Priority Geoms

### Statistical Geoms
- [x] **Boxplot** (`geom_boxplot`) - Essential for distribution visualization - COMPLETED
  - [x] Show median, quartiles, and outliers
  - [x] Support for grouped boxplots by category (via Position::Dodge and Fill aesthetic)
  - [x] Outlier detection with configurable IQR coefficient
  - [ ] Notched boxplots option (future enhancement)
  
- [ ] **Violin Plot** (`geom_violin`) - Distribution density visualization
  - Kernel density estimation on both sides
  - Optional embedded boxplot
  - Half-violin option for compact layouts

### Additional Geoms
- [ ] **Error Bars** (`geom_errorbar`, `geom_errorbarh`) - Confidence intervals and ranges
- [ ] **Ribbon** (`geom_ribbon`) - Filled areas with upper/lower bounds
- [ ] **Area** (`geom_area`) - Filled line plots
- [ ] **Step** (`geom_step`) - Step function plots
- [ ] **Tile/Raster** (`geom_tile`, `geom_raster`) - Heatmaps
- [ ] **Contour** (`geom_contour`) - 2D density contours
- [ ] **Text Labels** (`geom_text`, `geom_label`) - Annotations on plots
- [ ] **Path** (`geom_path`) - Connected points in data order (vs geom_line by x)
- [ ] **Polygon** (`geom_polygon`) - Arbitrary polygons
- [x] **Smooth** (`geom_smooth`) - Add trend lines with confidence intervals - COMPLETED
  - [x] LOESS smoothing (default, with configurable span)
  - [x] Cubic spline smoothing (with automatic knot selection)
  - [x] Linear regression (lm)
  - [x] Confidence bands with local variance estimation
  - [x] Configurable confidence level (default 95%)
  - [x] Support for grouped smoothing

## Scales and Axes

- [x] **Scale transformations** - COMPLETED
  - Pluggable Transform trait following ggplot2/scales design
  - IdentityTransform (linear), SqrtTransform, Log10Transform, ReverseTransform
  - Unified ContinuousScaleImpl replaces separate scale structs
  - Extensible for custom transformations
  - Domain constraints (e.g., 1e-300 for p-values)
  - Transformation-specific break generation and formatting
  
- [ ] **Axis label formatting for small values** - Review formatting logic for scales with very small values (e.g., p-values)
  - Current logic may not handle ranges like 1e-5 to 1e-3 optimally
  - Check if scientific notation is consistently applied when all values are small
  - Consider context-aware formatting based on the full range, not individual values
  
- [ ] **Logarithmic scales** - Have log10, need log2, natural log (easy to add with Transform trait)
- [ ] **Date/Time scales** - Proper handling of temporal data
- [ ] **Continuous color scales** - Implement scaling for color aesthetic
  - Viridis, gradient, and custom continuous palettes
  - Currently only categorical color mapping is supported
  
- [ ] **Continuous fill scales** - Implement scaling for fill aesthetic
  - Same palettes as color scales
  - Currently only categorical fill mapping is supported
  
- [ ] **Alpha/opacity scales** - Implement scaling for alpha aesthetic
  - Map data values to opacity range [0,1]
  - Currently alpha values are used directly without transformation
  
- [ ] **Size scales** - Implement scaling for size aesthetic
  - Map data values to reasonable pixel sizes with min/max constraints
  - Currently size values are used directly without transformation
  
- [ ] **Custom color palettes** - Beyond the basic discrete colors
  - ColorBrewer palettes
  - Viridis-style continuous palettes
  - Custom user-defined palettes
  
- [ ] **Secondary axes** - Dual y-axes for different scales
- [ ] **Manual scale limits** - Already partially implemented, need better API
- [ ] **Scale breaks** - Discontinuous axes

## Faceting

- [ ] **Facet wrap** (`facet_wrap`) - Multiple plots in grid based on one variable
- [ ] **Facet grid** (`facet_grid`) - Grid layout based on two variables
- [ ] **Free scales** - Independent axis ranges per facet
- [ ] **Facet labels** - Custom labeling for facets

## Themes and Styling

- [ ] **Theme presets** - Already have some, expand:
  - `theme_minimal()`
  - `theme_classic()`
  - `theme_dark()`
  - `theme_void()`
  - `theme_bw()`
  
- [x] **Geom defaults from theme** - PARTIALLY IMPLEMENTED
  - [x] `geom_text` defaults (size, color, alpha, hjust, vjust) controlled via `theme.geom_text`
  - [ ] Extend to other geoms where appropriate:
    - [ ] `geom_point` - default size, color, alpha, shape
    - [ ] `geom_line` - default size/width, color, alpha, linetype
    - [ ] `geom_bar` - default fill, color, alpha, width
    - [ ] `geom_boxplot` - default fill, color, alpha, outlier size/shape
    - [ ] `geom_smooth` - default color, alpha, line width, confidence band styling
    - [ ] `geom_histogram` - default fill, color, alpha, binwidth behavior
    - [ ] `geom_density` - default color, alpha, fill, line width
    - [ ] `geom_segment` - default color, alpha, line width, arrow styling
  - [ ] Consider adding `GeomDefaults` or similar theme component for common properties
  
- [ ] **Grid customization** - Major/minor grid lines control
- [ ] **Axis styling** - Tick marks, labels, titles
- [ ] **Panel customization** - Background, borders, spacing
- [ ] **Strip customization** - Facet label appearance

## Legends

- [ ] **Legend positioning** - More flexible placement (inside plot, outside, custom coordinates)
- [ ] **Multiple legends** - When using multiple aesthetics
- [ ] **Legend ordering** - Control order of legend items
- [ ] **Legend customization** - Size, shape, labels, title
- [ ] **No legend option** - Easy way to suppress legends

## Statistics

- [x] **stat_summary** - Compute summaries (mean, median, etc.) - COMPLETED
- [ ] **stat_bin2d** - 2D binning for heatmaps
- [ ] **stat_density2d** - 2D density estimation
- [ ] **stat_ecdf** - Empirical cumulative distribution
- [ ] **stat_qq** - Quantile-quantile plots
- [x] **stat_smooth** - Various smoothing methods - COMPLETED (loess, lm, spline)

## Reference Lines Enhancement

- [x] **hline/vline with statistical y aesthetics** - COMPLETED
  - `geom_hline` can use y aesthetic mapped to mean/median of data column
  - Example: `.geom_hline_with(|layer| { layer.aes.yintercept("mean"); layer.stat = Stat::Summary(...); })`
  - Similarly for `geom_vline` with x aesthetic
  - Computes statistics from the layer's data source automatically

## Coordinate Systems

- [ ] **Polar coordinates** (`coord_polar`) - Pie charts, wind roses
- [ ] **Flipped coordinates** (`coord_flip`) - Easy horizontal bar charts
- [ ] **Fixed aspect ratio** (`coord_fixed`) - 1:1 or custom ratios
- [ ] **Transformed coordinates** - Log, sqrt transformations
- [ ] **Map projections** - For geographic data

## Data Integration

- [ ] **Parquet support** - Direct reading via Arrow
- [ ] **Database connections** - Via DataFusion SQL queries
- [ ] **Streaming data** - Incremental updates
- [ ] **Multiple data sources per plot** - Different data for different layers
- [ ] **DataSource to Arrow RecordBatch converter** - Utility function to convert any DataSource to Arrow RecordBatch
  - Add trait extension for dot notation: `data_source.to_arrow_record_batch()`
  - Similar converter for Polars: `data_source.to_polars_dataframe()`
  - Enables easy interop between different data formats

## Interactivity (Future)

- [ ] **Hover tooltips** - Show values on hover
- [ ] **Zoom and pan** - Interactive exploration
- [ ] **Linked plots** - Brushing and linking
- [ ] **Animation** - Animated transitions and time series

## Output Formats

- [x] PNG (already supported)
- [x] PDF (already supported via cairo)
- [x] SVG (already supported via cairo)
- [ ] **HTML canvas** - Interactive web output
- [ ] **WebP** - Modern image format

## Documentation

- [ ] **Comprehensive examples** - More examples like mtcars
- [ ] **API documentation** - Complete rustdoc for all public APIs
- [ ] **User guide** - Step-by-step tutorials
- [ ] **Gallery** - Visual showcase of capabilities
- [ ] **Migration guide** - For users familiar with ggplot2

## Performance

- [ ] **Caching** - Cache computed statistics and scales
- [ ] **Progressive rendering** - For large datasets
- [ ] **GPU acceleration** - For intensive computations
- [ ] **Parallel processing** - Multi-threaded rendering

## Testing

- [ ] **Visual regression tests** - Ensure plots render consistently
- [ ] **Property-based tests** - Test edge cases
- [ ] **Benchmark suite** - Track performance over time

## Nice to Have

- [ ] **Equation support** - LaTeX-style math in labels
- [ ] **Unicode support** - Better font handling
- [ ] **Themes from ggplot2** - Port popular ggplot2 themes
- [ ] **Color blindness simulation** - Preview how plots look with different vision types
- [ ] **Export to ggplot2** - Generate R code to recreate plots
