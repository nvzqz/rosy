use super::{
    prelude::*,
    RBasic,
    fl_type,
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RArray {
    pub basic: RBasic,
    pub as_: RArrayAs,
}

impl RArray {
    #[inline]
    fn embed_len(&self) -> usize {
        use rarray_flags::*;

        const MASK: usize = EMBED_LEN_MASK >> EMBED_LEN_SHIFT;
        MASK & (self.basic.flags >> EMBED_LEN_SHIFT)
    }

    #[inline]
    fn is_embedded(&self) -> bool {
        self.basic.flags & rarray_flags::EMBED_FLAG != 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        if self.is_embedded() {
            self.embed_len()
        } else {
            unsafe { self.as_.heap.len as usize }
        }
    }

    #[inline]
    pub fn start(&self) -> *const VALUE {
        if self.is_embedded() {
            unsafe { self.as_.ary.as_ptr() }
        } else {
            unsafe { self.as_.heap.ptr }
        }
    }

    #[inline]
    pub fn start_mut(&mut self) -> *mut VALUE {
        if self.is_embedded() {
            unsafe { self.as_.ary.as_mut_ptr() }
        } else {
            unsafe { self.as_.heap.ptr as *mut VALUE }
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union RArrayAs {
    pub heap: RArrayHeap,
    pub ary: [VALUE; rarray_flags::EMBED_LEN_MAX],
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RArrayHeap {
    pub len: c_long,
    pub ptr: *const VALUE,
    pub aux: RArrayHeapAux,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub union RArrayHeapAux {
    pub capa: c_long,
    pub shared: VALUE,
}

pub mod rarray_flags {
    use super::fl_type::*;

    pub const EMBED_LEN_MAX: usize = 3;
    pub const EMBED_FLAG: usize = FL_USER_1;
    pub const EMBED_LEN_MASK: usize = FL_USER_4 | FL_USER_3;
    pub const EMBED_LEN_SHIFT: usize = FL_USHIFT + 3;
}

extern "C" {
    // void rb_ary_modify(VALUE ary)
    pub fn rb_ary_modify(ary: VALUE);

    // VALUE rb_ary_cat(VALUE ary, const VALUE *argv, long len)
    pub fn rb_ary_cat(ary: VALUE, argv: *const VALUE, len: c_long) -> VALUE;
    // VALUE rb_ary_clear(VALUE ary)
    pub fn rb_ary_clear(ary: VALUE);
    // VALUE rb_ary_cmp(VALUE ary1, VALUE ary2)
    pub fn rb_ary_cmp(ary1: VALUE, ary2: VALUE) -> VALUE;
    // VALUE rb_ary_delete(VALUE ary, VALUE item)
    pub fn rb_ary_delete(ary: VALUE, item: VALUE) -> VALUE;
    // VALUE rb_ary_includes(VALUE ary, VALUE item)
    pub fn rb_ary_includes(ary: VALUE, item: VALUE) -> VALUE;
    // VALUE rb_ary_join(VALUE ary, VALUE sep)
    pub fn rb_ary_join(ary: VALUE, sep: VALUE) -> VALUE;
    // VALUE rb_ary_new(void)
    pub fn rb_ary_new() -> VALUE;
    // VALUE rb_ary_new_capa(long capa)
    pub fn rb_ary_new_capa(capa: c_long) -> VALUE;
    // VALUE rb_ary_new_from_values(long n, const VALUE *elts)
    pub fn rb_ary_new_from_values(n: c_long, elts: *const VALUE) -> VALUE;
    // VALUE rb_ary_plus(VALUE x, VALUE y)
    pub fn rb_ary_plus(x: VALUE, y: VALUE) -> VALUE;
    // VALUE rb_ary_pop(VALUE ary)
    pub fn rb_ary_pop(ary: VALUE) -> VALUE;
    // VALUE rb_ary_push(VALUE ary, VALUE item)
    pub fn rb_ary_push(ary: VALUE, item: VALUE) -> VALUE;
    // VALUE rb_ary_reverse(VALUE ary)
    pub fn rb_ary_reverse(ary: VALUE) -> VALUE;
    // VALUE rb_ary_sort(VALUE ary)
    pub fn rb_ary_sort(ary: VALUE) -> VALUE;
    // VALUE rb_ary_sort_bang(VALUE ary)
    pub fn rb_ary_sort_bang(ary: VALUE) -> VALUE;
}
