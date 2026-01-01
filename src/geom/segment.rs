use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::builder::{
    AesMapBuilder, AlphaContinuousAesBuilder, AlphaDiscreteAesBuilder, ColorContinuousAesBuilder,
    ColorDiscreteAesBuilder, SizeContinuousAesBuilder, SizeDiscreteAesBuilder, XBeginAesBuilder,
    XEndAesBuilder, YBeginAesBuilder, YEndAesBuilder,
};
use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain, AestheticProperty};
use crate::error::Result;
use crate::geom::properties::{ColorProperty, FloatProperty, PropertyVector};
use crate::geom::{AestheticRequirement, DomainConstraint};
use crate::layer::{Layer, LayerBuilder, LayerBuilderCore};
use crate::scale::ScaleIdentifier;
use crate::theme::{color, Color};

pub trait GeomSegmentAesBuilderTrait:
    XBeginAesBuilder
    + XEndAesBuilder
    + YBeginAesBuilder
    + YEndAesBuilder
    + ColorContinuousAesBuilder
    + ColorDiscreteAesBuilder
    + AlphaContinuousAesBuilder
    + AlphaDiscreteAesBuilder
    + SizeContinuousAesBuilder
    + SizeDiscreteAesBuilder
{
}

impl GeomSegmentAesBuilderTrait for AesMapBuilder {}

pub struct GeomSegmentBuilder {
    core: LayerBuilderCore,
    size: Option<FloatProperty>,
    color: Option<ColorProperty>,
    alpha: Option<FloatProperty>,
}

impl GeomSegmentBuilder {
    pub fn new() -> Self {
        Self {
            core: LayerBuilderCore::default(),
            size: None,
            color: None,
            alpha: None,
        }
    }

    pub fn size<Size: Into<FloatProperty>>(mut self, size: Size) -> Self {
        self.size = Some(size.into());
        self
    }

    pub fn color<Color: Into<ColorProperty>>(mut self, color: Color) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn alpha<Alpha: Into<FloatProperty>>(mut self, alpha: Alpha) -> Self {
        self.alpha = Some(alpha.into());
        self
    }

    pub fn aes(mut self, closure: impl FnOnce(&mut dyn GeomSegmentAesBuilderTrait)) -> Self {
        if self.core.stat.is_none() {
            if self.core.aes_builder.is_none() {
                self.core.aes_builder = Some(AesMapBuilder::new());
            }
            closure(self.core.aes_builder.as_mut().unwrap());
        } else {
            if self.core.after_aes_builder.is_none() {
                self.core.after_aes_builder = Some(AesMapBuilder::new());
            }
            closure(self.core.after_aes_builder.as_mut().unwrap());
        }
        self
    }
}

