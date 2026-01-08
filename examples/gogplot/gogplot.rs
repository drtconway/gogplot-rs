use clap::Parser;
use jsonschema::JSONSchema;
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use gogplot::aesthetics::{Aesthetic, AestheticDomain};
use gogplot::layer::LayerBuilder;
use gogplot::position::{dodge::Dodge, stack::Stack, Position};
use gogplot::prelude::*;
use gogplot::stat::bin::Bin;
use gogplot::stat::count::Count;
use gogplot::stat::summary::Summary;
use gogplot::stat::Stat;
use gogplot::theme::Color;
use gogplot::geom::rect::geom_rect;
use gogplot::geom::segment::geom_segment;
use gogplot::geom::text::geom_text;
use gogplot::geom::label::geom_label;
use gogplot::geom::errorbar::geom_errorbar;
use gogplot::geom::smooth::geom_smooth;

/// CLI for rendering plots from YAML/JSON specs.
#[derive(Debug, Parser)]
#[command(author, version, about = "Render plots from spec files", long_about = None)]
struct Args {
    /// One or more spec files (YAML or JSON)
    #[arg(required = true)]
    specs: Vec<PathBuf>,

    /// Optional JSON Schema file to validate specs against
    #[arg(long)]
    schema: Option<PathBuf>,
}

fn main() {
    if let Err(err) = try_main() {
        eprintln!("error: {}", err);
        let mut source = err.source();
        while let Some(cause) = source {
            eprintln!("caused by: {}", cause);
            source = cause.source();
        }
        std::process::exit(1);
    }
}

fn try_main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let schema_value = if let Some(schema_path) = args.schema.as_ref() {
        Some(load_value(schema_path)?)
    } else {
        None
    };

    for spec in &args.specs {
        let spec_value = load_value(spec)?;

        if let Some(schema_value) = schema_value.as_ref() {
            let schema = JSONSchema::compile(schema_value)
                .map_err(|e| format!("schema compile error: {e}"))?;
            if let Err(errors) = schema.validate(&spec_value) {
                eprintln!("Spec {} failed validation:", spec.display());
                for err in errors {
                    eprintln!("  - {}", err);
                }
                return Err("validation failed".into());
            }
        }

        println!("Validated: {}", spec.display());

        process_spec(spec, spec_value)?;
    }

    Ok(())
}

fn load_value(path: &Path) -> Result<Value, Box<dyn Error>> {
    let content = fs::read_to_string(path)?;
    let ext = path
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    let value = match ext.as_str() {
        "yaml" | "yml" => serde_yaml::from_str(&content)?,
        _ => serde_json::from_str(&content)?,
    };

    Ok(value)
}

// ===== Skeleton processing pipeline =====

