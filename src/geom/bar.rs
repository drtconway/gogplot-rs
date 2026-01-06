use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::builder::{
    AesMapBuilder, AlphaContinuousAesBuilder, AlphaDiscreteAesBuilder, FillContinuousAesBuilder,
    FillDiscreteAesBuilder, GroupAesBuilder, XContinuousAesBuilder, XDiscreteAesBuilder,
    YContinuousAesBuilder, YDiscreteAesBuilder,
};
use crate::aesthetics::{AesMap, AestheticProperty};
use crate::data::PrimitiveValue;
use crate::error::Result;
use crate::geom::properties::{Property, PropertyValue, PropertyVector};
use crate::geom::{AestheticRequirement, DomainConstraint};
use crate::layer::{Layer, LayerBuilder, LayerBuilderCore};
use crate::scale::ScaleIdentifier;
use crate::scale::traits::{ContinuousDomainScale, ScaleBase};
use crate::stat::Stat;
use crate::theme::{AreaElement, Color};

pub trait GeomBarAesBuilderTrait:
    XContinuousAesBuilder
    + XDiscreteAesBuilder
    + YContinuousAesBuilder
    + YDiscreteAesBuilder
    + FillContinuousAesBuilder
    + FillDiscreteAesBuilder
    + AlphaContinuousAesBuilder
    + AlphaDiscreteAesBuilder
    + GroupAesBuilder
{
}

impl GeomBarAesBuilderTrait for AesMapBuilder {}

pub struct GeomBarBuilder {
    core: LayerBuilderCore,
    area: AreaElement,
    width: f64,
}

impl GeomBarBuilder {
    pub fn new() -> Self {
        Self {
            core: LayerBuilderCore::default(),
            area: AreaElement::default(),
            width: 0.9,
        }
    }

    /// Set the bar width (as a proportion of spacing, typically 0.0-1.0)
    pub fn width(mut self, width: f64) -> Self {
        self.width = width.max(0.0);
        self
    }

    pub fn aes(mut self, closure: impl FnOnce(&mut dyn GeomBarAesBuilderTrait)) -> Self {
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

    pub fn position(mut self, position: &str) -> Self {
        self.core.position = Some(position.into());
        self
    }
}

impl crate::theme::traits::AreaElement for GeomBarBuilder {
    fn this(&self) -> &AreaElement {
        &self.area
    }

    fn this_mut(&mut self) -> &mut AreaElement {
        &mut self.area
    }
}

impl LayerBuilder for GeomBarBuilder {
    fn build(self: Box<Self>, parent_mapping: &AesMap) -> Result<Layer> {
        let mut geom_bar = GeomBar::new();
        geom_bar.area = self.area;
        geom_bar.width = self.width;

        // Build the mapping (merging layer + parent)
        let mut overrides = Vec::new();

        geom_bar.area.overrides(&mut overrides);

        LayerBuilderCore::build(
            self.core,
            parent_mapping,
            Box::new(geom_bar),
            HashMap::new(),
            &overrides,
        )
    }
}

pub fn geom_bar() -> GeomBarBuilder {
    GeomBarBuilder::new()
}

/// GeomBar renders bars from y=0 to y=value
///
/// Bars are drawn as rectangles from the baseline (y=0) to the y value.
/// The x position determines the center of each bar, and width controls
/// the bar width as a proportion of the spacing between x values.
///
/// # Required Aesthetics
///
/// - `X`: Position along x-axis (discrete or continuous)
/// - `Y`: Height of the bar
///
/// # Optional Aesthetics
///
/// - `Fill`: Bar fill color (can be constant or mapped to data)
/// - `Color`: Bar border color (can be constant or mapped to data)
/// - `Alpha`: Bar transparency (0.0 = transparent, 1.0 = opaque)
pub struct GeomBar {
    area: AreaElement,

    /// Bar width (as a proportion of the spacing between x values)
    pub width: f64,
}

impl GeomBar {
    /// Create a new bar geom with default settings
    pub fn new() -> Self {
        Self {
            area: AreaElement::default(),
            width: 0.9,
        }
    }

