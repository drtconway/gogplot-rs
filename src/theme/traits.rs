use crate::{
    theme::{Color, FontStyle, FontWeight},
    visuals::{LineStyle, Shape},
};

/// Point Element trait
pub trait PointElement: Sized {
    fn this(&self) -> &super::PointElement;

    fn this_mut(&mut self) -> &mut super::PointElement;

    fn color<C: Into<Color>>(mut self, color: C) -> Self {
        self.this_mut().color = Some(color.into());
        self
    }

    fn size(mut self, size: f64) -> Self {
        self.this_mut().size = Some(size);
        self
    }

    fn shape<S: Into<Shape>>(mut self, shape: S) -> Self {
        self.this_mut().shape = Some(shape.into());
        self
    }

    fn alpha(mut self, alpha: f64) -> Self {
        self.this_mut().alpha = Some(alpha);
        self
    }
}

/// Line Element trait
pub trait LineElement: Sized {
    fn this(&self) -> &super::LineElement;

    fn this_mut(&mut self) -> &mut super::LineElement;

    fn color<C: Into<Color>>(mut self, color: C) -> Self {
        self.this_mut().color = Some(color.into());
        self
    }

    fn size(mut self, size: f64) -> Self {
        self.this_mut().size = Some(size);
        self
    }

    fn linestyle<S: Into<LineStyle>>(mut self, linestyle: S) -> Self {
        self.this_mut().linestyle = Some(linestyle.into());
        self
    }

    fn alpha(mut self, alpha: f64) -> Self {
        self.this_mut().alpha = Some(alpha);
        self
    }
}

/// Area Element trait
pub trait AreaElement: Sized {
    fn this(&self) -> &super::AreaElement;

    fn this_mut(&mut self) -> &mut super::AreaElement;

    fn fill<C: Into<Color>>(mut self, color: C) -> Self {
        self.this_mut().fill = Some(color.into());
        self
    }

    fn color<C: Into<Color>>(mut self, color: C) -> Self {
        self.this_mut().color = Some(color.into());
        self
    }

    fn size(mut self, size: f64) -> Self {
        self.this_mut().size = Some(size);
        self
    }

    fn linestyle<S: Into<LineStyle>>(mut self, linestyle: S) -> Self {
        self.this_mut().linestyle = Some(linestyle.into());
        self
    }

    fn alpha(mut self, alpha: f64) -> Self {
        self.this_mut().alpha = Some(alpha);
        self
    }
}

/// Text Element trait
pub trait TextElement: Sized {
    fn this(&self) -> &super::TextElement;

    fn this_mut(&mut self) -> &mut super::TextElement;

    fn color<C: Into<Color>>(mut self, color: C) -> Self {
        self.this_mut().color = Some(color.into());
        self
    }

    fn size(mut self, size: f64) -> Self {
        self.this_mut().size = Some(size);
        self
    }

    fn alpha(mut self, alpha: f64) -> Self {
        self.this_mut().alpha = Some(alpha);
        self
    }

    fn family<S: Into<String>>(mut self, family: S) -> Self {
        self.this_mut().family = Some(family.into());
        self
    }

    fn weight<W: Into<FontWeight>>(mut self, weight: W) -> Self {
        self.this_mut().weight = Some(weight.into());
        self
    }

    fn style<S: Into<FontStyle>>(mut self, style: S) -> Self {
        self.this_mut().style = Some(style.into());
        self
    }

    fn hjust(mut self, hjust: f64) -> Self {
        self.this_mut().hjust = Some(hjust);
        self
    }

    fn vjust(mut self, vjust: f64) -> Self {
        self.this_mut().vjust = Some(vjust);
        self
    }
}