#[derive(Debug, Deserialize)]
struct Spec {
    input: InputSpec,
    plot: PlotSpec,
    output: OutputSpec,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum InputSpec {
    File { file: String },
    Data { data: HashMap<String, Vec<Value>> },
}

#[derive(Debug, Deserialize, Default)]
struct OutputSpec {
    filename: Option<String>,
    format: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
    dpi: Option<u32>,
    #[serde(default)]
    params: HashMap<String, Value>,
}

#[derive(Debug, Deserialize, Default)]
struct PlotSpec {
    #[serde(default)]
    mapping: MappingSpec,
    #[serde(default)]
    layers: Vec<LayerSpec>,
}

#[derive(Debug, Deserialize, Default)]
struct MappingSpec {
    #[serde(default)]
    x: Option<String>,
    #[serde(default)]
    y: Option<String>,
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    alpha: Option<String>,
    #[serde(default)]
    fill: Option<String>,
    #[serde(default)]
    size: Option<String>,
    #[serde(default)]
    linetype: Option<String>,
    #[serde(default)]
    xintercept: Option<String>,
    #[serde(default)]
    yintercept: Option<String>,
    #[serde(default)]
    group: Option<String>,
    #[serde(default)]
    xmin: Option<String>,
    #[serde(default)]
    xmax: Option<String>,
    #[serde(default)]
    ymin: Option<String>,
    #[serde(default)]
    ymax: Option<String>,
    #[serde(default)]
    xbegin: Option<String>,
    #[serde(default)]
    xend: Option<String>,
    #[serde(default)]
    ybegin: Option<String>,
    #[serde(default)]
    yend: Option<String>,
    #[serde(default)]
    label: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum StatName {
    Summary,
    Count,
    Bin,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum StatSpec {
    /// Object form: { name: "summary", aesthetic: "~y" }
    Named {
        #[serde(rename = "name")]
        name: StatName,
        #[serde(default)]
        aesthetic: Option<String>,
    },
    /// String shorthand: "summary"
    Simple(StatName),
}

#[derive(Debug, Deserialize, Default)]
struct LayerSpec {
    geom: GeomKind,
    #[serde(default)]
    mapping: MappingSpec,
    #[serde(default)]
    stat: Option<StatSpec>,
    #[serde(default)]
    params: LayerParams,
    #[serde(default)]
    position: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum GeomKind {
    Point,
    Line,
    Hline,
    Vline,
    Boxplot,
    Density,
    Bar,
    Histogram,
    Rect,
    Segment,
    Text,
    Label,
    Errorbar,
    Smooth,
}

impl Default for GeomKind {
    fn default() -> Self {
        GeomKind::Point
    }
}

#[derive(Debug, Deserialize, Default)]
struct LayerParams {
    #[serde(default)]
    color: Option<String>,
    #[serde(default)]
    fill: Option<String>,
    #[serde(default)]
    size: Option<f64>,
    #[serde(default)]
    alpha: Option<f64>,
    #[serde(default)]
    linetype: Option<String>,
    #[serde(default)]
    xintercept: Option<f64>,
    #[serde(default)]
    yintercept: Option<f64>,
    #[serde(default)]
    width: Option<f64>,
    #[serde(default)]
    bins: Option<i32>,
    #[serde(default)]
    binwidth: Option<f64>,
    #[serde(default)]
    linewidth: Option<f64>,
    #[serde(default)]
    angle: Option<f64>,
    #[serde(default)]
    padding: Option<f64>,
    #[serde(default)]
    radius: Option<f64>,
    #[serde(default)]
    confidence_interval: Option<bool>,
}

fn process_spec(path: &Path, spec_value: Value) -> Result<(), Box<dyn Error>> {
    let spec: Spec = serde_json::from_value(spec_value)?;

    let spec_dir = path.parent().unwrap_or_else(|| Path::new("."));
    let data = load_data(&spec.input, spec_dir)?;
    let plot = build_plot(&spec.plot, &data)?;
    render_plot(&plot, &spec.output)?;

    Ok(())
}

fn load_data(input: &InputSpec, spec_dir: &Path) -> Result<Box<dyn DataSource>, Box<dyn Error>> {
    match input {
        InputSpec::File { file } => {
            let path = spec_dir.join(file);
            let df = load_csv_to_dataframe(&path)?;
            Ok(Box::new(df))
        }
        InputSpec::Data { data } => {
            let df = build_inline_dataframe(data)?;
            Ok(Box::new(df))
        }
    }
}

fn build_plot<'a>(
    plot_spec: &'a PlotSpec,
    data: &'a Box<dyn DataSource>,
) -> Result<gogplot::plot::Plot<'a>, Box<dyn Error>> {
    let mut builder = plot(data);
    builder = apply_global_mapping(builder, &plot_spec.mapping);

    // Layers
    for layer in &plot_spec.layers {
        match layer.geom {
            GeomKind::Point => {
                let mut geom = geom_point();

                geom = apply_point_layer_mapping(geom, &layer.mapping);

                // Params (minimal; extend later)
                if let Some(size) = layer.params.size {
                    geom = geom.size(size);
                }
                if let Some(alpha) = layer.params.alpha {
                    geom = geom.alpha(alpha);
                }
                if let Some(color) = layer.params.color.as_ref() {
                    geom = geom.color(parse_color(color)?);
                }

                let geom = finalize_layer(geom, &layer.stat, &layer.position, None, None)?;
                builder = builder + geom;
            }
            GeomKind::Line => {
                let mut geom = geom_line();

                geom = apply_line_layer_mapping(geom, &layer.mapping);

                if let Some(size) = layer.params.size {
                    geom = geom.size(size);
                }
                if let Some(alpha) = layer.params.alpha {
                    geom = geom.alpha(alpha);
                }
                if let Some(color) = layer.params.color.as_ref() {
                    geom = geom.color(parse_color(color)?);
                }
                if let Some(linetype) = layer.params.linetype.as_ref() {
                    geom = geom.linestyle(LineStyle::from(linetype.as_str()));
                }

                let geom = finalize_layer(geom, &layer.stat, &layer.position, None, None)?;
                builder = builder + geom;
            }
            GeomKind::Hline => {
                let mut geom = geom_hline();

                geom = apply_hline_layer_mapping(geom, &layer.mapping);

                if let Some(yint) = layer.params.yintercept {
                    geom = geom.y_intercept(yint);
                }
                if let Some(size) = layer.params.size {
                    geom = geom.size(size);
                }
                if let Some(alpha) = layer.params.alpha {
                    geom = geom.alpha(alpha);
                }
                if let Some(color) = layer.params.color.as_ref() {
                    geom = geom.color(parse_color(color)?);
                }
                if let Some(linetype) = layer.params.linetype.as_ref() {
                    geom = geom.linestyle(LineStyle::from(linetype.as_str()));
                }

                let geom = finalize_layer(geom, &layer.stat, &layer.position, Some("y"), None)?;
                builder = builder + geom;
            }
            GeomKind::Vline => {
                let mut geom = geom_vline();

                geom = apply_vline_layer_mapping(geom, &layer.mapping);

                if let Some(xint) = layer.params.xintercept {
                    geom = geom.x_intercept(xint);
                }
                if let Some(size) = layer.params.size {
                    geom = geom.size(size);
                }
                if let Some(alpha) = layer.params.alpha {
                    geom = geom.alpha(alpha);
                }
                if let Some(color) = layer.params.color.as_ref() {
                    geom = geom.color(parse_color(color)?);
                }
                if let Some(linetype) = layer.params.linetype.as_ref() {
                    geom = geom.linestyle(LineStyle::from(linetype.as_str()));
                }

                let geom = finalize_layer(geom, &layer.stat, &layer.position, Some("x"), None)?;
                builder = builder + geom;
            }
            GeomKind::Boxplot => {
                let mut geom = geom_boxplot();

                geom = apply_boxplot_layer_mapping(geom, &layer.mapping);

                if let Some(fill) = layer.params.fill.as_ref() {
                    geom = geom.fill(parse_color(fill)?);
                }
                if let Some(color) = layer.params.color.as_ref() {
                    geom = geom.color(parse_color(color)?);
                }
                if let Some(alpha) = layer.params.alpha {
                    geom = geom.alpha(alpha);
                }
                if let Some(width) = layer.params.width {
                    geom = geom.width(width);
                }
                if let Some(linetype) = layer.params.linetype.as_ref() {
                    geom = geom.linestyle(LineStyle::from(linetype.as_str()));
                }

                let geom = finalize_layer(geom, &layer.stat, &layer.position, None, None)?;
                builder = builder + geom;
            }
            GeomKind::Density => {
                let mut geom = geom_density();

                geom = apply_density_layer_mapping(geom, &layer.mapping);

                if let Some(fill) = layer.params.fill.as_ref() {
                    geom = geom.fill(parse_color(fill)?);
                }
                if let Some(color) = layer.params.color.as_ref() {
                    geom = geom.color(parse_color(color)?);
                }
                if let Some(alpha) = layer.params.alpha {
                    geom = geom.alpha(alpha);
                }
                if let Some(size) = layer.params.size {
                    geom = geom.size(size);
                }
                if let Some(linetype) = layer.params.linetype.as_ref() {
                    geom = geom.linestyle(LineStyle::from(linetype.as_str()));
                }

                let geom = finalize_layer(geom, &layer.stat, &layer.position, None, None)?;
                builder = builder + geom;
            }
            GeomKind::Bar => {
                let mut geom = geom_bar();

                geom = apply_bar_layer_mapping(geom, &layer.mapping);

                if let Some(fill) = layer.params.fill.as_ref() {
                    geom = geom.fill(parse_color(fill)?);
                }
                if let Some(color) = layer.params.color.as_ref() {
                    geom = geom.color(parse_color(color)?);
                }
                if let Some(alpha) = layer.params.alpha {
                    geom = geom.alpha(alpha);
                }
                if let Some(width) = layer.params.width {
                    geom = geom.width(width);
                }

                let default_stat: Option<Box<dyn Stat>> = if layer.stat.is_none()
                    || matches!(&layer.stat, Some(StatSpec::Named { name, .. }) if *name == StatName::Count)
                    || matches!(&layer.stat, Some(StatSpec::Simple(StatName::Count)))
                {
                    Some(Box::new(Count::default()))
                } else {
                    None
                };

                let geom = finalize_layer(geom, &layer.stat, &layer.position, None, default_stat)?;
                builder = builder + geom;
            }
            GeomKind::Histogram => {
                let mut geom = geom_histogram();

                geom = apply_histogram_layer_mapping(geom, &layer.mapping);

                if let Some(fill) = layer.params.fill.as_ref() {
                    geom = geom.fill(parse_color(fill)?);
                }
                if let Some(color) = layer.params.color.as_ref() {
                    geom = geom.color(parse_color(color)?);
                }
                if let Some(alpha) = layer.params.alpha {
                    geom = geom.alpha(alpha);
                }

                let default_stat: Option<Box<dyn Stat>> = if layer.stat.is_none()
                    || matches!(&layer.stat, Some(StatSpec::Named { name, .. }) if *name == StatName::Bin)
                    || matches!(&layer.stat, Some(StatSpec::Simple(StatName::Bin)))
                {
                    let bin_stat = if let Some(bw) = layer.params.binwidth {
                        Bin::with_width(bw)
                    } else if let Some(b) = layer.params.bins {
                        Bin::with_count(b as usize)
                    } else {
                        Bin::default()
                    };
                    Some(Box::new(bin_stat))
                } else {
                    None
                };

                let geom = finalize_layer(geom, &layer.stat, &layer.position, None, default_stat)?;
                builder = builder + geom;
            }
            GeomKind::Rect => {
                let mut geom = geom_rect();

                geom = apply_rect_layer_mapping(geom, &layer.mapping);

                if let Some(fill) = layer.params.fill.as_ref() {
                    geom = geom.fill(parse_color(fill)?);
                }
                if let Some(color) = layer.params.color.as_ref() {
                    geom = geom.color(parse_color(color)?);
                }
                if let Some(alpha) = layer.params.alpha {
                    geom = geom.alpha(alpha);
                }

                let geom = finalize_layer(geom, &layer.stat, &layer.position, None, None)?;
                builder = builder + geom;
            }
            GeomKind::Segment => {
                let mut geom = geom_segment();

                geom = apply_segment_layer_mapping(geom, &layer.mapping);

                if let Some(color) = layer.params.color.as_ref() {
                    geom = geom.color(parse_color(color)?);
                }
                if let Some(alpha) = layer.params.alpha {
                    geom = geom.alpha(alpha);
                }
                if let Some(linewidth) = layer.params.linewidth {
                    geom = geom.size(linewidth);
                }
                if let Some(linetype) = layer.params.linetype.as_ref() {
                    geom = geom.linestyle(LineStyle::from(linetype.as_str()));
                }

                let geom = finalize_layer(geom, &layer.stat, &layer.position, None, None)?;
                builder = builder + geom;
            }
            GeomKind::Text => {
                let mut geom = geom_text();

                geom = apply_text_layer_mapping(geom, &layer.mapping);

                if let Some(color) = layer.params.color.as_ref() {
                    geom = geom.color(parse_color(color)?);
                }
                if let Some(alpha) = layer.params.alpha {
                    geom = geom.alpha(alpha);
                }
                if let Some(size) = layer.params.size {
                    geom = geom.size(size);
                }
                if let Some(angle) = layer.params.angle {
                    geom = geom.angle(angle);
                }

                let geom = finalize_layer(geom, &layer.stat, &layer.position, None, None)?;
                builder = builder + geom;
            }
            GeomKind::Label => {
                let mut geom = geom_label();

                geom = apply_label_layer_mapping(geom, &layer.mapping);

                if let Some(color) = layer.params.color.as_ref() {
                    geom = geom.color(parse_color(color)?);
                }
                if let Some(fill) = layer.params.fill.as_ref() {
                    geom = geom.fill(parse_color(fill)?);
                }
                if let Some(alpha) = layer.params.alpha {
                    geom = geom.alpha(alpha);
                }
                if let Some(size) = layer.params.size {
                    geom = geom.size(size);
                }
                if let Some(angle) = layer.params.angle {
                    geom = geom.angle(angle);
                }
                if let Some(padding) = layer.params.padding {
                    geom = geom.padding(padding);
                }
                if let Some(radius) = layer.params.radius {
                    geom = geom.radius(radius);
                }

                let geom = finalize_layer(geom, &layer.stat, &layer.position, None, None)?;
                builder = builder + geom;
            }
            GeomKind::Errorbar => {
                let mut geom = geom_errorbar();

                geom = apply_errorbar_layer_mapping(geom, &layer.mapping);

                if let Some(color) = layer.params.color.as_ref() {
                    geom = geom.color(parse_color(color)?);
                }
                if let Some(alpha) = layer.params.alpha {
                    geom = geom.alpha(alpha);
                }
                if let Some(size) = layer.params.size {
                    geom = geom.size(size);
                }
                if let Some(linetype) = layer.params.linetype.as_ref() {
                    geom = geom.linestyle(LineStyle::from(linetype.as_str()));
                }
                if let Some(width) = layer.params.width {
                    geom = geom.width(width);
                }

                let geom = finalize_layer(geom, &layer.stat, &layer.position, None, None)?;
                builder = builder + geom;
            }
            GeomKind::Smooth => {
                let mut geom = geom_smooth();

                // Smooth stat is applied automatically, but can be overridden
                if let Some(confidence_interval) = layer.params.confidence_interval {
                    geom = geom.confidence_interval(confidence_interval);
                }

                geom = apply_smooth_layer_mapping(geom, &layer.mapping);

                if let Some(color) = layer.params.color.as_ref() {
                    geom = geom.color(parse_color(color)?);
                }
                if let Some(fill) = layer.params.fill.as_ref() {
                    geom = geom.fill(parse_color(fill)?);
                }
                if let Some(alpha) = layer.params.alpha {
                    geom = geom.alpha(alpha);
                }
                if let Some(size) = layer.params.size {
                    geom = geom.size(size);
                }
                if let Some(linetype) = layer.params.linetype.as_ref() {
                    geom = geom.linestyle(LineStyle::from(linetype.as_str()));
                }

                let geom = finalize_layer(geom, &layer.stat, &layer.position, None, None)?;
                builder = builder + geom;
            }
        }
    }

    let plot = builder.build()?;
    Ok(plot)
}

fn render_plot(_plot: &gogplot::plot::Plot, output: &OutputSpec) -> Result<(), Box<dyn Error>> {
    // TODO: derive defaults from output or use theme defaults
    let filename = output
        .filename
        .clone()
        .unwrap_or_else(|| "output.png".to_string());
    let width = output.width.unwrap_or(800);
    let height = output.height.unwrap_or(600);

    _plot.save(&filename, width, height)?;
    println!("Wrote {}", filename);
    Ok(())
}

// ----- helpers -----

#[derive(Clone, Copy)]
enum DomainHint {
    Continuous,
    Discrete,
    Unknown,
}

fn parse_column_ref(s: &str) -> (String, DomainHint) {
    if let Some(rest) = s.strip_prefix('$') {
        (rest.to_string(), DomainHint::Discrete)
    } else if let Some(rest) = s.strip_prefix('~') {
        (rest.to_string(), DomainHint::Continuous)
    } else {
        (s.to_string(), DomainHint::Unknown)
    }
}

fn build_stat_from_spec(
    stat: &Option<StatSpec>,
    default_axis: Option<&str>,
) -> Result<Option<Box<dyn Stat>>, Box<dyn Error>> {
    match stat {
        Some(StatSpec::Named { name: StatName::Summary, aesthetic }) => {
            let target = aesthetic
                .as_deref()
                .or(default_axis)
                .ok_or_else(|| "summary stat requires an aesthetic (x/y)".to_string())?
                .trim();
            let (axis, dom) = parse_column_ref(target);
            let domain = match dom {
                DomainHint::Discrete => AestheticDomain::Discrete,
                _ => AestheticDomain::Continuous,
            };

            let aesthetic = match axis.to_ascii_lowercase().as_str() {
                "x" => Aesthetic::X(domain),
                "y" => Aesthetic::Y(domain),
                other => {
                    return Err(format!(
                        "summary stat aesthetic must be 'x' or 'y', got '{}'",
                        other
                    )
                    .into())
                }
            };

            Ok(Some(Box::new(Summary::from(aesthetic))))
        }
        Some(StatSpec::Simple(StatName::Summary)) => {
            let target = default_axis.ok_or_else(|| "summary stat requires an aesthetic (x/y)".to_string())?;
            let (axis, dom) = parse_column_ref(target);
            let domain = match dom {
                DomainHint::Discrete => AestheticDomain::Discrete,
                _ => AestheticDomain::Continuous,
            };

            let aesthetic = match axis.to_ascii_lowercase().as_str() {
                "x" => Aesthetic::X(domain),
                "y" => Aesthetic::Y(domain),
                other => {
                    return Err(format!(
                        "summary stat aesthetic must be 'x' or 'y', got '{}'",
                        other
                    )
                    .into())
                }
            };

            Ok(Some(Box::new(Summary::from(aesthetic))))
        }
        Some(StatSpec::Named { name: StatName::Count, .. })
        | Some(StatSpec::Simple(StatName::Count)) => Ok(Some(Box::new(Count::default()))),
        Some(StatSpec::Named { name: StatName::Bin, .. })
        | Some(StatSpec::Simple(StatName::Bin)) => Ok(Some(Box::new(Bin::default()))),
        None => Ok(None),
    }
}

fn build_position_from_spec(position: &Option<String>) -> Result<Option<Box<dyn Position>>, Box<dyn Error>> {
    match position.as_deref() {
        Some(name) => {
            let normalized = name.to_ascii_lowercase();
            let pos: Box<dyn Position> = match normalized.as_str() {
                "dodge" => Box::new(Dodge::default()),
                "stack" => Box::new(Stack::default()),
                other => return Err(format!("unknown position '{}'; expected 'dodge' or 'stack'", other).into()),
            };
            Ok(Some(pos))
        }
        None => Ok(None),
    }
}

fn apply_stat<B: LayerBuilder>(
    mut builder: B,
    stat: &Option<StatSpec>,
    default_axis: Option<&str>,
    default_stat: Option<Box<dyn Stat>>,
) -> Result<B, Box<dyn Error>> {
    if let Some(stat) = build_stat_from_spec(stat, default_axis)? {
        builder.set_stat(stat);
    } else if let Some(default_stat) = default_stat {
        builder.set_stat(default_stat);
    }
    Ok(builder)
}

fn apply_position<B: LayerBuilder>(
    mut builder: B,
    position: &Option<String>,
) -> Result<B, Box<dyn Error>> {
    if let Some(position) = build_position_from_spec(position)? {
        builder.set_position(position);
    }
    Ok(builder)
}

fn finalize_layer<B: LayerBuilder>(
    builder: B,
    stat: &Option<StatSpec>,
    position: &Option<String>,
    default_axis: Option<&str>,
    default_stat: Option<Box<dyn Stat>>,
) -> Result<B, Box<dyn Error>> {
    let builder = apply_stat(builder, stat, default_axis, default_stat)?;
    let builder = apply_position(builder, position)?;
    Ok(builder)
}

fn apply_global_mapping<'a>(builder: gogplot::plot::PlotBuilder<'a>, mapping: &MappingSpec) -> gogplot::plot::PlotBuilder<'a> {
    if mapping.x.is_none()
        && mapping.y.is_none()
        && mapping.color.is_none()
        && mapping.fill.is_none()
        && mapping.size.is_none()
        && mapping.alpha.is_none()
        && mapping.linetype.is_none()
        && mapping.xintercept.is_none()
        && mapping.yintercept.is_none()
        && mapping.group.is_none()
        && mapping.xmin.is_none()
        && mapping.xmax.is_none()
        && mapping.ymin.is_none()
        && mapping.ymax.is_none()
        && mapping.xbegin.is_none()
        && mapping.xend.is_none()
        && mapping.ybegin.is_none()
        && mapping.yend.is_none()
        && mapping.label.is_none()
    {
        return builder;
    }

