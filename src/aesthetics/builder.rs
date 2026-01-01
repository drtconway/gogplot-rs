use crate::aesthetics::{AesMap, AesValue, Aesthetic, AestheticDomain};

pub trait AesMapBuilderTrait {
    fn aes(&mut self) -> &mut AesMap;
}

pub trait XContininuousAesBuilder: AesMapBuilderTrait {
    fn x_continuous(&mut self, column: &str) {
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
    fn x_discrete(&mut self, column: &str) {
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

pub trait XInterceptAesBuilder: AesMapBuilderTrait {
    fn x_intercept(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::XIntercept,
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait YContininuousAesBuilder: AesMapBuilderTrait {
    fn y_continuous(&mut self, column: &str) {
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
    fn y_discrete(&mut self, column: &str) {
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

pub trait YInterceptAesBuilder: AesMapBuilderTrait {
    fn y_intercept(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::YIntercept,
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait XMinContinuousAesBuilder: AesMapBuilderTrait {
    fn xmin(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Xmin(AestheticDomain::Continuous),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait XMaxContinuousAesBuilder: AesMapBuilderTrait {
    fn xmax(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Xmax(AestheticDomain::Continuous),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait YMinContinuousAesBuilder: AesMapBuilderTrait {
    fn ymin(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Ymin(AestheticDomain::Continuous),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: None,
            },
        );
    }
}

pub trait YMaxContinuousAesBuilder: AesMapBuilderTrait {
    fn ymax(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Ymax(AestheticDomain::Continuous),
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
                original_name: Some(column.to_string()),
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
                original_name: Some(column.to_string()),
            },
        );
    }
}

pub trait FillContinuousAesBuilder: AesMapBuilderTrait {
    fn fill_continuous(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Fill(AestheticDomain::Continuous),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: Some(column.to_string()),
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
                original_name: Some(column.to_string()),
            },
        );
    }
}

pub trait AlphaContinuousAesBuilder: AesMapBuilderTrait {
    fn alpha_continuous(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Alpha(AestheticDomain::Continuous),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: Some(column.to_string()),
            },
        );
    }
}

pub trait AlphaDiscreteAesBuilder: AesMapBuilderTrait {
    fn alpha_discrete(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Alpha(AestheticDomain::Discrete),
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: Some(column.to_string()),
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
                original_name: Some(column.to_string()),
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
                original_name: Some(column.to_string()),
            },
        );
    }
}

pub trait ShapeAesBuilder: AesMapBuilderTrait {
    fn shape(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Shape,
            AesValue::Column {
                name: column.to_string(),
                hint: None,
                original_name: Some(column.to_string()),
            },
        );
    }
}

pub trait LabelAesBuilder: AesMapBuilderTrait {
    fn label(&mut self, column: &str) {
        self.aes().set(
            Aesthetic::Label,
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

    pub fn build(mut self, parent_mapping: &AesMap, overrides: &[Aesthetic]) -> AesMap {
        for (aes, aes_value) in parent_mapping.iter() {
            if !self.aes_map.contains(*aes) {
                self.aes_map.set(*aes, aes_value.clone());
            }
        }
        for aes in overrides {
            self.aes_map.remove(aes);
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
impl XInterceptAesBuilder for AesMapBuilder {}
impl YContininuousAesBuilder for AesMapBuilder {}
impl YDiscreteAesBuilder for AesMapBuilder {}
impl YInterceptAesBuilder for AesMapBuilder {}
impl XMinContinuousAesBuilder for AesMapBuilder {}
impl XMaxContinuousAesBuilder for AesMapBuilder {}
impl YMinContinuousAesBuilder for AesMapBuilder {}
impl YMaxContinuousAesBuilder for AesMapBuilder {}
impl ColorContinuousAesBuilder for AesMapBuilder {}
impl ColorDiscreteAesBuilder for AesMapBuilder {}
impl FillContinuousAesBuilder for AesMapBuilder {}
impl FillDiscreteAesBuilder for AesMapBuilder {}
impl AlphaContinuousAesBuilder for AesMapBuilder {}
impl AlphaDiscreteAesBuilder for AesMapBuilder {}
impl SizeContinuousAesBuilder for AesMapBuilder {}
impl SizeDiscreteAesBuilder for AesMapBuilder {}
impl ShapeAesBuilder for AesMapBuilder {}
impl LabelAesBuilder for AesMapBuilder {}
impl GroupAesBuilder for AesMapBuilder {}

