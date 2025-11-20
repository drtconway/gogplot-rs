// Plot export to various file formats

use crate::data::DataSource;
use crate::error::PlotError;
use crate::guide::Guides;
use crate::layer::Layer;
use crate::plot::render;
use crate::plot::scale_set::ScaleSet;
use crate::theme::Theme;
use cairo::{Context, PdfSurface, SvgSurface};
use std::path::Path;

/// Save the plot to a file
///
/// The output format is determined by the file extension:
/// - `.png` - PNG image
/// - `.svg` - SVG vector graphic
/// - `.pdf` - PDF document
///
/// # Arguments
///
/// * `path` - Output file path
/// * `width` - Width in pixels (for PNG) or points (for SVG/PDF)
/// * `height` - Height in pixels (for PNG) or points (for SVG/PDF)
///
/// # Examples
///
/// ```ignore
/// plot.save("output.png", 800, 600)?;
/// ```
pub fn save(
    path: impl AsRef<Path>,
    layers: &[Layer],
    scales: &ScaleSet,
    theme: &Theme,
    guides: &Guides,
    title: Option<&String>,
    data: Option<&dyn DataSource>,
    width: i32,
    height: i32,
) -> Result<(), PlotError> {
    let path = path.as_ref();
    let extension = path
        .extension()
        .and_then(|s| s.to_str())
        .ok_or_else(|| PlotError::ThemeError("Invalid file path".to_string()))?;

    match extension.to_lowercase().as_str() {
        "png" => {
            let surface = render::render(layers, scales, theme, guides, title, data, width, height)?;
            let mut file = std::fs::File::create(path)
                .map_err(|e| PlotError::ThemeError(format!("Failed to create file: {}", e)))?;
            surface
                .write_to_png(&mut file)
                .map_err(|e| PlotError::ThemeError(format!("Failed to write PNG: {}", e)))?;
        }
        "svg" => {
            let surface =
                SvgSurface::new(width as f64, height as f64, Some(path)).map_err(|e| {
                    PlotError::ThemeError(format!("Failed to create SVG surface: {}", e))
                })?;

            let mut ctx = Context::new(&surface)
                .map_err(|e| PlotError::ThemeError(format!("Failed to create context: {}", e)))?;
            render::render_with_context(
                &mut ctx, layers, scales, theme, guides, title, data, width, height,
            )?;
            surface.finish();
        }
        "pdf" => {
            let surface = PdfSurface::new(width as f64, height as f64, path).map_err(|e| {
                PlotError::ThemeError(format!("Failed to create PDF surface: {}", e))
            })?;

            let mut ctx = Context::new(&surface)
                .map_err(|e| PlotError::ThemeError(format!("Failed to create context: {}", e)))?;
            render::render_with_context(
                &mut ctx, layers, scales, theme, guides, title, data, width, height,
            )?;
            surface.finish();
        }
        _ => {
            return Err(PlotError::ThemeError(format!(
                "Unsupported file format: {}",
                extension
            )));
        }
    }

    Ok(())
}