    builder.aes(|a| {
        if let Some(ref x) = mapping.x {
            let (col, dom) = parse_column_ref(x);
            match dom {
                DomainHint::Discrete => a.x_discrete(&col),
                _ => a.x_continuous(&col),
            }
        }
        if let Some(ref y) = mapping.y {
            let (col, dom) = parse_column_ref(y);
            match dom {
                DomainHint::Discrete => a.y_discrete(&col),
                _ => a.y_continuous(&col),
            }
        }
        if let Some(ref color) = mapping.color {
            let (col, dom) = parse_column_ref(color);
            match dom {
                DomainHint::Discrete => a.color_discrete(&col),
                _ => a.color_continuous(&col),
            }
        }
        if let Some(ref fill) = mapping.fill {
            let (col, dom) = parse_column_ref(fill);
            match dom {
                DomainHint::Discrete => a.fill_discrete(&col),
                _ => a.fill_continuous(&col),
            }
        }
        if let Some(ref size) = mapping.size {
            let (col, dom) = parse_column_ref(size);
            match dom {
                DomainHint::Discrete => a.size_discrete(&col),
                _ => a.size_continuous(&col),
            }
        }
        if let Some(ref alpha) = mapping.alpha {
            let (col, dom) = parse_column_ref(alpha);
            match dom {
                DomainHint::Discrete => a.alpha_discrete(&col),
                _ => a.alpha_continuous(&col),
            }
        }
        if let Some(ref lt) = mapping.linetype {
            let (col, _dom) = parse_column_ref(lt);
            a.linestyle(&col);
        }
        if let Some(ref xint) = mapping.xintercept {
            let (col, _dom) = parse_column_ref(xint);
            a.x_intercept(&col);
        }
        if let Some(ref yint) = mapping.yintercept {
            let (col, _dom) = parse_column_ref(yint);
            a.y_intercept(&col);
        }
        if let Some(ref group) = mapping.group {
            let (col, _dom) = parse_column_ref(group);
            a.group(&col);
        }
        if let Some(ref xmin) = mapping.xmin {
            let (col, _dom) = parse_column_ref(xmin);
            a.xmin(&col);
        }
        if let Some(ref xmax) = mapping.xmax {
            let (col, _dom) = parse_column_ref(xmax);
            a.xmax(&col);
        }
        if let Some(ref ymin) = mapping.ymin {
            let (col, _dom) = parse_column_ref(ymin);
            a.ymin(&col);
        }
        if let Some(ref ymax) = mapping.ymax {
            let (col, _dom) = parse_column_ref(ymax);
            a.ymax(&col);
        }
        if let Some(ref xbegin) = mapping.xbegin {
            let (col, _dom) = parse_column_ref(xbegin);
            a.xbegin(&col);
        }
        if let Some(ref xend) = mapping.xend {
            let (col, _dom) = parse_column_ref(xend);
            a.xend(&col);
        }
        if let Some(ref ybegin) = mapping.ybegin {
            let (col, _dom) = parse_column_ref(ybegin);
            a.ybegin(&col);
        }
        if let Some(ref yend) = mapping.yend {
            let (col, _dom) = parse_column_ref(yend);
            a.yend(&col);
        }
        if let Some(ref label) = mapping.label {
            let (col, _dom) = parse_column_ref(label);
            a.label(&col);
        }
    })
}

