use std::ptr;
use super::{
    fl_type,
    OpaqueFn,
    prelude::*,
    RBasic,
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RString {
    pub basic: RBasic,
    pub as_: RStringAs,
}

impl RString {
    #[inline]
    fn embed_len(&self) -> usize {
        use rstring_flags::*;

        const MASK: usize = EMBED_LEN_MASK >> EMBED_LEN_SHIFT;
        MASK & (self.basic.volatile_flags() >> EMBED_LEN_SHIFT)
    }

    #[inline]
    fn is_embedded(&self) -> bool {
        self.basic.volatile_flags() & rstring_flags::NO_EMBED == 0
    }

    #[inline]
    pub fn is_locked(&self) -> bool {
        self.basic.volatile_flags() & STR_TMPLOCK != 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        if self.is_embedded() {
            self.embed_len()
        } else {
            unsafe { ptr::read_volatile(&self.as_.heap.len) as usize }
        }
    }

    #[inline]
    pub fn start(&self) -> *const c_char {
        if self.is_embedded() {
            unsafe { self.as_.ary.as_ptr() }
        } else {
            unsafe { ptr::read_volatile(&self.as_.heap.ptr) }
        }
    }

    #[inline]
    pub fn start_mut(&mut self) -> *mut c_char {
        if self.is_embedded() {
            unsafe { self.as_.ary.as_mut_ptr() }
        } else {
            unsafe { ptr::read_volatile(&self.as_.heap.ptr) as *mut c_char }
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union RStringAs {
    pub heap: RStringHeap,
    pub ary: [c_char; rstring_flags::EMBED_LEN_MAX + 1],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RStringHeap {
    pub len: c_long,
    pub ptr: *const c_char,
    pub aux: RStringHeapAux,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union RStringHeapAux {
    pub capa: c_long,
    pub shared: VALUE,
}

#[repr(C)]
pub struct rb_encoding {
    _precise_mbc_enc_len: OpaqueFn,
    pub name: *const c_char,
    pub max_enc_len: c_int,
    pub min_enc_len: c_int,
    _is_mbc_newline: OpaqueFn,
    _mbc_to_code: OpaqueFn,
    _code_to_mbclen: OpaqueFn,
    _code_to_mbc: OpaqueFn,
    _mbc_case_fold: OpaqueFn,
    _apply_all_case_fold: OpaqueFn,
    _get_case_fold_codes_by_str: OpaqueFn,
    _property_name_to_ctype: OpaqueFn,
    _is_code_ctype: OpaqueFn,
    _get_ctype_code_range: OpaqueFn,
    _left_adjust_char_head: OpaqueFn,
    _is_allowed_reverse_match: OpaqueFn,
    _case_map: OpaqueFn,
    pub ruby_encoding_index: c_int,
    pub flags: c_uint,
}

impl rb_encoding {
    #[inline]
    pub fn ascii_8bit_index() -> c_int {
        let index = preserved_encindex::ASCII as c_int;
        debug_assert_eq!(index, unsafe { rb_ascii8bit_encindex() });
        index
    }

    #[inline]
    pub fn utf8_index() -> c_int {
        let index = preserved_encindex::UTF_8 as c_int;
        debug_assert_eq!(index, unsafe { rb_utf8_encindex() });
        index
    }

    #[inline]
    pub fn us_ascii_index() -> c_int {
        let index = preserved_encindex::US_ASCII as c_int;
        debug_assert_eq!(index, unsafe { rb_usascii_encindex() });
        index
    }

    #[inline]
    pub fn index(&mut self) -> c_int {
        let index = self.ruby_encoding_index & ENC_INDEX_MASK;
        debug_assert_eq!(index, unsafe { rb_enc_to_index(self) });
        index
    }
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub enum preserved_encindex {
    ASCII,
    UTF_8,
    US_ASCII,

    /* preserved indexes */
    UTF_16BE,
    UTF_16LE,
    UTF_32BE,
    UTF_32LE,
    UTF_16,
    UTF_32,
    UTF_8_MAC,

    /* for old options of regexp */
    EUC_JP,
    Windows_31J,

    BUILTIN_MAX,
}

pub const ENC_INDEX_MASK: c_int = !(!(0 as c_uint) << 24) as c_int;

pub const STR_TMPLOCK: VALUE = fl_type::FL_USER_7;

pub mod rstring_flags {
    use std::mem::size_of;
    use super::{*, fl_type::*};

    pub const NO_EMBED: usize = FL_USER_1;

    pub const EMBED_LEN_MASK: usize = FL_USER_2 | FL_USER_3 | FL_USER_4 | FL_USER_5 | FL_USER_6;
    pub const EMBED_LEN_SHIFT: usize = FL_USHIFT + 2;
    pub const EMBED_LEN_MAX: usize = (size_of::<VALUE>() * 3) / size_of::<c_char>() - 1;

    pub const FSTR: usize = FL_USER_17;
}

// Taken from `ruby_encoding_consts` in 'encoding.h'
pub mod encoding_consts {
    pub const INLINE_MAX: usize = 127;
    pub const SHIFT: usize = super::fl_type::FL_USHIFT + 10;
    pub const MASK: usize = INLINE_MAX << SHIFT;
    pub const MAX_NAME_LEN: usize = 42;
}

impl RBasic {
    // Taken from `RB_ENCODING_GET_INLINED(obj)` in 'encoding.h'
    #[inline]
    pub fn encoding_index(&self) -> c_int {
        use encoding_consts::*;
        ((self.flags & MASK) >> SHIFT) as c_int
    }
}

extern "C" {
    // VALUE rb_str_buf_new(long capa)
    pub fn rb_str_buf_new(capa: c_long) -> VALUE;
    // VALUE rb_external_str_new_with_enc(const char *ptr, long len, rb_encoding *eenc)
    pub fn rb_external_str_new_with_enc(ptr: *const c_char, len: c_long, enc: *mut rb_encoding) -> VALUE;

    // VALUE rb_str_cat(VALUE str, const char *ptr, long len)
    pub fn rb_str_cat(str: VALUE, ptr: *const c_char, len: c_long) -> VALUE;
    // int rb_str_cmp(VALUE str1, VALUE str2)
    pub fn rb_str_cmp(str1: VALUE, str2: VALUE) -> c_int;
    // VALUE rb_str_dup(VALUE str)
    pub fn rb_str_dup(str: VALUE) -> VALUE;
    // VALUE rb_str_ellipsize(VALUE str, long len)
    pub fn rb_str_ellipsize(str: VALUE, len: c_long) -> VALUE;
    // VALUE rb_str_equal(VALUE str1, VALUE str2)
    pub fn rb_str_equal(str1: VALUE, str2: VALUE) -> VALUE;
    // VALUE rb_str_new(const char *ptr, long len)
    pub fn rb_str_new(ptr: *const c_char, len: c_long) -> VALUE;
    // VALUE rb_utf8_str_new(const char *ptr, long len)
    pub fn rb_utf8_str_new(ptr: *const c_char, len: c_long) -> VALUE;
    // long rb_str_strlen(VALUE str)
    pub fn rb_str_strlen(str: VALUE) -> c_long;

    // VALUE rb_str_locktmp(VALUE str)
    pub fn rb_str_locktmp(str: VALUE) -> VALUE;
    // VALUE rb_str_unlocktmp(VALUE str)
    pub fn rb_str_unlocktmp(str: VALUE) -> VALUE;
}

// Encoding
extern "C" {
    // rb_encoding * rb_default_external_encoding(void)
    pub fn rb_default_external_encoding() -> *mut rb_encoding;
    // rb_encoding * rb_default_internal_encoding(void)
    pub fn rb_default_internal_encoding() -> *mut rb_encoding;

    // VALUE rb_enc_associate_index(VALUE obj, int idx)
    pub fn rb_enc_associate_index(obj: VALUE, idx: c_int) -> VALUE;

    // int rb_enc_find_index(const char *name)
    pub fn rb_enc_find_index(name: *const c_char) -> c_int;
    // VALUE rb_enc_from_encoding(rb_encoding *encoding)
    pub fn rb_enc_from_encoding(encoding: *mut rb_encoding) -> VALUE;
    // rb_encoding * rb_enc_from_index(int index)
    pub fn rb_enc_from_index(index: c_int) -> *mut rb_encoding;
    // int rb_enc_get_index(VALUE obj)
    pub fn rb_enc_get_index(obj: VALUE) -> c_int;
    // int rb_enc_to_index(rb_encoding *enc)
    pub fn rb_enc_to_index(enc: *mut rb_encoding) -> c_int;

    // int rb_filesystem_encindex(void)
    pub fn rb_filesystem_encindex() -> c_int;
    // int rb_locale_encindex(void)
    pub fn rb_locale_encindex() -> c_int;
    // int rb_ascii8bit_encindex(void)
    pub fn rb_ascii8bit_encindex() -> c_int;
    // rb_encoding * rb_ascii8bit_encoding(void)
    pub fn rb_ascii8bit_encoding() -> *mut rb_encoding;
    // int rb_usascii_encindex(void)
    pub fn rb_usascii_encindex() -> c_int;
    // rb_encoding * rb_usascii_encoding(void)
    pub fn rb_usascii_encoding() -> *mut rb_encoding;
    // int rb_utf8_encindex(void)
    pub fn rb_utf8_encindex() -> c_int;
    // rb_encoding * rb_utf8_encoding(void)
    pub fn rb_utf8_encoding() -> *mut rb_encoding;

    // rb_encoding * rb_to_encoding(VALUE enc)
    pub fn rb_to_encoding(enc: VALUE) -> *mut rb_encoding;
}
