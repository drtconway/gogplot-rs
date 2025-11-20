// Axis and grid rendering

use crate::error::PlotError;
use crate::guide::{AxisGuide, AxisType, XAxisPosition, YAxisPosition};
use crate::scale::ContinuousScale;
use crate::theme::Theme;
use cairo::Context;

use super::cairo_helpers::{apply_line_style, apply_text_theme};

/// Draw grid lines in the panel area
pub fn draw_grid_lines(
    ctx: &mut Context,
    theme: &Theme,
    x_scale: Option<&Box<dyn ContinuousScale>>,
    y_scale: Option<&Box<dyn ContinuousScale>>,
    plot_x0: f64,
    plot_x1: f64,
    plot_y0: f64,
    plot_y1: f64,
) -> Result<(), PlotError> {
    // Draw minor grid lines first (if present) so major grid lines are on top
    if let Some(ref grid_minor) = theme.panel.grid_minor {
        apply_line_style(ctx, grid_minor);

        // X-axis minor grid lines
        if let Some(x_scale) = x_scale {
            let breaks = x_scale.breaks();
            // Generate minor breaks between major breaks
            for i in 0..breaks.len().saturating_sub(1) {
                let mid = (breaks[i] + breaks[i + 1]) / 2.0;
                if let Some(normalized) = x_scale.map_value(mid) {
                    let x = plot_x0 + normalized * (plot_x1 - plot_x0);
                    ctx.move_to(x, plot_y0);
                    ctx.line_to(x, plot_y1);
                }
            }
        }

        // Y-axis minor grid lines
        if let Some(y_scale) = y_scale {
            let breaks = y_scale.breaks();
            // Generate minor breaks between major breaks
            for i in 0..breaks.len().saturating_sub(1) {
                let mid = (breaks[i] + breaks[i + 1]) / 2.0;
                if let Some(normalized) = y_scale.map_value(mid) {
                    let y = plot_y1 - normalized * (plot_y1 - plot_y0);
                    ctx.move_to(plot_x0, y);
                    ctx.line_to(plot_x1, y);
                }
            }
        }

        ctx.stroke().ok();
    }

    // Draw major grid lines
    if let Some(ref grid_major) = theme.panel.grid_major {
        apply_line_style(ctx, grid_major);

        // X-axis major grid lines (vertical lines at tick positions)
        if let Some(x_scale) = x_scale {
            for &break_value in x_scale.breaks() {
                if let Some(normalized) = x_scale.map_value(break_value) {
                    let x = plot_x0 + normalized * (plot_x1 - plot_x0);
                    ctx.move_to(x, plot_y0);
                    ctx.line_to(x, plot_y1);
                }
            }
        }

        // Y-axis major grid lines (horizontal lines at tick positions)
        if let Some(y_scale) = y_scale {
            for &break_value in y_scale.breaks() {
                if let Some(normalized) = y_scale.map_value(break_value) {
                    let y = plot_y1 - normalized * (plot_y1 - plot_y0);
                    ctx.move_to(plot_x0, y);
                    ctx.line_to(plot_x1, y);
                }
            }
        }

        ctx.stroke().ok();
    }

    Ok(())
}

