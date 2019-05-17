use std::{
    mem,
    os::raw::c_char,
};
use crate::prelude::*;

/// A Rust type that can be used as a object.
///
/// # Examples
///
/// Implementing this trait allows for wrapping Rust data in a
/// [`RosyObject`](object/struct.RosyObject.html).
///
/// ```
/// # rosy::vm::init().unwrap();
/// use std::os::raw::c_char;
/// use rosy::{Rosy, RosyObject, Class};
///
/// struct Flag<'a> {
///     did_free: &'a mut bool
/// }
///
/// unsafe impl Rosy for Flag<'_> {
///     const ID: *const c_char = "rosy_flag\0".as_ptr() as _;
///
///     fn class() -> Class {
///         Class::get_or_def("RosyFlag").unwrap()
///     }
///
///     fn mark(&self) {}
///
///     fn free(self: Box<Self>) {
///         *self.did_free = true;
///     }
/// }
///
/// let mut did_free = false;
/// let obj = RosyObject::from(Flag { did_free: &mut did_free });
///
/// unsafe { rosy::vm::destroy() };
///
/// assert!(did_free);
/// ```
pub unsafe trait Rosy: Sized {
    /// A C string of the unique identifier of the type.
    ///
    /// Note that the type is not
    /// [`CStr`](https://doc.rust-lang.org/std/ffi/struct.CStr.html). This is
    /// because creating a constant instance can only be done on nightly.
    const ID: *const c_char;

    /// A unique identifier for `RosyObject<Self>` to facilitate casting.
    ///
    /// # Safety
    ///
    /// This value _must_ be unique. Rosy's built-in objects use identifiers
    /// that are very close to `u128::max_value()`, so those are easy to avoid.
    #[inline]
    fn unique_object_id() -> Option<u128> {
        None
    }

    /// Returns the class defined for this type.
    ///
    /// The default is `RustObject`, however other implementors of this trait
    /// should consider using a different class to define methods or properties
    /// on.
    #[inline]
    fn class() -> Class {
        Class::rust_object()
    }

    /// Attempts to create a `RosyObject` instance by casting `obj`.
    ///
    /// This could be implemented by checking against [`class`](#method.class)
    /// but care must be taken to ensure that all instances of this type's class
    /// refer to Rust data of type `Self`.
    ///
    /// The default implementation checks the
    /// [`unique_object_id`](#method.unique_object_id) of `Self` against the
    /// `unique_id` of `A`.
    #[inline]
    #[allow(unused_variables)]
    fn cast<A: Object>(obj: A) -> Option<RosyObject<Self>> {
        if A::unique_id() == Self::unique_object_id() {
            unsafe { Some(RosyObject::cast_unchecked(obj)) }
        } else {
            None
        }
    }

    /// Called during Ruby's mark phase of garbage collection to determine which
    /// Ruby references in `self` are live and should not be swept.
    ///
    /// # Safety
    ///
    /// This method is called during garbage collection and it is required that:
    /// - _All_ live Ruby objects are properly marked
    /// - No new Ruby objects are allocated
    fn mark(&self);

    /// Runs destructors and frees `self`.
    ///
    /// # Safety
    ///
    /// The implementor must ensure that no new Ruby objects are allocated.
    #[inline]
    fn free(self: Box<Self>) {
        drop(self);
    }

    /// Returns the estimated memory consumption of `self` in bytes.
    #[inline]
    fn size(&self) -> usize {
        mem::size_of_val(self)
    }
}

unsafe impl<R: Rosy> Rosy for &[R] {
    const ID: *const c_char = b"rust_slice\0".as_ptr() as _;

    #[inline]
    fn mark(&self) {
        self.iter().for_each(Rosy::mark);
    }

    #[inline]
    fn size(&self) -> usize {
        self.iter().fold(0, |cur, r| cur + r.size())
    }
}

unsafe impl<R: Rosy> Rosy for &mut [R] {
    const ID: *const c_char = b"rust_mut_slice\0".as_ptr() as _;

    #[inline]
    fn mark(&self) {
        self.iter().for_each(Rosy::mark);
    }

    #[inline]
    fn size(&self) -> usize {
        self.iter().fold(0, |cur, r| cur + r.size())
    }
}

unsafe impl<R: Rosy> Rosy for Vec<R> {
    const ID: *const c_char = b"rust_vec\0".as_ptr() as _;

    #[inline]
    fn unique_object_id() -> Option<u128> {
        let inner = R::unique_object_id()?;
        let base = u128::from_le_bytes(*b"built-in Vec<T> ");
        Some(inner.rotate_right(1) ^ base)
    }

    #[inline]
    fn mark(&self) {
        self.iter().for_each(Rosy::mark);
    }

    #[inline]
    fn size(&self) -> usize {
        self.iter().fold(0, |cur, r| cur + r.size())
    }
}

unsafe impl Rosy for &str {
    const ID: *const c_char = b"rust_str\0".as_ptr() as _;

    #[inline]
    fn mark(&self) {}

    #[inline]
    fn size(&self) -> usize {
        mem::size_of_val(*self)
    }
}

unsafe impl Rosy for std::string::String {
    const ID: *const c_char = b"rust_string\0".as_ptr() as _;

    #[inline]
    fn unique_object_id() -> Option<u128> {
        Some((!0) - 0xff)
    }

    #[inline]
    fn mark(&self) {}

    #[inline]
    fn size(&self) -> usize {
        self.as_str().size()
    }
}
