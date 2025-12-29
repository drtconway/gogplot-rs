use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::builder::{
    AesMapBuilder, AlphaContinuousAesBuilder, AlphaDiscreteAesBuilder, ColorContinuousAesBuilder, ColorDiscreteAesBuilder, ShapeAesBuilder, SizeContinuousAesBuilder, SizeDiscreteAesBuilder, XContininuousAesBuilder, XDiscreteAesBuilder, YContininuousAesBuilder, YDiscreteAesBuilder
};
use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain, AestheticProperty};
use crate::error::PlotError;
use crate::geom::properties::{ColorProperty, FloatProperty, PropertyValue, PropertyVector, ShapeProperty};
use crate::geom::{AestheticRequirement, DomainConstraint};
use crate::layer::{Layer, LayerBuilder};
use crate::scale::ScaleIdentifier;
use crate::theme::{Color, Theme, color};
use crate::visuals::Shape;

pub trait GeomPointAesBuilderTrait:
    XContininuousAesBuilder
    + XDiscreteAesBuilder
    + YContininuousAesBuilder
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
    size: Option<FloatProperty>,
    color: Option<ColorProperty>,
    shape: Option<ShapeProperty>,
    alpha: Option<FloatProperty>,
    aes_builder: AesMapBuilder,
}

impl GeomPointBuilder {
    pub fn new() -> Self {
        Self {
            size: None,
            color: None,
            shape: None,
            alpha: None,
            aes_builder: AesMapBuilder::new(),
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

    pub fn shape<Shape: Into<ShapeProperty>>(mut self, shape: Shape) -> Self {
        self.shape = Some(shape.into());
        self
    }

    pub fn alpha<Alpha: Into<FloatProperty>>(mut self, alpha: Alpha) -> Self {
        self.alpha = Some(alpha.into());
        self
    }

    pub fn aes(mut self, closure: impl FnOnce(&mut dyn GeomPointAesBuilderTrait)) -> Self {
        closure(&mut self.aes_builder);
        self
    }
}

impl LayerBuilder for GeomPointBuilder {
    fn build(self: Box<Self>, parent_mapping: &AesMap) -> Layer {
        let mut geom_point = GeomPoint::new();

        // Build the mapping (merging layer + parent)
        let mut mapping = self.aes_builder.build(parent_mapping);

        // Set fixed property values and remove from inherited mapping
        if self.size.is_some() {
            geom_point.size = self.size;
            mapping.remove(&Aesthetic::Size(AestheticDomain::Continuous));
            mapping.remove(&Aesthetic::Size(AestheticDomain::Discrete));
        }
        if self.color.is_some() {
            geom_point.color = self.color;
            mapping.remove(&Aesthetic::Color(AestheticDomain::Continuous));
            mapping.remove(&Aesthetic::Color(AestheticDomain::Discrete));
        }
        if self.shape.is_some() {
            geom_point.shape = self.shape;
            mapping.remove(&Aesthetic::Shape);
        }
        if self.alpha.is_some() {
            geom_point.alpha = self.alpha;
            mapping.remove(&Aesthetic::Alpha(AestheticDomain::Continuous));
            mapping.remove(&Aesthetic::Alpha(AestheticDomain::Discrete));
        }

        // Determine and validate aesthetic domains
        let requirements = geom_point.aesthetic_requirements();
        let aesthetic_domains = crate::layer::determine_aesthetic_domains(&mapping, requirements, HashMap::new())
            .expect("Invalid aesthetic configuration for geom_point");

        // Create the layer
        let mut layer = crate::layer::Layer::new(Box::new(geom_point));
        layer.mapping = Some(mapping);
        layer.aesthetic_domains = aesthetic_domains;

        layer
    }
}

pub fn geom_point() -> GeomPointBuilder {
    GeomPointBuilder::new()
}

/// GeomPoint renders points/scatterplot
pub struct GeomPoint {
    size: Option<FloatProperty>,
    color: Option<ColorProperty>,
    shape: Option<ShapeProperty>,
    alpha: Option<FloatProperty>,
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

    /// Set the default point size
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size = Some(FloatProperty::new().value(size).clone());
        self
    }

    /// Set the default point color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        self.color = Some(ColorProperty::new().color(color).clone());
        self
    }

    /// Set the default point shape
    pub fn shape(&mut self, shape: Shape) -> &mut Self {
        self.shape = Some(ShapeProperty::new().shape(shape).clone());
        self
    }

    /// Set the default alpha/opacity
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = Some(FloatProperty::new().value(alpha.clamp(0.0, 1.0)).clone());
        self
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
    ) -> Result<(), PlotError> {
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

            log::info!(
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

impl Default for GeomPoint {
    fn default() -> Self {
        Self::new()
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
        if let Some(shape_prop) = &self.shape {
            props.insert(
                AestheticProperty::Shape,
                super::properties::Property::Shape(shape_prop.clone()),
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

    fn property_defaults(&self, _theme: &Theme) -> HashMap<AestheticProperty, PropertyValue> {
        let mut defaults = HashMap::new();
        if self.size.is_none() {
            defaults.insert(
                AestheticProperty::Size,
                PropertyValue::Float(3.0)
            );
        }
        if self.color.is_none() {
            defaults.insert(
                AestheticProperty::Color,
                PropertyValue::Color(color::BLACK)
            );
        }
        if self.shape.is_none() {
            defaults.insert(
                AestheticProperty::Shape,
                PropertyValue::Shape(Shape::Circle)
            );
        }
        if self.alpha.is_none() {
            defaults.insert(
                AestheticProperty::Alpha,
                PropertyValue::Float(1.0),
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
    ) -> Result<(), PlotError> {
        let props = properties.keys().cloned().collect::<Vec<_>>();
        log::info!("GeomPoint render with properties: {:?}", props);
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
        log::info!("Color property before conversion: {:?}", color_prop);
        let color_values = color_prop.to_color().as_colors();
        log::info!("Color values after conversion: {} values", color_values.len());
        
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
        log::info!("Shape property before conversion: {:?}", shape_prop);
        let shape_values = shape_prop.to_shape().as_shapes();
        log::info!("Shape values after conversion: {} values", shape_values.len());
        
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
