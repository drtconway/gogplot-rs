
use std::collections::HashMap;

use super::{Geom, RenderContext, AestheticRequirement, DomainConstraint};
use crate::aesthetics::builder::{
    AesMapBuilder, AlphaContinuousAesBuilder, AlphaDiscreteAesBuilder, FillContinuousAesBuilder,
    FillDiscreteAesBuilder, GroupAesBuilder, XDiscreteAesBuilder, YContininuousAesBuilder,
};
use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain, AestheticProperty};
use crate::error::{PlotError, Result};
use crate::geom::properties::{Property, PropertyValue, PropertyVector};
use crate::layer::{Layer, LayerBuilder, LayerBuilderCore};
use crate::scale::ScaleIdentifier;
use crate::stat::boxplot::Boxplot;
use crate::theme::{Color, color};

pub trait GeomBoxplotAesBuilderTrait:
    XDiscreteAesBuilder
    + YContininuousAesBuilder
    + FillContinuousAesBuilder
    + FillDiscreteAesBuilder
    + AlphaContinuousAesBuilder
    + AlphaDiscreteAesBuilder
    + GroupAesBuilder
{
}

impl GeomBoxplotAesBuilderTrait for AesMapBuilder {}

pub struct GeomBoxplotBuilder {
    core: LayerBuilderCore,
    color: Option<Color>,
    fill: Option<Color>,
    alpha: Option<f64>,
    width: f64,
}

