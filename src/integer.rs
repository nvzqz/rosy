//! Ruby integers.

use std::{
    fmt,
    mem,
};
use crate::{
    prelude::*,
    object::{NonNullObject, Ty},
    ruby,
};

/// An instance of Ruby's `Integer` class.
#[derive(Clone, Copy, Debug)]
pub struct Integer(NonNullObject);

impl AsRef<AnyObject> for Integer {
    #[inline]
    fn as_ref(&self) -> &AnyObject { self.0.as_ref() }
}

impl From<Integer> for AnyObject {
    #[inline]
    fn from(obj: Integer) -> Self { obj.0.into() }
}

impl<O: Object> PartialEq<O> for Integer {
    #[inline]
    fn eq(&self, other: &O) -> bool {
        self.as_any_object() == other
    }
}

unsafe impl Object for Integer {
    #[inline]
    fn unique_id() -> Option<u128> {
        Some(!((Ty::Fixnum as u128) | ((Ty::Bignum as u128) << 8)))
    }

    fn cast<A: Object>(object: A) -> Option<Self> {
        unimplemented!()
    }

    #[inline]
    fn ty(self) -> Ty {
        if self.is_fixnum() {
            Ty::Fixnum
        } else {
            Ty::Bignum
        }
    }

    #[inline]
    fn is_ty(self, ty: Ty) -> bool {
        self.ty() == ty
    }
}

impl From<usize> for Integer {
    #[inline]
    fn from(int: usize) -> Self {
        unsafe { Self::from_raw(ruby::rb_uint2inum(int)) }
    }
}

impl From<isize> for Integer {
    #[inline]
    fn from(int: isize) -> Self {
        unsafe { Self::from_raw(ruby::rb_int2inum(int)) }
    }
}

impl From<u128> for Integer {
    #[inline]
    fn from(int: u128) -> Self {
        unsafe { Self::from_raw(crate::util::unpack_word(int, false)) }
    }
}

impl From<i128> for Integer {
    #[inline]
    fn from(int: i128) -> Self {
        unsafe { Self::from_raw(crate::util::unpack_word(int, true)) }
    }
}

impl From<u64> for Integer {
    #[inline]
    fn from(int: u64) -> Self {
        if mem::size_of::<u64>() == mem::size_of::<usize>() {
            (int as usize).into()
        } else {
            unsafe { Self::from_raw(crate::util::unpack_word(int, false)) }
        }
    }
}

impl From<i64> for Integer {
    #[inline]
    fn from(int: i64) -> Self {
        if mem::size_of::<i64>() == mem::size_of::<isize>() {
            (int as isize).into()
        } else {
            unsafe { Self::from_raw(crate::util::unpack_word(int, true)) }
        }
    }
}

impl From<u32> for Integer {
    #[inline]
    fn from(int: u32) -> Self {
        (int as usize).into()
    }
}

impl From<i32> for Integer {
    #[inline]
    fn from(int: i32) -> Self {
        (int as isize).into()
    }
}

impl From<u16> for Integer {
    #[inline]
    fn from(int: u16) -> Self {
        (int as usize).into()
    }
}

impl From<i16> for Integer {
    #[inline]
    fn from(int: i16) -> Self {
        (int as isize).into()
    }
}

impl From<u8> for Integer {
    #[inline]
    fn from(int: u8) -> Self {
        (int as usize).into()
    }
}

impl From<i8> for Integer {
    #[inline]
    fn from(int: i8) -> Self {
        (int as isize).into()
    }
}

macro_rules! forward_from {
    ($($t:ty)+) => { $(
        impl From<$t> for AnyObject {
            #[inline]
            fn from(int: $t) -> Self {
                Integer::from(int).into()
            }
        }
    )+ }
}

forward_from! {
    usize u128 u64 u32 u16 u8
    isize i128 i64 i32 i16 i8
}

impl fmt::Display for Integer {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_any_object().fmt(f)
    }
}

impl Integer {
    /// Returns whether `self` is a variable-sized integer.
    #[inline]
    pub fn is_bignum(self) -> bool {
        !self.is_fixnum()
    }

    /// Returns whether `self` is a fixed-sized integer.
    #[inline]
    pub fn is_fixnum(self) -> bool {
        crate::util::value_is_fixnum(self.raw())
    }

    /// Returns the value of the fixed-width integer stored in `self`.
    #[inline]
    pub fn fixnum_value(self) -> Option<i64> {
        if self.is_fixnum() {
            Some(crate::util::value_to_fixnum(self.raw()) as i64)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values() {
        crate::vm::init().unwrap();

        macro_rules! test {
            ($($t:ty)+) => { $({
                let values = [
                    0,
                    <$t>::min_value(),
                    <$t>::max_value(),
                ];
                for &value in &values {
                    let int = Integer::from(value);
                    assert_eq!(int.to_s(), value.to_string());
                }
            })+ }
        }

        crate::protected(|| {
            test! {
                usize u128 u64 u32 u16 u8
                isize i128 i64 i32 i16 i8
            }
        }).unwrap();
    }
}
