use ruby::VALUE;

mod any;
mod non_null;
mod ty;
pub mod array;
pub mod exception;
pub mod hash;
pub mod instr_seq;
pub mod mixin;
pub mod string;
pub mod symbol;

use non_null::NonNullObject;

#[doc(inline)]
pub use self::{
    any::AnyObject,
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
    /// Creates a new object from `raw` without checking.
    ///
    /// # Safety
    ///
    /// The following invariants must be upheld:
    /// - The value came from the Ruby VM
    /// - The value is a valid representation of `Self`
    ///
    /// Not following this will lead to
    /// [nasal demons](https://en.wikipedia.org/wiki/Nasal_demons). You've been
    /// warned.
    #[inline]
    unsafe fn from_raw(raw: ruby::VALUE) -> Self {
        Self::cast_unchecked(AnyObject::from_raw(raw))
    }

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
        unsafe { Class::from_raw(ruby::rb_obj_class(self.raw())) }
    }

    /// Returns the singleton `Class` of `self`, creating one if it doesn't
    /// exist already.
    #[inline]
    fn singleton_class(self) -> Class {
        unsafe { Class::from_raw(ruby::rb_singleton_class(self.raw())) }
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
        AnyObject::from_raw(ruby::rb_funcallv(
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
        AnyObject::from_raw(ruby::rb_funcallv_public(
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
        unsafe { String::from_raw(ruby::rb_inspect(self.raw())) }
    }

    /// Returns the result of calling the `to_s` method on `self`.
    #[inline]
    fn to_s(self) -> String {
        unsafe { String::from_raw(ruby::rb_obj_as_string(self.raw())) }
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
