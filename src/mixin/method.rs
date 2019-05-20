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
pub unsafe trait MethodFn<Receiver: Object> {
    /// The type that the method returns.
    type Output: Object;

    /// The number of arguments taken by `self`.
    const ARITY: c_int;

    /// Returns the raw function pointer for `self`.
    fn raw_fn(self) -> unsafe extern "C" fn() -> VALUE;
}

/// Defines a method on a [`Class`] instance in a simple manner.
///
/// This is purely a convenience wrapper for [`def_method`] that makes the
/// process much less painful and tedious.
///
/// # Examples
///
/// This macro skips all of the necessary type shenanigans when calling the
/// method on [`Class`]. The focus is instead placed where it should be: on the
/// method's definition.
///
/// ```rust,edition2018
/// # rosy::vm::init().unwrap();
/// # rosy::protected(|| {
/// use rosy::prelude::*;
///
/// let class = Class::of::<String>();
///
/// rosy::def_method!(class, "blank?", |this: String| {
///     this.is_whitespace()
/// }).unwrap();
///
/// let string = String::from(" \n\r\t");
/// let result = string.call("blank?");
///
/// assert_eq!(result.unwrap(), true);
/// # }).unwrap();
/// ```
///
/// All argument counts supported by [`def_method`] work here as well:
///
/// ```rust,edition2018
/// # rosy::vm::init().unwrap();
/// # rosy::protected(|| {
/// use rosy::prelude::*;
///
/// let class = Class::object();
///
/// rosy::def_method!(class, "eql_either?", |snap, crackle, pop| {
///     snap == crackle || snap == pop
/// }).unwrap();
///
/// let object = AnyObject::from("snap");
/// let result = object.call_with("eql_either?", &[AnyObject::nil(), object]);
///
/// assert_eq!(result.unwrap(), true);
/// # }).unwrap();
/// ```
///
/// The same types supported in [`def_method`] are supported here via explicit
/// type annotations:
///
/// ```rust,edition2018
/// # rosy::vm::init().unwrap();
/// # rosy::protected(|| {
/// use rosy::prelude::*;
///
/// let class = Class::of::<Array>();
///
/// rosy::def_method!(class, "plus_args", |this: Array, args: Array| {
///     this.plus(args)
/// }).unwrap();
///
/// let expected: &[i32] = &[0, 1, 2, 3, 4, 5, 6];
/// let array: Array<Integer> = (0..4).collect();
///
/// let value = array.call_with("plus_args", &[
///     Integer::from(4),
///     Integer::from(5),
///     Integer::from(6),
/// ]).unwrap().to_array().unwrap();
///
/// assert_eq!(value, *expected);
/// # }).unwrap();
/// ```
///
/// [`Class`]: struct.Class.html
/// [`def_method`]: struct.Class.html#method.def_method
#[macro_export]
macro_rules! def_method {
    (
        $class:expr,
        $name:expr,
        |
                $this:ident $(: $this_ty:ty)?
            $(, $args:ident $(: $args_ty:ty)?)*
            $(,)?
        |
        $body:expr
    ) => { {
        type __AnyObject = $crate::AnyObject;
        type __Class = $crate::Class;

        macro_rules! _replace {
            ($__t:tt $sub:tt) => { $sub }
        }
        macro_rules! _substitute_any_object {
            () => { __AnyObject };
            ($__t:ty) => { $__t };
        }
        macro_rules! _cast_class {
            ($c:expr,) => { __Class::into_any_class($c) };
            ($c:expr, $_t:ty) => { $c };
        }

        extern "C" fn _method(
               $this : _substitute_any_object!($($this_ty)?),
            $( $args : _substitute_any_object!($($args_ty)?) ),*
        ) -> AnyObject { $body.into() }

        let _method: extern "C" fn(_, $( _replace!($args _) ),*) -> _ = _method;

        let _class = _cast_class!($class, $($this_ty)?);
        $crate::Class::def_method(_class, $name, _method)
    } };
}

/// Defines a method on a [`Class`](struct.Class.html) instance in a simple
/// manner, without checking for exceptions.
///
/// This is purely a convenience wrapper for
/// [`def_method_unchecked`](struct.Class.html#method.def_method_unchecked) that
/// makes the process much less painful and tedious.
///
/// See [`def_method!`](macro.def_method.html) for usage info.
///
/// # Safety
///
/// The caller must ensure that `self` is not frozen or else a `FrozenError`
/// exception will be raised.
#[macro_export]
macro_rules! def_method_unchecked {
    (
        $class:expr,
        $name:expr,
        |
                $this:ident $(: $this_ty:ty)?
            $(, $args:ident $(: $args_ty:ty)?)*
            $(,)?
        |
        $body:expr
    ) => { {
        type __AnyObject = $crate::AnyObject;
        type __Class = $crate::Class;

        macro_rules! _replace {
            ($__t:tt $sub:tt) => { $sub }
        }
        macro_rules! _substitute_any_object {
            () => { __AnyObject };
            ($__t:ty) => { $__t };
        }
        macro_rules! _cast_class {
            ($c:expr,) => { __Class::into_any_class($c) };
            ($c:expr, $_t:ty) => { $c };
        }

        extern "C" fn _method(
               $this : _substitute_any_object!($($this_ty)?),
            $( $args : _substitute_any_object!($($args_ty)?) ),*
        ) -> AnyObject { $body.into() }

        let _method: extern "C" fn(_, $( _replace!($args _) ),*) -> _ = _method;

        let _class = _cast_class!($class, $($this_ty)?);
        $crate::Class::def_method_unchecked(_class, $name, _method)
    } };
}

macro_rules! impl_trait {
    ($($a:expr $(,$args:ty)*;)+) => { $(
        impl_trait!(@fn $a, unsafe extern "C" fn(this: R $(,$args)*));
        impl_trait!(@fn $a,        extern "C" fn(this: R $(,$args)*));
    )+ };
    (@fn $a:expr, $($f:tt)+) => {
        unsafe impl<R: Object, O: Object> MethodFn<R> for $($f)+ -> O {
            type Output = O;

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
