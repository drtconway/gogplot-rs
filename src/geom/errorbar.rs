use std::collections::HashMap;

use super::{AestheticRequirement, DomainConstraint, Geom, RenderContext};
use crate::aesthetics::builder::{
    AesMapBuilder, AlphaContinuousAesBuilder, AlphaDiscreteAesBuilder, ColorContinuousAesBuilder,
    ColorDiscreteAesBuilder, LineStyleAesBuilder, SizeContinuousAesBuilder,
    SizeDiscreteAesBuilder, XContinuousAesBuilder, YMaxContinuousAesBuilder,
    YMinContinuousAesBuilder,
};
use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain, AestheticProperty};
use crate::error::Result;
use crate::geom::properties::{Property, PropertyValue, PropertyVector};
use crate::layer::{Layer, LayerBuilder, LayerBuilderCore};
use crate::scale::ScaleIdentifier;
use crate::theme::{Color, color};
use crate::visuals::LineStyle;

pub trait GeomErrorbarAesBuilderTrait:
    XContinuousAesBuilder
    + YMinContinuousAesBuilder
    + YMaxContinuousAesBuilder
    + ColorContinuousAesBuilder
    + ColorDiscreteAesBuilder
    + AlphaContinuousAesBuilder
    + AlphaDiscreteAesBuilder
    + SizeContinuousAesBuilder
    + SizeDiscreteAesBuilder
    + LineStyleAesBuilder
{
}

impl GeomErrorbarAesBuilderTrait for AesMapBuilder {}

pub struct GeomErrorbarBuilder {
    core: LayerBuilderCore,
    color: Option<Color>,
    size: Option<f64>,
    alpha: Option<f64>,
    width: f64,
    linestyle: Option<LineStyle>,
}

impl GeomErrorbarBuilder {
    pub fn new() -> Self {
        Self {
            core: LayerBuilderCore::default(),
            color: None,
            size: None,
            alpha: None,
            width: 0.9,
            linestyle: None,
        }
    }

    pub fn color<C: Into<Color>>(mut self, color: C) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn size<S: Into<f64>>(mut self, size: S) -> Self {
        self.size = Some(size.into());
        self
    }

    pub fn alpha<A: Into<f64>>(mut self, alpha: A) -> Self {
        self.alpha = Some(alpha.into());
        self
    }

    pub fn width(mut self, width: f64) -> Self {
        self.width = width.max(0.0);
        self
    }

    pub fn linestyle<L: Into<LineStyle>>(mut self, linestyle: L) -> Self {
        self.linestyle = Some(linestyle.into());
        self
    }

