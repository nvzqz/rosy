//! Types and traits that are commonly used within this library.
//!
//! This module is intended to be glob imported via `use rosy::prelude::*` when
//! primarily working with Rosy types. This allows for having access to _almost_
//! everything one may need.
//!
//! **Note:** These items are all already available at the top crate level. If
//! only certain items are required, importing from the prelude directly is
//! unnecessary.
//!
//! **Important:** Rosy's [`String`][rb] type **will conflict** with Rust's
//! built-in [`String`][rs] type when imported as-is into the same module.
//!
//! [rb]: string/struct.String.html
//! [rs]: https://doc.rust-lang.org/std/string/struct.String.html

// This should match the import in `lib.rs` verbatim (literally copy + paste)
#[doc(no_inline)]
pub use crate::{
    array::Array,
    exception::{AnyException, Exception},
    hash::Hash,
    integer::Integer,
    mixin::{Mixin, Class, Module},
    object::{AnyObject, Object, RosyObject},
    Result,
    rosy::Rosy,
    string::String,
    symbol::{Symbol, SymbolId},
};
