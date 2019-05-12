//! Ruby classes.

use std::{
    cmp::Ordering,
    fmt,
    os::raw::c_int,
};
use crate::{
    mixin::DefMixinError,
    object::{NonNullObject, Ty},
    prelude::*,
};

/// An instance of Ruby's `Class` type.
///
/// # Examples
///
/// Class inheritance can be expressed in terms of logical comparison operators:
///
/// ```
/// use rosy::Class;
/// # rosy::vm::init().unwrap();
///
/// assert!(Class::object()    < Class::basic_object());
/// assert!(Class::exception() < Class::object());
/// assert!(Class::arg_error() < Class::exception());
/// ```
///
/// This very closely resembles Ruby's syntax for subclassing:
///
/// ```ruby
/// class Mammal
///   def breathe
///     puts "inhale and exhale"
///   end
/// end
///
/// class Cat < Mammal
///   def speak
///     puts "meow"
///   end
/// end
/// ```
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Class(NonNullObject);

impl AsRef<AnyObject> for Class {
    #[inline]
    fn as_ref(&self) -> &AnyObject { self.0.as_ref() }
}

impl From<Class> for AnyObject {
    #[inline]
    fn from(object: Class) -> AnyObject { object.0.into() }
}

unsafe impl Object for Class {
    #[inline]
    fn cast(obj: impl Object) -> Option<Self> {
        if obj.is_ty(Ty::Class) {
            unsafe { Some(Self::cast_unchecked(obj)) }
        } else {
            None
        }
    }

    #[inline]
    fn ty(self) -> Ty { Ty::Class }

    #[inline]
    fn is_ty(self, ty: Ty) -> bool { ty == Ty::Class }
}

impl crate::util::Sealed for Class {}

impl fmt::Display for Class {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_any_object().fmt(f)
    }
}

impl<O: Object> PartialEq<O> for Class {
    #[inline]
    fn eq(&self, other: &O) -> bool {
        self.raw() == other.raw()
    }
}

impl Eq for Class {}

impl PartialOrd for Class {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self == other {
            Some(Ordering::Equal)
        } else {
            match self.inheritance(*other) {
                Inheritance::SubEq => Some(Ordering::Less),
                Inheritance::Super => Some(Ordering::Greater),
                Inheritance::None  => None,
            }
        }
    }

    #[inline]
    fn lt(&self, other: &Self) -> bool {
        other.inheritance(*self).is_super()
    }

    #[inline]
    fn le(&self, other: &Self) -> bool {
        self.inheritance(*other).is_sub_eq()
    }

    #[inline]
    fn gt(&self, other: &Self) -> bool {
        self.inheritance(*other).is_super()
    }

    #[inline]
    fn ge(&self, other: &Self) -> bool {
        other.inheritance(*self).is_sub_eq()
    }
}

impl Class {
    pub(crate) fn _def_under(
        m: ruby::VALUE,
        superclass: Class,
        name: SymbolId,
    ) -> Result<Class, DefMixinError> {
        if let Some(err) = DefMixinError::_get(m, name) {
            return Err(err);
        }
        unsafe { Ok(Class::from_raw(ruby::rb_define_class_id_under(
            m,
            name.raw(),
            superclass.raw(),
        ))) }
    }

    /// Defines a new top-level class with `name`.
    ///
    /// # Examples
    ///
    /// Defining a new class is straightforward:
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// let my_object = rosy::Class::def("MyObject").unwrap();
    /// ```
    ///
    /// Attempting to define an existing class will result in an error:
    ///
    /// ```
    /// use rosy::Class;
    /// # rosy::vm::init().unwrap();
    ///
    /// let array = Class::def("Array").unwrap_err().existing_object();
    /// assert_eq!(Class::array(), array);
    /// ```
    #[inline]
    pub fn def(name: impl Into<SymbolId>) -> Result<Self, DefMixinError> {
        Class::object().def_class(name)
    }