fn apply_point_layer_mapping(mut geom: gogplot::geom::point::GeomPointBuilder, mapping: &MappingSpec) -> gogplot::geom::point::GeomPointBuilder {
    if mapping.x.is_none() && mapping.y.is_none() && mapping.color.is_none() && mapping.size.is_none() && mapping.linetype.is_none() && mapping.alpha.is_none() {
        return geom;
    }

    geom = geom.aes(|a| {
        if let Some(ref x) = mapping.x {
            let (col, dom) = parse_column_ref(x);
            match dom {
                DomainHint::Discrete => a.x_discrete(&col),
                _ => a.x_continuous(&col),
            }
        }
        if let Some(ref y) = mapping.y {
            let (col, dom) = parse_column_ref(y);
            match dom {
                DomainHint::Discrete => a.y_discrete(&col),
                _ => a.y_continuous(&col),
            }
        }
        if let Some(ref color) = mapping.color {
            let (col, dom) = parse_column_ref(color);
            match dom {
                DomainHint::Discrete => a.color_discrete(&col),
                _ => a.color_continuous(&col),
            }
        }
        if let Some(ref size) = mapping.size {
            let (col, dom) = parse_column_ref(size);
            match dom {
                DomainHint::Discrete => a.size_discrete(&col),
                _ => a.size_continuous(&col),
            }
        }
        if let Some(ref alpha) = mapping.alpha {
            let (col, dom) = parse_column_ref(alpha);
            match dom {
                DomainHint::Discrete => a.alpha_discrete(&col),
                _ => a.alpha_continuous(&col),
            }
        }
    });

    geom
}

