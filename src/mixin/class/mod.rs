//! Ruby classes.

use std::{
    cmp::Ordering,
    fmt,
    marker::PhantomData,
    os::raw::c_int,
};
use crate::{
    mixin::{DefMixinError, MethodFn},
    object::{NonNullObject, Ty},
    prelude::*,
    ruby,
};

mod classify;
mod inheritance;

pub use self::{
    classify::*,
    inheritance::*,
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
#[repr(transparent)]
pub struct Class<O = AnyObject> {
    inner: NonNullObject,
    _marker: PhantomData<fn() -> O>,
}

impl<O> Clone for Class<O> {
    fn clone(&self) -> Self { *self }
}

impl<O> Copy for Class<O> {}

impl<O: Object> AsRef<AnyObject> for Class<O> {
    #[inline]
    fn as_ref(&self) -> &AnyObject { self.inner.as_ref() }
}

impl<O: Object> From<Class<O>> for AnyObject {
    #[inline]
    fn from(object: Class<O>) -> AnyObject { object.inner.into() }
}

impl<O: Object> fmt::Debug for Class<O> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Class")
            .field(&self.inner)
            .finish()
    }
}

impl<O: Object> fmt::Display for Class<O> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_any_object().fmt(f)
    }
}

unsafe impl<O: Object> Object for Class<O> {
    #[inline]
    fn unique_id() -> Option<u128> {
        Some(!(Ty::Class as u128))
    }

