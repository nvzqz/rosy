use std::ffi::CStr;
use crate::{
    prelude::*,
    ruby::{self, VALUE},
};

/// Evaluates `script` in an isolated binding without handling exceptions.
///
/// Variables:
/// - `__FILE__`: "(eval)"
/// - `__LINE__`: starts at 1
///
/// # Safety
///
/// Code executed from `script` may void the type safety of objects accessible
/// from Rust. For example, if one calls `push` on `Array<A>` with an object of
/// type `B`, then the inserted object will be treated as being of type `A`.
///
/// ```
/// # rosy::vm::init().unwrap();
/// use std::ffi::CStr;
/// use rosy::prelude::*;
///
/// let array: Array<Integer> = (1..=3).collect(); // [1, 2, 3]
///
/// let module = Module::def("Evil").unwrap();
/// unsafe { module.set_const("ARR", array) };
///
/// let script = b"Evil::ARR.push('hi')\0";
/// let script = CStr::from_bytes_with_nul(script).unwrap();
///
/// unsafe { rosy::vm::eval(script) };
/// let hi = array.get(3).unwrap();
/// ```
///
/// If we try using `hi` as an `Integer` here, we will get a segmentation fault:
///
// Supports all failures, apparently
/// ```should_panic
// This is just for demonstration purposes
/// # rosy::vm::init().unwrap();
/// # use rosy::prelude::*;
/// # let hi = unsafe { Integer::cast_unchecked(String::from("hi")) };
/// let val = hi.to_truncated::<i32>();
/// ```
///
/// However, we can see that `hi` is actually a `String` despite being typed as
/// an `Integer`:
///
/// ```
// This is just for demonstration purposes
/// # rosy::vm::init().unwrap();
/// # use rosy::prelude::*;
/// # let hi = AnyObject::from("hi");
/// let hi = unsafe { String::cast_unchecked(hi) };
/// assert_eq!(hi, "hi");
/// ```
///
/// ...also, any exceptions raised in `script` must be handled in Rust-land.
#[inline]
pub unsafe fn eval(script: &CStr) -> AnyObject {
    AnyObject::from_raw(ruby::rb_eval_string(script.as_ptr()))
}

/// Evaluates `script` in an isolated binding, returning an exception if one is
/// raised.
///
/// Variables:
/// - `__FILE__`: "(eval)"
/// - `__LINE__`: starts at 1
///
/// # Safety
///
/// Code executed from `script` may void the type safety of objects accessible
/// from Rust. For example, if one calls `push` on `Array<A>` with an object of
/// type `B`, then the inserted object will be treated as being of type `A`.
#[inline]
pub unsafe fn eval_protected(script: &CStr) -> Result<AnyObject> {
    let mut err = 0;
    let raw = ruby::rb_eval_string_protect(script.as_ptr(), &mut err);
    match err {
        0 => Ok(AnyObject::from_raw(raw)),
        _ => Err(AnyException::_take_current()),
    }
}

/// Evaluates `script` under a module binding in an isolated binding, returning
/// an exception if one is raised.
///
/// Variables:
/// - `__FILE__`: "(eval)"
/// - `__LINE__`: starts at 1
///
/// # Safety
///
/// Code executed from `script` may void the type safety of objects accessible
/// from Rust. For example, if one calls `push` on `Array<A>` with an object of
/// type `B`, then the inserted object will be treated as being of type `A`.
#[inline]
pub unsafe fn eval_wrapped(script: &CStr) -> Result<AnyObject> {
    let mut err = 0;
    let raw = ruby::rb_eval_string_wrap(script.as_ptr(), &mut err);
    match err {
        0 => Ok(AnyObject::from_raw(raw)),
        _ => Err(AnyException::_take_current()),
    }
}

/// A type that can be used as one or more arguments for evaluating code within
/// the context of a [`Mixin`](trait.Mixin.html).
///
/// The difference between `eval_in_object` and `eval_in_mixin`
///
/// See the documentation of [its implementors](#foreign-impls) for much more
/// detailed information.
///
/// # Safety
///
/// Code executed from `self` may void the type safety of objects accessible
/// from Rust. For example, if one calls `push` on an `Array<A>` with an
/// object of type `B`, then the inserted object will be treated as being of
/// type `A`.
///
/// For non-`protected` variants, if an exception is raised due to an argument
/// error or from evaluating the script itself, it should be caught.
pub trait EvalArgs: Sized {
    /// Evaluates `self` in the context of `object`.
    ///
    /// This corresponds directly to `rb_obj_instance_eval`.
    ///
    /// In order to set the context, the variable `self` is set to `object`
    /// while the code is executing, giving the code access to `object`'s
    /// instance variables and private methods.
    unsafe fn eval_in_object(self, object: impl Into<AnyObject>) -> AnyObject;

    /// Evaluates `self` in the context of `object`, returning any raised
    /// exceptions.
    unsafe fn eval_in_object_protected(self, object: impl Into<AnyObject>) -> Result<AnyObject>;

    /// Evaluates `self` in the context of `mixin`.
    ///
    /// This corresponds directly to `rb_mod_module_eval`.
    unsafe fn eval_in_mixin(self, mixin: impl Mixin) -> AnyObject;

    /// Evaluates `self` in the context of `mixin`, returning any raised
    /// exceptions.
    unsafe fn eval_in_mixin_protected(self, mixin: impl Mixin) -> Result<AnyObject>;
}

/// Unchecked arguments directly to the evaluation function.
impl<O: Object> EvalArgs for &[O] {
    #[inline]
    unsafe fn eval_in_object_protected(self, object: impl Into<AnyObject>) -> Result<AnyObject> {
        // monomorphization
        unsafe fn eval(args: &[AnyObject], object: AnyObject) -> Result<AnyObject> {
            crate::protected_no_panic(|| args.eval_in_object(object))
        }
        eval(AnyObject::convert_slice(self), object.into())
    }

