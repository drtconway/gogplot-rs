use std::sync::Arc;

use crate::{
    data::{ContinuousType, DiscreteType, GenericVector, PrimitiveType, VectorIter, VectorType, VectorValue},
    error::{DataType, PlotError},
    theme::Color,
    utils::dataframe::{BoolVec, FloatVec, IntVec, StrVec},
    visuals::Shape,
};

pub trait GenericVectorable: PrimitiveType {
    fn make_vector(vector: Vec<Self>) -> Arc<dyn GenericVector>;
}

impl GenericVectorable for i64 {
    fn make_vector(vector: Vec<Self>) -> Arc<dyn GenericVector> {
        Arc::new(IntVec::from(vector))
    }
}

impl GenericVectorable for f64 {
    fn make_vector(vector: Vec<Self>) -> Arc<dyn GenericVector> {
        Arc::new(FloatVec::from(vector))
    }
}

impl GenericVectorable for String {
    fn make_vector(vector: Vec<Self>) -> Arc<dyn GenericVector> {
        Arc::new(StrVec::from(vector))
    }
}

impl GenericVectorable for bool {
    fn make_vector(vector: Vec<Self>) -> Arc<dyn GenericVector> {
        Arc::new(BoolVec::from(vector))
    }
}

pub trait Vectorable: PrimitiveType {
    fn make_vector(vector: Vec<Self>) -> VectorValue;
}

impl Vectorable for i64 {
    fn make_vector(vector: Vec<Self>) -> VectorValue {
        VectorValue::Int(vector)
    }
}

impl Vectorable for f64 {
    fn make_vector(vector: Vec<Self>) -> VectorValue {
        VectorValue::Float(vector)
    }
}

impl Vectorable for String {
    fn make_vector(vector: Vec<Self>) -> VectorValue {
        VectorValue::Str(vector)
    }
}

impl Vectorable for bool {
    fn make_vector(vector: Vec<Self>) -> VectorValue {
        VectorValue::Bool(vector)
    }
}

pub trait VectorVisitor {
    type Output;

    fn visit<T: Vectorable>(&mut self, value: impl Iterator<Item = T>) -> std::result::Result<Self::Output, PlotError>;
}

pub fn visit<'a, V: VectorVisitor>(iter: VectorIter<'a>, visitor: &mut V) -> Result<V::Output, PlotError> {
    match iter {
        VectorIter::Int(it) => visitor.visit(it),
        VectorIter::Float(it) => visitor.visit(it),
        VectorIter::Str(it) => visitor.visit(it.map(|s| s.to_string())),
        VectorIter::Bool(it) => visitor.visit(it),
    }
}

/// Visitor that only accepts continuous types (i64, f64)
pub trait ContinuousVectorVisitor {
    type Output;

    fn visit<T: Vectorable + ContinuousType>(
        &mut self,
        value: impl Iterator<Item = T>,
    ) -> std::result::Result<Self::Output, PlotError>;
}

pub fn visit_c<'a, V: ContinuousVectorVisitor>(
    iter: VectorIter<'a>,
    visitor: &mut V,
) -> Result<V::Output, PlotError> {
    match iter {
        VectorIter::Int(it) => visitor.visit(it),
        VectorIter::Float(it) => visitor.visit(it),
        VectorIter::Str(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Continuous,
            actual: DataType::Vector(VectorType::Str),
        }),
        VectorIter::Bool(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Continuous,
            actual: DataType::Vector(VectorType::Bool),
        }),
    }
}

/// Visitor that only accepts discrete types (i64, String, bool)
pub trait DiscreteVectorVisitor {
    type Output;

    fn visit<T: Vectorable + DiscreteType>(
        &mut self,
        value: impl Iterator<Item = T>,
    ) -> std::result::Result<Self::Output, PlotError>;
}

pub fn visit_d<'a, V: DiscreteVectorVisitor>(
    iter: VectorIter<'a>,
    visitor: &mut V,
) -> Result<V::Output, PlotError> {
    match iter {
        VectorIter::Int(it) => visitor.visit(it),
        VectorIter::Float(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Discrete,
            actual: DataType::Vector(VectorType::Float),
        }),
        VectorIter::Str(it) => visitor.visit(it.map(|s| s.to_string())),
        VectorIter::Bool(it) => visitor.visit(it),
    }
}

pub trait VectorVisitor2 {
    fn visit<T: Vectorable, U: Vectorable>(
        &mut self,
        value1: impl Iterator<Item = T>,
        value2: impl Iterator<Item = U>,
    );
}

