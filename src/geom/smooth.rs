use std::collections::HashMap;

use super::{AestheticRequirement, DomainConstraint, Geom, RenderContext};
use crate::aesthetics::builder::{
    AesMapBuilder, AlphaContinuousAesBuilder, AlphaDiscreteAesBuilder, ColorContinuousAesBuilder,
    ColorDiscreteAesBuilder, LineStyleAesBuilder, SizeContinuousAesBuilder, SizeDiscreteAesBuilder,
    XContinuousAesBuilder, YContinuousAesBuilder, YMaxContinuousAesBuilder,
    YMinContinuousAesBuilder,
};
use crate::aesthetics::{AesMap, AestheticDomain, AestheticProperty};
use crate::error::{PlotError, Result};
use crate::geom::properties::{Property, PropertyValue, PropertyVector};
use crate::layer::{Layer, LayerBuilder, LayerBuilderCore};
use crate::scale::ScaleIdentifier;
use crate::stat::smooth::Smooth;
use crate::theme::{AreaElement, Color};
use crate::visuals::LineStyle;

pub trait GeomSmoothAesBuilderTrait:
    XContinuousAesBuilder
    + YContinuousAesBuilder
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

impl GeomSmoothAesBuilderTrait for AesMapBuilder {}

pub struct GeomSmoothBuilder {
    core: LayerBuilderCore,
    area: AreaElement,
    confidence_interval: bool,
}

impl GeomSmoothBuilder {
    pub fn new() -> Self {
        Self {
            core: LayerBuilderCore::default(),
            area: AreaElement::default(),
            confidence_interval: true,
        }
    }

    pub fn confidence_interval(mut self, confidence_interval: bool) -> Self {
        self.confidence_interval = confidence_interval;
        self
    }

