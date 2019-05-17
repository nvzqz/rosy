//! Ruby's garbage collector.

use crate::{
    prelude::*,
    ruby::{self, VALUE},
};

/// Starts the garbage collector.
#[inline]
pub fn start() {
    unsafe { ruby::rb_gc_start() };
}

/// Tells the garbage collector how much memory is being used by an external
/// library. This may trigger the GC to resize or free emory blocks.
#[inline]
pub fn adjust_mem_usage(diff: isize) {
    unsafe { ruby::rb_gc_adjust_memory_usage(diff) };
}

/// Returns the number of times the GC has ran.
#[inline]
pub fn count() -> usize {
    unsafe { ruby::rb_gc_count() }
}

/// Disables the garbage collector, returning whether it was already previously
/// disabled.
#[inline]
pub fn disable() -> bool {
    unsafe { ruby::rb_gc_disable() != 0 }
}

/// Enables the garbage collector, returning whether it was already previously
/// enabled.
#[inline]
pub fn enable() -> bool {
    unsafe { ruby::rb_gc_enable() != 0 }
}

/// Calls `f` while the garbage collector is disabled.
#[inline]
pub fn disabled<F, O>(f: F) -> O
    where F: FnOnce() -> O
{
    disable();
    let output = f();
    enable();
    output
}

/// Forces `obj` to be garbage-collected.
///
/// # Safety
///
/// The caller must ensure that `obj` does not have ownership over any
/// currently-referenced memory.
#[inline]
pub unsafe fn force_recycle(obj: impl Object) {
    ruby::rb_gc_force_recycle(obj.raw());
}

// Only safely usable with `Symbol` and `Hash`
#[inline]
unsafe fn _stat(key: impl Object) -> usize {
    ruby::rb_gc_stat(key.raw())
}

/// Returns the status information for `key`, or an exception if one is raised.
///
/// # Examples
///
/// The number of available heap slots can be retrieved as such:
///
/// ```
/// # rosy::vm::init().unwrap();
/// let slots = rosy::gc::stat("heap_available_slots").unwrap();
/// assert_ne!(slots, 0);
/// ```
#[inline]
pub fn stat(key: impl GcInfoKey) -> Result<usize> {
    key.stat_gc()
}

/// Returns the status information for `key`.
///
/// # Safety
///
/// An exception may be raised if `key` is unknown.
#[inline]
pub unsafe fn stat_unchecked(key: impl GcInfoKey) -> usize {
    key.stat_gc_unchecked()
}

// Only safely usable with `Symbol` and `Hash`
#[inline]
unsafe fn _latest_info(key: impl Object) -> AnyObject {
    AnyObject::from_raw(ruby::rb_gc_latest_gc_info(key.raw()))
}

/// Returns the latest information regarding `key` with respect to the garbage
/// collector.
#[inline]
pub fn latest_info(key: impl GcInfoKey) -> Result<AnyObject> {
    key.latest_gc_info()
}

/// Returns the latest information regarding `key` with respect to the garbage
/// collector.
///
/// # Safety
///
/// An exception may be raised if `key` is unknown.
#[inline]
pub unsafe fn latest_info_unchecked(key: impl GcInfoKey) -> AnyObject {
    key.latest_gc_info_unchecked()
}

/// Marks the object for Ruby to avoid garbage collecting it.
#[inline]
pub fn mark(obj: impl Object) {
    unsafe { ruby::rb_gc_mark(obj.raw()) };
}

/// Iterates over `objs`, `mark`ing each one.
#[inline]
pub fn mark_iter<I, O>(objs: I)
where
    I: IntoIterator<Item = O>,
    O: Object,
{
    objs.into_iter().for_each(mark);
}

/// Marks the object for Ruby to avoid garbage collecting it.
// TODO: Figure out what the difference is between this and `mark`
#[inline]
pub fn mark_maybe(obj: impl Object) {
    unsafe { ruby::rb_gc_mark_maybe(obj.raw()) };
}

/// Registers the object address with the garbage collector and tells it to
/// avoid collecting it.
#[inline]
pub fn register_mark(obj: impl Object) {
    unsafe { ruby::rb_gc_register_mark_object(obj.raw()) };
}

/// Registers `address` with the garbage collector.
#[inline]
pub fn register(address: &impl Object) {
    let address = address as *const _ as *const VALUE as *mut VALUE;
    unsafe { ruby::rb_gc_register_address(address) };
}

/// Unregisters `address` with the garbage collector.
#[inline]
pub fn unregister(address: &impl Object) {
    let address = address as *const _ as *const VALUE as *mut VALUE;
    unsafe { ruby::rb_gc_unregister_address(address) };
}

/// A key that can be used to look up what the latest information is about the
/// garbage collector.
pub trait GcInfoKey: Sized {
    /// Returns the status information for `self` with respect to the garbage
    /// collector, or an exception if one is raised.
    #[inline]
    fn stat_gc(self) -> Result<usize> {
        crate::protected(|| unsafe { self.stat_gc_unchecked() })
    }

    /// Returns the status information for `self` with respect to the garbage
    /// collector.
    ///
    /// # Safety
    ///
    /// If the key is not available, an exception is thrown that should be
    /// caught and handled correctly.
    unsafe fn stat_gc_unchecked(self) -> usize;

    /// Returns the latest information regarding `self` with respect to the
    /// garbage collector.
    ///
    /// If an exception is raised, it is returned as a `Result::Err`.
    #[inline]
    fn latest_gc_info(self) -> Result<AnyObject> {
        crate::protected(|| unsafe { self.latest_gc_info_unchecked() })
    }

    /// Returns the latest information regarding `self` with respect to the
    /// garbage collector.
    ///
    /// # Safety
    ///
    /// If the key is not available, an exception is thrown that should be
    /// caught and handled correctly.
    unsafe fn latest_gc_info_unchecked(self) -> AnyObject;
}

impl GcInfoKey for Hash {
    #[inline]
    unsafe fn stat_gc_unchecked(self) -> usize {
       _stat(self)
    }

    #[inline]
    unsafe fn latest_gc_info_unchecked(self) -> AnyObject {
        _latest_info(self)
    }
}

impl GcInfoKey for Symbol {
    #[inline]
    unsafe fn stat_gc_unchecked(self) -> usize {
        _stat(self)
    }

    #[inline]
    unsafe fn latest_gc_info_unchecked(self) -> AnyObject {
        _latest_info(self)
    }
}

impl GcInfoKey for &str {
    #[inline]
    unsafe fn stat_gc_unchecked(self) -> usize {
        Symbol::from(self).stat_gc_unchecked()
    }

    #[inline]
    unsafe fn latest_gc_info_unchecked(self) -> AnyObject {
        Symbol::from(self).latest_gc_info_unchecked()
    }
}

impl GcInfoKey for String {
    #[inline]
    unsafe fn stat_gc_unchecked(self) -> usize {
        Symbol::from(self).stat_gc_unchecked()
    }

    #[inline]
    unsafe fn latest_gc_info_unchecked(self) -> AnyObject {
        Symbol::from(self).latest_gc_info_unchecked()
    }
}
