// Theme component templates for grammar of graphics

/// Color representation (could be RGB, RGBA, etc.)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color(pub u8, pub u8, pub u8, pub u8); // RGBA

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

/// Theme for axis components
#[derive(Clone, Debug, PartialEq)]
pub struct AxisTheme {
	pub line: LineStyle,
	pub ticks: LineStyle,
	pub label_font: Font,
	pub label_color: Color,
}

impl Default for AxisTheme {
    fn default() -> Self {
        AxisTheme {
            line: LineStyle {
                color: color::BLACK, // black
                width: 0.5,
                dash: None,
            },
            ticks: LineStyle {
                color: color::BLACK, // black
                width: 0.5,
                dash: None,
            },
            label_font: Font {
                family: "sans-serif".to_string(),
                size: 11.0,
                weight: FontWeight::Normal,
                style: FontStyle::Normal,
            },
            label_color: color::BLACK, // black
        }
    }
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

/// Theme for title
#[derive(Clone, Debug, PartialEq)]
pub struct TitleTheme {
	pub font: Font,
	pub color: Color,
	pub margin: Spacing,
}

impl Default for TitleTheme {
    fn default() -> Self {
        TitleTheme {
            font: Font {
                family: "sans-serif".to_string(),
                size: 14.0,
                weight: FontWeight::Bold,
                style: FontStyle::Normal,
            },
            color: color::BLACK, // black
            margin: Spacing {
                top: 10.0,
                right: 0.0,
                bottom: 10.0,
                left: 0.0,
            },
        }
    }
}

/// Main Theme struct
#[derive(Clone, Debug, PartialEq)]
pub struct Theme {
	pub background: Background,
	pub axis: AxisTheme,
	pub legend: LegendTheme,
	pub title: TitleTheme,
	pub plot_margin: Spacing,
}

impl Default for Theme {
	fn default() -> Self {
		Theme {
			background: Background::default(),
			axis: AxisTheme::default(),
			legend: LegendTheme::default(),
			title: TitleTheme::default(),
			plot_margin: Spacing {
				top: 5.5,
				right: 5.5,
				bottom: 5.5,
				left: 5.5,
			},
		}
	}
}
