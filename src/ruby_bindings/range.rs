use super::prelude::*;

extern "C" {
    // VALUE rb_range_new(VALUE beg, VALUE end, int exclude_end)
    pub fn rb_range_new(beg: VALUE, end: VALUE, exclude_end: c_int) -> VALUE;
    // int rb_range_values(VALUE range, VALUE *begp, VALUE *endp, int *exclp)
    pub fn rb_range_values(range: VALUE, begp: *mut VALUE, endp: *mut VALUE, exclp: *mut c_int) -> c_int;
}
