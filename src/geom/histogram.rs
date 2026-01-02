use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::builder::{
    AesMapBuilder, AlphaContinuousAesBuilder, AlphaDiscreteAesBuilder, FillContinuousAesBuilder,
    FillDiscreteAesBuilder, GroupAesBuilder, XContininuousAesBuilder, YContininuousAesBuilder,
};
use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain, AestheticProperty};
use crate::data::PrimitiveValue;
use crate::error::Result;
use crate::geom::properties::{
    ColorProperty, FloatProperty, Property, PropertyValue, PropertyVector,
};
use crate::geom::{AestheticRequirement, DomainConstraint};
use crate::layer::{Layer, LayerBuilder, LayerBuilderCore};
use crate::scale::ScaleIdentifier;
use crate::scale::traits::ScaleBase;
use crate::stat::bin::Bin;
use crate::theme::Color;

pub trait GeomHistogramAesBuilderTrait:
    XContininuousAesBuilder
    + YContininuousAesBuilder
    + FillContinuousAesBuilder
    + FillDiscreteAesBuilder
    + AlphaContinuousAesBuilder
    + AlphaDiscreteAesBuilder
    + GroupAesBuilder
{
}

impl GeomHistogramAesBuilderTrait for AesMapBuilder {}

pub struct GeomHistogramBuilder {
    core: LayerBuilderCore,
    color: Option<ColorProperty>,
    fill: Option<ColorProperty>,
    alpha: Option<FloatProperty>,
}

impl GeomHistogramBuilder {
    pub fn new() -> Self {
        Self {
            core: LayerBuilderCore::default(),
            color: None,
            fill: None,
            alpha: None,
        }
    }

    pub fn color<C: Into<ColorProperty>>(mut self, color: C) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn fill<F: Into<ColorProperty>>(mut self, fill: F) -> Self {
        self.fill = Some(fill.into());
        self
    }

    pub fn alpha<A: Into<FloatProperty>>(mut self, alpha: A) -> Self {
        self.alpha = Some(alpha.into());
        self
    }

