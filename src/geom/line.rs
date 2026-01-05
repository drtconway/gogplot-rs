use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::builder::{
    AesMapBuilder, AlphaContinuousAesBuilder, AlphaDiscreteAesBuilder, ColorContinuousAesBuilder,
    ColorDiscreteAesBuilder, SizeContinuousAesBuilder, SizeDiscreteAesBuilder,
    XContinuousAesBuilder, XDiscreteAesBuilder, YContinuousAesBuilder, YDiscreteAesBuilder,
};
use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain, AestheticProperty};
use crate::error::Result;
use crate::geom::properties::PropertyVector;
use crate::geom::{AestheticRequirement, DomainConstraint};
use crate::layer::{Layer, LayerBuilder, LayerBuilderCore};
use crate::scale::ScaleIdentifier;
use crate::theme::{Color, color};

pub trait GeomLineAesBuilderTrait:
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
{
}

impl GeomLineAesBuilderTrait for AesMapBuilder {}

pub struct GeomLineBuilder {
    core: LayerBuilderCore,
    size: Option<f64>,
    color: Option<Color>,
    alpha: Option<f64>,
}

impl GeomLineBuilder {
    pub fn new() -> Self {
        Self {
            core: LayerBuilderCore::default(),
            size: None,
            color: None,
            alpha: None,
        }
    }

    pub fn size<Size: Into<f64>>(mut self, size: Size) -> Self {
        self.size = Some(size.into());
        self
    }

    pub fn color<C: Into<Color>>(mut self, color: C) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn alpha<Alpha: Into<f64>>(mut self, alpha: Alpha) -> Self {
        self.alpha = Some(alpha.into());
        self
    }

    pub fn aes(mut self, closure: impl FnOnce(&mut dyn GeomLineAesBuilderTrait)) -> Self {
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

impl LayerBuilder for GeomLineBuilder {
    fn build(self: Box<Self>, parent_mapping: &AesMap) -> Result<Layer> {
        let mut geom_line = GeomLine::new();

        // Build the mapping (merging layer + parent)
        let mut overrides = Vec::new();

        // Set fixed property values and remove from inherited mapping
        if self.size.is_some() {
            geom_line.size = self.size;
            overrides.push(Aesthetic::Size(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Size(AestheticDomain::Discrete));
        }
        if self.color.is_some() {
            geom_line.color = self.color;
            overrides.push(Aesthetic::Color(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Color(AestheticDomain::Discrete));
        }
        if self.alpha.is_some() {
            geom_line.alpha = self.alpha;
            overrides.push(Aesthetic::Alpha(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Alpha(AestheticDomain::Discrete));
        }

        LayerBuilderCore::build(
            self.core,
            parent_mapping,
            Box::new(geom_line),
            HashMap::new(),
            &overrides,
        )
    }
}

pub fn geom_line() -> GeomLineBuilder {
    GeomLineBuilder::new()
}

/// GeomLine renders points/scatterplot
pub struct GeomLine {
    size: Option<f64>,
    color: Option<Color>,
    alpha: Option<f64>,
}

impl GeomLine {
    /// Create a new point geom with default settings from theme
    pub fn new() -> Self {
        Self {
            size: None,
            color: None,
            alpha: None,
        }
    }

    fn draw_lines(
        &self,
        ctx: &mut RenderContext,
        x_values: impl Iterator<Item = f64>,
        y_values: impl Iterator<Item = f64>,
        color_values: impl Iterator<Item = Color>,
        size_values: impl Iterator<Item = f64>,
        alpha_values: impl Iterator<Item = f64>,
    ) -> Result<()> {
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
    fn basic_lines_1() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
            a.y_continuous("mpg");
        }) + geom_line().size(3.0).alpha(0.5);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_lines_1.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_lines_2() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
            a.y_continuous("mpg");
        }) + geom_line().color(color::BLUEVIOLET);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_lines_2.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_lines_3() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
            a.y_continuous("mpg");
        }) + geom_line().aes(|a| {
            a.color_continuous("hp");
            a.size_discrete("cyl");
        });

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_lines_3.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }
}
