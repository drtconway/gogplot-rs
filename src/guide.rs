// Guide system for displaying legends and other visual aids

use crate::theme::Color;
use crate::visuals::Shape;

/// Position for a legend
#[derive(Clone, Debug, PartialEq)]
#[derive(Default)]
pub enum LegendPosition {
    /// No legend displayed
    None,
    /// Left side of plot
    Left,
    /// Right side of plot (default)
    #[default]
    Right,
    /// Top of plot
    Top,
    /// Bottom of plot
    Bottom,
    /// Inside plot at specified normalized coordinates (x, y) where 0,0 is bottom-left
    Inside(f64, f64),
}


/// Direction for legend items
#[derive(Clone, Debug, PartialEq)]
#[derive(Default)]
pub enum LegendDirection {
    /// Items arranged vertically (default)
    #[default]
    Vertical,
    /// Items arranged horizontally
    Horizontal,
}


/// Position for X-axis
#[derive(Clone, Debug, PartialEq)]
#[derive(Default)]
pub enum XAxisPosition {
    /// Bottom of plot (default)
    #[default]
    Bottom,
    /// Top of plot
    Top,
}


/// Position for Y-axis
#[derive(Clone, Debug, PartialEq)]
#[derive(Default)]
pub enum YAxisPosition {
    /// Left side of plot (default)
    #[default]
    Left,
    /// Right side of plot
    Right,
}


/// A single legend entry
#[derive(Clone, Debug)]
#[derive(Default)]
pub struct LegendEntry {
    /// Label text
    pub label: String,
    /// Outline/stroke color (if applicable)
    pub color: Option<Color>,
    /// Fill color (if applicable)
    pub fill: Option<Color>,
    /// Shape (if applicable)
    pub shape: Option<Shape>,
    /// Size (if applicable)
    pub size: Option<f64>,
    /// Alpha/transparency (if applicable)
    pub alpha: Option<f64>,
}


impl LegendEntry {
    pub fn new(label: impl Into<String>) -> Self {
        LegendEntry {
            label: label.into(),
            ..Default::default()
        }
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn fill(mut self, fill: Color) -> Self {
        self.fill = Some(fill);
        self
    }

    pub fn shape(mut self, shape: Shape) -> Self {
        self.shape = Some(shape);
        self
    }

    pub fn size(mut self, size: f64) -> Self {
        self.size = Some(size);
        self
    }

    pub fn alpha(mut self, alpha: f64) -> Self {
        self.alpha = Some(alpha);
        self
    }
}

/// Type of legend guide
#[derive(Clone, Debug)]
pub enum LegendType {
    /// Discrete legend with individual entries
    Discrete,
    /// Continuous color bar with gradient
    ColorBar {
        /// Domain range (min, max)
        domain: (f64, f64),
        /// Colors to interpolate
        colors: Vec<Color>,
        /// Break positions for tick marks
        breaks: Vec<f64>,
        /// Labels for the breaks
        labels: Vec<String>,
    },
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
    /// Legend entries (for discrete legends)
    pub entries: Vec<LegendEntry>,
    /// Type of legend (discrete or continuous color bar)
    pub legend_type: LegendType,
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
            legend_type: LegendType::Discrete,
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

    pub fn entry(mut self, entry: LegendEntry) -> Self {
        self.entries.push(entry);
        self
    }
}

/// Axis position (used internally to determine X or Y axis)
#[derive(Clone, Debug, PartialEq)]
pub enum AxisType {
    X(XAxisPosition),
    Y(YAxisPosition),
}

/// Configuration for an axis guide
#[derive(Clone, Debug)]
pub struct AxisGuide {
    /// Title for the axis
    pub title: Option<String>,
    /// Position of the axis (X: Bottom/Top, Y: Left/Right)
    pub position: AxisType,
    // Future: could add custom breaks, labels, angle, etc.
}

impl AxisGuide {
    /// Create a new X-axis guide with bottom position (default)
    pub fn x() -> Self {
        AxisGuide {
            title: None,
            position: AxisType::X(XAxisPosition::Bottom),
        }
    }

    /// Create a new Y-axis guide with left position (default)
    pub fn y() -> Self {
        AxisGuide {
            title: None,
            position: AxisType::Y(YAxisPosition::Left),
        }
    }

    /// Set the title for the axis
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set X-axis position to bottom
    pub fn bottom(mut self) -> Self {
        self.position = AxisType::X(XAxisPosition::Bottom);
        self
    }

    /// Set X-axis position to top
    pub fn top(mut self) -> Self {
        self.position = AxisType::X(XAxisPosition::Top);
        self
    }

    /// Set Y-axis position to left
    pub fn left(mut self) -> Self {
        self.position = AxisType::Y(YAxisPosition::Left);
        self
    }

    /// Set Y-axis position to right
    pub fn right(mut self) -> Self {
        self.position = AxisType::Y(YAxisPosition::Right);
        self
    }
}

/// Guide configuration for a plot
#[derive(Clone, Debug)]
#[derive(Default)]
pub struct Guides {
    /// X-axis guide
    pub x_axis: Option<AxisGuide>,
    /// Y-axis guide
    pub y_axis: Option<AxisGuide>,
    /// Color legend
    pub color: Option<LegendGuide>,
    /// Fill legend
    pub fill: Option<LegendGuide>,
    /// Shape legend
    pub shape: Option<LegendGuide>,
    /// Size legend
    pub size: Option<LegendGuide>,
    /// Alpha legend
    pub alpha: Option<LegendGuide>,
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

    pub fn fill(mut self, guide: LegendGuide) -> Self {
        self.fill = Some(guide);
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

    /// Suppress the color legend
    pub fn no_color_legend(mut self) -> Self {
        self.color = Some(LegendGuide {
            position: LegendPosition::None,
            ..Default::default()
        });
        self
    }

    /// Suppress the shape legend
    pub fn no_shape_legend(mut self) -> Self {
        self.shape = Some(LegendGuide {
            position: LegendPosition::None,
            ..Default::default()
        });
        self
    }

    /// Suppress the size legend
    pub fn no_size_legend(mut self) -> Self {
        self.size = Some(LegendGuide {
            position: LegendPosition::None,
            ..Default::default()
        });
        self
    }

    /// Suppress the alpha legend
    pub fn no_alpha_legend(mut self) -> Self {
        self.alpha = Some(LegendGuide {
            position: LegendPosition::None,
            ..Default::default()
        });
        self
    }
}
