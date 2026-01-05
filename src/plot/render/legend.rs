// Legend rendering

use crate::error::PlotError;
use crate::guide::{Guides, LegendEntry, LegendGuide, LegendPosition, LegendType};
use crate::layer::Layer;
use crate::scale::ScaleSet;
use crate::scale::traits::{
    ColorRangeScale, ContinuousDomainScale, ContinuousRangeScale, DiscreteDomainScale,
};
use crate::theme::{Color, Theme};
use crate::utils::set::DiscreteSet;
use crate::visuals::Shape;
use cairo::Context;

use super::cairo_helpers::{apply_color, apply_fill_style, apply_font, apply_line_style};

/// Helper to create discrete legend entries from categories
fn create_discrete_entries<T>(
    categories: &DiscreteSet,
    mut map_value: impl FnMut(&crate::data::DiscreteValue) -> Option<T>,
    mut apply_value: impl FnMut(&mut LegendEntry, T),
) -> Vec<LegendEntry> {
    let mut entries = Vec::new();

    for i in 0..categories.len() {
        if let Some(value) = categories.get_at(i) {
            let label = match &value {
                crate::data::DiscreteValue::Int(x) => x.to_string(),
                crate::data::DiscreteValue::Str(x) => x.clone(),
                crate::data::DiscreteValue::Bool(x) => x.to_string(),
            };

            if let Some(mapped) = map_value(&value) {
                let mut entry = LegendEntry::new(label);
                apply_value(&mut entry, mapped);
                entries.push(entry);
            }
        }
    }

    entries
}

/// Helper to create a discrete color legend
fn create_discrete_color_legend(title: String, scales: &ScaleSet) -> LegendGuide {
    use crate::scale::traits::ColorRangeScale;
    let entries = create_discrete_entries(
        scales.color_discrete.categories(),
        |value| match value {
            crate::data::DiscreteValue::Int(x) => scales.color_discrete.map_value(x),
            crate::data::DiscreteValue::Str(x) => scales.color_discrete.map_value(x),
            crate::data::DiscreteValue::Bool(x) => scales.color_discrete.map_value(x),
        },
        |entry, color| {
            entry.color = Some(color);
            entry.size = Some(5.0);
        },
    );

    LegendGuide {
        title: Some(title),
        entries,
        legend_type: LegendType::Discrete,
        ..Default::default()
    }
}

/// Helper to create a continuous color legend
fn create_continuous_color_legend(title: String, scales: &ScaleSet) -> LegendGuide {
    if let Some(domain) = scales.color_continuous.domain() {
        let colors = vec![
            scales
                .color_continuous
                .map_value(&domain.0)
                .unwrap_or(crate::theme::color::BLACK),
            scales
                .color_continuous
                .map_value(&domain.1)
                .unwrap_or(crate::theme::color::WHITE),
        ];

        LegendGuide {
            title: Some(title),
            legend_type: LegendType::ColorBar {
                domain,
                colors,
                breaks: scales.color_continuous.breaks().to_vec(),
                labels: scales.color_continuous.labels().to_vec(),
            },
            ..Default::default()
        }
    } else {
        LegendGuide::default()
    }
}

/// Helper to create a discrete fill legend
fn create_discrete_fill_legend(title: String, scales: &ScaleSet) -> LegendGuide {
    use crate::scale::traits::ColorRangeScale;
    let entries = create_discrete_entries(
        scales.fill_discrete.categories(),
        |value| match value {
            crate::data::DiscreteValue::Int(x) => scales.fill_discrete.map_value(x),
            crate::data::DiscreteValue::Str(x) => scales.fill_discrete.map_value(x),
            crate::data::DiscreteValue::Bool(x) => scales.fill_discrete.map_value(x),
        },
        |entry, color| {
            entry.color = Some(color);
            entry.size = Some(5.0);
        },
    );

    LegendGuide {
        title: Some(title),
        entries,
        legend_type: LegendType::Discrete,
        ..Default::default()
    }
}

