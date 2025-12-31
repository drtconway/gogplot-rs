use std::sync::Arc;

use crate::{aesthetics::AesValue, data::VectorValue, utils::dataframe::DataFrame};

pub struct AesValueBuilder {
    value: AesValue,
}

impl From<AesValue> for AesValueBuilder {
    fn from(value: AesValue) -> Self {
        let value = match value {
            AesValue::Column {
                name,
                hint,
                original_name,
            } => AesValue::Column {
                name,
                hint,
                original_name,
            },
            AesValue::Constant { value, hint } => AesValue::Constant { value, hint },
            AesValue::Vector {
                values,
                original_name,
            } => AesValue::Vector {
                values: Arc::new(values.empty_copy()),
                original_name,
            },
        };
        Self { value }
    }
}

impl AesValueBuilder {
    pub fn append(&mut self, data: &mut DataFrame, extra: impl Into<VectorValue>) {
        match &mut self.value {
            AesValue::Column { name, .. } => {
                if data.has_column(name) {
                    panic!("Column {} already exists in DataFrame", name);
                } else {
                    data.add_column(name.clone(), extra.into());
                }
            }
            AesValue::Constant { value: _, .. } => todo!(),
            AesValue::Vector { values, .. } => {
                let vec = Arc::get_mut(values).expect("Cannot get mutable reference to vector");
                vec.append(&mut extra.into());
            }
        }
    }

    pub fn build(self) -> AesValue {
        self.value
    }
}
