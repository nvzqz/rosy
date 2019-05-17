use super::prelude::*;

extern "C" {
    // VALUE rb_uint2inum(uintptr_t n)
    pub fn rb_uint2inum(n: usize) -> VALUE;
    // VALUE rb_int2inum(intptr_t n)
    pub fn rb_int2inum(n: isize) -> VALUE;
}
