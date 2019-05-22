//! Ruby arrays.

use std::{
    cmp::Ordering,
    fmt,
    iter::FromIterator,
    marker::PhantomData,
    ops::Add,
};
use crate::{
    object::{NonNullObject, Ty},
    prelude::*,
    ruby,
};

/// An instance of Ruby's `Array` class.
///
/// # Performance
///
/// Although caution must be taken with [`as_slice`](#method.as_slice) and its
/// mutable counterpart, it is _much_ faster to iterate over the inner slice of
/// objects in an array than it is to iterate over the array directly.
///
/// # Examples
///
/// Ruby arrays can be treated as somewhat like a [`Vec`] without the borrow
/// checker.
///
/// ```
/// # rosy::vm::init().unwrap();
/// # rosy::protected(|| {
/// use rosy::prelude::*;
///
/// let s = String::from("hellooo");
///
/// let a = Array::from_slice(&[s, s, s]);
/// assert_eq!(a.len(), 3);
///
/// for obj in a {
///     assert_eq!(obj, s);
/// }
/// # }).unwrap();
/// ```
///
/// Because the `Iterator` for `Array` performs a volatile read of the
/// array length each time, a
/// [buffer overrun](https://en.wikipedia.org/wiki/Buffer_overrun) will never
/// occur.
///
/// ```
/// # rosy::vm::init().unwrap();
/// # rosy::protected(|| {
/// # use rosy::prelude::*;
/// # let s = String::from("hellooo");
/// # let a = Array::from_slice(&[s, s, s]);
/// assert_eq!(a.len(), 3);
/// let mut num_iter = 0;
///
/// for _ in a {
///     // `unsafe` required because `pop` raises an exception if `a` is frozen
///     unsafe { a.pop() };
///     num_iter += 1;
/// }
///
/// assert_eq!(num_iter, 2);
/// # }).unwrap();
/// ```
///
/// Just like [`Vec`], one can even safely [`collect`] an iterator into an
/// `Array`:
///
/// ```
/// # rosy::vm::init().unwrap();
/// # rosy::protected(|| {
/// # use rosy::prelude::*;
/// let array: Array = (0..10).collect();
///
/// for (i, obj) in array.into_iter().enumerate() {
///     assert_eq!(obj, i);
/// }
/// # }).unwrap();
/// ```
///
/// [`Vec`]: https://doc.rust-lang.org/std/vec/struct.Vec.html
/// [`collect`]: https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect
#[repr(transparent)]
pub struct Array<O = AnyObject> {
    inner: NonNullObject,
    _marker: PhantomData<*mut O>,
}

impl<O> Clone for Array<O> {
    #[inline]
    fn clone(&self) -> Self { *self }
}

impl<O> Copy for Array<O> {}

impl<O: Object> AsRef<AnyObject> for Array<O> {
    #[inline]
    fn as_ref(&self) -> &AnyObject { self.inner.as_ref() }
}

impl<O: Object> From<Array<O>> for AnyObject {
    #[inline]
    fn from(object: Array<O>) -> AnyObject { object.inner.into() }
}

impl<O: Object> PartialEq<AnyObject> for Array<O> {
    #[inline]
    fn eq(&self, obj: &AnyObject) -> bool {
        self.as_any_object() == obj
    }
}

unsafe impl<O: Object> Object for Array<O> {
    #[inline]
    fn unique_id() -> Option<u128> {
        let base = O::unique_id()?;
        let this = Ty::ARRAY.id() as u128;
        Some(base.rotate_right(3) ^ this)
    }

    #[inline]
    fn cast<A: Object>(obj: A) -> Option<Self> {
        // Either:
        // - `A` is of type `Self`
        // - `Self` is `Array<AnyObject>` and `obj` is an array
        let is_valid = Self::unique_id() == A::unique_id() ||
            (O::unique_id() == AnyObject::unique_id() && obj.is_ty(Ty::ARRAY));
        if is_valid {
            unsafe { Some(Self::cast_unchecked(obj)) }
        } else {
            None
        }
    }

    #[inline]
    fn ty(self) -> Ty { Ty::ARRAY }

    #[inline]
    fn is_ty(self, ty: Ty) -> bool { ty == Ty::ARRAY }
}

