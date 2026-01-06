use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::builder::{
    AesMapBuilder, AlphaContinuousAesBuilder, AlphaDiscreteAesBuilder, ColorContinuousAesBuilder,
    ColorDiscreteAesBuilder, LabelAesBuilder, SizeContinuousAesBuilder, SizeDiscreteAesBuilder,
    XContinuousAesBuilder, XDiscreteAesBuilder, YContinuousAesBuilder, YDiscreteAesBuilder,
};
use crate::aesthetics::{AesMap, AestheticProperty};
use crate::error::Result;
use crate::geom::properties::{Property, PropertyValue, PropertyVector};
use crate::geom::{AestheticRequirement, DomainConstraint};
use crate::layer::{Layer, LayerBuilder, LayerBuilderCore};
use crate::scale::ScaleIdentifier;
use crate::stat::Stat;
use crate::theme::{Color, TextElement, color};

pub trait GeomLabelAesBuilderTrait:
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

impl GeomLabelAesBuilderTrait for AesMapBuilder {}

pub struct GeomLabelBuilder {
    core: LayerBuilderCore,
    text: TextElement,
    fill: Option<Color>,
    angle: Option<f64>,
    padding: Option<f64>,
    radius: Option<f64>,
}

impl crate::theme::traits::TextElement for GeomLabelBuilder {
    fn this(&self) -> &TextElement {
        &self.text
    }

    fn this_mut(&mut self) -> &mut TextElement {
        &mut self.text
    }
}

impl GeomLabelBuilder {
    pub fn new() -> Self {
        Self {
            core: LayerBuilderCore::default(),
            text: TextElement::default(),
            fill: None,
            angle: None,
            padding: None,
            radius: None,
        }
    }

    pub fn fill<F: Into<Color>>(mut self, fill: F) -> Self {
        self.fill = Some(fill.into());
        self
    }

    /// Set text rotation angle in degrees
    pub fn angle(mut self, angle: f64) -> Self {
        self.angle = Some(angle);
        self
    }

    /// Set padding around text in label box
    pub fn padding(mut self, padding: f64) -> Self {
        self.padding = Some(padding.max(0.0));
        self
    }

    /// Set corner radius for rounded label boxes
    pub fn radius(mut self, radius: f64) -> Self {
        self.radius = Some(radius.max(0.0));
        self
    }

