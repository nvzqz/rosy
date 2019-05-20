//! General functionality over Ruby objects.

use std::fmt;
use crate::{
    prelude::*,
    ruby,
    mixin::MethodFn,
};

mod any;
mod non_null;
mod rosy;
mod ty;

pub(crate) use non_null::NonNullObject;

#[doc(inline)]
pub use self::{
    any::AnyObject,
    rosy::RosyObject,
    ty::Ty,
};

/// Some concrete Ruby object.
///
/// # Safety
///
/// All types that implement this trait _must_ be light wrappers around an
/// [`AnyObject`](struct.AnyObject.html) and thus have the same size and layout.
pub unsafe trait Object: Copy
    + Into<AnyObject>
    + AsRef<AnyObject>
    + PartialEq<AnyObject>
    + fmt::Debug
{
    /// Returns a unique identifier for an object type to facilitate casting.
    ///
    /// # Safety
    ///
    /// This value _must_ be unique. Rosy's built-in objects use identifiers
    /// that are very close to `u128::max_value()`, so those are easy to avoid.
    #[inline]
    fn unique_id() -> Option<u128> {
        None
    }

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
    // TODO: Make a `const fn` once it stabilizes on trait items
    #[inline]
    unsafe fn from_raw(raw: ruby::VALUE) -> Self {
        Self::cast_unchecked(AnyObject::from_raw(raw))
    }

    /// Attempts to create an instance by casting `obj`.
    ///
    /// The default implementation checks the [`unique_id`](#method.unique_id)
    /// of `A` against that of `Self`.
    #[inline]
    fn cast<A: Object>(obj: A) -> Option<Self> {
        if A::unique_id() == Self::unique_id() {
            unsafe { Some(Self::cast_unchecked(obj)) }
        } else {
            None
        }
    }

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
    #[inline]
    fn raw(self) -> ruby::VALUE {
        self.as_any_object().raw()
    }

    /// Casts `self` to `O` without checking whether it is one.
    #[inline]
    unsafe fn as_unchecked<O: Object>(&self) -> &O {
        &*(self as *const _ as *const _)
    }

    /// Converts `self` to `O` without checking whether it is one.
    #[inline]
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

    /// Marks `self` for Ruby to avoid garbage collecting it.
    #[inline]
    fn mark(self) {
        crate::gc::mark(self);
    }

    /// Forces the garbage collector to free the contents of `self`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` does not have ownership over any
    /// currently-referenced memory.
    #[inline]
    unsafe fn force_recycle(self) {
        crate::gc::force_recycle(self);
    }

    /// Defines a method for `name` on the singleton class of `self` that calls
    /// `f` when invoked.
    #[inline]
    fn def_singleton_method<N, F>(self, name: N, f: F) -> Result
    where
        N: Into<SymbolId>,
        F: MethodFn<Self>,
    {
        // TODO: Create `SingletonClass` type
        let class = unsafe {
            Class::<Self>::cast_unchecked(self.singleton_class())
        };
        class.def_method(name, f)
    }

    /// Defines a method for `name` on the singleton class of `self` that calls
    /// `f` when invoked.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised.
    #[inline]
    unsafe fn def_singleton_method_unchecked<N, F>(self, name: N, f: F)
    where
        N: Into<SymbolId>,
        F: MethodFn<Self>,
    {
        // TODO: Create `SingletonClass` type
        let class = Class::<Self>::cast_unchecked(self.singleton_class());
        class.def_method_unchecked(name, f);
    }

    /// Calls `method` on `self` and returns the result.
    #[inline]
    fn call(self, method: impl Into<SymbolId>) -> Result<AnyObject> {
        let args: &[AnyObject] = &[];
        self.call_with(method, args)
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

    /// Calls `method` on `self` with `args` and returns the result.
    #[inline]
    fn call_with(
        self,
        method: impl Into<SymbolId>,
        args: &[impl Object]
    ) -> Result<AnyObject> {
        // monomorphization
        fn call_with(object: AnyObject, method: SymbolId, args: &[AnyObject]) -> Result<AnyObject> {
            unsafe { crate::protected_no_panic(|| {
                object.call_with_unchecked(method, args)
            }) }
        }
        call_with(self.into(), method.into(), AnyObject::convert_slice(args))
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

    /// Calls the public `method` on `self` and returns the result.
    #[inline]
    fn call_public(
        self,
        method: impl Into<SymbolId>,
    ) -> Result<AnyObject> {
        let args: &[AnyObject] = &[];
        self.call_public_with(method, args)
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

    /// Calls the public `method` on `self` with `args` and returns the result.
    #[inline]
    fn call_public_with(
        self,
        method: impl Into<SymbolId>,
        args: &[impl Object]
    ) -> Result<AnyObject> {
        // monomorphization
        fn call_public_with(object: AnyObject, method: SymbolId, args: &[AnyObject]) -> Result<AnyObject> {
            unsafe { crate::protected_no_panic(|| {
                object.call_public_with_unchecked(method, args)
            }) }
        }
        let args = AnyObject::convert_slice(args);
        call_public_with(self.into(), method.into(), args)
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

    /// Returns a printable string representation of `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn array_unique_id() {
        let expected = !(Ty::Array as u128);
        let array_id = Array::<AnyObject>::unique_id().unwrap();
        assert_eq!(array_id, expected);
    }

    #[test]
    fn hash_unique_id() {
        let expected = !(Ty::Hash as u128);
        let hash_id = Hash::<AnyObject, AnyObject>::unique_id().unwrap();
        assert_eq!(hash_id, expected);
    }

    #[test]
    fn unique_ids() {
        // Takes a sequence of `ty` and transforms it into an NxN sequence where
        // the work is done in the `single` branch over each `Hash` combination
        macro_rules! nxn_hash_ids {
            ($ids:expr => $($t:ty),*) => {
                nxn_hash_ids! { do_stuff $ids => $($t),* ; $($t),* }
            };
            (do_stuff $ids:expr => $t1next:ty ; $($t2s:ty),*) => {
                nxn_hash_ids! { expand $ids => $t1next, $($t2s),* }
            };
            (do_stuff $ids:expr => $t1next:ty, $($t1rest:ty),+ ; $($t2s:ty),*) => {
                nxn_hash_ids! { expand   $ids => $t1next, $($t2s),* }
                nxn_hash_ids! { do_stuff $ids => $($t1rest),* ; $($t2s),* }
            };
            (expand $ids:expr => $t1:ty, $($t2s:ty),*) => {
                $(nxn_hash_ids! { single $ids => $t1, $t2s })*
            };
            (single $ids:expr => $t1:ty, $t2:ty) => {
                $ids.push((stringify!(Hash<$t1, $t2>), Hash::<$t1, $t2>::unique_id()));
            };
        }
        macro_rules! ids {
            ($($t:ty,)+) => { {
                let mut ids = vec![
                    $(
                        (stringify!(<$t>),              <$t>::unique_id()),
                        (stringify!(Array<$t>),         Array::<$t>::unique_id()),
                        (stringify!(Array<Array<$t>>),  Array::<Array<$t>>::unique_id()),
                    )+
                ];
                nxn_hash_ids!(ids => $($t),+);
                ids
            } }
        }
        let ids: &[(&str, _)] = &ids! {
            AnyException,
            AnyObject,
            Float,
            Integer,
            String,
            Symbol,
            crate::vm::InstrSeq,
        };
        for a in ids {
            for b in ids {
                if std::ptr::eq(a, b) {
                    continue;
                }
                match (a, b) {
                    ((ty_a, Some(a)), (ty_b, Some(b))) => {
                        assert_ne!(a, b, "{} and {} have same ID", ty_a, ty_b);
                    },
                    (_, _) => {},
                }
            }
        }
    }
}
