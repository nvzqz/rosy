use crate::ruby::value_type::{self, *};

/// A Ruby virtual type.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum Ty {
    None     = NONE     as u8,
    Object   = OBJECT   as u8,
    Class    = CLASS    as u8,
    Module   = MODULE   as u8,
    Float    = FLOAT    as u8,
    String   = STRING   as u8,
    Regexp   = REGEXP   as u8,
    Array    = ARRAY    as u8,
    Hash     = HASH     as u8,
    Struct   = STRUCT   as u8,
    Bignum   = BIGNUM   as u8,
    File     = FILE     as u8,
    Data     = DATA     as u8,
    Match    = MATCH    as u8,
    Complex  = COMPLEX  as u8,
    Rational = RATIONAL as u8,
    Nil      = NIL      as u8,
    True     = TRUE     as u8,
    False    = FALSE    as u8,
    Symbol   = SYMBOL   as u8,
    Fixnum   = FIXNUM   as u8,
    Undef    = UNDEF    as u8,
    IMemo    = IMEMO    as u8,
    Node     = NODE     as u8,
    IClass   = ICLASS   as u8,
    Zombie   = ZOMBIE   as u8,
    _Mask    = MASK     as u8,
}

impl From<value_type> for Ty {
    #[inline]
    fn from(ty: value_type) -> Self {
        unsafe { std::mem::transmute(ty as u8) }
    }
}

impl From<Ty> for value_type {
    #[inline]
    fn from(ty: Ty) -> Self {
        unsafe { std::mem::transmute(ty as u32) }
    }
}