    fn draw_bars(
        &self,
        ctx: &mut RenderContext,
        x_values: &[f64],
        y_values: &[f64],
        color_values: &[Color],
        fill_values: &[Color],
        alpha_values: &[f64],
        x_offset: Option<&[f64]>,
        width_factor: Option<&[f64]>,
        y_offset: Option<&[f64]>,
    ) -> Result<()> {
        if x_values.is_empty() {
            return Ok(());
        }

        // Use pre-calculated bar width if available (for continuous x),
        // otherwise calculate from unique x positions (for discrete x)
        let (spacing, base_bar_width) = {
            // Find unique x positions to determine spacing
            let mut unique_x: Vec<f64> = x_values.to_vec();
            unique_x.sort_by(|a, b| a.partial_cmp(b).unwrap());
            unique_x.dedup();

            // Calculate spacing between bars
            // For discrete x with n categories, spacing in normalized coords is 1.0/n
            let spacing = if unique_x.len() > 1 {
                // Calculate average spacing from actual data
                let mut spacings = Vec::new();
                for i in 1..unique_x.len() {
                    spacings.push(unique_x[i] - unique_x[i - 1]);
                }
                spacings.iter().sum::<f64>() / spacings.len() as f64
            } else {
                // Single bar - use reasonable default
                0.2
            };

            (spacing, spacing * self.width)
        };

        for i in 0..x_values.len() {
            // Apply optional x offset from position adjustment (e.g., dodge)
            // XOffset is a fraction of spacing, so multiply by actual spacing
            let x_center =
                x_values[i] + x_offset.map(|offsets| offsets[i] * spacing).unwrap_or(0.0);

            // Apply optional width scaling factor from position adjustment
            // Width is a scaling factor applied to the base bar width
            let bar_width = base_bar_width * width_factor.map(|factors| factors[i]).unwrap_or(1.0);

            let y_top = y_values[i];
            let color = color_values[i];
            let fill = fill_values[i];
            let alpha = alpha_values[i];

            // Calculate bar bounds in normalized space
            let x_left = x_center - bar_width / 2.0;
            let x_right = x_center + bar_width / 2.0;

            // Use YOffset for bar bottom if available (for stacking), otherwise use 0
            let y_bottom = y_offset.map(|offsets| offsets[i]).unwrap_or(0.0);

            // Convert to pixel coordinates
            let x_left_px = ctx.map_x(x_left);
            let x_right_px = ctx.map_x(x_right);
            let y_top_px = ctx.map_y(y_top);
            let y_bottom_px = ctx.map_y(y_bottom);

            let width = x_right_px - x_left_px;
            let height = y_bottom_px - y_top_px; // Note: y is inverted in screen coords

            // Draw filled rectangle
            let Color(r, g, b, a) = fill;
            ctx.cairo.set_source_rgba(
                r as f64 / 255.0,
                g as f64 / 255.0,
                b as f64 / 255.0,
                a as f64 / 255.0 * alpha,
            );
            ctx.cairo.rectangle(x_left_px, y_top_px, width, height);
            ctx.cairo.fill().ok();

            // Draw border/stroke
            let Color(r, g, b, a) = color;
            ctx.cairo.set_source_rgba(
                r as f64 / 255.0,
                g as f64 / 255.0,
                b as f64 / 255.0,
                a as f64 / 255.0 * alpha,
            );
            ctx.cairo.set_line_width(1.0);
            ctx.cairo.rectangle(x_left_px, y_top_px, width, height);
            ctx.cairo.stroke().ok();
        }

        Ok(())
    }
}

impl Default for GeomBar {
    fn default() -> Self {
        Self::new()
    }
}

const AESTHETIC_REQUIREMENTS: [AestheticRequirement; 5] = [
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
];

impl Geom for GeomBar {
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
        theme: &crate::prelude::Theme,
    ) -> HashMap<AestheticProperty, PropertyValue> {
        let mut defaults = HashMap::new();
        self.area.defaults("bar", "bar", theme, &mut defaults);
        defaults
    }

    fn required_scales(&self) -> Vec<ScaleIdentifier> {
        vec![ScaleIdentifier::XContinuous, ScaleIdentifier::YContinuous]
    }

    fn train_scales(&self, scales: &mut crate::scale::ScaleSet) {
        // Bars need to ensure the y-scale includes 0 (the baseline)
        // Train with a single 0.0 value to expand the domain if needed
        scales.y_continuous.train_one(&PrimitiveValue::Float(0.0));

        // For continuous x scales, adjust the domain to ensure proper bar spacing
        // We want equal padding on both sides equal to half the bar spacing
        if let Some((original_min, original_max)) = scales.x_continuous.domain() {
            // Remove the existing expansion to get back to data range
            let range = original_max - original_min;
            let expansion = range * 0.05 / 1.10; // Reverse the 5% expansion on each side
            let data_min = original_min + expansion;
            let data_max = original_max - expansion;

            // Calculate the data spacing (assumes uniform spacing of 1.0)
            let data_spacing = 1.0;
            let half_spacing = data_spacing / 2.0;

            // Set domain that adds half-spacing on each side
            let new_min = data_min - half_spacing;
            let new_max = data_max + half_spacing;
            scales.x_continuous.set_domain((new_min, new_max));
        }
    }

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

        // Extract optional position adjustment aesthetics
        let x_offset = properties
            .remove(&AestheticProperty::XOffset)
            .map(|v| v.as_floats());

        let width_factor = properties
            .remove(&AestheticProperty::Width)
            .map(|v| v.as_floats());

        let y_offset = properties
            .remove(&AestheticProperty::YOffset)
            .map(|v| v.as_floats());

