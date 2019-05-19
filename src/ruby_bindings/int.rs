use super::prelude::*;

pub mod integer_flags {
    use super::*;

    pub const PACK_MSWORD_FIRST:                 c_int = 0x01;
    pub const PACK_LSWORD_FIRST:                 c_int = 0x02;
    pub const PACK_MSBYTE_FIRST:                 c_int = 0x10;
    pub const PACK_LSBYTE_FIRST:                 c_int = 0x20;
    pub const PACK_NATIVE_BYTE_ORDER:            c_int = 0x40;
    pub const PACK_2COMP:                        c_int = 0x80; // 2's compliment
    pub const PACK_FORCE_GENERIC_IMPLEMENTATION: c_int = 0x400;

    // For rb_integer_unpack:
    pub const PACK_FORCE_BIGNUM: c_int = 0x100;
    pub const PACK_NEGATIVE:     c_int = 0x200;

    // Combinations
    pub const PACK_LITTLE_ENDIAN: c_int = PACK_LSWORD_FIRST | PACK_LSBYTE_FIRST;
    pub const PACK_BIG_ENDIAN:    c_int = PACK_MSWORD_FIRST | PACK_MSBYTE_FIRST;
}

extern "C" {
    // VALUE rb_uint2inum(uintptr_t n)
    pub fn rb_uint2inum(n: usize) -> VALUE;
    // VALUE rb_int2inum(intptr_t n)
    pub fn rb_int2inum(n: isize) -> VALUE;

    // VALUE rb_integer_unpack(const void *words, size_t numwords, size_t wordsize, size_t nails, int flags)
    pub fn rb_integer_unpack(
        words: *const c_void,
        numwords: usize,
        wordsize: usize,
        nails: usize,
        flags: c_int,
    ) -> VALUE;
}
