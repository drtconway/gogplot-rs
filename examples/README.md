# gogplot-rs Examples

This directory contains examples demonstrating the features and capabilities of gogplot-rs, organized by complexity level.

## 🚀 Getting Started

**Start here:** Run `cargo run --example simple_api` to see the simplest possible plot.

All examples can be run with:
```bash
cargo run --example <example_name>
```

## 📁 Organization

### Basic Examples

Start here if you're new to gogplot-rs. These examples demonstrate core functionality with minimal code.

- **`simple_api`** ⭐ - The simplest possible plot (start here!)
- **`scatter_plot`** - Basic scatter plot with points
- **`line_plot`** - Simple line plot connecting points
- **`bar_chart`** - Bar chart with automatic counting
- **`histogram`** - Distribution visualization with binning

**Concepts covered:** Basic geoms, aesthetic mappings, data input, saving plots

---

### Intermediate Examples

Once you're comfortable with the basics, explore these examples showing more complex scenarios.

- **`bar_chart_identity`** - Using pre-computed values (Identity stat)
- **`categorical_scatter`** - Scatter plots with categorical data
- **`density_plot`** - Smooth density estimation
- **`fill_scales`** - Using fill aesthetics and scales
- **`grouped_histogram`** - Histograms with grouping by category
- **`stacked_histogram`** - Stacked histogram visualization
- **`rectangles`** - Drawing rectangular regions
- **`segments`** - Line segments between points

**Concepts covered:** Stats (Count, Identity, Density), grouping, categorical data, custom geoms

---

### Advanced Examples

Advanced customization, styling, and specialized features.

- **`legends`** - Automatic, manual, and themed legends
- **`themes`** - Built-in themes (light, dark, minimal)
- **`axis_positions`** - Customizing axis placement and style
- **`reference_lines`** - Adding horizontal and vertical reference lines (hline/vline)
- **`continuous_color`** - Continuous color scales for numeric data
- **`line_styles`** - Different line styles (solid, dashed, dotted)
- **`shapes`** - Different point shapes (circle, square, triangle, etc.)

**Concepts covered:** Guides, themes, scales, visual customization, reference lines

---

## 📚 Examples by Feature

### By Geom Type

**Points:**
- `scatter_plot` - Basic points
- `categorical_scatter` - Points with categories
- `shapes` - Different point shapes

**Lines:**
- `line_plot` - Basic lines
- `line_styles` - Line styling options

**Bars:**
- `bar_chart` - Automatic counting
- `bar_chart_identity` - Pre-computed values
- `grouped_histogram` - Grouped bars
- `stacked_histogram` - Stacked bars

**Distributions:**
- `histogram` - Basic histogram
- `density_plot` - Smooth density

**Geometric Shapes:**
- `rectangles` - Rectangles
- `segments` - Line segments
- `reference_lines` - hline/vline

### By Aesthetic

**Color:**
- `categorical_scatter` - Discrete colors
- `continuous_color` - Continuous color scales
- `fill_scales` - Fill aesthetic

**Size:**
- `shapes` - Varying point sizes

**Shape:**
- `shapes` - Different point shapes

**Line Type:**
- `line_styles` - Dashed, dotted, solid lines

### By Customization Type

**Scales:**
- `fill_scales` - Fill scales
- `continuous_color` - Color scales

**Guides:**
- `legends` - Legend creation and positioning
- `axis_positions` - Axis customization

**Themes:**
- `themes` - Light, dark, and minimal themes

**Reference Lines:**
- `reference_lines` - Adding hline/vline for reference

---

## 💡 Tips

1. **Start simple:** Begin with `simple_api` and work your way up
2. **Aesthetic mappings:** Use `.aes()` to map data columns to visual properties
3. **Geom configuration:** Use `geom_*_with()` methods for customization
4. **Scales are automatic:** In most cases, scales and legends are generated automatically
5. **Save your plots:** All plots use `.save(filename, width, height)?`

## 🔍 Need Help?

- Look at similar examples to what you're trying to build
- Check the inline comments in each example
- Start with the simplest example that demonstrates your feature
- Build up complexity gradually

---

## 📊 Quick Reference

| Want to... | See Example |
|-----------|-------------|
| Create my first plot | `simple_api` |
| Plot x vs y points | `scatter_plot` |
| Draw lines | `line_plot` |
| Show counts by category | `bar_chart` |
| Show a distribution | `histogram` |
| Use pre-computed values | `bar_chart_identity` |
| Group by color | `categorical_scatter` |
| Stack or group bars | `grouped_histogram`, `stacked_histogram` |
| Customize colors | `continuous_color` |
| Change the theme | `themes` |
| Add a legend | `legends` |
| Add reference lines | `reference_lines` |
| Style lines differently | `line_styles` |
| Use different shapes | `shapes` |

