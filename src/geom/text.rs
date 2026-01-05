use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::builder::{
    AesMapBuilder, AlphaContinuousAesBuilder, AlphaDiscreteAesBuilder, ColorContinuousAesBuilder,
    ColorDiscreteAesBuilder, LabelAesBuilder, SizeContinuousAesBuilder, SizeDiscreteAesBuilder,
    XContinuousAesBuilder, XDiscreteAesBuilder, YContinuousAesBuilder, YDiscreteAesBuilder,
};
use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain, AestheticProperty};
use crate::error::Result;
use crate::geom::properties::{Property, PropertyValue, PropertyVector};
use crate::geom::{AestheticRequirement, DomainConstraint};
use crate::layer::{Layer, LayerBuilder, LayerBuilderCore};
use crate::scale::ScaleIdentifier;
use crate::stat::Stat;
use crate::theme::{Color, color};

pub trait GeomTextAesBuilderTrait:
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
    + LabelAesBuilder
{
}

impl GeomTextAesBuilderTrait for AesMapBuilder {}

pub struct GeomTextBuilder {
    core: LayerBuilderCore,
    color: Option<Color>,
    size: Option<f64>,
    alpha: Option<f64>,
    hjust: Option<f64>,
    vjust: Option<f64>,
    angle: Option<f64>,
}

