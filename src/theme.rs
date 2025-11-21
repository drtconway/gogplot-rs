// Theme component templates for grammar of graphics

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

pub mod color;

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

#[derive(Clone, Debug, PartialEq)]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

/// Line style representation
#[derive(Clone, Debug, PartialEq)]
pub struct LineStyle {
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
    pub line: Option<LineStyle>,
    pub ticks: Option<LineStyle>,
    pub tick_length: f32,
}

impl Default for AxisLineTheme {
    fn default() -> Self {
        AxisLineTheme {
            line: Some(LineStyle {
                color: color::BLACK,
                width: 1.0,
                dash: None,
            }),
            ticks: Some(LineStyle {
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
#[derive(Clone, Debug, PartialEq)]
#[derive(Default)]
pub struct AxisTheme {
    pub line: AxisLineTheme,
    pub text: AxisTextTheme,
}


/// Theme for legend components
#[derive(Clone, Debug, PartialEq)]
pub struct LegendTheme {
    pub background: FillStyle,
    pub border: LineStyle,
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
            border: LineStyle {
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
    pub border: Option<LineStyle>,
    pub grid_major: Option<LineStyle>,
    pub grid_minor: Option<LineStyle>,
}

impl Default for PanelTheme {
    fn default() -> Self {
        PanelTheme {
            background: Some(FillStyle {
                color: Color(245, 245, 245, 255), // light gray
                opacity: 1.0,
            }),
            border: None,
            grid_major: Some(LineStyle {
                color: color::WHITE,
                width: 1.5,
                dash: None,
            }),
            grid_minor: Some(LineStyle {
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
        theme.panel.border = Some(LineStyle {
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
        theme.panel.grid_major = Some(LineStyle {
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
}
