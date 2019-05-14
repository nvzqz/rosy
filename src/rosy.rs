use std::os::raw::c_char;
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
///     fn mark(&mut self) {}
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

    /// Returns the class defined for this type.
    fn class() -> Class;

    /// Called during Ruby's mark phase of garbage collection to determine which
    /// Ruby references in `self` are live and should not be swept.
    ///
    /// # Safety
    ///
    /// This method is called during garbage collection and it is required that:
    /// - _All_ live Ruby objects are properly marked
    /// - No new Ruby objects are allocated
    fn mark(&mut self);

    /// Runs destructors and frees `self`.
    ///
    /// # Safety
    ///
    /// The implementor
    #[inline]
    fn free(self: Box<Self>) {
        drop(self);
    }

    /// Returns the estimated memory consumption of `self` in bytes.
    #[inline]
    fn size(&self) -> usize {
        std::mem::size_of_val(self)
    }
}
