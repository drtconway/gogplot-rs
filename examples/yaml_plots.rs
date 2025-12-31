use gogplot::{
    aesthetics::{
        AestheticDomain,
        builder::{
            ColorContinuousAesBuilder, ColorDiscreteAesBuilder, SizeContinuousAesBuilder,
            SizeDiscreteAesBuilder, XContininuousAesBuilder, XDiscreteAesBuilder,
            YContininuousAesBuilder, YDiscreteAesBuilder,
        },
    },
    error::to_io_error,
    geom::{geom_rect, hline::geom_hline, line::geom_line, point::geom_point},
    plot::{PlotBuilder, plot},
    utils::mtcars::mtcars,
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

// ========== YAML Structure Definitions ==========

/// Top-level plot specification matching the YAML structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotYaml {
    pub plot: PlotDefinition,
}

/// The main plot definition with mappings and layers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotDefinition {
    /// Global aesthetic mappings applied to all layers
    #[serde(default)]
    pub mapping: MappingDefinition,

    /// List of geom layers to render
    pub layers: Vec<LayerDefinition>,

    /// Optional output settings
    #[serde(default)]
    pub output: OutputSettings,
}

/// Aesthetic mappings from data columns to visual properties
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MappingDefinition {
    /// X-axis mapping
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<String>,

    /// Y-axis mapping
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<String>,

    /// X-axis minimum mapping
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xmin: Option<String>,

    /// X-axis maximum mapping
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xmax: Option<String>,

    /// Y-axis minimum mapping
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ymin: Option<String>,

    /// Y-axis maximum mapping
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ymax: Option<String>,

    /// Color aesthetic mapping
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// Size aesthetic mapping
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,

    /// Fill aesthetic mapping
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,

    /// Alpha (transparency) aesthetic mapping
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpha: Option<String>,

    /// Shape aesthetic mapping
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape: Option<String>,

    /// Group aesthetic for grouping data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group: Option<String>,

    /// Linetype aesthetic mapping
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linetype: Option<String>,

    /// X-intercept mapping for vertical lines
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xintercept: Option<String>,

    /// Y-intercept mapping for horizontal lines
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yintercept: Option<String>,

    /// Additional custom mappings
    #[serde(flatten)]
    pub extra: HashMap<String, String>,
}

/// Individual layer with geom type and optional mappings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerDefinition {
    /// Type of geom (point, line, bar, etc.)
    pub geom: GeomType,

    /// Layer-specific aesthetic mappings (override global)
    #[serde(default)]
    pub mapping: MappingDefinition,

    /// Statistical transformation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stat: Option<StatType>,

    /// Position adjustment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<PositionType>,

    /// Additional parameters for the geom
    #[serde(default)]
    pub params: GeomParams,
}

/// Types of geoms available
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum GeomType {
    Point,
    Line,
    Rect,
    HLine,
    VLine,
}

/// Statistical transformations
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StatType {
    Identity,
    Count,
    Bin,
}

/// Position adjustments
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PositionType {
    Identity,
    Stack,
    Dodge,
    Fill,
    Jitter,
}

/// Parameters for customizing geom appearance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GeomParams {
    /// Fixed color value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,

    /// Fixed fill value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fill: Option<String>,

    /// Fixed size value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<f64>,

    /// Fixed alpha value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alpha: Option<f64>,

    /// Line width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linewidth: Option<f64>,

    /// Point shape
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape: Option<String>,

    /// Fixed x-intercept value for vertical lines
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xintercept: Option<f64>,

    /// Fixed y-intercept value for horizontal lines
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yintercept: Option<f64>,

    /// Additional custom parameters
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
}

/// Output settings for the plot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSettings {
    /// Output filename
    #[serde(default = "default_filename")]
    pub filename: String,

    /// Plot width in pixels
    #[serde(default = "default_width")]
    pub width: u32,

    /// Plot height in pixels
    #[serde(default = "default_height")]
    pub height: u32,

    /// Output format (png, svg, pdf)
    #[serde(default = "default_format")]
    pub format: String,

    /// DPI for raster outputs
    #[serde(default = "default_dpi")]
    pub dpi: u32,
}

impl Default for OutputSettings {
    fn default() -> Self {
        Self {
            filename: default_filename(),
            width: default_width(),
            height: default_height(),
            format: default_format(),
            dpi: default_dpi(),
        }
    }
}

fn default_filename() -> String {
    "output.png".to_string()
}

