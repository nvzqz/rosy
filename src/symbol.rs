//! Ruby symbols.

use std::{
    convert::TryFrom,
    ffi::CStr,
    fmt,
};
use crate::{
    object::{NonNullObject, Ty},
    prelude::*,
    string::Encoding,
    ruby,
};

/// An instance of Ruby's `Symbol` class.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Symbol(NonNullObject);

impl AsRef<AnyObject> for Symbol {
    #[inline]
    fn as_ref(&self) -> &AnyObject { self.0.as_ref() }
}

impl From<Symbol> for AnyObject {
    #[inline]
    fn from(object: Symbol) -> AnyObject { object.0.into() }
}

impl PartialEq<AnyObject> for Symbol {
    #[inline]
    fn eq(&self, obj: &AnyObject) -> bool {
        self.as_any_object() == obj
    }
}

unsafe impl Object for Symbol {
    #[inline]
    fn cast<A: Object>(obj: A) -> Option<Self> {
        if obj.is_ty(Ty::SYMBOL) {
            unsafe { Some(Self::cast_unchecked(obj)) }
        } else {
            None
        }
    }

    #[inline]
    fn ty(self) -> Ty { Ty::SYMBOL }

    #[inline]
    fn is_ty(self, ty: Ty) -> bool { ty == Ty::SYMBOL }
}

impl fmt::Display for Symbol {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_any_object().fmt(f)
    }
}

impl From<String> for Symbol {
    #[inline]
    fn from(s: String) -> Symbol {
        SymbolId::from(s).into()
    }
}

impl From<&str> for Symbol {
    #[inline]
    fn from(s: &str) -> Symbol {
        SymbolId::from(s).into()
    }
}

impl TryFrom<Symbol> for &str {
    type Error = std::str::Utf8Error;

    #[inline]
    fn try_from(s: Symbol) -> Result<Self, Self::Error> {
        s.name().to_str()
    }
}

impl TryFrom<Symbol> for std::string::String {
    type Error = std::str::Utf8Error;

    #[inline]
    fn try_from(s: Symbol) -> Result<Self, Self::Error> {
        s.name().to_str().map(Into::into)
    }
}

impl Symbol {
    #[inline]
    pub(crate) fn _id(self) -> ruby::ID {
        unsafe { ruby::rb_sym2id(self.raw()) }
    }

    /// Returns an array of all the symbols currently in Ruby's symbol table.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// assert!(rosy::Symbol::all().contains("eql?"));
    /// ```
    #[inline]
    pub fn all() -> Array<Self> {
        unsafe { Array::from_raw(ruby::rb_sym_all_symbols()) }
    }

    /// Returns an array of the names of global variables.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// assert!(rosy::Symbol::global_vars().contains("$SAFE"));
    /// ```
    #[inline]
    pub fn global_vars() -> Array<Self> {
        unsafe { Array::from_raw(ruby::rb_f_global_variables()) }
    }

    /// Returns whether `name` is valid as a symbol value.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::Symbol;
    ///
    /// assert!(Symbol::is_valid("@hello"));
    ///
    /// assert!(!Symbol::is_valid("\0a"));
    /// assert!(!Symbol::is_valid("$"));
    /// assert!(!Symbol::is_valid("@"));
    /// assert!(!Symbol::is_valid(""));
    /// ```
    #[inline]
    pub fn is_valid(name: impl AsRef<[u8]>) -> bool {
        let name = name.as_ref();
        let ptr = name.as_ptr();
        let len = name.len();
        let enc = Encoding::utf8()._enc();
        unsafe { ruby::rb_enc_symname2_p(ptr as _, len as _, enc) != 0 }
    }

    /// Returns the identifier associated with this symbol.
    #[inline]
    pub fn id(self) -> SymbolId {
        SymbolId(self._id())
    }

    /// Returns the symbol's name as a nul-terminated C string.
    #[inline]
    pub fn name(self) -> &'static CStr {
        self.id().name()
    }
}