    pub fn aes(mut self, closure: impl FnOnce(&mut dyn GeomHistogramAesBuilderTrait)) -> Self {
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

impl LayerBuilder for GeomHistogramBuilder {
    fn build(mut self: Box<Self>, parent_mapping: &AesMap) -> Result<Layer> {
        let mut geom_histogram = GeomHistogram::new();

        // Build the mapping (merging layer + parent)
        let mut overrides = Vec::new();

        // Set fixed property values and remove from inherited mapping
        if self.color.is_some() {
            geom_histogram.color = self.color;
            overrides.push(Aesthetic::Color(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Color(AestheticDomain::Discrete));
        }
        if self.fill.is_some() {
            geom_histogram.fill = self.fill;
            overrides.push(Aesthetic::Fill(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Fill(AestheticDomain::Discrete));
        }
        if self.alpha.is_some() {
            geom_histogram.alpha = self.alpha;
            overrides.push(Aesthetic::Alpha(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Alpha(AestheticDomain::Discrete));
        }

        // Make Bin the default stat if none specified
        if self.core.stat.is_none() {
            self.core.stat = Some(Box::new(Bin::default()));
        }

        LayerBuilderCore::build(
            self.core,
            parent_mapping,
            Box::new(geom_histogram),
            HashMap::new(),
            &overrides,
        )
    }
}

pub fn geom_histogram() -> GeomHistogramBuilder {
    GeomHistogramBuilder::new()
}

/// GeomHistogram renders a histogram by binning continuous data
///
/// Histograms display the distribution of a continuous variable by dividing
/// the data into bins and showing the count or density in each bin as bars.
///
/// # Required Aesthetics
///
/// - `X`: Continuous variable to bin (bin centers after stat)
/// - `Y`: Count or density in each bin (computed by stat)
///
/// # Optional Aesthetics
///
/// - `Fill`: Bar fill color (can be constant or mapped to data)
/// - `Color`: Bar border color (can be constant or mapped to data)
/// - `Alpha`: Bar transparency (0.0 = transparent, 1.0 = opaque)
pub struct GeomHistogram {
    /// Default color (border)
    pub color: Option<ColorProperty>,

    /// Default fill color
    pub fill: Option<ColorProperty>,

    /// Default alpha/opacity
    pub alpha: Option<FloatProperty>,
}

impl GeomHistogram {
    /// Create a new histogram geom with default settings
    pub fn new() -> Self {
        Self {
            color: None,
            fill: None,
            alpha: None,
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

        // Calculate bar width and spacing from consecutive x values
        let (spacing, base_bar_width) = {
            let mut unique_x: Vec<f64> = x_values.to_vec();
            unique_x.sort_by(|a, b| a.partial_cmp(b).unwrap());
            unique_x.dedup();

            let spacing = if unique_x.len() > 1 {
                let mut spacings = Vec::new();
                for i in 1..unique_x.len() {
                    spacings.push(unique_x[i] - unique_x[i - 1]);
                }
                spacings.iter().sum::<f64>() / spacings.len() as f64
            } else {
                0.2
            };
            
            (spacing, spacing)
        };

        // Calculate the baseline (where y=0 in data space maps to in normalized space)
        // For histograms, the minimum y value should represent the baseline
        let y_baseline = y_values
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min)
            .max(0.0);

        for i in 0..x_values.len() {
            // Apply optional x offset from position adjustment (e.g., dodge)
            // XOffset is a fraction of spacing, so multiply by actual spacing
            let x_center = x_values[i] + x_offset.map(|offsets| offsets[i] * spacing).unwrap_or(0.0);
            
            // Apply optional width scaling factor from position adjustment
            let bar_width = base_bar_width * width_factor.map(|factors| factors[i]).unwrap_or(1.0);
            
            let y_top = y_values[i];
            let color = color_values[i];
            let fill = fill_values[i];
            let alpha = alpha_values[i];

            // Calculate bar bounds in normalized space
            let x_left = (x_center - bar_width / 2.0).max(0.0);
            let x_right = (x_center + bar_width / 2.0).min(1.0);
            
            // Use YOffset for bar bottom if available (for stacking), otherwise use baseline
            let y_bottom = y_offset.map(|offsets| offsets[i]).unwrap_or(y_baseline);
            let y_top_clamped = y_top.min(1.0);

            if y_top_clamped <= y_bottom {
                continue; // Skip bars with no height
            }

            // Convert to pixel coordinates
            let x_left_px = ctx.map_x(x_left);
            let x_right_px = ctx.map_x(x_right);
            let y_top_px = ctx.map_y(y_top_clamped);
            let y_bottom_px = ctx.map_y(y_bottom);

            let width = x_right_px - x_left_px;
            let height = y_bottom_px - y_top_px;

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

impl Default for GeomHistogram {
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
        constraint: DomainConstraint::MustBe(AestheticDomain::Continuous),
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

impl Geom for GeomHistogram {
    fn aesthetic_requirements(&self) -> &'static [AestheticRequirement] {
        &AESTHETIC_REQUIREMENTS
    }

    fn properties(&self) -> HashMap<AestheticProperty, Property> {
        let mut properties = HashMap::new();

        if let Some(ref color) = self.color {
            properties.insert(AestheticProperty::Color, Property::Color(color.clone()));
        }
        if let Some(ref fill) = self.fill {
            properties.insert(AestheticProperty::Fill, Property::Color(fill.clone()));
        }
        if let Some(ref alpha) = self.alpha {
            properties.insert(AestheticProperty::Alpha, Property::Float(alpha.clone()));
        }

        properties
    }

    fn property_defaults(
        &self,
        theme: &crate::theme::Theme,
    ) -> HashMap<AestheticProperty, PropertyValue> {
        let mut defaults = HashMap::new();

        if self.color.is_none() {
            defaults.insert(
                AestheticProperty::Color,
                PropertyValue::Color(theme.geom_rect.color),
            );
        }
        if self.alpha.is_none() {
            defaults.insert(
                AestheticProperty::Alpha,
                PropertyValue::Float(theme.geom_rect.alpha),
            );
        }

        defaults
    }

    fn required_scales(&self) -> Vec<ScaleIdentifier> {
        vec![ScaleIdentifier::XContinuous, ScaleIdentifier::YContinuous]
    }

    fn train_scales(&self, scales: &mut crate::scale::ScaleSet) {
        // Histograms need to ensure the y-scale includes 0 (the baseline)
        scales.y_continuous.train_one(&PrimitiveValue::Float(0.0));
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
    use crate::error::to_io_error;
    use crate::plot::plot;
    use crate::theme::color;
    use crate::utils::mtcars::mtcars;

    fn init_test_logging() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn basic_histogram_1() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("mpg");
        }) + geom_histogram().fill(color::STEELBLUE).alpha(0.7);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_histogram_1.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_histogram_2() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("mpg");
        }) + geom_histogram()
            .stat(Bin::with_width(2.0))
            .fill(color::STEELBLUE)
            .alpha(0.7);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_histogram_2.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_histogram_3() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("mpg");
            a.fill_discrete("cyl");
        }) + geom_histogram()
            .stat(Bin::with_width(2.0))
            .position("stack")
            .alpha(0.7);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_histogram_3.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_histogram_4() {
        init_test_logging();

        let data = mtcars();

        let builder = plot(&data).aes(|a| {
            a.x_continuous("mpg");
            a.fill_discrete("cyl");
        }) + geom_histogram()
            .stat(Bin::with_width(4.0))
            .position("dodge")
            .alpha(0.7);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_histogram_4.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }
}
