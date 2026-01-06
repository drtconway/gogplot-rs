// Theme component templates for grammar of graphics

use crate::{
    aesthetics::{Aesthetic, AestheticDomain, AestheticProperty},
    geom::properties::{Property, PropertyValue},
    visuals::{LineStyle, Shape},
};
use std::collections::HashMap;

/// Color representation (could be RGB, RGBA, etc.)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(pub u8, pub u8, pub u8, pub u8); // RGBA

impl Color {
    /// Create a color with RGB values and alpha set to 255 (fully opaque)
    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color(r, g, b, 255)
    }
}

impl From<Color> for i64 {
    fn from(color: Color) -> i64 {
        ((color.0 as i64) << 24)
            | ((color.1 as i64) << 16)
            | ((color.2 as i64) << 8)
            | (color.3 as i64)
    }
}

impl From<i64> for Color {
    fn from(rgba: i64) -> Color {
        let r = ((rgba >> 24) & 0xFF) as u8;
        let g = ((rgba >> 16) & 0xFF) as u8;
        let b = ((rgba >> 8) & 0xFF) as u8;
        let a = (rgba & 0xFF) as u8;
        Color(r, g, b, a)
    }
}

pub mod color;
pub mod traits;

// ============================================================================
// Theme Element Types - used for per-geom customization
// ============================================================================

/// Element types for theme customization
#[derive(Clone, Debug, PartialEq)]
pub enum Element {
    Point(PointElement),
    Line(LineElement),
    Area(AreaElement),
    Text(TextElement),
}

/// Point element properties
#[derive(Clone, Debug, PartialEq)]
pub struct PointElement {
    pub color: Option<Color>,
    pub size: Option<f64>,
    pub alpha: Option<f64>,
    pub shape: Option<Shape>,
}

impl Default for PointElement {
    fn default() -> Self {
        PointElement {
            color: None,
            size: None,
            alpha: None,
            shape: None,
        }
    }
}

impl traits::PointElement for PointElement {
    fn this(&self) -> &self::PointElement {
        self
    }

    fn this_mut(&mut self) -> &mut self::PointElement {
        self
    }
}