        self.draw_bars(
            ctx,
            &x_values,
            &y_values,
            &color_values,
            &fill_values,
            &alpha_values,
            x_offset.as_deref(),
            width_factor.as_deref(),
            y_offset.as_deref(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{DataSource, VectorValue};
    use crate::error::to_io_error;
    use crate::plot::plot;
    use crate::stat::count::Count;
    use crate::theme::color;
    use crate::theme::traits::AreaElement;
    use crate::utils::dataframe::DataFrame;
    use crate::utils::mtcars::mtcars;

    fn init_test_logging() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn basic_bar_1() {
        init_test_logging();

        let categories = vec!["A", "B", "C", "D", "E"];
        let values = vec![3.0, 7.0, 5.0, 9.0, 4.0];

        let data: Box<dyn DataSource> = Box::new(DataFrame::from_columns(vec![
            ("category", VectorValue::from(categories)),
            ("value", VectorValue::from(values)),
        ]));

        let builder = plot(&data).aes(|a| {
            a.x_discrete("category");
            a.y_continuous("value");
        }) + geom_bar().fill(color::STEELBLUE);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_bar_1.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_bar_2() {
        init_test_logging();

        let categories = vec!["Q1", "Q2", "Q3", "Q4"];
        let values = vec![23.5, 28.3, 31.2, 26.8];

        let data: Box<dyn DataSource> = Box::new(DataFrame::from_columns(vec![
            ("quarter", VectorValue::from(categories)),
            ("sales", VectorValue::from(values)),
        ]));

        let builder = plot(&data).aes(|a| {
            a.x_discrete("quarter");
            a.y_continuous("sales");
        }) + geom_bar()
            .fill(color::CORAL)
            .color(color::DARKRED)
            .width(0.7);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_bar_2.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_bar_3() {
        init_test_logging();

        let categories = vec!["Red", "Green", "Blue", "Red", "Green", "Blue"];
        let groups = vec!["A", "A", "A", "B", "B", "B"];
        let values = vec![5.0, 7.0, 4.0, 6.0, 8.0, 5.0];

        let data: Box<dyn DataSource> = Box::new(DataFrame::from_columns(vec![
            ("category", VectorValue::from(categories)),
            ("group", VectorValue::from(groups)),
            ("value", VectorValue::from(values)),
        ]));

        let builder = plot(&data).aes(|a| {
            a.x_discrete("category");
            a.y_continuous("value");
        }) + geom_bar().aes(|a| {
            a.fill_discrete("category");
        });

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_bar_3.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_bar_4() {
        init_test_logging();

        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![2.3, 4.1, 3.5, 5.2, 4.8];

        let data: Box<dyn DataSource> = Box::new(DataFrame::from_columns(vec![
            ("x", VectorValue::from(x)),
            ("y", VectorValue::from(y)),
        ]));

        let builder = plot(&data).aes(|a| {
            a.x_continuous("x");
            a.y_continuous("y");
        }) + geom_bar().fill(color::FORESTGREEN).alpha(0.7);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_bar_4.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_bar_5() {
        init_test_logging();

        let groups = vec!["A", "A", "B", "B", "C", "C"];
        let categories = vec!["X", "Y", "X", "Y", "X", "Y"];
        let values = vec![5.0, 7.0, 6.0, 8.0, 4.0, 9.0];

        let data: Box<dyn DataSource> = Box::new(DataFrame::from_columns(vec![
            ("group", VectorValue::from(groups)),
            ("category", VectorValue::from(categories)),
            ("value", VectorValue::from(values)),
        ]));

        let builder = plot(&data).aes(|a| {
            a.x_discrete("group");
            a.y_continuous("value");
        }) + geom_bar().width(0.8).aes(|a| {
            a.fill_discrete("category");
            a.alpha_discrete("category");
        });

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_bar_5.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_bar_6() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_discrete("cyl");
            a.fill_discrete("gear");
        }) + geom_bar().stat(Count::default()).position("dodge");

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_bar_6.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_bar_7() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_discrete("cyl");
            a.fill_discrete("gear");
        }) + geom_bar().stat(Count::default()).position("stack");

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_bar_7.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_bar_8() {
        init_test_logging();

        // Create data with large stacked values to demonstrate the overflow issue
        // Three categories, each with 4 groups that stack to large totals
        let categories = vec!["A", "A", "A", "A", "B", "B", "B", "B", "C", "C", "C", "C"];
        let groups = vec!["G1", "G2", "G3", "G4", "G1", "G2", "G3", "G4", "G1", "G2", "G3", "G4"];
        let values = vec![
            25.0, 30.0, 35.0, 40.0,  // A stacks to 130
            20.0, 25.0, 30.0, 35.0,  // B stacks to 110
            30.0, 35.0, 40.0, 45.0,  // C stacks to 150
        ];

        let data: Box<dyn DataSource> = Box::new(DataFrame::from_columns(vec![
            ("category", VectorValue::from(categories)),
            ("group", VectorValue::from(groups)),
            ("value", VectorValue::from(values)),
        ]));

        let builder = plot(&data).aes(|a| {
            a.x_discrete("category");
            a.y_continuous("value");
            a.fill_discrete("group");
        }) + geom_bar().position("stack");

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_bar_8.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }
}
