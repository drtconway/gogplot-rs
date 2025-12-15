// Layer scaffolding for grammar of graphics

use crate::aesthetics::{AesMap, AestheticDomain};
use crate::data::DataSource;
use crate::geom::Geom;
use crate::position::Position;
use crate::scale::ScaleSet;
use crate::scale::traits::ContinuousRangeScale;
use crate::stat::Stat;
use crate::utils::dataframe::DataFrame;

/// Layer struct - represents one layer in a plot
/// Each layer has its own geom, optional data, aesthetic mappings, stat, and position
pub struct Layer {
    pub stat: Option<Box<dyn Stat>>,
    pub position: Option<Box<dyn Position>>,
    pub geom: Box<dyn Geom>,
    pub data: Option<Box<dyn DataSource>>,
    pub mapping: Option<AesMap>,
}

impl Layer {
    /// Create a new layer with the specified geom
    pub fn new(geom: Box<dyn Geom>) -> Self {
        Self {
            stat: None,
            position: None,
            geom,
            data: None,
            mapping: None,
        }
    }

    pub fn with_stat(mut self, stat: Box<dyn Stat>) -> Self {
        self.stat = Some(stat);
        self
    }

    pub fn with_position(mut self, position: Box<dyn Position>) -> Self {
        self.position = Some(position);
        self
    }

    pub fn with_data(mut self, data: Box<dyn DataSource>) -> Self {
        self.data = Some(data);
        self
    }

    pub fn with_mapping(mut self, mapping: AesMap) -> Self {
        self.mapping = Some(mapping);
        self
    }

