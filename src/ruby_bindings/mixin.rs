use super::prelude::*;

extern "C" {
    #[cfg_attr(dllimport, link_name="__imp_rb_mKernel")]
    pub static rb_mKernel: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_mComparable")]
    pub static rb_mComparable: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_mEnumerable")]
    pub static rb_mEnumerable: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_mErrno")]
    pub static rb_mErrno: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_mFileTest")]
    pub static rb_mFileTest: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_mGC")]
    pub static rb_mGC: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_mMath")]
    pub static rb_mMath: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_mProcess")]
    pub static rb_mProcess: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_mWaitReadable")]
    pub static rb_mWaitReadable: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_mWaitWritable")]
    pub static rb_mWaitWritable: Var<VALUE>;

    #[cfg_attr(dllimport, link_name="__imp_rb_cBasicObject")]
    pub static rb_cBasicObject: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cObject")]
    pub static rb_cObject: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cArray")]
    pub static rb_cArray: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cBinding")]
    pub static rb_cBinding: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cClass")]
    pub static rb_cClass: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cCont")]
    pub static rb_cCont: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cData")]
    pub static rb_cData: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cDir")]
    pub static rb_cDir: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cEncoding")]
    pub static rb_cEncoding: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cEnumerator")]
    pub static rb_cEnumerator: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cFalseClass")]
    pub static rb_cFalseClass: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cFile")]
    pub static rb_cFile: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cComplex")]
    pub static rb_cComplex: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cFloat")]
    pub static rb_cFloat: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cHash")]
    pub static rb_cHash: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cIO")]
    pub static rb_cIO: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cInteger")]
    pub static rb_cInteger: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cMatch")]
    pub static rb_cMatch: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cMethod")]
    pub static rb_cMethod: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cModule")]
    pub static rb_cModule: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cNameErrorMesg")]
    pub static rb_cNameErrorMesg: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cNilClass")]
    pub static rb_cNilClass: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cNumeric")]
    pub static rb_cNumeric: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cProc")]
    pub static rb_cProc: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cRandom")]
    pub static rb_cRandom: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cRange")]
    pub static rb_cRange: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cRational")]
    pub static rb_cRational: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cRegexp")]
    pub static rb_cRegexp: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cStat")]
    pub static rb_cStat: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cString")]
    pub static rb_cString: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cStruct")]
    pub static rb_cStruct: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cSymbol")]
    pub static rb_cSymbol: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cThread")]
    pub static rb_cThread: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cTime")]
    pub static rb_cTime: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cTrueClass")]
    pub static rb_cTrueClass: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cUnboundMethod")]
    pub static rb_cUnboundMethod: Var<VALUE>;

    // Found in 'vm_core.h'
    #[cfg_attr(dllimport, link_name="__imp_rb_cRubyVM")]
    pub static rb_cRubyVM: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_cISeq")]
    pub static rb_cISeq: Var<VALUE>;

    #[cfg_attr(dllimport, link_name="__imp_rb_eException")]
    pub static rb_eException: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eStandardError")]
    pub static rb_eStandardError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eSystemExit")]
    pub static rb_eSystemExit: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eInterrupt")]
    pub static rb_eInterrupt: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eSignal")]
    pub static rb_eSignal: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eFatal")]
    pub static rb_eFatal: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eArgError")]
    pub static rb_eArgError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eEOFError")]
    pub static rb_eEOFError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eIndexError")]
    pub static rb_eIndexError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eStopIteration")]
    pub static rb_eStopIteration: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eKeyError")]
    pub static rb_eKeyError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eRangeError")]
    pub static rb_eRangeError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eIOError")]
    pub static rb_eIOError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eRuntimeError")]
    pub static rb_eRuntimeError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eFrozenError")]
    pub static rb_eFrozenError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eSecurityError")]
    pub static rb_eSecurityError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eSystemCallError")]
    pub static rb_eSystemCallError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eThreadError")]
    pub static rb_eThreadError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eTypeError")]
    pub static rb_eTypeError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eZeroDivError")]
    pub static rb_eZeroDivError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eNotImpError")]
    pub static rb_eNotImpError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eNoMemError")]
    pub static rb_eNoMemError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eNoMethodError")]
    pub static rb_eNoMethodError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eFloatDomainError")]
    pub static rb_eFloatDomainError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eLocalJumpError")]
    pub static rb_eLocalJumpError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eSysStackError")]
    pub static rb_eSysStackError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eRegexpError")]
    pub static rb_eRegexpError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eEncodingError")]
    pub static rb_eEncodingError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eEncCompatError")]
    pub static rb_eEncCompatError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eScriptError")]
    pub static rb_eScriptError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eNameError")]
    pub static rb_eNameError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eSyntaxError")]
    pub static rb_eSyntaxError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eLoadError")]
    pub static rb_eLoadError: Var<VALUE>;
    #[cfg_attr(dllimport, link_name="__imp_rb_eMathDomainError")]
    pub static rb_eMathDomainError: Var<VALUE>;

    // void rb_attr(VALUE klass, ID id, int read, int write, int ex)
    pub fn rb_attr(klass: VALUE, id: ID, read: c_int, write: c_int, ex: c_int);
    // VALUE rb_ivar_get(VALUE obj, ID id)
    pub fn rb_attr_get(obj: VALUE, id: ID) -> VALUE;

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
