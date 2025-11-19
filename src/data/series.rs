use std::sync::Arc;

use super::PrimitiveType;

pub trait Series: Send + Sync {
    type Type: PrimitiveType;

    fn name(&self) -> &str;

    fn len(&self) -> usize;

    fn get(&self, index: usize) -> Option<Self::Type>;

    fn as_any(&self) -> &dyn std::any::Any;

    fn range(&self) -> Option<(Self::Type, Self::Type)> {
        let n = self.len();

        if n == 0 {
            return None;
        }

        let mut min = self.get(0).unwrap();
        let mut max = self.get(0).unwrap();
        for i in 0..n {
            let item = self.get(i).unwrap();

            if item < min {
                min = item;
            } else if item > max {
                max = item;
            }
        }
        Some((min, max))
    }
}

pub trait TransformableSeries: Series {
    fn trans<F, U>(&self, f: F) -> Transform<Self::Type, U, Self, F>
    where
        F: Fn(Self::Type) -> U + Send + Sync + 'static,
        U: PrimitiveType,
        Self: Sized;
}

// Implement Series for Arc<T: Series>
impl<T: Series + 'static> Series for Arc<T> {
    type Type = T::Type;
    fn name(&self) -> &str {
        (**self).name()
    }
    fn len(&self) -> usize {
        (**self).len()
    }
    fn get(&self, index: usize) -> Option<Self::Type> {
        (**self).get(index)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn range(&self) -> Option<(Self::Type, Self::Type)> {
        (**self).range()
    }
}

impl<S: Series + 'static> TransformableSeries for Arc<S> {
    fn trans<F, U>(&self, f: F) -> Transform<S::Type, U, Arc<S>, F>
    where
        F: Fn(S::Type) -> U + Send + Sync + 'static,
        U: PrimitiveType,
        S: Sized,
    {
        Transform::new(self.clone(), f)
    }
}

impl<T: PrimitiveType> Series for Vec<T> {
    type Type = T;

