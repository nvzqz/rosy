//! Interacting with the Ruby VM directly.

use std::{
    error::Error,
    ffi::CStr,
    fmt,
    os::raw::c_int,
};
use crate::{
    prelude::*,
    ruby,
};

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

/// Returns Ruby's level of paranoia. This is equivalent to reading `$SAFE`.
#[inline]
pub fn safe_level() -> c_int {
    unsafe { ruby::rb_safe_level() }
}

/// Sets Ruby's level of paranoia. The default value is 0.
///
/// # Safety
///
/// An exception will be raised if `level` is either negative or not supported.
#[inline]
pub unsafe fn set_safe_level(level: c_int) {
    ruby::rb_set_safe_level(level);
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

// monomorphization
fn _require(file: String, safe: c_int) -> Result<ruby::VALUE> {
    unsafe {
        crate::protected_no_panic(|| ruby::rb_require_safe(file.raw(), safe))
    }
}

/// Loads `file` with the current `safe_level`.
///
/// This returns `true` if successful or `false` if already loaded.
///
/// See [`require_with`](fn.require_with.html) for more info.
#[inline]
pub fn require(file: impl Into<String>) -> Result<bool> {
    require_with(file, safe_level())
}

/// Loads `file` with `safe_level`.
///
/// This returns `true` if successful or `false` if already loaded.
///
// Taken from docs on `rb_f_require` in Ruby's source code
/// If the filename does not resolve to an absolute path, it will be searched
/// for in the directories listed in`$LOAD_PATH` (`$:`).
///
/// If the filename has the extension `.rb`, it is loaded as a source file; if
/// the extension is `.so`, `.o`, or `.dll`, or the default shared library
/// extension on the current platform, Ruby loads the shared library as a Ruby
/// extension. Otherwise, Ruby tries adding `.rb`, `.so`, and so on to the name
/// until found. If the file named cannot be found, a `LoadError` will be
/// returned.
///
/// For Ruby extensions the filename given may use any shared library extension.
/// For example, on Linux the socket extension is `socket.so` and `require
/// 'socket.dll'` will load the socket extension.
///
/// The absolute path of the loaded file is added to `$LOADED_FEATURES` (`$"`).
/// A file will not be loaded again if its path already appears in `$"`.  For
/// example, `require 'a'; require './a'` will not load `a.rb` again.
///
/// ```ruby
/// require "my-library.rb"
/// require "db-driver"
/// ```
///
/// Any constants or globals within the loaded source file will be available in
/// the calling program's global namespace. However, local variables will not be
/// propagated to the loading environment.
#[inline]
pub fn require_with(
    file: impl Into<String>,
    safe_level: c_int,
) -> Result<bool> {
    // Convert to `bool` here for inlining
    Ok(_require(file.into(), safe_level)? != 0)
}

/// Loads `file` with the current `safe_level`, without checking for exceptions.
///
/// This returns `true` if successful or `false` if already loaded.
///
/// See [`require_with`](fn.require_with.html) for more info.
#[inline]
pub unsafe fn require_unchecked(file: impl Into<String>) -> bool {
    require_with_unchecked(file, safe_level())
}

/// Loads `file` with `safe_level`, without checking for exceptions.
///
/// This returns `true` if successful or `false` if already loaded.
///
/// See [`require_with`](fn.require_with.html) for more info.
#[inline]
pub unsafe fn require_with_unchecked(
    file: impl Into<String>,
    safe_level: c_int,
) -> bool {
    ruby::rb_require_safe(file.into().raw(), safe_level) != 0
}

/// Loads and executes the Ruby program `file`.
///
/// If the filename does not resolve to an absolute path, the file is searched
/// for in the library directories listed in `$:`.
///
/// If `wrap` is `true`, the loaded script will be executed under an anonymous
/// module, protecting the calling program's global namespace. In no
/// circumstance will any local variables in the loaded file be propagated to
/// the loading environment.
#[inline]
pub fn load(file: impl Into<String>, wrap: bool) -> Result {
    unsafe {
        let mut err = 0;
        ruby::rb_load_protect(file.into().raw(), wrap as c_int, &mut err);
        match err {
            0 => Ok(()),
            _ => Err(AnyException::_take_current()),
        }
    }
}

/// Loads and executes the Ruby program `file`, without checking for exceptions.
///
/// See [`load`](fn.load.html) for more info.
#[inline]
pub unsafe fn load_unchecked(file: impl Into<String>, wrap: bool) {
    ruby::rb_load(file.into().raw(), wrap as c_int)
}

/// Evaluates `script` in an isolated binding, returning an exception if one is
/// raised.
///
/// Variables:
/// - `__FILE__`: "(eval)"
/// - `__LINE__`: starts at 1
#[inline]
pub fn eval(script: &CStr) -> Result<AnyObject> {
    unsafe {
        let mut err = 0;
        let raw = ruby::rb_eval_string_protect(script.as_ptr(), &mut err);
        match err {
            0 => Ok(AnyObject::from_raw(raw)),
            _ => Err(AnyException::_take_current()),
        }
    }
}

/// Evaluates `script` under a module binding in an isolated binding, returning
/// an exception if one is raised.
///
/// Variables:
/// - `__FILE__`: "(eval)"
/// - `__LINE__`: starts at 1
#[inline]
pub fn eval_wrapped(script: &CStr) -> Result<AnyObject> {
    unsafe {
        let mut err = 0;
        let raw = ruby::rb_eval_string_wrap(script.as_ptr(), &mut err);
        match err {
            0 => Ok(AnyObject::from_raw(raw)),
            _ => Err(AnyException::_take_current()),
        }
    }
}

/// Evaluates `script` in an isolated binding without handling exceptions.
///
/// Variables:
/// - `__FILE__`: "(eval)"
/// - `__LINE__`: starts at 1
///
/// # Safety
///
/// Any raised errors must be handled in Rust-land.
#[inline]
pub unsafe fn eval_unchecked(script: &CStr) -> AnyObject {
    AnyObject::from_raw(ruby::rb_eval_string(script.as_ptr()))
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
