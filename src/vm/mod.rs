//! Interacting with the Ruby VM directly.

use std::{error::Error, fmt};
use crate::ruby;

mod instr_seq;
pub use instr_seq::*;

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
/// rosy::vm::init().unwrap();
/// rosy::vm::init_load_path();
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