fn default_width() -> u32 {
    800
}

fn default_height() -> u32 {
    600
}

fn default_format() -> String {
    "png".to_string()
}

fn default_dpi() -> u32 {
    96
}

// ========== Plot Generation ==========

/// Generate a plot from a YAML specification
pub fn generate_plot_from_yaml(yaml_spec: &PlotYaml) -> std::io::Result<()> {
    let data = mtcars();
    let plot_def = &yaml_spec.plot;

    // Create base plot with global mappings
    let mut builder = plot(&data);

    // Apply global mappings
    let global_mapping = &plot_def.mapping;
    if global_mapping.x.is_some() || global_mapping.y.is_some() {
        builder = builder.aes(|a| {
            if let Some(ref x) = global_mapping.x {
                let (col, domain) = decode_column_reference(x);
                match domain {
                    Some(AestheticDomain::Discrete) => a.x_discrete(&col),
                    Some(AestheticDomain::Continuous) => a.x_continuous(&col),
                    None => a.x_continuous(&col),
                }
            }
            if let Some(ref y) = global_mapping.y {
                let (col, domain) = decode_column_reference(y);
                match domain {
                    Some(AestheticDomain::Discrete) => a.y_discrete(&col),
                    Some(AestheticDomain::Continuous) => a.y_continuous(&col),
                    None => a.y_continuous(&col),
                }
            }
            if let Some(ref color) = global_mapping.color {
                let (col, domain) = decode_column_reference(color);
                match domain {
                    Some(AestheticDomain::Continuous) => a.color_continuous(&col),
                    Some(AestheticDomain::Discrete) => a.color_discrete(&col),
                    None => a.color_continuous(&col),
                }
            }
            if let Some(ref size) = global_mapping.size {
                let (col, domain) = decode_column_reference(size);
                match domain {
                    Some(AestheticDomain::Continuous) => a.size_continuous(&col),
                    Some(AestheticDomain::Discrete) => a.size_discrete(&col),
                    None => a.size_continuous(&col),
                }
            }
        });
    }

    // Add each layer
    for layer in &plot_def.layers {
        builder = add_layer(builder, layer)?;
    }

    // Build and save
    let plot = builder.build().map_err(to_io_error)?;
    plot.save(
        &plot_def.output.filename,
        plot_def.output.width,
        plot_def.output.height,
    )
    .map_err(to_io_error)?;

    println!("Generated plot: {}", plot_def.output.filename);
    Ok(())
}