    /// Retrieves an existing top-level `Class` defined by `name`.
    #[inline]
    pub fn get(name: impl Into<SymbolId>) -> Option<Self> {
        Class::object().get_class(name)
    }

    /// Retrieves an existing top-level `Class` defined by `name` or defines one
    /// if it doesn't exist.
    #[inline]
    pub fn get_or_def(name: impl Into<SymbolId>) -> Result<Self, DefMixinError> {
        match Class::def(name) {
            Ok(class) => Ok(class),
            Err(error) => if let Some(class) = error.existing_class() {
                Ok(class)
            } else {
                Err(error)
            }
        }
    }

    /// Creates a new instance from `args` to pass into `#initialize`.
    #[inline]
    pub fn new_instance(self, args: &[impl Object]) -> AnyObject {
        unsafe { AnyObject::from_raw(ruby::rb_class_new_instance(
            args.len() as c_int,
            args.as_ptr() as *const ruby::VALUE,
            self.raw(),
        )) }
    }

    /// Returns the parent class of `self`.
    #[inline]
    pub fn superclass(self) -> Class {
        unsafe { Class::from_raw(ruby::rb_class_superclass(self.raw())) }
    }

    /// Defines a new subclass of `self` with `name`.
    #[inline]
    pub fn subclass(
        self,
        name: impl Into<SymbolId>,
    ) -> Result<Self, DefMixinError> {
        self.subclass_under(Class::object(), name)
    }

    /// Defines a subclass of `self` under `namespace` with `name`.
    #[inline]
    pub fn subclass_under(
        self,
        namespace: impl Mixin,
        name: impl Into<SymbolId>,
    ) -> Result<Self, DefMixinError> {
        namespace.def_subclass(self, name)
    }

    /// Returns the inheritance relationship between `self` and `other`.
    #[inline]
    pub fn inheritance(self, other: Class) -> Inheritance {
        let v = unsafe { ruby::rb_class_inherited_p(self.raw(), other.raw()) };
        match v {
            crate::util::TRUE_VALUE  => Inheritance::SubEq,
            crate::util::FALSE_VALUE => Inheritance::Super,
            _ => Inheritance::None,
        }
    }

    /// Returns whether the relationship between `self` and `other` is `A < B`.
    #[inline]
    pub fn inherits(self, other: Class) -> bool {
        self <= other
    }

    /// Returns the name of `self`.
    #[inline]
    pub fn name(self) -> String {
        unsafe { String::from_raw(ruby::rb_class_name(self.raw())) }
    }
}

/// The [`inheritance`](struct.Class.html#method.inheritance) relationship
/// between two classes.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum Inheritance {
    /// There is no relationship between the two classes.
    None,
    /// The first class inherits or is the same as the second; `A < B`.
    SubEq,
    /// The second class inherits the first; `B < A`.
    Super,
}

impl Inheritance {
    /// Returns whether there's no relationship between the classes.
    #[inline]
    pub fn is_none(self) -> bool {
        self == Inheritance::None
    }

    /// Returns whether the first class inherits or is the same as the second.
    #[inline]
    pub fn is_sub_eq(self) -> bool {
        self == Inheritance::SubEq
    }

    /// Returns whether the second class inherits the first.
    #[inline]
    pub fn is_super(self) -> bool {
        self == Inheritance::Super
    }
}

macro_rules! built_in_classes {
    ($($vm_name:expr, $method:ident, $konst:ident;)+) => {
        /// Built-in classes.
        impl Class {$(
            /// The `
            #[doc = $vm_name]
            ///` class.
            #[inline]
            pub fn $method() -> Self {
                unsafe { Self::from_raw(ruby::$konst) }
            }
        )+}
    }
}