/// Helper to create a discrete size legend
fn create_discrete_size_legend(title: String, scales: &ScaleSet) -> LegendGuide {
    use crate::scale::traits::ContinuousRangeScale;
    let entries = create_discrete_entries(
        scales.size_discrete.categories(),
        |value| match value {
            crate::data::DiscreteValue::Int(x) => scales.size_discrete.map_value(x),
            crate::data::DiscreteValue::Str(x) => scales.size_discrete.map_value(x),
            crate::data::DiscreteValue::Bool(x) => scales.size_discrete.map_value(x),
        },
        |entry, size| {
            entry.color = Some(crate::theme::color::BLACK);
            entry.size = Some(size);
        },
    );

    LegendGuide {
        title: Some(title),
        entries,
        legend_type: LegendType::Discrete,
        ..Default::default()
    }
}

/// Helper to create a continuous size legend
fn create_continuous_size_legend(title: String, scales: &ScaleSet) -> LegendGuide {
    if let Some(domain) = scales.size_continuous.domain() {
        let mut entries = Vec::new();
        let num_breaks = 4;

        for i in 0..num_breaks {
            let t = i as f64 / (num_breaks - 1) as f64;
            let value = domain.0 + t * (domain.1 - domain.0);
            let size = scales.size_continuous.map_value(&value);

            if let Some(size) = size {
                let label = if (value - value.round()).abs() < 0.01 {
                    format!("{}", value.round() as i64)
                } else {
                    format!("{:.1}", value)
                };

                entries.push(
                    LegendEntry::new(label)
                        .color(crate::theme::color::BLACK)
                        .size(size),
                );
            }
        }

        LegendGuide {
            title: Some(title),
            entries,
            legend_type: LegendType::Discrete,
            ..Default::default()
        }
    } else {
        LegendGuide::default()
    }
}

/// Helper to create a discrete shape legend
fn create_discrete_shape_legend(title: String, scales: &ScaleSet) -> LegendGuide {
    use crate::scale::traits::ShapeRangeScale;

    let entries = create_discrete_entries(
        scales.shape_scale.categories(),
        |value| match value {
            crate::data::DiscreteValue::Int(x) => scales.shape_scale.map_value(x),
            crate::data::DiscreteValue::Str(x) => scales.shape_scale.map_value(x),
            crate::data::DiscreteValue::Bool(x) => scales.shape_scale.map_value(x),
        },
        |entry, shape| {
            entry.color = Some(crate::theme::color::BLACK);
            entry.shape = Some(shape);
            entry.size = Some(5.0);
        },
    );

    LegendGuide {
        title: Some(title),
        entries,
        legend_type: LegendType::Discrete,
        ..Default::default()
    }
}

/// Helper to create a discrete alpha legend
fn create_discrete_alpha_legend(title: String, scales: &ScaleSet) -> LegendGuide {
    use crate::scale::traits::ContinuousRangeScale;

    let entries = create_discrete_entries(
        scales.alpha_discrete.categories(),
        |value| match value {
            crate::data::DiscreteValue::Int(x) => scales.alpha_discrete.map_value(x),
            crate::data::DiscreteValue::Str(x) => scales.alpha_discrete.map_value(x),
            crate::data::DiscreteValue::Bool(x) => scales.alpha_discrete.map_value(x),
        },
        |entry, alpha| {
            // Show alpha as gray circles with varying transparency
            let gray = 128u8;
            let alpha_u8 = (alpha * 255.0) as u8;
            entry.color = Some(Color(gray, gray, gray, alpha_u8));
            entry.size = Some(5.0);
        },
    );

    LegendGuide {
        title: Some(title),
        entries,
        legend_type: LegendType::Discrete,
        ..Default::default()
    }
}