fn add_layer<'a>(
    builder: impl Into<PlotBuilder<'a>>,
    layer: &LayerDefinition,
) -> std::io::Result<PlotBuilder<'a>> {
    let builder = builder.into();

    let result = match layer.geom {
        GeomType::Point => {
            let mut geom = geom_point();

            // Apply fixed parameters
            if let Some(color) = &layer.params.color {
                let color = string_to_color(&color)?;
                geom = geom.color(color);
            }
            if let Some(size) = layer.params.size {
                geom = geom.size(size);
            }
            if let Some(alpha) = layer.params.alpha {
                geom = geom.alpha(alpha);
            }
            if let Some(shape) = &layer.params.shape {
                let shape = string_to_shape(&shape)?;
                geom = geom.shape(shape);
            }

            // Apply layer-specific mappings if present
            if has_mappings(&layer.mapping) {
                geom = geom.aes(|a| {
                    if let Some(ref x) = layer.mapping.x {
                        let (col, domain) = decode_column_reference(x);
                        match domain {
                            Some(AestheticDomain::Discrete) => a.x_discrete(&col),
                            Some(AestheticDomain::Continuous) => a.x_continuous(&col),
                            None => a.x_continuous(&col),
                        }
                    }
                    if let Some(ref y) = layer.mapping.y {
                        let (col, domain) = decode_column_reference(y);
                        match domain {
                            Some(AestheticDomain::Discrete) => a.y_discrete(&col),
                            Some(AestheticDomain::Continuous) => a.y_continuous(&col),
                            None => a.y_continuous(&col),
                        }
                    }
                    if let Some(ref color) = layer.mapping.color {
                        let (col, domain) = decode_column_reference(color);
                        match domain {
                            Some(AestheticDomain::Continuous) => a.color_continuous(&col),
                            Some(AestheticDomain::Discrete) => a.color_discrete(&col),
                            None => a.color_continuous(&col),
                        }
                    }
                    if let Some(ref size) = layer.mapping.size {
                        let (col, domain) = decode_column_reference(size);
                        match domain {
                            Some(AestheticDomain::Continuous) => a.size_continuous(&col),
                            Some(AestheticDomain::Discrete) => a.size_discrete(&col),
                            None => a.size_continuous(&col),
                        }
                    }
                    if let Some(ref alpha) = layer.mapping.alpha {
                        let (col, domain) = decode_column_reference(alpha);
                        match domain {
                            Some(AestheticDomain::Continuous) => a.alpha_continuous(&col),
                            Some(AestheticDomain::Discrete) => a.alpha_discrete(&col),
                            None => a.alpha_continuous(&col),
                        }
                    }
                    if let Some(ref shape) = layer.mapping.shape {
                        let (col, domain) = decode_column_reference(shape);
                        if matches!(domain, Some(AestheticDomain::Continuous)) {
                            panic!("Shape aesthetic must be discrete");
                        }
                        a.shape(&col);
                    }
                });
            }

            builder + geom
        }
        GeomType::Line => {
            let mut geom = geom_line();

            if let Some(size) = layer.params.size {
                geom = geom.size(size);
            }

            if has_mappings(&layer.mapping) {
                geom = geom.aes(|a| {
                    if let Some(ref x) = layer.mapping.x {
                        let (col, domain) = decode_column_reference(x);
                        match domain {
                            Some(AestheticDomain::Discrete) => a.x_discrete(&col),
                            Some(AestheticDomain::Continuous) => a.x_continuous(&col),
                            None => a.x_continuous(&col),
                        }
                    }
                    if let Some(ref y) = layer.mapping.y {
                        let (col, domain) = decode_column_reference(y);
                        match domain {
                            Some(AestheticDomain::Discrete) => a.y_discrete(&col),
                            Some(AestheticDomain::Continuous) => a.y_continuous(&col),
                            None => a.y_continuous(&col),
                        }
                    }
                    if let Some(ref color) = layer.mapping.color {
                        let (col, domain) = decode_column_reference(color);
                        match domain {
                            Some(AestheticDomain::Continuous) => a.color_continuous(&col),
                            Some(AestheticDomain::Discrete) => a.color_discrete(&col),
                            None => a.color_continuous(&col),
                        }
                    }
                    if let Some(ref size) = layer.mapping.size {
                        let (col, domain) = decode_column_reference(size);
                        match domain {
                            Some(AestheticDomain::Continuous) => a.size_continuous(&col),
                            Some(AestheticDomain::Discrete) => a.size_discrete(&col),
                            None => a.size_continuous(&col),
                        }
                    }
                    if let Some(ref alpha) = layer.mapping.alpha {
                        let (col, domain) = decode_column_reference(alpha);
                        match domain {
                            Some(AestheticDomain::Continuous) => a.alpha_continuous(&col),
                            Some(AestheticDomain::Discrete) => a.alpha_discrete(&col),
                            None => a.alpha_continuous(&col),
                        }
                    }
                });
            }

            builder + geom
        }
        GeomType::Rect => {
            let mut geom = geom_rect();

            // Apply fixed parameters
            if let Some(color) = &layer.params.fill {
                let color = string_to_color(&color)?;
                geom = geom.fill(color);
            }
            if let Some(alpha) = layer.params.alpha {
                geom = geom.alpha(alpha);
            }

            if has_mappings(&layer.mapping) {
                geom = geom.aes(|a| {
                    if let Some(ref x) = layer.mapping.xmin {
                        let (col, _domain) = decode_column_reference(x);
                        a.xmin(&col);
                    }
                    if let Some(ref x) = layer.mapping.xmax {
                        let (col, _domain) = decode_column_reference(x);
                        a.xmax(&col);
                    }
                    if let Some(ref y) = layer.mapping.ymin {
                        let (col, _domain) = decode_column_reference(y);
                        a.ymin(&col);
                    }
                    if let Some(ref y) = layer.mapping.ymax {
                        let (col, _domain) = decode_column_reference(y);
                        a.ymax(&col);
                    }
                    if let Some(ref fill) = layer.mapping.fill {
                        let (col, domain) = decode_column_reference(fill);
                        match domain {
                            Some(AestheticDomain::Continuous) => a.fill_continuous(&col),
                            Some(AestheticDomain::Discrete) => a.fill_discrete(&col),
                            None => a.fill_continuous(&col),
                        }
                    }
                });
            }

            builder + geom
        }

        GeomType::HLine => {
            let mut geom = geom_hline();

            // Apply fixed parameters
            if let Some(yintercept) = layer.params.yintercept {
                geom = geom.y_intercept(yintercept);
            }
            if let Some(color_str) = &layer.params.color {
                let color = string_to_color(color_str)?;
                geom = geom.color(color);
            }
            if let Some(size) = layer.params.size {
                geom = geom.size(size);
            }
            if let Some(alpha) = layer.params.alpha {
                geom = geom.alpha(alpha);
            }

            builder + geom
        }
        GeomType::VLine => {
            let mut geom = gogplot::geom::vline::geom_vline();

            // Apply fixed parameters
            if let Some(xintercept) = layer.params.xintercept {
                geom = geom.xintercept(xintercept);
            }
            if let Some(color_str) = &layer.params.color {
                let color = string_to_color(color_str)?;
                geom = geom.color(color);
            }
            if let Some(size) = layer.params.size {
                geom = geom.size(size);
            }
            if let Some(alpha) = layer.params.alpha {
                geom = geom.alpha(alpha);
            }

            builder + geom
        }
    };

    Ok(result)
}