built_in_classes! {
    "BasicObject",   basic_object,   rb_cBasicObject;
    "Object",        object,         rb_cObject;
    "Array",         array,          rb_cArray;
    "Binding",       binding,        rb_cBinding;
    "Class",         class,          rb_cClass;
    "Cont",          cont,           rb_cCont;
    "Data",          data,           rb_cData;
    "Dir",           dir,            rb_cDir;
    "Encoding",      encoding,       rb_cEncoding;
    "Enumerator",    enumerator,     rb_cEnumerator;
    "FalseClass",    false_class,    rb_cFalseClass;
    "File",          file,           rb_cFile;
    // "Fixnum",        class_fixnum,         rb_cFixnum;
    "Complex",       complex,        rb_cComplex;
    "Float",         float,          rb_cFloat;
    "Hash",          hash,           rb_cHash;
    "IO",            io,             rb_cIO;
    "Integer",       integer,        rb_cInteger;
    "Match",         mtch,           rb_cMatch;
    "Method",        method,         rb_cMethod;
    "Module",        module,         rb_cModule;
    "NameErrorMesg", name_error_msg, rb_cNameErrorMesg;
    "NilClass",      nil,            rb_cNilClass;
    "Numeric",       numeric,        rb_cNumeric;
    "Proc",          proc,           rb_cProc;
    "Random",        random,         rb_cRandom;
    "Range",         range,          rb_cRange;
    "Rational",      rational,       rb_cRational;
    "Regexp",        regexp,         rb_cRegexp;
    "Stat",          stat,           rb_cStat;
    "String",        string,         rb_cString;
    "Struct",        strukt,         rb_cStruct;
    "Symbol",        symbol,         rb_cSymbol;
    "Thread",        thread,         rb_cThread;
    "Time",          time,           rb_cTime;
    "TrueClass",     true_class,     rb_cTrueClass;
    "UnboundMethod", unbound_method, rb_cUnboundMethod;

    "RubyVM",                      ruby_vm,   rb_cRubyVM;
    "RubyVM::InstructionSequence", instr_seq, rb_cISeq;

    "Exception",        exception,          rb_eException;
    "StandardError",    standard_error,     rb_eStandardError;
    "SystemExit",       system_exit,        rb_eSystemExit;
    "Interrupt",        interrupt,          rb_eInterrupt;
    "Signal",           signal,             rb_eSignal;
    "Fatal",            fatal,              rb_eFatal;
    "ArgumentError",    arg_error,          rb_eArgError;
    "EOFError",         eof_error,          rb_eEOFError;
    "IndexError",       index_error,        rb_eIndexError;
    "StopIteration",    stop_iteration,     rb_eStopIteration;
    "KeyError",         key_error,          rb_eKeyError;
    "RangeError",       range_error,        rb_eRangeError;
    "IOError",          io_error,           rb_eIOError;
    "RuntimeError",     runtime_error,      rb_eRuntimeError;
    "FrozenError",      frozen_error,       rb_eFrozenError;
    "SecurityError",    security_error,     rb_eSecurityError;
    "SystemCallError",  system_call_error,  rb_eSystemCallError;
    "ThreadError",      thread_error,       rb_eThreadError;
    "TypeError",        type_error,         rb_eTypeError;
    "ZeroDivError",     zero_div_error,     rb_eZeroDivError;
    "NotImpError",      not_imp_error,      rb_eNotImpError;
    "NoMemError",       no_mem_error,       rb_eNoMemError;
    "NoMethodError",    no_method_error,    rb_eNoMethodError;
    "FloatDomainError", float_domain_error, rb_eFloatDomainError;
    "LocalJumpError",   local_jump_error,   rb_eLocalJumpError;
    "SysStackError",    sys_stack_error,    rb_eSysStackError;
    "RegexpError",      regexp_error,       rb_eRegexpError;
    "EncodingError",    encoding_error,     rb_eEncodingError;
    "EncCompatError",   enc_compat_error,   rb_eEncCompatError;
    "ScriptError",      script_error,       rb_eScriptError;
    "NameError",        name_error,         rb_eNameError;
    "SyntaxError",      syntax_error,       rb_eSyntaxError;
    "LoadError",        load_error,         rb_eLoadError;
    "MathDomainError",  math_domain_error,  rb_eMathDomainError;
}
