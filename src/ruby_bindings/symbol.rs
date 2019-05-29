use super::{
    prelude::*,
    rb_encoding,
};

pub type ID = usize;

extern "C" {
    // ID rb_intern_str(VALUE str)
    pub fn rb_intern_str(str: VALUE) -> ID;
    // ID rb_sym2id(VALUE sym)
    pub fn rb_sym2id(sym: VALUE) -> ID;
    // int rb_enc_symname2_p(const char *name, long len, rb_encoding *enc)
    pub fn rb_enc_symname2_p(name: *const c_char, len: c_long, enc: *mut rb_encoding) -> c_int;
    // ID rb_intern3(const char *name, long len, rb_encoding *enc)
    pub fn rb_intern3(name: *const c_char, len: c_long, enc: *mut rb_encoding) -> ID;
    // VALUE rb_id2sym(ID x)
    pub fn rb_id2sym(x: ID) -> VALUE;
    // const char * rb_id2name(ID id)
    pub fn rb_id2name(id: ID) -> *const c_char;
    // VALUE rb_sym_all_symbols(void)
    pub fn rb_sym_all_symbols() -> VALUE;
    // VALUE rb_f_global_variables(void)
    pub fn rb_f_global_variables() -> VALUE;
}
