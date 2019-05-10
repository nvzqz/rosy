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

use std::error::Error;
use std::fmt;

#[doc(inline)]
pub use ruby::RUBY_VERSION;

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
