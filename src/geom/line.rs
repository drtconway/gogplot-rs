use std::collections::HashMap;

use super::{Geom, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic, AestheticDomain};
use crate::data::{ContinuousType, DiscreteType, PrimitiveValue};
use crate::error::PlotError;
use crate::geom::properties::{ColorProperty, FloatProperty};
use crate::utils::data::{DiscreteContinuousContinuousVisitor3, Vectorable, visit3_dcc};

/// GeomLine renders lines connecting points
pub struct GeomLine {
    /// Default line color (if not mapped)
    pub color: ColorProperty,

    /// Default line width (if not mapped)
    pub size: FloatProperty,

    /// Default alpha/opacity (if not mapped)
    pub alpha: FloatProperty,

    /// Default line style pattern (if not mapped)
    pub linetype: Option<AesValue>,
}

impl GeomLine {
    /// Create a new line geom with default settings
    pub fn new() -> Self {
        Self {
            color: ColorProperty::new(),
            size: FloatProperty::new(),
            alpha: FloatProperty::new(),
            linetype: None,
        }
    }

    /// Set the default line color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        self.color.color(color);
        self
    }

    /// Set the default line width
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size.value(size);
        self
    }

    /// Set the default alpha/opacity
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha.value(alpha);
        self
    }

    /// Set the default line style pattern
    ///
    /// Pattern characters:
    /// - `-` : dash
    /// - `.` : dot
    /// - ` ` : long gap
    ///
    /// Examples: `"-"`, `"."`, `"-."`, `"- -"`, `". ."`
    pub fn linetype(&mut self, pattern: impl Into<String>) -> &mut Self {
        self.linetype = Some(AesValue::constant(PrimitiveValue::Str(pattern.into())));
        self
    }

    fn draw_lines(
        &self,
        _ctx: &mut RenderContext,
        _x_values: impl Iterator<Item = f64>,
        _y_values: impl Iterator<Item = f64>,
    ) -> Result<(), PlotError>
    {
        // Implementation of drawing lines goes here
        Ok(())
    }
}

impl Default for GeomLine {
    fn default() -> Self {
        Self::new()
    }
}

impl Geom for GeomLine {
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

            let mut grouper = LineGrouper::new();
            let groups = visit3_dcc(group_values, x_values, y_values, &mut grouper)?;
            for (x_values, y_values) in groups.into_iter() {
                self.draw_lines(ctx, x_values.into_iter(), y_values.into_iter())?;
            }
        } else {
            // Get x and y values
            let x_values: Vec<f64> = mapping.get_iter_float(&Aesthetic::X(AestheticDomain::Continuous), data).unwrap().collect();
            let y_values: Vec<f64> = mapping.get_iter_float(&Aesthetic::Y(AestheticDomain::Continuous), data).unwrap().collect();
            self.draw_lines(ctx, x_values.into_iter(), y_values.into_iter())?;
        }

        Ok(())
    }
}

struct LineGrouper {
}

impl LineGrouper {
    fn new() -> Self {
        Self { }
    }
}

impl DiscreteContinuousContinuousVisitor3 for LineGrouper {
    type Output = Vec<(Vec<f64>, Vec<f64>)>;

    fn visit<G: Vectorable + DiscreteType, T: Vectorable + ContinuousType, U: Vectorable + ContinuousType>(
        &mut self,
        group_iter: impl Iterator<Item = G>,
        x_iter: impl Iterator<Item = T>,
        y_iter: impl Iterator<Item = U>,
    ) -> std::result::Result<Self::Output, PlotError> {
        let mut groups: HashMap<G::Sortable, (Vec<f64>, Vec<f64>)> = HashMap::new();
        for ((g, x), y) in group_iter.zip(x_iter).zip(y_iter) {
            let x = x.to_f64();
            let y = y.to_f64();
            let entry = groups.entry(g.to_sortable()).or_insert((Vec::new(), Vec::new()));
            entry.0.push(x);
            entry.1.push(y);
        }

        let mut groups = groups.into_iter().collect::<Vec<_>>();
        groups.sort_by(|a, b| a.0.cmp(&b.0));

        let result = groups
            .into_iter()
            .map(|(_, (x_vals, y_vals))| (x_vals, y_vals))
            .collect();
        Ok(result)
    }
}