pub fn visit2<'a, V: VectorVisitor2>(
    iter1: VectorIter<'a>,
    iter2: VectorIter<'a>,
    visitor: &mut V,
) {
    match iter1 {
        VectorIter::Int(it1) => visit2_inner(it1, iter2, visitor),
        VectorIter::Float(it1) => visit2_inner(it1, iter2, visitor),
        VectorIter::Str(it1) => visit2_inner(it1.map(|s| s.to_string()), iter2, visitor),
        VectorIter::Bool(it1) => visit2_inner(it1, iter2, visitor),
    }
}

fn visit2_inner<T: Vectorable, V: VectorVisitor2>(
    it1: impl Iterator<Item = T>,
    iter2: VectorIter,
    visitor: &mut V,
) {
    match iter2 {
        VectorIter::Int(it2) => visitor.visit(it1, it2),
        VectorIter::Float(it2) => visitor.visit(it1, it2),
        VectorIter::Str(it2) => visitor.visit(it1, it2.map(|s| s.to_string())),
        VectorIter::Bool(it2) => visitor.visit(it1, it2),
    }
}

/// Visitor for two vectors where the first must be continuous
pub trait ContinuousVectorVisitor2 {
    fn visit<T: Vectorable + ContinuousType, U: Vectorable>(
        &mut self,
        value1: impl Iterator<Item = T>,
        value2: impl Iterator<Item = U>,
    );
}

pub fn visit2_ca<'a, V: ContinuousVectorVisitor2>(
    iter1: VectorIter<'a>,
    iter2: VectorIter<'a>,
    visitor: &mut V,
) -> Result<(), PlotError> {
    match iter1 {
        VectorIter::Int(it1) => {
            visit2_ca_inner(it1, iter2, visitor);
            Ok(())
        }
        VectorIter::Float(it1) => {
            visit2_ca_inner(it1, iter2, visitor);
            Ok(())
        }
        VectorIter::Str(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Continuous,
            actual: DataType::Vector(VectorType::Str),
        }),
        VectorIter::Bool(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Continuous,
            actual: DataType::Vector(VectorType::Bool),
        }),
    }
}

fn visit2_ca_inner<T: Vectorable + ContinuousType, V: ContinuousVectorVisitor2>(
    it1: impl Iterator<Item = T>,
    iter2: VectorIter,
    visitor: &mut V,
) {
    match iter2 {
        VectorIter::Int(it2) => visitor.visit(it1, it2),
        VectorIter::Float(it2) => visitor.visit(it1, it2),
        VectorIter::Str(it2) => visitor.visit(it1, it2.map(|s| s.to_string())),
        VectorIter::Bool(it2) => visitor.visit(it1, it2),
    }
}

/// Visitor for two vectors where both must be continuous
pub trait ContinuousContinuousVisitor2 {
    type Output;

    fn visit<T: Vectorable + ContinuousType, U: Vectorable + ContinuousType>(
        &mut self,
        value1: impl Iterator<Item = T>,
        value2: impl Iterator<Item = U>,
    ) -> std::result::Result<Self::Output, PlotError>;
}

pub fn visit2_cc<'a, V: ContinuousContinuousVisitor2>(
    iter1: VectorIter<'a>,
    iter2: VectorIter<'a>,
    visitor: &mut V,
) -> Result<V::Output, PlotError> {
    match iter1 {
        VectorIter::Int(it1) => visit2_cc_inner(it1, iter2, visitor),
        VectorIter::Float(it1) => visit2_cc_inner(it1, iter2, visitor),
        VectorIter::Str(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Continuous,
            actual: DataType::Vector(VectorType::Str),
        }),
        VectorIter::Bool(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Continuous,
            actual: DataType::Vector(VectorType::Bool),
        }),
    }
}

fn visit2_cc_inner<T: Vectorable + ContinuousType, V: ContinuousContinuousVisitor2>(
    it1: impl Iterator<Item = T>,
    iter2: VectorIter,
    visitor: &mut V,
) -> Result<V::Output, PlotError> {
    match iter2 {
        VectorIter::Int(it2) => visitor.visit(it1, it2),
        VectorIter::Float(it2) => visitor.visit(it1, it2),
        VectorIter::Str(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Continuous,
            actual: DataType::Vector(VectorType::Str),
        }),
        VectorIter::Bool(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Continuous,
            actual: DataType::Vector(VectorType::Bool),
        }),
    }
}

pub trait DiscreteVectorVisitor2 {
    fn visit<T: Vectorable + DiscreteType, U: Vectorable>(
        &mut self,
        value1: impl Iterator<Item = T>,
        value2: impl Iterator<Item = U>,
    );
}

