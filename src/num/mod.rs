//! Ruby numbers.

use std::cmp::Ordering;
use crate::{
    prelude::*,
    ruby,
};

mod float;
mod integer;

pub use self::{
    float::*,
    integer::*,
};

impl PartialEq<Integer> for Float {
    #[inline]
    fn eq(&self, other: &Integer) -> bool {
        other == self
    }
}

impl PartialOrd<Integer> for Float {
    #[inline]
    fn partial_cmp(&self, other: &Integer) -> Option<Ordering> {
        Some(other.partial_cmp(self)?.reverse())
    }
}

impl PartialOrd<Float> for Integer {
    #[inline]
    fn partial_cmp(&self, other: &Float) -> Option<Ordering> {
        let val = unsafe { ruby::rb_big_cmp(self.raw(), other.raw()) };
        if val != crate::util::NIL_VALUE {
            Some(crate::util::value_to_fixnum(val).cmp(&0))
        } else {
            None
        }
    }
}

macro_rules! forward_int_cmp {
    ($($t:ty)+) => { $(
        impl PartialEq<$t> for AnyObject {
            #[inline]
            fn eq(&self, other: &$t) -> bool {
                if let Some(integer) = self.to_integer() {
                    integer == *other
                } else if let Some(float) = self.to_float() {
                    float == Integer::from(*other)
                } else {
                    false
                }
            }
        }

        impl PartialEq<AnyObject> for $t {
            #[inline]
            fn eq(&self, other: &AnyObject) -> bool {
                other == self
            }
        }

        impl PartialOrd<$t> for AnyObject {
            #[inline]
            fn partial_cmp(&self, other: &$t) -> Option<Ordering> {
                if let Some(integer) = self.to_integer() {
                    integer.partial_cmp(other)
                } else if let Some(float) = self.to_float() {
                    float.partial_cmp(&Integer::from(*other))
                } else {
                    None
                }
            }
        }

        impl PartialOrd<AnyObject> for $t {
            #[inline]
            fn partial_cmp(&self, other: &AnyObject) -> Option<Ordering> {
                Some(other.partial_cmp(self)?.reverse())
            }
        }
    )+ }
}

forward_int_cmp! {
    usize u128 u64 u32 u16 u8
    isize i128 i64 i32 i16 i8
}
