// Layer scaffolding for grammar of graphics

use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain, AestheticProperty};
use crate::data::{DataSource, VectorIter};
use crate::geom::{Geom, AestheticRequirement, DomainConstraint};
use crate::position::Position;
use crate::scale::ScaleSet;
use crate::scale::traits::{ColorRangeScale, ContinuousRangeScale, ScaleBase, ShapeRangeScale};
use crate::stat::Stat;
use crate::utils::dataframe::DataFrame;
use std::collections::HashMap;

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
    
    /// Track which domain each aesthetic property uses in this layer
    pub aesthetic_domains: HashMap<AestheticProperty, AestheticDomain>,
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
            aesthetic_domains: HashMap::new(),
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
            
            // Extract the property and look up the domain from aesthetic_domains
            if let Some(property) = aes.to_property() {
                if let Some(domain) = self.aesthetic_domains.get(&property) {
                    match (property, domain) {
                        (AestheticProperty::X, AestheticDomain::Discrete) => {
                            scales.x_discrete.train(iter);
                        }
                        (AestheticProperty::X, AestheticDomain::Continuous) => {
                            scales.x_continuous.train(iter);
                        }
                        (AestheticProperty::Y, AestheticDomain::Discrete) => {
                            scales.y_discrete.train(iter);
                        }
                        (AestheticProperty::Y, AestheticDomain::Continuous) => {
                            scales.y_continuous.train(iter);
                        }
                        (AestheticProperty::Color, AestheticDomain::Continuous) => {
                            scales.color_continuous.train(iter);
                        }
                        (AestheticProperty::Color, AestheticDomain::Discrete) => {
                            scales.color_discrete.train(iter);
                        }
                        (AestheticProperty::Fill, AestheticDomain::Continuous) => {
                            scales.fill_continuous.train(iter);
                        }
                        (AestheticProperty::Fill, AestheticDomain::Discrete) => {
                            scales.fill_discrete.train(iter);
                        }
                        (AestheticProperty::Alpha, _) => {
                            scales.alpha_scale.train(iter);
                        }
                        (AestheticProperty::Size, AestheticDomain::Continuous) => {
                            scales.size_continuous.train(iter);
                        }
                        (AestheticProperty::Size, AestheticDomain::Discrete) => {
                            scales.size_discrete.train(iter);
                        }
                        (AestheticProperty::Shape, _) => {
                            scales.shape_scale.train(iter);
                        }
                        (AestheticProperty::Linetype, _) => {
                            // do nothing
                        }
                    }
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
            // Extract the property and look up the domain from aesthetic_domains
            if let Some(property) = aes.to_property() {
                if let Some(domain) = self.aesthetic_domains.get(&property) {
                    let new_value = match (property, domain) {
                        (AestheticProperty::X, AestheticDomain::Discrete) => {
                            scales.x_discrete.map_aesthetic_value(value, data, &mut new_data).unwrap()
                        }
                        (AestheticProperty::X, AestheticDomain::Continuous) => {
                            let v = scales.x_continuous.map_aesthetic_value(value, data, &mut new_data).unwrap();
                            log::info!("Mapped X aesthetic value: {:?}", v);
                            v
                        }
                        (AestheticProperty::Y, AestheticDomain::Discrete) => {
                            scales.y_discrete.map_aesthetic_value(value, data, &mut new_data).unwrap()
                        }
                        (AestheticProperty::Y, AestheticDomain::Continuous) => {
                            scales.y_continuous.map_aesthetic_value(value, data, &mut new_data).unwrap()
                        }
                        (AestheticProperty::Color, AestheticDomain::Continuous) => {
                            scales.color_continuous.map_aesthetic_value(value, data, &mut new_data).unwrap()
                        }
                        (AestheticProperty::Color, AestheticDomain::Discrete) => {
                            scales.color_discrete.map_aesthetic_value(value, data, &mut new_data).unwrap()
                        }
                        (AestheticProperty::Fill, AestheticDomain::Continuous) => {
                            scales.fill_continuous.map_aesthetic_value(value, data, &mut new_data).unwrap()
                        }
                        (AestheticProperty::Fill, AestheticDomain::Discrete) => {
                            scales.fill_discrete.map_aesthetic_value(value, data, &mut new_data).unwrap()
                        }
                        (AestheticProperty::Alpha, _) => {
                            scales.alpha_scale.map_aesthetic_value(value, data, &mut new_data).unwrap()
                        }
                        (AestheticProperty::Size, AestheticDomain::Continuous) => {
                            scales.size_continuous.map_aesthetic_value(value, data, &mut new_data).unwrap()
                        }
                        (AestheticProperty::Size, AestheticDomain::Discrete) => {
                            scales.size_discrete.map_aesthetic_value(value, data, &mut new_data).unwrap()
                        }
                        (AestheticProperty::Shape, _) => {
                            scales.shape_scale.map_aesthetic_value(value, data, &mut new_data).unwrap()
                        }
                        (AestheticProperty::Linetype, _) => {
                            // Copy through without scaling
                            value.clone()
                        }
                    };
                    // Write back using canonical domain (Continuous for most, Shape/Linetype have no domain)
                    let canonical_aes = match property {
                        AestheticProperty::X => Aesthetic::X(AestheticDomain::Continuous),
                        AestheticProperty::Y => Aesthetic::Y(AestheticDomain::Continuous),
                        AestheticProperty::Color => Aesthetic::Color(AestheticDomain::Continuous),
                        AestheticProperty::Fill => Aesthetic::Fill(AestheticDomain::Continuous),
                        AestheticProperty::Size => Aesthetic::Size(AestheticDomain::Continuous),
                        AestheticProperty::Alpha => Aesthetic::Alpha(AestheticDomain::Continuous),
                        AestheticProperty::Shape => Aesthetic::Shape,
                        AestheticProperty::Linetype => Aesthetic::Linetype,
                    };
                    new_mapping.set(canonical_aes, new_value);
                } else {
                    // No domain specified for this property, copy through
                    new_mapping.set(aes.clone(), value.clone());
                }
            } else {
                // Group and Label aesthetics don't have properties, copy through
                new_mapping.set(aes.clone(), value.clone());
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

/// Determine aesthetic domains from the mapping and validate against geom requirements
pub fn determine_aesthetic_domains(
    mapping: &AesMap,
    requirements: &[AestheticRequirement],
) -> Result<HashMap<AestheticProperty, AestheticDomain>, crate::error::PlotError> {
    let mut domains = HashMap::new();
    
    // First pass: extract domains from mapping
    for (aesthetic, _value) in mapping.iter() {
        if let Some(property) = aesthetic.to_property() {
            let domain = aesthetic.domain();
            
            // Check for conflicts with existing domain for this property
            if let Some(existing_domain) = domains.get(&property) {
                if existing_domain != &domain {
                    return Err(crate::error::PlotError::AestheticDomainConflict {
                        property,
                        domain1: *existing_domain,
                        domain2: domain,
                    });
                }
            }
            
            // Find the requirement for this property (if any)
            if let Some(req) = requirements.iter().find(|r| r.property == property) {
                // Validate against constraint
                match &req.constraint {
                    DomainConstraint::MustBe(required_domain) => {
                        if &domain != required_domain {
                            return Err(crate::error::PlotError::IncompatibleDomain {
                                property,
                                required: *required_domain,
                                actual: domain,
                            });
                        }
                    }
                    DomainConstraint::Any => {
                        // No constraint, accept any domain
                    }
                }
            }
            
            domains.insert(property, domain);
        }
    }
    
    // Second pass: check required aesthetics are present
    for req in requirements {
        if req.required && !domains.contains_key(&req.property) {
            return Err(crate::error::PlotError::MissingRequiredAesthetic {
                property: req.property,
            });
        }
    }
    
    Ok(domains)
}