    pub fn apply_stat(
        &mut self,
        data: &Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<(), crate::error::PlotError> {
        if let Some(stat) = &self.stat {
            let (new_data, new_mapping) = stat.compute(data, mapping)?;
            self.data = Some(new_data);
            self.mapping = Some(new_mapping);
        }
        Ok(())
    }

    pub fn apply_position(
        &mut self,
        data: &Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<(), crate::error::PlotError> {
        if let Some(position) = &self.position {
            let (new_data, new_mapping) = position.adjust(data, mapping)?;
            self.data = Some(new_data);
            self.mapping = Some(new_mapping);
        }
        Ok(())
    }

    pub fn train_scales(
        &mut self,
        scales: &mut ScaleSet,
        data: &Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<(), crate::error::PlotError> {
        let data = if let Some(data) = &self.data {
            data
        } else {
            data
        };
        let mapping = if let Some(mapping) = &self.mapping {
            mapping
        } else {
            mapping
        };
        for aes in mapping.iter_aesthetics() {
            let iter = mapping.get_vector_iter(aes, data).unwrap();
            match aes {
                crate::aesthetics::Aesthetic::X(AestheticDomain::Discrete)
                | crate::aesthetics::Aesthetic::Xmin(AestheticDomain::Discrete)
                | crate::aesthetics::Aesthetic::Xmax(AestheticDomain::Discrete) => {
                    scales.x_discrete.train(iter);
                }

                crate::aesthetics::Aesthetic::X(AestheticDomain::Continuous)
                | crate::aesthetics::Aesthetic::Xmin(AestheticDomain::Continuous)
                | crate::aesthetics::Aesthetic::Xmax(AestheticDomain::Continuous)
                | crate::aesthetics::Aesthetic::XBegin
                | crate::aesthetics::Aesthetic::XEnd
                | crate::aesthetics::Aesthetic::XIntercept => {
                    scales.x_continuous.train(iter);
                }
                crate::aesthetics::Aesthetic::Y(AestheticDomain::Discrete)
                | crate::aesthetics::Aesthetic::Ymin(AestheticDomain::Discrete)
                | crate::aesthetics::Aesthetic::Ymax(AestheticDomain::Discrete) => {
                    scales.y_discrete.train(iter);
                }
                crate::aesthetics::Aesthetic::Y(AestheticDomain::Continuous)
                | crate::aesthetics::Aesthetic::Ymin(AestheticDomain::Continuous)
                | crate::aesthetics::Aesthetic::Ymax(AestheticDomain::Continuous)
                | crate::aesthetics::Aesthetic::YBegin
                | crate::aesthetics::Aesthetic::YEnd
                | crate::aesthetics::Aesthetic::YIntercept
                | crate::aesthetics::Aesthetic::Lower
                | crate::aesthetics::Aesthetic::Middle
                | crate::aesthetics::Aesthetic::Upper => {
                    scales.y_continuous.train(iter);
                }
                crate::aesthetics::Aesthetic::Color(aesthetic_domain) => todo!(),
                crate::aesthetics::Aesthetic::Fill(aesthetic_domain) => todo!(),
                crate::aesthetics::Aesthetic::Alpha => {
                    scales.alpha_scale.train(iter);
                }
                crate::aesthetics::Aesthetic::Size => {
                    scales.size_continuous.train(iter);
                }
                crate::aesthetics::Aesthetic::Shape => {
                    scales.shape_scale.train(iter);
                }
                crate::aesthetics::Aesthetic::Linetype
                | crate::aesthetics::Aesthetic::Group
                | crate::aesthetics::Aesthetic::Label => {
                    // do nothing
                }
            }
        }
        Ok(())
    }

    pub fn apply_scales(
        &mut self,
        scales: &ScaleSet,
        data: &Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<(), crate::error::PlotError> {
        let data = if let Some(data) = &self.data {
            data
        } else {
            data
        };
        let mapping = if let Some(mapping) = &self.mapping {
            mapping
        } else {
            mapping
        };

        let mut new_data = DataFrame::new();
        let mut new_mapping = AesMap::new();

        for (aes, value) in mapping.iter() {
            match aes {
                crate::aesthetics::Aesthetic::X(AestheticDomain::Discrete)
                | crate::aesthetics::Aesthetic::Xmin(AestheticDomain::Discrete)
                | crate::aesthetics::Aesthetic::Xmax(AestheticDomain::Discrete) => {
                    let new_value = scales.x_discrete.map_aesthetic_value(value, data, &mut new_data).unwrap();
                    new_mapping.insert(aes.clone(), new_value);
                }

                crate::aesthetics::Aesthetic::X(AestheticDomain::Continuous)
                | crate::aesthetics::Aesthetic::Xmin(AestheticDomain::Continuous)
                | crate::aesthetics::Aesthetic::Xmax(AestheticDomain::Continuous)
                | crate::aesthetics::Aesthetic::XBegin
                | crate::aesthetics::Aesthetic::XEnd
                | crate::aesthetics::Aesthetic::XIntercept => {
                    let new_value = scales.x_continuous.map_aesthetic_value(value, data, &mut new_data).unwrap();
                    new_mapping.insert(aes.clone(), new_value);
                }
                crate::aesthetics::Aesthetic::Y(AestheticDomain::Discrete)
                | crate::aesthetics::Aesthetic::Ymin(AestheticDomain::Discrete)
                | crate::aesthetics::Aesthetic::Ymax(AestheticDomain::Discrete) => {
                    let new_value = scales.y_discrete.map_aesthetic_value(value, data, &mut new_data).unwrap();
                    new_mapping.insert(aes.clone(), new_value);
                }
                crate::aesthetics::Aesthetic::Y(AestheticDomain::Continuous)
                | crate::aesthetics::Aesthetic::Ymin(AestheticDomain::Continuous)
                | crate::aesthetics::Aesthetic::Ymax(AestheticDomain::Continuous)
                | crate::aesthetics::Aesthetic::YBegin
                | crate::aesthetics::Aesthetic::YEnd
                | crate::aesthetics::Aesthetic::YIntercept
                | crate::aesthetics::Aesthetic::Lower
                | crate::aesthetics::Aesthetic::Middle
                | crate::aesthetics::Aesthetic::Upper => {
                    let new_value = scales.y_continuous.map_aesthetic_value(value, data, &mut new_data).unwrap();
                    new_mapping.insert(aes.clone(), new_value);
                }
                crate::aesthetics::Aesthetic::Color(aesthetic_domain) => {
                    let new_value = match aesthetic_domain {
                        AestheticDomain::Continuous => {
                            scales.color_continuous.map_aesthetic_value(value, data, &mut new_data).unwrap()
                        }
                        AestheticDomain::Discrete => {
                            scales.color_discrete.map_aesthetic_value(value, data, &mut new_data).unwrap()
                        }
                    };
                    new_mapping.insert(aes.clone(), new_value);
                },
                crate::aesthetics::Aesthetic::Fill(aesthetic_domain) => {
                    let new_value = match aesthetic_domain {
                        AestheticDomain::Continuous => {
                            scales.fill_continuous.map_aesthetic_value(value, data, &mut new_data).unwrap()
                        }
                        AestheticDomain::Discrete => {
                            scales.fill_discrete.map_aesthetic_value(value, data, &mut new_data).unwrap()
                        }
                    };
                    new_mapping.insert(aes.clone(), new_value);
                },
                crate::aesthetics::Aesthetic::Alpha => {
                        let new_value = scales.alpha_scale.map_aesthetic_value(value, data, &mut new_data).unwrap();
                        new_mapping.insert(aes.clone(), new_value);
                }
                crate::aesthetics::Aesthetic::Size => {
                    let new_value = scales.size_continuous.map_aesthetic_value(value, data, &mut new_data).unwrap();
                    new_mapping.insert(aes.clone(), new_value);
                }
                crate::aesthetics::Aesthetic::Shape => {
                    let new_value = scales.shape_scale.map_aesthetic_value(value, data, &mut new_data).unwrap();
                    new_mapping.insert(aes.clone(), new_value);
                }
                crate::aesthetics::Aesthetic::Linetype
                | crate::aesthetics::Aesthetic::Group
                | crate::aesthetics::Aesthetic::Label => {
                    // do nothing
                }
            }
        }
        Ok(())
    }
}
