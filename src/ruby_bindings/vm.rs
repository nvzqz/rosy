use super::prelude::*;

extern "C" {
    // int ruby_cleanup(volatile int ex)
    pub fn ruby_cleanup(ex: c_int) -> c_int;
    // void ruby_init_loadpath(void)
    pub fn ruby_init_loadpath();
    // int ruby_setup(void)
    pub fn ruby_setup() -> c_int;
}
