use ruby::VALUE;
use std::{
    fmt,
    ffi::{CStr, CString},
};

mod ty;
pub mod array;
pub mod exception;
pub mod hash;
pub mod instr_seq;
pub mod mixin;
pub mod string;
pub mod symbol;

#[doc(inline)]
pub use self::{
    array::Array,
    exception::{Exception, AnyException},
    hash::Hash,
    instr_seq::InstrSeq,
    mixin::{Mixin, Class, Module},
    string::String,
    symbol::{Symbol, SymbolId},
    ty::Ty,
};

/// Some concrete Ruby object.
///
/// # Safety
///
/// All types that implement this trait _must_ be light wrappers around an
/// [`AnyObject`](struct.AnyObject.html) and thus have the same size and layout.
pub unsafe trait Object: Copy + Into<AnyObject> + AsRef<AnyObject> {
    /// Attempts to create an instance by casting `obj`.
    #[inline]
    #[allow(unused)]
    fn cast(obj: impl Object) -> Option<Self>;

    /// Casts `obj` to `Self` without checking its type.
    #[inline]
    unsafe fn cast_unchecked(obj: impl Object) -> Self {
        let mut result = std::mem::uninitialized::<Self>();
        std::ptr::write((&mut result) as *mut Self as *mut _, obj);
        result
    }

    /// Returns `self` as an `AnyObject`.
    #[inline]
    fn into_any_object(self) -> AnyObject { self.into() }

    /// Returns a reference to `self` as an `AnyObject`.
    #[inline]
    fn as_any_object(&self) -> &AnyObject { self.as_ref() }

    /// Returns `self` as a reference to a single-element slice.
    #[inline]
    fn as_any_slice(&self) -> &[AnyObject] {
        std::slice::from_ref(self.as_any_object())
    }

    /// Returns the raw object pointer.
    fn raw(self) -> VALUE {
        self.as_any_object().raw()
    }

    /// Casts `self` to `O` without checking whether it is one.
    unsafe fn as_unchecked<O: Object>(&self) -> &O {
        &*(self as *const _ as *const _)
    }

    /// Converts `self` to `O` without checking whether it is one.
    unsafe fn into_unchecked<O: Object>(self) -> O {
        *self.as_unchecked()
    }

    /// Returns the object's identifier.
    #[inline]
    fn id(self) -> u64 {
        unsafe { ruby::rb_obj_id(self.raw()) as _ }
    }

    /// Returns the virtual type of `self`.
    #[inline]
    fn ty(self) -> Ty {
        self.as_any_object().ty()
    }

    /// Returns whether the virtual type of `self` is `ty`.
    #[inline]
    fn is_ty(self, ty: Ty) -> bool {
        crate::util::value_is_ty(self.raw(), ty)
    }

    /// Returns the `Class` for `self`.
    #[inline]
    fn class(self) -> Class {
        unsafe { Class::_new(ruby::rb_obj_class(self.raw())) }
    }

    /// Returns the singleton `Class` of `self`, creating one if it doesn't
    /// exist already.
    #[inline]
    fn singleton_class(self) -> Class {
        unsafe { Class::_new(ruby::rb_singleton_class(self.raw())) }
    }

    /// Calls `method` on `self` and returns the result.
    ///
    /// # Safety
    ///
    /// An exception will be raised if `method` is not defined on `self`.
    #[inline]
    unsafe fn call_unchecked(self, method: impl Into<SymbolId>) -> AnyObject {
        let args: &[AnyObject] = &[];
        self.call_with_unchecked(method, args)
    }

    /// Calls `method` on `self` and returns the result.
    #[inline]
    fn call(self, method: impl Into<SymbolId>) -> Result<AnyObject, AnyException> {
        crate::protected(|| unsafe { self.call_unchecked(method) })
    }

    /// Calls `method` on `self` with `args` and returns the result.
    ///
    /// # Safety
    ///
    /// An exception will be raised if `method` is not defined on `self`.
    #[inline]
    unsafe fn call_with_unchecked(
        self,
        method: impl Into<SymbolId>,
        args: &[impl Object]
    ) -> AnyObject {
        AnyObject(ruby::rb_funcallv(
            self.raw(),
            method.into().raw(),
            args.len() as _,
            args.as_ptr() as _,
        ))
    }

    /// Calls `method` on `self` with `args` and returns the result.
    #[inline]
    fn call_with(
        self,
        method: impl Into<SymbolId>,
        args: &[impl Object]
    ) -> Result<AnyObject, AnyException> {
        crate::protected(|| unsafe { self.call_with_unchecked(method, args) })
    }

    /// Calls the public `method` on `self` and returns the result.
    ///
    /// # Safety
    ///
    /// An exception will be raised if either `method` is not defined on `self`
    /// or `method` is not publicly callable.
    #[inline]
    unsafe fn call_public_unchecked(
        self,
        method: impl Into<SymbolId>,
    ) -> AnyObject {
        let args: &[AnyObject] = &[];
        self.call_public_with_unchecked(method, args)
    }

