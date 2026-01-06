use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::builder::{
    AesMapBuilder, AlphaContinuousAesBuilder, AlphaDiscreteAesBuilder, ColorContinuousAesBuilder, ColorDiscreteAesBuilder, LineStyleAesBuilder, SizeContinuousAesBuilder, SizeDiscreteAesBuilder, XContinuousAesBuilder, XDiscreteAesBuilder, YContinuousAesBuilder, YDiscreteAesBuilder
};
use crate::aesthetics::{AesMap, AestheticDomain, AestheticProperty};
use crate::error::Result;
use crate::geom::properties::PropertyVector;
use crate::geom::{AestheticRequirement, DomainConstraint};
use crate::layer::{Layer, LayerBuilder, LayerBuilderCore};
use crate::scale::ScaleIdentifier;
use crate::theme::{Color, LineElement};
use crate::visuals::LineStyle;

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
    + LineStyleAesBuilder
{
}

impl GeomLineAesBuilderTrait for AesMapBuilder {}

pub struct GeomLineBuilder {
    core: LayerBuilderCore,
    line: LineElement,
}

impl GeomLineBuilder {
    pub fn new() -> Self {
        Self {
            core: LayerBuilderCore::default(),
            line: LineElement::default(),
        }
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

impl crate::theme::traits::LineElement for GeomLineBuilder {
    fn this(&self) -> &LineElement {
        &self.line
    }

    fn this_mut(&mut self) -> &mut LineElement {
        &mut self.line
    }
}

impl LayerBuilder for GeomLineBuilder {
    fn build(self: Box<Self>, parent_mapping: &AesMap) -> Result<Layer> {
        let mut geom_line = GeomLine::new();

        geom_line.line = self.line;

        // Build the mapping (merging layer + parent)
        let mut overrides = Vec::new();

        geom_line.line.overrides(&mut overrides);

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
    line: LineElement,
}

impl GeomLine {
    /// Create a new point geom with default settings from theme
    pub fn new() -> Self {
        Self {
            line: LineElement::default(),
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
        linestyle_values: impl Iterator<Item = LineStyle>,
    ) -> Result<()> {
        // Collect all points for this line segment
        let points: Vec<_> = x_values
            .zip(y_values)
            .zip(color_values)
            .zip(size_values)
            .zip(alpha_values)
            .zip(linestyle_values)
            .collect();

        if points.is_empty() {
            return Ok(());
        }

        // Draw lines as segments to support varying aesthetics
        // Each segment from point i to i+1 uses the aesthetics of point i
        for i in 0..points.len() - 1 {
            let (((((x1_norm, y1_norm), color1), size1), alpha1), linestyle1) = &points[i];
            let (((((x2_norm, y2_norm), _), _), _), _) = &points[i + 1];

            // Set color and alpha for this segment
            let Color(r, g, b, a) = *color1;
            ctx.cairo.set_source_rgba(
                r as f64 / 255.0,
                g as f64 / 255.0,
                b as f64 / 255.0,
                a as f64 / 255.0 * alpha1,
            );

            // Set line width (size)
            ctx.cairo.set_line_width(*size1);

            // Apply line style
            linestyle1.apply(&mut ctx.cairo);

            // Draw this segment
            let x1_px = ctx.map_x(*x1_norm);
            let y1_px = ctx.map_y(*y1_norm);
            let x2_px = ctx.map_x(*x2_norm);
            let y2_px = ctx.map_y(*y2_norm);

            ctx.cairo.move_to(x1_px, y1_px);
            ctx.cairo.line_to(x2_px, y2_px);
            ctx.cairo.stroke().ok();
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
        property: AestheticProperty::Linetype,
        required: false,
        constraint: DomainConstraint::MustBe(AestheticDomain::Discrete),
    },
];

impl Geom for GeomLine {
    fn aesthetic_requirements(&self) -> &'static [AestheticRequirement] {
        &AESTHETIC_REQUIREMENTS
    }

    fn properties(&self) -> HashMap<AestheticProperty, super::properties::Property> {
        let mut props = HashMap::new();
        self.line.properties(&mut props);
        props
    }

    fn property_defaults(
        &self,
        theme: &crate::theme::Theme,
    ) -> HashMap<AestheticProperty, super::properties::PropertyValue> {
        let mut defaults = HashMap::new();

        self.line.defaults("line", "line", theme, &mut defaults);

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
        let linestyles = properties
            .remove(&AestheticProperty::Linetype)
            .unwrap()
            .as_linestyles();

        // Create a permutation iterator to sort points by x-value
        let mut indices: Vec<usize> = (0..x_values.len()).collect();
        indices.sort_by(|&i, &j| x_values[i].partial_cmp(&x_values[j]).unwrap());

        let x_values = indices.iter().map(|&i| x_values[i]);
        let y_values = indices.iter().map(|&i| y_values[i]);
        let color_values = indices.iter().map(|&i| color_values[i]);
        let size_values = indices.iter().map(|&i| size_values[i]);
        let alpha_values = indices.iter().map(|&i| alpha_values[i]);
        let linestyle_values = indices.iter().map(|&i| linestyles[i].clone());

        self.draw_lines(
            ctx,
            x_values,
            y_values,
            color_values,
            size_values,
            alpha_values,
            linestyle_values,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{error::to_io_error, plot::plot, theme::{color, traits::LineElement}, utils::mtcars::mtcars};

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

    #[test]
    fn basic_lines_4() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
            a.y_continuous("mpg");
        }) + geom_line()
            .size(2.0)
            .color(color::FIREBRICK)
            .linestyle(LineStyle::from("-."));

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_lines_4.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_lines_5_mapped_linestyle() {
        init_test_logging();

        let data = mtcars();

        // Test linestyle mapped from discrete column
        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
            a.y_continuous("mpg");
        }) + geom_line().aes(|a| {
            a.linestyle("cyl");
        });

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_lines_5_mapped_linestyle.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }
}
