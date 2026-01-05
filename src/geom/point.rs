use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::builder::{
    AesMapBuilder, AlphaContinuousAesBuilder, AlphaDiscreteAesBuilder, ColorContinuousAesBuilder,
    ColorDiscreteAesBuilder, ShapeAesBuilder, SizeContinuousAesBuilder, SizeDiscreteAesBuilder,
    XContinuousAesBuilder, XDiscreteAesBuilder, YContinuousAesBuilder, YDiscreteAesBuilder,
};
use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain, AestheticProperty};
use crate::error::Result;
use crate::geom::properties::{PropertyValue, PropertyVector};
use crate::geom::{AestheticRequirement, DomainConstraint};
use crate::layer::{Layer, LayerBuilder, LayerBuilderCore};
use crate::scale::ScaleIdentifier;
use crate::theme::{Color, Theme, color};
use crate::visuals::Shape;

pub trait GeomPointAesBuilderTrait:
    XContinuousAesBuilder
    + XDiscreteAesBuilder
    + YContinuousAesBuilder
    + YDiscreteAesBuilder
    + ColorContinuousAesBuilder
    + ColorDiscreteAesBuilder
    + AlphaContinuousAesBuilder
    + AlphaDiscreteAesBuilder
    + SizeContinuousAesBuilder
    + SizeDiscreteAesBuilder
    + ShapeAesBuilder
{
}

impl GeomPointAesBuilderTrait for AesMapBuilder {}

pub struct GeomPointBuilder {
    core: LayerBuilderCore,
    size: Option<f64>,
    color: Option<Color>,
    shape: Option<Shape>,
    alpha: Option<f64>,
}

impl GeomPointBuilder {
    pub fn new() -> Self {
        Self {
            core: LayerBuilderCore::default(),
            size: None,
            color: None,
            shape: None,
            alpha: None,
        }
    }

    pub fn size(mut self, size: f64) -> Self {
        self.size = Some(size);
        self
    }

    pub fn color(mut self, color: Color) -> Self {
        self.color = Some(color);
        self
    }

    pub fn shape(mut self, shape: Shape) -> Self {
        self.shape = Some(shape);
        self
    }

    pub fn alpha(mut self, alpha: f64) -> Self {
        self.alpha = Some(alpha);
        self
    }