fn apply_line_layer_mapping(mut geom: gogplot::geom::line::GeomLineBuilder, mapping: &MappingSpec) -> gogplot::geom::line::GeomLineBuilder {
    if mapping.x.is_none()
        && mapping.y.is_none()
        && mapping.color.is_none()
        && mapping.size.is_none()
        && mapping.linetype.is_none()
        && mapping.alpha.is_none()
    {
        return geom;
    }

    geom = geom.aes(|a| {
        if let Some(ref x) = mapping.x {
            let (col, dom) = parse_column_ref(x);
            match dom {
                DomainHint::Discrete => a.x_discrete(&col),
                _ => a.x_continuous(&col),
            }
        }
        if let Some(ref y) = mapping.y {
            let (col, dom) = parse_column_ref(y);
            match dom {
                DomainHint::Discrete => a.y_discrete(&col),
                _ => a.y_continuous(&col),
            }
        }
        if let Some(ref color) = mapping.color {
            let (col, dom) = parse_column_ref(color);
            match dom {
                DomainHint::Discrete => a.color_discrete(&col),
                _ => a.color_continuous(&col),
            }
        }
        if let Some(ref size) = mapping.size {
            let (col, dom) = parse_column_ref(size);
            match dom {
                DomainHint::Discrete => a.size_discrete(&col),
                _ => a.size_continuous(&col),
            }
        }
        if let Some(ref lt) = mapping.linetype {
            let (col, _dom) = parse_column_ref(lt);
            a.linestyle(&col);
        }
        if let Some(ref alpha) = mapping.alpha {
            let (col, dom) = parse_column_ref(alpha);
            match dom {
                DomainHint::Discrete => a.alpha_discrete(&col),
                _ => a.alpha_continuous(&col),
            }
        }
    });

    geom
}

fn apply_hline_layer_mapping(mut geom: gogplot::geom::hline::GeomHLineBuilder, mapping: &MappingSpec) -> gogplot::geom::hline::GeomHLineBuilder {
    if mapping.yintercept.is_none() && mapping.color.is_none() && mapping.size.is_none() && mapping.linetype.is_none() {
        return geom;
    }

    geom = geom.aes(|a| {
        if let Some(ref yint) = mapping.yintercept {
            let (col, _dom) = parse_column_ref(yint);
            a.y_intercept(&col);
        }
        if let Some(ref color) = mapping.color {
            let (col, dom) = parse_column_ref(color);
            match dom {
                DomainHint::Discrete => a.color_discrete(&col),
                _ => a.color_continuous(&col),
            }
        }
        if let Some(ref size) = mapping.size {
            let (col, dom) = parse_column_ref(size);
            match dom {
                DomainHint::Discrete => a.size_discrete(&col),
                _ => a.size_continuous(&col),
            }
        }
        if let Some(ref lt) = mapping.linetype {
            let (col, _dom) = parse_column_ref(lt);
            a.linestyle(&col);
        }
    });

    geom
}

fn apply_vline_layer_mapping(mut geom: gogplot::geom::vline::GeomVLineBuilder, mapping: &MappingSpec) -> gogplot::geom::vline::GeomVLineBuilder {
    if mapping.xintercept.is_none() && mapping.color.is_none() && mapping.size.is_none() && mapping.linetype.is_none() {
        return geom;
    }

    geom = geom.aes(|a| {
        if let Some(ref xint) = mapping.xintercept {
            let (col, _dom) = parse_column_ref(xint);
            a.x_intercept(&col);
        }
        if let Some(ref color) = mapping.color {
            let (col, dom) = parse_column_ref(color);
            match dom {
                DomainHint::Discrete => a.color_discrete(&col),
                _ => a.color_continuous(&col),
            }
        }
        if let Some(ref size) = mapping.size {
            let (col, dom) = parse_column_ref(size);
            match dom {
                DomainHint::Discrete => a.size_discrete(&col),
                _ => a.size_continuous(&col),
            }
        }
        if let Some(ref lt) = mapping.linetype {
            let (col, _dom) = parse_column_ref(lt);
            a.linestyle(&col);
        }
    });

    geom
}