impl LayerBuilder for GeomSegmentBuilder {
    fn build(self: Box<Self>, parent_mapping: &AesMap) -> Result<Layer> {
        let mut geom_segment = GeomSegment::new();

        // Build the mapping (merging layer + parent)
        let mut overrides = Vec::new();

        // Set fixed property values and remove from inherited mapping
        if self.size.is_some() {
            geom_segment.size = self.size;
            overrides.push(Aesthetic::Size(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Size(AestheticDomain::Discrete));
        }
        if self.color.is_some() {
            geom_segment.color = self.color;
            overrides.push(Aesthetic::Color(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Color(AestheticDomain::Discrete));
        }
        if self.alpha.is_some() {
            geom_segment.alpha = self.alpha;
            overrides.push(Aesthetic::Alpha(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Alpha(AestheticDomain::Discrete));
        }

        LayerBuilderCore::build(
            self.core,
            parent_mapping,
            Box::new(geom_segment),
            HashMap::new(),
            &overrides,
        )
    }
}

pub fn geom_segment() -> GeomSegmentBuilder {
    GeomSegmentBuilder::new()
}

/// Geometry for drawing line segments.
///
/// Draws line segments from (xbegin, ybegin) to (xend, yend). Each segment can have
/// its own color, alpha, and size (line width).
///
/// # Required Aesthetics
///
/// - `XBegin`: Starting x coordinate
/// - `YBegin`: Starting y coordinate  
/// - `XEnd`: Ending x coordinate
/// - `YEnd`: Ending y coordinate
///
/// # Optional Aesthetics
///
/// - `Color`: Line color (can be constant or mapped to data)
/// - `Alpha`: Line transparency (0.0 = transparent, 1.0 = opaque)
/// - `Size`: Line width in pixels
pub struct GeomSegment {
    size: Option<FloatProperty>,
    color: Option<ColorProperty>,
    alpha: Option<FloatProperty>,
}

impl GeomSegment {
    /// Create a new segment geom with default settings
    pub fn new() -> Self {
        Self {
            size: None,
            color: None,
            alpha: None,
        }
    }

    fn draw_segments(
        &self,
        ctx: &mut RenderContext,
        xbegin_values: &[f64],
        ybegin_values: &[f64],
        xend_values: &[f64],
        yend_values: &[f64],
        color_values: &[Color],
        size_values: &[f64],
        alpha_values: &[f64],
    ) -> Result<()> {
        for i in 0..xbegin_values.len() {
            let xbegin = xbegin_values[i];
            let ybegin = ybegin_values[i];
            let xend = xend_values[i];
            let yend = yend_values[i];
            let color = color_values[i];
            let size = size_values[i];
            let alpha = alpha_values[i];

            // Set color and alpha
            let Color(r, g, b, a) = color;
            ctx.cairo.set_source_rgba(
                r as f64 / 255.0,
                g as f64 / 255.0,
                b as f64 / 255.0,
                a as f64 / 255.0 * alpha,
            );

            // Set line width
            ctx.cairo.set_line_width(size);

            // Draw the segment
            let x_begin_px = ctx.map_x(xbegin);
            let y_begin_px = ctx.map_y(ybegin);
            let x_end_px = ctx.map_x(xend);
            let y_end_px = ctx.map_y(yend);

            ctx.cairo.move_to(x_begin_px, y_begin_px);
            ctx.cairo.line_to(x_end_px, y_end_px);
            ctx.cairo.stroke().ok();
        }

        Ok(())
    }
}

impl Default for GeomSegment {
    fn default() -> Self {
        Self::new()
    }
}

const AESTHETIC_REQUIREMENTS: [AestheticRequirement; 7] = [
    AestheticRequirement {
        property: AestheticProperty::XBegin,
        required: true,
        constraint: DomainConstraint::Any,
    },
    AestheticRequirement {
        property: AestheticProperty::YBegin,
        required: true,
        constraint: DomainConstraint::Any,
    },
    AestheticRequirement {
        property: AestheticProperty::XEnd,
        required: true,
        constraint: DomainConstraint::Any,
    },
    AestheticRequirement {
        property: AestheticProperty::YEnd,
        required: true,
        constraint: DomainConstraint::Any,
    },
    AestheticRequirement {
        property: AestheticProperty::Color,
        required: false,
        constraint: DomainConstraint::Any,
    },
    AestheticRequirement {
        property: AestheticProperty::Size,
        required: false,
        constraint: DomainConstraint::Any,
    },
    AestheticRequirement {
        property: AestheticProperty::Alpha,
        required: false,
        constraint: DomainConstraint::Any,
    },
];

impl Geom for GeomSegment {
    fn aesthetic_requirements(&self) -> &'static [AestheticRequirement] {
        &AESTHETIC_REQUIREMENTS
    }

    fn properties(&self) -> HashMap<AestheticProperty, super::properties::Property> {
        let mut props = HashMap::new();
        if let Some(size_prop) = &self.size {
            props.insert(
                AestheticProperty::Size,
                super::properties::Property::Float(size_prop.clone()),
            );
        }
        if let Some(color_prop) = &self.color {
            props.insert(
                AestheticProperty::Color,
                super::properties::Property::Color(color_prop.clone()),
            );
        }
        if let Some(alpha_prop) = &self.alpha {
            props.insert(
                AestheticProperty::Alpha,
                super::properties::Property::Float(alpha_prop.clone()),
            );
        }
        props
    }

    fn property_defaults(
        &self,
        _theme: &crate::prelude::Theme,
    ) -> HashMap<AestheticProperty, super::properties::PropertyValue> {
        let mut defaults = HashMap::new();
        if self.size.is_none() {
            defaults.insert(
                AestheticProperty::Size,
                super::properties::PropertyValue::Float(1.0),
            );
        }
        if self.color.is_none() {
            defaults.insert(
                AestheticProperty::Color,
                super::properties::PropertyValue::Color(color::BLACK),
            );
        }
        if self.alpha.is_none() {
            defaults.insert(
                AestheticProperty::Alpha,
                super::properties::PropertyValue::Float(1.0),
            );
        }
        defaults
    }

    fn required_scales(&self) -> Vec<ScaleIdentifier> {
        vec![ScaleIdentifier::XContinuous, ScaleIdentifier::YContinuous]
    }

    fn train_scales(&self, _scales: &mut crate::scale::ScaleSet) {}

    fn apply_scales(&mut self, _scales: &crate::scale::ScaleSet) {}

    fn render(
        &self,
        ctx: &mut RenderContext,
        mut properties: HashMap<AestheticProperty, PropertyVector>,
    ) -> Result<()> {
        let xbegin_values = properties
            .remove(&AestheticProperty::XBegin)
            .unwrap()
            .as_floats();
        let ybegin_values = properties
            .remove(&AestheticProperty::YBegin)
            .unwrap()
            .as_floats();
        let xend_values = properties
            .remove(&AestheticProperty::XEnd)
            .unwrap()
            .as_floats();
        let yend_values = properties
            .remove(&AestheticProperty::YEnd)
            .unwrap()
            .as_floats();
        let color_values = properties
            .remove(&AestheticProperty::Color)
            .unwrap()
            .as_colors();
        let size_values = properties
            .remove(&AestheticProperty::Size)
            .unwrap()
            .as_floats();
        let alpha_values = properties
            .remove(&AestheticProperty::Alpha)
            .unwrap()
            .as_floats();

        self.draw_segments(
            ctx,
            &xbegin_values,
            &ybegin_values,
            &xend_values,
            &yend_values,
            &color_values,
            &size_values,
            &alpha_values,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::VectorValue;
    use crate::error::to_io_error;
    use crate::plot::plot;
    use crate::theme::color;
    use crate::data::DataSource;
    use crate::utils::dataframe::DataFrame;

    fn init_test_logging() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn basic_segment_1() {
        init_test_logging();

        let x1 = vec![1.0, 2.0, 3.0, 4.0];
        let y1 = vec![1.0, 2.0, 1.5, 3.0];
        let x2 = vec![2.0, 3.0, 4.0, 5.0];
        let y2 = vec![2.0, 3.0, 3.5, 4.0];

        let data: Box<dyn DataSource> = Box::new(DataFrame::from_columns(vec![
            ("x1", VectorValue::from(x1)),
            ("y1", VectorValue::from(y1)),
            ("x2", VectorValue::from(x2)),
            ("y2", VectorValue::from(y2)),
        ]));

        let builder = plot(&data).aes(|a| {
            a.xbegin("x1");
            a.ybegin("y1");
            a.xend("x2");
            a.yend("y2");
        }) + geom_segment().size(2.0).color(color::BLUE);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_segment_1.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_segment_2() {
        init_test_logging();

        // Create a grid of arrows pointing in different directions
        let x1 = vec![1.0, 2.0, 3.0, 1.0, 2.0, 3.0, 1.0, 2.0, 3.0];
        let y1 = vec![1.0, 1.0, 1.0, 2.0, 2.0, 2.0, 3.0, 3.0, 3.0];
        let x2 = vec![1.5, 2.5, 3.5, 1.5, 2.5, 3.5, 1.5, 2.5, 3.5];
        let y2 = vec![1.5, 1.5, 1.5, 2.5, 2.5, 2.5, 3.5, 3.5, 3.5];

        let data: Box<dyn DataSource> = Box::new(DataFrame::from_columns(vec![
            ("x1", VectorValue::from(x1)),
            ("y1", VectorValue::from(y1)),
            ("x2", VectorValue::from(x2)),
            ("y2", VectorValue::from(y2)),
        ]));

        let builder = plot(&data).aes(|a| {
            a.xbegin("x1");
            a.ybegin("y1");
            a.xend("x2");
            a.yend("y2");
        }) + geom_segment().size(3.0).color(color::RED).alpha(0.7);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_segment_2.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_segment_3() {
        init_test_logging();

        // Star pattern from center
        let center_x = 3.0;
        let center_y = 3.0;
        let x1 = vec![center_x; 8];
        let y1 = vec![center_y; 8];
        let angles: Vec<f64> = (0..8).map(|i| i as f64 * std::f64::consts::PI / 4.0).collect();
        let x2: Vec<f64> = angles.iter().map(|a| center_x + a.cos() * 1.5).collect();
        let y2: Vec<f64> = angles.iter().map(|a| center_y + a.sin() * 1.5).collect();

        let data: Box<dyn DataSource> = Box::new(DataFrame::from_columns(vec![
            ("x1", VectorValue::from(x1)),
            ("y1", VectorValue::from(y1)),
            ("x2", VectorValue::from(x2)),
            ("y2", VectorValue::from(y2)),
        ]));

        let builder = plot(&data).aes(|a| {
            a.xbegin("x1");
            a.ybegin("y1");
            a.xend("x2");
            a.yend("y2");
        }) + geom_segment().size(4.0).color(color::DARKGREEN);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_segment_3.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_segment_4() {
        init_test_logging();

        // Segments with varying sizes
        let x1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y1 = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let x2 = vec![1.5, 2.5, 3.5, 4.5, 5.5];
        let y2 = vec![2.0, 3.0, 4.0, 5.0, 6.0];
        let sizes = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        let data: Box<dyn DataSource> = Box::new(DataFrame::from_columns(vec![
            ("x1", VectorValue::from(x1)),
            ("y1", VectorValue::from(y1)),
            ("x2", VectorValue::from(x2)),
            ("y2", VectorValue::from(y2)),
            ("size", VectorValue::from(sizes)),
        ]));

        let builder = plot(&data).aes(|a| {
            a.xbegin("x1");
            a.ybegin("y1");
            a.xend("x2");
            a.yend("y2");
        }) + geom_segment().color(color::PURPLE).aes(|a| {
            a.size_continuous("size");
        });

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_segment_4.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_segment_5() {
        init_test_logging();

        // Segments with different colors and alphas
        let x1 = vec![1.0, 2.0, 3.0, 4.0];
        let y1 = vec![1.0, 2.0, 3.0, 4.0];
        let x2 = vec![2.0, 3.0, 4.0, 5.0];
        let y2 = vec![3.0, 4.0, 5.0, 6.0];
        let groups = vec!["A", "B", "C", "D"];

        let data: Box<dyn DataSource> = Box::new(DataFrame::from_columns(vec![
            ("x1", VectorValue::from(x1)),
            ("y1", VectorValue::from(y1)),
            ("x2", VectorValue::from(x2)),
            ("y2", VectorValue::from(y2)),
            ("group", VectorValue::from(groups)),
        ]));

        let builder = plot(&data).aes(|a| {
            a.xbegin("x1");
            a.ybegin("y1");
            a.xend("x2");
            a.yend("y2");
        }) + geom_segment().size(3.0).aes(|a| {
            a.color_discrete("group");
            a.alpha_discrete("group");
        });

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_segment_5.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }
}
