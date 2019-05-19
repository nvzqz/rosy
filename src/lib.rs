//! This crate provides high-level bindings to the [Ruby] virtual machine.
//!
//! # Installation
//!
//! This crate is available [on crates.io][crate] and can be used by adding the
//! following to your project's [`Cargo.toml`]:
//!
//! ```toml
//! [dependencies]
//! rosy = "0.0.4"
//! ```
//!
//! Rosy has functionality that is only available for certain Ruby versions. The
//! following features can currently be enabled:
//!
//! - `ruby_2_6`
//!
//! For example:
//!
//! ```toml
//! [dependencies.rosy]
//! version = "0.0.4"
//! features = ["ruby_2_6"]
//! ```
//!
//! Finally add this to your crate root (`main.rs` or `lib.rs`):
//!
//! ```
//! extern crate rosy;
//! ```
//!
//! # Initialization
//!
//! The Ruby virtual machine is initialized via [`vm::init`]:
//!
//! ```
//! rosy::vm::init().expect("Failed to initialize Ruby");
//! ```
//!
//! This should be called
//! once by the thread expected to be associated with Ruby. All mutations to
//! Ruby objects from there on are only safe from that same thread since the VM
//! is not known to be thread-safe.
//!
//! # Cleaning Up
//!
//! When done with the Ruby VM, one should call [`vm::destroy`], which will
//! return a status code appropriate for exiting the program.
//!
//! ```
//! # rosy::vm::init().unwrap();
//! if let Err(code) = unsafe { rosy::vm::destroy() } {
//!     std::process::exit(code);
//! }
//! ```
//!
//! # Catching Ruby Exceptions
//!
//! With Rosy, your Rust code can be [`protected`](fn.protected.html) from Ruby
//! exceptions when calling unchecked functions that may throw.
//!
//! Not catching an exception from Rust will result in a segmentation fault at
//! best. As a result, every function that throws an exception is annotated as
//! [`unsafe`] in Rust-land. If a function is found to not uphold this
//! invariant, please report it at [issue #4][issue4] or file a pull request to
//! fix this.
//!
//! ```
//! # rosy::vm::init().unwrap();
//! use rosy::{Object, String};
//!
//! let string = String::from("hello\r\n");
//!
//! let value: usize = rosy::protected(|| unsafe {
//!     string.call_unchecked("chomp!");
//!     string.len()
//! }).unwrap();
//!
//! assert_eq!(value, 5);
//! ```
//!
//! [`Cargo.toml`]: https://doc.rust-lang.org/cargo/reference/manifest.html
//! [crate]: https://crates.io/crates/rosy
//! [Ruby]: https://www.ruby-lang.org
//! [`vm::init`]: vm/fn.init.html
//! [`vm::destroy`]: vm/fn.destroy.html
//! [`unsafe`]: https://doc.rust-lang.org/book/ch19-01-unsafe-rust.html
//! [issue4]: https://github.com/oceanpkg/rosy/issues/4

#![cfg_attr(nightly, feature(doc_cfg))]
#![deny(missing_docs)]
#![cfg_attr(all(test, nightly), feature(test))]

#[cfg(all(test, nightly))]
extern crate test;

include!(env!("ROSY_RUBY_VERSION_CONST"));

#[path = "ruby_bindings/mod.rs"]
mod ruby;

mod rosy;
mod protected;
mod util;
pub mod array;
pub mod exception;
pub mod gc;
pub mod hash;
pub mod integer;
pub mod mixin;
pub mod object;
pub mod prelude;
pub mod string;
pub mod symbol;
pub mod vm;

#[doc(inline)]
pub use protected::*;

#[doc(inline)] // prelude
pub use self::{
    array::Array,
    exception::{AnyException, Exception},
    hash::Hash,
    integer::Integer,
    mixin::{Mixin, Class, Module},
    object::{AnyObject, Object, RosyObject},
    rosy::Rosy,
    string::String,
    symbol::{Symbol, SymbolId},
};

/// A simplified form of
/// [`Result`](https://doc.rust-lang.org/std/result/enum.Result.html) for
/// when exceptions are caught.
pub type Result<T = (), E = AnyException> = std::result::Result<T, E>;