impl<O: Object> fmt::Debug for Array<O> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Array")
            .field(&self.inner)
            .finish()
    }
}

impl<O: Object> fmt::Display for Array<O> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_any_object().fmt(f)
    }
}

// Safe because this is part of the contract of implementing `Object`.
impl<O: Object> From<&[O]> for Array<O> {
    #[inline]
    fn from(slice: &[O]) -> Self {
        let ptr = slice.as_ptr() as *const ruby::VALUE;
        let len = slice.len();
        unsafe { Array::from_raw(ruby::rb_ary_new_from_values(len as _, ptr)) }
    }
}

impl<O, A> PartialEq<[A]> for Array<O>
    where O: Object + PartialEq<A>,
{
    #[inline]
    fn eq(&self, other: &[A]) -> bool {
        unsafe { self.as_slice().eq(other) }
    }
}

impl<O, A> PartialEq<Vec<A>> for Array<O>
    where O: Object + PartialEq<A>,
{
    #[inline]
    fn eq(&self, other: &Vec<A>) -> bool {
        self == other.as_slice()
    }
}

impl<A> PartialEq<[A]> for AnyObject
    where AnyObject: PartialEq<A>
{
    #[inline]
    fn eq(&self, other: &[A]) -> bool {
        if let Some(array) = self.to_array() {
            array == *other
        } else {
            false
        }
    }
}

impl<A> PartialEq<&[A]> for AnyObject
    where AnyObject: PartialEq<A>
{
    #[inline]
    fn eq(&self, other: &&[A]) -> bool {
        PartialEq::<[A]>::eq(self, *other)
    }
}

impl<A> PartialEq<Vec<A>> for AnyObject
    where AnyObject: PartialEq<A>
{
    #[inline]
    fn eq(&self, other: &Vec<A>) -> bool {
        self == other.as_slice()
    }
}

impl<A> PartialEq<&Vec<A>> for AnyObject
    where AnyObject: PartialEq<A>
{
    #[inline]
    fn eq(&self, other: &&Vec<A>) -> bool {
        self == other.as_slice()
    }
}

impl<T: Object, U: Object> PartialEq<Array<U>> for Array<T> {
    #[inline]
    fn eq(&self, other: &Array<U>) -> bool {
        self.partial_cmp(other) == Some(Ordering::Equal)
    }
}

impl<T: Object, U: Object> PartialOrd<Array<U>> for Array<T> {
    #[inline]
    fn partial_cmp(&self, other: &Array<U>) -> Option<Ordering> {
        let value = unsafe { ruby::rb_ary_cmp(self.raw(), other.raw()) };
        if value == crate::util::NIL_VALUE {
            return None;
        }
        Some(crate::util::value_to_fixnum(value).cmp(&0))
    }
}

impl<O: Object, A: Into<O>> FromIterator<A> for Array<O> {
    #[inline]
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (size, _) = iter.size_hint();
        let array = Self::with_capacity(size);
        for obj in iter {
            unsafe { array.push(obj.into()) };
        }
        array
    }
}

impl<O: Object> IntoIterator for Array<O> {
    type Item = O;
    type IntoIter = Iter<O>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Iter { array: self, current: 0 }
    }
}

// Allows for `a1 + a2` in Rust
impl<O: Object> Add for Array<O> {
    type Output = Self;

    #[inline]
    fn add(self, other: Self) -> Self::Output {
        self.plus(other)
    }
}

impl<O: Object> Array<O> {
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

    /// Duplicates the contents of `self` into a new instance.
    #[inline]
    pub fn duplicate(self) -> Self {
        unsafe { Self::from_raw(ruby::rb_ary_dup(self.raw())) }
    }

    /// Returns the number of elements in `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// # rosy::protected(|| {
    /// use rosy::{Array, String};
    ///
    /// let s = String::from("hi");
    /// let a = Array::from_slice(&[s, s, s]);
    ///
    /// assert_eq!(a.len(), 3);
    /// # }).unwrap();
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

    /// Returns a pointer to the first object in `self`.
    #[inline]
    pub fn as_ptr(self) -> *const O {
        unsafe { (*self.rarray()).start() as *const O }
    }