/// Helper to create a continuous alpha legend
fn create_continuous_alpha_legend(title: String, scales: &ScaleSet) -> LegendGuide {
    use crate::scale::traits::ContinuousRangeScale;

    if let Some(domain) = scales.alpha_continuous.domain() {
        let mut entries = Vec::new();
        let num_breaks = 4;

        for i in 0..num_breaks {
            let t = i as f64 / (num_breaks - 1) as f64;
            let value = domain.0 + t * (domain.1 - domain.0);
            let alpha = scales.alpha_continuous.map_value(&value);

            if let Some(alpha) = alpha {
                let label = if (value - value.round()).abs() < 0.01 {
                    format!("{}", value.round() as i64)
                } else {
                    format!("{:.1}", value)
                };

                // Show alpha as gray circles with varying transparency
                let gray = 128u8;
                let alpha_u8 = (alpha * 255.0) as u8;

                entries.push(
                    LegendEntry::new(label)
                        .color(Color(gray, gray, gray, alpha_u8))
                        .size(5.0),
                );
            }
        }

        LegendGuide {
            title: Some(title),
            entries,
            legend_type: LegendType::Discrete,
            ..Default::default()
        }
    } else {
        LegendGuide::default()
    }
}

/// Create an individual legend for a single aesthetic
fn create_individual_legend(
    guides: &mut Guides,
    property: crate::aesthetics::AestheticProperty,
    domain: crate::aesthetics::AestheticDomain,
    title: &str,
    scales: &ScaleSet,
) {
    use crate::aesthetics::{AestheticDomain, AestheticProperty};

    let check_and_set = |guide_field: &mut Option<LegendGuide>| {
        if guide_field.is_none() {
            *guide_field = Some(match domain {
                AestheticDomain::Discrete => match property {
                    AestheticProperty::Color => {
                        create_discrete_color_legend(title.to_string(), scales)
                    }
                    AestheticProperty::Fill => {
                        create_discrete_fill_legend(title.to_string(), scales)
                    }
                    AestheticProperty::Size => {
                        create_discrete_size_legend(title.to_string(), scales)
                    }
                    AestheticProperty::Shape => {
                        create_discrete_shape_legend(title.to_string(), scales)
                    }
                    AestheticProperty::Alpha => {
                        create_discrete_alpha_legend(title.to_string(), scales)
                    }
                    _ => LegendGuide::default(),
                },
                AestheticDomain::Continuous => match property {
                    AestheticProperty::Color => {
                        create_continuous_color_legend(title.to_string(), scales)
                    }
                    AestheticProperty::Size => {
                        create_continuous_size_legend(title.to_string(), scales)
                    }
                    AestheticProperty::Alpha => {
                        create_continuous_alpha_legend(title.to_string(), scales)
                    }
                    _ => LegendGuide::default(),
                },
            });
        }
    };

    match property {
        AestheticProperty::Color => check_and_set(&mut guides.color),
        AestheticProperty::Fill => check_and_set(&mut guides.fill),
        AestheticProperty::Size => check_and_set(&mut guides.size),
        AestheticProperty::Shape => check_and_set(&mut guides.shape),
        AestheticProperty::Alpha => check_and_set(&mut guides.alpha),
        _ => {}
    }
}

