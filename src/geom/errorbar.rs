use super::{Geom, IntoLayer, RenderContext};
use crate::aesthetics::{AesValue, Aesthetic};
use crate::data::PrimitiveValue;
use crate::error::{DataType, PlotError};

/// GeomErrorbar renders vertical error bars with optional caps
pub struct GeomErrorbar {
    /// Default line color (if not mapped)
    pub color: Option<AesValue>,

    /// Default line width (if not mapped)
    pub size: Option<AesValue>,

    /// Default alpha/opacity (if not mapped)
    pub alpha: Option<AesValue>,

    /// Width of the caps at the ends of the error bars (in data coordinates)
    pub width: f64,
}

impl GeomErrorbar {
    /// Create a new errorbar geom with default settings
    pub fn new() -> Self {
        Self {
            color: None,
            size: None,
            alpha: None,
            width: 0.5,
        }
    }

    /// Set the default line color
    pub fn color(&mut self, color: crate::theme::Color) -> &mut Self {
        self.color = Some(AesValue::constant(PrimitiveValue::Int(color.into())));
        self
    }

    /// Set the default line width
    pub fn size(&mut self, size: f64) -> &mut Self {
        self.size = Some(AesValue::constant(PrimitiveValue::Float(size)));
        self
    }

    /// Set the default alpha/opacity
    pub fn alpha(&mut self, alpha: f64) -> &mut Self {
        self.alpha = Some(AesValue::constant(PrimitiveValue::Float(
            alpha.clamp(0.0, 1.0),
        )));
        self
    }

    /// Set the width of the caps (in data coordinates)
    pub fn width(&mut self, width: f64) -> &mut Self {
        self.width = width.max(0.0);
        self
    }
}

impl Default for GeomErrorbar {
    fn default() -> Self {
        Self::new()
    }
}

impl IntoLayer for GeomErrorbar {
    fn default_aesthetics(&self) -> Vec<(Aesthetic, AesValue)> {
        use crate::theme::Theme;
        
        let mut defaults = Vec::new();
        let theme = Theme::default();

        if let Some(color) = &self.color {
            defaults.push((Aesthetic::Color, color.clone()));
        } else {
            defaults.push((Aesthetic::Color, AesValue::constant(PrimitiveValue::Int(theme.geom_line.color.into()))));
        }
        
        if let Some(alpha) = &self.alpha {
            defaults.push((Aesthetic::Alpha, alpha.clone()));
        } else {
            defaults.push((Aesthetic::Alpha, AesValue::constant(PrimitiveValue::Float(theme.geom_line.alpha))));
        }
        
        if let Some(size) = &self.size {
            defaults.push((Aesthetic::Size, size.clone()));
        } else {
            defaults.push((Aesthetic::Size, AesValue::constant(PrimitiveValue::Float(theme.geom_line.size))));
        }

        defaults
    }
}

impl Geom for GeomErrorbar {
    fn required_aesthetics(&self) -> &[Aesthetic] {
        &[Aesthetic::X, Aesthetic::Ymin, Aesthetic::Ymax]
    }