    pub fn aes(mut self, closure: impl FnOnce(&mut dyn GeomSmoothAesBuilderTrait)) -> Self {
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

impl crate::theme::traits::AreaElement for GeomSmoothBuilder {
    fn this(&self) -> &AreaElement {
        &self.area
    }

    fn this_mut(&mut self) -> &mut AreaElement {
        &mut self.area
    }
}

impl LayerBuilder for GeomSmoothBuilder {
    fn this(&self) -> &LayerBuilderCore {
        &self.core
    }

    fn this_mut(&mut self) -> &mut LayerBuilderCore {
        &mut self.core
    }

    fn build(mut self: Box<Self>, parent_mapping: &AesMap) -> Result<Layer> {
        let mut geom = GeomSmooth::new().confidence_interval(self.confidence_interval);
        geom.area = self.area;

        let mut overrides = Vec::new();
        geom.area.overrides(&mut overrides);

        if self.core.stat.is_none() {
            self.core.stat = Some(Box::new(Smooth::default()))
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

pub fn geom_smooth() -> GeomSmoothBuilder {
    GeomSmoothBuilder::new()
}

/// GeomSmooth renders fitted curves with confidence intervals
pub struct GeomSmooth {
    area: AreaElement,
    confidence_interval: bool,
}

impl GeomSmooth {
    pub fn new() -> Self {
        Self {
            area: AreaElement::default(),
            confidence_interval: true,
        }
    }

    pub fn confidence_interval(mut self, confidence_interval: bool) -> Self {
        self.confidence_interval = confidence_interval;
        self
    }

    fn draw_ribbon(
        &self,
        ctx: &mut RenderContext,
        x_values: &[f64],
        ymin_values: &[f64],
        ymax_values: &[f64],
        fill_values: &[Color],
        alpha_values: &[f64],
    ) -> Result<()> {
        if x_values.is_empty() || !self.confidence_interval {
            return Ok(());
        }

        // Build polygon path: forward along ymax, backward along ymin
        let Color(r, g, b, a) = fill_values[0];
        let alpha = alpha_values[0];

        ctx.cairo.set_source_rgba(
            r as f64 / 255.0,
            g as f64 / 255.0,
            b as f64 / 255.0,
            (a as f64 / 255.0) * alpha,
        );

        // Forward path along ymax
        for i in 0..x_values.len() {
            let x_px = ctx.map_x(x_values[i]);
            let ymax_px = ctx.map_y(ymax_values[i]);

            if i == 0 {
                ctx.cairo.move_to(x_px, ymax_px);
            } else {
                ctx.cairo.line_to(x_px, ymax_px);
            }
        }

        // Backward path along ymin
        for i in (0..x_values.len()).rev() {
            let x_px = ctx.map_x(x_values[i]);
            let ymin_px = ctx.map_y(ymin_values[i]);
            ctx.cairo.line_to(x_px, ymin_px);
        }

        ctx.cairo.close_path();
        ctx.cairo.fill().map_err(|e| PlotError::RenderError {
            operation: "fill ribbon".to_string(),
            message: e.to_string(),
        })?;

        Ok(())
    }

    fn draw_line(
        &self,
        ctx: &mut RenderContext,
        x_values: &[f64],
        y_values: &[f64],
        color_values: &[Color],
        size_values: &[f64],
        alpha_values: &[f64],
        linestyles: &[LineStyle],
    ) -> Result<()> {
        if x_values.is_empty() {
            return Ok(());
        }

        let Color(r, g, b, a) = color_values[0];
        let line_width = size_values[0];
        let alpha = alpha_values[0];
        let linestyle = &linestyles[0];

        ctx.cairo.set_source_rgba(
            r as f64 / 255.0,
            g as f64 / 255.0,
            b as f64 / 255.0,
            (a as f64 / 255.0) * alpha,
        );
        ctx.cairo.set_line_width(line_width);

        // Apply line style
        linestyle.apply(&mut ctx.cairo);

        for i in 0..x_values.len() {
            let x_px = ctx.map_x(x_values[i]);
            let y_px = ctx.map_y(y_values[i]);

            if i == 0 {
                ctx.cairo.move_to(x_px, y_px);
            } else {
                ctx.cairo.line_to(x_px, y_px);
            }
        }

        ctx.cairo.stroke().map_err(|e| PlotError::RenderError {
            operation: "stroke line".to_string(),
            message: e.to_string(),
        })?;

        Ok(())
    }
}

impl Default for GeomSmooth {
    fn default() -> Self {
        Self::new()
    }
}

const AESTHETIC_REQUIREMENTS: [AestheticRequirement; 9] = [
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
        property: AestheticProperty::YMin,
        required: false,
        constraint: DomainConstraint::Any,
    },
    AestheticRequirement {
        property: AestheticProperty::YMax,
        required: false,
        constraint: DomainConstraint::Any,
    },
    AestheticRequirement {
        property: AestheticProperty::Color,
        required: false,
        constraint: DomainConstraint::Any,
    },
    AestheticRequirement {
        property: AestheticProperty::Fill,
        required: false,
        constraint: DomainConstraint::Any,
    },
    AestheticRequirement {
        property: AestheticProperty::Alpha,
        required: false,
        constraint: DomainConstraint::Any,
    },
    AestheticRequirement {
        property: AestheticProperty::Size,
        required: false,
        constraint: DomainConstraint::Any,
    },
    AestheticRequirement {
        property: AestheticProperty::Linetype,
        required: false,
        constraint: DomainConstraint::MustBe(AestheticDomain::Discrete),
    },
];

impl Geom for GeomSmooth {
    fn aesthetic_requirements(&self) -> &'static [AestheticRequirement] {
        &AESTHETIC_REQUIREMENTS
    }

    fn properties(&self) -> HashMap<AestheticProperty, Property> {
        let mut props = HashMap::new();
        self.area.properties(&mut props);
        props
    }

    fn property_defaults(
        &self,
        theme: &crate::theme::Theme,
    ) -> HashMap<AestheticProperty, PropertyValue> {
        let mut defaults = HashMap::new();
        self.area.defaults("smooth", "smooth", theme, &mut defaults);
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
            .expect("X values required for smooth")
            .as_floats();

        let y_values = properties
            .remove(&AestheticProperty::Y)
            .expect("Y values required for smooth")
            .as_floats();

        let ymin_values = properties
            .remove(&AestheticProperty::YMin)
            .map(|v| v.as_floats());

        let ymax_values = properties
            .remove(&AestheticProperty::YMax)
            .map(|v| v.as_floats());

        let color_values = properties
            .remove(&AestheticProperty::Color)
            .unwrap()
            .to_color()
            .as_colors();

        let fill_values = properties
            .remove(&AestheticProperty::Fill)
            .unwrap()
            .to_color()
            .as_colors();

        let size_values = properties
            .remove(&AestheticProperty::Size)
            .expect("Size values required for smooth")
            .as_floats();

        let alpha_values = properties
            .remove(&AestheticProperty::Alpha)
            .expect("Alpha values required for smooth")
            .as_floats();

        let linestyles = properties
            .remove(&AestheticProperty::Linetype)
            .expect("Linetype values required for smooth")
            .as_linestyles();

        // Draw ribbon (confidence interval) first if se is enabled and we have ymin/ymax
        if self.confidence_interval {
            if let (Some(ymin), Some(ymax)) = (ymin_values.as_ref(), ymax_values.as_ref()) {
                self.draw_ribbon(ctx, &x_values, ymin, ymax, &fill_values, &alpha_values)?;
            }
        }

        // Draw line on top
        self.draw_line(
            ctx,
            &x_values,
            &y_values,
            &color_values,
            &size_values,
            &alpha_values,
            &linestyles,
        )?;

        Ok(())
    }
}

impl Clone for GeomSmooth {
    fn clone(&self) -> Self {
        Self {
            area: self.area.clone(),
            confidence_interval: self.confidence_interval,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::color;
    use crate::theme::traits::AreaElement;
    use crate::utils::mtcars::mtcars;
    use crate::{error::to_io_error, plot::plot};

    fn init_test_logging() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn basic_smooth_1() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
            a.y_continuous("mpg");
        }) + geom_smooth();

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_smooth_1.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_smooth_2() {
        init_test_logging();

        let data = mtcars();

        // Smooth with points, colored by group
        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
            a.y_continuous("mpg");
            a.color_discrete("cyl");
        }) + crate::geom::point::geom_point()
            + geom_smooth();

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_smooth_2.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_smooth_3() {
        init_test_logging();

        let data = mtcars();

        // Smooth without confidence interval
        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
            a.y_continuous("mpg");
        }) + geom_smooth().confidence_interval(false);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_smooth_3.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_smooth_4() {
        init_test_logging();

        let data = mtcars();

        // Smooth with custom styling
        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
            a.y_continuous("mpg");
        }) + geom_smooth()
            .color(color::RED)
            .fill(color::BLUE)
            .size(2.0)
            .alpha(0.2);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_smooth_4.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_smooth_5() {
        init_test_logging();

        let data = mtcars();

        // Smooth with dashed line
        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
            a.y_continuous("mpg");
        }) + geom_smooth()
            .color(color::BLUE)
            .size(2.0)
            .linestyle(LineStyle::Custom(vec![10, 5]))
            .confidence_interval(true);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_smooth_5.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }
}