    pub fn aes(mut self, closure: impl FnOnce(&mut dyn GeomLabelAesBuilderTrait)) -> Self {
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

impl LayerBuilder for GeomLabelBuilder {
    fn build(self: Box<Self>, parent_mapping: &AesMap) -> Result<Layer> {
        let mut geom_label = GeomLabel::new();
        geom_label.text = self.text;

        // Set fixed property values and remove from inherited mapping
        let mut overrides = Vec::new();

        geom_label.text.overrides(&mut overrides);

        if self.fill.is_some() {
            geom_label.fill = self.fill;
        }

        geom_label.angle = self.angle.unwrap_or(0.0);
        geom_label.padding = self.padding.unwrap_or(2.0);
        geom_label.radius = self.radius.unwrap_or(2.0);

        LayerBuilderCore::build(
            self.core,
            parent_mapping,
            Box::new(geom_label),
            HashMap::new(),
            &overrides,
        )
    }
}

impl Default for GeomLabelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub fn geom_label() -> GeomLabelBuilder {
    GeomLabelBuilder::new()
}

/// GeomLabel renders text labels with a background box at specified positions
pub struct GeomLabel {
    /// Text element properties (color, size, alpha, font family/weight/style, hjust, vjust)
    pub text: TextElement,

    /// Fill color for label background
    pub fill: Option<Color>,

    /// Text rotation angle in degrees
    pub angle: f64,

    /// Padding around text in the label box (in points)
    pub padding: f64,

    /// Corner radius for rounded label boxes (0 = sharp corners)
    pub radius: f64,
}

impl GeomLabel {
    pub fn new() -> Self {
        Self {
            text: TextElement::default(),
            fill: None,
            angle: 0.0,
            padding: 2.0,
            radius: 2.0,
        }
    }

    fn draw_label(
        &self,
        ctx: &mut RenderContext,
        x_values: impl Iterator<Item = f64>,
        y_values: impl Iterator<Item = f64>,
        label_values: impl Iterator<Item = String>,
        color_values: impl Iterator<Item = Color>,
        size_values: impl Iterator<Item = f64>,
        alpha_values: impl Iterator<Item = f64>,
        fill_values: impl Iterator<Item = Color>,
    ) -> Result<()> {
        for ((((((x_norm, y_norm), label), color), size), alpha), fill) in x_values
            .zip(y_values)
            .zip(label_values)
            .zip(color_values)
            .zip(size_values)
            .zip(alpha_values)
            .zip(fill_values)
        {
            let x_px = ctx.map_x(x_norm);
            let y_px = ctx.map_y(y_norm);

            // Map size to visual range
            let visual_size = if size > 6.0 {
                size
            } else {
                8.0 + (size - 1.0) * (24.0 - 8.0) / (6.0 - 1.0)
            };

            ctx.cairo.set_font_size(visual_size);
            let extents = ctx.cairo.text_extents(&label).ok();

            if let Some(extents) = extents {
                // Save context for rotation and translation
                ctx.cairo.save().ok();
                ctx.cairo.translate(x_px, y_px);

                if self.angle != 0.0 {
                    ctx.cairo.rotate(self.angle.to_radians());
                }

                // Calculate text position (same as text geom)
                let text_x = -extents.width() * self.text.hjust.unwrap_or(0.5);
                let text_y = extents.height() * (1.0 - self.text.vjust.unwrap_or(0.5));

                // Calculate box dimensions and position around the text
                let box_width = extents.width() + 2.0 * self.padding;
                let box_height = extents.height() + 2.0 * self.padding;
                let box_x = text_x - self.padding;
                let box_y = text_y - extents.height() - self.padding;

                // Draw background box with rounded corners
                let Color(fr, fg, fb, fa) = fill;
                ctx.cairo.set_source_rgba(
                    fr as f64 / 255.0,
                    fg as f64 / 255.0,
                    fb as f64 / 255.0,
                    fa as f64 / 255.0 * alpha,
                );

                if self.radius > 0.0 {
                    // Draw rounded rectangle
                    let r = self.radius.min(box_width / 2.0).min(box_height / 2.0);
                    ctx.cairo.new_path();
                    ctx.cairo.arc(
                        box_x + r,
                        box_y + r,
                        r,
                        std::f64::consts::PI,
                        3.0 * std::f64::consts::PI / 2.0,
                    );
                    ctx.cairo.arc(
                        box_x + box_width - r,
                        box_y + r,
                        r,
                        3.0 * std::f64::consts::PI / 2.0,
                        0.0,
                    );
                    ctx.cairo.arc(
                        box_x + box_width - r,
                        box_y + box_height - r,
                        r,
                        0.0,
                        std::f64::consts::PI / 2.0,
                    );
                    ctx.cairo.arc(
                        box_x + r,
                        box_y + box_height - r,
                        r,
                        std::f64::consts::PI / 2.0,
                        std::f64::consts::PI,
                    );
                    ctx.cairo.close_path();
                } else {
                    // Draw sharp rectangle
                    ctx.cairo.rectangle(box_x, box_y, box_width, box_height);
                }

                ctx.cairo.fill().ok();

                // Draw text
                let Color(r, g, b, a) = color;
                ctx.cairo.set_source_rgba(
                    r as f64 / 255.0,
                    g as f64 / 255.0,
                    b as f64 / 255.0,
                    a as f64 / 255.0 * alpha,
                );

                // Set font family, weight, and style
                let family = self.text.family.as_deref().unwrap_or("sans-serif");
                
                let weight = match self.text.weight {
                    Some(crate::theme::FontWeight::Bold) => cairo::FontWeight::Bold,
                    Some(crate::theme::FontWeight::Light) => cairo::FontWeight::Normal,
                    _ => cairo::FontWeight::Normal,
                };
                
                let slant = match self.text.style {
                    Some(crate::theme::FontStyle::Italic) => cairo::FontSlant::Italic,
                    Some(crate::theme::FontStyle::Oblique) => cairo::FontSlant::Oblique,
                    _ => cairo::FontSlant::Normal,
                };
                
                ctx.cairo.select_font_face(family, slant, weight);

                ctx.cairo.move_to(text_x, text_y);
                ctx.cairo.show_text(&label).ok();

                ctx.cairo.restore().ok();
            }
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

impl Default for GeomLabel {
    fn default() -> Self {
        Self::new()
    }
}

impl Geom for GeomLabel {
    fn aesthetic_requirements(&self) -> &'static [AestheticRequirement] {
        &AESTHETIC_REQUIREMENTS
    }

    fn properties(&self) -> HashMap<AestheticProperty, Property> {
        let mut props = HashMap::new();
        self.text.properties(&mut props);
        props
    }

    fn property_defaults(
        &self,
        _theme: &crate::theme::Theme,
    ) -> HashMap<AestheticProperty, PropertyValue> {
        let mut defaults = HashMap::new();

        self.text.defaults("label", "text", _theme, &mut defaults);

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
            PropertyVector::LineStyle(linestyles) => {
                linestyles.into_iter().map(|ls| format!("{:?}", ls)).collect()
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

        // Get fill color (default to white if not specified)
        let fill_color = if let Some(fill_color) = self.fill {
            fill_color
        } else {
            color::WHITE
        };

        let n = x_values.len();
        let fill_values = vec![fill_color; n];

        self.draw_label(
            ctx,
            x_values.into_iter(),
            y_values.into_iter(),
            label_values.into_iter(),
            color_values.into_iter(),
            size_values.into_iter(),
            alpha_values.into_iter(),
            fill_values.into_iter(),
        )?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        data::{DataSource, VectorValue}, error::to_io_error, plot::plot, theme::traits::TextElement, utils::{dataframe::DataFrame, mtcars::mtcars}
    };

    fn init_test_logging() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn basic_label_1() {
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
        }) + geom_label().size(16.0).color(color::BLUE).fill(color::GRAY);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_label_1.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_label_2() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
            a.y_continuous("mpg");
            a.label("model");
        }) + geom_label().size(8.0).alpha(0.8).fill(color::LINEN);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_label_2.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_label_3() {
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
        }) + geom_label()
            .size(14.0)
            .hjust(0.0)
            .color(color::DARKGREEN)
            .fill(color::LIGHTGREEN)
            .padding(4.0);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_label_3.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_label_4() {
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
        }) + geom_label()
            .size(16.0)
            .angle(45.0)
            .color(color::RED)
            .fill(color::LIGHTBLUE)
            .radius(4.0);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_label_4.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_label_5() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("wt");
            a.y_continuous("mpg");
            a.label("cyl");
        }) + geom_label().fill(color::WHITE).aes(|a| {
            a.color_discrete("cyl");
            a.size_continuous("hp");
        });

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_label_5.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }
}