    fn setup_data(
        &self,
        data: &dyn crate::data::DataSource,
        mapping: &crate::aesthetics::AesMap,
    ) -> Result<(Option<Box<dyn crate::data::DataSource>>, Option<crate::aesthetics::AesMap>), PlotError> {
        use crate::utils::dataframe::{DataFrame, FloatVec};

        // Check if X aesthetic is mapped
        let x_aes = match mapping.get(&Aesthetic::X) {
            Some(aes) => aes,
            None => return Ok((None, None)), // No X mapping, nothing to set up
        };

        // Determine if X will use a categorical scale
        use crate::scale::ScaleType;
        let scale_preference = self.aesthetic_scale_type(Aesthetic::X);

        let mut new_mapping = mapping.clone();

        // Determine if X will be categorical based on the aesthetic value type and scale preference
        let is_categorical = match x_aes {
            AesValue::Column { name, .. } => {
                let x_col = data.get(name.as_str())
                    .ok_or_else(|| PlotError::missing_column(name))?;
                match scale_preference {
                    ScaleType::Categorical => true,
                    ScaleType::Continuous => false,
                    ScaleType::Either => x_col.iter_str().is_some(),
                }
            }
            AesValue::Constant { value, .. } => {
                matches!(value, PrimitiveValue::Str(_))
            }
        };

        if is_categorical {
            // For categorical X, map both Xmin and Xmax to the same aesthetic (column or constant)
            // No need to modify data - just update mapping
            new_mapping.set(Aesthetic::Xmin, x_aes.clone());
            new_mapping.set(Aesthetic::Xmax, x_aes.clone());
            Ok((None, Some(new_mapping)))
        } else {
            // For continuous X, we need to compute xmin/xmax with width offsets
            match x_aes {
                AesValue::Column { name: x_col_name, .. } => {
                    // Get the X column
                    let x_col = data.get(x_col_name.as_str())
                        .ok_or_else(|| PlotError::missing_column(x_col_name))?;

                    // Convert to floats
                    let x_vals: Vec<f64> = if let Some(int_iter) = x_col.iter_int() {
                        int_iter.map(|v| v as f64).collect()
                    } else if let Some(float_iter) = x_col.iter_float() {
                        float_iter.collect()
                    } else {
                        return Err(PlotError::InvalidAestheticType {
                            aesthetic: Aesthetic::X,
                            expected: DataType::Custom("numeric".to_string()),
                            actual: DataType::Custom("unknown".to_string()),
                        });
                    };

                    let half_width = self.width / 2.0;
                    let xmin_vals: Vec<f64> = x_vals.iter().map(|x| x - half_width).collect();
                    let xmax_vals: Vec<f64> = x_vals.iter().map(|x| x + half_width).collect();

                    // Create a new dataframe with all original columns plus xmin/xmax
                    let mut new_df = DataFrame::new();
                    
                    use crate::data::{VectorIter, GenericVector};
                    use crate::utils::dataframe::{IntVec, StrVec, BoolVec};
                    
                    for col_name in data.column_names() {
                        if let Some(col) = data.get(&col_name) {
                            let new_col: Box<dyn GenericVector> = match col.iter() {
                                VectorIter::Int(iter) => Box::new(IntVec(iter.collect())),
                                VectorIter::Float(iter) => Box::new(FloatVec(iter.collect())),
                                VectorIter::Str(iter) => Box::new(StrVec(iter.map(|s| s.to_string()).collect())),
                                VectorIter::Bool(iter) => Box::new(BoolVec(iter.collect())),
                            };
                            new_df.add_column(&col_name, new_col);
                        }
                    }

                    new_df.add_column("xmin", Box::new(FloatVec(xmin_vals)));
                    new_df.add_column("xmax", Box::new(FloatVec(xmax_vals)));

                    new_mapping.set(Aesthetic::Xmin, AesValue::column("xmin"));
                    new_mapping.set(Aesthetic::Xmax, AesValue::column("xmax"));
                    
                    Ok((Some(Box::new(new_df)), Some(new_mapping)))
                }
                AesValue::Constant { value, .. } => {
                    // For numeric constants, apply width offset
                    let half_width = self.width / 2.0;
                    match value {
                        PrimitiveValue::Int(x) => {
                            let x_f64 = *x as f64;
                            new_mapping.set(Aesthetic::Xmin, AesValue::constant(PrimitiveValue::Float(x_f64 - half_width)));
                            new_mapping.set(Aesthetic::Xmax, AesValue::constant(PrimitiveValue::Float(x_f64 + half_width)));
                        }
                        PrimitiveValue::Float(x) => {
                            new_mapping.set(Aesthetic::Xmin, AesValue::constant(PrimitiveValue::Float(x - half_width)));
                            new_mapping.set(Aesthetic::Xmax, AesValue::constant(PrimitiveValue::Float(x + half_width)));
                        }
                        _ => {
                            return Err(PlotError::InvalidAestheticType {
                                aesthetic: Aesthetic::X,
                                expected: DataType::Custom("numeric".to_string()),
                                actual: DataType::Custom("unknown".to_string()),
                            });
                        }
                    }
                    Ok((None, Some(new_mapping)))
                }
            }
        }
    }

    fn render(&self, ctx: &mut RenderContext) -> Result<(), PlotError> {
        // Get position aesthetics (all pre-normalized to [0,1])
        let x_normalized = ctx.get_x_aesthetic_values(Aesthetic::X)?;
        let xmin_normalized = ctx.get_x_aesthetic_values(Aesthetic::Xmin)?;
        let xmax_normalized = ctx.get_x_aesthetic_values(Aesthetic::Xmax)?;
        let ymin_normalized = ctx.get_y_aesthetic_values(Aesthetic::Ymin)?;
        let ymax_normalized = ctx.get_y_aesthetic_values(Aesthetic::Ymax)?;

        // Get styling aesthetics
        let colors = ctx.get_color_values()?;
        let alphas = ctx.get_aesthetic_values(Aesthetic::Alpha, None)?;
        let sizes = ctx.get_aesthetic_values(Aesthetic::Size, None)?;

        // Zip all iterators together
        let iter = x_normalized
            .zip(xmin_normalized)
            .zip(xmax_normalized)
            .zip(ymin_normalized)
            .zip(ymax_normalized)
            .zip(colors)
            .zip(alphas)
            .zip(sizes);

        for (((((((x_norm, xmin_norm), xmax_norm), ymin_norm), ymax_norm), color), alpha), size) in iter {
            // Map normalized [0,1] coordinates to device coordinates
            let x_visual = ctx.map_x(x_norm);
            let xmin_visual = ctx.map_x(xmin_norm);
            let xmax_visual = ctx.map_x(xmax_norm);
            let ymin_visual = ctx.map_y(ymin_norm);
            let ymax_visual = ctx.map_y(ymax_norm);

            // Set drawing properties
            ctx.set_color_alpha(&color, alpha);
            ctx.cairo.set_line_width(size);

            // Draw vertical line from ymin to ymax
            ctx.cairo.move_to(x_visual, ymin_visual);
            ctx.cairo.line_to(x_visual, ymax_visual);
            ctx.cairo.stroke().ok();

            // Draw caps if width > 0
            if self.width > 0.0 {
                // Draw bottom cap
                ctx.cairo.move_to(xmin_visual, ymin_visual);
                ctx.cairo.line_to(xmax_visual, ymin_visual);
                ctx.cairo.stroke().ok();

                // Draw top cap
                ctx.cairo.move_to(xmin_visual, ymax_visual);
                ctx.cairo.line_to(xmax_visual, ymax_visual);
                ctx.cairo.stroke().ok();
            }
        }

        Ok(())
    }
}