    #[inline]
    unsafe fn eval_in_object(self, object: impl Into<AnyObject>) -> AnyObject {
        let raw = ruby::rb_obj_instance_eval(
            self.len() as _,
            self.as_ptr() as *const VALUE,
            object.into().raw(),
        );
        AnyObject::from_raw(raw)
    }

    #[inline]
    unsafe fn eval_in_mixin_protected(self, mixin: impl Mixin) -> Result<AnyObject> {
        // monomorphization
        unsafe fn eval(args: &[AnyObject], mixin: VALUE) -> Result<AnyObject> {
            let raw = crate::protected_no_panic(|| ruby::rb_mod_module_eval(
                args.len() as _,
                args.as_ptr() as *const VALUE,
                mixin,
            ))?;
            Ok(AnyObject::from_raw(raw))
        }
        eval(AnyObject::convert_slice(self), mixin.raw())
    }

    #[inline]
    unsafe fn eval_in_mixin(self, mixin: impl Mixin) -> AnyObject {
        let raw = ruby::rb_mod_module_eval(
            self.len() as _,
            self.as_ptr() as *const VALUE,
            mixin.raw(),
        );
        AnyObject::from_raw(raw)
    }
}

/// The script argument without any extra information.
impl EvalArgs for String {
    #[inline]
    unsafe fn eval_in_object_protected(self, object: impl Into<AnyObject>) -> Result<AnyObject> {
        self.as_any_slice().eval_in_object_protected(object)
    }

    #[inline]
    unsafe fn eval_in_object(self, object: impl Into<AnyObject>) -> AnyObject {
        self.as_any_slice().eval_in_object(object)
    }

    #[inline]
    unsafe fn eval_in_mixin_protected(self, mixin: impl Mixin) -> Result<AnyObject> {
        self.as_any_slice().eval_in_mixin_protected(mixin)
    }

    #[inline]
    unsafe fn eval_in_mixin(self, mixin: impl Mixin) -> AnyObject {
        self.as_any_slice().eval_in_mixin(mixin)
    }
}

/// The script argument as a UTF-8 string, without any extra information.
// TODO: Impl for `Into<String>` when specialization stabilizes
impl EvalArgs for &str {
    #[inline]
    unsafe fn eval_in_object_protected(self, object: impl Into<AnyObject>) -> Result<AnyObject> {
        String::from(self).eval_in_object_protected(object)
    }

    #[inline]
    unsafe fn eval_in_object(self, object: impl Into<AnyObject>) -> AnyObject {
        String::from(self).eval_in_object(object)
    }

    #[inline]
    unsafe fn eval_in_mixin_protected(self, mixin: impl Mixin) -> Result<AnyObject> {
        String::from(self).eval_in_mixin_protected(mixin)
    }

    #[inline]
    unsafe fn eval_in_mixin(self, mixin: impl Mixin) -> AnyObject {
        String::from(self).eval_in_mixin(mixin)
    }
}

/// The script and filename arguments.
impl<S: Into<String>, F: Into<String>> EvalArgs for (S, F) {
    #[inline]
    unsafe fn eval_in_object_protected(self, object: impl Into<AnyObject>) -> Result<AnyObject> {
        let (s, f) = self;
        [s.into(), f.into()].eval_in_object_protected(object)
    }

    #[inline]
    unsafe fn eval_in_object(self, object: impl Into<AnyObject>) -> AnyObject {
        let (s, f) = self;
        [s.into(), f.into()].eval_in_object(object)
    }

    #[inline]
    unsafe fn eval_in_mixin_protected(self, mixin: impl Mixin) -> Result<AnyObject> {
        let (s, f) = self;
        [s.into(), f.into()].eval_in_mixin_protected(mixin)
    }

    #[inline]
    unsafe fn eval_in_mixin(self, mixin: impl Mixin) -> AnyObject {
        let (s, f) = self;
        [s.into(), f.into()].eval_in_mixin(mixin)
    }
}

/// The script, filename, and line number arguments.
impl<S, F, L> EvalArgs for (S, F, L)
where
    S: Into<String>,
    F: Into<String>,
    L: Into<Integer>,
{
    #[inline]
    unsafe fn eval_in_object_protected(self, object: impl Into<AnyObject>) -> Result<AnyObject> {
        let (s, f, l) = self;
        let s = AnyObject::from(s.into());
        let f = AnyObject::from(f.into());
        let l = AnyObject::from(l.into());
        [s, f, l].eval_in_object_protected(object)
    }

    #[inline]
    unsafe fn eval_in_object(self, object: impl Into<AnyObject>) -> AnyObject {
        let (s, f, l) = self;
        let s = AnyObject::from(s.into());
        let f = AnyObject::from(f.into());
        let l = AnyObject::from(l.into());
        [s, f, l].eval_in_object(object)
    }

    #[inline]
    unsafe fn eval_in_mixin_protected(self, mixin: impl Mixin) -> Result<AnyObject> {
        let (s, f, l) = self;
        let s = AnyObject::from(s.into());
        let f = AnyObject::from(f.into());
        let l = AnyObject::from(l.into());
        [s, f, l].eval_in_mixin_protected(mixin)
    }

    #[inline]
    unsafe fn eval_in_mixin(self, mixin: impl Mixin) -> AnyObject {
        let (s, f, l) = self;
        let s = AnyObject::from(s.into());
        let f = AnyObject::from(f.into());
        let l = AnyObject::from(l.into());
        [s, f, l].eval_in_mixin(mixin)
    }
}
