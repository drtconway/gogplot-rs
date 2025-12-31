use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::builder::{
    AesMapBuilder, AlphaContinuousAesBuilder, AlphaDiscreteAesBuilder, FillContinuousAesBuilder,
    FillDiscreteAesBuilder, XMaxContinuousAesBuilder, XMinContinuousAesBuilder,
    YMaxContinuousAesBuilder, YMinContinuousAesBuilder,
};
use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain, AestheticProperty};
use crate::error::Result;
use crate::geom::properties::{ColorProperty, FloatProperty, Property, PropertyVector};
use crate::geom::{AestheticRequirement, DomainConstraint};
use crate::layer::{Layer, LayerBuilder, LayerBuilderCore};
use crate::scale::ScaleIdentifier;
use crate::theme::{Color, color};

pub trait GeomRectAesBuilderTrait:
    XMinContinuousAesBuilder
    + XMaxContinuousAesBuilder
    + YMinContinuousAesBuilder
    + YMaxContinuousAesBuilder
    + FillContinuousAesBuilder
    + FillDiscreteAesBuilder
    + AlphaContinuousAesBuilder
    + AlphaDiscreteAesBuilder
{
}

impl GeomRectAesBuilderTrait for AesMapBuilder {}

pub struct GeomRectBuilder {
    core: LayerBuilderCore,
    fill: Option<ColorProperty>,
    alpha: Option<FloatProperty>,
}

impl GeomRectBuilder {
    pub fn new() -> Self {
        Self {
            core: LayerBuilderCore::default(),
            fill: None,
            alpha: None,
        }
    }

    pub fn fill<Fill: Into<ColorProperty>>(mut self, fill: Fill) -> Self {
        self.fill = Some(fill.into());
        self
    }

    pub fn alpha<Alpha: Into<FloatProperty>>(mut self, alpha: Alpha) -> Self {
        self.alpha = Some(alpha.into());
        self
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
}

impl LayerBuilder for GeomRectBuilder {
    fn build(self: Box<Self>, parent_mapping: &AesMap) -> Result<Layer> {
        let mut geom_rect = GeomRect::new();

        // Build the mapping (merging layer + parent)
        let mut overrides = Vec::new();

        // Set fixed property values and remove from inherited mapping
        if self.fill.is_some() {
            geom_rect.fill = self.fill;
            overrides.push(Aesthetic::Fill(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Fill(AestheticDomain::Discrete));
        }
        if self.alpha.is_some() {
            geom_rect.alpha = self.alpha;
            overrides.push(Aesthetic::Alpha(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Alpha(AestheticDomain::Discrete));
        }

        LayerBuilderCore::build(
            self.core,
            parent_mapping,
            Box::new(geom_rect),
            HashMap::new(),
            &overrides,
        )
    }
}

impl Default for GeomRectBuilder {
    fn default() -> Self {
        Self::new()
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
    /// Default fill color
    pub fill: Option<ColorProperty>,

    /// Default alpha/opacity
    pub alpha: Option<FloatProperty>,
}

impl GeomRect {
    /// Create a new rect geom with default theme values
    pub fn new() -> Self {
        Self {
            fill: None,
            alpha: None,
        }
    }

    /// Set the default fill color
    pub fn fill(&mut self, color: crate::theme::Color) -> &mut Self {
        self.fill = Some(ColorProperty::new().color(color).clone());
        self
    }

    /// Set the default alpha/opacity
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = Some(FloatProperty::new().value(alpha.clamp(0.0, 1.0)).clone());
        self
    }

    fn draw_rects(
        &self,
        ctx: &mut RenderContext,
        xmin_values: impl Iterator<Item = f64>,
        xmax_values: impl Iterator<Item = f64>,
        ymin_values: impl Iterator<Item = f64>,
        ymax_values: impl Iterator<Item = f64>,
        fill_values: impl Iterator<Item = Color>,
        alpha_values: impl Iterator<Item = f64>,
    ) -> Result<()> {
        // All values are already normalized [0,1] by scales
        // Draw rectangles at the specified bounds
        for (((((xmin_norm, xmax_norm), ymin_norm), ymax_norm), fill), alpha) in xmin_values
            .zip(xmax_values)
            .zip(ymin_values)
            .zip(ymax_values)
            .zip(fill_values)
            .zip(alpha_values)
        {
            let xmin_px = ctx.map_x(xmin_norm);
            let xmax_px = ctx.map_x(xmax_norm);
            let ymin_px = ctx.map_y(ymin_norm);
            let ymax_px = ctx.map_y(ymax_norm);

            log::debug!(
                "Drawing rect at xmin_norm={}, xmax_norm={}, ymin_norm={}, ymax_norm={}, xmin_px={}, xmax_px={}, ymin_px={}, ymax_px={}, fill={:?}, alpha={}",
                xmin_norm,
                xmax_norm,
                ymin_norm,
                ymax_norm,
                xmin_px,
                xmax_px,
                ymin_px,
                ymax_px,
                fill,
                alpha
            );

            let Color(r, g, b, a) = fill;
            ctx.cairo.set_source_rgba(
                r as f64 / 255.0,
                g as f64 / 255.0,
                b as f64 / 255.0,
                a as f64 / 255.0 * alpha,
            );

            // Draw filled rectangle
            let width = xmax_px - xmin_px;
            let height = ymax_px - ymin_px;
            ctx.cairo.rectangle(xmin_px, ymin_px, width, height);
            ctx.cairo.fill().ok();
        }

        Ok(())
    }
}

impl Default for GeomRect {
    fn default() -> Self {
        Self::new()
    }
}

const AESTHETIC_REQUIREMENTS: [AestheticRequirement; 6] = [
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
        if let Some(fill_prop) = &self.fill {
            props.insert(AestheticProperty::Fill, Property::Color(fill_prop.clone()));
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
        if self.fill.is_none() {
            defaults.insert(
                AestheticProperty::Fill,
                super::properties::PropertyValue::Color(color::GREY50),
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
            fill_values.into_iter(),
            alpha_values.into_iter(),
        )?;

        Ok(())
    }
}