/// Draw axes with tick marks and labels
pub fn draw_axes(
    ctx: &mut Context,
    theme: &Theme,
    x_axis: Option<&AxisGuide>,
    y_axis: Option<&AxisGuide>,
    x_scale: Option<&Box<dyn ContinuousScale>>,
    y_scale: Option<&Box<dyn ContinuousScale>>,
    title: Option<&String>,
    x0: f64,
    x1: f64,
    y0: f64,
    y1: f64,
) -> Result<(), PlotError> {
    // Draw axis lines based on position

    // X axis line
    let x_position = x_axis
        .and_then(|guide| match &guide.position {
            AxisType::X(pos) => Some(pos.clone()),
            _ => None,
        })
        .unwrap_or(XAxisPosition::Bottom);

    if let Some(ref line_style) = theme.axis_x.line.line {
        apply_line_style(ctx, line_style);
        match x_position {
            XAxisPosition::Bottom => {
                ctx.move_to(x0, y1);
                ctx.line_to(x1, y1);
            }
            XAxisPosition::Top => {
                ctx.move_to(x0, y0);
                ctx.line_to(x1, y0);
            }
        }
        ctx.stroke().ok();
    }

    // Y axis line
    let y_position = y_axis
        .and_then(|guide| match &guide.position {
            AxisType::Y(pos) => Some(pos.clone()),
            _ => None,
        })
        .unwrap_or(YAxisPosition::Left);

    if let Some(ref line_style) = theme.axis_y.line.line {
        apply_line_style(ctx, line_style);
        match y_position {
            YAxisPosition::Left => {
                ctx.move_to(x0, y0);
                ctx.line_to(x0, y1);
            }
            YAxisPosition::Right => {
                ctx.move_to(x1, y0);
                ctx.line_to(x1, y1);
            }
        }
        ctx.stroke().ok();
    }

    // Draw X axis ticks and labels
    if let Some(x_scale) = x_scale {
        let breaks = x_scale.breaks();
        let labels = x_scale.labels();
        let tick_length = theme.axis_x.line.tick_length as f64;

        apply_text_theme(ctx, &theme.axis_x.text.text);

        for (value, label) in breaks.iter().zip(labels.iter()) {
            if let Some(normalized) = x_scale.map_value(*value) {
                let x_pos = x0 + normalized * (x1 - x0);

                // Draw tick mark
                if let Some(ref tick_style) = theme.axis_x.line.ticks {
                    apply_line_style(ctx, tick_style);
                    match x_position {
                        XAxisPosition::Bottom => {
                            ctx.move_to(x_pos, y1);
                            ctx.line_to(x_pos, y1 + tick_length);
                        }
                        XAxisPosition::Top => {
                            ctx.move_to(x_pos, y0);
                            ctx.line_to(x_pos, y0 - tick_length);
                        }
                    }
                    ctx.stroke().ok();
                }

                // Draw label
                apply_text_theme(ctx, &theme.axis_x.text.text);
                let extents = ctx.text_extents(label).ok();
                if let Some(ext) = extents {
                    let margin = theme.axis_x.text.text.margin.top as f64;
                    match x_position {
                        XAxisPosition::Bottom => {
                            ctx.move_to(
                                x_pos - ext.width() / 2.0,
                                y1 + tick_length + margin + ext.height(),
                            );
                        }
                        XAxisPosition::Top => {
                            ctx.move_to(x_pos - ext.width() / 2.0, y0 - tick_length - margin);
                        }
                    }
                    ctx.show_text(label).ok();
                }
            }
        }
    }

    // Draw Y axis ticks and labels
    if let Some(y_scale) = y_scale {
        let breaks = y_scale.breaks();
        let labels = y_scale.labels();
        let tick_length = theme.axis_y.line.tick_length as f64;

        apply_text_theme(ctx, &theme.axis_y.text.text);

        for (value, label) in breaks.iter().zip(labels.iter()) {
            if let Some(normalized) = y_scale.map_value(*value) {
                let y_pos = y1 - normalized * (y1 - y0); // Y is inverted

                // Draw tick mark
                if let Some(ref tick_style) = theme.axis_y.line.ticks {
                    apply_line_style(ctx, tick_style);
                    match y_position {
                        YAxisPosition::Left => {
                            ctx.move_to(x0 - tick_length, y_pos);
                            ctx.line_to(x0, y_pos);
                        }
                        YAxisPosition::Right => {
                            ctx.move_to(x1, y_pos);
                            ctx.line_to(x1 + tick_length, y_pos);
                        }
                    }
                    ctx.stroke().ok();
                }

                // Draw label
                apply_text_theme(ctx, &theme.axis_y.text.text);
                let extents = ctx.text_extents(label).ok();
                if let Some(ext) = extents {
                    let margin = theme.axis_y.text.text.margin.right as f64;
                    match y_position {
                        YAxisPosition::Left => {
                            ctx.move_to(
                                x0 - tick_length - margin - ext.width(),
                                y_pos + ext.height() / 2.0,
                            );
                        }
                        YAxisPosition::Right => {
                            ctx.move_to(x1 + tick_length + margin, y_pos + ext.height() / 2.0);
                        }
                    }
                    ctx.show_text(label).ok();
                }
            }
        }
    }

    // Draw X axis title
    if let Some(x_axis) = x_axis {
        if let Some(x_label) = &x_axis.title {
            let x_position = match &x_axis.position {
                AxisType::X(pos) => pos.clone(),
                _ => XAxisPosition::Bottom,
            };

            apply_text_theme(ctx, &theme.axis_x.text.title);
            let extents = ctx.text_extents(x_label).ok();
            if let Some(ext) = extents {
                let x_center = (x0 + x1) / 2.0;
                let tick_length = theme.axis_x.line.tick_length as f64;
                let label_margin = theme.axis_x.text.text.margin.top as f64;
                let typical_label_height = theme.axis_x.text.text.font.size as f64;
                let title_margin = theme.axis_x.text.title.margin.top as f64;

                match x_position {
                    XAxisPosition::Bottom => {
                        // Position below: axis line + ticks + tick label margin + typical label height + title margin
                        let y_offset = y1
                            + tick_length
                            + label_margin
                            + typical_label_height
                            + title_margin
                            + ext.height();
                        ctx.move_to(x_center - ext.width() / 2.0, y_offset);
                    }
                    XAxisPosition::Top => {
                        // Position above: axis line + ticks + tick label margin + title margin
                        let y_offset =
                            y0 - tick_length - label_margin - typical_label_height - title_margin;
                        ctx.move_to(x_center - ext.width() / 2.0, y_offset);
                    }
                }
                ctx.show_text(x_label).ok();
            }
        }
    }

    // Draw Y axis title (rotated)
    if let Some(y_axis) = y_axis {
        if let Some(y_label) = &y_axis.title {
            let y_position = match &y_axis.position {
                AxisType::Y(pos) => pos.clone(),
                _ => YAxisPosition::Left,
            };

            ctx.save().ok();
            apply_text_theme(ctx, &theme.axis_y.text.title);
            let y_center = (y0 + y1) / 2.0;
            let extents = ctx.text_extents(y_label).ok();
            if let Some(ext) = extents {
                let tick_length = theme.axis_y.line.tick_length as f64;
                let label_margin = theme.axis_y.text.text.margin.right as f64;
                // Estimate max label width (rough approximation based on font size * typical digits)
                let typical_label_width = theme.axis_y.text.text.font.size as f64 * 2.5;
                let title_margin = theme.axis_y.text.title.margin.right as f64;
                let title_height = ext.height();

                match y_position {
                    YAxisPosition::Left => {
                        // Position to left of: axis line + ticks + tick label margin + typical max label width + title margin
                        let x_offset = x0
                            - tick_length
                            - label_margin
                            - typical_label_width
                            - title_margin
                            - title_height;
                        ctx.move_to(x_offset, y_center + ext.width() / 2.0);
                        ctx.rotate(-std::f64::consts::PI / 2.0);
                    }
                    YAxisPosition::Right => {
                        // Position to right of: axis line + ticks + tick label margin + typical max label width + title margin
                        let x_offset = x1
                            + tick_length
                            + label_margin
                            + typical_label_width
                            + title_margin
                            + title_height;
                        ctx.move_to(x_offset, y_center - ext.width() / 2.0);
                        ctx.rotate(std::f64::consts::PI / 2.0);
                    }
                }
                ctx.show_text(y_label).ok();
            }
            ctx.restore().ok();
        }
    }

    // Draw plot title
    if let Some(title) = title {
        apply_text_theme(ctx, &theme.plot_title.text);
        let extents = ctx.text_extents(title).ok();
        if let Some(ext) = extents {
            let x_center = (x0 + x1) / 2.0;
            let y_offset = theme.plot_title.text.margin.top as f64;
            ctx.move_to(x_center - ext.width() / 2.0, y_offset + ext.height());
            ctx.show_text(title).ok();
        }
    }

    Ok(())
}
