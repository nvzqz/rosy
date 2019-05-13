use super::prelude::*;

extern "C" {
    // void rb_gc_adjust_memory_usage(ssize_t diff)
    pub fn rb_gc_adjust_memory_usage(diff: isize);
    // size_t rb_gc_count(void)
    pub fn rb_gc_count() -> usize;
    // VALUE rb_gc_disable(void)
    pub fn rb_gc_disable() -> VALUE;
    // VALUE rb_gc_enable(void)
    pub fn rb_gc_enable() -> VALUE;
    // void rb_gc_force_recycle(VALUE obj)
    pub fn rb_gc_force_recycle(obj: VALUE);
    // VALUE rb_gc_latest_gc_info(VALUE key)
    pub fn rb_gc_latest_gc_info(key: VALUE) -> VALUE;
    // VALUE rb_gc_start(void)
    pub fn rb_gc_start() -> VALUE;
    // size_t rb_gc_stat(VALUE key)
    pub fn rb_gc_stat(key: VALUE) -> usize;

    // void rb_gc_mark(VALUE ptr)
    pub fn rb_gc_mark(ptr: VALUE);
    // void rb_gc_mark_maybe(VALUE obj)
    pub fn rb_gc_mark_maybe(obj: VALUE);

    // void rb_gc_register_address(VALUE *addr)
    pub fn rb_gc_register_address(addr: *mut VALUE);
    // void rb_gc_register_mark_object(VALUE obj)
    pub fn rb_gc_register_mark_object(obj: VALUE);
    // void rb_gc_unregister_address(VALUE *addr)
    pub fn rb_gc_unregister_address(addr: *mut VALUE);
}
