#![allow(dead_code, non_upper_case_globals, non_snake_case)]

mod prelude {
    pub use std::{
        ffi::c_void,
        os::raw::{c_char, c_int, c_uint, c_long},
    };
    pub use super::{VALUE, ID, Var};
}

mod array;
mod data;
mod exception;
mod float;
mod gc;
mod hash;
mod int;
mod mixin;
mod object;
mod range;
mod string;
mod symbol;
mod vm;

// `USE_FLONUM` is defined by `SIZEOF_VALUE >= SIZEOF_DOUBLE`
#[cfg(not(target_pointer_width = "32"))]
mod USE_FLONUM { // true
    use super::VALUE;

    #[inline]
    pub fn rb_flonum_p(x: VALUE) -> bool {
        (x & special_consts::FLONUM_MASK) == special_consts::FLONUM_FLAG
    }

    pub mod special_consts {
        pub const Qfalse: usize = 0x00; // ...0000 0000
        pub const Qtrue:  usize = 0x14; // ...0001 0100
        pub const Qnil:   usize = 0x08; // ...0000 1000
        pub const Qundef: usize = 0x34; // ...0011 0100

        pub const IMMEDIATE_MASK: usize = 0x07;
        pub const FIXNUM_FLAG:    usize = 0x01; // ...xxxx xxx1
        pub const FLONUM_MASK:    usize = 0x03;
        pub const FLONUM_FLAG:    usize = 0x02; // ...xxxx xx10
        pub const SYMBOL_FLAG:    usize = 0x0c; // ...0000 1100

        pub const SPECIAL_SHIFT: usize = 8;
    }
}

#[cfg(target_pointer_width = "32")]
mod USE_FLONUM { // false
    use super::VALUE;

    #[inline]
    pub fn rb_flonum_p(x: VALUE) -> bool {
        false
    }

    pub mod special_consts {
        pub const Qfalse: usize = 0; // ...0000 0000
        pub const Qtrue:  usize = 2; // ...0000 0010
        pub const Qnil:   usize = 4; // ...0000 0100
        pub const Qundef: usize = 6; // ...0000 0110

        pub const IMMEDIATE_MASK: usize = 0x03;
        pub const FIXNUM_FLAG:    usize = 0x01; // ...xxxx xxx1
        pub const FLONUM_MASK:    usize = 0x00; // any values ANDed with FLONUM_MASK cannot be FLONUM_FLAG
        pub const FLONUM_FLAG:    usize = 0x02;
        pub const SYMBOL_FLAG:    usize = 0x0e; // ...0000 1110

        pub const SPECIAL_SHIFT: usize = 8;
    }
}

pub mod fl_type {
    pub const FL_WB_PROTECTED: usize = 1 << 5;
    pub const FL_PROMOTED0:    usize = 1 << 5;
    pub const FL_PROMOTED1:    usize = 1 << 6;
    pub const FL_PROMOTED:     usize = FL_PROMOTED0 | FL_PROMOTED1;
    pub const FL_FINALIZE:     usize = 1 << 7;
    pub const FL_TAINT:        usize = 1 << 8;
    pub const FL_UNTRUSTED:    usize = FL_TAINT;
    pub const FL_EXIVAR:       usize = 1 << 10;
    pub const FL_FREEZE:       usize = 1 << 11;

    pub const FL_USHIFT: usize = 12;

    const fn fl_user(n: usize) -> usize {
        1 << (FL_USHIFT + n)
    }

    pub const FL_USER_0:  usize = fl_user(0);
    pub const FL_USER_1:  usize = fl_user(1);
    pub const FL_USER_2:  usize = fl_user(2);
    pub const FL_USER_3:  usize = fl_user(3);
    pub const FL_USER_4:  usize = fl_user(4);
    pub const FL_USER_5:  usize = fl_user(5);
    pub const FL_USER_6:  usize = fl_user(6);
    pub const FL_USER_7:  usize = fl_user(7);
    pub const FL_USER_8:  usize = fl_user(8);
    pub const FL_USER_9:  usize = fl_user(9);
    pub const FL_USER_10: usize = fl_user(10);
    pub const FL_USER_11: usize = fl_user(11);
    pub const FL_USER_12: usize = fl_user(12);
    pub const FL_USER_13: usize = fl_user(13);
    pub const FL_USER_14: usize = fl_user(14);
    pub const FL_USER_15: usize = fl_user(15);
    pub const FL_USER_16: usize = fl_user(16);
    pub const FL_USER_17: usize = fl_user(17);
    pub const FL_USER_18: usize = fl_user(18);
}

