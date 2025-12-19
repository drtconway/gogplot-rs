use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic, AestheticDomain};
use crate::data::{ContinuousType, DiscreteType, PrimitiveValue};
use crate::error::PlotError;
use crate::utils::data::{DiscreteContinuousContinuousVisitor3, Vectorable, visit3_dcc};
use crate::visuals::Shape;

/// GeomPoint renders points/scatterplot
pub struct GeomPoint {
    /// Default point size (if not mapped)
    pub size: Option<AesValue>,

    /// Default point color (if not mapped)
    pub color: Option<AesValue>,

    /// Default point shape (if not mapped)
    pub shape: Option<AesValue>,

    /// Default alpha/opacity (if not mapped)
    pub alpha: Option<AesValue>,
}

impl GeomPoint {
    /// Create a new point geom with default settings from theme
    pub fn new() -> Self {
        Self {
            size: None,
            color: None,
            shape: None,
            alpha: None,
        }
    }

    /// Set the default point size
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size = Some(AesValue::constant(PrimitiveValue::Float(size)));
        self
    }

    /// Set the default point color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        self.color = Some(AesValue::constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the default point shape
    pub fn shape(&mut self, shape: Shape) -> &mut Self {
        self.shape = Some(AesValue::constant(PrimitiveValue::Int(i64::from(shape))));
        self
    }

    /// Set the default alpha/opacity
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = Some(AesValue::constant(PrimitiveValue::Float(
            alpha.clamp(0.0, 1.0),
        )));
        self
    }

    fn draw_points(
        &self,
        _ctx: &mut RenderContext,
        _x_values: impl Iterator<Item = f64>,
        _y_values: impl Iterator<Item = f64>,
    ) -> Result<(), PlotError>
    {
        // Implementation of drawing points goes here
        Ok(())
    }
}

impl Default for GeomPoint {
    fn default() -> Self {
        Self::new()
    }
}

impl Geom for GeomPoint {
    fn train_scales(&self, _scales: &mut crate::scale::ScaleSet) {
        
    }

    fn apply_scales(&mut self, _scales: &crate::scale::ScaleSet) {
        
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        let data = ctx.layer.data(ctx.data());
        let mapping = ctx.layer.mapping(ctx.mapping());

        if mapping.contains(Aesthetic::Group) {
            let group_values = mapping.get_vector_iter(&Aesthetic::Group, data).unwrap();
            let x_values = mapping.get_vector_iter(&Aesthetic::X(AestheticDomain::Continuous), data).unwrap();
            let y_values = mapping.get_vector_iter(&Aesthetic::Y(AestheticDomain::Continuous), data).unwrap();

            let mut grouper = PointGrouper::new();
            let groups = visit3_dcc(group_values, x_values, y_values, &mut grouper)?;
            for (x_values, y_values) in groups.into_iter() {
                self.draw_points(ctx, x_values.into_iter(), y_values.into_iter())?;
            }
        } else {
            // Get x and y values
            let x_values: Vec<f64> = mapping.get_iter_float(&Aesthetic::X(AestheticDomain::Continuous), data).unwrap().collect();
            let y_values: Vec<f64> = mapping.get_iter_float(&Aesthetic::Y(AestheticDomain::Continuous), data).unwrap().collect();
            self.draw_points(ctx, x_values.into_iter(), y_values.into_iter())?;
        }


        Ok(())
    }
}

struct PointGrouper {

}

impl PointGrouper {
    fn new() -> Self {
        Self {
        }
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
            .map(|(_, (x_vals, y_vals))| (x_vals, y_vals)).collect();

        Ok(groups)
    }
    

}