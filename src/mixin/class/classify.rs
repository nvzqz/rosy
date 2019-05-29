use crate::{
    prelude::*,
    string::Encoding,
    vm::InstrSeq,
};

/// A type that can be instantiated from a typed `Class` instance.
pub trait Classify: Object {
    /// Returns the typed class that can be used to get an instance of `self`.
    fn class() -> Class<Self>;
}

macro_rules! impl_trait {
    ($($t:ty, $c:ident ;)+) => { $(
        impl Classify for $t {
            #[inline]
            fn class() -> Class<Self> {
                unsafe { Class::cast_unchecked(Class::$c()) }
            }
        }
    )+ };
}

impl<O: Object> Classify for Array<O> {
    #[inline]
    fn class() -> Class<Self> {
        unsafe { Class::cast_unchecked(Class::array()) }
    }
}

impl<K: Object, V: Object> Classify for Hash<K, V> {
    #[inline]
    fn class() -> Class<Self> {
        unsafe { Class::cast_unchecked(Class::hash()) }
    }
}

impl_trait! {
    AnyObject,    object;
    Class,        class;
    Module,       module;
    Integer,      integer;
    String,       string;
    Symbol,       symbol;
    Encoding,     encoding;
    AnyException, exception;
    InstrSeq,     instr_seq;
}
