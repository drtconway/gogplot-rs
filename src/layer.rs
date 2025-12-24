// Layer scaffolding for grammar of graphics

use crate::PlotError;
use crate::aesthetics::{AesMap, Aesthetic, AestheticDomain, AestheticProperty};
use crate::data::{DataSource, DiscreteValue, VectorIter};
use crate::geom::properties::{Property, PropertyValue, PropertyVector};
use crate::geom::{AestheticRequirement, DomainConstraint, Geom};
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

    /// Render this layer by materializing groups and calling the geom
    pub fn render(
        &self,
        ctx: &mut crate::geom::RenderContext,
        plot_data: &dyn DataSource,
        plot_mapping: &AesMap,
    ) -> Result<(), PlotError> {
        let data = self.data(plot_data);
        let mapping = self.mapping(plot_mapping);
        let n = data.len();

        // Get aesthetic requirements and properties from geom
        let requirements = self.geom.aesthetic_requirements();
        let properties = self.geom.properties();
        let defaults = self.geom.property_defaults(&ctx.theme);

        // Build index: property -> (aesthetic, domain) from the mapping
        let mut index: HashMap<AestheticProperty, Aesthetic> = HashMap::new();
        for aesthetic in mapping.aesthetics() {
            if let Some(property) = aesthetic.to_property() {
                index.insert(property, aesthetic.clone());
            }
        }

        // Materialize PropertyVectors for all required aesthetics
        let mut all_vectors = HashMap::new();

        for req in requirements {
            let property = req.property;

            // Priority 1: Check if geom has an explicit property set
            if let Some(prop_value) = properties.get(&property) {
                // Use property (prioritize set values over inherited mappings)
                let vector = self.materialize_constant_aesthetic(prop_value, n);
                all_vectors.insert(property, vector);
            } else if let Some(aesthetic) = index.get(&property) {
                // Priority 2: Use mapping
                let vector: PropertyVector =
                    PropertyVector::from(mapping.get_vector_iter(aesthetic, data).unwrap());
                all_vectors.insert(property, vector);
            } else if let Some(default_value) = defaults.get(&property) {
                // Priority 3: get the default value
                let vector = self.make_property_value_vector(&default_value, n);
                all_vectors.insert(property, vector);
            }
        }

        // Check for grouping
        if mapping.contains(Aesthetic::Group) {
            // Materialize group values
            let group_iter = mapping.get_iter_discrete(&Aesthetic::Group, data).ok_or(
                crate::error::PlotError::MissingColumn {
                    column: "Group".to_string(),
                },
            )?;
            let group_values: Vec<DiscreteValue> = group_iter.collect();

            // Split into groups: HashMap<DiscreteValue, Vec<usize>>
            let mut groups: HashMap<DiscreteValue, Vec<usize>> = HashMap::new();
            for (i, group) in group_values.iter().enumerate() {
                groups.entry(group.clone()).or_insert_with(Vec::new).push(i);
            }

            // Sort groups for consistent rendering order
            let mut sorted_groups: Vec<_> = groups.into_iter().collect();
            sorted_groups.sort_by(|a, b| a.0.cmp(&b.0));

            // Render each group
            for (_group_value, indices) in sorted_groups {
                // Create subset PropertyVectors for this group
                let group_data = self.subset_vectors(&all_vectors, &indices);
                self.geom.render(ctx, group_data)?;
            }
        } else {
            // No grouping - render all data at once
            self.geom.render(ctx, all_vectors)?;
        }

        Ok(())
    }

    /// Materialize a constant aesthetic from property default
    fn materialize_constant_aesthetic(
        &self,
        prop_value: &Property,
        n: usize,
    ) -> PropertyVector {
        match prop_value {
            Property::Float(fp) => {
                // Extract constant value and repeat n times
                match &fp.value {
                    crate::utils::Either::Left(val) => PropertyVector::Float(vec![*val; n]),
                    crate::utils::Either::Right(_) => {
                        // Column reference in property - shouldn't happen for constants
                        PropertyVector::Float(vec![1.0; n])
                    }
                }
            }
            Property::Color(cp) => match &cp.color {
                crate::utils::Either::Left(color) => PropertyVector::Color(vec![*color; n]),
                crate::utils::Either::Right(_) => {
                    PropertyVector::Color(vec![crate::theme::color::BLACK; n])
                }
            },
            Property::Shape(sp) => match &sp.shape {
                crate::utils::Either::Left(shape) => PropertyVector::Shape(vec![*shape; n]),
                crate::utils::Either::Right(_) => {
                    PropertyVector::Shape(vec![crate::visuals::Shape::Circle; n])
                }
            },
            Property::String(sp) => match &sp.value {
                crate::utils::Either::Left(s) => PropertyVector::String(vec![s.clone(); n]),
                crate::utils::Either::Right(_) => PropertyVector::String(vec![String::new(); n]),
            },
        }
    }

    fn make_property_value_vector(&self, value: &PropertyValue, n: usize) -> PropertyVector {
        match value {
            PropertyValue::Int(val) => PropertyVector::Int(vec![*val; n]),
            PropertyValue::Float(val) => PropertyVector::Float(vec![*val; n]),
            PropertyValue::Color(color) => PropertyVector::Color(vec![*color; n]),
            PropertyValue::Shape(shape) => PropertyVector::Shape(vec![*shape; n]),
            PropertyValue::String(s) => PropertyVector::String(vec![s.clone(); n]),
        }
    }

    /// Create subset PropertyVectors for a specific group using indices
    fn subset_vectors(
        &self,
        all_vectors: &HashMap<AestheticProperty, PropertyVector>,
        indices: &[usize],
    ) -> HashMap<AestheticProperty, PropertyVector> {
        let mut subset = HashMap::new();

        for (property, vector) in all_vectors {
            let subset_vector = match vector {
                PropertyVector::Int(v) => {
                    PropertyVector::Int(indices.iter().map(|&i| v[i]).collect())
                }
                PropertyVector::Float(v) => {
                    PropertyVector::Float(indices.iter().map(|&i| v[i]).collect())
                }
                PropertyVector::Color(v) => {
                    PropertyVector::Color(indices.iter().map(|&i| v[i]).collect())
                }
                PropertyVector::Shape(v) => {
                    PropertyVector::Shape(indices.iter().map(|&i| v[i]).collect())
                }
                PropertyVector::String(v) => {
                    PropertyVector::String(indices.iter().map(|&i| v[i].clone()).collect())
                }
            };
            subset.insert(*property, subset_vector);
        }

        subset
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
                        (AestheticProperty::X, AestheticDomain::Discrete) => scales
                            .x_discrete
                            .map_aesthetic_value(value, data, &mut new_data)
                            .unwrap(),
                        (AestheticProperty::X, AestheticDomain::Continuous) => {
                            let v = scales
                                .x_continuous
                                .map_aesthetic_value(value, data, &mut new_data)
                                .unwrap();
                            log::info!("Mapped X aesthetic value: {:?}", v);
                            v
                        }
                        (AestheticProperty::Y, AestheticDomain::Discrete) => scales
                            .y_discrete
                            .map_aesthetic_value(value, data, &mut new_data)
                            .unwrap(),
                        (AestheticProperty::Y, AestheticDomain::Continuous) => scales
                            .y_continuous
                            .map_aesthetic_value(value, data, &mut new_data)
                            .unwrap(),
                        (AestheticProperty::Color, AestheticDomain::Continuous) => scales
                            .color_continuous
                            .map_aesthetic_value(value, data, &mut new_data)
                            .unwrap(),
                        (AestheticProperty::Color, AestheticDomain::Discrete) => scales
                            .color_discrete
                            .map_aesthetic_value(value, data, &mut new_data)
                            .unwrap(),
                        (AestheticProperty::Fill, AestheticDomain::Continuous) => scales
                            .fill_continuous
                            .map_aesthetic_value(value, data, &mut new_data)
                            .unwrap(),
                        (AestheticProperty::Fill, AestheticDomain::Discrete) => scales
                            .fill_discrete
                            .map_aesthetic_value(value, data, &mut new_data)
                            .unwrap(),
                        (AestheticProperty::Alpha, _) => scales
                            .alpha_scale
                            .map_aesthetic_value(value, data, &mut new_data)
                            .unwrap(),
                        (AestheticProperty::Size, AestheticDomain::Continuous) => scales
                            .size_continuous
                            .map_aesthetic_value(value, data, &mut new_data)
                            .unwrap(),
                        (AestheticProperty::Size, AestheticDomain::Discrete) => scales
                            .size_discrete
                            .map_aesthetic_value(value, data, &mut new_data)
                            .unwrap(),
                        (AestheticProperty::Shape, _) => scales
                            .shape_scale
                            .map_aesthetic_value(value, data, &mut new_data)
                            .unwrap(),
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