/// An identifier for a [`Symbol`](struct.Symbol.html).
///
/// # What is `Into<SymbolId>`?
///
/// This is a convenience trait implementation that allows for instantiating a
/// `SymbolId` from a Rust
/// [`&str`](https://doc.rust-lang.org/std/primitive.str.html) or
/// Ruby [`String`](../struct.String.html). With it, one can simply pass a
/// normal string literal into a function that takes some `impl Into<SymbolId>`.
/// For an example, look at [`Object::call`](../trait.Object.html#method.call).
#[derive(Clone, Copy)]
pub struct SymbolId(ruby::ID);

impl fmt::Debug for SymbolId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("SymbolId")
            .field(&self.raw())
            .finish()
    }
}

impl From<String> for SymbolId {
    #[inline]
    fn from(s: String) -> SymbolId {
        unsafe { SymbolId(ruby::rb_intern_str(s.raw())) }
    }
}

impl From<&str> for SymbolId {
    #[inline]
    fn from(s: &str) -> SymbolId {
        let raw = unsafe { ruby::rb_intern3(
            s.as_ptr() as _,
            s.len() as _,
            Encoding::utf8()._enc(),
        ) };
        SymbolId(raw)
    }
}

impl From<Symbol> for SymbolId {
    #[inline]
    fn from(s: Symbol) -> Self {
        SymbolId(s._id())
    }
}

impl From<SymbolId> for Symbol {
    #[inline]
    fn from(id: SymbolId) -> Symbol {
        unsafe { Symbol::from_raw(ruby::rb_id2sym(id.raw())) }
    }
}

impl SymbolId {
    /// Creates a symbol ID from the raw value.
    ///
    /// # Safety
    ///
    /// The value must have come from the Ruby VM.
    #[inline]
    pub const unsafe fn from_raw(raw: ruby::ID) -> Self {
        SymbolId(raw)
    }

    /// Returns the raw underlying ID.
    #[inline]
    pub const fn raw(self) -> ruby::ID {
        self.0
    }

    /// Returns the symbol's name as a nul-terminated C string.
    #[inline]
    pub fn name(self) -> &'static CStr {
        unsafe { CStr::from_ptr(ruby::rb_id2name(self.raw())) }
    }
}

macro_rules! common_ids {
    ($($name:ident => $sym:expr,)+) => {
        struct Common {
            $($name: SymbolId,)+
        }

        impl Common {
            // Initializes the `COMMON` table when first called and makes
            // subsequent calls simply access `COMMON` directly
            #[inline]
            fn get() -> &'static Common {
                static mut COMMON: Common = Common {
                    $($name: SymbolId(0),)+
                };

                static mut GET_COMMON: fn() -> &'static Common = || unsafe {
                    $(COMMON.$name = SymbolId::from($sym);)+

                    GET_COMMON = || &COMMON;

                    &COMMON
                };
                unsafe { GET_COMMON() }
            }
        }

        /// Commonly used symbol IDs.
        ///
        /// These functions are generally faster than using `SymbolId::from`.
        impl SymbolId {
            $(
                /// ID for the
                /// `
                #[doc = $sym]
                /// ` symbol.
                #[inline]
                pub fn $name() -> SymbolId {
                    Common::get().$name
                }
            )+
        }
    };
}

common_ids! {
    equal_op            => "==",
    backtrace           => "backtrace",
    cause               => "cause",
    size                => "size",
    eval                => "eval",
    to_binary           => "to_binary",
    load_from_binary    => "load_from_binary",
    disasm              => "disasm",
    path                => "path",
    absolute_path       => "absolute_path",
    include_q           => "include?",
    compile             => "compile",
    compile_file        => "compile_file",
}

#[cfg(all(test, nightly))]
mod benches {
    use test::{Bencher, black_box};
    use super::*;

    #[bench]
    fn get_equal_op_sym(b: &mut Bencher) {
        crate::vm::init().unwrap();

        b.iter(|| {
            black_box(black_box(SymbolId::equal_op)());
        });
    }

    #[bench]
    fn intern_equal_op_sym(b: &mut Bencher) {
        crate::vm::init().unwrap();

        b.iter(|| {
            black_box(SymbolId::from(black_box("==")));
        });
    }
}
