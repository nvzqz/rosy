use super::prelude::*;

extern "C" {
    // VALUE rb_hash_aref(VALUE hash, VALUE key)
    pub fn rb_hash_aref(hash: VALUE, key: VALUE) -> VALUE;
    // VALUE rb_hash_aset(VALUE hash, VALUE key, VALUE val)
    pub fn rb_hash_aset(hash: VALUE, key: VALUE, val: VALUE) -> VALUE;
    // void rb_hash_bulk_insert_into_st_table(long argc, const VALUE *argv, VALUE hash)
    pub fn rb_hash_bulk_insert_into_st_table(argc: c_long, argv: *const VALUE, hash: VALUE);
    // VALUE rb_hash_clear(VALUE hash)
    pub fn rb_hash_clear(hash: VALUE) -> VALUE;
    // VALUE rb_hash_delete(VALUE hash, VALUE key)
    pub fn rb_hash_delete(hash: VALUE, key: VALUE) -> VALUE;
    // VALUE rb_hash_new(void)
    pub fn rb_hash_new() -> VALUE;
    // size_t rb_hash_size_num(VALUE hash)
    pub fn rb_hash_size_num(hash: VALUE) -> usize;
    // VALUE rb_hash_dup(VALUE hash)
    pub fn rb_hash_dup(hash: VALUE) -> VALUE;
}
