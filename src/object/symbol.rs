//! Ruby symbols.

use crate::object::{
    Object,
    AnyObject,
    string::{String, Encoding},
    Ty,
};
use std::{
    convert::TryFrom,
    ffi::CStr,
    fmt,
};

/// An instance of Ruby's `Symbol` class.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Symbol(AnyObject);

impl AsRef<AnyObject> for Symbol {
    #[inline]
    fn as_ref(&self) -> &AnyObject { &self.0 }
}

impl From<Symbol> for AnyObject {
    #[inline]
    fn from(object: Symbol) -> AnyObject { object.0 }
}

unsafe impl Object for Symbol {
    #[inline]
    fn cast(obj: impl Object) -> Option<Self> {
        if obj.is_ty(Ty::Symbol) {
            Some(Self::_new(obj.raw()))
        } else {
            None
        }
    }

    #[inline]
    fn ty(self) -> Ty { Ty::Symbol }

    #[inline]
    fn is_ty(self, ty: Ty) -> bool { ty == Ty::Symbol }
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
    pub(crate) fn _new(raw: ruby::VALUE) -> Self {
        Self(AnyObject(raw))
    }

    #[inline]
    pub(crate) fn _id(self) -> ruby::ID {
        unsafe { ruby::rb_sym2id(self.raw()) }
    }

    /// Returns whether `name` is valid as a symbol value.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::init().unwrap();
    /// use rosy::Symbol;
    ///
    /// assert!(Symbol::is_valid("@hello"));
    ///
    /// assert!(!Symbol::is_valid("$"));
    /// assert!(!Symbol::is_valid("@"));
    /// assert!(!Symbol::is_valid(""));
    /// ```
    #[inline]
    pub fn is_valid(name: impl AsRef<str>) -> bool {
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
        unsafe { Symbol::_new(ruby::rb_id2sym(id.raw())) }
    }
}

impl SymbolId {
    /// Returns the raw underlying ID.
    #[inline]
    pub fn raw(self) -> ruby::ID {
        self.0
    }

    /// Returns the symbol's name as a nul-terminated C string.
    #[inline]
    pub fn name(self) -> &'static CStr {
        unsafe { CStr::from_ptr(ruby::rb_id2name(self.raw())) }
    }
}