fn apply_boxplot_layer_mapping(mut geom: gogplot::geom::boxplot::GeomBoxplotBuilder, mapping: &MappingSpec) -> gogplot::geom::boxplot::GeomBoxplotBuilder {
    if mapping.x.is_none() && mapping.y.is_none() && mapping.fill.is_none() && mapping.alpha.is_none() && mapping.group.is_none() {
        return geom;
    }

    geom = geom.aes(|a| {
        if let Some(ref x) = mapping.x {
            let (col, dom) = parse_column_ref(x);
            match dom {
                DomainHint::Discrete => a.x_discrete(&col),
                _ => a.x_discrete(&col),
            }
        }
        if let Some(ref y) = mapping.y {
            let (col, _dom) = parse_column_ref(y);
            a.y_continuous(&col);
        }
        if let Some(ref fill) = mapping.fill {
            let (col, dom) = parse_column_ref(fill);
            match dom {
                DomainHint::Discrete => a.fill_discrete(&col),
                _ => a.fill_continuous(&col),
            }
        }
        if let Some(ref alpha) = mapping.alpha {
            let (col, dom) = parse_column_ref(alpha);
            match dom {
                DomainHint::Discrete => a.alpha_discrete(&col),
                _ => a.alpha_continuous(&col),
            }
        }
        if let Some(ref group) = mapping.group {
            let (col, _dom) = parse_column_ref(group);
            a.group(&col);
        }
    });

    geom
}

fn apply_density_layer_mapping(mut geom: gogplot::geom::density::GeomDensityBuilder, mapping: &MappingSpec) -> gogplot::geom::density::GeomDensityBuilder {
    if mapping.x.is_none() && mapping.y.is_none() && mapping.color.is_none() && mapping.fill.is_none() && mapping.alpha.is_none() && mapping.linetype.is_none() && mapping.group.is_none() {
        return geom;
    }

    geom = geom.aes(|a| {
        if let Some(ref x) = mapping.x {
            let (col, dom) = parse_column_ref(x);
            match dom {
                DomainHint::Discrete => a.x_continuous(&col),
                _ => a.x_continuous(&col),
            }
        }
        if let Some(ref y) = mapping.y {
            let (col, dom) = parse_column_ref(y);
            match dom {
                DomainHint::Discrete => a.y_continuous(&col),
                _ => a.y_continuous(&col),
            }
        }
        if let Some(ref color) = mapping.color {
            let (col, dom) = parse_column_ref(color);
            match dom {
                DomainHint::Discrete => a.color_discrete(&col),
                _ => a.color_continuous(&col),
            }
        }
        if let Some(ref fill) = mapping.fill {
            let (col, dom) = parse_column_ref(fill);
            match dom {
                DomainHint::Discrete => a.fill_discrete(&col),
                _ => a.fill_continuous(&col),
            }
        }
        if let Some(ref lt) = mapping.linetype {
            let (col, _dom) = parse_column_ref(lt);
            a.linestyle(&col);
        }
        if let Some(ref alpha) = mapping.alpha {
            let (col, dom) = parse_column_ref(alpha);
            match dom {
                DomainHint::Discrete => a.alpha_discrete(&col),
                _ => a.alpha_continuous(&col),
            }
        }
        if let Some(ref group) = mapping.group {
            let (col, _dom) = parse_column_ref(group);
            a.group(&col);
        }
    });

    geom
}
fn apply_bar_layer_mapping(mut geom: gogplot::geom::bar::GeomBarBuilder, mapping: &MappingSpec) -> gogplot::geom::bar::GeomBarBuilder {
    if mapping.x.is_none() && mapping.y.is_none() && mapping.fill.is_none() && mapping.alpha.is_none() && mapping.group.is_none() {
        return geom;
    }

    geom = geom.aes(|a| {
        if let Some(ref x) = mapping.x {
            let (col, dom) = parse_column_ref(x);
            match dom {
                DomainHint::Discrete => a.x_discrete(&col),
                _ => a.x_discrete(&col),
            }
        }
        if let Some(ref y) = mapping.y {
            let (col, _dom) = parse_column_ref(y);
            a.y_continuous(&col);
        }
        if let Some(ref fill) = mapping.fill {
            let (col, dom) = parse_column_ref(fill);
            match dom {
                DomainHint::Discrete => a.fill_discrete(&col),
                _ => a.fill_continuous(&col),
            }
        }
        if let Some(ref alpha) = mapping.alpha {
            let (col, dom) = parse_column_ref(alpha);
            match dom {
                DomainHint::Discrete => a.alpha_discrete(&col),
                _ => a.alpha_continuous(&col),
            }
        }
        if let Some(ref group) = mapping.group {
            let (col, _dom) = parse_column_ref(group);
            a.group(&col);
        }
    });

    geom
}

fn apply_histogram_layer_mapping(mut geom: gogplot::geom::histogram::GeomHistogramBuilder, mapping: &MappingSpec) -> gogplot::geom::histogram::GeomHistogramBuilder {
    if mapping.x.is_none() && mapping.y.is_none() && mapping.fill.is_none() && mapping.alpha.is_none() && mapping.group.is_none() {
        return geom;
    }

    geom = geom.aes(|a| {
        if let Some(ref x) = mapping.x {
            let (col, _dom) = parse_column_ref(x);
            a.x_continuous(&col);
        }
        if let Some(ref y) = mapping.y {
            let (col, _dom) = parse_column_ref(y);
            a.y_continuous(&col);
        }
        if let Some(ref fill) = mapping.fill {
            let (col, dom) = parse_column_ref(fill);
            match dom {
                DomainHint::Discrete => a.fill_discrete(&col),
                _ => a.fill_continuous(&col),
            }
        }
        if let Some(ref alpha) = mapping.alpha {
            let (col, dom) = parse_column_ref(alpha);
            match dom {
                DomainHint::Discrete => a.alpha_discrete(&col),
                _ => a.alpha_continuous(&col),
            }
        }
        if let Some(ref group) = mapping.group {
            let (col, _dom) = parse_column_ref(group);
            a.group(&col);
        }
    });

    geom
}