    pub fn aes(mut self, closure: impl FnOnce(&mut dyn GeomPointAesBuilderTrait)) -> Self {
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

impl LayerBuilder for GeomPointBuilder {
    fn build(self: Box<Self>, parent_mapping: &AesMap) -> Result<Layer> {
        let mut geom_point = GeomPoint::new();

        let mut overrides = Vec::new();

        // Set fixed property values and remove from inherited mapping
        if self.size.is_some() {
            geom_point.size = self.size;
            overrides.push(Aesthetic::Size(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Size(AestheticDomain::Discrete));
        }
        if self.color.is_some() {
            geom_point.color = self.color;
            overrides.push(Aesthetic::Color(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Color(AestheticDomain::Discrete));
        }
        if self.shape.is_some() {
            geom_point.shape = self.shape;
            overrides.push(Aesthetic::Shape);
        }
        if self.alpha.is_some() {
            geom_point.alpha = self.alpha;
            overrides.push(Aesthetic::Alpha(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Alpha(AestheticDomain::Discrete));
        }

        LayerBuilderCore::build(
            self.core,
            parent_mapping,
            Box::new(geom_point),
            HashMap::new(),
            &overrides,
        )
    }
}

pub fn geom_point() -> GeomPointBuilder {
    GeomPointBuilder::new()
}

/// GeomPoint renders points/scatterplot
pub struct GeomPoint {
    size: Option<f64>,
    color: Option<Color>,
    shape: Option<Shape>,
    alpha: Option<f64>,
}

impl GeomPoint {
    /// Create a new point geom with default settings from theme
    pub fn new() -> Self {
        Self {
            size: None,
            color: None,
            shape: None,
            alpha: None,
        }
    }

    fn draw_points(
        &self,
        ctx: &mut RenderContext,
        x_values: impl Iterator<Item = f64>,
        y_values: impl Iterator<Item = f64>,
        color_values: impl Iterator<Item = Color>,
        size_values: impl Iterator<Item = f64>,
        alpha_values: impl Iterator<Item = f64>,
        shape_values: impl Iterator<Item = Shape>,
    ) -> Result<()> {
        // Set default color
        let Color(r, g, b, a) = color::BLACK; // Placeholder for now
        ctx.cairo.set_source_rgba(
            r as f64 / 255.0,
            g as f64 / 255.0,
            b as f64 / 255.0,
            a as f64 / 255.0,
        );

        // Data values are already normalized [0,1] by compose()->apply_scales()
        // Just map to viewport pixel coordinates
        for (((((x_norm, y_norm), color), size), alpha), shape) in x_values
            .zip(y_values)
            .zip(color_values)
            .zip(size_values)
            .zip(alpha_values)
            .zip(shape_values)
        {
            // Convert normalized [0,1] to viewport pixel coordinates
            let x_px = ctx.map_x(x_norm);
            let y_px = ctx.map_y(y_norm);

            log::debug!(
                "Drawing point at data=({}, {}), norm=({}, {}), px=({}, {}), size={}, color={:?}, alpha={}, shape={:?}",
                x_norm,
                y_norm,
                x_norm,
                y_norm,
                x_px,
                y_px,
                size,
                color,
                alpha,
                shape
            );

            let Color(r, g, b, a) = color;
            ctx.cairo.set_source_rgba(
                r as f64 / 255.0,
                g as f64 / 255.0,
                b as f64 / 255.0,
                a as f64 / 255.0 * alpha,
            );

            // Size is already a radius value from the scale (default range 1.0-6.0)
            let point_radius = size;

            // Draw shape at position
            shape.draw(&mut ctx.cairo, x_px, y_px, point_radius);
        }

        Ok(())
    }
}

const AESTHETIC_REQUIREMENTS: [AestheticRequirement; 6] = [
    AestheticRequirement {
        property: AestheticProperty::X,
        required: true,
        constraint: DomainConstraint::Any,
    },
    AestheticRequirement {
        property: AestheticProperty::Y,
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
    AestheticRequirement {
        property: AestheticProperty::Shape,
        required: false,
        constraint: DomainConstraint::MustBe(AestheticDomain::Discrete),
    },
];

impl Geom for GeomPoint {
    fn aesthetic_requirements(&self) -> &'static [AestheticRequirement] {
        &AESTHETIC_REQUIREMENTS
    }

    fn properties(&self) -> HashMap<AestheticProperty, super::properties::Property> {
        let mut props = HashMap::new();
        if let Some(size) = self.size {
            props.insert(
                AestheticProperty::Size,
                super::properties::Property::Float(size),
            );
        }
        if let Some(color) = self.color {
            props.insert(
                AestheticProperty::Color,
                super::properties::Property::Color(color),
            );
        }
        if let Some(shape) = self.shape {
            props.insert(
                AestheticProperty::Shape,
                super::properties::Property::Shape(shape),
            );
        }
        if let Some(alpha) = self.alpha {
            props.insert(
                AestheticProperty::Alpha,
                super::properties::Property::Float(alpha),
            );
        }
        props
    }

    fn property_defaults(&self, theme: &Theme) -> HashMap<AestheticProperty, PropertyValue> {
        let mut defaults = HashMap::new();

        // Start with hardcoded defaults
        let mut default_size = 3.0;
        let mut default_color = color::BLACK;
        let mut default_shape = Shape::Circle;
        let mut default_alpha = 1.0;

        // Apply theme overrides if present
        if let Some(crate::theme::Element::Point(elem)) = theme.get_element("point", "point") {
            if let Some(size) = elem.size {
                default_size = size;
            }
            if let Some(color) = elem.color {
                default_color = color;
            }
            if let Some(shape) = elem.shape {
                default_shape = shape;
            }
            if let Some(alpha) = elem.alpha {
                default_alpha = alpha;
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
        if self.shape.is_none() {
            defaults.insert(
                AestheticProperty::Shape,
                PropertyValue::Shape(default_shape),
            );
        }
        if self.alpha.is_none() {
            defaults.insert(
                AestheticProperty::Alpha,
                PropertyValue::Float(default_alpha),
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
        let x_values = properties
            .remove(&AestheticProperty::X)
            .unwrap()
            .as_floats();
        let y_values = properties
            .remove(&AestheticProperty::Y)
            .unwrap()
            .as_floats();

        // Extract color - need to convert Int to Color ONLY for color property
        let color_prop = properties.remove(&AestheticProperty::Color).unwrap();
        let color_values = color_prop.to_color().as_colors();

        let size_values = properties
            .remove(&AestheticProperty::Size)
            .unwrap()
            .as_floats();
        let alpha_values = properties
            .remove(&AestheticProperty::Alpha)
            .unwrap()
            .as_floats();

        // Extract shape - need to convert Int to Shape ONLY for shape property
        let shape_prop = properties.remove(&AestheticProperty::Shape).unwrap();
        let shape_values = shape_prop.to_shape().as_shapes();

        self.draw_points(
            ctx,
            x_values.into_iter(),
            y_values.into_iter(),
            color_values.into_iter(),
            size_values.into_iter(),
            alpha_values.into_iter(),
            shape_values.into_iter(),
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::to_io_error, plot::plot, utils::mtcars::mtcars};

    fn init_test_logging() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn basic_points_1() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
            a.y_continuous("mpg");
        }) + geom_point().size(3.0).alpha(0.5);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_points_1.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_points_2() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
            a.y_continuous("mpg");
        }) + geom_point().color(color::BLUEVIOLET).shape(Shape::Square);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_points_2.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_points_3() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
            a.y_continuous("mpg");
        }) + geom_point().aes(|a| {
            a.color_continuous("hp");
            a.size_discrete("cyl");
        });

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_points_3.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }
}
