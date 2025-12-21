use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain};
use crate::data::{ContinuousType, DiscreteType};
use crate::error::PlotError;
use crate::geom::properties::{ColorProperty, FloatProperty, ShapeProperty};
use crate::geom::{
    AesMapBuilder, ColorContinuousAesBuilder, ColorDiscreteAesBuilder, XContininuousAesBuilder,
    XDiscreteAesBuilder, YContininuousAesBuilder, YDiscreteAesBuilder,
};
use crate::layer::{Layer, LayerBuilder};
use crate::scale::ScaleIdentifier;
use crate::theme::{Color, color};
use crate::utils::data::{DiscreteContinuousContinuousVisitor3, Vectorable, visit3_dcc};
use crate::visuals::Shape;

pub trait GeomPointAesBuilderTrait:
    XContininuousAesBuilder
    + XDiscreteAesBuilder
    + YContininuousAesBuilder
    + YDiscreteAesBuilder
    + ColorContinuousAesBuilder
    + ColorDiscreteAesBuilder
{
}

impl GeomPointAesBuilderTrait for AesMapBuilder {}

pub struct GeomPointBuilder {
    size: Option<FloatProperty>,
    color: Option<ColorProperty>,
    shape: Option<ShapeProperty>,
    alpha: Option<FloatProperty>,
    aes_builder: AesMapBuilder,
}

impl GeomPointBuilder {
    pub fn new() -> Self {
        Self {
            size: None,
            color: None,
            shape: None,
            alpha: None,
            aes_builder: AesMapBuilder::new(),
        }
    }

    pub fn size<Size: Into<FloatProperty>>(mut self, size: Size) -> Self {
        self.size = Some(size.into());
        self
    }

    pub fn color<Color: Into<ColorProperty>>(mut self, color: Color) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn shape<Shape: Into<ShapeProperty>>(mut self, shape: Shape) -> Self {
        self.shape = Some(shape.into());
        self
    }

    pub fn alpha<Alpha: Into<FloatProperty>>(mut self, alpha: Alpha) -> Self {
        self.alpha = Some(alpha.into());
        self
    }

    pub fn aes(mut self, closure: impl FnOnce(&mut dyn GeomPointAesBuilderTrait)) -> Self {
        closure(&mut self.aes_builder);
        self
    }
}

impl LayerBuilder for GeomPointBuilder {
    fn build(self: Box<Self>, parent_mapping: &AesMap) -> Layer {
        let mut geom_point = GeomPoint::new();
        if let Some(size_prop) = self.size {
            geom_point.size = size_prop;
        }
        if let Some(color_prop) = self.color {
            geom_point.color = color_prop;
        }
        if let Some(shape_prop) = self.shape {
            geom_point.shape = shape_prop;
        }
        if let Some(alpha_prop) = self.alpha {
            geom_point.alpha = alpha_prop;
        }

        let mapping = self.aes_builder.build(parent_mapping);

        let mut layer = crate::layer::Layer::new(Box::new(geom_point));
        layer.mapping = Some(mapping);

        layer
    }
}

pub fn geom_point() -> GeomPointBuilder {
    GeomPointBuilder::new()
}

/// GeomPoint renders points/scatterplot
pub struct GeomPoint {
    /// Default point size (if not mapped)
    pub size: FloatProperty,

    /// Default point color (if not mapped)
    pub color: ColorProperty,

    /// Default point shape (if not mapped)
    pub shape: ShapeProperty,

    /// Default alpha/opacity (if not mapped)
    pub alpha: FloatProperty,
}

impl GeomPoint {
    /// Create a new point geom with default settings from theme
    pub fn new() -> Self {
        Self {
            size: FloatProperty::new(),
            color: ColorProperty::new(),
            shape: ShapeProperty::new(),
            alpha: FloatProperty::new(),
        }
    }

