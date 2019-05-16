use std::{
    fmt,
    ffi::{CStr, CString},
};
use crate::{
    object::Ty,
    prelude::*,
    ruby,
};

/// An instance of any Ruby object.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct AnyObject(ruby::VALUE);

impl AsRef<AnyObject> for AnyObject {
    #[inline]
    fn as_ref(&self) -> &Self { self }
}

unsafe impl Object for AnyObject {
    #[inline]
    unsafe fn from_raw(raw: ruby::VALUE) -> Self { Self(raw) }

    #[inline]
    fn cast(obj: impl Object) -> Option<Self> {
        Some(obj.into_any_object())
    }

    fn ty(self) -> Ty {
        crate::util::value_type(self.raw()).into()
    }

    #[inline]
    fn raw(self) -> ruby::VALUE {
        self.0
    }

    #[inline]
    fn as_any_object(&self) -> &Self { &self }

    #[inline]
    fn into_any_object(self) -> Self { self }
}

impl<O: Object> PartialEq<O> for AnyObject {
    #[inline]
    fn eq(&self, other: &O) -> bool {
        let result = unsafe { self.call_with_unchecked("==", &[*other]) };
        result.raw() == crate::util::TRUE_VALUE
    }
}

// Implements `PartialEq` against all relevant convertible types
macro_rules! impl_eq {
    ($($t:ty, $convert:ident;)+) => { $(
        impl PartialEq<$t> for AnyObject {
            #[inline]
            fn eq(&self, other: &$t) -> bool {
                if let Some(value) = AnyObject::$convert(*self) {
                    value == *other
                } else {
                    false
                }
            }
        }

        // Needed to prevent conflict with `PartialEq<impl Object>`
        impl PartialEq<&$t> for AnyObject {
            #[inline]
            fn eq(&self, other: &&$t) -> bool {
                *self == **other
            }
        }

        impl PartialEq<AnyObject> for $t {
            #[inline]
            fn eq(&self, obj: &AnyObject) -> bool {
                obj == self
            }
        }

        impl PartialEq<AnyObject> for &$t {
            #[inline]
            fn eq(&self, obj: &AnyObject) -> bool {
                obj == self
            }
        }
    )+ }
}

impl_eq! {
    [u8],                   to_string;
    Vec<u8>,                to_string;
    str,                    to_string;
    std::string::String,    to_string;
    CStr,                   to_string;
    CString,                to_string;
    bool,                   to_bool;
}

impl Eq for AnyObject {}

impl fmt::Debug for AnyObject {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.inspect(), f)
    }
}

impl fmt::Display for AnyObject {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.to_s(), f)
    }
}

impl<O: Object> From<Option<O>> for AnyObject {
    #[inline]
    fn from(option: Option<O>) -> Self {
        option.map(Object::into_any_object).unwrap_or(AnyObject::nil())
    }
}

impl<O: Object, E: Object> From<Result<O, E>> for AnyObject {
    #[inline]
    fn from(result: Result<O, E>) -> Self {
        match result {
            Ok(obj) => obj.into_any_object(),
            Err(err) => err.into_any_object(),
        }
    }
}

impl From<bool> for AnyObject {
    #[inline]
    fn from(b: bool) -> Self {
        use crate::util::*;
        AnyObject(if b { TRUE_VALUE } else { FALSE_VALUE })
    }
}

impl From<&str> for AnyObject {
    #[inline]
    fn from(s: &str) -> Self {
        String::from(s).into()
    }
}

impl From<&CStr> for AnyObject {
    #[inline]
    fn from(s: &CStr) -> Self {
        String::from(s).into()
    }
}

impl AnyObject {
    #[inline]
    pub(crate) fn _ptr(self) -> *mut std::ffi::c_void {
        self.raw() as usize as _
    }

    /// Calls `super` on the current receiver without any arguments in the
    /// context of a method.
    #[inline]
    pub fn call_super() -> Result<AnyObject, AnyException> {
        crate::protected(|| unsafe { Self::call_super_unchecked() })
    }

    /// Calls `super` on the current receiver without any arguments in the
    /// context of a method, without checking for an exception.
    #[inline]
    pub unsafe fn call_super_unchecked() -> AnyObject {
        let args: &[AnyObject] = &[];
        Self::call_super_with_unchecked(args)
    }

    /// Calls `super` on the current receiver with `args` in the context of a
    /// method.
    #[inline]
    pub fn call_super_with(args: &[impl Object]) -> Result<AnyObject, AnyException> {
        crate::protected(|| unsafe {  Self::call_super_with_unchecked(args) })
    }

