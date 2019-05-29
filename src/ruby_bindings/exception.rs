use super::prelude::*;

extern "C" {
    // VALUE rb_errinfo(void)
    pub fn rb_errinfo() -> VALUE;
    // void rb_set_errinfo(VALUE err)
    pub fn rb_set_errinfo(err: VALUE);

    // NORETURN(void rb_exc_raise(VALUE mesg))
    pub fn rb_exc_raise(mesg: VALUE) -> !;
    // VALUE rb_protect(VALUE (* proc) (VALUE), VALUE data, int *pstate)
    pub fn rb_protect(
        proc: Option<unsafe extern "C" fn(VALUE) -> VALUE>,
        data: VALUE,
        pstate: *mut c_int,
    ) -> VALUE;
}