pub fn visit2_da<'a, V: DiscreteVectorVisitor2>(
    iter1: VectorIter<'a>,
    iter2: VectorIter<'a>,
    visitor: &mut V,
) -> Result<(), PlotError> {
    match iter1 {
        VectorIter::Int(it1) => {
            visit2_da_inner(it1, iter2, visitor);
            Ok(())
        }
        VectorIter::Str(it1) => {
            visit2_da_inner(it1.map(|s| s.to_string()), iter2, visitor);
            Ok(())
        }
        VectorIter::Float(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Discrete,
            actual: DataType::Vector(VectorType::Float),
        }),
        VectorIter::Bool(it1) => {
            visit2_da_inner(it1, iter2, visitor);
            Ok(())
        }
    }
}

fn visit2_da_inner<T: Vectorable + DiscreteType, V: DiscreteVectorVisitor2>(
    it1: impl Iterator<Item = T>,
    iter2: VectorIter,
    visitor: &mut V,
) {
    match iter2 {
        VectorIter::Int(it2) => visitor.visit(it1, it2),
        VectorIter::Float(it2) => visitor.visit(it1, it2),
        VectorIter::Str(it2) => visitor.visit(it1, it2.map(|s| s.to_string())),
        VectorIter::Bool(it2) => visitor.visit(it1, it2),
    }
}

pub trait DiscreteDiscreteVisitor2 {
    fn visit<T: Vectorable + DiscreteType, U: Vectorable + DiscreteType>(
        &mut self,
        value1: impl Iterator<Item = T>,
        value2: impl Iterator<Item = U>,
    );
}

pub fn visit2_dd<'a, V: DiscreteDiscreteVisitor2>(
    iter1: VectorIter<'a>,
    iter2: VectorIter<'a>,
    visitor: &mut V,
) -> Result<(), PlotError> {
    match iter1 {
        VectorIter::Int(it1) => visit2_dd_inner(it1, iter2, visitor),
        VectorIter::Str(it1) => visit2_dd_inner(it1.map(|s| s.to_string()), iter2, visitor),
        VectorIter::Float(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Discrete,
            actual: DataType::Vector(VectorType::Float),
        }),
        VectorIter::Bool(it1) => visit2_dd_inner(it1, iter2, visitor),
    }
}

fn visit2_dd_inner<T: Vectorable + DiscreteType, V: DiscreteDiscreteVisitor2>(
    it1: impl Iterator<Item = T>,
    iter2: VectorIter,
    visitor: &mut V,
) -> Result<(), PlotError> {
    match iter2 {
        VectorIter::Int(it2) => {
            visitor.visit(it1, it2);
            Ok(())
        }
        VectorIter::Str(it2) => {
            visitor.visit(it1, it2.map(|s| s.to_string()));
            Ok(())
        }
        VectorIter::Float(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Discrete,
            actual: DataType::Vector(VectorType::Float),
        }),
        VectorIter::Bool(it2) => {
            visitor.visit(it1, it2);
            Ok(())
        }
    }
}

pub trait DiscreteContinuousVisitor2 {
    type Output;

    fn visit<T: Vectorable + DiscreteType, U: Vectorable + ContinuousType>(
        &mut self,
        value1: impl Iterator<Item = T>,
        value2: impl Iterator<Item = U>,
    ) -> std::result::Result<Self::Output, PlotError>;
}

pub fn visit2_dc<'a, V: DiscreteContinuousVisitor2>(
    iter1: VectorIter<'a>,
    iter2: VectorIter<'a>,
    visitor: &mut V,
) -> Result<V::Output, PlotError> {
    match iter1 {
        VectorIter::Int(it1) => visit2_dc_inner(it1, iter2, visitor),
        VectorIter::Str(it1) => visit2_dc_inner(it1.map(|s| s.to_string()), iter2, visitor),
        VectorIter::Float(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Discrete,
            actual: DataType::Vector(VectorType::Float),
        }),
        VectorIter::Bool(it1) => visit2_dc_inner(it1, iter2, visitor),
    }
}

fn visit2_dc_inner<T: Vectorable + DiscreteType, V: DiscreteContinuousVisitor2>(
    it1: impl Iterator<Item = T>,
    iter2: VectorIter,
    visitor: &mut V,
) -> Result<V::Output, PlotError> {
    match iter2 {
        VectorIter::Int(it2) => visitor.visit(it1, it2),
        VectorIter::Float(it2) => visitor.visit(it1, it2),
        VectorIter::Str(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Continuous,
            actual: DataType::Vector(VectorType::Str),
        }),
        VectorIter::Bool(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Continuous,
            actual: DataType::Vector(VectorType::Bool),
        }),
    }
}

