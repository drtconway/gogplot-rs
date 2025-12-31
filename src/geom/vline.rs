use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::builder::{
    AesMapBuilder, AlphaContinuousAesBuilder, AlphaDiscreteAesBuilder, ColorContinuousAesBuilder,
    ColorDiscreteAesBuilder, SizeContinuousAesBuilder, SizeDiscreteAesBuilder,
    XInterceptAesBuilder,
};
use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain, AestheticProperty};
use crate::data::PrimitiveValue;
use crate::error::Result;
use crate::geom::properties::{ColorProperty, FloatProperty, Property, PropertyVector};
use crate::geom::{AestheticRequirement, DomainConstraint};
use crate::layer::{Layer, LayerBuilder, LayerBuilderCore};
use crate::scale::ScaleIdentifier;
use crate::scale::traits::{ContinuousRangeScale, ScaleBase};
use crate::stat::Stat;
use crate::theme::{Color, color};

pub trait GeomVLineAesBuilderTrait:
    XInterceptAesBuilder
    + ColorContinuousAesBuilder
    + ColorDiscreteAesBuilder
    + AlphaContinuousAesBuilder
    + AlphaDiscreteAesBuilder
    + SizeContinuousAesBuilder
    + SizeDiscreteAesBuilder
{
}

impl GeomVLineAesBuilderTrait for AesMapBuilder {}

pub struct GeomVLineBuilder {
    core: LayerBuilderCore,
    x_intercept: Option<FloatProperty>,
    size: Option<FloatProperty>,
    color: Option<ColorProperty>,
    alpha: Option<FloatProperty>,
}

impl GeomVLineBuilder {
    pub fn new() -> Self {
        Self {
            core: LayerBuilderCore::default(),
            x_intercept: None,
            size: None,
            color: None,
            alpha: None,
        }
    }

