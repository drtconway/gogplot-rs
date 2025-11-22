# TODO: gogplot-rs Features

## High Priority Geoms

### Statistical Geoms
- [ ] **Boxplot** (`geom_boxplot`) - Essential for distribution visualization
  - Show median, quartiles, and outliers
  - Support for grouped boxplots by category
  - Notched boxplots option
  
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
- [ ] **Smooth** (`geom_smooth`) - Add trend lines with confidence intervals

## Scales and Axes

- [x] **Scale transformations** - COMPLETED
  - Pluggable Transform trait following ggplot2/scales design
  - IdentityTransform (linear), SqrtTransform, Log10Transform, ReverseTransform
  - Unified ContinuousScaleImpl replaces separate scale structs
  - Extensible for custom transformations
  - Domain constraints (e.g., 1e-300 for p-values)
  - Transformation-specific break generation and formatting
  
- [ ] **Logarithmic scales** - Have log10, need log2, natural log (easy to add with Transform trait)
- [ ] **Date/Time scales** - Proper handling of temporal data
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
- [ ] **stat_smooth** - Various smoothing methods (loess, lm, glm)

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
