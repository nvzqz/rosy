use std::{
    fmt,
    ffi::{c_void, CStr, CString},
    marker::PhantomData,
};
use crate::{
    object::Ty,
    prelude::*,
    ruby,
};

/// An instance of any Ruby object.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct AnyObject {
    raw: ruby::VALUE,
    // !Send + !Sync
    _marker: PhantomData<*const c_void>,
}

impl AsRef<AnyObject> for AnyObject {
    #[inline]
    fn as_ref(&self) -> &Self { self }
}

unsafe impl Object for AnyObject {
    #[inline]
    fn unique_id() -> Option<u128> {
        Some(!0)
    }

    #[inline]
    unsafe fn from_raw(raw: ruby::VALUE) -> Self {
        AnyObject { raw, _marker: PhantomData }
    }

    #[inline]
    fn cast<A: Object>(obj: A) -> Option<Self> {
        Some(obj.into_any_object())
    }

    fn ty(self) -> Ty {
        crate::util::value_type(self.raw()).into()
    }

    #[inline]
    fn raw(self) -> ruby::VALUE {
        self.raw
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

impl<O: Into<AnyObject>> From<Option<O>> for AnyObject {
    #[inline]
    fn from(option: Option<O>) -> Self {
        option.map(Into::into).unwrap_or(AnyObject::nil())
    }
}

impl<O: Into<AnyObject>, E: Into<AnyObject>> From<Result<O, E>> for AnyObject {
    #[inline]
    fn from(result: Result<O, E>) -> Self {
        match result {
            Ok(obj) => obj.into(),
            Err(err) => err.into(),
        }
    }
}

impl From<usize> for AnyObject {
    #[inline]
    fn from(int: usize) -> Self {
        unsafe { Self::from_raw(ruby::rb_uint2inum(int)) }
    }
}

impl From<u32> for AnyObject {
    #[inline]
    fn from(int: u32) -> Self {
        (int as usize).into()
    }
}

impl From<u16> for AnyObject {
    #[inline]
    fn from(int: u16) -> Self {
        (int as usize).into()
    }
}

impl From<u8> for AnyObject {
    #[inline]
    fn from(int: u8) -> Self {
        (int as usize).into()
    }
}

impl From<isize> for AnyObject {
    #[inline]
    fn from(int: isize) -> Self {
        unsafe { Self::from_raw(ruby::rb_int2inum(int)) }
    }
}

impl From<i32> for AnyObject {
    #[inline]
    fn from(int: i32) -> Self {
        (int as isize).into()
    }
}

impl From<i16> for AnyObject {
    #[inline]
    fn from(int: i16) -> Self {
        (int as isize).into()
    }
}

impl From<i8> for AnyObject {
    #[inline]
    fn from(int: i8) -> Self {
        (int as isize).into()
    }
}

impl From<f32> for AnyObject {
    #[inline]
    fn from(f: f32) -> Self {
        (f as f64).into()
    }
}

impl From<f64> for AnyObject {
    #[inline]
    fn from(f: f64) -> Self {
        unsafe { AnyObject::from_raw(ruby::rb_float_new(f)) }
    }
}

impl From<bool> for AnyObject {
    #[inline]
    fn from(b: bool) -> Self {
        Self::from_bool(b)
    }
}

impl AnyObject {
    #[inline]
    pub(crate) fn _ptr(self) -> *mut std::ffi::c_void {
        self.raw() as usize as _
    }

    /// Casts the concrete slice `objects` into a slice of `AnyObject`.
    #[inline]
    pub fn convert_slice(objects: &[impl Object]) -> &[AnyObject] {
        unsafe { &*(objects as *const [_] as *const _) }
    }

    /// Calls `super` on the current receiver without any arguments in the
    /// context of a method.
    #[inline]
    pub fn call_super() -> Result<AnyObject> {
        let args: &[AnyObject] = &[];
        Self::call_super_with(args)
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
    pub fn call_super_with(args: &[impl Object]) -> Result<AnyObject> {
        Self::_call_super_with(Self::convert_slice(args))
    }

    // monomorphization
    fn _call_super_with(args: &[AnyObject]) -> Result<AnyObject> {
        unsafe {
            crate::protected_no_panic(|| Self::call_super_with_unchecked(args))
        }
    }

    /// Calls `super` on the current receiver with `args` in the context of a
    /// method, without checking for an exception.
    #[inline]
    pub unsafe fn call_super_with_unchecked(args: &[impl Object]) -> AnyObject {
        let len = args.len();
        let ptr = args.as_ptr() as *const ruby::VALUE;
        AnyObject::from_raw(ruby::rb_call_super(len as _, ptr))
    }

    /// An alternative to
    /// [`Object::from_raw`](trait.Object.html#method.from_raw) that works in a
    /// `const` context.
    #[inline]
    pub const unsafe fn from_raw(raw: ruby::VALUE) -> AnyObject {
        AnyObject { raw, _marker: PhantomData }
    }

    /// Returns a `nil` instance.
    #[inline]
    pub const fn nil() -> AnyObject {
        unsafe { AnyObject::from_raw(crate::util::NIL_VALUE) }
    }

    /// Returns an instance from a boolean.
    #[inline]
    pub const fn from_bool(b: bool) -> AnyObject {
        // `false` uses 0 in Ruby
        let raw = crate::util::TRUE_VALUE * b as ruby::VALUE;
        unsafe { AnyObject::from_raw(raw) }
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

    /// Returns whether `self` is an `Array`.
    #[inline]
    pub fn is_array(self) -> bool {
        crate::util::value_is_built_in_ty(self.raw(), Ty::Array)
    }

    /// Returns `self` as an `Array` if it is one.
    ///
    /// # Safety
    ///
    /// If `self` refers to an `Array<X>` and after this method objects of type
    /// `Y` are inserted, expect
    /// [nasal demons](https://en.wikipedia.org/wiki/Nasal_demons). You've been
    /// warned.
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
