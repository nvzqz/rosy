use super::{
    prelude::*,
    RBasic,
    fl_type,
};

pub const RUBY_TYPED_FREE_IMMEDIATELY: usize = 1;
pub const RUBY_FL_WB_PROTECTED: usize = fl_type::FL_WB_PROTECTED;
pub const RUBY_TYPED_PROMOTED1: usize = fl_type::FL_PROMOTED1;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RData {
    pub basic: RBasic,
    pub dmark: Option<unsafe extern "C" fn(*mut c_void)>,
    pub dfree: Option<unsafe extern "C" fn(*mut c_void)>,
    pub data: *mut c_void,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct rb_data_type_t {
    pub wrap_struct_name: *const c_char,
    pub function: rb_data_type_t_function,
    pub parent: *const rb_data_type_t,
    pub data: *mut c_void,
    pub flags: VALUE,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct rb_data_type_t_function {
    pub dmark: Option<unsafe extern "C" fn(*mut c_void)>,
    pub dfree: Option<unsafe extern "C" fn(*mut c_void)>,
    pub dsize: Option<unsafe extern "C" fn(*const c_void) -> usize>,
    pub reserved: [*mut c_void; 2],
}

extern "C" {
    // VALUE rb_data_typed_object_wrap(VALUE klass, void *datap, const rb_data_type_t *type)
    pub fn rb_data_typed_object_wrap(klass: VALUE, datap: *mut c_void, ty: *const rb_data_type_t) -> VALUE;
}