pub use self::{
    array::*,
    data::*,
    exception::*,
    float::*,
    gc::*,
    hash::*,
    int::*,
    mixin::*,
    object::*,
    range::*,
    string::*,
    symbol::*,
    vm::*,
    USE_FLONUM::*,
};

type OpaqueFn = Option<unsafe extern "C" fn ()>;

pub type VALUE = usize;

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum value_type {
    NONE   = 0x00,

    OBJECT   = 0x01,
    CLASS    = 0x02,
    MODULE   = 0x03,
    FLOAT    = 0x04,
    STRING   = 0x05,
    REGEXP   = 0x06,
    ARRAY    = 0x07,
    HASH     = 0x08,
    STRUCT   = 0x09,
    BIGNUM   = 0x0a,
    FILE     = 0x0b,
    DATA     = 0x0c,
    MATCH    = 0x0d,
    COMPLEX  = 0x0e,
    RATIONAL = 0x0f,

    NIL    = 0x11,
    TRUE   = 0x12,
    FALSE  = 0x13,
    SYMBOL = 0x14,
    FIXNUM = 0x15,
    UNDEF  = 0x16,

    IMEMO  = 0x1a,
    NODE   = 0x1b,
    ICLASS = 0x1c,
    ZOMBIE = 0x1d,

    MASK   = 0x1f,

    // Defined here to ensure that no other values conflict
    _Unknown = !0,
}

extern "C" {
    #[cfg_attr(dllimport, link_name="__imp_ruby_api_version")]
    pub static ruby_api_version:  Var<[prelude::c_int;  3]>;
    #[cfg_attr(dllimport, link_name="__imp_ruby_version")]
    pub static ruby_version:      Var<[prelude::c_char; 0]>;
    #[cfg_attr(dllimport, link_name="__imp_ruby_release_date")]
    pub static ruby_release_date: Var<[prelude::c_char; 0]>;
    #[cfg_attr(dllimport, link_name="__imp_ruby_platform")]
    pub static ruby_platform:     Var<[prelude::c_char; 0]>;
    #[cfg_attr(dllimport, link_name="__imp_ruby_patchlevel")]
    pub static ruby_patchlevel:   Var<[prelude::c_char; 0]>;
    #[cfg_attr(dllimport, link_name="__imp_ruby_description")]
    pub static ruby_description:  Var<[prelude::c_char; 0]>;
    #[cfg_attr(dllimport, link_name="__imp_ruby_copyright")]
    pub static ruby_copyright:    Var<[prelude::c_char; 0]>;
    #[cfg_attr(dllimport, link_name="__imp_ruby_engine")]
    pub static ruby_engine:       Var<[prelude::c_char; 0]>;
}

#[cfg(not(dllimport))]
mod var {
    use std::ops::Deref;

    #[repr(C)]
    pub struct Var<T> {
        inner: T,
    }

    impl<T> Var<T> {
        pub fn inner(&self) -> T where T: Copy {
            self.inner
        }
    }

    impl<T> Deref for Var<T> {
        type Target = T;

        fn deref(&self) -> &T {
            &self.inner
        }
    }
}

#[cfg(dllimport)]
mod var {
    // module for variables declared with __declspec(dllimport) on Windows
    use std::ops::Deref;

    #[repr(C)]
    pub struct Var<T> {
        inner: *const T,
    }

    impl<T> Var<T> {
        pub fn inner(&self) -> T where T: Copy {
            unsafe { *self.inner }
        }
    }

    impl<T> Deref for Var<T> {
        type Target = T;

        fn deref(&self) -> &T {
            unsafe { &*self.inner }
        }
    }
}
pub use var::Var;
