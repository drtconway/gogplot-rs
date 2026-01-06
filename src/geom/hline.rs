use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::builder::{
    AesMapBuilder, AlphaContinuousAesBuilder, AlphaDiscreteAesBuilder, ColorContinuousAesBuilder,
    ColorDiscreteAesBuilder, LineStyleAesBuilder, SizeContinuousAesBuilder, SizeDiscreteAesBuilder,
    YInterceptAesBuilder,
};
use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain, AestheticProperty};
use crate::data::PrimitiveValue;
use crate::error::Result;
use crate::geom::properties::{Property, PropertyVector};
use crate::geom::{AestheticRequirement, DomainConstraint};
use crate::layer::{Layer, LayerBuilder, LayerBuilderCore};
use crate::scale::ScaleIdentifier;
use crate::scale::traits::{ContinuousRangeScale, ScaleBase};
use crate::stat::Stat;
use crate::theme::{Color, LineElement};
use crate::visuals::LineStyle;

pub trait GeomHLineAesBuilderTrait:
    YInterceptAesBuilder
    + ColorContinuousAesBuilder
    + ColorDiscreteAesBuilder
    + AlphaContinuousAesBuilder
    + AlphaDiscreteAesBuilder
    + SizeContinuousAesBuilder
    + SizeDiscreteAesBuilder
    + LineStyleAesBuilder
{
}

impl GeomHLineAesBuilderTrait for AesMapBuilder {}

pub struct GeomHLineBuilder {
    core: LayerBuilderCore,
    y_intercept: Option<f64>,
    line: LineElement,
}

impl GeomHLineBuilder {
    pub fn new() -> Self {
        Self {
            core: LayerBuilderCore::default(),
            y_intercept: None,
            line: LineElement::default(),
        }
    }

    pub fn y_intercept<YIntercept: Into<f64>>(mut self, y_intercept: YIntercept) -> Self {
        self.y_intercept = Some(y_intercept.into());
        self
    }

    pub fn aes(mut self, closure: impl FnOnce(&mut dyn GeomHLineAesBuilderTrait)) -> Self {
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

    pub fn stat<S: Stat + 'static>(mut self, stat: S) -> Self {
        self.core.stat = Some(Box::new(stat));
        self
    }
}

impl crate::theme::traits::LineElement for GeomHLineBuilder {
    fn this(&self) -> &LineElement {
        &self.line
    }

    fn this_mut(&mut self) -> &mut LineElement {
        &mut self.line
    }
}

impl LayerBuilder for GeomHLineBuilder {
    fn build(self: Box<Self>, parent_mapping: &AesMap) -> Result<Layer> {
        let mut geom_hline = GeomHLine::new();
        geom_hline.line = self.line;

        let mut overrides = Vec::new();

        // Set fixed property values and remove from inherited mapping
        if self.y_intercept.is_some() {
            geom_hline.y_intercept = self.y_intercept;
            overrides.push(Aesthetic::YIntercept);
        }

        geom_hline.line.overrides(&mut overrides);

        // Build initial domains from properties
        let mut initial_domains = HashMap::new();
        if geom_hline.y_intercept.is_some() {
            initial_domains.insert(AestheticProperty::YIntercept, AestheticDomain::Continuous);
        }

        LayerBuilderCore::build(
            self.core,
            parent_mapping,
            Box::new(geom_hline),
            initial_domains,
            &overrides,
        )
    }
}

impl Default for GeomHLineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn geom_hline() -> GeomHLineBuilder {
    GeomHLineBuilder::new()
}

/// GeomHLine renders horizontal reference lines at specified y-intercepts
///
/// Y-intercept is provided via the Y aesthetic (mapped or constant).
/// Draws horizontal lines across the full width of the plot at each y value.
pub struct GeomHLine {
    /// Fixed y-intercept value(s) for the horizontal line(s)
    pub y_intercept: Option<f64>,
    line: LineElement,
}

impl GeomHLine {
    /// Create a new horizontal line geom with default theme values
    pub fn new() -> Self {
        Self {
            y_intercept: None,
            line: LineElement::default(),
        }
    }

