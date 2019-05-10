use ruby::ruby_value_type::{self, *};

/// A Ruby virtual type.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum Ty {
    None     = RUBY_T_NONE as u8,
    Object   = RUBY_T_OBJECT as u8,
    Class    = RUBY_T_CLASS as u8,
    Module   = RUBY_T_MODULE as u8,
    Float    = RUBY_T_FLOAT as u8,
    String   = RUBY_T_STRING as u8,
    Regexp   = RUBY_T_REGEXP as u8,
    Array    = RUBY_T_ARRAY as u8,
    Hash     = RUBY_T_HASH as u8,
    Struct   = RUBY_T_STRUCT as u8,
    Bignum   = RUBY_T_BIGNUM as u8,
    File     = RUBY_T_FILE as u8,
    Data     = RUBY_T_DATA as u8,
    Match    = RUBY_T_MATCH as u8,
    Complex  = RUBY_T_COMPLEX as u8,
    Rational = RUBY_T_RATIONAL as u8,
    Nil      = RUBY_T_NIL as u8,
    True     = RUBY_T_TRUE as u8,
    False    = RUBY_T_FALSE as u8,
    Symbol   = RUBY_T_SYMBOL as u8,
    Fixnum   = RUBY_T_FIXNUM as u8,
    Undef    = RUBY_T_UNDEF as u8,
    IMemo    = RUBY_T_IMEMO as u8,
    Node     = RUBY_T_NODE as u8,
    IClass   = RUBY_T_ICLASS as u8,
    Zombie   = RUBY_T_ZOMBIE as u8,
    _Mask    = RUBY_T_MASK as u8,
}

impl From<ruby_value_type> for Ty {
    #[inline]
    fn from(ty: ruby_value_type) -> Self {
        unsafe { std::mem::transmute(ty as u8) }
    }
}

impl From<Ty> for ruby_value_type {
    #[inline]
    fn from(ty: Ty) -> Self {
        unsafe { std::mem::transmute(ty as u32) }
    }
}