    pub fn x_intercept<XIntercept: Into<FloatProperty>>(mut self, x_intercept: XIntercept) -> Self {
        self.x_intercept = Some(x_intercept.into());
        self
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

    pub fn aes(mut self, closure: impl FnOnce(&mut dyn GeomVLineAesBuilderTrait)) -> Self {
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

impl LayerBuilder for GeomVLineBuilder {
    fn build(self: Box<Self>, parent_mapping: &AesMap) -> Result<Layer> {
        let mut geom_vline = GeomVLine::new();

        // Build the mapping (merging layer + parent)
        let mut overrides = Vec::new();

        // Set fixed property values and remove from inherited mapping
        if self.x_intercept.is_some() {
            geom_vline.x_intercept = self.x_intercept;
            overrides.push(Aesthetic::XIntercept);
        }
        if self.size.is_some() {
            geom_vline.size = self.size;
            overrides.push(Aesthetic::Size(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Size(AestheticDomain::Discrete));
        }
        if self.color.is_some() {
            geom_vline.color = self.color;
            overrides.push(Aesthetic::Color(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Color(AestheticDomain::Discrete));
        }
        if self.alpha.is_some() {
            geom_vline.alpha = self.alpha;
            overrides.push(Aesthetic::Alpha(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Alpha(AestheticDomain::Discrete));
        }

        // Build initial domains from properties
        let mut initial_domains = HashMap::new();
        if geom_vline.x_intercept.is_some() {
            initial_domains.insert(AestheticProperty::XIntercept, AestheticDomain::Continuous);
        }

        LayerBuilderCore::build(
            self.core,
            parent_mapping,
            Box::new(geom_vline),
            initial_domains,
            &overrides,
        )
    }
}

pub fn geom_vline() -> GeomVLineBuilder {
    GeomVLineBuilder::new()
}

/// GeomVLine renders horizontal reference lines at specified y-intercepts
///
/// Y-intercept is provided via the Y aesthetic (mapped or constant).
/// Draws horizontal lines across the full width of the plot at each y value.
pub struct GeomVLine {
    /// Fixed y-intercept value(s) for the horizontal line(s)
    pub x_intercept: Option<FloatProperty>,

    /// Default line color
    pub color: Option<ColorProperty>,

    /// Default line width
    pub size: Option<FloatProperty>,

    /// Default alpha/opacity
    pub alpha: Option<FloatProperty>,
}

impl GeomVLine {
    /// Create a new horizontal line geom with default theme values
    pub fn new() -> Self {
        Self {
            x_intercept: None,
            color: None,
            size: None,
            alpha: None,
        }
    }

    fn draw_vlines(
        &self,
        ctx: &mut RenderContext,
        x_values: impl Iterator<Item = f64>,
        color_values: impl Iterator<Item = Color>,
        size_values: impl Iterator<Item = f64>,
        alpha_values: impl Iterator<Item = f64>,
    ) -> Result<()> {
        // X values are already normalized [0,1] by scales
        // Draw vertical line across full viewport height for each x value
        for (((x_norm, color), size), alpha) in x_values
            .zip(color_values)
            .zip(size_values)
            .zip(alpha_values)
        {
            let x_px = ctx.map_x(x_norm);

            let Color(r, g, b, a) = color;
            ctx.cairo.set_source_rgba(
                r as f64 / 255.0,
                g as f64 / 255.0,
                b as f64 / 255.0,
                a as f64 / 255.0 * alpha,
            );

            ctx.cairo.set_line_width(size);

            // Draw line from top edge to bottom edge of viewport
            ctx.cairo.move_to(x_px, ctx.y_range.0);
            ctx.cairo.line_to(x_px, ctx.y_range.1);
            ctx.cairo.stroke().ok();
        }

        Ok(())
    }
}

impl Default for GeomVLine {
    fn default() -> Self {
        Self::new()
    }
}

const AESTHETIC_REQUIREMENTS: [AestheticRequirement; 4] = [
    AestheticRequirement {
        property: AestheticProperty::XIntercept,
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
];

impl Geom for GeomVLine {
    fn aesthetic_requirements(&self) -> &'static [AestheticRequirement] {
        &AESTHETIC_REQUIREMENTS
    }

    fn properties(&self) -> HashMap<AestheticProperty, Property> {
        let mut props = HashMap::new();
        if let Some(x_intercept_prop) = &self.x_intercept {
            props.insert(
                AestheticProperty::XIntercept,
                Property::Float(x_intercept_prop.clone()),
            );
        }
        if let Some(size_prop) = &self.size {
            props.insert(AestheticProperty::Size, Property::Float(size_prop.clone()));
        }
        if let Some(color_prop) = &self.color {
            props.insert(
                AestheticProperty::Color,
                Property::Color(color_prop.clone()),
            );
        }
        if let Some(alpha_prop) = &self.alpha {
            props.insert(
                AestheticProperty::Alpha,
                Property::Float(alpha_prop.clone()),
            );
        }
        props
    }

    fn property_defaults(
        &self,
        _theme: &crate::prelude::Theme,
    ) -> HashMap<AestheticProperty, super::properties::PropertyValue> {
        let mut defaults = HashMap::new();

        // Only provide defaults for properties not explicitly set
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
        vec![ScaleIdentifier::XContinuous]
    }

    fn train_scales(&self, scales: &mut crate::scale::ScaleSet) {
        // If x_intercept is set as a property, train the X scale with it
        if let Some(x_prop) = &self.x_intercept {
            if let Some(value) = x_prop.get_value() {
                scales.x_continuous.train_one(&PrimitiveValue::Float(value));
            }
        }
    }

    fn apply_scales(&mut self, scales: &crate::scale::ScaleSet) {
        // Transform x_intercept through the X scale
        if let Some(x_prop) = &mut self.x_intercept {
            if let Some(value) = x_prop.get_value() {
                if let Some(normalized) = scales.x_continuous.map_value(&value) {
                    x_prop.value(normalized);
                } else {
                    self.x_intercept = None;
                    log::warn!(
                        "X-intercept value {} is outside the X scale domain and will not be rendered.",
                        value
                    );
                }
            }
        }
    }

    fn render(
        &self,
        ctx: &mut RenderContext,
        mut properties: HashMap<AestheticProperty, PropertyVector>,
    ) -> Result<()> {
        let x_values = properties
            .remove(&AestheticProperty::XIntercept)
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

        self.draw_vlines(
            ctx,
            x_values.into_iter(),
            color_values.into_iter(),
            size_values.into_iter(),
            alpha_values.into_iter(),
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        aesthetics::builder::{XContininuousAesBuilder, YContininuousAesBuilder},
        error::to_io_error,
        geom::point::geom_point,
        plot::plot,
        stat::summary::Summary,
        utils::mtcars::mtcars,
    };

    fn init_test_logging() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn basic_vlines_1() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
            a.y_continuous("mpg");
        }) + geom_point()
            + geom_vline().x_intercept(3.0).color(color::RED).size(2.0).alpha(0.75);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_vlines_1.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_vlines_2() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
            a.y_continuous("mpg");
        }) + geom_point()
            + geom_vline()
                .stat(Summary::from(Aesthetic::X(AestheticDomain::Continuous)))
                .aes(|a| a.x_intercept("mean"));

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_vlines_2.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }
}