    /// Calls the public `method` on `self` and returns the result.
    #[inline]
    fn call_public(
        self,
        method: impl Into<SymbolId>,
    ) -> Result<AnyObject, AnyException> {
        crate::protected(|| unsafe { self.call_public_unchecked(method) })
    }

    /// Calls the public `method` on `self` with `args` and returns the result.
    ///
    /// # Safety
    ///
    /// An exception will be raised if either `method` is not defined on `self`
    /// or `method` is not publicly callable.
    #[inline]
    unsafe fn call_public_with_unchecked(
        self,
        method: impl Into<SymbolId>,
        args: &[impl Object]
    ) -> AnyObject {
        AnyObject(ruby::rb_funcallv_public(
            self.raw(),
            method.into().raw(),
            args.len() as _,
            args.as_ptr() as _,
        ))
    }

    /// Calls the public `method` on `self` with `args` and returns the result.
    #[inline]
    fn call_public_with(
        self,
        method: impl Into<SymbolId>,
        args: &[impl Object]
    ) -> Result<AnyObject, AnyException> {
        crate::protected(|| unsafe {
            self.call_public_with_unchecked(method, args)
        })
    }

    /// Returns a printable string representation of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::init().unwrap();
    /// use rosy::{Object, Class};
    ///
    /// let a = Class::array();
    /// assert_eq!(a.inspect(), a.call("inspect").unwrap());
    /// ```
    #[inline]
    fn inspect(self) -> String {
        unsafe { String::_new(ruby::rb_inspect(self.raw())) }
    }

    /// Returns the result of calling the `to_s` method on `self`.
    #[inline]
    fn to_s(self) -> String {
        unsafe { String::_new(ruby::rb_obj_as_string(self.raw())) }
    }

    /// Returns whether modifications can be made to `self`.
    #[inline]
    fn is_frozen(self) -> bool {
        unsafe { ruby::rb_obj_frozen_p(self.raw()) != 0 }
    }

    /// Freezes `self`, preventing any further mutations.
    #[inline]
    fn freeze(self) {
        unsafe { ruby::rb_obj_freeze(self.raw()) };
    }

    /// Returns whether `self` is equal to `other` in terms of the `eql?`
    /// method.
    #[inline]
    fn is_eql<O: Object>(self, other: &O) -> bool {
        let this = self.raw();
        let that = other.raw();
        unsafe { ruby::rb_eql(this, that) != 0 }
    }
}

/// Any Ruby object.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct AnyObject(ruby::VALUE);

impl AsRef<AnyObject> for AnyObject {
    #[inline]
    fn as_ref(&self) -> &Self { self }
}

unsafe impl Object for AnyObject {
    #[inline]
    fn cast(obj: impl Object) -> Option<Self> {
        Some(obj.into_any_object())
    }

    fn ty(self) -> ty::Ty {
        crate::util::value_type(self.raw()).into()
    }

    #[inline]
    fn raw(self) -> VALUE {
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

        impl PartialEq<AnyObject> for $t {
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

    /// Returns a `nil` instance.
    #[inline]
    pub fn nil() -> AnyObject {
        AnyObject(crate::util::NIL_VALUE)
    }

    /// Creates a new object from `raw` without checking whether it came from
    /// the Ruby VM.
    #[inline]
    pub unsafe fn from_raw(raw: VALUE) -> Self {
        AnyObject(raw)
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
        if self.is_string() { Some(String::_new(self.raw())) } else { None }
    }

    /// Returns whether `self` is a `Symbol`.
    #[inline]
    pub fn is_symbol(self) -> bool {
        crate::util::value_is_sym(self.raw())
    }

    /// Returns `self` as a `Symbol` if it is one.
    #[inline]
    pub fn to_symbol(self) -> Option<Symbol> {
        if self.is_symbol() { Some(Symbol::_new(self.raw())) } else { None }
    }

    /// Returns whether `self` is a `Array`.
    #[inline]
    pub fn is_array(self) -> bool {
        crate::util::value_is_built_in_ty(self.raw(), Ty::Array)
    }

    /// Returns `self` as an `Array` if it is one.
    #[inline]
    pub fn to_array(self) -> Option<Array> {
        if self.is_array() { Some(Array::_new(self.raw())) } else { None }
    }

    /// Returns whether `self` is a `Class`.
    #[inline]
    pub fn is_class(self) -> bool {
        crate::util::value_is_class(self.raw())
    }

    /// Returns `self` as a `Class` if it is one.
    #[inline]
    pub fn to_class(self) -> Option<Class> {
        if self.is_class() { Some(Class::_new(self.raw())) } else { None }
    }

    /// Returns whether `self` is a `Module`.
    #[inline]
    pub fn is_module(self) -> bool {
        crate::util::value_is_module(self.raw())
    }

    /// Returns `self` as a `Module` if it is one.
    #[inline]
    pub fn to_module(self) -> Option<Module> {
        if self.is_module() { Some(Module::_new(self.raw())) } else { None }
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
            Some(AnyException::_new(self.raw()))
        } else {
            None
        }
    }
}
