use std::os::raw::c_long;
use crate::{
    object::{AnyObject, Ty},
    ruby::{
        self,
        RBasic,
        VALUE,
        value_type,
        special_consts::*,
    }
};

pub const NIL_VALUE:   VALUE = Qnil   as VALUE;
pub const TRUE_VALUE:  VALUE = Qtrue  as VALUE;
pub const FALSE_VALUE: VALUE = Qfalse as VALUE;
pub const UNDEF_VALUE: VALUE = Qundef as VALUE;

pub const MAX_VALUE: VALUE = !0;
pub const MAX_VALUE_SHIFTED: VALUE = MAX_VALUE << SPECIAL_SHIFT;

#[inline]
pub fn matches_ruby_size_align<T>() -> bool {
    use std::mem::{align_of, size_of};
    size_of::<T>()  == size_of::<VALUE>() &&
    align_of::<T>() == align_of::<VALUE>()
}

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

pub fn value_type(v: VALUE) -> value_type {
    match v {
        NIL_VALUE => value_type::NIL,
        TRUE_VALUE => value_type::TRUE,
        FALSE_VALUE => value_type::FALSE,
        UNDEF_VALUE => value_type::UNDEF,
        _ => if value_is_sym(v) {
            value_type::SYMBOL
        } else if value_is_float(v) {
            value_type::FLOAT
        } else if value_is_fixnum(v) {
            value_type::FIXNUM
        } else if let Some(built_in) = value_built_in_type(v) {
            built_in
        } else {
            debug_assert!(
                false,
                "Unknown type of {:?} (raw: {})",
                unsafe { AnyObject::from_raw(v) },
                v,
            );
            value_type::NONE
        }
    }
}

#[inline]
pub unsafe fn value_flags(v: VALUE) -> VALUE {
    (*(v as *const RBasic)).flags
}

#[inline]
pub unsafe fn value_built_in_type_unchecked(v: VALUE) -> value_type {
    std::mem::transmute((value_flags(v) & value_type::MASK as VALUE) as u32)
}

#[inline]
pub fn value_built_in_type(v: VALUE) -> Option<value_type> {
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
pub fn value_is_built_in_type(v: VALUE, t: value_type) -> bool {
    value_built_in_type(v) == Some(t)
}

#[inline]
pub fn value_flag(v: VALUE) -> VALUE {
    v & !MAX_VALUE_SHIFTED
}

#[inline]
pub fn fixnum_to_value(i: c_long) -> VALUE {
    ((i as VALUE) << 1) | FIXNUM_FLAG
}

#[inline]
pub fn value_to_fixnum(v: VALUE) -> c_long {
    ((v & !(FIXNUM_FLAG as VALUE)) >> 1) as c_long
}

#[inline]
pub fn value_is_fixnum(v: VALUE) -> bool {
    v & FIXNUM_FLAG as VALUE != 0
}

#[inline]
pub fn value_is_float(v: VALUE) -> bool {
    ruby::rb_flonum_p(v) || value_is_built_in_type(v, value_type::FLOAT)
}

#[inline]
pub fn value_is_immediate(v: VALUE) -> bool {
    v & IMMEDIATE_MASK as VALUE != 0
}

#[inline]
pub fn value_is_special_const(v: VALUE) -> bool {
    value_is_immediate(v) || !test_value(v)
}

#[inline]
pub fn value_is_static_sym(v: VALUE) -> bool {
    value_flag(v) == SYMBOL_FLAG as VALUE
}

#[inline]
pub fn value_is_dyn_sym(v: VALUE) -> bool {
    value_is_built_in_type(v, value_type::SYMBOL)
}

#[inline]
pub fn value_is_sym(v: VALUE) -> bool {
    value_is_static_sym(v) || value_is_dyn_sym(v)
}

#[inline]
pub fn value_is_class(v: VALUE) -> bool {
    value_is_built_in_type(v, value_type::CLASS)
}

#[inline]
pub fn value_is_module(v: VALUE) -> bool {
    value_is_built_in_type(v, value_type::MODULE)
}

pub trait Sealed {}
