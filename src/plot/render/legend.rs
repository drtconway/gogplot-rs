// Legend rendering

use crate::aesthetics::{AesValue, Aesthetic};
use crate::error::PlotError;
use crate::guide::{Guides, LegendEntry, LegendGuide, LegendPosition, LegendType};
use crate::layer::Layer;
use crate::plot::scale_set::ScaleSet;
use crate::theme::{Color, Theme};
use crate::visuals::Shape;
use cairo::Context;

use super::cairo_helpers::{apply_color, apply_fill_style, apply_font, apply_line_style};

/// Generate legends automatically from scales when aesthetics are mapped
pub fn generate_automatic_legends(layers: &[Layer], scales: &ScaleSet, guides: &Guides) -> Guides {
    let mut guides = guides.clone();

    // Check if any layer maps Color aesthetic to a column
    let has_color_mapping = layers.iter().any(|layer| {
        let mapping = layer.computed_mapping.as_ref().unwrap_or(&layer.mapping);
        matches!(
            mapping.get(&Aesthetic::Color),
            Some(AesValue::Column(_) | AesValue::CategoricalColumn(_))
        )
    });

    // Check if any layer maps Fill aesthetic to a column
    let has_fill_mapping = layers.iter().any(|layer| {
        let mapping = layer.computed_mapping.as_ref().unwrap_or(&layer.mapping);
        matches!(
            mapping.get(&Aesthetic::Fill),
            Some(AesValue::Column(_) | AesValue::CategoricalColumn(_))
        )
    });

    // Check if any layer maps Shape aesthetic to a column
    let has_shape_mapping = layers.iter().any(|layer| {
        let mapping = layer.computed_mapping.as_ref().unwrap_or(&layer.mapping);
        matches!(
            mapping.get(&Aesthetic::Shape),
            Some(AesValue::Column(_) | AesValue::CategoricalColumn(_))
        )
    });

    // Generate color legend if Color is mapped and we have a color scale
    if has_color_mapping && scales.color.is_some() && guides.color.is_none() {
        if let Some(ref color_scale) = scales.color {
            let mut legend = LegendGuide::new();
            legend.position = LegendPosition::Right;

            // Get the column name for the title
            for layer in layers {
                let mapping = layer.computed_mapping.as_ref().unwrap_or(&layer.mapping);
                if let Some(col_name) = mapping.get(&Aesthetic::Color).and_then(|v| v.as_column_name()) {
                    legend.title = Some(col_name.to_string());
                    break;
                }
            }

            if color_scale.is_continuous() {
                // Create a continuous color bar
                if let Some(domain) = color_scale.get_continuous_domain() {
                    // Sample colors across the domain
                    let n_samples = 50;
                    let mut colors = Vec::new();
                    for i in 0..=n_samples {
                        let t = i as f64 / n_samples as f64;
                        let value = domain.0 + t * (domain.1 - domain.0);
                        if let Some(color) = color_scale.map_continuous_to_color(value) {
                            colors.push(color);
                        }
                    }

                    legend.legend_type = LegendType::ColorBar { domain, colors };
                    guides.color = Some(legend);
                }
            } else {
                // Create discrete legend entries
                let breaks = color_scale.legend_breaks();
                if !breaks.is_empty() {
                    for category in breaks {
                        if let Some(color) = color_scale.map_discrete_to_color(&category) {
                            legend.entries.push(LegendEntry {
                                label: category.clone(),
                                color: Some(color),
                                shape: Some(Shape::Circle),
                                size: Some(5.0),
                            });
                        }
                    }
                    guides.color = Some(legend);
                }
            }
        }
    }

    // Generate fill legend if Fill is mapped and we have a fill scale
    if has_fill_mapping && scales.fill.is_some() && guides.fill.is_none() {
        if let Some(ref fill_scale) = scales.fill {
            let mut legend = LegendGuide::new();
            legend.position = LegendPosition::Right;

            // Get the column name for the title
            for layer in layers {
                let mapping = layer.computed_mapping.as_ref().unwrap_or(&layer.mapping);
                if let Some(col_name) = mapping.get(&Aesthetic::Fill).and_then(|v| v.as_column_name()) {
                    legend.title = Some(col_name.to_string());
                    break;
                }
            }

            if fill_scale.is_continuous() {
                // Create a continuous color bar
                if let Some(domain) = fill_scale.get_continuous_domain() {
                    // Sample colors across the domain
                    let n_samples = 50;
                    let mut colors = Vec::new();
                    for i in 0..=n_samples {
                        let t = i as f64 / n_samples as f64;
                        let value = domain.0 + t * (domain.1 - domain.0);
                        if let Some(color) = fill_scale.map_continuous_to_color(value) {
                            colors.push(color);
                        }
                    }

                    legend.legend_type = LegendType::ColorBar { domain, colors };
                    guides.fill = Some(legend);
                }
            } else {
                // Create discrete legend entries
                let breaks = fill_scale.legend_breaks();
                if !breaks.is_empty() {
                    for category in breaks {
                        if let Some(color) = fill_scale.map_discrete_to_color(&category) {
                            legend.entries.push(LegendEntry {
                                label: category.clone(),
                                color: Some(color),
                                shape: Some(Shape::Square), // Use square for fill legend
                                size: Some(5.0),
                            });
                        }
                    }
                    guides.fill = Some(legend);
                } 
            }
        }
    }

    // Generate shape legend if Shape is mapped and we have a shape scale
    if has_shape_mapping && scales.shape.is_some() && guides.shape.is_none() {
        if let Some(ref shape_scale) = scales.shape {
            let breaks = shape_scale.legend_breaks();
            if !breaks.is_empty() {
                let mut legend = LegendGuide::new();
                legend.position = LegendPosition::Right;

                // Get the column name for the title
                for layer in layers {
                    let mapping = layer.computed_mapping.as_ref().unwrap_or(&layer.mapping);
                    if let Some(col_name) = mapping.get(&Aesthetic::Shape).and_then(|v| v.as_column_name()) {
                        legend.title = Some(col_name.to_string());
                        break;
                    }
                }

                // Generate entries from breaks
                for category in breaks {
                    if let Some(shape) = shape_scale.map_to_shape(&category) {
                        legend.entries.push(LegendEntry {
                            label: category.clone(),
                            color: Some(Color(100, 100, 100, 255)), // Default gray
                            shape: Some(shape),
                            size: Some(5.0),
                        });
                    }
                }

                guides.shape = Some(legend);
            }
        }
    }

    guides
}