fn has_mappings(mapping: &MappingDefinition) -> bool {
    mapping.x.is_some()
        || mapping.y.is_some()
        || mapping.color.is_some()
        || mapping.size.is_some()
        || mapping.fill.is_some()
        || mapping.alpha.is_some()
        || mapping.shape.is_some()
}

fn string_to_shape(shape_str: &str) -> std::io::Result<gogplot::visuals::Shape> {
    match shape_str.to_lowercase().as_str() {
        "circle" => Ok(gogplot::visuals::Shape::Circle),
        "square" => Ok(gogplot::visuals::Shape::Square),
        "triangle" => Ok(gogplot::visuals::Shape::Triangle),
        "diamond" => Ok(gogplot::visuals::Shape::Diamond),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Unknown shape: {}", shape_str),
        )),
    }
}

fn string_to_color(color_str: &str) -> std::io::Result<gogplot::theme::Color> {
    let color_map = gogplot::theme::color::color_map();
    let color_lower = color_str.to_lowercase();
    for (name, color) in color_map.iter() {
        if *name == color_lower {
            return Ok(*color);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        format!("Unknown color: {}", color_str),
    ))
}

fn decode_column_reference(column: &str) -> (String, Option<AestheticDomain>) {
    if column.starts_with("$") {
        (column[1..].to_string(), Some(AestheticDomain::Discrete))
    } else if column.starts_with("~") {
        (column[1..].to_string(), Some(AestheticDomain::Continuous))
    } else {
        (column.to_string(), None)
    }
}

// ========== Main Entry Point ==========

fn main() -> std::io::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    // Read YAML file
    let yaml_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "examples/example_plot.yaml".to_string());

    let yaml_content = fs::read_to_string(&yaml_path)?;

    // Parse YAML
    let plot_spec: PlotYaml = serde_yaml::from_str(&yaml_content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    println!("Loaded plot specification from: {}", yaml_path);
    println!("Plot has {} layer(s)", plot_spec.plot.layers.len());

    // Generate the plot
    generate_plot_from_yaml(&plot_spec)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_yaml() {
        let yaml = r#"
plot:
  mapping:
    x: mpg
    y: wt
  layers:
    - geom: point
"#;

        let spec: PlotYaml = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(spec.plot.mapping.x, Some("mpg".to_string()));
        assert_eq!(spec.plot.mapping.y, Some("wt".to_string()));
        assert_eq!(spec.plot.layers.len(), 1);
        assert_eq!(spec.plot.layers[0].geom, GeomType::Point);
    }

    #[test]
    fn test_parse_complex_yaml() {
        let yaml = r#"
plot:
  mapping:
    x: wt
    y: mpg
    color: cyl
  layers:
    - geom: point
      params:
        size: 3.0
        alpha: 0.8
    - geom: smooth
      params:
        color: blue
  output:
    filename: complex_plot.png
    width: 1000
    height: 800
"#;

        let spec: PlotYaml = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(spec.plot.layers.len(), 2);
        assert_eq!(spec.plot.layers[0].geom, GeomType::Point);
        assert_eq!(spec.plot.layers[1].geom, GeomType::Smooth);
        assert_eq!(spec.plot.output.width, 1000);
    }
}
