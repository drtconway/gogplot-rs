use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::builder::{
    AesMapBuilder, AlphaContinuousAesBuilder, AlphaDiscreteAesBuilder, FillContinuousAesBuilder,
    FillDiscreteAesBuilder, GroupAesBuilder, XMaxContinuousAesBuilder, XMinContinuousAesBuilder,
    YMaxContinuousAesBuilder, YMinContinuousAesBuilder,
};
use crate::aesthetics::{AesMap, AestheticProperty};
use crate::error::Result;
use crate::geom::properties::{Property, PropertyVector};
use crate::geom::{AestheticRequirement, DomainConstraint};
use crate::layer::{Layer, LayerBuilder, LayerBuilderCore};
use crate::scale::ScaleIdentifier;
use crate::stat::Stat;
use crate::theme::{AreaElement, Color};

pub trait GeomRectAesBuilderTrait:
    XMinContinuousAesBuilder
    + XMaxContinuousAesBuilder
    + YMinContinuousAesBuilder
    + YMaxContinuousAesBuilder
    + FillContinuousAesBuilder
    + FillDiscreteAesBuilder
    + AlphaContinuousAesBuilder
    + AlphaDiscreteAesBuilder
    + GroupAesBuilder
{
}

impl GeomRectAesBuilderTrait for AesMapBuilder {}

pub struct GeomRectBuilder {
    core: LayerBuilderCore,
    area: AreaElement,
}

impl GeomRectBuilder {
    pub fn new() -> Self {
        Self {
            core: LayerBuilderCore::default(),
            area: AreaElement::default(),
        }
    }