pub trait VectorVisitor3 {
    fn visit<T: Vectorable, U: Vectorable, V: Vectorable>(
        &mut self,
        value1: impl Iterator<Item = T>,
        value2: impl Iterator<Item = U>,
        value3: impl Iterator<Item = V>,
    );
}

pub fn visit3<'a, V: VectorVisitor3>(
    iter1: VectorIter<'a>,
    iter2: VectorIter<'a>,
    iter3: VectorIter<'a>,
    visitor: &mut V,
) {
    match iter1 {
        VectorIter::Int(it1) => visit3_inner1(it1, iter2, iter3, visitor),
        VectorIter::Float(it1) => visit3_inner1(it1, iter2, iter3, visitor),
        VectorIter::Str(it1) => visit3_inner1(it1.map(|s| s.to_string()), iter2, iter3, visitor),
        VectorIter::Bool(it1) => visit3_inner1(it1, iter2, iter3, visitor),
    }
}

fn visit3_inner1<T: Vectorable, V: VectorVisitor3>(
    it1: impl Iterator<Item = T>,
    iter2: VectorIter,
    iter3: VectorIter,
    visitor: &mut V,
) {
    match iter2 {
        VectorIter::Int(it2) => visit3_inner2(it1, it2, iter3, visitor),
        VectorIter::Float(it2) => visit3_inner2(it1, it2, iter3, visitor),
        VectorIter::Str(it2) => visit3_inner2(it1, it2.map(|s| s.to_string()), iter3, visitor),
        VectorIter::Bool(it2) => visit3_inner2(it1, it2, iter3, visitor),
    }
}

fn visit3_inner2<T: Vectorable, U: Vectorable, V: VectorVisitor3>(
    it1: impl Iterator<Item = T>,
    it2: impl Iterator<Item = U>,
    iter3: VectorIter,
    visitor: &mut V,
) {
    match iter3 {
        VectorIter::Int(it3) => visitor.visit(it1, it2, it3),
        VectorIter::Float(it3) => visitor.visit(it1, it2, it3),
        VectorIter::Str(it3) => visitor.visit(it1, it2, it3.map(|s| s.to_string())),
        VectorIter::Bool(it3) => visitor.visit(it1, it2, it3),
    }
}

pub trait DiscreteContinuousContinuousVisitor3 {
    type Output;

    fn visit<
        T: Vectorable + DiscreteType,
        U: Vectorable + ContinuousType,
        V: Vectorable + ContinuousType,
    >(
        &mut self,
        value1: impl Iterator<Item = T>,
        value2: impl Iterator<Item = U>,
        value3: impl Iterator<Item = V>,
    ) -> std::result::Result<Self::Output, PlotError>;
}

pub fn visit3_dcc<'a, V: DiscreteContinuousContinuousVisitor3>(
    iter1: VectorIter<'a>,
    iter2: VectorIter<'a>,
    iter3: VectorIter<'a>,
    visitor: &mut V,
) -> Result<V::Output, PlotError> {
    match iter1 {
        VectorIter::Int(it1) => visit3_dcc_inner1(it1, iter2, iter3, visitor),
        VectorIter::Str(it1) => {
            visit3_dcc_inner1(it1.map(|s| s.to_string()), iter2, iter3, visitor)
        }
        VectorIter::Float(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Discrete,
            actual: DataType::Vector(VectorType::Float),
        }),
        VectorIter::Bool(it1) => visit3_dcc_inner1(it1, iter2, iter3, visitor),
    }
}

fn visit3_dcc_inner1<T: Vectorable + DiscreteType, V: DiscreteContinuousContinuousVisitor3>(
    it1: impl Iterator<Item = T>,
    iter2: VectorIter,
    iter3: VectorIter,
    visitor: &mut V,
) -> Result<V::Output, PlotError> {
    match iter2 {
        VectorIter::Int(it2) => visit3_dcc_inner2(it1, it2, iter3, visitor),
        VectorIter::Float(it2) => visit3_dcc_inner2(it1, it2, iter3, visitor),
        VectorIter::Str(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Continuous,
            actual: DataType::Vector(VectorType::Str),
        }),
        VectorIter::Bool(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Continuous,
            actual: DataType::Vector(VectorType::Bool),
        }),
    }
}

fn visit3_dcc_inner2<
    T: Vectorable + DiscreteType,
    U: Vectorable + ContinuousType,
    V: DiscreteContinuousContinuousVisitor3,
