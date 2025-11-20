// Rendering module for plot output

pub mod axes;
pub mod cairo_helpers;
pub mod legend;

use crate::data::DataSource;
use crate::error::PlotError;
use crate::geom::RenderContext;
use crate::layer::Layer;
use crate::plot::scale_set::ScaleSet;
use crate::theme::Theme;
use cairo::{Context, Format, ImageSurface};

use self::axes::{draw_axes, draw_grid_lines};
use self::cairo_helpers::{apply_fill_style, apply_line_style};
use self::legend::{calculate_legend_width, draw_legends};

/// Render the plot to an ImageSurface
///
/// # Arguments
///
/// * `width` - Width of the plot in pixels
/// * `height` - Height of the plot in pixels
///
/// # Returns
///
/// An ImageSurface containing the rendered plot, or an error
pub fn render(
    layers: &[Layer],
    scales: &ScaleSet,
    theme: &Theme,
    guides: &crate::guide::Guides,
    title: Option<&String>,
    data: Option<&dyn DataSource>,
    width: i32,
    height: i32,
) -> Result<ImageSurface, PlotError> {
    // Create the surface
    let surface = ImageSurface::create(Format::ARgb32, width, height)
        .map_err(|e| PlotError::ThemeError(format!("Failed to create surface: {}", e)))?;

    let mut ctx = Context::new(&surface)
        .map_err(|e| PlotError::ThemeError(format!("Failed to create context: {}", e)))?;

    // Use the common rendering code
    render_with_context(
        &mut ctx, layers, scales, theme, guides, title, data, width, height,
    )?;

    Ok(surface)
}

/// Helper method to render using an existing Cairo context
pub fn render_with_context(
    ctx: &mut Context,
    layers: &[Layer],
    scales: &ScaleSet,
    theme: &Theme,
    guides: &crate::guide::Guides,
    title: Option<&String>,
    data: Option<&dyn DataSource>,
    width: i32,
    height: i32,
) -> Result<(), PlotError> {
    // Fill background
    apply_fill_style(ctx, &theme.background.fill);
    ctx.paint()
        .map_err(|e| PlotError::ThemeError(format!("Failed to paint background: {}", e)))?;

    // Calculate required legend width and adjust right margin
    let legend_width = calculate_legend_width(layers, scales, guides);

    // Determine axis positions
    use crate::guide::{AxisType, XAxisPosition, YAxisPosition};

    let x_position = guides
        .x_axis
        .as_ref()
        .and_then(|guide| match &guide.position {
            AxisType::X(pos) => Some(pos.clone()),
            _ => None,
        })
        .unwrap_or(XAxisPosition::Bottom);

    let y_position = guides
        .y_axis
        .as_ref()
        .and_then(|guide| match &guide.position {
            AxisType::Y(pos) => Some(pos.clone()),
            _ => None,
        })
        .unwrap_or(YAxisPosition::Left);

    // Get base margins from theme
    let theme_margin_left = theme.plot_margin.left as f64;
    let theme_margin_right = theme.plot_margin.right as f64;
    let theme_margin_top = theme.plot_margin.top as f64;
    let theme_margin_bottom = theme.plot_margin.bottom as f64;

    // Adjust margins based on axis positions
    // When axis moves to opposite side, use theme's opposite margin
    let margin_left = match y_position {
        YAxisPosition::Left => theme_margin_left,
        YAxisPosition::Right => theme_margin_right,
    };

    let mut margin_right = match y_position {
        YAxisPosition::Left => theme_margin_right,
        YAxisPosition::Right => theme_margin_left,
    };

    let margin_top = match x_position {
        XAxisPosition::Top => theme_margin_bottom,
        XAxisPosition::Bottom => theme_margin_top,
    };

    let margin_bottom = match x_position {
        XAxisPosition::Bottom => theme_margin_bottom,
        XAxisPosition::Top => theme_margin_top,
    };

    // Increase right margin if legends are present
    if legend_width > 0.0 {
        margin_right = f64::max(margin_right, legend_width);
    }

    let plot_x0 = margin_left;
    let plot_x1 = width as f64 - margin_right;
    let plot_y0 = margin_top;
    let plot_y1 = height as f64 - margin_bottom;

    // Draw panel background
    if let Some(ref panel_bg) = theme.panel.background {
        apply_fill_style(ctx, panel_bg);
        ctx.rectangle(plot_x0, plot_y0, plot_x1 - plot_x0, plot_y1 - plot_y0);
        ctx.fill().ok();
    }

    // Draw grid lines (before border so border is on top)
    draw_grid_lines(
        ctx,
        theme,
        scales.x.as_ref(),
        scales.y.as_ref(),
        plot_x0,
        plot_x1,
        plot_y0,
        plot_y1,
    )?;

    // Draw panel border
    if let Some(ref border) = theme.panel.border {
        apply_line_style(ctx, border);
        ctx.rectangle(plot_x0, plot_y0, plot_x1 - plot_x0, plot_y1 - plot_y0);
        ctx.stroke().ok();
    }

    // Draw axes before rendering layers
    draw_axes(
        ctx,
        theme,
        guides.x_axis.as_ref(),
        guides.y_axis.as_ref(),
        scales.x.as_ref(),
        scales.y.as_ref(),
        title,
        plot_x0,
        plot_x1,
        plot_y0,
        plot_y1,
    )?;

    // Render each layer
    for layer in layers {
        // Use computed data if available, otherwise use original data
        let layer_data: &dyn DataSource = if let Some(ref computed) = layer.computed_data {
            computed as &dyn DataSource
        } else {
            match &layer.data {
                Some(d) => d.as_ref(),
                None => match data {
                    Some(d) => d,
                    None => return Err(PlotError::MissingAesthetic("No data source".to_string())),
                },
            }
        };

        // Use computed mapping if available, otherwise use original mapping
        let mapping = layer.computed_mapping.as_ref().unwrap_or(&layer.mapping);

        let mut render_ctx = RenderContext::new(
            ctx,
            layer_data,
            mapping,
            scales,
            (plot_x0, plot_x1),
            (plot_y1, plot_y0),
        );

        layer.geom.render(&mut render_ctx)?;
    }

    // Draw legends
    draw_legends(
        ctx, theme, layers, scales, guides, plot_x0, plot_x1, plot_y0, plot_y1, width, height,
    )?;

    Ok(())
}