/// Create a merged legend for multiple aesthetics mapping to the same column
fn create_merged_legend(
    guides: &mut Guides,
    aesthetics: &[MappedAesthetic],
    title: &str,
    scales: &ScaleSet,
) {
    use crate::aesthetics::{AestheticDomain, AestheticProperty};
    use crate::scale::traits::{ColorRangeScale, ContinuousRangeScale, ShapeRangeScale};

    let domain = aesthetics[0].domain;

    // Only handle discrete merging for now
    if domain != AestheticDomain::Discrete {
        // Fall back to individual legends for continuous
        for aesthetic in aesthetics {
            create_individual_legend(guides, aesthetic.property, aesthetic.domain, title, scales);
        }
        return;
    }

    // Determine which aesthetics we're merging
    let has_color = aesthetics
        .iter()
        .any(|a| a.property == AestheticProperty::Color);
    let has_fill = aesthetics
        .iter()
        .any(|a| a.property == AestheticProperty::Fill);
    let has_size = aesthetics
        .iter()
        .any(|a| a.property == AestheticProperty::Size);
    let has_shape = aesthetics
        .iter()
        .any(|a| a.property == AestheticProperty::Shape);
    let has_alpha = aesthetics
        .iter()
        .any(|a| a.property == AestheticProperty::Alpha);

    // Get categories from the first available scale
    let categories = if has_color {
        scales.color_discrete.categories()
    } else if has_fill {
        scales.fill_discrete.categories()
    } else if has_size {
        scales.size_discrete.categories()
    } else if has_shape {
        scales.shape_scale.categories()
    } else if has_alpha {
        scales.alpha_discrete.categories()
    } else {
        return;
    };

    // Create merged entries
    let mut entries = Vec::new();

    for i in 0..categories.len() {
        if let Some(value) = categories.get_at(i) {
            let label = match &value {
                crate::data::DiscreteValue::Int(x) => x.to_string(),
                crate::data::DiscreteValue::Str(x) => x.clone(),
                crate::data::DiscreteValue::Bool(x) => x.to_string(),
            };

            let mut entry = LegendEntry::new(label);

            // Get values from each scale
            if has_color {
                if let Some(color) = match &value {
                    crate::data::DiscreteValue::Int(x) => scales.color_discrete.map_value(x),
                    crate::data::DiscreteValue::Str(x) => scales.color_discrete.map_value(x),
                    crate::data::DiscreteValue::Bool(x) => scales.color_discrete.map_value(x),
                } {
                    entry.color = Some(color);
                }
            }

            if has_fill {
                if let Some(fill) = match &value {
                    crate::data::DiscreteValue::Int(x) => scales.fill_discrete.map_value(x),
                    crate::data::DiscreteValue::Str(x) => scales.fill_discrete.map_value(x),
                    crate::data::DiscreteValue::Bool(x) => scales.fill_discrete.map_value(x),
                } {
                    entry.fill = Some(fill);
                }
            }

            if has_size {
                if let Some(size) = match &value {
                    crate::data::DiscreteValue::Int(x) => scales.size_discrete.map_value(x),
                    crate::data::DiscreteValue::Str(x) => scales.size_discrete.map_value(x),
                    crate::data::DiscreteValue::Bool(x) => scales.size_discrete.map_value(x),
                } {
                    entry.size = Some(size);
                }
            } else {
                // Default size if not mapped
                entry.size = Some(5.0);
            }

            if has_shape {
                if let Some(shape) = match &value {
                    crate::data::DiscreteValue::Int(x) => scales.shape_scale.map_value(x),
                    crate::data::DiscreteValue::Str(x) => scales.shape_scale.map_value(x),
                    crate::data::DiscreteValue::Bool(x) => scales.shape_scale.map_value(x),
                } {
                    entry.shape = Some(shape);
                }
            }

            if has_alpha {
                if let Some(alpha) = match &value {
                    crate::data::DiscreteValue::Int(x) => scales.alpha_discrete.map_value(x),
                    crate::data::DiscreteValue::Str(x) => scales.alpha_discrete.map_value(x),
                    crate::data::DiscreteValue::Bool(x) => scales.alpha_discrete.map_value(x),
                } {
                    entry.alpha = Some(alpha);
                }
            }

            entries.push(entry);
        }
    }

    let merged_guide = LegendGuide {
        title: Some(title.to_string()),
        entries,
        legend_type: LegendType::Discrete,
        ..Default::default()
    };

    // Decide which guide field to use - prioritize color > fill > size > shape > alpha
    // and suppress others
    if has_color && guides.color.is_none() {
        guides.color = Some(merged_guide);
        // Suppress other aesthetics in this merge
        if has_fill {
            guides.fill = Some(LegendGuide {
                position: LegendPosition::None,
                ..Default::default()
            });
        }
        if has_size {
            guides.size = Some(LegendGuide {
                position: LegendPosition::None,
                ..Default::default()
            });
        }
        if has_shape {
            guides.shape = Some(LegendGuide {
                position: LegendPosition::None,
                ..Default::default()
            });
        }
        if has_alpha {
            guides.alpha = Some(LegendGuide {
                position: LegendPosition::None,
                ..Default::default()
            });
        }
    } else if has_fill && guides.fill.is_none() {
        guides.fill = Some(merged_guide);
        if has_size {
            guides.size = Some(LegendGuide {
                position: LegendPosition::None,
                ..Default::default()
            });
        }
        if has_shape {
            guides.shape = Some(LegendGuide {
                position: LegendPosition::None,
                ..Default::default()
            });
        }
        if has_alpha {
            guides.alpha = Some(LegendGuide {
                position: LegendPosition::None,
                ..Default::default()
            });
        }
    } else if has_size && guides.size.is_none() {
        guides.size = Some(merged_guide);
        if has_shape {
            guides.shape = Some(LegendGuide {
                position: LegendPosition::None,
                ..Default::default()
            });
        }
        if has_alpha {
            guides.alpha = Some(LegendGuide {
                position: LegendPosition::None,
                ..Default::default()
            });
        }
    } else if has_shape && guides.shape.is_none() {
        guides.shape = Some(merged_guide);
        if has_alpha {
            guides.alpha = Some(LegendGuide {
                position: LegendPosition::None,
                ..Default::default()
            });
        }
    } else if has_alpha && guides.alpha.is_none() {
        guides.alpha = Some(merged_guide);
    }
}

