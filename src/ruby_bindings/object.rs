use std::ptr;
use super::prelude::*;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RBasic {
    pub flags: VALUE,
    pub klass: VALUE,
}

impl RBasic {
    // Used to ensure the read doesn't get optimized out
    #[inline]
    pub fn volatile_flags(&self) -> VALUE {
        unsafe { ptr::read_volatile(&self.flags) }
    }
}

extern "C" {
    // int rb_eql(VALUE obj1, VALUE obj2)
    pub fn rb_eql(obj1: VALUE, obj2: VALUE) -> c_int;

    // VALUE rb_funcallv(VALUE recv, ID mid, int argc, const VALUE *argv);
    pub fn rb_funcallv(recv: VALUE, mid: ID, argc: c_int, argv: *const VALUE) -> VALUE;
    // VALUE rb_funcallv_public(VALUE recv, ID mid, int argc, const VALUE *argv)
    pub fn rb_funcallv_public(recv: VALUE, mid: ID, argc: c_int, argv: *const VALUE) -> VALUE;

    // VALUE rb_inspect(VALUE obj)
    pub fn rb_inspect(obj: VALUE) -> VALUE;
    // VALUE rb_obj_as_string(VALUE obj)
    pub fn rb_obj_as_string(obj: VALUE) -> VALUE;

    // VALUE rb_obj_class(VALUE obj)
    pub fn rb_obj_class(obj: VALUE) -> VALUE;
    // VALUE rb_obj_id(VALUE obj)
    pub fn rb_obj_id(obj: VALUE) -> VALUE;

    // VALUE rb_obj_freeze(VALUE obj)
    pub fn rb_obj_freeze(obj: VALUE) -> VALUE;
    // VALUE rb_obj_frozen_p(VALUE obj)
    pub fn rb_obj_frozen_p(obj: VALUE) -> VALUE;

    // VALUE rb_singleton_class(VALUE obj)
    pub fn rb_singleton_class(obj: VALUE) -> VALUE;
}