    /// Calls `super` on the current receiver with `args` in the context of a
    /// method, without checking for an exception.
    #[inline]
    pub unsafe fn call_super_with_unchecked(args: &[impl Object]) -> AnyObject {
        let len = args.len();
        let ptr = args.as_ptr() as *const ruby::VALUE;
        AnyObject::from_raw(ruby::rb_call_super(len as _, ptr))
    }

    /// Returns a `nil` instance.
    #[inline]
    pub const fn nil() -> AnyObject {
        AnyObject(crate::util::NIL_VALUE)
    }

    /// Returns an instance from a boolean.
    #[inline]
    pub const fn from_bool(b: bool) -> AnyObject {
        // `false` uses 0 in Ruby
        AnyObject(crate::util::TRUE_VALUE * b as ruby::VALUE)
    }

    /// Returns whether `self` is `nil`.
    #[inline]
    pub fn is_nil(self) -> bool {
        self.raw() == crate::util::NIL_VALUE
    }

    /// Returns whether `self` is undefined.
    #[inline]
    pub fn is_undefined(self) -> bool {
        self.raw() == crate::util::UNDEF_VALUE
    }

    /// Returns whether `self` is `true`.
    #[inline]
    pub fn is_true(self) -> bool {
        self.raw() == crate::util::TRUE_VALUE
    }

    /// Returns whether `self` is `false`.
    #[inline]
    pub fn is_false(self) -> bool {
        self.raw() == crate::util::FALSE_VALUE
    }

    /// Returns the boolean value for `self`, if any.
    #[inline]
    pub fn to_bool(self) -> Option<bool> {
        match self.raw() {
            crate::util::TRUE_VALUE => Some(true),
            crate::util::FALSE_VALUE => Some(false),
            _ => None,
        }
    }

    /// Returns whether `self` is a fixed-sized number.
    #[inline]
    pub fn is_fixnum(self) -> bool {
        crate::util::value_is_fixnum(self.raw())
    }

    /// Returns whether `self` is a floating point number type.
    #[inline]
    pub fn is_float(self) -> bool {
        crate::util::value_is_float(self.raw())
    }

    /// Returns whether `self` is a `String`.
    #[inline]
    pub fn is_string(self) -> bool {
        crate::util::value_is_built_in_ty(self.raw(), Ty::String)
    }

    /// Returns `self` as a `String` if it is one.
    #[inline]
    pub fn to_string(self) -> Option<String> {
        if self.is_string() {
            unsafe { Some(String::cast_unchecked(self)) }
        } else {
            None
        }
    }

    /// Returns whether `self` is a `Symbol`.
    #[inline]
    pub fn is_symbol(self) -> bool {
        crate::util::value_is_sym(self.raw())
    }

    /// Returns `self` as a `Symbol` if it is one.
    #[inline]
    pub fn to_symbol(self) -> Option<Symbol> {
        if self.is_symbol() {
            unsafe { Some(Symbol::cast_unchecked(self)) }
        } else {
            None
        }
    }

    /// Returns whether `self` is a `Array`.
    #[inline]
    pub fn is_array(self) -> bool {
        crate::util::value_is_built_in_ty(self.raw(), Ty::Array)
    }

    /// Returns `self` as an `Array` if it is one.
    #[inline]
    pub fn to_array(self) -> Option<Array> {
        if self.is_array() {
            unsafe { Some(Array::cast_unchecked(self)) }
        } else {
            None
        }
    }

    /// Returns whether `self` is a `Class`.
    #[inline]
    pub fn is_class(self) -> bool {
        crate::util::value_is_class(self.raw())
    }

    /// Returns `self` as a `Class` if it is one.
    #[inline]
    pub fn to_class(self) -> Option<Class> {
        if self.is_class() {
            unsafe { Some(Class::cast_unchecked(self)) }
        } else {
            None
        }
    }

    /// Returns whether `self` is a `Module`.
    #[inline]
    pub fn is_module(self) -> bool {
        crate::util::value_is_module(self.raw())
    }

    /// Returns `self` as a `Module` if it is one.
    #[inline]
    pub fn to_module(self) -> Option<Module> {
        if self.is_module() {
            unsafe { Some(Module::cast_unchecked(self)) }
        } else {
            None
        }
    }

    /// Returns whether `self` is an `Exception`.
    #[inline]
    pub fn is_exception(self) -> bool {
        self.class().inherits(Class::exception())
    }

    /// Returns `self` as an `AnyException` if it is one.
    #[inline]
    pub fn to_exception(self) -> Option<AnyException> {
        if self.is_exception() {
            unsafe { Some(AnyException::cast_unchecked(self)) }
        } else {
            None
        }
    }
}