>(
    it1: impl Iterator<Item = T>,
    it2: impl Iterator<Item = U>,
    iter3: VectorIter,
    visitor: &mut V,
) -> Result<V::Output, PlotError> {
    match iter3 {
        VectorIter::Int(it3) => visitor.visit(it1, it2, it3),
        VectorIter::Float(it3) => visitor.visit(it1, it2, it3),
        VectorIter::Str(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Continuous,
            actual: DataType::Vector(VectorType::Str),
        }),
        VectorIter::Bool(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Continuous,
            actual: DataType::Vector(VectorType::Bool),
        }),
    }
}

pub trait DiscreteDiscreteContinuousVisitor3 {
    type Output;

    fn visit<
        T: Vectorable + DiscreteType,
        U: Vectorable + DiscreteType,
        V: Vectorable + ContinuousType,
    >(
        &mut self,
        value1: impl Iterator<Item = T>,
        value2: impl Iterator<Item = U>,
        value3: impl Iterator<Item = V>,
    ) -> std::result::Result<Self::Output, PlotError>;
}

pub fn visit3_ddc<'a, V: DiscreteDiscreteContinuousVisitor3>(
    iter1: VectorIter<'a>,
    iter2: VectorIter<'a>,
    iter3: VectorIter<'a>,
    visitor: &mut V,
) -> Result<V::Output, PlotError> {
    match iter1 {
        VectorIter::Int(it1) => visit3_ddc_inner1(it1, iter2, iter3, visitor),
        VectorIter::Str(it1) => {
            visit3_ddc_inner1(it1.map(|s| s.to_string()), iter2, iter3, visitor)
        }
        VectorIter::Float(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Discrete,
            actual: DataType::Vector(VectorType::Float),
        }),
        VectorIter::Bool(it1) => visit3_ddc_inner1(it1, iter2, iter3, visitor),
    }
}

fn visit3_ddc_inner1<T: Vectorable + DiscreteType, V: DiscreteDiscreteContinuousVisitor3>(
    it1: impl Iterator<Item = T>,
    iter2: VectorIter,
    iter3: VectorIter,
    visitor: &mut V,
) -> Result<V::Output, PlotError> {
    match iter2 {
        VectorIter::Int(it2) => visit3_ddc_inner2(it1, it2, iter3, visitor),
        VectorIter::Str(it2) => visit3_ddc_inner2(it1, it2.map(|s| s.to_string()), iter3, visitor),
        VectorIter::Float(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Discrete,
            actual: DataType::Vector(VectorType::Float),
        }),
        VectorIter::Bool(it2) => visit3_ddc_inner2(it1, it2, iter3, visitor),
    }
}

fn visit3_ddc_inner2<
    T: Vectorable + DiscreteType,
    U: Vectorable + DiscreteType,
    V: DiscreteDiscreteContinuousVisitor3,
>(
    it1: impl Iterator<Item = T>,
    it2: impl Iterator<Item = U>,
    iter3: VectorIter,
    visitor: &mut V,
) -> Result<V::Output, PlotError> {
    match iter3 {
        VectorIter::Int(it3) => visitor.visit(it1, it2, it3),
        VectorIter::Float(it3) => visitor.visit(it1, it2, it3),
        VectorIter::Str(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Continuous,
            actual: DataType::Vector(VectorType::Str),
        }),
        VectorIter::Bool(_) => Err(PlotError::AestheticDomainMismatch {
            expected: crate::aesthetics::AestheticDomain::Continuous,
            actual: DataType::Vector(VectorType::Bool),
        }),
    }
}

pub fn make_color_iter<'a>(
    iter: VectorIter<'a>,
) -> impl Iterator<Item = Color> + 'a {
    match iter {
        VectorIter::Int(int_iter) => int_iter.map(|v| Color::from(v)),
        _ => panic!("Color must be specified as integer RGBA values"),
    }
}

pub fn make_float_iter<'a>(
    iter: VectorIter<'a>,
) -> impl Iterator<Item = f64> + 'a {
    match iter {
        VectorIter::Float(float_iter) => float_iter.map(|v| v),
        _ => panic!("Size must be specified as float values"),
    }
}

pub fn make_string_iter<'a>(
    iter: VectorIter<'a>,
) -> impl Iterator<Item = String> + 'a {
    match iter {
        VectorIter::Str(str_iter) => str_iter.map(|v| v.to_string()),
        _ => panic!("Shape must be specified as string values"),
    }
}

pub fn make_shape_iter<'a>(
    iter: VectorIter<'a>,
) -> impl Iterator<Item = Shape> + 'a {
    match iter {
        VectorIter::Int(int_iter) => int_iter.map(|v| Shape::from(v)),
        _ => panic!("Size must be specified as int values"),
    }
}
