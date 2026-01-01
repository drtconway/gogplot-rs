// Legend rendering

use crate::error::PlotError;
use crate::guide::{Guides, LegendEntry, LegendGuide, LegendPosition, LegendType};
use crate::layer::Layer;
use crate::scale::ScaleSet;
use crate::theme::{Color, Theme};
use crate::visuals::Shape;
use cairo::Context;

use super::cairo_helpers::{apply_color, apply_fill_style, apply_font, apply_line_style};

/// Generate legends automatically from scales when aesthetics are mapped
pub fn generate_automatic_legends(
    layers: &[Layer],
    scales: &ScaleSet,
    guides: &Guides,
    plot_mapping: &crate::aesthetics::AesMap,
) -> Guides {
    use crate::aesthetics::{AestheticProperty, AestheticDomain};
    use crate::scale::traits::{DiscreteDomainScale, ContinuousDomainScale, ColorRangeScale, ContinuousRangeScale};
    
    let mut guides = guides.clone();

    // Collect which aesthetics are mapped across all layers
    let mut has_color = false;
    let mut has_fill = false;
    let mut has_size = false;
    let mut has_shape = false;
    let mut has_alpha = false;
    
    let mut color_domain: Option<AestheticDomain> = None;
    let mut fill_domain: Option<AestheticDomain> = None;
    let mut size_domain: Option<AestheticDomain> = None;
    let mut alpha_domain: Option<AestheticDomain> = None;
    
    // Check plot-level mappings
    for aesthetic in plot_mapping.aesthetics() {
        if let Some(property) = aesthetic.to_property() {
            match property {
                AestheticProperty::Color => {
                    has_color = true;
                    color_domain = Some(aesthetic.domain());
                }
                AestheticProperty::Fill => {
                    has_fill = true;
                    fill_domain = Some(aesthetic.domain());
                }
                AestheticProperty::Size => {
                    has_size = true;
                    size_domain = Some(aesthetic.domain());
                }
                AestheticProperty::Shape => has_shape = true,
                AestheticProperty::Alpha => {
                    has_alpha = true;
                    alpha_domain = Some(aesthetic.domain());
                }
                _ => {}
            }
        }
    }
    
    // Check layer-level aesthetic_domains
    for layer in layers {
        for (property, domain) in &layer.aesthetic_domains {
            match property {
                AestheticProperty::Color => {
                    has_color = true;
                    color_domain = Some(*domain);
                }
                AestheticProperty::Fill => {
                    has_fill = true;
                    fill_domain = Some(*domain);
                }
                AestheticProperty::Size => {
                    has_size = true;
                    size_domain = Some(*domain);
                }
                AestheticProperty::Shape => has_shape = true,
                AestheticProperty::Alpha => {
                    has_alpha = true;
                    alpha_domain = Some(*domain);
                }
                _ => {}
            }
        }
    }
    
    // Get default legend titles from aesthetic mappings
    // Check plot-level mapping first, then layer mappings
    let get_title = |property: AestheticProperty, default: &str| -> String {
        // First check plot-level mapping
        if let Some(title) = plot_mapping.get_label(property) {
            return title;
        }
        // Then check each layer's mapping
        for layer in layers.iter() {
            if let Some(mapping) = &layer.mapping {
                if let Some(title) = mapping.get_label(property) {
                    return title;
                }
            }
        }
        default.to_string()
    };
    
    let color_title = get_title(AestheticProperty::Color, "Color");
    let fill_title = get_title(AestheticProperty::Fill, "Fill");
    let size_title = get_title(AestheticProperty::Size, "Size");
    let shape_title = get_title(AestheticProperty::Shape, "Shape");
    let alpha_title = get_title(AestheticProperty::Alpha, "Alpha");
    
    // Generate color legend if not already specified
    if has_color && guides.color.is_none() {
        let legend = match color_domain {
            Some(AestheticDomain::Discrete) => {
                let mut entries = Vec::new();
                let categories = scales.color_discrete.categories();
                
                for i in 0..categories.len() {
                    if let Some(value) = categories.get_at(i) {
                        let label = match &value {
                            crate::data::DiscreteValue::Int(x) => x.to_string(),
                            crate::data::DiscreteValue::Str(x) => x.clone(),
                            crate::data::DiscreteValue::Bool(x) => x.to_string(),
                        };
                        
                        let color = match &value {
                            crate::data::DiscreteValue::Int(x) => scales.color_discrete.map_value(x),
                            crate::data::DiscreteValue::Str(x) => scales.color_discrete.map_value(x),
                            crate::data::DiscreteValue::Bool(x) => scales.color_discrete.map_value(x),
                        };
                        
                        if let Some(color) = color {
                            entries.push(
                                LegendEntry::new(label)
                                    .color(color)
                                    .size(5.0)
                            );
                        }
                    }
                }
                
                LegendGuide {
                    title: Some(color_title),
                    entries,
                    legend_type: LegendType::Discrete,
                    ..Default::default()
                }
            }
            Some(AestheticDomain::Continuous) => {
                if let Some(domain) = scales.color_continuous.domain() {
                    // Get the gradient colors from the scale
                    let colors = vec![
                        scales.color_continuous.map_value(&domain.0).unwrap_or(crate::theme::color::BLACK),
                        scales.color_continuous.map_value(&domain.1).unwrap_or(crate::theme::color::WHITE),
                    ];
                    
                    LegendGuide {
                        title: Some(color_title),
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
            None => LegendGuide::default(),
        };
        guides.color = Some(legend);
    }
    
    // Generate size legend if not already specified
    if has_size && guides.size.is_none() {
        let legend = match size_domain {
            Some(AestheticDomain::Discrete) => {
                let mut entries = Vec::new();
                let categories = scales.size_discrete.categories();
                
                for i in 0..categories.len() {
                    if let Some(value) = categories.get_at(i) {
                        let label = match &value {
                            crate::data::DiscreteValue::Int(x) => x.to_string(),
                            crate::data::DiscreteValue::Str(x) => x.clone(),
                            crate::data::DiscreteValue::Bool(x) => x.to_string(),
                        };
                        
                        let size = match &value {
                            crate::data::DiscreteValue::Int(x) => scales.size_discrete.map_value(x),
                            crate::data::DiscreteValue::Str(x) => scales.size_discrete.map_value(x),
                            crate::data::DiscreteValue::Bool(x) => scales.size_discrete.map_value(x),
                        };
                        
                        if let Some(size) = size {
                            entries.push(
                                LegendEntry::new(label)
                                    .color(crate::theme::color::BLACK)
                                    .size(size)
                            );
                        }
                    }
                }
                
                LegendGuide {
                    title: Some(size_title),
                    entries,
                    legend_type: LegendType::Discrete,
                    ..Default::default()
                }
            }
            Some(AestheticDomain::Continuous) => {
                // For continuous size, show a few representative sizes
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
                                    .size(size)
                            );
                        }
                    }
                    
                    LegendGuide {
                        title: Some(size_title),
                        entries,
                        legend_type: LegendType::Discrete, // Even for continuous, show discrete samples
                        ..Default::default()
                    }
                } else {
                    LegendGuide::default()
                }
            }
            None => LegendGuide::default(),
        };
        guides.size = Some(legend);
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

                    // Draw symbol
                    if let Some(color) = entry.color {
                        apply_color(ctx, &color);
                        let size = entry.size.unwrap_or(5.0);

                        if let Some(shape) = entry.shape {
                            draw_shape(ctx, symbol_x + 10.0, symbol_y, size, shape);
                        } else {
                            // Default to circle
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
            LegendType::ColorBar { domain, colors, breaks, labels } => {
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

                for (i, (&break_value, label)) in breaks.iter().zip(labels.iter()).enumerate() {
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