impl GeomBoxplotBuilder {
    pub fn new() -> Self {
        Self {
            core: LayerBuilderCore::default(),
            color: None,
            fill: None,
            alpha: None,
            width: 0.75,
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

    pub fn width(mut self, width: f64) -> Self {
        self.width = width;
        self
    }

    pub fn aes(mut self, closure: impl FnOnce(&mut dyn GeomBoxplotAesBuilderTrait)) -> Self {
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

impl LayerBuilder for GeomBoxplotBuilder {
    fn build(mut self: Box<Self>, parent_mapping: &AesMap) -> Result<Layer> {
        let mut geom_boxplot = GeomBoxplot::new();

        // Build the mapping (merging layer + parent)
        let mut overrides = Vec::new();

        // Set fixed property values and remove from inherited mapping
        if self.color.is_some() {
            geom_boxplot.color = self.color;
            overrides.push(Aesthetic::Color(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Color(AestheticDomain::Discrete));
        }
        if self.fill.is_some() {
            geom_boxplot.fill = self.fill;
            overrides.push(Aesthetic::Fill(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Fill(AestheticDomain::Discrete));
        }
        if self.alpha.is_some() {
            geom_boxplot.alpha = self.alpha;
            overrides.push(Aesthetic::Alpha(AestheticDomain::Continuous));
            overrides.push(Aesthetic::Alpha(AestheticDomain::Discrete));
        }

        geom_boxplot.width = self.width;

        // Make Boxplot the default stat if none specified
        if self.core.stat.is_none() {
            self.core.stat = Some(Box::new(Boxplot::default()));
        }

        LayerBuilderCore::build(
            self.core,
            parent_mapping,
            Box::new(geom_boxplot),
            HashMap::new(),
            &overrides,
        )
    }
}

pub fn geom_boxplot() -> GeomBoxplotBuilder {
    GeomBoxplotBuilder::new()
}

/// GeomBoxplot renders box-and-whisker plots
///
/// Box-and-whisker plots display the distribution of a continuous variable.
/// They show five key statistics (computed by stat_boxplot):
/// - Ymin: Lower whisker extent
/// - Lower: First quartile (Q1)
/// - Middle: Median
/// - Upper: Third quartile (Q3)
/// - Ymax: Upper whisker extent
///
/// The box spans from Q1 to Q3, with a line at the median.
/// Whiskers extend to Ymin and Ymax (typically 1.5 * IQR from the box).
/// Outliers beyond the whiskers are shown as points.
///
/// # Required Aesthetics
///
/// When using Stat::Boxplot (default), only X and Y are required.
/// The stat computes Lower, Middle, Upper, Ymin, Ymax.
///
/// When using Stat::Identity, these are required:
/// - X: Position along x-axis (typically categorical)
/// - Lower: First quartile (Q1)
/// - Middle: Median
/// - Upper: Third quartile (Q3)
/// - Ymin: Lower whisker extent
/// - Ymax: Upper whisker extent
///
/// # Optional Aesthetics
///
/// - Fill: Box fill color (can be constant or mapped)
/// - Color: Box outline and whisker color
/// - Alpha: Transparency (0.0 = transparent, 1.0 = opaque)
/// - Size: Line width for box outline and whiskers
pub struct GeomBoxplot {
    /// Default fill color (if not mapped)
    pub fill: Option<Color>,

    /// Default stroke color (if not mapped)
    pub color: Option<Color>,

    /// Default alpha/opacity (if not mapped)
    pub alpha: Option<f64>,

    /// Box width (as proportion of spacing between x values)
    pub width: f64,
}

impl GeomBoxplot {
    /// Create a new boxplot geom with default settings
    pub fn new() -> Self {
        Self {
            fill: None,
            color: None,
            alpha: None,
            width: 0.75,
        }
    }
}

impl Default for GeomBoxplot {
    fn default() -> Self {
        Self::new()
    }
}

const AESTHETIC_REQUIREMENTS: [AestheticRequirement; 10] = [
    AestheticRequirement {
        property: AestheticProperty::X,
        required: true,
        constraint: DomainConstraint::Any,
    },
    AestheticRequirement {
        property: AestheticProperty::Lower,
        required: true,
        constraint: DomainConstraint::Any,
    },
    AestheticRequirement {
        property: AestheticProperty::Middle,
        required: true,
        constraint: DomainConstraint::Any,
    },
    AestheticRequirement {
        property: AestheticProperty::Upper,
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
        property: AestheticProperty::Y,
        required: false,  // Optional - for outliers
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

impl Geom for GeomBoxplot {
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
        props
    }

    fn property_defaults(&self, _theme: &crate::theme::Theme) -> HashMap<AestheticProperty, PropertyValue> {
        let mut defaults = HashMap::new();
        if self.color.is_none() {
            defaults.insert(AestheticProperty::Color, PropertyValue::Color(color::BLACK));
        }
        if self.fill.is_none() {
            defaults.insert(AestheticProperty::Fill, PropertyValue::Color(color::GRAY));
        }
        if self.alpha.is_none() {
            defaults.insert(AestheticProperty::Alpha, PropertyValue::Float(1.0));
        }
        defaults
    }

    fn required_scales(&self) -> Vec<ScaleIdentifier> {
        vec![ScaleIdentifier::XDiscrete, ScaleIdentifier::YContinuous]
    }

    fn train_scales(&self, _scales: &mut crate::scale::ScaleSet) {
        // Boxplots don't need additional scale training
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
            .expect("X values required for boxplot")
            .as_floats();

        let ymin_values = properties
            .remove(&AestheticProperty::YMin)
            .expect("YMin values required for boxplot")
            .as_floats();

        let lower_values = properties
            .remove(&AestheticProperty::Lower)
            .expect("Lower values required for boxplot")
            .as_floats();

        let middle_values = properties
            .remove(&AestheticProperty::Middle)
            .expect("Middle values required for boxplot")
            .as_floats();

        let upper_values = properties
            .remove(&AestheticProperty::Upper)
            .expect("Upper values required for boxplot")
            .as_floats();

        let ymax_values = properties
            .remove(&AestheticProperty::YMax)
            .expect("YMax values required for boxplot")
            .as_floats();

        let y_values = properties
            .remove(&AestheticProperty::Y)
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

        self.draw_boxplots(
            ctx,
            &x_values,
            &ymin_values,
            &lower_values,
            &middle_values,
            &upper_values,
            &ymax_values,
            y_values.as_deref(),
            &color_values,
            &fill_values,
            &alpha_values,
            x_offset.as_deref(),
            width_factor.as_deref(),
        )
    }
}

impl GeomBoxplot {
    fn draw_boxplots(
        &self,
        ctx: &mut RenderContext,
        x_values: &[f64],
        ymin_values: &[f64],
        lower_values: &[f64],
        middle_values: &[f64],
        upper_values: &[f64],
        ymax_values: &[f64],
        y_values: Option<&[f64]>,
        color_values: &[Color],
        fill_values: &[Color],
        alpha_values: &[f64],
        x_offset: Option<&[f64]>,
        width_factor: Option<&[f64]>,
    ) -> Result<()> {
        if x_values.is_empty() {
            return Ok(());
        }

        // Calculate spacing between unique x positions
        let (spacing, base_box_width) = {
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

            (spacing, spacing * self.width)
        };

        for i in 0..x_values.len() {
            // Skip rows that are outliers (no box statistics)
            if ymin_values[i].is_nan() || lower_values[i].is_nan() || middle_values[i].is_nan() 
                || upper_values[i].is_nan() || ymax_values[i].is_nan() {
                // This is an outlier point - draw it if Y is present
                if let Some(y_vals) = y_values {
                    if !y_vals[i].is_nan() {
                        self.draw_outlier(
                            ctx,
                            x_values[i],
                            y_vals[i],
                            &color_values[i],
                            alpha_values[i],
                            x_offset,
                            spacing,
                            i,
                        )?;
                    }
                }
                continue;
            }

            // Apply optional x offset from position adjustment (e.g., dodge)
            let x_center = x_values[i] + x_offset.map(|offsets| offsets[i] * spacing).unwrap_or(0.0);
            
            // Apply optional width scaling factor from position adjustment
            let box_width = base_box_width * width_factor.map(|factors| factors[i]).unwrap_or(1.0);

            self.draw_box(
                ctx,
                x_center,
                box_width,
                ymin_values[i],
                lower_values[i],
                middle_values[i],
                upper_values[i],
                ymax_values[i],
                &color_values[i],
                &fill_values[i],
                alpha_values[i],
            )?;
        }

        Ok(())
    }

    fn draw_box(
        &self,
        ctx: &mut RenderContext,
        x_center: f64,
        box_width: f64,
        ymin: f64,
        lower: f64,
        middle: f64,
        upper: f64,
        ymax: f64,
        color: &Color,
        fill: &Color,
        alpha: f64,
    ) -> Result<()> {
        let x_left = x_center - box_width / 2.0;
        let x_right = x_center + box_width / 2.0;

        // Convert to pixel coordinates
        let x_left_px = ctx.map_x(x_left);
        let x_right_px = ctx.map_x(x_right);
        let x_center_px = ctx.map_x(x_center);
        
        let ymin_px = ctx.map_y(ymin);
        let lower_px = ctx.map_y(lower);
        let middle_px = ctx.map_y(middle);
        let upper_px = ctx.map_y(upper);
        let ymax_px = ctx.map_y(ymax);

        let width = x_right_px - x_left_px;
        let box_height = lower_px - upper_px;  // Y inverted in screen coords

        // Draw the box (Q1 to Q3)
        let Color(r, g, b, a) = fill;
        ctx.cairo.set_source_rgba(
            *r as f64 / 255.0,
            *g as f64 / 255.0,
            *b as f64 / 255.0,
            *a as f64 / 255.0 * alpha,
        );
        ctx.cairo.rectangle(x_left_px, upper_px, width, box_height);
        ctx.cairo.fill_preserve().map_err(|e| PlotError::RenderError { 
            operation: "fill_preserve".to_string(), 
            message: e.to_string() 
        })?;

        // Draw box outline
        let Color(r, g, b, a) = color;
        ctx.cairo.set_source_rgba(
            *r as f64 / 255.0,
            *g as f64 / 255.0,
            *b as f64 / 255.0,
            *a as f64 / 255.0 * alpha,
        );
        ctx.cairo.set_line_width(1.0);
        ctx.cairo.stroke().map_err(|e| PlotError::RenderError { 
            operation: "stroke".to_string(), 
            message: e.to_string() 
        })?;

        // Draw median line
        ctx.cairo.move_to(x_left_px, middle_px);
        ctx.cairo.line_to(x_right_px, middle_px);
        ctx.cairo.stroke().map_err(|e| PlotError::RenderError { 
            operation: "stroke".to_string(), 
            message: e.to_string() 
        })?;

        // Draw lower whisker
        ctx.cairo.move_to(x_center_px, lower_px);
        ctx.cairo.line_to(x_center_px, ymin_px);
        ctx.cairo.stroke().map_err(|e| PlotError::RenderError { 
            operation: "stroke".to_string(), 
            message: e.to_string() 
        })?;

        // Draw lower whisker cap
        let cap_width = width * 0.5;
        ctx.cairo.move_to(x_center_px - cap_width / 2.0, ymin_px);
        ctx.cairo.line_to(x_center_px + cap_width / 2.0, ymin_px);
        ctx.cairo.stroke().map_err(|e| PlotError::RenderError { 
            operation: "stroke".to_string(), 
            message: e.to_string() 
        })?;

        // Draw upper whisker
        ctx.cairo.move_to(x_center_px, upper_px);
        ctx.cairo.line_to(x_center_px, ymax_px);
        ctx.cairo.stroke().map_err(|e| PlotError::RenderError { 
            operation: "stroke".to_string(), 
            message: e.to_string() 
        })?;

        // Draw upper whisker cap
        ctx.cairo.move_to(x_center_px - cap_width / 2.0, ymax_px);
        ctx.cairo.line_to(x_center_px + cap_width / 2.0, ymax_px);
        ctx.cairo.stroke().map_err(|e| PlotError::RenderError { 
            operation: "stroke".to_string(), 
            message: e.to_string() 
        })?;

        Ok(())
    }

    fn draw_outlier(
        &self,
        ctx: &mut RenderContext,
        x: f64,
        y: f64,
        color: &Color,
        alpha: f64,
        x_offset: Option<&[f64]>,
        spacing: f64,
        i: usize,
    ) -> Result<()> {
        // Apply x offset if present
        let x_center = x + x_offset.map(|offsets| offsets[i] * spacing).unwrap_or(0.0);
        
        let x_px = ctx.map_x(x_center);
        let y_px = ctx.map_y(y);

        // Draw outlier as a small circle
        let Color(r, g, b, a) = color;
        ctx.cairo.set_source_rgba(
            *r as f64 / 255.0,
            *g as f64 / 255.0,
            *b as f64 / 255.0,
            *a as f64 / 255.0 * alpha,
        );
        ctx.cairo.arc(x_px, y_px, 2.0, 0.0, 2.0 * std::f64::consts::PI);
        ctx.cairo.fill().map_err(|e| PlotError::RenderError { 
            operation: "fill".to_string(), 
            message: e.to_string() 
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aesthetics::builder::{XDiscreteAesBuilder, YContininuousAesBuilder};
    use crate::data::{DataSource, VectorValue};
    use crate::error::to_io_error;
    use crate::plot::plot;
    use crate::utils::dataframe::DataFrame;

    fn init_test_logging() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Debug)
            .try_init();
    }

    #[test]
    fn basic_boxplot_1() {
        init_test_logging();

        // Create sample data with different groups
        let groups = vec!["A", "A", "A", "A", "A", "A", "A", "A", "A", "A",
                         "B", "B", "B", "B", "B", "B", "B", "B", "B", "B",
                         "C", "C", "C", "C", "C", "C", "C", "C", "C", "C"];
        let values = vec![5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 25.0,  // A with outlier
                         10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0,  // B
                         3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 1.0];  // C with outlier

        let data: Box<dyn DataSource> = Box::new(DataFrame::from_columns(vec![
            ("group", VectorValue::from(groups)),
            ("value", VectorValue::from(values)),
        ]));

        let builder = plot(&data).aes(|a| {
            a.x_discrete("group");
            a.y_continuous("value");
        }) + geom_boxplot()
            .fill(color::LIGHTBLUE)
            .color(color::DARKBLUE);

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_boxplot_1.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }

    #[test]
    fn basic_boxplot_2() {
        init_test_logging();

        // Create sample data with two groups per X position for dodge test
        let categories = vec![
            "A", "A", "A", "A", "A", "A", "A", "A", "A", "A",
            "A", "A", "A", "A", "A", "A", "A", "A", "A", "A",
            "B", "B", "B", "B", "B", "B", "B", "B", "B", "B",
            "B", "B", "B", "B", "B", "B", "B", "B", "B", "B",
            "C", "C", "C", "C", "C", "C", "C", "C", "C", "C",
            "C", "C", "C", "C", "C", "C", "C", "C", "C", "C",
        ];
        
        let groups = vec![
            "X", "X", "X", "X", "X", "X", "X", "X", "X", "X",
            "Y", "Y", "Y", "Y", "Y", "Y", "Y", "Y", "Y", "Y",
            "X", "X", "X", "X", "X", "X", "X", "X", "X", "X",
            "Y", "Y", "Y", "Y", "Y", "Y", "Y", "Y", "Y", "Y",
            "X", "X", "X", "X", "X", "X", "X", "X", "X", "X",
            "Y", "Y", "Y", "Y", "Y", "Y", "Y", "Y", "Y", "Y",
        ];
        
        let values = vec![
            // Category A, Group X: 5-14
            5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0,
            // Category A, Group Y: 8-17
            8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0,
            // Category B, Group X: 12-21
            12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0, 20.0, 21.0,
            // Category B, Group Y: 15-24
            15.0, 16.0, 17.0, 18.0, 19.0, 20.0, 21.0, 22.0, 23.0, 24.0,
            // Category C, Group X: 3-12
            3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0,
            // Category C, Group Y: 6-15
            6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
        ];

        let data: Box<dyn DataSource> = Box::new(DataFrame::from_columns(vec![
            ("category", VectorValue::from(categories)),
            ("group", VectorValue::from(groups)),
            ("value", VectorValue::from(values)),
        ]));

        let builder = plot(&data).aes(|a| {
            a.x_discrete("category");
            a.y_continuous("value");
        }) + geom_boxplot()
            .aes(|a| {
                a.fill_discrete("group");
            })
            .position("dodge");

        let p = builder
            .build()
            .map_err(to_io_error)
            .expect("Failed to build plot");
        p.save("tests/images/basic_boxplot_2.png", 800, 600)
            .map_err(to_io_error)
            .expect("Failed to save plot image");
    }
}
