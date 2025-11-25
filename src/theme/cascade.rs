

pub struct ThemePoint {
    pub size: f64,
    pub color: i32,
    pub shape: i32,
    pub alpha: f64,
}

pub struct ThemeText {
    pub family: String,
    pub face: String,
    pub color: i32,
    pub size: f64,
    pub hjust: f64,
    pub vjust: f64,
}

pub struct ThemeLine {
    pub color: i32,
    pub size: f64,
    pub linetype: String,
}

pub struct ThemeRect {
    pub fill: i32,
    pub alpha: f64,
    pub line: Option<ThemeLine>,
}

pub struct ThemeAxis {
    pub title: Option<ThemeText>,
    pub text: Option<ThemeText>,
    pub line: Option<ThemeLine>,
    pub ticks: Option<ThemeLine>,
}

pub struct ThemeGeomPoint {
    pub point: Option<ThemePoint>,
}

pub struct ThemeGeomLine {
    pub line: Option<ThemeLine>,
}

pub struct ThemeGeomSegment {
    pub line: Option<ThemeLine>,
}

pub struct ThemeGeomBoxplot {
    pub line: Option<ThemeLine>,
    pub rect: Option<ThemeRect>,}

pub struct PlotTheme {
    pub point: ThemePoint,
    pub text: ThemeText,
    pub line: ThemeLine,
    pub axis: ThemeAxis,
    pub x_axis: Option<ThemeAxis>,
    pub y_axis: Option<ThemeAxis>,
    pub geom_point: ThemeGeomPoint,
    pub geom_line: ThemeGeomLine,
    pub geom_segment: ThemeGeomSegment,
    pub geom_boxplot: ThemeGeomBoxplot,
}

pub trait ThemeElement {
    fn provide_point(&self) -> Option<&ThemePoint> {
        None
    }

    fn provide_text(&self) -> Option<&ThemeText> {
        None
    }

    fn provide_line(&self) -> Option<&ThemeLine> {
        None
    }
}

pub struct CascadeAccessor<'a> {
    stack: Vec<&'a dyn ThemeElement>,
}

impl<'a> CascadeAccessor<'a> {
    pub fn new(root: &'a dyn ThemeElement) -> Self {
        Self { stack: vec![root] }
    }


    pub fn cons(&self, element: &'a dyn ThemeElement) -> Self {
        let mut stack = self.stack.clone();
        stack.push(element);
        Self { stack }
    }

    pub fn point(&self) -> Option<&'a ThemePoint> {
        for element in self.stack.iter().rev() {
            if let Some(point) = element.provide_point() {
                return Some(point);
            }
        }
        None
    }

    pub fn text(&self) -> Option<&'a ThemeText> {
        for element in self.stack.iter().rev() {
            if let Some(text) = element.provide_text() {
                return Some(text);
            }
        }
        None
    }

    pub fn line(&self) -> Option<&'a ThemeLine> {
        for element in self.stack.iter().rev() {
            if let Some(line) = element.provide_line() {
                return Some(line);
            }
        }
        None
    }
}