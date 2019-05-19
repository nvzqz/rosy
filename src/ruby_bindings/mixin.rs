use super::prelude::*;

extern "C" {
    pub static rb_mKernel: VALUE;
    pub static rb_mComparable: VALUE;
    pub static rb_mEnumerable: VALUE;
    pub static rb_mErrno: VALUE;
    pub static rb_mFileTest: VALUE;
    pub static rb_mGC: VALUE;
    pub static rb_mMath: VALUE;
    pub static rb_mProcess: VALUE;
    pub static rb_mWaitReadable: VALUE;
    pub static rb_mWaitWritable: VALUE;

    pub static rb_cBasicObject: VALUE;
    pub static rb_cObject: VALUE;
    pub static rb_cArray: VALUE;
    pub static rb_cBinding: VALUE;
    pub static rb_cClass: VALUE;
    pub static rb_cCont: VALUE;
    pub static rb_cData: VALUE;
    pub static rb_cDir: VALUE;
    pub static rb_cEncoding: VALUE;
    pub static rb_cEnumerator: VALUE;
    pub static rb_cFalseClass: VALUE;
    pub static rb_cFile: VALUE;
    pub static rb_cComplex: VALUE;
    pub static rb_cFloat: VALUE;
    pub static rb_cHash: VALUE;
    pub static rb_cIO: VALUE;
    pub static rb_cInteger: VALUE;
    pub static rb_cMatch: VALUE;
    pub static rb_cMethod: VALUE;
    pub static rb_cModule: VALUE;
    pub static rb_cNameErrorMesg: VALUE;
    pub static rb_cNilClass: VALUE;
    pub static rb_cNumeric: VALUE;
    pub static rb_cProc: VALUE;
    pub static rb_cRandom: VALUE;
    pub static rb_cRange: VALUE;
    pub static rb_cRational: VALUE;
    pub static rb_cRegexp: VALUE;
    pub static rb_cStat: VALUE;
    pub static rb_cString: VALUE;
    pub static rb_cStruct: VALUE;
    pub static rb_cSymbol: VALUE;
    pub static rb_cThread: VALUE;
    pub static rb_cTime: VALUE;
    pub static rb_cTrueClass: VALUE;
    pub static rb_cUnboundMethod: VALUE;

    // Found in 'vm_core.h'
    pub static rb_cRubyVM: VALUE;
    pub static rb_cISeq: VALUE;

    pub static rb_eException: VALUE;
    pub static rb_eStandardError: VALUE;
    pub static rb_eSystemExit: VALUE;
    pub static rb_eInterrupt: VALUE;
    pub static rb_eSignal: VALUE;
    pub static rb_eFatal: VALUE;
    pub static rb_eArgError: VALUE;
    pub static rb_eEOFError: VALUE;
    pub static rb_eIndexError: VALUE;
    pub static rb_eStopIteration: VALUE;
    pub static rb_eKeyError: VALUE;
    pub static rb_eRangeError: VALUE;
    pub static rb_eIOError: VALUE;
    pub static rb_eRuntimeError: VALUE;
    pub static rb_eFrozenError: VALUE;
    pub static rb_eSecurityError: VALUE;
    pub static rb_eSystemCallError: VALUE;
    pub static rb_eThreadError: VALUE;
    pub static rb_eTypeError: VALUE;
    pub static rb_eZeroDivError: VALUE;
    pub static rb_eNotImpError: VALUE;
    pub static rb_eNoMemError: VALUE;
    pub static rb_eNoMethodError: VALUE;
    pub static rb_eFloatDomainError: VALUE;
    pub static rb_eLocalJumpError: VALUE;
    pub static rb_eSysStackError: VALUE;
    pub static rb_eRegexpError: VALUE;
    pub static rb_eEncodingError: VALUE;
    pub static rb_eEncCompatError: VALUE;
    pub static rb_eScriptError: VALUE;
    pub static rb_eNameError: VALUE;
    pub static rb_eSyntaxError: VALUE;
    pub static rb_eLoadError: VALUE;
    pub static rb_eMathDomainError: VALUE;

    // void rb_attr(VALUE klass, ID id, int read, int write, int ex)
    pub fn rb_attr(klass: VALUE, id: ID, read: c_int, write: c_int, ex: c_int);

    // VALUE rb_call_super(int argc, const VALUE *argv)
    pub fn rb_call_super(argc: c_int, argv: *const VALUE) -> VALUE;

    // VALUE rb_class_inherited_p(VALUE mod, VALUE arg)
    pub fn rb_class_inherited_p(r#mod: VALUE, arg: VALUE) -> VALUE;
    // VALUE rb_class_name(VALUE klass)
    pub fn rb_class_name(klass: VALUE) -> VALUE;
    // VALUE rb_class_new_instance(int argc, const VALUE *argv, VALUE klass)
    pub fn rb_class_new_instance(argc: c_int, argv: *const VALUE, klass: VALUE) -> VALUE;
    // VALUE rb_class_superclass(VALUE klass)
    pub fn rb_class_superclass(klass: VALUE) -> VALUE;

    // int rb_const_defined(VALUE klass, ID id)
    pub fn rb_const_defined(klass: VALUE, id: ID) -> c_int;
    // VALUE rb_const_get(VALUE klass, ID id)
    pub fn rb_const_get(klass: VALUE, id: ID) -> VALUE;
    // VALUE rb_const_remove(VALUE klass, ID id)
    pub fn rb_const_remove(klass: VALUE, id: ID) -> VALUE;
    // void rb_const_set(VALUE klass, ID id, VALUE val)
    pub fn rb_const_set(klass: VALUE, id: ID, val: VALUE);

    // VALUE rb_cvar_defined(VALUE klass, ID id)
    pub fn rb_cvar_defined(klass: VALUE, id: ID) -> VALUE;
    // VALUE rb_cvar_get(VALUE klass, ID id)
    pub fn rb_cvar_get(klass: VALUE, id: ID) -> VALUE;
    // void rb_cvar_set(VALUE klass, ID id, VALUE val)
    pub fn rb_cvar_set(klass: VALUE, id: ID, val: VALUE);

    // void rb_define_method_id(VALUE klass, ID mid, VALUE (*func)(ANYARGS), int argc)
    pub fn rb_define_method_id(
        klass: VALUE,
        mid: ID,
        func: Option<unsafe extern "C" fn() -> VALUE>,
        argc: c_int,
    );

    // TODO: implement custom argument parsing rules
    // int rb_scan_args(int argc, const VALUE *argv, const char *fmt, ...)
    pub fn rb_scan_args(
        argc: c_int,
        argv: *const VALUE,
        fmt: *const c_char,
        ...
    ) -> c_int;

    // VALUE rb_define_class_id_under(VALUE outer, ID id, VALUE super)
    pub fn rb_define_class_id_under(outer: VALUE, id: ID, sup: VALUE) -> VALUE;
    // VALUE rb_define_class_id_under(VALUE outer, ID id)
    pub fn rb_define_module_id_under(outer: VALUE, id: ID) -> VALUE;

    // void rb_prepend_module(VALUE klass, VALUE module)
    pub fn rb_prepend_module(klass: VALUE, module: VALUE);
    // void rb_include_module(VALUE klass, VALUE module)
    pub fn rb_include_module(klass: VALUE, module: VALUE);
    // void rb_extend_object(VALUE obj, VALUE module)
    pub fn rb_extend_object(obj: VALUE, module: VALUE);

    // VALUE rb_mod_ancestors(VALUE mod)
    pub fn rb_mod_ancestors(module: VALUE) -> VALUE;
    // VALUE rb_mod_include_p(VALUE mod, VALUE mod2)
    pub fn rb_mod_include_p(mod1: VALUE, mod2: VALUE) -> VALUE;
    // VALUE rb_mod_included_modules(VALUE mod)
    pub fn rb_mod_included_modules(module: VALUE) -> VALUE;
    // VALUE rb_mod_module_eval(int argc, const VALUE *argv, VALUE mod)
    pub fn rb_mod_module_eval(argc: c_int, argv: *const VALUE, module: VALUE) -> VALUE;
    // VALUE rb_mod_name(VALUE mod)
    pub fn rb_mod_name(module: VALUE) -> VALUE;
}
