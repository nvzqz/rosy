use super::prelude::*;

extern "C" {
    // int ruby_cleanup(volatile int ex)
    pub fn ruby_cleanup(ex: c_int) -> c_int;
    // void ruby_init_loadpath(void)
    pub fn ruby_init_loadpath();
    // int ruby_setup(void)
    pub fn ruby_setup() -> c_int;

    // int rb_safe_level(void)
    pub fn rb_safe_level() -> c_int;
    // void rb_set_safe_level(int level)
    pub fn rb_set_safe_level(level: c_int);

    // VALUE rb_require_safe(VALUE fname, int safe)
    pub fn rb_require_safe(fname: VALUE, safe: c_int) -> VALUE;

    // VALUE rb_eval_string(const char *str)
    pub fn rb_eval_string(str: *const c_char) -> VALUE;
    // VALUE rb_eval_string_protect(const char *str, int *pstate)
    pub fn rb_eval_string_protect(str: *const c_char, pstate: *mut c_int) -> VALUE;
    // VALUE rb_eval_string_wrap(const char *str, int *pstate)
    pub fn rb_eval_string_wrap(str: *const c_char, pstate: *mut c_int) -> VALUE;
}
