
use crate::{data::{GenericVector, PrimitiveType, VectorIter}, utils::dataframe::{BoolVec, FloatVec, IntVec, StrVec}};

pub trait Vectorable: PrimitiveType {
    fn make_vector(vector: Vec<Self>) -> Box<dyn GenericVector>;
}

impl Vectorable for i64 {
    fn make_vector(vector: Vec<Self>) -> Box<dyn GenericVector> {
        Box::new(IntVec::from(vector))
    }
}

impl Vectorable for f64 {
    fn make_vector(vector: Vec<Self>) -> Box<dyn GenericVector> {
        Box::new(FloatVec::from(vector))
    }
}

impl Vectorable for String {
    fn make_vector(vector: Vec<Self>) -> Box<dyn GenericVector> {
        Box::new(StrVec::from(vector))
    }
}

impl Vectorable for bool {
    fn make_vector(vector: Vec<Self>) -> Box<dyn GenericVector> {
        Box::new(BoolVec::from(vector))
    }
}

pub trait VectorVisitor {
    fn visit<T: Vectorable>(&mut self, value: impl Iterator<Item = T>);
}

pub fn visit<'a, V: VectorVisitor>(
    iter: VectorIter<'a>,
    visitor: &mut V,
) {
    match iter {
        VectorIter::Int(it) => visitor.visit(it),
        VectorIter::Float(it) => visitor.visit(it),
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
    it1: impl  Iterator<Item = T>,
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
    it1: impl  Iterator<Item = T>,
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
    it1: impl  Iterator<Item = T>,
    it2: impl  Iterator<Item = U>,
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