    fn name(&self) -> &str {
        "unnamed_series"
    }

    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn get(&self, index: usize) -> Option<Self::Type> {
        self.as_slice().get(index).cloned()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub struct Transform<
    T: PrimitiveType,
    U: PrimitiveType,
    S: Series<Type = T> + Sized + 'static,
    F: Fn(T) -> U + Send + Sync + 'static,
> {
    pub(crate) series: S,
    pub(crate) f: F,
    pub(crate) _t_and_u: std::marker::PhantomData<(T, U)>,
}

impl<
    T: PrimitiveType,
    U: PrimitiveType,
    S: Series<Type = T> + Sized + 'static,
    F: Fn(T) -> U + Send + Sync + 'static,
> Transform<T, U, S, F>
{
    pub fn new(series: S, f: F) -> Self {
        Self {
            series,
            f,
            _t_and_u: std::marker::PhantomData,
        }
    }
}

impl<
    T: PrimitiveType,
    U: PrimitiveType,
    S: Series<Type = T> + Sized + 'static,
    F: Fn(T) -> U + Send + Sync + 'static,
> Series for Transform<T, U, S, F>
{
    type Type = U;
    fn name(&self) -> &str {
        "transformed_series"
    }
    fn len(&self) -> usize {
        self.series.len()
    }
    fn get(&self, index: usize) -> Option<Self::Type> {
        self.series.get(index).map(|v| (self.f)(v))
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub trait NumericTransform: TransformableSeries {
    fn sqrt(&self) -> Transform<f64, f64, Self, fn(f64) -> f64>
    where
        Self: Series<Type = f64> + Sized,
    {
       self.trans(f64::sqrt)
    }

    fn log(&self) -> Transform<f64, f64, Self, fn(f64) -> f64>
    where
        Self: Series<Type = f64> + Sized,
    {
        self.trans(f64::ln)
    }

    fn log10(&self) -> Transform<f64, f64, Self, fn(f64) -> f64>
    where
        Self: Series<Type = f64> + Sized,
    {
        self.trans(f64::log10)
    }

    fn exp(&self) -> Transform<f64, f64, Self, fn(f64) -> f64>
    where
        Self: Series<Type = f64> + Sized,
    {
        self.trans(f64::exp)
    }

    fn neg(&self) -> Transform<f64, f64, Self, fn(f64) -> f64>
    where
        Self: Series<Type = f64> + Sized,
    {
        self.trans(|x| -x)
    }
}

pub trait IntegerTransform: TransformableSeries {
    fn as_f64(&self) -> Transform<i64, f64, Self, fn(i64) -> f64>
    where
        Self: Series<Type = i64> + Sized,
    {
        self.trans(|x| x as f64)
    }

    fn sqrt(&self) -> Transform<i64, f64, Self, fn(i64) -> f64>
    where
        Self: Series<Type = i64> + Sized,
    {
        self.trans(|x| (x as f64).sqrt())
    }

    fn log(&self) -> Transform<i64, f64, Self, fn(i64) -> f64>
    where
        Self: Series<Type = i64> + Sized,
    {
        self.trans(|x| (x as f64).ln())
    }

    fn log10(&self) -> Transform<i64, f64, Self, fn(i64) -> f64>
    where
        Self: Series<Type = i64> + Sized,
    {
        self.trans(|x| (x as f64).log10())
    }

    fn exp(&self) -> Transform<i64, f64, Self, fn(i64) -> f64>
    where
        Self: Series<Type = i64> + Sized,
    {
        self.trans(|x| (x as f64).exp())
    }

    fn neg(&self) -> Transform<i64, i64, Self, fn(i64) -> i64>
    where
        Self: Series<Type = i64> + Sized,
    {
        self.trans(|x| -x)
    }
}

// Blanket implementation for all Series of i64
impl<S: Series<Type = i64> + TransformableSeries> IntegerTransform for S {}

// Blanket implementation for all Series of f64
impl<S: Series<Type = f64> + TransformableSeries> NumericTransform for S {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_series_len() {
        let s: Vec<i64> = vec![1, 2, 3];
        assert_eq!(s.len(), 3);
    }

    #[test]
    fn test_series_get() {
        let s: Vec<f64> = vec![1.0, 2.0, 3.0];
        assert_eq!(s.get(1), Some(2.0));
        assert_eq!(s.get(10), None);
    }

    #[test]
    fn test_series_name() {
        let s: Vec<String> = vec!["a".to_string(), "b".to_string()];
        assert_eq!(s.name(), "unnamed_series");
    }

    #[test]
    fn test_series_range_i64() {
        let s: Vec<i64> = vec![10, 2, 8, 5];
        let r = s.range();
        assert_eq!(r, Some((2, 10)));
    }

    #[test]
    fn test_series_range_f64() {
        let s: Vec<f64> = vec![1.5, 3.2, -7.0, 0.0];
        let r = s.range();
        assert_eq!(r, Some((-7.0, 3.2)));
    }

    #[test]
    fn test_series_range_empty() {
        let s: Vec<i64> = vec![];
        let r = s.range();
        assert_eq!(r, None);
    }

    #[test]
    fn test_series_range_string() {
        let s: Vec<String> = vec!["b".to_string(), "a".to_string(), "c".to_string()];
        let r = s.range();
        assert_eq!(r, Some(("a".to_string(), "c".to_string())));
    }

    #[test]
    fn test_trans_method() {
        let s: Arc<Vec<i64>> = Arc::new(vec![1, 2, 3]);
        let t = s.trans(|x| x * 2);
        assert_eq!(t.len(), 3);
        assert_eq!(t.get(0), Some(2));
        assert_eq!(t.get(1), Some(4));
        assert_eq!(t.get(2), Some(6));
        assert_eq!(t.name(), "transformed_series");
    }

    #[test]
    fn test_numeric_transform_sqrt() {
        let s: Arc<Vec<f64>> = Arc::new(vec![4.0, 9.0, 16.0]);
        let t = s.sqrt();
        assert!((t.get(0).unwrap() - 2.0).abs() < 1e-12);
        assert!((t.get(1).unwrap() - 3.0).abs() < 1e-12);
        assert!((t.get(2).unwrap() - 4.0).abs() < 1e-12);
    }

    #[test]
    fn test_numeric_transform_log() {
        let s: Arc<Vec<f64>> = Arc::new(vec![1.0, std::f64::consts::E, 10.0]);
        let t = s.log();
        assert!((t.get(0).unwrap() - 0.0).abs() < 1e-12);
        assert!((t.get(1).unwrap() - 1.0).abs() < 1e-12);
        assert!((t.get(2).unwrap() - 10.0_f64.ln()).abs() < 1e-12);
    }

    #[test]
    fn test_numeric_transform_log10() {
        let s: Arc<Vec<f64>> = Arc::new(vec![1.0, 10.0, 100.0]);
        let t = s.log10();
        assert!((t.get(0).unwrap() - 0.0).abs() < 1e-12);
        assert!((t.get(1).unwrap() - 1.0).abs() < 1e-12);
        assert!((t.get(2).unwrap() - 2.0).abs() < 1e-12);
    }

    #[test]
    fn test_integer_transform_sqrt() {
        let s: Arc<Vec<i64>> = Arc::new(vec![4, 9, 16]);
        let t = s.sqrt();
        assert!((t.get(0).unwrap() - 2.0).abs() < 1e-12);
        assert!((t.get(1).unwrap() - 3.0).abs() < 1e-12);
        assert!((t.get(2).unwrap() - 4.0).abs() < 1e-12);
    }

    #[test]
    fn test_integer_transform_log() {
        let s: Arc<Vec<i64>> = Arc::new(vec![1, 2, 10]);
        let t = s.log();
        assert!((t.get(0).unwrap() - 0.0).abs() < 1e-12);
        assert!((t.get(1).unwrap() - (2.0_f64).ln()).abs() < 1e-12);
        assert!((t.get(2).unwrap() - (10.0_f64).ln()).abs() < 1e-12);
    }

    #[test]
    fn test_integer_transform_log10() {
        let s: Arc<Vec<i64>> = Arc::new(vec![1, 10, 100]);
        let t = s.log10();
        assert!((t.get(0).unwrap() - 0.0).abs() < 1e-12);
        assert!((t.get(1).unwrap() - 1.0).abs() < 1e-12);
        assert!((t.get(2).unwrap() - 2.0).abs() < 1e-12);
    }

    #[test]
    fn test_integer_transform_exp() {
        let s: Arc<Vec<i64>> = Arc::new(vec![0, 1, 2]);
        let t = s.exp();
        assert!((t.get(0).unwrap() - 1.0).abs() < 1e-12);
        assert!((t.get(1).unwrap() - std::f64::consts::E).abs() < 1e-12);
        assert!((t.get(2).unwrap() - std::f64::consts::E.powi(2)).abs() < 1e-12);
    }

    #[test]
    fn test_integer_transform_neg() {
        let s: Arc<Vec<i64>> = Arc::new(vec![1, -2, 3]);
        let t = s.neg();
        assert!(t.get(0).unwrap() == -1);
        assert!(t.get(1).unwrap() == 2);
        assert!(t.get(2).unwrap() == -3);
    }
}