    pub fn aes(mut self, closure: impl FnOnce(&mut dyn GeomRectAesBuilderTrait)) -> Self {
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

impl crate::theme::traits::AreaElement for GeomRectBuilder {
    fn this(&self) -> &AreaElement {
        &self.area
    }

    fn this_mut(&mut self) -> &mut AreaElement {
        &mut self.area
    }
}

impl LayerBuilder for GeomRectBuilder {
    fn build(self: Box<Self>, parent_mapping: &AesMap) -> Result<Layer> {
        let mut geom_rect = GeomRect::new();
        geom_rect.area = self.area;

        // Build the mapping (merging layer + parent)
        let mut overrides = Vec::new();

        geom_rect.area.overrides(&mut overrides);

        LayerBuilderCore::build(
            self.core,
            parent_mapping,
            Box::new(geom_rect),
            HashMap::new(),
            &overrides,
        )
    }
}

pub fn geom_rect() -> GeomRectBuilder {
    GeomRectBuilder::new()
}

/// GeomRect renders rectangles defined by xmin, xmax, ymin, ymax
///
/// Rectangles are defined by their bounding boxes, which must come from data.
/// Useful for heatmaps, tile plots, and annotating regions.
pub struct GeomRect {
    area: AreaElement,
}

impl GeomRect {
    /// Create a new rect geom with default theme values
    pub fn new() -> Self {
        Self {
            area: AreaElement::default(),
        }
    }

    fn draw_rects(
        &self,
        ctx: &mut RenderContext,
        xmin_values: impl Iterator<Item = f64>,
        xmax_values: impl Iterator<Item = f64>,
        ymin_values: impl Iterator<Item = f64>,
        ymax_values: impl Iterator<Item = f64>,
        color_values: impl Iterator<Item = Color>,
        fill_values: impl Iterator<Item = Color>,
        alpha_values: impl Iterator<Item = f64>,
    ) -> Result<()> {
        // All values are already normalized [0,1] by scales
        // Draw rectangles at the specified bounds
        for ((((((xmin_norm, xmax_norm), ymin_norm), ymax_norm), color), fill), alpha) in
            xmin_values
                .zip(xmax_values)
                .zip(ymin_values)
                .zip(ymax_values)
                .zip(color_values)
                .zip(fill_values)
                .zip(alpha_values)
        {
            let xmin_px = ctx.map_x(xmin_norm);
            let xmax_px = ctx.map_x(xmax_norm);
            let ymin_px = ctx.map_y(ymin_norm);
            let ymax_px = ctx.map_y(ymax_norm);

            // Draw filled rectangle
            let width = xmax_px - xmin_px;
            let height = ymax_px - ymin_px;

            // Fill
            let Color(r, g, b, a) = fill;
            ctx.cairo.set_source_rgba(
                r as f64 / 255.0,
                g as f64 / 255.0,
                b as f64 / 255.0,
                a as f64 / 255.0 * alpha,
            );
            ctx.cairo.rectangle(xmin_px, ymin_px, width, height);
            ctx.cairo.fill().ok();

            // Border/stroke
            let Color(r, g, b, a) = color;
            ctx.cairo.set_source_rgba(
                r as f64 / 255.0,
                g as f64 / 255.0,
                b as f64 / 255.0,
                a as f64 / 255.0 * alpha,
            );
            ctx.cairo.rectangle(xmin_px, ymin_px, width, height);
            ctx.cairo.stroke().ok();
        }

        Ok(())
    }
}

const AESTHETIC_REQUIREMENTS: [AestheticRequirement; 7] = [
    AestheticRequirement {
        property: AestheticProperty::XMin,
        required: true, // Must have xmin from mapping
        constraint: DomainConstraint::Any,
    },
    AestheticRequirement {
        property: AestheticProperty::XMax,
        required: true, // Must have xmax from mapping
        constraint: DomainConstraint::Any,
    },
    AestheticRequirement {
        property: AestheticProperty::YMin,
        required: true, // Must have ymin from mapping
        constraint: DomainConstraint::Any,
    },
    AestheticRequirement {
        property: AestheticProperty::YMax,
        required: true, // Must have ymax from mapping
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
];

impl Geom for GeomRect {
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
    ) -> HashMap<AestheticProperty, super::properties::PropertyValue> {
        let mut defaults = HashMap::new();

        self.area.defaults("rect", "rect", theme, &mut defaults);

        defaults
    }

    fn required_scales(&self) -> Vec<ScaleIdentifier> {
        vec![ScaleIdentifier::XContinuous, ScaleIdentifier::YContinuous]
    }

    fn train_scales(&self, _scales: &mut crate::scale::ScaleSet) {
        // Bounds come from data, no property training needed
    }

    fn apply_scales(&mut self, _scales: &crate::scale::ScaleSet) {
        // Bounds come from data, no property scaling needed
    }

    fn render(
        &self,
        ctx: &mut RenderContext,
        mut properties: HashMap<AestheticProperty, PropertyVector>,
    ) -> Result<()> {
        let xmin_values = properties
            .remove(&AestheticProperty::XMin)
            .unwrap()
            .as_floats();

        let xmax_values = properties
            .remove(&AestheticProperty::XMax)
            .unwrap()
            .as_floats();

        let ymin_values = properties
            .remove(&AestheticProperty::YMin)
            .unwrap()
            .as_floats();

        let ymax_values = properties
            .remove(&AestheticProperty::YMax)
            .unwrap()
            .as_floats();

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

        let alpha_values = properties
            .remove(&AestheticProperty::Alpha)
            .unwrap()
            .as_floats();

        self.draw_rects(
            ctx,
            xmin_values.into_iter(),
            xmax_values.into_iter(),
            ymin_values.into_iter(),
            ymax_values.into_iter(),
            color_values.into_iter(),
            fill_values.into_iter(),
            alpha_values.into_iter(),
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        aesthetics::{Aesthetic, AestheticDomain},
        data::DataSource,
        error::to_io_error,
        plot::plot,
        stat::summary::Summary,
        theme::{color, traits::AreaElement},
        utils::{dataframe::DataFrame, mtcars::mtcars},
    };

    fn init_test_logging() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn basic_rects_1() {
        init_test_logging();

        let xmins = vec![1.0, 2.0, 3.0];
        let xmaxs = vec![1.5, 2.5, 3.5];
        let ymins = vec![10.0, 20.0, 30.0];
        let ymaxs = vec![15.0, 25.0, 35.0];

        let data: Box<dyn DataSource> = Box::new(DataFrame::from_columns(vec![
            ("xmin", xmins),
            ("xmax", xmaxs),
            ("ymin", ymins),
            ("ymax", ymaxs),
        ]));

        let builder = plot(&data).aes(|a| {
            a.xmin("xmin");
            a.xmax("xmax");
            a.ymin("ymin");
            a.ymax("ymax");
        }) + geom_rect()
            .color(color::RED)
            .fill(color::FIREBRICK)
            .alpha(0.75);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_rects_1.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_rects_2() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data)
            + geom_rect()
                .aes(|a| {
                    a.xmin("wt");
                    a.ymin("mpg");
                    a.group("cyl");
                })
                .stat(Summary::from(vec![
                    Aesthetic::Xmin(AestheticDomain::Continuous),
                    Aesthetic::Ymin(AestheticDomain::Continuous),
                ]))
                .aes(|a| {
                    a.xmin("xmin_min");
                    a.xmax("xmin_max");
                    a.ymin("ymin_min");
                    a.ymax("ymin_max");
                    a.fill_discrete("cyl");
                })
                .alpha(0.5);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_rects_2.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }
}