/// Calculate the required width for legends
pub fn calculate_legend_width(layers: &[Layer], scales: &ScaleSet, guides: &Guides) -> f64 {
    let mut total_width = 0.0;
    let legend_width = 120.0; // Base legend width
    let legend_spacing = 10.0;

    // Generate automatic legends to get accurate count
    let guides = generate_automatic_legends(layers, scales, guides);

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
    _plot_x0: f64,
    plot_x1: f64,
    plot_y0: f64,
    _plot_y1: f64,
    width: i32,
    _height: i32,
) -> Result<(), PlotError> {
    // Generate automatic legends from scales if aesthetics are mapped
    let guides = generate_automatic_legends(layers, scales, guides);

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
            LegendType::ColorBar { domain, colors } => {
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

                // Draw tick marks and labels with 5 evenly spaced breaks
                let label_x = bar_x + bar_width + 5.0;
                let num_breaks = 5;

                apply_font(ctx, &theme.legend.text_font);
                apply_color(ctx, &theme.legend.text_color);
                ctx.set_font_size(9.0);

                for i in 0..num_breaks {
                    let t = i as f64 / (num_breaks - 1) as f64;
                    let value = domain.0 + t * (domain.1 - domain.0);
                    // Calculate position on the bar
                    let t = (value - domain.0) / (domain.1 - domain.0);
                    let tick_y = bar_y + bar_height - t * bar_height;

                    // Draw tick mark
                    ctx.move_to(bar_x + bar_width, tick_y);
                    ctx.line_to(bar_x + bar_width + 3.0, tick_y);
                    ctx.stroke().ok();

                    // Draw label - format intelligently
                    ctx.move_to(label_x + 3.0, tick_y + 3.0);
                    let label = if value.abs() < 1e-10 {
                        // Treat very small values as zero
                        "0".to_string()
                    } else if (value - value.round()).abs() < 0.01 {
                        // Value is close to an integer - show it as an integer
                        format!("{}", value.round() as i64)
                    } else if value.abs() < 0.01 || value.abs() > 10000.0 {
                        format!("{:.2e}", value)
                    } else {
                        format!("{:.2}", value)
                    };
                    ctx.show_text(&label).ok();
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
