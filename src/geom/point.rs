use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::builder::{
    AesMapBuilder, ColorContinuousAesBuilder, ColorDiscreteAesBuilder, SizeContinuousAesBuilder,
    SizeDiscreteAesBuilder, XContininuousAesBuilder, XDiscreteAesBuilder, YContininuousAesBuilder,
    YDiscreteAesBuilder,
};
use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain, AestheticProperty};
use crate::data::DiscreteValue;
use crate::error::PlotError;
use crate::geom::properties::{ColorProperty, FloatProperty, ShapeProperty};
use crate::geom::{AestheticRequirement, DomainConstraint};
use crate::layer::{Layer, LayerBuilder};
use crate::scale::ScaleIdentifier;
use crate::theme::{Color, color};
use crate::visuals::Shape;

pub trait GeomPointAesBuilderTrait:
    XContininuousAesBuilder
    + XDiscreteAesBuilder
    + YContininuousAesBuilder
    + YDiscreteAesBuilder
    + ColorContinuousAesBuilder
    + ColorDiscreteAesBuilder
    + SizeContinuousAesBuilder
    + SizeDiscreteAesBuilder
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
        if let Some(size_prop) = self.size {
            geom_point.size = size_prop;
            mapping.remove(&Aesthetic::Size(AestheticDomain::Continuous));
            mapping.remove(&Aesthetic::Size(AestheticDomain::Discrete));
        }
        if let Some(color_prop) = self.color {
            geom_point.color = color_prop;
            mapping.remove(&Aesthetic::Color(AestheticDomain::Continuous));
            mapping.remove(&Aesthetic::Color(AestheticDomain::Discrete));
        }
        if let Some(shape_prop) = self.shape {
            geom_point.shape = shape_prop;
            mapping.remove(&Aesthetic::Shape);
        }
        if let Some(alpha_prop) = self.alpha {
            geom_point.alpha = alpha_prop;
            mapping.remove(&Aesthetic::Alpha(AestheticDomain::Continuous));
            mapping.remove(&Aesthetic::Alpha(AestheticDomain::Discrete));
        }

        // Determine and validate aesthetic domains
        let requirements = geom_point.aesthetic_requirements();
        let aesthetic_domains = crate::layer::determine_aesthetic_domains(&mapping, requirements)
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
    /// Default point size (if not mapped)
    pub size: FloatProperty,

    /// Default point color (if not mapped)
    pub color: ColorProperty,

    /// Default point shape (if not mapped)
    pub shape: ShapeProperty,

    /// Default alpha/opacity (if not mapped)
    pub alpha: FloatProperty,
}

impl GeomPoint {
    /// Create a new point geom with default settings from theme
    pub fn new() -> Self {
        Self {
            size: FloatProperty::new(),
            color: ColorProperty::new(),
            shape: ShapeProperty::new(),
            alpha: FloatProperty::new(),
        }
    }

    /// Set the default point size
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size.value(size);
        self
    }

    /// Set the default point color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        self.color.color(color);
        self
    }

    /// Set the default point shape
    pub fn shape(&mut self, shape: Shape) -> &mut Self {
        self.shape.shape(shape);
        self
    }

    /// Set the default alpha/opacity
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha.value(alpha.clamp(0.0, 1.0));
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
        for ((((x_norm, y_norm), color), size), alpha) in x_values
            .zip(y_values)
            .zip(color_values)
            .zip(size_values)
            .zip(alpha_values)
        {
            // Convert normalized [0,1] to viewport pixel coordinates
            let x_px = ctx.map_x(x_norm);
            let y_px = ctx.map_y(y_norm);

            log::debug!(
                "Drawing point at data=({}, {}), norm=({}, {}), px=({}, {}), size={}, color={:?}, alpha={}",
                x_norm,
                y_norm,
                x_norm,
                y_norm,
                x_px,
                y_px,
                size,
                color,
                alpha
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

            // Draw circle
            ctx.cairo
                .arc(x_px, y_px, point_radius, 0.0, 2.0 * std::f64::consts::PI);
            ctx.cairo.fill().ok();
        }

        Ok(())
    }
}

