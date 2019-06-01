//! Ruby ranges.

use std::{
    fmt,
    marker::PhantomData,
    ops::Bound,
    os::raw::c_int,
};
use crate::{
    prelude::*,
    object::NonNullObject,
    ruby,
};

mod into_bounds;
pub use into_bounds::*;

/// An instance of Ruby's `Range` type.
///
/// This type supports being instantiated from [`Range`], [`RangeInclusive`],
/// and [`RangeTo`].
///
/// # `a..b` and `a...b` versus `a..b` and `a..=b`
///
/// In Rust (and many other languages), `a..b` denotes an _inclusive_ range.
/// However, in Ruby this syntax denotes an _exclusive_ range.
///
/// In Rust, `a..=b` denotes an _exclusive_ range whereas in Ruby, `...` denotes
/// an _inclusive_ range.
///
/// # Examples
///
/// An exclusive range can be instantiated quite simply:
///
/// ```
/// # rosy::vm::init().unwrap();
/// use rosy::{Range, Integer, Object};
///
/// let range = Range::<Integer>::new(0..10).unwrap();
/// assert_eq!(range.to_s(), "0...10");
/// ```
///
/// The start and end bounds can be retrieved via `into_bounds`:
///
/// ```
/// # rosy::vm::init().unwrap();
/// # use rosy::{Range, Integer, Object};
/// use std::ops::Bound;
///
/// let range = Range::<Integer>::new(1..=10).unwrap();
///
/// let (start, end) = range.into_bounds();
///
/// assert_eq!(start, 1);
/// assert_eq!(end, Bound::Included(Integer::from(10)));
/// ```
///
/// [`Range`]: https://doc.rust-lang.org/std/ops/struct.Range.html
/// [`RangeInclusive`]: https://doc.rust-lang.org/std/ops/struct.RangeInclusive.html
/// [`RangeTo`]: https://doc.rust-lang.org/std/ops/struct.RangeTo.html
#[repr(transparent)]
pub struct Range<S = AnyObject, E = S> {
    inner: NonNullObject,
    _marker: PhantomData<(S, E)>,
}

impl<S, E> Clone for Range<S, E> {
    fn clone(&self) -> Self { *self }
}

impl<S, E> Copy for Range<S, E> {}

impl<S: Object, E: Object> AsRef<AnyObject> for Range<S, E> {
    #[inline]
    fn as_ref(&self) -> &AnyObject { self.inner.as_ref() }
}

impl<S: Object, E: Object> From<Range<S, E>> for AnyObject {
    #[inline]
    fn from(object: Range<S, E>) -> AnyObject { object.inner.into() }
}

impl<S: Object, E: Object> PartialEq<AnyObject> for Range<S, E> {
    #[inline]
    fn eq(&self, obj: &AnyObject) -> bool {
        self.as_any_object() == obj
    }
}

impl<S: Object, E: Object> fmt::Debug for Range<S, E> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Range")
            .field(&self.inner)
            .finish()
    }
}

unsafe impl<S: Object, E: Object> Object for Range<S, E> {
}

impl<S: Object, E: Object> IntoBounds<S, E> for Range<S, E> {
    #[inline]
    fn into_bounds(self) -> (S, Bound<E>) {
        unsafe {
            let mut start: ruby::VALUE = 0;
            let mut end: ruby::VALUE = 0;
            let mut excl: c_int = 0;
            ruby::rb_range_values(self.raw(), &mut start, &mut end, &mut excl);

            let start = S::from_raw(start);

            let end = if end == crate::util::NIL_VALUE {
                Bound::Unbounded
            } else {
                let end = E::from_raw(end);
                if excl != 0 {
                    Bound::Excluded(end)
                } else {
                    Bound::Included(end)
                }
            };

            (start, end)
        }
    }
}

impl Range {
    /// Creates a new instance from the given bounds, returning an exception if
    /// one is raised.
    ///
    /// If `end` is `nil`, an infinite (unbounded) range is produced.
    pub fn from_bounds(
        start: AnyObject,
        end: AnyObject,
        exclusive: bool,
    ) -> Result<Self> {
        unsafe {
            crate::protected_no_panic(|| {
                Self::from_bounds_unchecked(start, end, exclusive)
            })
        }
    }