fn apply_rect_layer_mapping(mut geom: gogplot::geom::rect::GeomRectBuilder, mapping: &MappingSpec) -> gogplot::geom::rect::GeomRectBuilder {
    if mapping.xmin.is_none() && mapping.xmax.is_none() && mapping.ymin.is_none() && mapping.ymax.is_none() && mapping.fill.is_none() && mapping.alpha.is_none() && mapping.group.is_none() {
        return geom;
    }

    geom = geom.aes(|a| {
        if let Some(ref xmin) = mapping.xmin {
            let (col, _dom) = parse_column_ref(xmin);
            a.xmin(&col);
        }
        if let Some(ref xmax) = mapping.xmax {
            let (col, _dom) = parse_column_ref(xmax);
            a.xmax(&col);
        }
        if let Some(ref ymin) = mapping.ymin {
            let (col, _dom) = parse_column_ref(ymin);
            a.ymin(&col);
        }
        if let Some(ref ymax) = mapping.ymax {
            let (col, _dom) = parse_column_ref(ymax);
            a.ymax(&col);
        }
        if let Some(ref fill) = mapping.fill {
            let (col, dom) = parse_column_ref(fill);
            match dom {
                DomainHint::Discrete => a.fill_discrete(&col),
                _ => a.fill_continuous(&col),
            }
        }
        if let Some(ref alpha) = mapping.alpha {
            let (col, dom) = parse_column_ref(alpha);
            match dom {
                DomainHint::Discrete => a.alpha_discrete(&col),
                _ => a.alpha_continuous(&col),
            }
        }
        if let Some(ref group) = mapping.group {
            let (col, _dom) = parse_column_ref(group);
            a.group(&col);
        }
    });

    geom
}

fn apply_segment_layer_mapping(mut geom: gogplot::geom::segment::GeomSegmentBuilder, mapping: &MappingSpec) -> gogplot::geom::segment::GeomSegmentBuilder {
    if mapping.xbegin.is_none() && mapping.xend.is_none() && mapping.ybegin.is_none() && mapping.yend.is_none() && mapping.color.is_none() && mapping.alpha.is_none() && mapping.size.is_none() && mapping.linetype.is_none() && mapping.group.is_none() {
        return geom;
    }

    geom = geom.aes(|a| {
        if let Some(ref xbegin) = mapping.xbegin {
            let (col, _dom) = parse_column_ref(xbegin);
            a.xbegin(&col);
        }
        if let Some(ref xend) = mapping.xend {
            let (col, _dom) = parse_column_ref(xend);
            a.xend(&col);
        }
        if let Some(ref ybegin) = mapping.ybegin {
            let (col, _dom) = parse_column_ref(ybegin);
            a.ybegin(&col);
        }
        if let Some(ref yend) = mapping.yend {
            let (col, _dom) = parse_column_ref(yend);
            a.yend(&col);
        }
        if let Some(ref color) = mapping.color {
            let (col, dom) = parse_column_ref(color);
            match dom {
                DomainHint::Discrete => a.color_discrete(&col),
                _ => a.color_continuous(&col),
            }
        }
        if let Some(ref alpha) = mapping.alpha {
            let (col, dom) = parse_column_ref(alpha);
            match dom {
                DomainHint::Discrete => a.alpha_discrete(&col),
                _ => a.alpha_continuous(&col),
            }
        }
        if let Some(ref size) = mapping.size {
            let (col, dom) = parse_column_ref(size);
            match dom {
                DomainHint::Discrete => a.size_discrete(&col),
                _ => a.size_continuous(&col),
            }
        }
        if let Some(ref linetype) = mapping.linetype {
            let (col, _dom) = parse_column_ref(linetype);
            a.linestyle(&col);
        }
        if let Some(ref group) = mapping.group {
            let (col, _dom) = parse_column_ref(group);
            a.group(&col);
        }
    });

    geom
}

fn apply_text_layer_mapping(mut geom: gogplot::geom::text::GeomTextBuilder, mapping: &MappingSpec) -> gogplot::geom::text::GeomTextBuilder {
    if mapping.x.is_none() && mapping.y.is_none() && mapping.label.is_none() && mapping.color.is_none() && mapping.alpha.is_none() && mapping.size.is_none() {
        return geom;
    }

    geom = geom.aes(|a| {
        if let Some(ref x) = mapping.x {
            let (col, dom) = parse_column_ref(x);
            match dom {
                DomainHint::Discrete => a.x_discrete(&col),
                _ => a.x_continuous(&col),
            }
        }
        if let Some(ref y) = mapping.y {
            let (col, dom) = parse_column_ref(y);
            match dom {
                DomainHint::Discrete => a.y_discrete(&col),
                _ => a.y_continuous(&col),
            }
        }
        if let Some(ref label) = mapping.label {
            let (col, _dom) = parse_column_ref(label);
            a.label(&col);
        }
        if let Some(ref color) = mapping.color {
            let (col, dom) = parse_column_ref(color);
            match dom {
                DomainHint::Discrete => a.color_discrete(&col),
                _ => a.color_continuous(&col),
            }
        }
        if let Some(ref alpha) = mapping.alpha {
            let (col, dom) = parse_column_ref(alpha);
            match dom {
                DomainHint::Discrete => a.alpha_discrete(&col),
                _ => a.alpha_continuous(&col),
            }
        }
        if let Some(ref size) = mapping.size {
            let (col, dom) = parse_column_ref(size);
            match dom {
                DomainHint::Discrete => a.size_discrete(&col),
                _ => a.size_continuous(&col),
            }
        }
    });

    geom
}

fn apply_label_layer_mapping(mut geom: gogplot::geom::label::GeomLabelBuilder, mapping: &MappingSpec) -> gogplot::geom::label::GeomLabelBuilder {
    if mapping.x.is_none() && mapping.y.is_none() && mapping.label.is_none() && mapping.color.is_none() && mapping.alpha.is_none() && mapping.size.is_none() {
        return geom;
    }

    geom = geom.aes(|a| {
        if let Some(ref x) = mapping.x {
            let (col, dom) = parse_column_ref(x);
            match dom {
                DomainHint::Discrete => a.x_discrete(&col),
                _ => a.x_continuous(&col),
            }
        }
        if let Some(ref y) = mapping.y {
            let (col, dom) = parse_column_ref(y);
            match dom {
                DomainHint::Discrete => a.y_discrete(&col),
                _ => a.y_continuous(&col),
            }
        }
        if let Some(ref label) = mapping.label {
            let (col, _dom) = parse_column_ref(label);
            a.label(&col);
        }
        if let Some(ref color) = mapping.color {
            let (col, dom) = parse_column_ref(color);
            match dom {
                DomainHint::Discrete => a.color_discrete(&col),
                _ => a.color_continuous(&col),
            }
        }
        if let Some(ref alpha) = mapping.alpha {
            let (col, dom) = parse_column_ref(alpha);
            match dom {
                DomainHint::Discrete => a.alpha_discrete(&col),
                _ => a.alpha_continuous(&col),
            }
        }
        if let Some(ref size) = mapping.size {
            let (col, dom) = parse_column_ref(size);
            match dom {
                DomainHint::Discrete => a.size_discrete(&col),
                _ => a.size_continuous(&col),
            }
        }
    });

    geom
}