impl PointElement {
    pub fn overrides(&self, overrides: &mut Vec<Aesthetic>) {
        if self.size.is_some() {
            overrides.push(Aesthetic::Size(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Size(AestheticDomain::Discrete));
        }
        if self.color.is_some() {
            overrides.push(Aesthetic::Color(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Color(AestheticDomain::Discrete));
        }
        if self.alpha.is_some() {
            overrides.push(Aesthetic::Alpha(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Alpha(AestheticDomain::Discrete));
        }
        if self.shape.is_some() {
            overrides.push(Aesthetic::Shape);
        }
    }

    pub fn properties(&self, props: &mut HashMap<AestheticProperty, Property>) {
        if let Some(size_prop) = &self.size {
            props.insert(AestheticProperty::Size, Property::Float(size_prop.clone()));
        }
        if let Some(color_prop) = &self.color {
            props.insert(
                AestheticProperty::Color,
                Property::Color(color_prop.clone()),
            );
        }
        if let Some(alpha_prop) = &self.alpha {
            props.insert(
                AestheticProperty::Alpha,
                Property::Float(alpha_prop.clone()),
            );
        }
        if let Some(shape_prop) = &self.shape {
            props.insert(
                AestheticProperty::Shape,
                Property::Shape(shape_prop.clone()),
            );
        }
    }

    pub fn defaults(
        &self,
        geom: &'static str,
        group: &'static str,
        theme: &Theme,
        defaults: &mut HashMap<AestheticProperty, PropertyValue>,
    ) {
        // Start with hardcoded defaults
        let mut default_size = 1.0;
        let mut default_color = color::BLACK;
        let mut default_alpha = 1.0;
        let mut default_shape = crate::visuals::Shape::Circle;

        // Apply theme overrides if present
        if let Some(crate::theme::Element::Point(elem)) = theme.get_element(geom, group) {
            if let Some(size) = elem.size {
                default_size = size;
            }
            if let Some(color) = elem.color {
                default_color = color;
            }
            if let Some(alpha) = elem.alpha {
                default_alpha = alpha;
            }
            if let Some(ref shape) = elem.shape {
                default_shape = shape.clone();
            }
        }

        // Only set defaults for properties not explicitly set on the geom
        if self.size.is_none() {
            defaults.insert(AestheticProperty::Size, PropertyValue::Float(default_size));
        }
        if self.color.is_none() {
            defaults.insert(
                AestheticProperty::Color,
                PropertyValue::Color(default_color),
            );
        }
        if self.alpha.is_none() {
            defaults.insert(
                AestheticProperty::Alpha,
                PropertyValue::Float(default_alpha),
            );
        }
        if self.shape.is_none() {
            defaults.insert(
                AestheticProperty::Shape,
                PropertyValue::Shape(default_shape),
            );
        }
    }
}

/// Line element properties
#[derive(Clone, Debug, PartialEq)]
pub struct LineElement {
    pub color: Option<Color>,
    pub size: Option<f64>,
    pub alpha: Option<f64>,
    pub linestyle: Option<LineStyle>,
}

impl Default for LineElement {
    fn default() -> Self {
        LineElement {
            color: None,
            size: None,
            alpha: None,
            linestyle: None,
        }
    }
}

impl traits::LineElement for LineElement {
    fn this(&self) -> &self::LineElement {
        self
    }

    fn this_mut(&mut self) -> &mut self::LineElement {
        self
    }
}

impl LineElement {
    pub fn overrides(&self, overrides: &mut Vec<Aesthetic>) {
        if self.size.is_some() {
            overrides.push(Aesthetic::Size(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Size(AestheticDomain::Discrete));
        }
        if self.color.is_some() {
            overrides.push(Aesthetic::Color(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Color(AestheticDomain::Discrete));
        }
        if self.alpha.is_some() {
            overrides.push(Aesthetic::Alpha(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Alpha(AestheticDomain::Discrete));
        }
        if self.linestyle.is_some() {
            overrides.push(Aesthetic::Linetype);
        }
    }

    pub fn properties(&self, props: &mut HashMap<AestheticProperty, Property>) {
        if let Some(size_prop) = &self.size {
            props.insert(AestheticProperty::Size, Property::Float(size_prop.clone()));
        }
        if let Some(color_prop) = &self.color {
            props.insert(
                AestheticProperty::Color,
                Property::Color(color_prop.clone()),
            );
        }
        if let Some(alpha_prop) = &self.alpha {
            props.insert(
                AestheticProperty::Alpha,
                Property::Float(alpha_prop.clone()),
            );
        }
        if let Some(linestyle_prop) = &self.linestyle {
            props.insert(
                AestheticProperty::Linetype,
                Property::LineStyle(linestyle_prop.clone()),
            );
        }
    }

    pub fn defaults(
        &self,
        geom: &'static str,
        group: &'static str,
        theme: &Theme,
        defaults: &mut HashMap<AestheticProperty, PropertyValue>,
    ) {
        // Start with hardcoded defaults
        let mut default_size = 1.0;
        let mut default_color = color::BLACK;
        let mut default_alpha = 1.0;
        let mut default_linestyle = crate::visuals::LineStyle::Solid;

        // Apply theme overrides if present
        if let Some(crate::theme::Element::Line(elem)) = theme.get_element(geom, group) {
            if let Some(size) = elem.size {
                default_size = size;
            }
            if let Some(color) = elem.color {
                default_color = color;
            }
            if let Some(alpha) = elem.alpha {
                default_alpha = alpha;
            }
            if let Some(ref linestyle) = elem.linestyle {
                default_linestyle = linestyle.clone();
            }
        }

        // Only set defaults for properties not explicitly set on the geom
        if self.size.is_none() {
            defaults.insert(AestheticProperty::Size, PropertyValue::Float(default_size));
        }
        if self.color.is_none() {
            defaults.insert(
                AestheticProperty::Color,
                PropertyValue::Color(default_color),
            );
        }
        if self.alpha.is_none() {
            defaults.insert(
                AestheticProperty::Alpha,
                PropertyValue::Float(default_alpha),
            );
        }
        if self.linestyle.is_none() {
            defaults.insert(
                AestheticProperty::Linetype,
                PropertyValue::LineStyle(default_linestyle),
            );
        }
    }
}

/// area element properties
#[derive(Clone, Debug, PartialEq)]
pub struct AreaElement {
    pub fill: Option<Color>,
    pub color: Option<Color>, // border color
    pub alpha: Option<f64>,
    pub size: Option<f64>, // border width
    pub linestyle: Option<LineStyle>,
}

impl Default for AreaElement {
    fn default() -> Self {
        AreaElement {
            fill: None,
            color: None,
            alpha: None,
            size: None,
            linestyle: None,
        }
    }
}

impl traits::AreaElement for AreaElement {
    fn this(&self) -> &Self {
        self
    }

    fn this_mut(&mut self) -> &mut Self {
        self
    }
}

impl AreaElement {
    pub fn overrides(&self, overrides: &mut Vec<Aesthetic>) {
        if self.size.is_some() {
            overrides.push(Aesthetic::Size(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Size(AestheticDomain::Discrete));
        }
        if self.color.is_some() {
            overrides.push(Aesthetic::Color(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Color(AestheticDomain::Discrete));
        }
        if self.fill.is_some() {
            overrides.push(Aesthetic::Fill(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Fill(AestheticDomain::Discrete));
        }
        if self.alpha.is_some() {
            overrides.push(Aesthetic::Alpha(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Alpha(AestheticDomain::Discrete));
        }
        if self.linestyle.is_some() {
            overrides.push(Aesthetic::Linetype);
        }
    }

    pub fn properties(&self, props: &mut HashMap<AestheticProperty, Property>) {
        if let Some(size_prop) = &self.size {
            props.insert(AestheticProperty::Size, Property::Float(size_prop.clone()));
        }
        if let Some(color_prop) = &self.color {
            props.insert(
                AestheticProperty::Color,
                Property::Color(color_prop.clone()),
            );
        }
        if let Some(fill_prop) = &self.fill {
            props.insert(AestheticProperty::Fill, Property::Color(fill_prop.clone()));
        }
        if let Some(alpha_prop) = &self.alpha {
            props.insert(
                AestheticProperty::Alpha,
                Property::Float(alpha_prop.clone()),
            );
        }
        if let Some(linestyle_prop) = &self.linestyle {
            props.insert(
                AestheticProperty::Linetype,
                Property::LineStyle(linestyle_prop.clone()),
            );
        }
    }

    pub fn defaults(
        &self,
        geom: &'static str,
        group: &'static str,
        theme: &Theme,
        defaults: &mut HashMap<AestheticProperty, PropertyValue>,
    ) {
        // Start with hardcoded defaults
        let mut default_size = 1.0;
        let mut default_color = color::BLACK;
        let mut default_fill = color::GRAY90;
        let mut default_alpha = 1.0;
        let mut default_linestyle = crate::visuals::LineStyle::Solid;

        // Apply theme overrides if present
        if let Some(crate::theme::Element::Area(elem)) = theme.get_element(geom, group) {
            if let Some(size) = elem.size {
                default_size = size;
            }
            if let Some(color) = elem.color {
                default_color = color;
            }
            if let Some(fill) = elem.fill {
                default_fill = fill;
            }
            if let Some(alpha) = elem.alpha {
                default_alpha = alpha;
            }
            if let Some(ref linestyle) = elem.linestyle {
                default_linestyle = linestyle.clone();
            }
        }

        // Only set defaults for properties not explicitly set on the geom
        if self.size.is_none() {
            defaults.insert(AestheticProperty::Size, PropertyValue::Float(default_size));
        }
        if self.color.is_none() {
            defaults.insert(
                AestheticProperty::Color,
                PropertyValue::Color(default_color),
            );
        }
        if self.fill.is_none() {
            defaults.insert(AestheticProperty::Fill, PropertyValue::Color(default_fill));
        }
        if self.alpha.is_none() {
            defaults.insert(
                AestheticProperty::Alpha,
                PropertyValue::Float(default_alpha),
            );
        }
        if self.linestyle.is_none() {
            defaults.insert(
                AestheticProperty::Linetype,
                PropertyValue::LineStyle(default_linestyle),
            );
        }
    }
}

/// Text element properties
#[derive(Clone, Debug, PartialEq, Default)]
pub struct TextElement {
    pub color: Option<Color>,
    pub size: Option<f64>,
    pub alpha: Option<f64>,
    pub family: Option<String>,
    pub weight: Option<FontWeight>,
    pub style: Option<FontStyle>,
    pub hjust: Option<f64>,
    pub vjust: Option<f64>,
}

impl TextElement {
    pub fn overrides(&self, overrides: &mut Vec<Aesthetic>) {
        if self.color.is_some() {
            overrides.push(Aesthetic::Color(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Color(AestheticDomain::Discrete));
        }
        if self.size.is_some() {
            overrides.push(Aesthetic::Size(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Size(AestheticDomain::Discrete));
        }
        if self.alpha.is_some() {
            overrides.push(Aesthetic::Alpha(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Alpha(AestheticDomain::Discrete));
        }
    }

    pub fn properties(&self, props: &mut HashMap<AestheticProperty, Property>) {
        if let Some(color_prop) = &self.color {
            props.insert(
                AestheticProperty::Color,
                Property::Color(color_prop.clone()),
            );
        }
        if let Some(size_prop) = &self.size {
            props.insert(AestheticProperty::Size, Property::Float(size_prop.clone()));
        }
        if let Some(alpha_prop) = &self.alpha {
            props.insert(
                AestheticProperty::Alpha,
                Property::Float(alpha_prop.clone()),
            );
        }
    }

    pub fn defaults(
        &self,
        geom: &'static str,
        group: &'static str,
        theme: &Theme,
        defaults: &mut HashMap<AestheticProperty, PropertyValue>,
    ) {
        // Start with hardcoded defaults
        let mut default_color = color::BLACK;
        let mut default_size = 1.0;
        let mut default_alpha = 1.0;

        // Apply theme overrides if present
        if let Some(crate::theme::Element::Area(elem)) = theme.get_element(geom, group) {
            if let Some(size) = elem.size {
                default_size = size;
            }
            if let Some(color) = elem.color {
                default_color = color;
            }
            if let Some(alpha) = elem.alpha {
                default_alpha = alpha;
            }
        }

        // Only set defaults for properties not explicitly set on the geom
        if self.color.is_none() {
            defaults.insert(
                AestheticProperty::Color,
                PropertyValue::Color(default_color),
            );
        }
        if self.size.is_none() {
            defaults.insert(AestheticProperty::Size, PropertyValue::Float(default_size));
        }
        if self.alpha.is_none() {
            defaults.insert(
                AestheticProperty::Alpha,
                PropertyValue::Float(default_alpha),
            );
        }
    }
}

impl traits::TextElement for TextElement {
    fn this(&self) -> &Self {
        self
    }

    fn this_mut(&mut self) -> &mut Self {
        self
    }
}

// Helper constructors for ergonomic API
pub fn point() -> PointElement {
    PointElement::default()
}

pub fn line() -> LineElement {
    LineElement::default()
}

pub fn rect() -> AreaElement {
    AreaElement::default()
}

pub fn text() -> TextElement {
    TextElement::default()
}

// Into implementations for Element enum
impl From<PointElement> for Element {
    fn from(e: PointElement) -> Element {
        Element::Point(e)
    }
}

impl From<LineElement> for Element {
    fn from(e: LineElement) -> Element {
        Element::Line(e)
    }
}

impl From<AreaElement> for Element {
    fn from(e: AreaElement) -> Element {
        Element::Area(e)
    }
}

impl From<TextElement> for Element {
    fn from(e: TextElement) -> Element {
        Element::Text(e)
    }
}

// ============================================================================
// Structural Theme Components (existing)
// ============================================================================

/// Font representation
#[derive(Clone, Debug, PartialEq)]
pub struct Font {
    pub family: String,
    pub size: f32,
    pub weight: FontWeight,
    pub style: FontStyle,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FontWeight {
    Normal,
    Bold,
    Light,
}

impl From<&str> for FontWeight {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "bold" => FontWeight::Bold,
            "light" => FontWeight::Light,
            _ => FontWeight::Normal,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

impl From<&str> for FontStyle {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "italic" => FontStyle::Italic,
            "oblique" => FontStyle::Oblique,
            _ => FontStyle::Normal,
        }
    }
}

/// Line drawing style for theme borders, axes, etc. (not to be confused with visuals::LineStyle)
#[derive(Clone, Debug, PartialEq)]
pub struct LineDrawStyle {
    pub color: Color,
    pub width: f32,
    pub dash: Option<Vec<f32>>, // Dash pattern
}

/// Fill style representation
#[derive(Clone, Debug, PartialEq)]
pub struct FillStyle {
    pub color: Color,
    pub opacity: f32,
}

/// Margin and padding
#[derive(Clone, Debug, PartialEq)]
pub struct Spacing {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

/// Theme for plot background
#[derive(Clone, Debug, PartialEq)]
pub struct Background {
    pub fill: FillStyle,
}

impl Default for Background {
    fn default() -> Self {
        Background {
            fill: FillStyle {
                color: color::WHITE,
                opacity: 1.0,
            },
        }
    }
}

/// Text element theme (for labels, titles, etc.)
#[derive(Clone, Debug, PartialEq)]
pub struct TextTheme {
    pub font: Font,
    pub color: Color,
    pub margin: Spacing,
}

impl Default for TextTheme {
    fn default() -> Self {
        TextTheme {
            font: Font {
                family: "Sans".to_string(),
                size: 11.0,
                weight: FontWeight::Normal,
                style: FontStyle::Normal,
            },
            color: color::BLACK,
            margin: Spacing {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            },
        }
    }
}

/// Theme for axis line and ticks
#[derive(Clone, Debug, PartialEq)]
pub struct AxisLineTheme {
    pub line: Option<LineDrawStyle>,
    pub ticks: Option<LineDrawStyle>,
    pub tick_length: f32,
}

impl Default for AxisLineTheme {
    fn default() -> Self {
        AxisLineTheme {
            line: Some(LineDrawStyle {
                color: color::BLACK,
                width: 1.0,
                dash: None,
            }),
            ticks: Some(LineDrawStyle {
                color: color::BLACK,
                width: 1.0,
                dash: None,
            }),
            tick_length: 5.0,
        }
    }
}

/// Theme for axis text (labels and title)
#[derive(Clone, Debug, PartialEq)]
pub struct AxisTextTheme {
    pub text: TextTheme,
    pub title: TextTheme,
}

impl Default for AxisTextTheme {
    fn default() -> Self {
        AxisTextTheme {
            text: TextTheme {
                font: Font {
                    family: "Sans".to_string(),
                    size: 10.0,
                    weight: FontWeight::Normal,
                    style: FontStyle::Normal,
                },
                color: color::BLACK,
                margin: Spacing {
                    top: 5.0,
                    right: 5.0,
                    bottom: 5.0,
                    left: 5.0,
                },
            },
            title: TextTheme {
                font: Font {
                    family: "Sans".to_string(),
                    size: 12.0,
                    weight: FontWeight::Normal,
                    style: FontStyle::Normal,
                },
                color: color::BLACK,
                margin: Spacing {
                    top: 10.0,
                    right: 0.0,
                    bottom: 10.0,
                    left: 0.0,
                },
            },
        }
    }
}

/// Theme for axis components
#[derive(Clone, Debug, PartialEq, Default)]
pub struct AxisTheme {
    pub line: AxisLineTheme,
    pub text: AxisTextTheme,
}

/// Theme for legend components
#[derive(Clone, Debug, PartialEq)]
pub struct LegendTheme {
    pub background: FillStyle,
    pub border: LineDrawStyle,
    pub text_font: Font,
    pub text_color: Color,
}

impl Default for LegendTheme {
    fn default() -> Self {
        LegendTheme {
            background: FillStyle {
                color: color::WHITE, // white
                opacity: 0.8,
            },
            border: LineDrawStyle {
                color: color::BLACK, // black
                width: 0.5,
                dash: None,
            },
            text_font: Font {
                family: "sans-serif".to_string(),
                size: 10.0,
                weight: FontWeight::Normal,
                style: FontStyle::Normal,
            },
            text_color: color::BLACK, // black
        }
    }
}

/// Theme for panel (plot area) background and grid
#[derive(Clone, Debug, PartialEq)]
pub struct PanelTheme {
    pub background: Option<FillStyle>,
    pub border: Option<LineDrawStyle>,
    pub grid_major: Option<LineDrawStyle>,
    pub grid_minor: Option<LineDrawStyle>,
}

impl Default for PanelTheme {
    fn default() -> Self {
        PanelTheme {
            background: Some(FillStyle {
                color: Color(245, 245, 245, 255), // light gray
                opacity: 1.0,
            }),
            border: None,
            grid_major: Some(LineDrawStyle {
                color: color::WHITE,
                width: 1.5,
                dash: None,
            }),
            grid_minor: Some(LineDrawStyle {
                color: color::WHITE,
                width: 0.75,
                dash: None,
            }),
        }
    }
}

/// Theme for plot title
#[derive(Clone, Debug, PartialEq)]
pub struct PlotTitleTheme {
    pub text: TextTheme,
}

impl Default for PlotTitleTheme {
    fn default() -> Self {
        PlotTitleTheme {
            text: TextTheme {
                font: Font {
                    family: "Sans".to_string(),
                    size: 14.0,
                    weight: FontWeight::Bold,
                    style: FontStyle::Normal,
                },
                color: color::BLACK,
                margin: Spacing {
                    top: 5.0,
                    right: 0.0,
                    bottom: 10.0,
                    left: 0.0,
                },
            },
        }
    }
}

/// Theme for geom_line
#[derive(Clone, Debug, PartialEq)]
pub struct GeomLineTheme {
    pub color: Color,
    pub size: f64, // Line width
    pub alpha: f64,
    pub linestyle: crate::visuals::LineStyle,
}

impl Default for GeomLineTheme {
    fn default() -> Self {
        GeomLineTheme {
            color: color::BLACK,
            size: 1.0,
            alpha: 1.0,
            linestyle: crate::visuals::LineStyle::Solid,
        }
    }
}

/// Theme for geom_point
#[derive(Clone, Debug, PartialEq)]
pub struct GeomPointTheme {
    pub color: Color,
    pub size: f64,
    pub alpha: f64,
    pub shape: i64, // Shape as i64 to match visuals::Shape
}

impl Default for GeomPointTheme {
    fn default() -> Self {
        GeomPointTheme {
            color: color::BLACK,
            size: 3.0,
            alpha: 1.0,
            shape: 16, // Circle (Shape::Circle as i64)
        }
    }
}

/// Theme for geom_rect (bars, histograms, boxplots)
#[derive(Clone, Debug, PartialEq)]
pub struct GeomRectTheme {
    pub fill: Color,
    pub color: Color,
    pub alpha: f64,
}

impl Default for GeomRectTheme {
    fn default() -> Self {
        GeomRectTheme {
            fill: Color(128, 128, 128, 255), // Gray fill
            color: color::BLACK,             // Black outline
            alpha: 1.0,
        }
    }
}

/// Theme for geom_text
#[derive(Clone, Debug, PartialEq)]
pub struct GeomTextTheme {
    pub font: Font,
    pub color: Color,
    pub size: f64,
    pub alpha: f64,
    pub hjust: f64,
    pub vjust: f64,
}

impl Default for GeomTextTheme {
    fn default() -> Self {
        GeomTextTheme {
            font: Font {
                family: "Sans".to_string(),
                size: 11.0,
                weight: FontWeight::Normal,
                style: FontStyle::Normal,
            },
            color: color::BLACK,
            size: 11.0,
            alpha: 1.0,
            hjust: 0.0, // Left edge at point (text extends right)
            vjust: 1.0, // Top at point (text extends down/below point in visual space)
        }
    }
}

/// Main Theme struct
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
    pub background: Background,
    pub panel: PanelTheme,
    pub axis_x: AxisTheme,
    pub axis_y: AxisTheme,
    pub legend: LegendTheme,
    pub plot_title: PlotTitleTheme,
    pub plot_margin: Spacing,
    pub geom_line: GeomLineTheme,
    pub geom_point: GeomPointTheme,
    pub geom_rect: GeomRectTheme,
    pub geom_text: GeomTextTheme,

    // Per-geom element overrides: geom_name -> element_name -> Element
    geom_elements: HashMap<&'static str, HashMap<&'static str, Element>>,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            background: Background::default(),
            panel: PanelTheme::default(),
            axis_x: AxisTheme::default(),
            axis_y: AxisTheme::default(),
            legend: LegendTheme::default(),
            plot_title: PlotTitleTheme::default(),
            plot_margin: Spacing {
                top: 40.0,
                right: 20.0,
                bottom: 60.0,
                left: 70.0,
            },
            geom_line: GeomLineTheme::default(),
            geom_point: GeomPointTheme::default(),
            geom_rect: GeomRectTheme::default(),
            geom_text: GeomTextTheme::default(),
            geom_elements: HashMap::new(),
        }
    }
}

impl Theme {
    /// Create a minimal theme with no panel background or grid
    pub fn minimal() -> Self {
        let mut theme = Theme::default();
        theme.panel.background = None;
        theme.panel.grid_major = None;
        theme.panel.grid_minor = None;
        theme
    }

    /// Create a classic theme similar to base R graphics
    pub fn classic() -> Self {
        let mut theme = Theme::default();
        theme.panel.background = Some(FillStyle {
            color: color::WHITE,
            opacity: 1.0,
        });
        theme.panel.border = Some(LineDrawStyle {
            color: color::BLACK,
            width: 1.0,
            dash: None,
        });
        theme.panel.grid_major = None;
        theme.panel.grid_minor = None;
        theme
    }

    /// Create a dark theme
    pub fn dark() -> Self {
        let mut theme = Theme::default();
        let dark_bg = Color(30, 30, 30, 255);
        let light_gray = Color(200, 200, 200, 255);

        theme.background.fill.color = dark_bg;
        theme.panel.background = Some(FillStyle {
            color: Color(50, 50, 50, 255),
            opacity: 1.0,
        });
        theme.panel.grid_major = Some(LineDrawStyle {
            color: Color(70, 70, 70, 255),
            width: 1.0,
            dash: None,
        });

        // Update all text colors
        theme.axis_x.text.text.color = light_gray;
        theme.axis_x.text.title.color = light_gray;
        theme.axis_y.text.text.color = light_gray;
        theme.axis_y.text.title.color = light_gray;
        theme.plot_title.text.color = light_gray;

        // Update axis lines
        if let Some(ref mut line) = theme.axis_x.line.line {
            line.color = light_gray;
        }
        if let Some(ref mut ticks) = theme.axis_x.line.ticks {
            ticks.color = light_gray;
        }
        if let Some(ref mut line) = theme.axis_y.line.line {
            line.color = light_gray;
        }
        if let Some(ref mut ticks) = theme.axis_y.line.ticks {
            ticks.color = light_gray;
        }

        // Update legend styling
        theme.legend.background.color = Color(50, 50, 50, 255); // Same as panel background
        theme.legend.text_color = light_gray;
        theme.legend.border.color = Color(70, 70, 70, 255); // Same as grid lines

        theme
    }

    /// Get a builder for customizing a specific geom's theme
    pub fn geom(&mut self, geom_name: &'static str) -> GeomThemeBuilder<'_> {
        GeomThemeBuilder {
            theme: self,
            geom_name,
        }
    }

    /// Get an element override for a specific geom
    pub fn get_element(&self, geom_name: &str, element_name: &str) -> Option<&Element> {
        self.geom_elements.get(geom_name)?.get(element_name)
    }
}

/// Builder for customizing geom-specific theme elements
pub struct GeomThemeBuilder<'a> {
    theme: &'a mut Theme,
    geom_name: &'static str,
}

impl<'a> GeomThemeBuilder<'a> {
    /// Select a specific element to customize
    pub fn element(self, element_name: &'static str) -> ElementBuilder<'a> {
        ElementBuilder {
            theme: self.theme,
            geom_name: self.geom_name,
            element_name,
        }
    }
}

/// Builder for setting element values
pub struct ElementBuilder<'a> {
    theme: &'a mut Theme,
    geom_name: &'static str,
    element_name: &'static str,
}

impl<'a> ElementBuilder<'a> {
    /// Set the element value
    pub fn set(self, element: impl Into<Element>) {
        self.theme
            .geom_elements
            .entry(self.geom_name)
            .or_insert_with(HashMap::new)
            .insert(self.element_name, element.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        aesthetics::builder::{
            ColorDiscreteAesBuilder, XContinuousAesBuilder, YContinuousAesBuilder,
        },
        error::to_io_error,
        geom::{line::geom_line, point::geom_point},
        plot::plot,
        theme::{
            point,
            traits::{LineElement, PointElement},
        },
        utils::mtcars::mtcars,
    };

    fn init_test_logging() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn theme_custom_color() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data)
            .aes(|a| {
                a.x_continuous("wt");
                a.y_continuous("mpg");
            })
            .theme(|theme| {
                theme
                    .geom("point")
                    .element("point")
                    .set(point().color(color::CORAL).size(5.0));
            })
            + geom_point();

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/theme_points_color.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn theme_custom_size_shape() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data)
            .aes(|a| {
                a.x_continuous("wt");
                a.y_continuous("mpg");
                a.color_discrete("cyl");
            })
            .theme(|theme| {
                theme
                    .geom("point")
                    .element("point")
                    .set(point().size(8.0).shape(Shape::Triangle));
            })
            + geom_point();

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/theme_points_size_shape.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn theme_custom_alpha() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data)
            .aes(|a| {
                a.x_continuous("wt");
                a.y_continuous("mpg");
            })
            .theme(|theme| {
                theme
                    .geom("point")
                    .element("point")
                    .set(point().color(color::STEELBLUE).size(6.0).alpha(0.3));
            })
            + geom_point();

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/theme_points_alpha.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn theme_overridden_by_geom() {
        init_test_logging();

        let data = mtcars();

        // Theme sets one color, but geom explicitly overrides it
        let builder = plot(&data)
            .aes(|a| {
                a.x_continuous("wt");
                a.y_continuous("mpg");
            })
            .theme(|theme| {
                theme
                    .geom("point")
                    .element("point")
                    .set(crate::theme::point().color(color::RED).size(3.0));
            })
            + geom_point().color(color::GREEN).size(7.0);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/theme_points_override.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn theme_partial_override() {
        init_test_logging();

        let data = mtcars();

        // Theme sets color and size, but geom only overrides color
        // Size should come from theme
        let builder = plot(&data)
            .aes(|a| {
                a.x_continuous("wt");
                a.y_continuous("mpg");
            })
            .theme(|theme| {
                theme.geom("point").element("point").set(
                    point()
                        .color(color::ORANGE)
                        .size(10.0)
                        .shape(Shape::Diamond),
                );
            })
            + geom_point().color(color::PURPLE);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/theme_points_partial.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn theme_custom_linestyle() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data)
            .aes(|a| {
                a.x_continuous("wt");
                a.y_continuous("mpg");
            })
            .theme(|theme| {
                theme.geom("line").element("line").set(
                    line()
                        .linestyle(LineStyle::from("-- "))
                        .color(color::DARKBLUE)
                        .size(2.0),
                );
            })
            + geom_line(); // Should use theme's dashed linestyle

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/theme_lines_custom_linestyle.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }
}