/// Helper structure to track mapped aesthetics
#[derive(Clone, Debug)]
struct MappedAesthetic {
    property: crate::aesthetics::AestheticProperty,
    domain: crate::aesthetics::AestheticDomain,
    column_name: String,
}

/// Generate legends automatically from scales when aesthetics are mapped
/// Merges legends when multiple aesthetics map to the same column
pub fn generate_automatic_legends(
    layers: &[Layer],
    scales: &ScaleSet,
    guides: &Guides,
    plot_mapping: &crate::aesthetics::AesMap,
) -> Guides {
    use crate::aesthetics::AestheticProperty;
    use std::collections::HashMap;

    let mut guides = guides.clone();

    // Collect all mapped aesthetics with their column names
    let mut mapped_aesthetics: Vec<MappedAesthetic> = Vec::new();

    // Helper to get column name for a property
    let get_column_name = |property: AestheticProperty| -> Option<String> {
        // First check plot-level mapping
        if let Some(title) = plot_mapping.get_label(property) {
            return Some(title);
        }
        // Then check each layer's mapping
        for layer in layers.iter() {
            if let Some(title) = layer.mapping.get_label(property) {
                return Some(title);
            }
        }
        None
    };

    // Check plot-level mappings
    for aesthetic in plot_mapping.aesthetics() {
        if let Some(property) = aesthetic.to_property() {
            if let Some(column_name) = get_column_name(property) {
                let domain = aesthetic.domain();
                mapped_aesthetics.push(MappedAesthetic {
                    property,
                    domain,
                    column_name,
                });
            }
        }
    }

    // Check layer-level aesthetic_domains
    for layer in layers {
        for (property, domain) in &layer.aesthetic_domains {
            if let Some(column_name) = get_column_name(*property) {
                // Only add if not already present from plot-level
                if !mapped_aesthetics.iter().any(|m| m.property == *property) {
                    mapped_aesthetics.push(MappedAesthetic {
                        property: *property,
                        domain: *domain,
                        column_name,
                    });
                }
            }
        }
    }

    // Group aesthetics by column name
    let mut aesthetic_groups: HashMap<String, Vec<MappedAesthetic>> = HashMap::new();
    for mapped in mapped_aesthetics {
        aesthetic_groups
            .entry(mapped.column_name.clone())
            .or_insert_with(Vec::new)
            .push(mapped);
    }

    // Process each group
    for (column_name, aesthetics) in aesthetic_groups {
        if aesthetics.is_empty() {
            continue;
        }

        // Check if all aesthetics in the group have the same domain
        let first_domain = aesthetics[0].domain;
        let same_domain = aesthetics.iter().all(|a| a.domain == first_domain);

        if !same_domain {
            // Don't merge if domains don't match - fall back to individual legends
            for aesthetic in aesthetics {
                create_individual_legend(
                    &mut guides,
                    aesthetic.property,
                    aesthetic.domain,
                    &column_name,
                    scales,
                );
            }
            continue;
        }

        // Determine if we should merge: we merge if multiple aesthetics map to same column
        if aesthetics.len() > 1 {
            // Create merged legend
            create_merged_legend(&mut guides, &aesthetics, &column_name, scales);
        } else {
            // Single aesthetic, create individual legend
            create_individual_legend(
                &mut guides,
                aesthetics[0].property,
                aesthetics[0].domain,
                &column_name,
                scales,
            );
        }
    }

    guides
}