    /// Returns a mutable pointer to the first object in `self`.
    #[inline]
    pub fn as_ptr_mut(self) -> *mut O {
        unsafe { (*self.rarray()).start_mut() as *mut O }
    }

    /// Returns a slice to the underlying objects of `self`.
    ///
    /// # Safety
    ///
    /// Care must be taken to ensure that the length of `self` is not changed
    /// through the VM or otherwise.
    #[inline]
    pub unsafe fn as_slice(&self) -> &[O] {
        std::slice::from_raw_parts(self.as_ptr(), self.len())
    }

    /// Returns a mutable slice to the underlying objects of `self`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised. Care must also be taken to ensure that the
    /// length of `self` is not changed through the VM or otherwise.
    #[inline]
    pub unsafe fn as_slice_mut(&mut self) -> &mut [O] {
        ruby::rb_ary_modify(self.raw());
        std::slice::from_raw_parts_mut(self.as_ptr_mut(), self.len())
    }

    /// Returns the object at `index` or `None` if `index` is out-of-bounds.
    #[inline]
    pub fn get(self, index: usize) -> Option<O> {
        unsafe { self.as_slice().get(index).map(|&obj| obj) }
    }

    /// Returns the object at `index` without bounds checking.
    #[inline]
    pub unsafe fn get_unchecked(self, index: usize) -> O {
        *self.as_slice().get_unchecked(index)
    }

    /// Returns the first object in `self`.
    #[inline]
    pub fn first(self) -> Option<O> {
        unsafe { self.as_slice().first().map(|&obj| obj) }
    }

    /// Returns the last element in `self`.
    #[inline]
    pub fn last(self) -> Option<O> {
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
    pub unsafe fn extend_from_slice(self, slice: &[O]) {
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
    /// The caller must ensure that `self` is not:
    /// - Frozen, or else a `FrozenError` exception will be raised
    /// - `Array<AnyObject>` that references `Array<ConcreteObject>` where `obj`
    ///   is not the same type as `ConcreteObject`
    #[inline]
    pub unsafe fn push(self, obj: O) -> AnyObject {
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
    /// # rosy::protected(|| {
    /// use rosy::{Array, String};
    ///
    /// let s = String::from("Hi");
    /// let a = Array::from_slice(&[s]);
    ///
    /// unsafe {
    ///     assert!(!a.pop().is_nil());
    ///     assert!(a.pop().is_nil());
    /// }
    /// # }).unwrap();
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
    /// # rosy::protected(|| {
    /// use rosy::{Array, String};
    ///
    /// let s = String::from("hi");
    /// let a = Array::from_slice(&[String::from("yo"), s]);
    ///
    /// assert!(a.contains(s));
    /// # }).unwrap();
    /// ```
    // SAFETY: The use of `impl Object` is fine here since this method is not
    // inserting it into `self`
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
    // SAFETY: The use of `impl Object` is fine here since this method is not
    // inserting it into `self`
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
    /// # rosy::protected(|| {
    /// use rosy::{Array, String};
    ///
    /// let s = String::from("-");
    /// let a = Array::from_slice(&[s, s, s]);
    ///
    /// assert_eq!(a.join("."), "-.-.-");
    /// # }).unwrap();
    /// ```
    #[inline]
    pub fn join(self, separator: impl Into<String>) -> String {
        let separator = separator.into().raw();
        unsafe { String::from_raw(ruby::rb_ary_join(self.raw(), separator)) }
    }
}

/// An iterator over the elements of an [`Array`](struct.Array.html).
#[derive(Clone, Debug)]
pub struct Iter<O: Object> {
    array: Array<O>,
    current: usize,
}

impl<O: Object> Iterator for Iter<O> {
    type Item = O;

    #[inline]
    fn next(&mut self) -> Option<O> {
        let obj = self.array.get(self.current)?;
        self.current += 1;
        Some(obj)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        // Because `array` may be resized during the iteration, the lower and
        // upper bound may be different than the yielded number of elements;
        // however, it is safe for an `Iterator` implementation to do so
        let len = self.array.len();
        (len, Some(len))
    }

    #[inline]
    fn count(self) -> usize {
        self.array.len()
    }

    #[inline]
    fn last(self) -> Option<O> {
        self.array.last()
    }
}
