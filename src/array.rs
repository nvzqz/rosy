//! Ruby arrays.

use std::{
    cmp::Ordering,
    fmt,
    iter::FromIterator,
    ops::Add,
};
use crate::{
    object::{NonNullObject, Ty},
    prelude::*,
    ruby,
};

/// An instance of Ruby's `Array` class.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Array(NonNullObject);

impl AsRef<AnyObject> for Array {
    #[inline]
    fn as_ref(&self) -> &AnyObject { self.0.as_ref() }
}

impl From<Array> for AnyObject {
    #[inline]
    fn from(object: Array) -> AnyObject { object.0.into() }
}

impl PartialEq<AnyObject> for Array {
    #[inline]
    fn eq(&self, obj: &AnyObject) -> bool {
        self.as_any_object() == obj
    }
}

unsafe impl Object for Array {
    #[inline]
    fn cast(obj: impl Object) -> Option<Self> {
        if obj.is_ty(Ty::Array) {
            unsafe { Some(Self::from_raw(obj.raw())) }
        } else {
            None
        }
    }

    #[inline]
    fn ty(self) -> Ty { Ty::Array }

    #[inline]
    fn is_ty(self, ty: Ty) -> bool { ty == Ty::Array }
}

impl fmt::Display for Array {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_any_object().fmt(f)
    }
}

// Safe because this is part of the contract of implementing `Object`.
impl<'r, O: Object> From<&[O]> for Array {
    #[inline]
    fn from(slice: &[O]) -> Self {
        let ptr = slice.as_ptr() as *const ruby::VALUE;
        let len = slice.len();
        unsafe { Array::from_raw(ruby::rb_ary_new_from_values(len as _, ptr)) }
    }
}

impl PartialEq for Array {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

impl PartialOrd for Array {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let value = unsafe { ruby::rb_ary_cmp(self.raw(), other.raw()) };
        if value == crate::util::NIL_VALUE {
            return None;
        }
        Some(crate::util::value_to_fixnum(value).cmp(&0))
    }
}

impl<'r, O: Object> FromIterator<O> for Array {
    #[inline]
    fn from_iter<T: IntoIterator<Item=O>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let (size, _) = iter.size_hint();
        let array = Array::with_capacity(size);
        for obj in iter {
            unsafe { array.push(obj) };
        }
        array
    }
}

// Allows for `a1 + a2` in Rust
impl Add for Array {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self::Output {
        self.plus(other)
    }
}

impl Array {
    #[inline]
    pub(crate) fn rarray(self) -> *mut ruby::RArray {
        self.as_any_object()._ptr() as _
    }

    /// Creates a new empty instance.
    #[inline]
    pub fn new() -> Self {
        unsafe { Self::from_raw(ruby::rb_ary_new()) }
    }