/// Calculate the required width for legends
pub fn calculate_legend_width(
    layers: &[Layer],
    scales: &ScaleSet,
    guides: &Guides,
    plot_mapping: &crate::aesthetics::AesMap,
) -> f64 {
    let mut total_width = 0.0;
    let legend_width = 120.0; // Base legend width
    let legend_spacing = 10.0;

    // Generate automatic legends to get accurate count
    let guides = generate_automatic_legends(layers, scales, guides, plot_mapping);

    // Check if we have any legends to display
    let mut legend_count = 0;

    if let Some(ref legend) = guides.color {
        if !matches!(legend.position, LegendPosition::None) {
            legend_count += 1;
        }
    }

    if let Some(ref legend) = guides.fill {
        if !matches!(legend.position, LegendPosition::None) {
            legend_count += 1;
        }
    }

    if let Some(ref legend) = guides.shape {
        if !matches!(legend.position, LegendPosition::None) {
            legend_count += 1;
        }
    }

    if let Some(ref legend) = guides.size {
        if !matches!(legend.position, LegendPosition::None) {
            legend_count += 1;
        }
    }

    if let Some(ref legend) = guides.alpha {
        if !matches!(legend.position, LegendPosition::None) {
            legend_count += 1;
        }
    }

    if legend_count > 0 {
        // Add margin for legend placement (10px before legend, legend width, 10px after)
        total_width = legend_spacing + legend_width + legend_spacing;
    }

    total_width
}

