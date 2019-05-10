//! Ruby arrays.

use crate::object::{Object, AnyObject, Ty};
use std::{
    cmp::Ordering,
    fmt,
    iter::FromIterator,
    ops::Add,
};

/// An instance of Ruby's `Array` class.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Array(AnyObject);

unsafe impl Object for Array {
    #[inline]
    fn cast(obj: impl Object) -> Option<Self> {
        if obj.is_ty(Ty::Array) {
            Some(Self::_new(obj.raw()))
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
        self.as_any().fmt(f)
    }
}

impl AsRef<AnyObject> for Array {
    #[inline]
    fn as_ref(&self) -> &AnyObject { &self.0 }
}

impl From<Array> for AnyObject {
    #[inline]
    fn from(object: Array) -> AnyObject { object.0 }
}

// Safe because this is part of the contract of implementing `Object`.
impl<'r, O: Object> From<&[O]> for Array {
    #[inline]
    fn from(slice: &[O]) -> Self {
        let ptr = slice.as_ptr() as *const ruby::VALUE;
        let len = slice.len();
        unsafe { Array::_new(ruby::rb_ary_new_from_values(len as _, ptr)) }
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

impl<'r, O: Object> Extend<O> for Array {
    #[inline]
    fn extend<T: IntoIterator<Item=O>>(&mut self, iter: T) {
        for obj in iter {
            self.push(obj);
        }
    }
}

impl<'r, O: Object> FromIterator<O> for Array {
    #[inline]
    fn from_iter<T: IntoIterator<Item=O>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let (size, _) = iter.size_hint();
        let raw = unsafe { ruby::rb_ary_new_capa(size as _) };
        let mut array = Array::_new(raw);
        array.extend(iter);
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
    pub(crate) fn _new(raw: ruby::VALUE) -> Self {
        Self(AnyObject(raw))
    }

    #[inline]
    pub(crate) fn _ptr(self) -> *const ruby::VALUE {
        unsafe {
            if self._is_embedded() {
                (*self._rarray()).as_.ary.as_ptr()
            } else {
                (*self._rarray()).as_.heap.ptr
            }
        }
    }

    #[inline]
    pub(crate) fn _rarray(self) -> *mut ruby::RArray {
        self.as_any()._ptr() as _
    }

    #[inline]
    pub(crate) fn _flags(self) -> ruby::VALUE {
        unsafe { (*self._rarray()).basic.flags }
    }

    #[inline]
    pub(crate) fn _is_embedded(self) -> bool {
        use ruby::ruby_rstring_flags::*;
        self._flags() & (RSTRING_NOEMBED as ruby::VALUE) == 0
    }

    /// Creates a new instance from the elements in `slice`.
    #[inline]
    pub fn from_slice<'s, T>(slice: &'s [T]) -> Self
        where &'s [T]: Into<Self>
    {
        slice.into()
    }

    /// Returns the number of elements in `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::init().unwrap();
    /// use rosy::{Array, String};
    ///
    /// let s = String::from("hi");
    /// let a = Array::from_slice(&[s, s, s]);
    ///
    /// assert_eq!(a.len(), 3);
    /// ```
    #[inline]
    pub fn len(self) -> usize {
        use ruby::{ruby_rarray_flags::*, VALUE};
        unsafe {
            let rarray = &*self._rarray();
            let flags = rarray.basic.flags;
            if flags & (RARRAY_EMBED_FLAG as VALUE) == 0 {
                rarray.as_.heap.len as usize
            } else {
                let mask = RARRAY_EMBED_LEN_MASK >> RARRAY_EMBED_LEN_SHIFT;
                ((flags >> RARRAY_EMBED_LEN_SHIFT) & mask as VALUE) as usize
            }
        }
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
        let ptr = self._ptr() as *const AnyObject;
        std::slice::from_raw_parts(ptr, self.len())
    }

    /// Returns a mutable slice to the underlying objects of `self`.
    ///
    /// # Safety
    ///
    /// Care must be taken to ensure that the length of `self` is not changed
    /// through the VM or otherwise.
    #[inline]
    pub unsafe fn as_slice_mut(&mut self) -> &mut [AnyObject] {
        let ptr = self._ptr() as *mut AnyObject;
        std::slice::from_raw_parts_mut(ptr, self.len())
    }

    /// Removes all elements from `self`.
    #[inline]
    pub fn clear(self) {
        unsafe { ruby::rb_ary_clear(self.raw()) };
    }

    /// Appends all of the elements in `slice` to `self`.
    #[inline]
    pub fn append(self, slice: &[impl Object]) {
        let ptr = slice.as_ptr() as *const ruby::VALUE;
        let len = slice.len();
        unsafe { ruby::rb_ary_cat(self.raw(), ptr, len as _) };
    }

    /// Returns the result of performing `self + other`.
    #[inline]
    pub fn plus(self, other: Self) -> Self {
        unsafe { Array::_new(ruby::rb_ary_plus(self.raw(), other.raw())) }
    }

    /// Pushes `obj` onto the end of `self`.
    #[inline]
    pub fn push(self, obj: impl Object) -> AnyObject {
        unsafe { AnyObject(ruby::rb_ary_push(self.raw(), obj.raw())) }
    }

    /// Pops the last element from `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::init().unwrap();
    /// use rosy::{Array, String};
    ///
    /// let s = String::from("Hi");
    /// let a = Array::from_slice(&[s]);
    ///
    /// assert!(!a.pop().is_nil());
    /// assert!(a.pop().is_nil());
    /// ```
    #[inline]
    pub fn pop(self) -> AnyObject {
        unsafe { AnyObject(ruby::rb_ary_pop(self.raw())) }
    }

    /// Returns whether `self` contains `obj`.
    ///
    /// This is equivalent to the `include?` method.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::init().unwrap();
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
    #[inline]
    pub fn remove_all(self, obj: impl Object) -> AnyObject {
        unsafe { AnyObject(ruby::rb_ary_delete(self.raw(), obj.raw())) }
    }

    /// Reverses the contents of `self` in-palace.
    ///
    /// This is equivalent to the `reverse!` method.
    #[inline]
    pub fn reverse(self) {
        unsafe { ruby::rb_ary_reverse(self.raw()) };
    }

    /// Returns an instance with its contents sorted.
    #[inline]
    #[must_use]
    pub fn sorted(self) -> Self {
        unsafe { Array::_new(ruby::rb_ary_sort(self.raw())) }
    }

    /// Sorts the contents of `self` in-place.
    #[inline]
    pub fn sort(self) {
        unsafe { ruby::rb_ary_sort_bang(self.raw()) };
    }

    /// Joins the contents of `self` with `separator`.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::init().unwrap();
    /// use rosy::{Array, String};
    ///
    /// let s = String::from("-");
    /// let a = Array::from_slice(&[s, s, s]);
    ///
    /// assert_eq!(a.join("."), "-.-.-");
    /// ```
    #[inline]
    pub fn join(self, separator: impl Into<String>) -> String {
        let separator = separator.into();
        unsafe { String::_new(ruby::rb_ary_join(self.raw(), separator.raw())) }
    }
}