    #[inline]
    fn cast<A: Object>(obj: A) -> Option<Self> {
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

impl<O: Object, P: Object> PartialEq<P> for Class<O> {
    #[inline]
    fn eq(&self, other: &P) -> bool {
        self.raw() == other.raw()
    }
}

impl<O: Object> Eq for Class<O> {}

impl<O: Object, P: Object> PartialOrd<Class<P>> for Class<O> {
    #[inline]
    fn partial_cmp(&self, other: &Class<P>) -> Option<Ordering> {
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
    fn lt(&self, other: &Class<P>) -> bool {
        other.inheritance(*self).is_super()
    }

    #[inline]
    fn le(&self, other: &Class<P>) -> bool {
        self.inheritance(*other).is_sub_eq()
    }

    #[inline]
    fn gt(&self, other: &Class<P>) -> bool {
        self.inheritance(*other).is_super()
    }

    #[inline]
    fn ge(&self, other: &Class<P>) -> bool {
        other.inheritance(*self).is_sub_eq()
    }
}

impl Class {
    pub(crate) fn _def_under(
        m: impl Mixin,
        superclass: Class,
        name: SymbolId,
    ) -> Result<Class, DefMixinError> {
        if let Some(err) = DefMixinError::_get(m, name) {
            return Err(err);
        } else if m.is_frozen() {
            return Err(DefMixinError::_frozen(m));
        }
        unsafe { Ok(Class::from_raw(ruby::rb_define_class_id_under(
            m.raw(),
            name.raw(),
            superclass.raw(),
        ))) }
    }

    /// Returns the typed class of some type that implements `Classify`.
    ///
    /// # Examples
    ///
    /// We can see from getting the untyped version of the `Array` class that
    /// they both point to the same class:
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::{Array, Class};
    ///
    /// let class = Class::of::<Array>();
    /// assert_eq!(class, Class::array());
    /// ```
    #[inline]
    pub fn of<O: Classify>() -> Class<O> {
        <O as Classify>::class()
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
    /// assert_eq!(Class::array(), array.unwrap());
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

    /// Retrieves an existing top-level `Class` defined by `name`.
    ///
    /// # Safety
    ///
    /// This method does not:
    /// - Check whether an item for `name` exists (an exception will be thrown
    ///   if this is the case)
    /// - Check whether the returned item for `name` is actually a `Class`
    #[inline]
    pub unsafe fn get_unchecked(name: impl Into<SymbolId>) -> Self {
        Class::object().get_class_unchecked(name)
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
}

impl<O: Object> Class<O> {
    /// Converts `self` into an untyped class.
    #[inline]
    pub fn into_any_class(self) -> Class {
        Class { inner: self.inner, _marker: PhantomData }
    }

    /// Creates a new instance without arguments.
    #[inline]
    pub fn new_instance(self) -> Result<O> {
        let args: &[AnyObject] = &[];
        self.new_instance_with(args)
    }

    /// Creates a new instance without arguments.
    ///
    /// # Safety
    ///
    /// An exception may be thrown if the class expected arguments.
    #[inline]
    pub unsafe fn new_instance_unchecked(self) -> O {
        let args: &[AnyObject] = &[];
        self.new_instance_with_unchecked(args)
    }

    /// Creates a new instance from `args`.
    #[inline]
    pub fn new_instance_with<A: Object>(self, args: &[A]) -> Result<O> {
        // monomorphization
        fn new_instance_with(c: Class, a: &[AnyObject]) -> Result<AnyObject> {
            unsafe {
                crate::protected_no_panic(|| c.new_instance_with_unchecked(a))
            }
        }
        let class = self.into_any_class();
        let object = new_instance_with(class, AnyObject::convert_slice(args))?;
        unsafe { Ok(O::cast_unchecked(object)) }
    }

    /// Creates a new instance from `args`.
    ///
    /// # Safety
    ///
    /// An exception may be thrown if the class expected arguments.
    #[inline]
    pub unsafe fn new_instance_with_unchecked<A: Object>(
        self,
        args: &[A],
    ) -> O {
        O::from_raw(ruby::rb_class_new_instance(
            args.len() as c_int,
            args.as_ptr() as *const ruby::VALUE,
            self.raw(),
        ))
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
    ) -> Result<Class, DefMixinError> {
        self.subclass_under(Class::object(), name)
    }

    /// Defines a subclass of `self` under `namespace` with `name`.
    #[inline]
    pub fn subclass_under(
        self,
        namespace: impl Mixin,
        name: impl Into<SymbolId>,
    ) -> Result<Class, DefMixinError> {
        namespace.def_subclass(self, name)
    }

    /// Returns the inheritance relationship between `self` and `other`.
    #[inline]
    pub fn inheritance<P: Object>(self, other: Class<P>) -> Inheritance {
        let v = unsafe { ruby::rb_class_inherited_p(self.raw(), other.raw()) };
        match v {
            crate::util::TRUE_VALUE  => Inheritance::SubEq,
            crate::util::FALSE_VALUE => Inheritance::Super,
            _ => Inheritance::None,
        }
    }

    /// Returns whether the relationship between `self` and `other` is `A <= B`.
    #[inline]
    pub fn inherits<P: Object>(self, other: Class<P>) -> bool {
        self <= other
    }

    /// Returns the name of `self`.
    #[inline]
    pub fn name(self) -> String {
        unsafe { String::from_raw(ruby::rb_class_name(self.raw())) }
    }

    // monomorphization
    unsafe fn _def_method(
        self,
        name: SymbolId,
        f: unsafe extern "C" fn() -> ruby::VALUE,
        arity: c_int,
    ) -> Result {
        crate::protected_no_panic(|| self._def_method_unchecked(name, f, arity))
    }

    #[inline]
    unsafe fn _def_method_unchecked(
        self,
        name: SymbolId,
        f: unsafe extern "C" fn() -> ruby::VALUE,
        arity: c_int,
    ) {
        ruby::rb_define_method_id(self.raw(), name.raw(), Some(f), arity)
    }

    /// Defines a method for `name` on `self` that calls `f` when invoked.
    ///
    /// **Note:** This method can be unwieldy to use and so it is recommended to
    /// instead call the convenience macro [`def_method!`].
    ///
    /// # About `MethodFn`
    ///
    /// - The first argument is _always_ of type `O`. This means that if `self`
    ///   is a typed class, then no casting is required within the method.
    ///
    /// - Up to 15 arguments may be passed. If more or a variable amount is
    ///   needed, see the below examples.
    ///
    /// - They can return any type that implements the `Object` trait.
    ///
    /// - Unfortunately, because `MethodFn` is defined on `extern "C" fn` types
    ///   and these function declarations don't implicitly resolve to those
    ///   types, an `as` cast is required for `f`.
    ///
    /// # Examples
    ///
    /// Every method takes `this` (equivalent of `self`) as the first argument,
    /// which may be followed by up to 15 arguments.
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::prelude::*;
    ///
    /// extern "C" fn my_eql(this: Array, that: AnyObject) -> AnyObject {
    ///     AnyObject::from(this == that)
    /// }
    /// let my_eql: extern fn(_, _) -> _ = my_eql;
    ///
    /// Class::of::<Array>()
    ///     .def_method("my_eql?", my_eql)
    ///     .unwrap();
    ///
    /// let array: Array = (0..10).collect();
    /// let value = array.call_with("my_eql?", &[array]).unwrap();
    ///
    /// assert!(value.is_true());
    /// ```
    ///
    /// Passing in the wrong number of arguments will result in an
    /// `ArgumentError` exception being raised:
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// # use rosy::prelude::*;
    /// # extern "C" fn my_eq(this: Array, that: AnyObject) -> AnyObject {
    /// #     AnyObject::from(this == that)
    /// # }
    /// # let class = Class::of::<Array>();
    /// # let array = Array::from_slice(&[String::from("hello")]);
    /// # class.def_method("my_eql?", my_eq as extern fn(_, _) -> _).unwrap();
    /// assert!(array.call("my_eql?").unwrap_err().is_arg_error());
    /// ```
    ///
    /// ## Variable Arguments
    ///
    /// There are two ways of taking in a variable number of arguments.
    ///
    /// The first is by taking a pointer and a length:
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use std::os::raw::c_int;
    /// use std::slice::from_raw_parts;
    /// use rosy::prelude::*;
    ///
    /// unsafe extern "C" fn eq_all(this: AnyObject, len: c_int, ptr: *const AnyObject) -> AnyObject {
    ///     let slice = from_raw_parts(ptr, len as usize);
    ///     for &obj in slice {
    ///         if obj != this {
    ///             return false.into();
    ///         }
    ///     }
    ///     true.into()
    /// }
    ///
    /// let class = Class::string();
    /// let string = String::from("hellooo");
    ///
    /// class.def_method("eq_all?", eq_all as unsafe extern fn(_, _, _) -> _);
    ///
    /// let args = [string, String::from("byeee")];
    /// assert!(string.call_with("eq_all?", &args).unwrap().is_false());
    /// ```
    ///
    /// The second is by taking an `Array` as an argument:
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::prelude::*;
    ///
    /// unsafe extern "C" fn joining(this: String, args: Array) -> String {
    ///     args.join(this)
    /// }
    /// let joining: unsafe extern fn(_, _) -> _ = joining;
    ///
    /// let class = Class::of::<String>();
    /// class.def_method("joining", joining).unwrap();
    ///
    /// let string = String::from(", ");
    /// let output = string.call_with("joining", &[string, string]).unwrap();
    ///
    /// assert_eq!(output, ", , , ");
    /// ```
    ///
    // Link to docs.rs since `Class` may either be in `class` module or root
    /// [`def_method!`]: https://docs.rs/rosy/0.0.5/rosy/macro.def_method.html
    pub fn def_method<N, F>(self, name: N, f: F) -> Result
    where
        N: Into<SymbolId>,
        F: MethodFn<O>,
    {
        unsafe { self._def_method(name.into(), f.raw_fn(), F::ARITY) }
    }

    /// Defines a method for `name` on `self` that calls `f` when invoked.
    ///
    /// **Note:** This method can be unwieldy to use and so it is recommended to
    /// instead call the convenience macro [`def_method_unchecked!`].
    ///
    /// See [`def_method`](#method.def_method) for usage info.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised.
    ///
    // Link to docs.rs since `Class` may either be in `class` module or root
    /// [`def_method_unchecked!`]: https://docs.rs/rosy/0.0.5/rosy/macro.def_method_unchecked.html
    #[inline]
    pub unsafe fn def_method_unchecked<N, F>(self, name: N, f: F)
    where
        N: Into<SymbolId>,
        F: MethodFn<O>,
    {
        self._def_method_unchecked(name.into(), f.raw_fn(), F::ARITY)
    }
}

macro_rules! built_in_classes {
    ($($vm_name:expr, $method:ident, $konst:ident;)+) => {
        /// Built-in classes.
        impl Class {
            /// Returns the `RustObject` class.
            ///
            /// This class can be used when simply wrapping Rust data.
            #[inline]
            pub fn rust_object() -> Self {
                // A static function pointer that will get swapped out the first
                // time it's called and will simply return `RUST_OBJECT` on
                // all subsequent calls without branching
                static mut GET_RUST_OBJECT: fn() -> Class = || unsafe {
                    static mut RUST_OBJECT: AnyObject = unsafe {
                        AnyObject::from_raw(0)
                    };

                    let class = Class::get_or_def("RustObject")
                        .expect("Failed to create 'RustObject'");

                    RUST_OBJECT = class.into();
                    GET_RUST_OBJECT = || Class::cast_unchecked(RUST_OBJECT);

                    crate::gc::register(&RUST_OBJECT);
                    class.freeze();

                    class
                };

                unsafe { GET_RUST_OBJECT() }
            }

            $(
                /// The `
                #[doc = $vm_name]
                ///` class.
                #[inline]
                pub fn $method() -> Self {
                    unsafe { Self::from_raw(ruby::$konst) }
                }
            )+
        }
    }
}

built_in_classes! {
    // "Fixnum" is obsolete; use Integer
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