    /// Creates a new instance from the given bounds, without checking for
    /// exceptions.
    ///
    /// If `end` is `nil`, an infinite (unbounded) range is produced.
    ///
    /// # Safety
    ///
    /// An exception may be raised if `start` and `end` cannot be compared.
    #[inline]
    pub unsafe fn from_bounds_unchecked(
        start: AnyObject,
        end: AnyObject,
        exclusive: bool,
    ) -> Self {
        let raw = ruby::rb_range_new(start.raw(), end.raw(), exclusive as _);
        Self::from_raw(raw)
    }
}

impl<S: Object, E: Object> Range<S, E> {
    /// Creates a new instance from `range`, returning an exception if one is
    /// raised.
    #[inline]
    pub fn new<R, A, B>(range: R) -> Result<Self>
    where
        R: IntoBounds<A, B>,
        A: Into<S>,
        B: Into<E>,
    {
        let (start, end) = range.into_bounds();
        let start = start.into().into_any_object();
        let (end, exclusive) = match end {
            Bound::Included(end) => (end.into().into_any_object(), false),
            Bound::Excluded(end) => (end.into().into_any_object(), true),
            Bound::Unbounded => (AnyObject::nil(), true),
        };
        unsafe {
            let range = Range::from_bounds(start, end, exclusive)?;
            Ok(Self::cast_unchecked(range))
        }
    }

    /// Creates a new instance from `range`, without checking for exceptions.
    ///
    /// # Safety
    ///
    /// An exception may be raised if `S` and `E` cannot be compared.
    #[inline]
    pub unsafe fn new_unchecked<R, A, B>(range: R) -> Self
    where
        R: IntoBounds<A, B>,
        A: Into<S>,
        B: Into<E>,
    {
        let (start, end) = range.into_bounds();
        let start = start.into().into_any_object();
        let (end, exclusive) = match end {
            Bound::Included(end) => (end.into().into_any_object(), false),
            Bound::Excluded(end) => (end.into().into_any_object(), true),
            Bound::Unbounded => (AnyObject::nil(), true),
        };
        let range = Range::from_bounds_unchecked(start, end, exclusive);
        Self::cast_unchecked(range)
    }

    /// Returns a range over `AnyObject`s.
    #[inline]
    pub fn into_any_range(self) -> Range {
        unsafe { Range::cast_unchecked(self) }
    }

    /// Returns the start (inclusive) and end bounds of `self`.
    #[inline]
    pub fn into_bounds(self) -> (S, Bound<E>) {
        IntoBounds::into_bounds(self)
    }

    /// Returns whether `obj` is contained within `self`.
    ///
    /// # Examples
    ///
    /// This corresponds to the `include?` method in Ruby.
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::prelude::*;
    ///
    /// let range = Range::<Integer>::new(10..15).unwrap();
    ///
    /// assert!(range.contains(12));
    /// ```
    #[inline]
    pub fn contains(self, obj: impl Into<AnyObject>) -> bool {
        unsafe { self.call_with("include?", &[obj.into()]).is_true() }
    }

    /// Returns the size of `self` as an `Integer`, if the bounds are `Numeric`
    /// values.
    #[inline]
    pub fn size(self) -> Option<Integer> {
        unsafe {
            let size = self.call("size");
            if size.is_nil() {
                None
            } else {
                Some(Integer::cast_unchecked(size))
            }
        }
    }

    /// Returns the size of `self` as a `usize`, if the bounds are `Numeric`
    /// values and the value can be represented as a `usize`.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::prelude::*;
    ///
    /// let range = Range::<Integer>::new(0..10).unwrap();
    ///
    /// assert_eq!(range.len(), Some(10));
    /// ```
    #[inline]
    pub fn len(self) -> Option<usize> {
        self.size()?.to_value()
    }
}
