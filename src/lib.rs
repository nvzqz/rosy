//! This crate provides high-level bindings to the [Ruby] virtual machine.
//!
//! # Initialization
//!
//! The Ruby virtual machine is initialized via [`init`]:
//!
//! ```
//! rosy::init().expect("Failed to initialize Ruby");
//! ```
//!
//! This should be called
//! once by the thread expected to be associated with Ruby. All mutations to
//! Ruby objects from there on are only safe from that same thread since the VM
//! is not known to be thread-safe.
//!
//! # Cleaning Up
//!
//! When done with the Ruby VM, one should call [`destroy`], which will return a
//! status code appropriate for exiting the program.
//!
//! ```
//! # rosy::init().unwrap();
//! if let Err(code) = unsafe { rosy::destroy() } {
//!     std::process::exit(code);
//! }
//! ```
//!
//! [Ruby]: https://www.ruby-lang.org
//! [`init`]: fn.init.html
//! [`destroy`]: fn.destroy.html

#![deny(missing_docs)]

extern crate ruby_sys as ruby;

#[doc(inline)]
pub use ruby::RUBY_VERSION;

use std::{
    error::Error,
    fmt,
    mem,
};

mod util;
pub mod array;
pub mod exception;
pub mod gc;
pub mod hash;
pub mod mixin;
pub mod object;
pub mod prelude;
pub mod string;
pub mod symbol;
pub mod vm;

#[doc(inline)]
pub use self::{
    array::Array,
    exception::{AnyException, Exception},
    hash::Hash,
    mixin::{Mixin, Class, Module},
    object::{AnyObject, Object},
    string::String,
    symbol::{Symbol, SymbolId},
};

/// Initializes the Ruby VM, returning an error code if it failed.
#[inline]
pub fn init() -> Result<(), InitError> {
    match unsafe { ruby::ruby_setup() } {
        0 => Ok(()),
        e => Err(InitError(e)),
    }
}

/// Destructs the Ruby VM, runs its finalization processes, and frees all
/// resources used by it.
///
/// Returns an exit code on error appropriate for passing into
/// [`std::process::exit`](https://doc.rust-lang.org/std/process/fn.exit.html).
///
/// # Safety
///
/// The caller must ensure that no VM resources are being used by other threads
/// or will continue to be used after this function finishes.
///
/// After this function is called, it will no longer be possible to call
/// [`init`](fn.init.html).
#[inline]
pub unsafe fn destroy() -> Result<(), i32> {
    match ruby::ruby_cleanup(0) {
        0 => Ok(()),
        e => Err(e),
    }
}

/// Initializes the load path for `require`-ing gems.
///
/// # Examples
///
/// ```
/// rosy::init().unwrap();
/// rosy::init_load_path();
/// ```
#[inline]
pub fn init_load_path() {
    unsafe { ruby::ruby_init_loadpath() };
}

/// Calls `f` and returns its output or an exception if one is raised in `f`.
///
/// # Examples
///
/// This is great for calling methods that may not exist:
///
/// ```
/// # rosy::init();
/// use rosy::{Object, String, protected};
///
/// let string = String::from("¡Hola!");
/// let result = protected(|| unsafe { string.call_unchecked("likes_pie?") });
///
/// assert!(result.is_err());
/// ```
///
/// Calls can even be nested like so:
///
/// ```
/// # rosy::init();
/// use rosy::{Object, String, protected};
///
/// let string = String::from("Hiii!!!");
///
/// let outer = protected(|| {
///     protected(|| unsafe {
///         string.call_unchecked("likes_pie?")
///     }).unwrap_err();
///     string
/// });
///
/// assert_eq!(outer.unwrap(), string);
/// ```
#[inline]
pub fn protected<F, O>(f: F) -> Result<O, AnyException>
    where F: FnOnce() -> O
{
    unsafe extern "C" fn wrapper<F, O>(ctx: ruby::VALUE) -> ruby::VALUE
        where F: FnOnce() -> O
    {
        let (f, out) = &mut *(ctx as *mut (Option<F>, &mut O));

        // Get the `F` out of `Option<F>` to call by-value, which is required by
        // the `FnOnce` trait
        let f = f.take().unwrap_or_else(|| std::hint::unreachable_unchecked());

        std::ptr::write(*out, f());

        AnyObject::nil().raw()
    }
    unsafe {
        // Required to prevent stack unwinding (if there's a `panic!` in `f()`)
        // from dropping `out`, which is uninitialized memory until `f()`
        use mem::ManuallyDrop;

        // These shenanigans allow us to pass in a pointer to `f`, with a
        // pointer to its uninitialized output, into `rb_protect` to make them
        // accessible from `wrapper`
        let mut out = ManuallyDrop::new(mem::uninitialized::<O>());
        let mut ctx = (Some(f), &mut *out);
        let ctx = &mut ctx as *mut (Option<F>, &mut O) as ruby::VALUE;

        let mut err = 0;
        ruby::rb_protect(Some(wrapper::<F, O>), ctx, &mut err);
        match err {
            0 => Ok(ManuallyDrop::into_inner(out)),
            _ => Err(AnyException::_take_current()),
        }
    }
}

/// An error indicating that [`init`](fn.init.html) failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InitError(i32);

impl InitError {
    /// Returns the error code given by the VM.
    #[inline]
    pub fn code(&self) -> i32 {
        self.0
    }
}

impl fmt::Display for InitError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} (error code {})", self.description(), self.code())
    }
}

impl Error for InitError {
    #[inline]
    fn description(&self) -> &str {
        "Failed to initialize Ruby"
    }
}
