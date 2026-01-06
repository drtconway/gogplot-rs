use std::collections::HashMap;

use super::{Geom, RenderContext, AestheticRequirement, DomainConstraint};
use crate::aesthetics::builder::{
    AesMapBuilder, AlphaContinuousAesBuilder, AlphaDiscreteAesBuilder, ColorContinuousAesBuilder,
    ColorDiscreteAesBuilder, FillContinuousAesBuilder, FillDiscreteAesBuilder, GroupAesBuilder,
    LineStyleAesBuilder, XContinuousAesBuilder, YContinuousAesBuilder,
};
use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain, AestheticProperty};
use crate::error::Result;
use crate::geom::properties::{Property, PropertyValue, PropertyVector};
use crate::layer::{Layer, LayerBuilder, LayerBuilderCore};
use crate::scale::ScaleIdentifier;
use crate::stat::density::Density;
use crate::theme::Color;
use crate::visuals::LineStyle;

pub trait GeomDensityAesBuilderTrait:
    XContinuousAesBuilder
    + YContinuousAesBuilder
    + ColorContinuousAesBuilder
    + ColorDiscreteAesBuilder
    + FillContinuousAesBuilder
    + FillDiscreteAesBuilder
    + AlphaContinuousAesBuilder
    + AlphaDiscreteAesBuilder
    + GroupAesBuilder
    + LineStyleAesBuilder
{
}

impl GeomDensityAesBuilderTrait for AesMapBuilder {}

pub struct GeomDensityBuilder {
    core: LayerBuilderCore,
    color: Option<Color>,
    fill: Option<Color>,
    alpha: Option<f64>,
    size: Option<f64>,
    linestyle: Option<LineStyle>,
}

