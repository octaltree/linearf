use std::cmp::Ordering;

#[derive(Debug)]
pub enum Value {
    I64(i64),
    F64(f64),
    Vi16(Vec<i16>),
    Vi64(Vec<i64>),
    Vf64(Vec<f64>)
}

macro_rules! from_impl {
    ($v:ident, $t:ty) => {
        impl From<$t> for Value {
            fn from(x: $t) -> Self { Self::$v(x) }
        }
    };
}
from_impl!(I64, i64);
from_impl!(F64, f64);
from_impl!(Vi16, Vec<i16>);
from_impl!(Vi64, Vec<i64>);
from_impl!(Vf64, Vec<f64>);

/// Handles i64 as length 1
/// does not compare if type is mismatch
impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool { matches!(self.partial_cmp(other), Some(Ordering::Equal)) }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        use Value::*;
        macro_rules! iter_partial_cmp {
            ($a:expr, $b:expr) => {{
                for (a, b) in $a.iter().zip($b.iter()) {
                    match b.partial_cmp(a) {
                        Some(Ordering::Equal) => {}
                        Some(x) => return Some(x),
                        None => return None
                    }
                }
                None
            }};
        }
        match (self, other) {
            (I64(a), I64(b)) => a.partial_cmp(b),
            (F64(a), F64(b)) => a.partial_cmp(b),
            (Vi16(a), Vi16(b)) => iter_partial_cmp!(a, b),
            (Vi64(a), Vi64(b)) => iter_partial_cmp!(a, b),
            (Vf64(a), Vf64(b)) => iter_partial_cmp!(a, b),
            (F64(a), Vf64(b)) => iter_partial_cmp!(Some(a), b),
            (Vf64(a), F64(b)) => iter_partial_cmp!(a, Some(*b)),
            (I64(a), Vi64(b)) => iter_partial_cmp!(Some(a), b),
            (Vi64(a), I64(b)) => iter_partial_cmp!(a, Some(*b)),
            _ => None
        }
    }
}

/// Items will be displayed in v DESC, item_id ASC.
/// No guarantee of order when it is equal.
#[derive(Debug)]
pub struct Score {
    item_id: u32,
    value: Value
}

impl Score {
    pub fn new_excluded() -> Self {
        Self {
            item_id: 0,
            value: Value::F64(f64::NAN)
        }
    }

    pub fn value<V: Into<Value>>(item_id: u32, v: V) -> Self {
        Self {
            item_id,
            value: v.into()
        }
    }

    /// If true, the item will not be displayed.
    #[inline]
    pub fn should_be_excluded(&self) -> bool {
        use Value::*;
        match &self.value {
            F64(x) => x.is_nan(),
            Vi64(xs) => xs.is_empty(),
            Vi16(xs) => xs.is_empty(),
            Vf64(xs) => xs.is_empty(),
            _ => false
        }
    }
}

impl PartialEq for Score {
    fn eq(&self, other: &Self) -> bool { matches!(self.partial_cmp(other), Some(Ordering::Equal)) }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.value.partial_cmp(&other.value) {
            Some(Ordering::Equal) => Some(self.item_id.cmp(&other.item_id)),
            Some(x) => Some(x),
            None => None
        }
    }
}