fn apply_errorbar_layer_mapping(mut geom: gogplot::geom::errorbar::GeomErrorbarBuilder, mapping: &MappingSpec) -> gogplot::geom::errorbar::GeomErrorbarBuilder {
    if mapping.x.is_none() && mapping.ymin.is_none() && mapping.ymax.is_none() && mapping.color.is_none() && mapping.alpha.is_none() && mapping.size.is_none() && mapping.linetype.is_none() {
        return geom;
    }

    geom = geom.aes(|a| {
        if let Some(ref x) = mapping.x {
            let (col, _dom) = parse_column_ref(x);
            a.x_continuous(&col);
        }
        if let Some(ref ymin) = mapping.ymin {
            let (col, _dom) = parse_column_ref(ymin);
            a.ymin(&col);
        }
        if let Some(ref ymax) = mapping.ymax {
            let (col, _dom) = parse_column_ref(ymax);
            a.ymax(&col);
        }
        if let Some(ref color) = mapping.color {
            let (col, dom) = parse_column_ref(color);
            match dom {
                DomainHint::Discrete => a.color_discrete(&col),
                _ => a.color_continuous(&col),
            }
        }
        if let Some(ref alpha) = mapping.alpha {
            let (col, dom) = parse_column_ref(alpha);
            match dom {
                DomainHint::Discrete => a.alpha_discrete(&col),
                _ => a.alpha_continuous(&col),
            }
        }
        if let Some(ref size) = mapping.size {
            let (col, dom) = parse_column_ref(size);
            match dom {
                DomainHint::Discrete => a.size_discrete(&col),
                _ => a.size_continuous(&col),
            }
        }
        if let Some(ref linetype) = mapping.linetype {
            let (col, _dom) = parse_column_ref(linetype);
            a.linestyle(&col);
        }
    });

    geom
}

fn apply_smooth_layer_mapping(mut geom: gogplot::geom::smooth::GeomSmoothBuilder, mapping: &MappingSpec) -> gogplot::geom::smooth::GeomSmoothBuilder {
    if mapping.x.is_none() && mapping.y.is_none() && mapping.color.is_none() && mapping.alpha.is_none() && mapping.size.is_none() && mapping.linetype.is_none() {
        return geom;
    }

    geom = geom.aes(|a| {
        if let Some(ref x) = mapping.x {
            let (col, _dom) = parse_column_ref(x);
            a.x_continuous(&col);
        }
        if let Some(ref y) = mapping.y {
            let (col, _dom) = parse_column_ref(y);
            a.y_continuous(&col);
        }
        if let Some(ref color) = mapping.color {
            let (col, dom) = parse_column_ref(color);
            match dom {
                DomainHint::Discrete => a.color_discrete(&col),
                _ => a.color_continuous(&col),
            }
        }
        if let Some(ref alpha) = mapping.alpha {
            let (col, dom) = parse_column_ref(alpha);
            match dom {
                DomainHint::Discrete => a.alpha_discrete(&col),
                _ => a.alpha_continuous(&col),
            }
        }
        if let Some(ref size) = mapping.size {
            let (col, dom) = parse_column_ref(size);
            match dom {
                DomainHint::Discrete => a.size_discrete(&col),
                _ => a.size_continuous(&col),
            }
        }
        if let Some(ref linetype) = mapping.linetype {
            let (col, _dom) = parse_column_ref(linetype);
            a.linestyle(&col);
        }
    });

    geom
}

fn parse_color(name: &str) -> Result<Color, Box<dyn Error>> {
    let map: HashMap<String, Color> = color::color_map()
        .iter()
        .map(|(k, v)| (k.to_ascii_lowercase(), v.clone()))
        .collect();
    let key = name.to_ascii_lowercase();
    if let Some(c) = map.get(&key) {
        Ok(*c)
    } else {
        Err(format!("unknown color: {}", name).into())
    }
}

fn load_csv_to_dataframe(path: &Path) -> Result<DataFrame, Box<dyn Error>> {
    let mut reader = csv::Reader::from_path(path)?;
    let headers = reader
        .headers()
        .map(|h| h.iter().map(|s| s.to_string()).collect::<Vec<_>>())?;

    let mut columns: Vec<Vec<String>> = vec![Vec::new(); headers.len()];
    for record in reader.records() {
        let record = record?;
        for (i, field) in record.iter().enumerate() {
            if let Some(col) = columns.get_mut(i) {
                col.push(field.to_string());
            }
        }
    }

    let mut df = DataFrame::new();
    for (name, values) in headers.into_iter().zip(columns.into_iter()) {
        df.add_column(name, infer_column(values));
    }

    Ok(df)
}

fn build_inline_dataframe(data: &HashMap<String, Vec<Value>>) -> Result<DataFrame, Box<dyn Error>> {
    let mut df = DataFrame::new();
    let mut expected_len: Option<usize> = None;
    for (name, values) in data {
        if let Some(len) = expected_len {
            if values.len() != len {
                return Err(format!(
                    "column '{}' has length {} but expected {}",
                    name,
                    values.len(),
                    len
                )
                .into());
            }
        } else {
            expected_len = Some(values.len());
        }

        let column_strings = values
            .iter()
            .map(|v| value_to_string(v))
            .collect::<Vec<_>>();
        df.add_column(name.clone(), infer_column(column_strings));
    }
    Ok(df)
}

fn infer_column(values: Vec<String>) -> VectorValue {
    if values.iter().all(|v| v.parse::<i64>().is_ok()) {
        let parsed = values
            .into_iter()
            .map(|v| v.parse::<i64>().unwrap())
            .collect();
        return VectorValue::Int(parsed);
    }

    if values.iter().all(|v| v.parse::<f64>().is_ok()) {
        let parsed = values
            .into_iter()
            .map(|v| v.parse::<f64>().unwrap())
            .collect();
        return VectorValue::Float(parsed);
    }

    if values
        .iter()
        .all(|v| v.eq_ignore_ascii_case("true") || v.eq_ignore_ascii_case("false"))
    {
        let parsed = values
            .into_iter()
            .map(|v| v.eq_ignore_ascii_case("true"))
            .collect();
        return VectorValue::Bool(parsed);
    }

    VectorValue::Str(values)
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::String(s) => s.clone(),
        Value::Null => "".to_string(),
        Value::Array(_) | Value::Object(_) => v.to_string(),
    }
}