impl Default for GeomPoint {
    fn default() -> Self {
        Self::new()
    }
}

const AESTHETIC_REQUIREMENTS: [AestheticRequirement; 5] = [
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
];

impl Geom for GeomPoint {
    fn aesthetic_requirements(&self) -> &'static [AestheticRequirement] {
        &AESTHETIC_REQUIREMENTS
    }

    fn required_scales(&self) -> Vec<ScaleIdentifier> {
        vec![ScaleIdentifier::XContinuous, ScaleIdentifier::YContinuous]
    }

    fn train_scales(&self, _scales: &mut crate::scale::ScaleSet) {}

    fn apply_scales(&mut self, _scales: &crate::scale::ScaleSet) {}

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        let data = ctx.layer.data(ctx.data());
        let mapping = ctx.layer.mapping(ctx.mapping());

        if mapping.contains(Aesthetic::Group) {
            let group_values = mapping.get_iter_discrete(&Aesthetic::Group, data).unwrap();
            let x_values = mapping
                .get_iter_float(&Aesthetic::X(AestheticDomain::Continuous), data)
                .unwrap();
            let y_values = mapping
                .get_iter_float(&Aesthetic::Y(AestheticDomain::Continuous), data)
                .unwrap();
            let color_values =
                self.color
                    .iter(data, mapping, Aesthetic::Color(AestheticDomain::Discrete))?;
            let size_values =
                self.size
                    .iter(data, mapping, Aesthetic::Size(AestheticDomain::Continuous))?;
            let alpha_values =
                self.alpha
                    .iter(data, mapping, Aesthetic::Alpha(AestheticDomain::Continuous))?;

            let mut groups: HashMap<
                DiscreteValue,
                (Vec<f64>, Vec<f64>, Vec<Color>, Vec<f64>, Vec<f64>),
            > = HashMap::new();
            for (((((group, x), y), color), size), alpha) in group_values
                .zip(x_values)
                .zip(y_values)
                .zip(color_values.map(Color::from))
                .zip(size_values)
                .zip(alpha_values)
            {
                let entry = groups.entry(group).or_insert((
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                    Vec::new(),
                ));
                entry.0.push(x);
                entry.1.push(y);
                entry.2.push(color);
                entry.3.push(size);
                entry.4.push(alpha);
            }
            let mut groups: Vec<_> = groups.into_iter().collect();
            groups.sort_by(|a, b| a.0.cmp(&b.0));
            for (_, (x_vals, y_vals, color_vals, size_vals, alpha_vals)) in groups {
                self.draw_points(
                    ctx,
                    x_vals.into_iter(),
                    y_vals.into_iter(),
                    color_vals.into_iter(),
                    size_vals.into_iter(),
                    alpha_vals.into_iter(),
                )?;
            }
        } else {
            // Get x and y values
            let x_values: Vec<f64> = mapping
                .get_iter_float(&Aesthetic::X(AestheticDomain::Continuous), data)
                .unwrap()
                .collect();
            let n = x_values.len();
            let y_values: Vec<f64> = mapping
                .get_iter_float(&Aesthetic::Y(AestheticDomain::Continuous), data)
                .unwrap()
                .collect();
            let color_values: Vec<Color> = self
                .color
                .iter(data, mapping, Aesthetic::Color(AestheticDomain::Continuous))?
                .take(n)
                .collect();
            let size_values: Vec<f64> = self
                .size
                .iter(data, mapping, Aesthetic::Size(AestheticDomain::Continuous))?
                .take(n)
                .collect();
            let alpha_values: Vec<f64> = self
                .alpha
                .iter(data, mapping, Aesthetic::Alpha(AestheticDomain::Continuous))?
                .take(n)
                .collect();
            self.draw_points(
                ctx,
                x_values.into_iter(),
                y_values.into_iter(),
                color_values.into_iter(),
                size_values.into_iter(),
                alpha_values.into_iter(),
            )?;
        }

        Ok(())
    }
}