    /// Creates a new instance from the elements in `slice`.
    #[inline]
    pub fn from_slice<'s, T>(slice: &'s [T]) -> Self
        where &'s [T]: Into<Self>
    {
        slice.into()
    }

    /// Creates a new instance with `capacity` amount of storage.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        unsafe { Self::from_raw(ruby::rb_ary_new_capa(capacity as _)) }
    }

    /// Returns the number of elements in `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::{Array, String};
    ///
    /// let s = String::from("hi");
    /// let a = Array::from_slice(&[s, s, s]);
    ///
    /// assert_eq!(a.len(), 3);
    /// ```
    #[inline]
    pub fn len(self) -> usize {
        unsafe { (*self.rarray()).len() }
    }

    /// Returns whether `self` is empty.
    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Returns a slice to the underlying objects of `self`.
    ///
    /// # Safety
    ///
    /// Care must be taken to ensure that the length of `self` is not changed
    /// through the VM or otherwise.
    #[inline]
    pub unsafe fn as_slice(&self) -> &[AnyObject] {
        let ptr = (*self.rarray()).start() as *const AnyObject;
        std::slice::from_raw_parts(ptr, self.len())
    }

    /// Returns a mutable slice to the underlying objects of `self`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised. Care must also be taken to ensure that the
    /// length of `self` is not changed through the VM or otherwise.
    #[inline]
    pub unsafe fn as_slice_mut(&mut self) -> &mut [AnyObject] {
        ruby::rb_ary_modify(self.raw());
        let ptr = (*self.rarray()).start_mut() as *mut AnyObject;
        std::slice::from_raw_parts_mut(ptr, self.len())
    }

    /// Returns the object at `index` or `None` if `index` is out-of-bounds.
    #[inline]
    pub fn get(self, index: usize) -> Option<AnyObject> {
        unsafe { self.as_slice().get(index).map(|&obj| obj) }
    }

    /// Returns the object at `index` without bounds checking.
    #[inline]
    pub unsafe fn get_unchecked(self, index: usize) -> AnyObject {
        *self.as_slice().get_unchecked(index)
    }

    /// Returns the first object in `self`.
    #[inline]
    pub fn first(self) -> Option<AnyObject> {
        unsafe { self.as_slice().first().map(|&obj| obj) }
    }

    /// Returns the last element in `self`.
    #[inline]
    pub fn last(self) -> Option<AnyObject> {
        unsafe { self.as_slice().last().map(|&obj| obj) }
    }

    /// Removes all elements from `self`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised.
    #[inline]
    pub unsafe fn clear(self) {
        ruby::rb_ary_clear(self.raw())
    }

    /// Appends all of the elements in `slice` to `self`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised.
    #[inline]
    pub unsafe fn extend_from_slice(self, slice: &[impl Object]) {
        let ptr = slice.as_ptr() as *const ruby::VALUE;
        let len = slice.len();
        ruby::rb_ary_cat(self.raw(), ptr, len as _);
    }

    /// Returns the result of performing `self + other`.
    #[inline]
    pub fn plus(self, other: Self) -> Self {
        unsafe { Array::from_raw(ruby::rb_ary_plus(self.raw(), other.raw())) }
    }

    /// Pushes `obj` onto the end of `self`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised.
    #[inline]
    pub unsafe fn push(self, obj: impl Object) -> AnyObject {
        AnyObject::from_raw(ruby::rb_ary_push(self.raw(), obj.raw()))
    }

    /// Pops the last element from `self`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::{Array, String};
    ///
    /// let s = String::from("Hi");
    /// let a = Array::from_slice(&[s]);
    ///
    /// unsafe {
    ///     assert!(!a.pop().is_nil());
    ///     assert!(a.pop().is_nil());
    /// }
    /// ```
    #[inline]
    pub unsafe fn pop(self) -> AnyObject {
        AnyObject::from_raw(ruby::rb_ary_pop(self.raw()))
    }

    /// Returns whether `self` contains `obj`.
    ///
    /// This is equivalent to the `include?` method.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::{Array, String};
    ///
    /// let s = String::from("hi");
    /// let a = Array::from_slice(&[String::from("yo"), s]);
    ///
    /// assert!(a.contains(s));
    /// ```
    #[inline]
    pub fn contains(self, obj: impl Object) -> bool {
        let val = unsafe { ruby::rb_ary_includes(self.raw(), obj.raw()) };
        val == crate::util::TRUE_VALUE
    }

    /// Removes _all_ items in `self` that are equal to `obj`.
    ///
    /// This is equivalent to the `delete` method.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised.
    #[inline]
    pub unsafe fn remove_all(self, obj: impl Object) -> AnyObject {
        AnyObject::from_raw(ruby::rb_ary_delete(
            self.raw(),
            obj.raw(),
        ))
    }

    /// Reverses the contents of `self` in-palace.
    ///
    /// This is equivalent to the `reverse!` method.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised.
    #[inline]
    pub unsafe fn reverse(self) {
        ruby::rb_ary_reverse(self.raw());
    }

    /// Returns an instance with its contents sorted.
    #[inline]
    #[must_use]
    pub fn sorted(self) -> Self {
        unsafe { Array::from_raw(ruby::rb_ary_sort(self.raw())) }
    }

    /// Sorts the contents of `self` in-place without checking whether `self` is
    /// frozen.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised.
    #[inline]
    pub unsafe fn sort(self) {
        ruby::rb_ary_sort_bang(self.raw());
    }

    /// Joins the contents of `self` with `separator`.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::{Array, String};
    ///
    /// let s = String::from("-");
    /// let a = Array::from_slice(&[s, s, s]);
    ///
    /// assert_eq!(a.join("."), "-.-.-");
    /// ```
    #[inline]
    pub fn join(self, separator: impl Into<String>) -> String {
        let separator = separator.into().raw();
        unsafe { String::from_raw(ruby::rb_ary_join(self.raw(), separator)) }
    }
}