impl GeomTextBuilder {
    pub fn new() -> Self {
        Self {
            core: LayerBuilderCore::default(),
            color: None,
            size: None,
            alpha: None,
            hjust: None,
            vjust: None,
            angle: None,
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

    /// Set horizontal justification (0 = left, 0.5 = center, 1 = right)
    pub fn hjust(mut self, hjust: f64) -> Self {
        self.hjust = Some(hjust.clamp(0.0, 1.0));
        self
    }

    /// Set vertical justification (0 = bottom, 0.5 = middle, 1 = top)
    pub fn vjust(mut self, vjust: f64) -> Self {
        self.vjust = Some(vjust.clamp(0.0, 1.0));
        self
    }

    /// Set text rotation angle in degrees
    pub fn angle(mut self, angle: f64) -> Self {
        self.angle = Some(angle);
        self
    }

    pub fn aes(mut self, closure: impl FnOnce(&mut dyn GeomTextAesBuilderTrait)) -> Self {
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

impl LayerBuilder for GeomTextBuilder {
    fn build(self: Box<Self>, parent_mapping: &AesMap) -> Result<Layer> {
        let mut geom_text = GeomText::new();

        // Set fixed property values and remove from inherited mapping
        let mut overrides = Vec::new();

        if self.color.is_some() {
            geom_text.color = self.color;
            overrides.push(Aesthetic::Color(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Color(AestheticDomain::Discrete));
        }
        if self.size.is_some() {
            geom_text.size = self.size;
            overrides.push(Aesthetic::Size(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Size(AestheticDomain::Discrete));
        }
        if self.alpha.is_some() {
            geom_text.alpha = self.alpha;
            overrides.push(Aesthetic::Alpha(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Alpha(AestheticDomain::Discrete));
        }

        geom_text.hjust = self.hjust.unwrap_or(0.5);
        geom_text.vjust = self.vjust.unwrap_or(0.5);
        geom_text.angle = self.angle.unwrap_or(0.0);

        LayerBuilderCore::build(
            self.core,
            parent_mapping,
            Box::new(geom_text),
            HashMap::new(),
            &overrides,
        )
    }
}

impl Default for GeomTextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn geom_text() -> GeomTextBuilder {
    GeomTextBuilder::new()
}

/// GeomText renders text labels at specified positions
pub struct GeomText {
    /// Default color
    pub color: Option<Color>,

    /// Default text size
    pub size: Option<f64>,

    /// Default alpha/opacity
    pub alpha: Option<f64>,

    /// Horizontal justification (0 = left, 0.5 = center, 1 = right)
    pub hjust: f64,

    /// Vertical justification (0 = bottom, 0.5 = middle, 1 = top)
    pub vjust: f64,

    /// Text rotation angle in degrees
    pub angle: f64,
}

impl GeomText {
    pub fn new() -> Self {
        Self {
            color: None,
            size: None,
            alpha: None,
            hjust: 0.5,
            vjust: 0.5,
            angle: 0.0,
        }
    }

    fn draw_text(
        &self,
        ctx: &mut RenderContext,
        x_values: impl Iterator<Item = f64>,
        y_values: impl Iterator<Item = f64>,
        label_values: impl Iterator<Item = String>,
        color_values: impl Iterator<Item = Color>,
        size_values: impl Iterator<Item = f64>,
        alpha_values: impl Iterator<Item = f64>,
    ) -> Result<()> {
        // All values are already normalized [0,1] by scales
        for (((((x_norm, y_norm), label), color), size), alpha) in x_values
            .zip(y_values)
            .zip(label_values)
            .zip(color_values)
            .zip(size_values)
            .zip(alpha_values)
        {
            let x_px = ctx.map_x(x_norm);
            let y_px = ctx.map_y(y_norm);

            // Size scale maps to range (1.0, 6.0) by default, which is too small for text.
            // Map it to a better text range: (1.0, 6.0) -> (8.0, 24.0)
            // If size > 6.0, it's likely a constant size, use it directly
            let visual_size = if size > 6.0 {
                size
            } else {
                // Linear map from [1.0, 6.0] to [8.0, 24.0]
                8.0 + (size - 1.0) * (24.0 - 8.0) / (6.0 - 1.0)
            };

            log::debug!(
                "Drawing text '{}' at x_norm={}, y_norm={}, x_px={}, y_px={}, size={}, visual_size={}, color={:?}, alpha={}",
                label,
                x_norm,
                y_norm,
                x_px,
                y_px,
                size,
                visual_size,
                color,
                alpha
            );

            let Color(r, g, b, a) = color;
            ctx.cairo.set_source_rgba(
                r as f64 / 255.0,
                g as f64 / 255.0,
                b as f64 / 255.0,
                a as f64 / 255.0 * alpha,
            );

            ctx.cairo.set_font_size(visual_size);

            // Save context for rotation
            ctx.cairo.save().ok();

            // Move to position
            ctx.cairo.translate(x_px, y_px);

            // Rotate if needed
            if self.angle != 0.0 {
                ctx.cairo.rotate(self.angle.to_radians());
            }

            // Get text extents for positioning
            let extents = ctx.cairo.text_extents(&label).ok();
            if let Some(extents) = extents {
                let x_offset = -extents.width() * self.hjust;
                let y_offset = extents.height() * (1.0 - self.vjust);

                ctx.cairo.move_to(x_offset, y_offset);
                ctx.cairo.show_text(&label).ok();
            }

            // Restore context
            ctx.cairo.restore().ok();
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
        property: AestheticProperty::Label,
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

impl Geom for GeomText {
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
        props
    }

    fn property_defaults(
        &self,
        _theme: &crate::prelude::Theme,
    ) -> HashMap<AestheticProperty, PropertyValue> {
        let mut defaults = HashMap::new();

        // Only provide defaults for properties not explicitly set
        if self.color.is_none() {
            defaults.insert(AestheticProperty::Color, PropertyValue::Color(color::BLACK));
        }
        if self.size.is_none() {
            defaults.insert(AestheticProperty::Size, PropertyValue::Float(12.0));
        }
        if self.alpha.is_none() {
            defaults.insert(AestheticProperty::Alpha, PropertyValue::Float(1.0));
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

        // Extract label values (should be strings, but convert if needed)
        let label_prop = properties.remove(&AestheticProperty::Label).unwrap();

        let label_values = match label_prop {
            PropertyVector::String(strings) => strings,
            PropertyVector::Int(ints) => ints.into_iter().map(|i| i.to_string()).collect(),
            PropertyVector::Float(floats) => {
                floats.into_iter().map(|f| format!("{:.2}", f)).collect()
            }
            PropertyVector::Color(colors) => {
                colors.into_iter().map(|c| format!("{:?}", c)).collect()
            }
            PropertyVector::Shape(shapes) => {
                shapes.into_iter().map(|s| format!("{:?}", s)).collect()
            }
        };

        let color_prop = properties.remove(&AestheticProperty::Color).unwrap();
        let color_values = color_prop.to_color().as_colors();

        let size_values = properties
            .remove(&AestheticProperty::Size)
            .unwrap()
            .as_floats();

        let alpha_values = properties
            .remove(&AestheticProperty::Alpha)
            .unwrap()
            .as_floats();

        self.draw_text(
            ctx,
            x_values.into_iter(),
            y_values.into_iter(),
            label_values.into_iter(),
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
        data::{DataSource, VectorValue},
        error::to_io_error,
        plot::plot,
        utils::{dataframe::DataFrame, mtcars::mtcars},
    };

    fn init_test_logging() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn basic_text_1() {
        init_test_logging();

        let data: Box<dyn DataSource> = Box::new(DataFrame::from_columns(vec![
            ("x", VectorValue::from(vec![1.0, 2.0, 3.0, 4.0, 5.0])),
            ("y", VectorValue::from(vec![1.0, 2.0, 3.0, 4.0, 5.0])),
            ("label", VectorValue::from(vec!["A", "B", "C", "D", "E"])),
        ]));

        let builder = plot(&data).aes(|a| {
            a.x_continuous("x");
            a.y_continuous("y");
            a.label("label");
        }) + geom_text().size(16.0).color(color::BLUE);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_text_1.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_text_2() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
            a.y_continuous("mpg");
            a.label("model");
        }) + geom_text().size(8.0).alpha(0.7);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_text_2.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_text_3() {
        init_test_logging();

        let x = vec![1.0, 2.0, 3.0];
        let y = vec![1.0, 2.0, 3.0];
        let labels = vec!["Left", "Center", "Right"];

        let data: Box<dyn DataSource> = Box::new(DataFrame::from_columns(vec![
            ("x", VectorValue::from(x)),
            ("y", VectorValue::from(y)),
            ("label", VectorValue::from(labels)),
        ]));

        let builder = plot(&data).aes(|a| {
            a.x_continuous("x");
            a.y_continuous("y");
            a.label("label");
        }) + geom_text().size(14.0).hjust(0.0).color(color::DARKGREEN);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_text_3.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_text_4() {
        init_test_logging();

        let x = vec![1.0, 2.0, 3.0, 4.0];
        let y = vec![1.0, 2.0, 3.0, 4.0];
        let labels = vec!["0°", "45°", "90°", "-45°"];

        let data: Box<dyn DataSource> = Box::new(DataFrame::from_columns(vec![
            ("x", VectorValue::from(x)),
            ("y", VectorValue::from(y)),
            ("label", VectorValue::from(labels)),
        ]));

        let builder = plot(&data).aes(|a| {
            a.x_continuous("x");
            a.y_continuous("y");
            a.label("label");
        }) + geom_text().size(16.0).angle(45.0).color(color::RED);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_text_4.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_text_5() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
            a.y_continuous("mpg");
            a.label("cyl");
        }) + geom_text().aes(|a| {
            a.color_discrete("cyl");
            a.size_continuous("hp");
        });

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_text_5.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }
}
