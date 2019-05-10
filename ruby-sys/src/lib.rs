#![allow(warnings)]

include!(env!("ROSY_RUBY_VERSION_CONST"));
include!(env!("ROSY_BINDINGS_PATH"));

extern "C" {
    // Defined in `vm_core.h`
    pub static mut rb_cRubyVM: VALUE;
    pub static mut rb_cISeq: VALUE;
}
