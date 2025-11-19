// Guide system for displaying legends and other visual aids

use crate::theme::Color;

/// Position for a legend
#[derive(Clone, Debug, PartialEq)]
pub enum LegendPosition {
    /// No legend displayed
    None,
    /// Left side of plot
    Left,
    /// Right side of plot (default)
    Right,
    /// Top of plot
    Top,
    /// Bottom of plot
    Bottom,
    /// Inside plot at specified normalized coordinates (x, y) where 0,0 is bottom-left
    Inside(f64, f64),
}

impl Default for LegendPosition {
    fn default() -> Self {
        LegendPosition::Right
    }
}

/// Direction for legend items
#[derive(Clone, Debug, PartialEq)]
pub enum LegendDirection {
    /// Items arranged vertically (default)
    Vertical,
    /// Items arranged horizontally
    Horizontal,
}

impl Default for LegendDirection {
    fn default() -> Self {
        LegendDirection::Vertical
    }
}

/// A single legend entry
#[derive(Clone, Debug)]
pub struct LegendEntry {
    /// Label text
    pub label: String,
    /// Color (if applicable)
    pub color: Option<Color>,
    /// Shape (if applicable)
    pub shape: Option<Shape>,
    /// Size (if applicable)
    pub size: Option<f64>,
}

/// Shape types for legend symbols
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Shape {
    Circle,
    Square,
    Triangle,
    Diamond,
    Cross,
    Plus,
}

/// Configuration for a legend guide
#[derive(Clone, Debug)]
pub struct LegendGuide {
    /// Title for the legend
    pub title: Option<String>,
    /// Position of the legend
    pub position: LegendPosition,
    /// Direction of legend items
    pub direction: LegendDirection,
    /// Number of rows (for horizontal layout) or columns (for vertical layout)
    pub ncol: Option<usize>,
    pub nrow: Option<usize>,
    /// Legend entries
    pub entries: Vec<LegendEntry>,
}

impl Default for LegendGuide {
    fn default() -> Self {
        LegendGuide {
            title: None,
            position: LegendPosition::default(),
            direction: LegendDirection::default(),
            ncol: None,
            nrow: None,
            entries: Vec::new(),
        }
    }
}

impl LegendGuide {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn position(mut self, position: LegendPosition) -> Self {
        self.position = position;
        self
    }

    pub fn direction(mut self, direction: LegendDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn ncol(mut self, ncol: usize) -> Self {
        self.ncol = Some(ncol);
        self
    }

    pub fn nrow(mut self, nrow: usize) -> Self {
        self.nrow = Some(nrow);
        self
    }
}

/// Configuration for an axis guide
#[derive(Clone, Debug)]
pub struct AxisGuide {
    /// Title for the axis
    pub title: Option<String>,
    // Future: could add custom breaks, labels, angle, etc.
}

impl Default for AxisGuide {
    fn default() -> Self {
        AxisGuide {
            title: None,
        }
    }
}

impl AxisGuide {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }
}

/// Guide configuration for a plot
#[derive(Clone, Debug)]
pub struct Guides {
    /// X-axis guide
    pub x_axis: Option<AxisGuide>,
    /// Y-axis guide
    pub y_axis: Option<AxisGuide>,
    /// Color legend
    pub color: Option<LegendGuide>,
    /// Shape legend
    pub shape: Option<LegendGuide>,
    /// Size legend
    pub size: Option<LegendGuide>,
    /// Alpha legend
    pub alpha: Option<LegendGuide>,
}

impl Default for Guides {
    fn default() -> Self {
        Guides {
            x_axis: None,
            y_axis: None,
            color: None,
            shape: None,
            size: None,
            alpha: None,
        }
    }
}

impl Guides {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn x_axis(mut self, guide: AxisGuide) -> Self {
        self.x_axis = Some(guide);
        self
    }

    pub fn y_axis(mut self, guide: AxisGuide) -> Self {
        self.y_axis = Some(guide);
        self
    }

    pub fn color(mut self, guide: LegendGuide) -> Self {
        self.color = Some(guide);
        self
    }

    pub fn shape(mut self, guide: LegendGuide) -> Self {
        self.shape = Some(guide);
        self
    }

    pub fn size(mut self, guide: LegendGuide) -> Self {
        self.size = Some(guide);
        self
    }

    pub fn alpha(mut self, guide: LegendGuide) -> Self {
        self.alpha = Some(guide);
        self
    }
}
