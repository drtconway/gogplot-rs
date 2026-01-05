use std::collections::HashMap;

use super::{AestheticRequirement, DomainConstraint, Geom, RenderContext};
use crate::aesthetics::builder::{
    AesMapBuilder, AlphaContinuousAesBuilder, AlphaDiscreteAesBuilder, ColorContinuousAesBuilder,
    ColorDiscreteAesBuilder, SizeContinuousAesBuilder, SizeDiscreteAesBuilder,
    XContinuousAesBuilder, YContinuousAesBuilder, YMaxContinuousAesBuilder,
    YMinContinuousAesBuilder,
};
use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain, AestheticProperty};
use crate::error::{PlotError, Result};
use crate::geom::properties::{Property, PropertyValue, PropertyVector};
use crate::layer::{Layer, LayerBuilder, LayerBuilderCore};
use crate::scale::ScaleIdentifier;
use crate::stat::smooth::Smooth;
use crate::theme::{Color, color};
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
{
}

impl GeomSmoothAesBuilderTrait for AesMapBuilder {}

pub struct GeomSmoothBuilder {
    core: LayerBuilderCore,
    color: Option<Color>,
    fill: Option<Color>,
    size: Option<f64>,
    alpha: Option<f64>,
    confidence_interval: bool,
    linestyle: Option<LineStyle>,
}

impl GeomSmoothBuilder {
    pub fn new() -> Self {
        Self {
            core: LayerBuilderCore::default(),
            color: None,
            fill: None,
            size: None,
            alpha: None,
            confidence_interval: true,
            linestyle: None,
        }
    }

    pub fn color<C: Into<Color>>(mut self, color: C) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn fill<F: Into<Color>>(mut self, fill: F) -> Self {
        self.fill = Some(fill.into());
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

    pub fn confidence_interval(mut self, confidence_interval: bool) -> Self {
        self.confidence_interval = confidence_interval;
        self
    }

    pub fn linestyle<L: Into<LineStyle>>(mut self, linestyle: L) -> Self {
        self.linestyle = Some(linestyle.into());
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

impl LayerBuilder for GeomSmoothBuilder {
    fn build(mut self: Box<Self>, parent_mapping: &AesMap) -> Result<Layer> {
        let mut geom = GeomSmooth::new().confidence_interval(self.confidence_interval);

        let mut overrides = Vec::new();

        if self.color.is_some() {
            geom.color = self.color;
            overrides.push(Aesthetic::Color(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Color(AestheticDomain::Discrete));
        }
        if self.fill.is_some() {
            geom.fill = self.fill;
            overrides.push(Aesthetic::Fill(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Fill(AestheticDomain::Discrete));
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
        }

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
    color: Option<Color>,
    fill: Option<Color>,
    size: Option<f64>,
    alpha: Option<f64>,
    confidence_interval: bool,
    linestyle: Option<LineStyle>,
}

impl GeomSmooth {
    pub fn new() -> Self {
        Self {
            color: None,
            fill: None,
            size: None,
            alpha: None,
            confidence_interval: true,
            linestyle: None,
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
    ) -> Result<()> {
        if x_values.is_empty() {
            return Ok(());
        }

        let Color(r, g, b, a) = color_values[0];
        let line_width = size_values[0];
        let alpha = alpha_values[0];

        ctx.cairo.set_source_rgba(
            r as f64 / 255.0,
            g as f64 / 255.0,
            b as f64 / 255.0,
            (a as f64 / 255.0) * alpha,
        );
        ctx.cairo.set_line_width(line_width);

        // Apply line style
        let linestyle = self.linestyle.as_ref().unwrap_or(&LineStyle::Solid);
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

const AESTHETIC_REQUIREMENTS: [AestheticRequirement; 8] = [
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
];

impl Geom for GeomSmooth {
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
        if let Some(fill_prop) = &self.fill {
            props.insert(AestheticProperty::Fill, Property::Color(fill_prop.clone()));
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
        props
    }

    fn property_defaults(
        &self,
        _theme: &crate::theme::Theme,
    ) -> HashMap<AestheticProperty, PropertyValue> {
        let mut defaults = HashMap::new();
        if self.color.is_none() {
            defaults.insert(AestheticProperty::Color, PropertyValue::Color(color::BLUE));
        }
        if self.fill.is_none() {
            defaults.insert(
                AestheticProperty::Fill,
                PropertyValue::Color(Color::rgb(128, 128, 128)),
            );
        }
        if self.size.is_none() {
            defaults.insert(AestheticProperty::Size, PropertyValue::Float(1.0));
        }
        if self.alpha.is_none() {
            defaults.insert(AestheticProperty::Alpha, PropertyValue::Float(0.4));
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
        )?;

        Ok(())
    }
}

impl Clone for GeomSmooth {
    fn clone(&self) -> Self {
        Self {
            color: self.color.clone(),
            fill: self.fill.clone(),
            size: self.size.clone(),
            alpha: self.alpha.clone(),
            confidence_interval: self.confidence_interval,
            linestyle: self.linestyle.clone(),
        }
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