    pub fn aes(mut self, closure: impl FnOnce(&mut dyn GeomErrorbarAesBuilderTrait)) -> Self {
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

impl LayerBuilder for GeomErrorbarBuilder {
    fn build(self: Box<Self>, parent_mapping: &AesMap) -> Result<Layer> {
        let mut geom = GeomErrorbar::new().width(self.width);

        let mut overrides = Vec::new();

        if self.color.is_some() {
            geom.color = self.color;
            overrides.push(Aesthetic::Color(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Color(AestheticDomain::Discrete));
        }
        if self.size.is_some() {
            geom.size = self.size;
            overrides.push(Aesthetic::Size(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Size(AestheticDomain::Discrete));
        }
        if self.alpha.is_some() {
            geom.alpha = self.alpha;
            overrides.push(Aesthetic::Alpha(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Alpha(AestheticDomain::Discrete));
        }
        if self.linestyle.is_some() {
            geom.linestyle = self.linestyle;
            overrides.push(Aesthetic::Linetype);
        }

        LayerBuilderCore::build(
            self.core,
            parent_mapping,
            Box::new(geom),
            HashMap::new(),
            &overrides,
        )
    }
}

pub fn geom_errorbar() -> GeomErrorbarBuilder {
    GeomErrorbarBuilder::new()
}

/// GeomErrorbar renders vertical error bars with horizontal caps at ymin and ymax
pub struct GeomErrorbar {
    color: Option<Color>,
    size: Option<f64>,
    alpha: Option<f64>,
    width: f64,
    linestyle: Option<LineStyle>,
}

impl GeomErrorbar {
    /// Create a new errorbar geom with default settings
    pub fn new() -> Self {
        Self {
            color: None,
            size: None,
            alpha: None,
            width: 0.9,
            linestyle: None,
        }
    }

    /// Set the width of the caps (in data coordinates)
    pub fn width(mut self, width: f64) -> Self {
        self.width = width.max(0.0);
        self
    }

    fn draw_errorbars(
        &self,
        ctx: &mut RenderContext,
        x_values: &[f64],
        ymin_values: &[f64],
        ymax_values: &[f64],
        color_values: &[Color],
        size_values: &[f64],
        alpha_values: &[f64],
        linestyles: &[LineStyle],
        width_values: Option<&[f64]>,
    ) -> Result<()> {
        if x_values.is_empty() {
            return Ok(());
        }

        // Calculate base cap width as a percentage of the x viewport width
        // This scales appropriately with plot size
        let base_cap_half_width_px = {
            let (x0, x1) = ctx.x_range;
            let viewport_width_px = (x1 - x0).abs();
            // Use 1.5% of viewport width per side (3% total) as default
            // This ensures caps are visible but not overwhelming
            (viewport_width_px * 0.015) * self.width
        };

        for i in 0..x_values.len() {
            let x = x_values[i];
            let ymin = ymin_values[i];
            let ymax = ymax_values[i];
            let Color(r, g, b, a) = color_values[i];
            let line_width = size_values[i];
            let alpha = alpha_values[i];
            let linestyle = &linestyles[i];

            // Use mapped width if available, otherwise use default
            let width_factor = width_values.map(|w| w[i]).unwrap_or(1.0);
            let cap_half_width_px = base_cap_half_width_px * width_factor;

            log::debug!(
                "Drawing errorbar at x={} from ymin={} to ymax={} with color=rgba({}, {}, {}, {}), line_width={}, alpha={}, cap_half_width_px={}",
                x,
                ymin,
                ymax,
                r,
                g,
                b,
                a,
                line_width,
                alpha,
                cap_half_width_px
            );

            // Set color with alpha
            ctx.cairo.set_source_rgba(
                r as f64 / 255.0,
                g as f64 / 255.0,
                b as f64 / 255.0,
                (a as f64 / 255.0) * alpha,
            );
            ctx.cairo.set_line_width(line_width);

            // Apply line style
            linestyle.apply(&mut ctx.cairo);

            // Map coordinates to pixel space
            let x_px = ctx.map_x(x);
            let ymin_px = ctx.map_y(ymin);
            let ymax_px = ctx.map_y(ymax);

            // Draw vertical line from ymin to ymax
            ctx.cairo.move_to(x_px, ymin_px);
            ctx.cairo.line_to(x_px, ymax_px);
            ctx.cairo.stroke().ok();

            // Draw caps directly in pixel space
            ctx.cairo.move_to(x_px - cap_half_width_px, ymin_px);
            ctx.cairo.line_to(x_px + cap_half_width_px, ymin_px);
            ctx.cairo.stroke().ok();

            ctx.cairo.move_to(x_px - cap_half_width_px, ymax_px);
            ctx.cairo.line_to(x_px + cap_half_width_px, ymax_px);
            ctx.cairo.stroke().ok();
        }

        Ok(())
    }
}

impl Default for GeomErrorbar {
    fn default() -> Self {
        Self::new()
    }
}

const AESTHETIC_REQUIREMENTS: [AestheticRequirement; 8] = [
    AestheticRequirement {
        property: AestheticProperty::X,
        required: true,
        constraint: DomainConstraint::Any,
    },
    AestheticRequirement {
        property: AestheticProperty::YMin,
        required: true,
        constraint: DomainConstraint::Any,
    },
    AestheticRequirement {
        property: AestheticProperty::YMax,
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
        property: AestheticProperty::Width,
        required: false,
        constraint: DomainConstraint::Any,
    },
    AestheticRequirement {
        property: AestheticProperty::Linetype,
        required: false,
        constraint: DomainConstraint::MustBe(AestheticDomain::Discrete),
    },
];

impl Geom for GeomErrorbar {
    fn aesthetic_requirements(&self) -> &'static [AestheticRequirement] {
        &AESTHETIC_REQUIREMENTS
    }

    fn properties(&self) -> HashMap<AestheticProperty, Property> {
        let mut props = HashMap::new();
        if let Some(color_prop) = &self.color {
            props.insert(
                AestheticProperty::Color,
                Property::Color(color_prop.clone()),
            );
        }
        if let Some(size_prop) = &self.size {
            props.insert(AestheticProperty::Size, Property::Float(size_prop.clone()));
        }
        if let Some(alpha_prop) = &self.alpha {
            props.insert(
                AestheticProperty::Alpha,
                Property::Float(alpha_prop.clone()),
            );
        }
        if let Some(linestyle_prop) = &self.linestyle {
            props.insert(
                AestheticProperty::Linetype,
                Property::LineStyle(linestyle_prop.clone()),
            );
        }
        props
    }

    fn property_defaults(
        &self,
        theme: &crate::prelude::Theme,
    ) -> HashMap<AestheticProperty, PropertyValue> {
        let mut defaults = HashMap::new();

        // Start with hardcoded defaults
        let mut default_size = 1.0;
        let mut default_color = color::BLACK;
        let mut default_alpha = 1.0;
        let mut default_linestyle = crate::visuals::LineStyle::Solid;

        // Apply theme overrides if present
        if let Some(crate::theme::Element::Line(elem)) = theme.get_element("errorbar", "line") {
            if let Some(size) = elem.size {
                default_size = size;
            }
            if let Some(color) = elem.color {
                default_color = color;
            }
            if let Some(alpha) = elem.alpha {
                default_alpha = alpha;
            }
            if let Some(ref linestyle) = elem.linestyle {
                default_linestyle = linestyle.clone();
            }
        }

        // Only set defaults for properties not explicitly set on the geom
        if self.color.is_none() {
            defaults.insert(AestheticProperty::Color, PropertyValue::Color(default_color));
        }
        if self.size.is_none() {
            defaults.insert(AestheticProperty::Size, PropertyValue::Float(default_size));
        }
        if self.alpha.is_none() {
            defaults.insert(AestheticProperty::Alpha, PropertyValue::Float(default_alpha));
        }
        if self.linestyle.is_none() {
            defaults.insert(
                AestheticProperty::Linetype,
                PropertyValue::LineStyle(default_linestyle),
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
            .expect("X values required for errorbar")
            .as_floats();

        let ymin_values = properties
            .remove(&AestheticProperty::YMin)
            .expect("Ymin values required for errorbar")
            .as_floats();

        let ymax_values = properties
            .remove(&AestheticProperty::YMax)
            .expect("Ymax values required for errorbar")
            .as_floats();

        let color_values = properties
            .remove(&AestheticProperty::Color)
            .expect("Color values required for errorbar")
            .as_colors();

        let size_values = properties
            .remove(&AestheticProperty::Size)
            .expect("Size values required for errorbar")
            .as_floats();

        let alpha_values = properties
            .remove(&AestheticProperty::Alpha)
            .expect("Alpha values required for errorbar")
            .as_floats();

        let linestyles = properties
            .remove(&AestheticProperty::Linetype)
            .expect("Linetype values required for errorbar")
            .as_linestyles();

        // Extract optional width aesthetic for cap width control
        let width_values = properties
            .remove(&AestheticProperty::Width)
            .map(|v| v.as_floats());

        self.draw_errorbars(
            ctx,
            &x_values,
            &ymin_values,
            &ymax_values,
            &color_values,
            &size_values,
            &alpha_values,
            &linestyles,
            width_values.as_deref(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::mtcars::mtcars;
    use crate::{error::to_io_error, plot::plot};

    fn init_test_logging() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn basic_errorbar_1() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
        }) + geom_errorbar().aes(|a| {
            a.ymin("qsec");
            a.ymax("hp");
        });

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_errorbar_1.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_errorbar_2() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
        }) + geom_errorbar().size(2.0).aes(|a| {
            a.ymin("qsec");
            a.ymax("hp");
            a.color_discrete("cyl");
        });

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_errorbar_2.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_errorbar_3() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
        }) + geom_errorbar()
            .size(2.0)
            .color(color::BLUE)
            .linestyle(LineStyle::Custom(vec![5, 3]))
            .aes(|a| {
                a.ymin("qsec");
                a.ymax("hp");
            });

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_errorbar_3.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }
}