    /// Set the default point size
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size.value(size);
        self
    }

    /// Set the default point color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        self.color.color(color);
        self
    }

    /// Set the default point shape
    pub fn shape(&mut self, shape: Shape) -> &mut Self {
        self.shape.shape(shape);
        self
    }

    /// Set the default alpha/opacity
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha.value(alpha.clamp(0.0, 1.0));
        self
    }

    fn draw_points(
        &self,
        ctx: &mut RenderContext,
        x_values: impl Iterator<Item = f64>,
        y_values: impl Iterator<Item = f64>,
    ) -> Result<(), PlotError> {
        // Get default point size from theme or property
        let point_size = 4.0; // Placeholder for now
        let point_radius = point_size / 2.0;

        // Set default color
        let Color(r, g, b, a) = color::BLACK; // Placeholder for now
        ctx.cairo.set_source_rgba(r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0, a as f64 / 255.0);

        // Data values are already normalized [0,1] by compose()->apply_scales()
        // Just map to viewport pixel coordinates
        for (x_norm, y_norm) in x_values.zip(y_values) {
            // Convert normalized [0,1] to viewport pixel coordinates
            let x_px = ctx.map_x(x_norm);
            let y_px = ctx.map_y(y_norm);

            log::info!("Drawing point at norm({}, {}) -> px({}, {})", 
                x_norm, y_norm, x_px, y_px);

            // Draw circle
            ctx.cairo
                .arc(x_px, y_px, point_radius, 0.0, 2.0 * std::f64::consts::PI);
            ctx.cairo.fill().ok();
        }

        Ok(())
    }
}

impl Default for GeomPoint {
    fn default() -> Self {
        Self::new()
    }
}

impl Geom for GeomPoint {
    fn required_scales(&self) -> Vec<ScaleIdentifier> {
        vec![ScaleIdentifier::XContinuous, ScaleIdentifier::YContinuous]
    }

    fn train_scales(&self, _scales: &mut crate::scale::ScaleSet) {}

    fn apply_scales(&mut self, _scales: &crate::scale::ScaleSet) {}

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        let data = ctx.layer.data(ctx.data());
        let mapping = ctx.layer.mapping(ctx.mapping());

        if mapping.contains(Aesthetic::Group) {
            let group_values = mapping.get_vector_iter(&Aesthetic::Group, data).unwrap();
            let x_values = mapping
                .get_vector_iter(&Aesthetic::X(AestheticDomain::Continuous), data)
                .unwrap();
            let y_values = mapping
                .get_vector_iter(&Aesthetic::Y(AestheticDomain::Continuous), data)
                .unwrap();

            let mut grouper = PointGrouper::new();
            let groups = visit3_dcc(group_values, x_values, y_values, &mut grouper)?;
            for (x_values, y_values) in groups.into_iter() {
                self.draw_points(ctx, x_values.into_iter(), y_values.into_iter())?;
            }
        } else {
            // Get x and y values
            let x_values: Vec<f64> = mapping
                .get_iter_float(&Aesthetic::X(AestheticDomain::Continuous), data)
                .unwrap()
                .collect();
            let y_values: Vec<f64> = mapping
                .get_iter_float(&Aesthetic::Y(AestheticDomain::Continuous), data)
                .unwrap()
                .collect();
            log::info!("Drawing points: {:?}", x_values);
            self.draw_points(ctx, x_values.into_iter(), y_values.into_iter())?;
        }

        Ok(())
    }
}

struct PointGrouper {}

impl PointGrouper {
    fn new() -> Self {
        Self {}
    }
}

impl DiscreteContinuousContinuousVisitor3 for PointGrouper {
    type Output = Vec<(Vec<f64>, Vec<f64>)>;

    fn visit<
        G: Vectorable + DiscreteType,
        T: Vectorable + ContinuousType,
        U: Vectorable + ContinuousType,
    >(
        &mut self,
        group_iter: impl Iterator<Item = G>,
        x_iter: impl Iterator<Item = T>,
        y_iter: impl Iterator<Item = U>,
    ) -> std::result::Result<Self::Output, PlotError> {
        let mut groups: HashMap<G::Sortable, (Vec<f64>, Vec<f64>)> = HashMap::new();
        for ((g, x), y) in group_iter.zip(x_iter).zip(y_iter) {
            let g_key = g.to_sortable();
            let x_f64 = x.to_f64();
            let y_f64 = y.to_f64();
            let entry = groups.entry(g_key).or_insert((Vec::new(), Vec::new()));
            entry.0.push(x_f64);
            entry.1.push(y_f64);
        }

        let mut groups = groups.into_iter().collect::<Vec<_>>();
        groups.sort_by(|a, b| a.0.cmp(&b.0));

        let groups = groups
            .into_iter()
            .map(|(_, (x_vals, y_vals))| (x_vals, y_vals))
            .collect();

        Ok(groups)
    }
}