/// Draw legends
pub fn draw_legends(
    ctx: &mut Context,
    theme: &Theme,
    layers: &[Layer],
    scales: &ScaleSet,
    guides: &Guides,
    plot_mapping: &crate::aesthetics::AesMap,
    _plot_x0: f64,
    plot_x1: f64,
    plot_y0: f64,
    _plot_y1: f64,
    width: i32,
    _height: i32,
) -> Result<(), PlotError> {
    // Generate automatic legends from scales if aesthetics are mapped
    let guides = generate_automatic_legends(layers, scales, guides, plot_mapping);

    // Collect all legends to draw
    let mut legends = Vec::new();

    // Add color legend if present
    if let Some(ref color_guide) = guides.color {
        legends.push(color_guide);
    }

    // Add fill legend if present
    if let Some(ref fill_guide) = guides.fill {
        legends.push(fill_guide);
    }

    // Add shape legend if present
    if let Some(ref shape_guide) = guides.shape {
        legends.push(shape_guide);
    }

    // Add size legend if present
    if let Some(ref size_guide) = guides.size {
        legends.push(size_guide);
    }

    // Add alpha legend if present
    if let Some(ref alpha_guide) = guides.alpha {
        legends.push(alpha_guide);
    }

    if legends.is_empty() {
        return Ok(());
    }

    // For now, render legends on the right side
    // Position legend between plot panel and right edge
    let legend_margin = 10.0;
    let legend_x = plot_x1 + legend_margin;
    let available_width = width as f64 - legend_x - legend_margin;

    // If there's not enough space, don't draw legends
    if available_width < 80.0 {
        return Ok(());
    }

    let mut legend_y = plot_y0;

    for legend in legends {
        if matches!(legend.position, LegendPosition::None) {
            continue;
        }

        // Draw legend background
        apply_fill_style(ctx, &theme.legend.background);
        let legend_width = 120.0;
        let item_height = 20.0;
        let padding = 10.0;
        let title_height = if legend.title.is_some() { 20.0 } else { 0.0 };

        let legend_height = match &legend.legend_type {
            LegendType::Discrete => {
                title_height + (legend.entries.len() as f64 * item_height) + padding * 2.0
            }
            LegendType::ColorBar { .. } => {
                title_height + 150.0 + padding * 2.0 // Fixed height for color bar
            }
        };

        ctx.rectangle(legend_x, legend_y, legend_width, legend_height);
        ctx.fill().ok();

        // Draw legend border
        apply_line_style(ctx, &theme.legend.border);
        ctx.rectangle(legend_x, legend_y, legend_width, legend_height);
        ctx.stroke().ok();

        let mut item_y = legend_y + padding;

        // Draw title if present
        if let Some(ref title) = legend.title {
            apply_font(ctx, &theme.legend.text_font);
            apply_color(ctx, &theme.legend.text_color);
            ctx.set_font_size(theme.legend.text_font.size as f64);
            ctx.move_to(legend_x + padding, item_y + 12.0);
            ctx.show_text(title).ok();
            item_y += title_height;
        }

        // Draw legend content based on type
        match &legend.legend_type {
            LegendType::Discrete => {
                // Draw discrete legend entries
                for entry in &legend.entries {
                    let symbol_x = legend_x + padding;
                    let symbol_y = item_y + item_height / 2.0;
                    let size = entry.size.unwrap_or(5.0);

                    // Determine what type of symbol to draw
                    let has_shape = entry.shape.is_some();
                    let has_fill = entry.fill.is_some();
                    let has_color = entry.color.is_some();
                    let alpha = entry.alpha.unwrap_or(1.0);

                    if has_shape {
                        // Draw shape with fill and/or color
                        let shape = entry.shape.unwrap();

                        // Draw fill if present
                        if let Some(fill) = entry.fill {
                            let Color(r, g, b, a) = fill;
                            ctx.set_source_rgba(
                                r as f64 / 255.0,
                                g as f64 / 255.0,
                                b as f64 / 255.0,
                                (a as f64 / 255.0) * alpha,
                            );
                            draw_shape(ctx, symbol_x + 10.0, symbol_y, size, shape);
                            ctx.fill().ok();
                        }

                        // Draw outline if present
                        if let Some(color) = entry.color {
                            apply_color(ctx, &color);
                            ctx.set_line_width(1.5);
                            draw_shape(ctx, symbol_x + 10.0, symbol_y, size, shape);
                            ctx.stroke().ok();
                        }
                    } else {
                        // Draw rectangle or circle with fill and/or color
                        // Use rectangle for better fill visibility
                        let symbol_size = size * 2.0;
                        let rect_x = symbol_x + 10.0 - symbol_size / 2.0;
                        let rect_y = symbol_y - symbol_size / 2.0;

                        // Draw fill if present
                        if let Some(fill) = entry.fill {
                            let Color(r, g, b, a) = fill;
                            ctx.set_source_rgba(
                                r as f64 / 255.0,
                                g as f64 / 255.0,
                                b as f64 / 255.0,
                                (a as f64 / 255.0) * alpha,
                            );
                            ctx.rectangle(rect_x, rect_y, symbol_size, symbol_size);
                            ctx.fill().ok();
                        }

                        // Draw outline/color if present
                        if let Some(color) = entry.color {
                            apply_color(ctx, &color);
                            ctx.set_line_width(1.5);
                            ctx.rectangle(rect_x, rect_y, symbol_size, symbol_size);
                            ctx.stroke().ok();
                        } else if !has_fill && has_color {
                            // Fallback: just color, draw filled circle
                            let color = entry.color.unwrap();
                            let Color(r, g, b, a) = color;
                            ctx.set_source_rgba(
                                r as f64 / 255.0,
                                g as f64 / 255.0,
                                b as f64 / 255.0,
                                (a as f64 / 255.0) * alpha,
                            );
                            ctx.arc(
                                symbol_x + 10.0,
                                symbol_y,
                                size,
                                0.0,
                                2.0 * std::f64::consts::PI,
                            );
                            ctx.fill().ok();
                        }
                    }

                    // Draw label
                    apply_font(ctx, &theme.legend.text_font);
                    apply_color(ctx, &theme.legend.text_color);
                    ctx.move_to(symbol_x + 25.0, symbol_y + 4.0);
                    ctx.show_text(&entry.label).ok();

                    item_y += item_height;
                }
            }
            LegendType::ColorBar {
                domain,
                colors,
                breaks,
                labels,
            } => {
                // Draw continuous color bar
                let bar_x = legend_x + padding + 10.0;
                let bar_width = 20.0;
                let bar_height = 120.0;
                let bar_y = item_y;

                // Draw color gradient
                let n_segments = colors.len().max(2) - 1;
                let segment_height = bar_height / n_segments as f64;

                for i in 0..n_segments {
                    let y = bar_y + i as f64 * segment_height;

                    if i < colors.len() - 1 {
                        // Create gradient pattern for this segment
                        let color1 = colors[i];
                        let color2 = colors[i + 1];

                        // For simplicity, just fill with interpolated colors
                        let steps = 10;
                        for j in 0..steps {
                            let t = j as f64 / steps as f64;
                            let r = (color1.0 as f64 * (1.0 - t) + color2.0 as f64 * t) as u8;
                            let g = (color1.1 as f64 * (1.0 - t) + color2.1 as f64 * t) as u8;
                            let b = (color1.2 as f64 * (1.0 - t) + color2.2 as f64 * t) as u8;
                            let a = (color1.3 as f64 * (1.0 - t) + color2.3 as f64 * t) as u8;

                            apply_color(ctx, &Color(r, g, b, a));
                            ctx.rectangle(
                                bar_x,
                                y + j as f64 * segment_height / steps as f64,
                                bar_width,
                                segment_height / steps as f64 + 0.5,
                            );
                            ctx.fill().ok();
                        }
                    }
                }

                // Draw border around color bar
                apply_line_style(ctx, &theme.legend.border);
                ctx.rectangle(bar_x, bar_y, bar_width, bar_height);
                ctx.stroke().ok();

                // Draw tick marks and labels using breaks from the scale
                let label_x = bar_x + bar_width + 5.0;

                apply_font(ctx, &theme.legend.text_font);
                apply_color(ctx, &theme.legend.text_color);
                ctx.set_font_size(9.0);

                for (_i, (&break_value, label)) in breaks.iter().zip(labels.iter()).enumerate() {
                    // Calculate position on the bar (inverted: high values at top)
                    let t = (break_value - domain.0) / (domain.1 - domain.0);
                    let tick_y = bar_y + bar_height - t * bar_height;

                    // Draw tick mark
                    ctx.move_to(bar_x + bar_width, tick_y);
                    ctx.line_to(bar_x + bar_width + 3.0, tick_y);
                    ctx.stroke().ok();

                    // Draw label
                    ctx.move_to(label_x + 3.0, tick_y + 3.0);
                    ctx.show_text(label).ok();
                }
            }
        }

        legend_y += legend_height + 10.0;
    }

    Ok(())
}

/// Helper to draw a shape at a position
fn draw_shape(ctx: &mut Context, x: f64, y: f64, size: f64, shape: Shape) {
    shape.draw(ctx, x, y, size);
}
