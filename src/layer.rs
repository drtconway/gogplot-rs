// Layer scaffolding for grammar of graphics

use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain};
use crate::data::{DataSource, VectorIter};
use crate::geom::Geom;
use crate::position::Position;
use crate::scale::ScaleSet;
use crate::scale::traits::{ColorRangeScale, ContinuousRangeScale, ScaleBase, ShapeRangeScale};
use crate::stat::Stat;
use crate::utils::dataframe::DataFrame;

pub trait LayerBuilder {
    fn build(self: Box<Self>, parent_mapping: &AesMap) -> Layer;
}

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

    pub fn data<'a>(&'a self, other_data: &'a dyn DataSource) -> &'a dyn DataSource {
        self.data.as_ref().map(|b| b.as_ref()).unwrap_or(other_data)
    }

    pub fn mapping<'a>(&'a self, other_mapping: &'a AesMap) -> &'a AesMap {
        self.mapping.as_ref().unwrap_or(other_mapping)
    }

    pub fn apply_stat(
        &mut self,
        data: &Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<(), crate::error::PlotError> {
        if let Some(stat) = &self.stat {
            if let Some((new_data, new_mapping)) = stat.apply(data, mapping)? {
                self.data = Some(new_data);
                self.mapping = Some(new_mapping);
            }
        }
        Ok(())
    }

    pub fn apply_position(
        &mut self,
        data: &Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<(), crate::error::PlotError> {
        if let Some(position) = &self.position {
            if let Some((new_data, new_mapping)) = position.apply(data, mapping)? {
                self.data = Some(new_data);
                self.mapping = Some(new_mapping);
            }
        }
        Ok(())
    }

    pub fn train_scales(
        &mut self,
        scales: &mut ScaleSet,
        data: &Box<dyn DataSource>,
        mapping: &AesMap,
    ) -> Result<(), crate::error::PlotError> {
        let data = self.data(data.as_ref());
        let mapping = self.mapping(mapping);
        for aes in mapping.aesthetics() {
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
                crate::aesthetics::Aesthetic::Color(AestheticDomain::Continuous) => {
                    scales.color_continuous.train(iter);
                }
                crate::aesthetics::Aesthetic::Color(AestheticDomain::Discrete) => {
                    scales.color_discrete.train(iter);
                }
                crate::aesthetics::Aesthetic::Fill(AestheticDomain::Continuous) => {
                    scales.fill_continuous.train(iter);
                }
                crate::aesthetics::Aesthetic::Fill(AestheticDomain::Discrete) => {
                    scales.fill_discrete.train(iter);
                }
                crate::aesthetics::Aesthetic::Alpha(AestheticDomain::Continuous) => {
                    scales.alpha_scale.train(iter);
                }
                crate::aesthetics::Aesthetic::Alpha(AestheticDomain::Discrete) => {
                    scales.alpha_scale.train(iter);
                }
                crate::aesthetics::Aesthetic::Size(AestheticDomain::Continuous) => {
                    scales.size_continuous.train(iter);
                }
                crate::aesthetics::Aesthetic::Size(AestheticDomain::Discrete) => {
                    scales.size_discrete.train(iter);
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
        let data = self.data(data.as_ref());
        let mapping = self.mapping(mapping);

        let mut new_data = DataFrame::new();
        let mut new_mapping = AesMap::new();

        for (aes, value) in mapping.iter() {
            match aes {
                crate::aesthetics::Aesthetic::X(AestheticDomain::Discrete)
                | crate::aesthetics::Aesthetic::Xmin(AestheticDomain::Discrete)
                | crate::aesthetics::Aesthetic::Xmax(AestheticDomain::Discrete) => {
                    let new_value = scales
                        .x_discrete
                        .map_aesthetic_value(value, data, &mut new_data)
                        .unwrap();
                    new_mapping.set(aes.clone(), new_value);
                }

                crate::aesthetics::Aesthetic::X(AestheticDomain::Continuous)
                | crate::aesthetics::Aesthetic::Xmin(AestheticDomain::Continuous)
                | crate::aesthetics::Aesthetic::Xmax(AestheticDomain::Continuous)
                | crate::aesthetics::Aesthetic::XBegin
                | crate::aesthetics::Aesthetic::XEnd
                | crate::aesthetics::Aesthetic::XIntercept => {
                    let new_value = scales
                        .x_continuous
                        .map_aesthetic_value(value, data, &mut new_data)
                        .unwrap();
                    log::info!("Mapped X aesthetic value: {:?}", new_value);
                    new_mapping.set(aes.clone(), new_value);
                }
                crate::aesthetics::Aesthetic::Y(AestheticDomain::Discrete)
                | crate::aesthetics::Aesthetic::Ymin(AestheticDomain::Discrete)
                | crate::aesthetics::Aesthetic::Ymax(AestheticDomain::Discrete) => {
                    let new_value = scales
                        .y_discrete
                        .map_aesthetic_value(value, data, &mut new_data)
                        .unwrap();
                    new_mapping.set(aes.clone(), new_value);
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
                    let new_value = scales
                        .y_continuous
                        .map_aesthetic_value(value, data, &mut new_data)
                        .unwrap();
                    new_mapping.set(aes.clone(), new_value);
                }
                crate::aesthetics::Aesthetic::Color(aesthetic_domain) => {
                    let new_value = match aesthetic_domain {
                        AestheticDomain::Continuous => scales
                            .color_continuous
                            .map_aesthetic_value(value, data, &mut new_data)
                            .unwrap(),
                        AestheticDomain::Discrete => scales
                            .color_discrete
                            .map_aesthetic_value(value, data, &mut new_data)
                            .unwrap(),
                    };
                    new_mapping.set(aes.clone(), new_value);
                }
                crate::aesthetics::Aesthetic::Fill(aesthetic_domain) => {
                    let new_value = match aesthetic_domain {
                        AestheticDomain::Continuous => scales
                            .fill_continuous
                            .map_aesthetic_value(value, data, &mut new_data)
                            .unwrap(),
                        AestheticDomain::Discrete => scales
                            .fill_discrete
                            .map_aesthetic_value(value, data, &mut new_data)
                            .unwrap(),
                    };
                    new_mapping.set(aes.clone(), new_value);
                }
                crate::aesthetics::Aesthetic::Alpha(aesthetic_domain) => {
                    let new_value = match aesthetic_domain {
                        AestheticDomain::Continuous => scales
                            .alpha_scale
                            .map_aesthetic_value(value, data, &mut new_data)
                            .unwrap(),
                        AestheticDomain::Discrete => scales
                            .alpha_scale
                            .map_aesthetic_value(value, data, &mut new_data)
                            .unwrap(),
                    };
                    new_mapping.set(aes.clone(), new_value);
                }
                crate::aesthetics::Aesthetic::Size(aesthetic_domain) => {
                    let new_value = match aesthetic_domain {
                        AestheticDomain::Continuous => scales
                            .size_continuous
                            .map_aesthetic_value(value, data, &mut new_data)
                            .unwrap(),
                        AestheticDomain::Discrete => scales
                            .size_discrete
                            .map_aesthetic_value(value, data, &mut new_data)
                            .unwrap(),
                    };
                    new_mapping.set(aes.clone(), new_value);
                }
                crate::aesthetics::Aesthetic::Shape => {
                    let new_value = scales
                        .shape_scale
                        .map_aesthetic_value(value, data, &mut new_data)
                        .unwrap();
                    new_mapping.set(aes.clone(), new_value);
                }
                crate::aesthetics::Aesthetic::Linetype
                | crate::aesthetics::Aesthetic::Group
                | crate::aesthetics::Aesthetic::Label => {
                    // do nothing
                }
            }
        }

        self.data = Some(Box::new(new_data));
        self.mapping = Some(new_mapping);

        Ok(())
    }

    pub fn aesthetic_value_iter<'a>(&'a self, aes: &'a Aesthetic) -> Option<VectorIter<'a>> {
        if let Some(mapping) = &self.mapping {
            if let Some(data) = &self.data {
                return mapping.get_vector_iter(aes, data.as_ref());
            }
        }
        None
    }
}
