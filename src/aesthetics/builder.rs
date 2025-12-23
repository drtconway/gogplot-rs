use crate::aesthetics::{AesMap, AesValue, Aesthetic, AestheticDomain};

pub trait AesMapBuilderTrait {
    fn aes(&mut self) -> &mut AesMap;
}

pub trait XContininuousAesBuilder: AesMapBuilderTrait {
    fn x(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::X(AestheticDomain::Continuous),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait XDiscreteAesBuilder: AesMapBuilderTrait {
    fn x(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::X(AestheticDomain::Discrete),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait YContininuousAesBuilder: AesMapBuilderTrait {
    fn y(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Y(AestheticDomain::Continuous),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait YDiscreteAesBuilder: AesMapBuilderTrait {
    fn y(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Y(AestheticDomain::Discrete),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait ColorContinuousAesBuilder: AesMapBuilderTrait {
    fn color_continuous(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Color(AestheticDomain::Continuous),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait ColorDiscreteAesBuilder: AesMapBuilderTrait {
    fn color_discrete(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Color(AestheticDomain::Discrete),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait FillContinuousAesBuilder: AesMapBuilderTrait {
    fn fill(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Fill(AestheticDomain::Continuous),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait FillDiscreteAesBuilder: AesMapBuilderTrait {
    fn fill_discrete(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Fill(AestheticDomain::Discrete),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait SizeContinuousAesBuilder: AesMapBuilderTrait {
    fn size_continuous(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Size(AestheticDomain::Continuous),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait SizeDiscreteAesBuilder: AesMapBuilderTrait {
    fn size_discrete(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Size(AestheticDomain::Discrete),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait GroupAesBuilder: AesMapBuilderTrait {
    fn group(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Group,
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub struct AesMapBuilder {
    aes_map: crate::aesthetics::AesMap,
}

impl AesMapBuilder {
    pub fn new() -> Self {
        Self {
            aes_map: crate::aesthetics::AesMap::new(),
        }
    }

    pub fn build(mut self, parent_mapping: &AesMap) -> AesMap {
        for (aes, aes_value) in parent_mapping.iter() {
            if !self.aes_map.contains(*aes) {
                self.aes_map.set(*aes, aes_value.clone());
            }
        }
        self.aes_map
    }
}

impl AesMapBuilderTrait for AesMapBuilder {
    fn aes(&mut self) -> &mut AesMap {
        &mut self.aes_map
    }
}

impl XContininuousAesBuilder for AesMapBuilder {}
impl XDiscreteAesBuilder for AesMapBuilder {}
impl YContininuousAesBuilder for AesMapBuilder {}
impl YDiscreteAesBuilder for AesMapBuilder {}
impl ColorContinuousAesBuilder for AesMapBuilder {}
impl ColorDiscreteAesBuilder for AesMapBuilder {}
impl FillContinuousAesBuilder for AesMapBuilder {}
impl FillDiscreteAesBuilder for AesMapBuilder {}
impl SizeContinuousAesBuilder for AesMapBuilder {}
impl SizeDiscreteAesBuilder for AesMapBuilder {}
impl GroupAesBuilder for AesMapBuilder {}

