// Cairo rendering helpers

use crate::theme::{Color, Font, FontStyle, FontWeight, LineElement, TextElement};
use crate::visuals::LineStyle;
use cairo::Context;

/// Apply color to Cairo context
pub fn apply_color(ctx: &mut Context, color: &Color) {
    ctx.set_source_rgba(
        color.0 as f64 / 255.0,
        color.1 as f64 / 255.0,
        color.2 as f64 / 255.0,
        color.3 as f64 / 255.0,
    );
}

/// Apply fill style to Cairo context
pub fn apply_fill_style(ctx: &mut Context, fill: &crate::theme::FillStyle) {
    let alpha = (fill.color.3 as f64 / 255.0) * fill.opacity as f64;
    ctx.set_source_rgba(
        fill.color.0 as f64 / 255.0,
        fill.color.1 as f64 / 255.0,
        fill.color.2 as f64 / 255.0,
        alpha,
    );
}

/// Apply font to Cairo context
pub fn apply_font(ctx: &mut Context, font: &Font) {
    let slant = match font.style {
        FontStyle::Normal => cairo::FontSlant::Normal,
        FontStyle::Italic => cairo::FontSlant::Italic,
        FontStyle::Oblique => cairo::FontSlant::Oblique,
    };
    let weight = match font.weight {
        FontWeight::Normal => cairo::FontWeight::Normal,
        FontWeight::Bold => cairo::FontWeight::Bold,
        FontWeight::Light => cairo::FontWeight::Normal, // Cairo doesn't have Light
    };
    ctx.select_font_face(&font.family, slant, weight);
    ctx.set_font_size(font.size as f64);
}

/// Apply line element to Cairo context
pub fn apply_line_element(ctx: &mut Context, line: &LineElement) {
    let color = line.color.unwrap_or(crate::theme::color::BLACK);
    let size = line.size.unwrap_or(1.0);
    let alpha = line.alpha.unwrap_or(1.0);
    
    ctx.set_source_rgba(
        color.0 as f64 / 255.0,
        color.1 as f64 / 255.0,
        color.2 as f64 / 255.0,
        (color.3 as f64 / 255.0) * alpha,
    );
    ctx.set_line_width(size);
    
    // Apply line style
    match line.linestyle.as_ref().unwrap_or(&LineStyle::Solid) {
        LineStyle::Solid => ctx.set_dash(&[], 0.0),
        LineStyle::Custom(pattern) => {
            let dash_f64: Vec<f64> = pattern.iter().map(|&d| d as f64).collect();
            ctx.set_dash(&dash_f64, 0.0);
        }
    }
}

/// Apply line style to Cairo context (backward compatibility wrapper)
pub fn apply_line_style(ctx: &mut Context, line: &LineElement) {
    apply_line_element(ctx, line);
}

/// Apply text element to Cairo context
pub fn apply_text_element(ctx: &mut Context, text: &TextElement) {
    // Set font family, weight, and style
    let family = text.family.as_deref().unwrap_or("sans-serif");
    
    let weight = match text.weight {
        Some(FontWeight::Bold) => cairo::FontWeight::Bold,
        Some(FontWeight::Light) => cairo::FontWeight::Normal,
        _ => cairo::FontWeight::Normal,
    };
    
    let slant = match text.style {
        Some(FontStyle::Italic) => cairo::FontSlant::Italic,
        Some(FontStyle::Oblique) => cairo::FontSlant::Oblique,
        _ => cairo::FontSlant::Normal,
    };
    
    ctx.select_font_face(family, slant, weight);
    ctx.set_font_size(text.size.unwrap_or(11.0));
    
    // Set color
    let color = text.color.unwrap_or(crate::theme::color::BLACK);
    let alpha = text.alpha.unwrap_or(1.0);
    ctx.set_source_rgba(
        color.0 as f64 / 255.0,
        color.1 as f64 / 255.0,
        color.2 as f64 / 255.0,
        (color.3 as f64 / 255.0) * alpha,
    );
}