impl GeomDensityBuilder {
    pub fn new() -> Self {
        Self {
            core: LayerBuilderCore::default(),
            color: None,
            fill: None,
            alpha: None,
            size: None,
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

    pub fn alpha<A: Into<f64>>(mut self, alpha: A) -> Self {
        self.alpha = Some(alpha.into());
        self
    }

    pub fn size<S: Into<f64>>(mut self, size: S) -> Self {
        self.size = Some(size.into());
        self
    }

    pub fn linestyle<L: Into<LineStyle>>(mut self, linestyle: L) -> Self {
        self.linestyle = Some(linestyle.into());
        self
    }

    pub fn aes(mut self, closure: impl FnOnce(&mut dyn GeomDensityAesBuilderTrait)) -> Self {
        if self.core.stat.is_none() {
            if self.core.aes_builder.is_none() {
                self.core.aes_builder = Some(AesMapBuilder::new());
            }
            if let Some(ref mut builder) = self.core.aes_builder {
                closure(builder);
            }
        } else {
            if self.core.after_aes_builder.is_none() {
                self.core.after_aes_builder = Some(AesMapBuilder::new());
            }
            if let Some(ref mut builder) = self.core.after_aes_builder {
                closure(builder);
            }
        }
        self
    }

    pub fn stat<S: 'static + crate::stat::Stat>(mut self, stat: S) -> Self {
        self.core.stat = Some(Box::new(stat));
        self
    }

    pub fn position(mut self, position: &str) -> Self {
        self.core.position = Some(position.into());
        self
    }
}

impl LayerBuilder for GeomDensityBuilder {
    fn build(mut self: Box<Self>, parent_mapping: &AesMap) -> Result<Layer> {
        let mut geom_density = GeomDensity::new();

        // Build the mapping (merging layer + parent)
        let mut overrides = Vec::new();

        // Set fixed property values and remove from inherited mapping
        if self.color.is_some() {
            geom_density.color = self.color;
            overrides.push(Aesthetic::Color(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Color(AestheticDomain::Discrete));
        }
        if self.fill.is_some() {
            geom_density.fill = self.fill;
            overrides.push(Aesthetic::Fill(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Fill(AestheticDomain::Discrete));
        }
        if self.alpha.is_some() {
            geom_density.alpha = self.alpha;
            overrides.push(Aesthetic::Alpha(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Alpha(AestheticDomain::Discrete));
        }
        if self.size.is_some() {
            geom_density.size = self.size;
        }
        if self.linestyle.is_some() {
            geom_density.linestyle = self.linestyle;
            overrides.push(Aesthetic::Linetype);
        }

        // Make Density the default stat if none specified
        if self.core.stat.is_none() {
            self.core.stat = Some(Box::new(Density::new()));
        }

        LayerBuilderCore::build(
            self.core,
            parent_mapping,
            Box::new(geom_density),
            HashMap::new(),
            &overrides,
        )
    }
}

pub fn geom_density() -> GeomDensityBuilder {
    GeomDensityBuilder::new()
}

/// GeomDensity renders kernel density estimates
///
/// This geom automatically computes the density using the Density stat
/// and renders it as a line plot.
pub struct GeomDensity {
    /// Default line color (if not mapped)
    pub color: Option<Color>,

    /// Default fill color for area under curve (if not mapped)
    pub fill: Option<Color>,

    /// Default alpha/opacity (if not mapped)
    pub alpha: Option<f64>,

    /// Default line width (if not mapped)
    pub size: Option<f64>,

    /// Line style for density curve
    pub linestyle: Option<LineStyle>,
}

impl GeomDensity {
    /// Create a new density geom with default settings
    pub fn new() -> Self {
        Self {
            color: None,
            fill: None,
            alpha: None,
            size: None,
            linestyle: None,
        }
    }
}

impl Default for GeomDensity {
    fn default() -> Self {
        Self::new()
    }
}

const AESTHETIC_REQUIREMENTS: [AestheticRequirement; 7] = [
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

impl Geom for GeomDensity {
    fn aesthetic_requirements(&self) -> &'static [AestheticRequirement] {
        &AESTHETIC_REQUIREMENTS
    }

    fn properties(&self) -> HashMap<AestheticProperty, Property> {
        let mut props = HashMap::new();
        if let Some(color_prop) = &self.color {
            props.insert(AestheticProperty::Color, Property::Color(color_prop.clone()));
        }
        if let Some(fill_prop) = &self.fill {
            props.insert(AestheticProperty::Fill, Property::Color(fill_prop.clone()));
        }
        if let Some(alpha_prop) = &self.alpha {
            props.insert(AestheticProperty::Alpha, Property::Float(alpha_prop.clone()));
        }
        if let Some(size_prop) = &self.size {
            props.insert(AestheticProperty::Size, Property::Float(size_prop.clone()));
        }
        if let Some(linestyle_prop) = &self.linestyle {
            props.insert(
                AestheticProperty::Linetype,
                Property::LineStyle(linestyle_prop.clone()),
            );
        }
        props
    }

    fn property_defaults(&self, theme: &crate::theme::Theme) -> HashMap<AestheticProperty, PropertyValue> {
        let mut defaults = HashMap::new();
        if self.color.is_none() {
            defaults.insert(AestheticProperty::Color, PropertyValue::Color(theme.geom_line.color));
        }
        if self.fill.is_none() {
            defaults.insert(AestheticProperty::Fill, PropertyValue::Color(Color(0, 0, 0, 0))); // Transparent by default
        }
        if self.alpha.is_none() {
            defaults.insert(AestheticProperty::Alpha, PropertyValue::Float(theme.geom_line.alpha));
        }
        if self.size.is_none() {
            defaults.insert(AestheticProperty::Size, PropertyValue::Float(theme.geom_line.size));
        }
        if self.linestyle.is_none() {
            defaults.insert(
                AestheticProperty::Linetype,
                PropertyValue::LineStyle(LineStyle::Solid),
            );
        }
        defaults
    }

    fn required_scales(&self) -> Vec<ScaleIdentifier> {
        vec![ScaleIdentifier::XContinuous, ScaleIdentifier::YContinuous]
    }

    fn train_scales(&self, _scales: &mut crate::scale::ScaleSet) {
        // Density doesn't need additional scale training
    }

    fn apply_scales(&mut self, _scales: &crate::scale::ScaleSet) {
        // No scale-dependent properties to update
    }

    fn render(
        &self,
        ctx: &mut RenderContext,
        mut properties: HashMap<AestheticProperty, PropertyVector>,
    ) -> Result<()> {
        let x_values = properties
            .remove(&AestheticProperty::X)
            .expect("X values required for density")
            .as_floats();

        let y_values = properties
            .remove(&AestheticProperty::Y)
            .expect("Y values required for density")
            .as_floats();

        let color_values = properties
            .remove(&AestheticProperty::Color)
            .expect("Color values required for density")
            .to_color()
            .as_colors();

        let fill_values = properties
            .remove(&AestheticProperty::Fill)
            .expect("Fill values required for density")
            .to_color()
            .as_colors();

        let alpha_values = properties
            .remove(&AestheticProperty::Alpha)
            .expect("Alpha values required for density")
            .as_floats();

        let size_values = properties
            .remove(&AestheticProperty::Size)
            .expect("Size values required for density")
            .as_floats();

        let linestyles = properties
            .remove(&AestheticProperty::Linetype)
            .expect("Linetype values required for density")
            .as_linestyles();

        self.draw_density(
            ctx,
            &x_values,
            &y_values,
            &color_values,
            &fill_values,
            &alpha_values,
            &size_values,
            &linestyles,
        )
    }
}

impl GeomDensity {
    fn draw_density(
        &self,
        ctx: &mut RenderContext,
        x_values: &[f64],
        y_values: &[f64],
        color_values: &[Color],
        fill_values: &[Color],
        alpha_values: &[f64],
        size_values: &[f64],
        linestyles: &[LineStyle],
    ) -> Result<()> {
        if x_values.is_empty() {
            return Ok(());
        }

        // Use first point's aesthetics (density curves are typically uniform in appearance)
        let line_color = color_values[0];
        let fill_color = fill_values[0];
        let alpha = alpha_values[0];
        let line_width = size_values[0];
        let linestyle = &linestyles[0];

        // Draw filled area if fill has alpha > 0
        let Color(fr, fg, fb, fa) = fill_color;
        if fa > 0 {
            ctx.cairo.set_source_rgba(
                fr as f64 / 255.0,
                fg as f64 / 255.0,
                fb as f64 / 255.0,
                fa as f64 / 255.0 * alpha,
            );

            // Start at bottom-left
            let x0_px = ctx.map_x(x_values[0]);
            let y0_px = ctx.map_y(0.0);
            ctx.cairo.move_to(x0_px, y0_px);

            // Draw along the density curve
            for i in 0..x_values.len() {
                let x_px = ctx.map_x(x_values[i]);
                let y_px = ctx.map_y(y_values[i]);
                ctx.cairo.line_to(x_px, y_px);
            }

            // Close path at bottom-right
            let xn_px = ctx.map_x(x_values[x_values.len() - 1]);
            let yn_px = ctx.map_y(0.0);
            ctx.cairo.line_to(xn_px, yn_px);
            ctx.cairo.close_path();
            ctx.cairo.fill().ok();
        }

        // Draw the density curve line
        let Color(r, g, b, a) = line_color;
        ctx.cairo.set_source_rgba(
            r as f64 / 255.0,
            g as f64 / 255.0,
            b as f64 / 255.0,
            a as f64 / 255.0 * alpha,
        );
        ctx.cairo.set_line_width(line_width);

        // Apply line style
        linestyle.apply(&mut ctx.cairo);

        // Start path at first point
        let x0_px = ctx.map_x(x_values[0]);
        let y0_px = ctx.map_y(y_values[0]);
        ctx.cairo.move_to(x0_px, y0_px);

        // Draw lines to subsequent points
        for i in 1..x_values.len() {
            let x_px = ctx.map_x(x_values[i]);
            let y_px = ctx.map_y(y_values[i]);
            ctx.cairo.line_to(x_px, y_px);
        }

        ctx.cairo.stroke().ok();

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aesthetics::builder::XContinuousAesBuilder;
    use crate::data::{DataSource, VectorValue};
    use crate::error::to_io_error;
    use crate::plot::plot;
    use crate::theme::color;
    use crate::utils::dataframe::DataFrame;

    fn init_test_logging() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn basic_density_1() {
        init_test_logging();

        // Create sample data with a mixture of values
        let values = vec![
            5.0, 5.2, 5.5, 5.8, 6.0, 6.2, 6.5, 6.8, 7.0, 7.2,
            7.5, 7.8, 8.0, 8.2, 8.5, 8.8, 9.0, 9.2, 9.5, 9.8,
            10.0, 10.5, 11.0, 11.5, 12.0, 12.5, 13.0, 13.5, 14.0, 14.5,
            15.0, 15.5, 16.0, 16.5, 17.0, 17.5, 18.0, 18.5, 19.0, 19.5,
        ];

        let data: Box<dyn DataSource> = Box::new(DataFrame::from_columns(vec![
            ("value", VectorValue::from(values)),
        ]));

        let builder = plot(&data).aes(|a| {
            a.x_continuous("value");
        }) + geom_density()
            .color(color::STEELBLUE)
            .fill(color::LIGHTBLUE);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_density_1.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_density_2() {
        init_test_logging();

        let data = crate::utils::mtcars::mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("mpg");
        }) + geom_density().aes(|a| {
            a.color_discrete("cyl");
            a.fill_discrete("cyl");
        }).alpha(0.5);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_density_2.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_density_3() {
        init_test_logging();

        let data = crate::utils::mtcars::mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("mpg");
        }) + geom_density()
            .color(crate::theme::color::BLUE)
            .size(2.0)
            .linestyle(LineStyle::Custom(vec![10, 5]))
            .fill(crate::theme::color::LIGHTBLUE)
            .alpha(0.3);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_density_3.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }
}
