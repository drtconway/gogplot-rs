// Visual elements and symbols shared across geoms and guides

/// Shape types for points and legend symbols
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    Circle,
    Square,
    Triangle,
    Diamond,
    Cross,
    Plus,
}
