use std::os::raw::{c_int, c_long};
use ruby::{VALUE, ruby_value_type::{self, *}, ruby_special_consts::*};
use crate::{Object, AnyObject, Ty};

extern "C" {
    // Defined in `wrapper.h`
    fn rb_float_type_p(v: VALUE) -> c_int;
}

pub const NIL_VALUE:   VALUE = RUBY_Qnil   as VALUE;
pub const TRUE_VALUE:  VALUE = RUBY_Qtrue  as VALUE;
pub const FALSE_VALUE: VALUE = RUBY_Qfalse as VALUE;
pub const UNDEF_VALUE: VALUE = RUBY_Qundef as VALUE;

pub const MAX_VALUE: VALUE = !0;
pub const MAX_VALUE_SHIFTED: VALUE = MAX_VALUE << RUBY_SPECIAL_SHIFT;

#[inline]
pub fn test_value(v: VALUE) -> bool {
    v & !NIL_VALUE != 0
}

pub fn value_is_ty(v: VALUE, ty: Ty) -> bool {
    match ty {
        Ty::Fixnum => value_is_fixnum(v),
        Ty::Float  => value_is_float(v),
        Ty::Symbol => value_is_sym(v),
        Ty::True   => v == TRUE_VALUE,
        Ty::False  => v == FALSE_VALUE,
        Ty::Nil    => v == NIL_VALUE,
        Ty::Undef  => v == UNDEF_VALUE,
        _ => if let Some(t) = value_built_in_type(v) {
            t as u32 == ty as u32
        } else {
            false
        }
    }
}

pub fn value_type(v: VALUE) -> ruby_value_type {
    match v {
        NIL_VALUE => RUBY_T_NIL,
        TRUE_VALUE => RUBY_T_TRUE,
        FALSE_VALUE => RUBY_T_FALSE,
        UNDEF_VALUE => RUBY_T_UNDEF,
        _ => if value_is_sym(v) {
            RUBY_T_SYMBOL
        } else if value_is_float(v) {
            RUBY_T_FLOAT
        } else if value_is_fixnum(v) {
            RUBY_T_FIXNUM
        } else if let Some(built_in) = value_built_in_type(v) {
            built_in
        } else {
            debug_assert!(
                false,
                "Unknown type of {:?} (raw: {})",
                unsafe { AnyObject::from_raw(v).inspect() },
                v,
            );
            RUBY_T_NONE
        }
    }
}

#[inline]
pub unsafe fn value_flags(v: VALUE) -> VALUE {
    (*(v as *const ruby::RBasic)).flags
}

#[inline]
pub unsafe fn value_built_in_type_unchecked(v: VALUE) -> ruby_value_type {
    std::mem::transmute((value_flags(v) & RUBY_T_MASK as VALUE) as u32)
}

#[inline]
pub fn value_built_in_type(v: VALUE) -> Option<ruby_value_type> {
    if value_is_special_const(v) {
        None
    } else {
        unsafe { Some(value_built_in_type_unchecked(v)) }
    }
}

#[inline]
pub fn value_is_built_in_ty(v: VALUE, ty: Ty) -> bool {
    value_is_built_in_type(v, ty.into())
}

#[inline]
pub fn value_is_built_in_type(v: VALUE, t: ruby_value_type) -> bool {
    value_built_in_type(v) == Some(t)
}

#[inline]
pub fn value_flag(v: VALUE) -> VALUE {
    v & !MAX_VALUE_SHIFTED
}

#[inline]
pub fn value_to_fixnum(v: VALUE) -> c_long {
    ((v & !(RUBY_FIXNUM_FLAG as VALUE)) >> 1) as c_long
}

#[inline]
pub fn value_is_fixnum(v: VALUE) -> bool {
    v & RUBY_FIXNUM_FLAG as VALUE != 0
}

#[inline]
pub fn value_is_float(v: VALUE) -> bool {
    unsafe { rb_float_type_p(v) != 0 }
}

#[inline]
pub fn value_is_immediate(v: VALUE) -> bool {
    v & RUBY_IMMEDIATE_MASK as VALUE != 0
}

#[inline]
pub fn value_is_special_const(v: VALUE) -> bool {
    value_is_immediate(v) || !test_value(v)
}

#[inline]
pub fn value_is_static_sym(v: VALUE) -> bool {
    value_flag(v) == RUBY_SYMBOL_FLAG as VALUE
}

#[inline]
pub fn value_is_dyn_sym(v: VALUE) -> bool {
    value_is_built_in_type(v, RUBY_T_SYMBOL)
}

#[inline]
pub fn value_is_sym(v: VALUE) -> bool {
    value_is_static_sym(v) || value_is_dyn_sym(v)
}

#[inline]
pub fn value_is_class(v: VALUE) -> bool {
    value_is_built_in_type(v, RUBY_T_CLASS)
}

#[inline]
pub fn value_is_module(v: VALUE) -> bool {
    value_is_built_in_type(v, RUBY_T_MODULE)
}

pub trait Sealed {}
