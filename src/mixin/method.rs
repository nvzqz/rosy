use std::{
    os::raw::c_int,
    mem,
};
use crate::{
    prelude::*,
    ruby::VALUE,
};

/// An `extern "C" fn` that can be used as a method in
/// [`Class::def_method`](struct.Class.html#method.def_method).
pub unsafe trait MethodFn {
    /// The number of arguments taken by `self`.
    const ARITY: c_int;

    /// Returns the raw function pointer for `self`.
    fn raw_fn(self) -> unsafe extern "C" fn() -> VALUE;
}

macro_rules! impl_trait {
    ($($a:expr $(,$args:ty)*;)+) => { $(
        impl_trait!(@fn $a, unsafe extern "C" fn(this: AnyObject $(,$args)*) -> AnyObject);
        impl_trait!(@fn $a,        extern "C" fn(this: AnyObject $(,$args)*) -> AnyObject);
    )+ };
    (@fn $a:expr, $f:ty) => {
        unsafe impl MethodFn for $f {
            const ARITY: c_int = $a;

            #[inline]
            fn raw_fn(self) -> unsafe extern "C" fn() -> VALUE {
                unsafe { mem::transmute(self) }
            }
        }
    };
}

impl_trait! {
    -2, Array;
    -1, c_int, *const AnyObject;
}

macro_rules! replace {
    ($_t:tt $sub:tt) => { $sub };
}

macro_rules! count {
    ($($t:tt)*) => { 0 $(+ replace!($t 1))* };
}

// This macro lets us create an implementation of `MethodFn` on a pair of
// `extern "C" fn` pairs (one being `unsafe`) for each comma token
macro_rules! impl_trait_many {
    () => {
        impl_trait! { 0; }
    };
    (, $($t:tt)*) => {
        impl_trait_many!($($t)*);
        impl_trait! { 1 + count!($($t)*), AnyObject $(, replace!($t AnyObject))* ; }
    };
}

// 15 is the maximum arity allowed
impl_trait_many!(,,,,, ,,,,, ,,,,,);
