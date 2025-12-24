use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::builder::{
    AesMapBuilder, ColorContinuousAesBuilder, ColorDiscreteAesBuilder, SizeContinuousAesBuilder,
    SizeDiscreteAesBuilder, XContininuousAesBuilder, XDiscreteAesBuilder, YContininuousAesBuilder,
    YDiscreteAesBuilder,
};
use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain, AestheticProperty};
use crate::error::PlotError;
use crate::geom::properties::{ColorProperty, FloatProperty, PropertyVector, ShapeProperty};
use crate::geom::{AestheticRequirement, DomainConstraint};
use crate::layer::{Layer, LayerBuilder};
use crate::scale::ScaleIdentifier;
use crate::theme::Color;
use crate::visuals::Shape;

pub trait GeomLineAesBuilderTrait:
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

impl GeomLineAesBuilderTrait for AesMapBuilder {}

pub struct GeomLineBuilder {
    size: Option<FloatProperty>,
    color: Option<ColorProperty>,
    shape: Option<ShapeProperty>,
    alpha: Option<FloatProperty>,
    aes_builder: AesMapBuilder,
}

impl GeomLineBuilder {
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

    pub fn aes(mut self, closure: impl FnOnce(&mut dyn GeomLineAesBuilderTrait)) -> Self {
        closure(&mut self.aes_builder);
        self
    }
}

impl LayerBuilder for GeomLineBuilder {
    fn build(self: Box<Self>, parent_mapping: &AesMap) -> Layer {
        let mut geom_point = GeomLine::new();

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

pub fn geom_point() -> GeomLineBuilder {
    GeomLineBuilder::new()
}

/// GeomLine renders points/scatterplot
pub struct GeomLine {
    /// Default point size (if not mapped)
    pub size: FloatProperty,

    /// Default point color (if not mapped)
    pub color: ColorProperty,

    /// Default point shape (if not mapped)
    pub shape: ShapeProperty,

    /// Default alpha/opacity (if not mapped)
    pub alpha: FloatProperty,
}

impl GeomLine {
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

    fn draw_lines(
        &self,
        ctx: &mut RenderContext,
        x_values: impl Iterator<Item = f64>,
        y_values: impl Iterator<Item = f64>,
        color_values: impl Iterator<Item = Color>,
        size_values: impl Iterator<Item = f64>,
        alpha_values: impl Iterator<Item = f64>,
    ) -> Result<(), PlotError> {
        // Collect all points for this line segment
        let points: Vec<_> = x_values
            .zip(y_values)
            .zip(color_values)
            .zip(size_values)
            .zip(alpha_values)
            .collect();

        if points.is_empty() {
            return Ok(());
        }

        // For lines, we'll use the first point's aesthetics for the entire line
        // (In ggplot2, lines typically have constant color/size per group)
        let ((((_, _), first_color), first_size), first_alpha) = points[0];

        let Color(r, g, b, a) = first_color;
        ctx.cairo.set_source_rgba(
            r as f64 / 255.0,
            g as f64 / 255.0,
            b as f64 / 255.0,
            a as f64 / 255.0 * first_alpha,
        );

        // Set line width (size)
        ctx.cairo.set_line_width(first_size);

        // Start the path at the first point
        let ((((x_norm, y_norm), _), _), _) = points[0];
        let x_px = ctx.map_x(x_norm);
        let y_px = ctx.map_y(y_norm);
        ctx.cairo.move_to(x_px, y_px);

        // Draw lines to subsequent points
        for ((((x_norm, y_norm), _), _), _) in points.iter().skip(1) {
            let x_px = ctx.map_x(*x_norm);
            let y_px = ctx.map_y(*y_norm);
            ctx.cairo.line_to(x_px, y_px);
        }

        // Stroke the path
        ctx.cairo.stroke().ok();

        Ok(())
    }
}

impl Default for GeomLine {
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

impl Geom for GeomLine {
    fn aesthetic_requirements(&self) -> &'static [AestheticRequirement] {
        &AESTHETIC_REQUIREMENTS
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
        let x_values = properties
            .remove(&AestheticProperty::X)
            .unwrap()
            .as_floats();
        let y_values = properties
            .remove(&AestheticProperty::Y)
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

        // Create a permutation iterator to sort points by x-value
        let mut indices: Vec<usize> = (0..x_values.len()).collect();
        indices.sort_by(|&i, &j| x_values[i].partial_cmp(&x_values[j]).unwrap());

        let x_values = indices.iter().map(|&i| x_values[i]);
        let y_values = indices.iter().map(|&i| y_values[i]);
        let color_values = indices.iter().map(|&i| color_values[i]);
        let size_values = indices.iter().map(|&i| size_values[i]);
        let alpha_values = indices.iter().map(|&i| alpha_values[i]);

        self.draw_lines(
            ctx,
            x_values,
            y_values,
            color_values,
            size_values,
            alpha_values,
        )
    }
}