    fn draw_hlines(
        &self,
        ctx: &mut RenderContext,
        y_values: impl Iterator<Item = f64>,
        color_values: impl Iterator<Item = Color>,
        size_values: impl Iterator<Item = f64>,
        alpha_values: impl Iterator<Item = f64>,
        linestyles: impl Iterator<Item = LineStyle>,
    ) -> Result<()> {
        // Y values are already normalized [0,1] by scales
        // Draw horizontal line across full viewport width for each y value
        for ((((y_norm, color), size), alpha), linestyle) in y_values
            .zip(color_values)
            .zip(size_values)
            .zip(alpha_values)
            .zip(linestyles)
        {
            let y_px = ctx.map_y(y_norm);

            let Color(r, g, b, a) = color;
            ctx.cairo.set_source_rgba(
                r as f64 / 255.0,
                g as f64 / 255.0,
                b as f64 / 255.0,
                a as f64 / 255.0 * alpha,
            );

            ctx.cairo.set_line_width(size);

            // Apply line style
            linestyle.apply(&mut ctx.cairo);

            // Draw line from left edge to right edge of viewport
            ctx.cairo.move_to(ctx.x_range.0, y_px);
            ctx.cairo.line_to(ctx.x_range.1, y_px);
            ctx.cairo.stroke().ok();
        }

        Ok(())
    }
}

const AESTHETIC_REQUIREMENTS: [AestheticRequirement; 5] = [
    AestheticRequirement {
        property: AestheticProperty::YIntercept,
        required: true, // Must have y-intercept (from property or mapping)
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

impl Geom for GeomHLine {
    fn aesthetic_requirements(&self) -> &'static [AestheticRequirement] {
        &AESTHETIC_REQUIREMENTS
    }

    fn properties(&self) -> HashMap<AestheticProperty, Property> {
        let mut props = HashMap::new();
        if let Some(y_intercept_prop) = &self.y_intercept {
            props.insert(
                AestheticProperty::YIntercept,
                Property::Float(y_intercept_prop.clone()),
            );
        }
        self.line.properties(&mut props);
        props
    }

    fn property_defaults(
        &self,
        theme: &crate::prelude::Theme,
    ) -> HashMap<AestheticProperty, super::properties::PropertyValue> {
        let mut defaults = HashMap::new();
        self.line.defaults("hline", "line", theme, &mut defaults);
        defaults
    }

    fn required_scales(&self) -> Vec<ScaleIdentifier> {
        vec![ScaleIdentifier::YContinuous]
    }

    fn train_scales(&self, scales: &mut crate::scale::ScaleSet) {
        // If y_intercept is set as a property, train the Y scale with it
        if let Some(value) = &self.y_intercept {
            scales
                .y_continuous
                .train_one(&PrimitiveValue::Float(*value));
        }
    }

    fn apply_scales(&mut self, scales: &crate::scale::ScaleSet) {
        // Transform y_intercept through the Y scale
        let mut warn = None;
        if let Some(value) = &mut self.y_intercept {
            if let Some(normalized) = scales.y_continuous.map_value(value) {
                *value = normalized;
            } else {
                warn = Some(*value);
            }
        }
        if let Some(value) = warn {
            self.y_intercept = None;
            log::warn!(
                "Y-intercept value {} is outside the Y scale domain and will not be rendered.",
                value
            );

        }
    }

    fn render(
        &self,
        ctx: &mut RenderContext,
        mut properties: HashMap<AestheticProperty, PropertyVector>,
    ) -> Result<()> {
        let y_values = properties
            .remove(&AestheticProperty::YIntercept)
            .unwrap()
            .as_floats();

        let color_values = properties
            .remove(&AestheticProperty::Color)
            .unwrap()
            .to_color()
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

        self.draw_hlines(
            ctx,
            y_values.into_iter(),
            color_values.into_iter(),
            size_values.into_iter(),
            alpha_values.into_iter(),
            linestyles.into_iter(),
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        aesthetics::builder::{XContinuousAesBuilder, YContinuousAesBuilder}, error::to_io_error, geom::point::geom_point, plot::plot, stat::summary::Summary, theme::{color, traits::LineElement}, utils::mtcars::mtcars
    };

    fn init_test_logging() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn basic_hlines_1() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
            a.y_continuous("mpg");
        }) + geom_point()
            + geom_hline().y_intercept(20.0).color(color::RED).size(2.0);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_hlines_1.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_hlines_2() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
            a.y_continuous("mpg");
        }) + geom_point()
            + geom_hline()
                .stat(Summary::from(Aesthetic::Y(AestheticDomain::Continuous)))
                .aes(|a| a.y_intercept("mean"));

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_hlines_2.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_hlines_3() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
            a.y_continuous("mpg");
        }) + geom_point()
            + geom_hline()
                .y_intercept(20.0)
                .color(color::DARKBLUE)
                .size(2.0)
                .linestyle(LineStyle::from("--"));

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_hlines_3.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }
}
