// Cairo rendering helpers

use crate::theme::{Color, Font, FontStyle, FontWeight, LineStyle, TextTheme};
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

/// Apply line style to Cairo context
pub fn apply_line_style(ctx: &mut Context, line: &LineStyle) {
    apply_color(ctx, &line.color);
    ctx.set_line_width(line.width as f64);
    if let Some(ref dash) = line.dash {
        let dash_f64: Vec<f64> = dash.iter().map(|&d| d as f64).collect();
        ctx.set_dash(&dash_f64, 0.0);
    } else {
        ctx.set_dash(&[], 0.0);
    }
}

/// Apply text theme to Cairo context
pub fn apply_text_theme(ctx: &mut Context, text: &TextTheme) {
    apply_font(ctx, &text.font);
    apply_color(ctx, &text.color);
}
