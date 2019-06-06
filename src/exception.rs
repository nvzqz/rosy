//! Ruby exceptions.

use std::{
    convert::Infallible,
    error::Error,
    fmt,
};
use crate::{
    object::NonNullObject,
    prelude::*,
    ruby,
};

/// Some concrete Ruby exception.
///
/// # Safety
///
/// The implementing object type _must_ be an exception type. Otherwise, methods
/// like [`backtrace`](#method.backtrace) and [`cause`](#method.cause) will
/// cause a segmentation fault.
pub unsafe trait Exception: Object + Error {
    /// Creates a new instance of `Self` with `message`.
    fn new(message: impl Into<String>) -> Self;

    /// Returns `self` as an [`AnyException`](struct.AnyException.html).
    #[inline]
    fn into_any_exception(self) -> AnyException { *self.as_any_exception() }

    /// Returns a reference to `self` as an `AnyObject`.
    #[inline]
    fn as_any_exception(&self) -> &AnyException {
        unsafe { &*(self as *const Self as *const AnyException) }
    }

    /// Raises the exception.
    ///
    /// # Safety
    ///
    /// This call should be wrapped around in code that can properly handle
    /// `self`; otherwise a segmentation fault will occur.
    ///
    /// # Examples
    ///
    /// Using `protected` ensures that calling this method is indeed safe:
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::{AnyException, Exception, protected};
    ///
    /// let exc = AnyException::new("Oh noes, something happened!");
    /// let err = protected(|| unsafe { exc.raise() }).unwrap_err();
    ///
    /// assert_eq!(exc, err);
    /// ```
    #[inline]
    unsafe fn raise(self) -> ! {
        ruby::rb_exc_raise(self.raw());
    }

    /// Returns a backtrace associated with `self`.
    #[inline]
    fn backtrace(&self) -> Option<Array<String>> {
        unsafe {
            let obj = self.call(SymbolId::backtrace());
            if obj.is_nil() {
                None
            } else {
                Some(Array::cast_unchecked(obj))
            }
        }
    }

    /// The underlying exception that caused `self`.
    #[inline]
    fn cause(&self) -> Option<AnyException> {
        unsafe {
            let obj = self.call(SymbolId::cause());
            if obj.is_nil() {
                None
            } else {
                Some(AnyException::cast_unchecked(obj))
            }
        }
    }
}

/// Any Ruby exception.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct AnyException(NonNullObject);

impl AsRef<AnyObject> for AnyException {
    #[inline]
    fn as_ref(&self) -> &AnyObject { self.0.as_ref() }
}

impl From<AnyException> for AnyObject {
    #[inline]
    fn from(obj: AnyException) -> Self { obj.0.into() }
}

unsafe impl Object for AnyException {
    #[inline]
    fn cast<A: Object>(obj: A) -> Option<Self> {
        if obj.class().inherits(Class::exception()) {
            unsafe { Some(Self::from_raw(obj.raw())) }
        } else {
            None
        }
    }

    #[inline]
    fn class(self) -> Class<Self> {
        let ptr = self.raw() as *const ruby::RBasic;
        unsafe { Class::from_raw((*ptr).klass) }
    }
}

impl fmt::Display for AnyException {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_any_object().fmt(f)
    }
}

impl Error for AnyException {}

unsafe impl Exception for AnyException {
    #[inline]
    fn new(message: impl Into<String>) -> Self {
        unsafe { Self::of_class(Class::exception(), message) }
    }
}

impl From<Infallible> for AnyException {
    #[inline]
    fn from(error: Infallible) -> Self {
        match error {}
    }
}

impl<O: Object> PartialEq<O> for AnyException {
    #[inline]
    fn eq(&self, other: &O) -> bool {
        self.raw() == other.raw()
    }
}

impl Eq for AnyException {}

impl AnyException {
    // Returns the current exception without checking, clearing it globally
    #[cold] // Exception is less likely than success
    #[inline]
    pub(crate) unsafe fn _take_current() -> AnyException {
        let exc = ruby::rb_errinfo();
        ruby::rb_set_errinfo(crate::util::NIL_VALUE);
        AnyException::from_raw(exc)
    }

    /// Creates a new instance of `class` with `message`.
    ///
    /// # Safety
    ///
    /// The `class` argument must inherit from an exception class.
    #[inline]
    pub unsafe fn of_class<O: Object>(
        class: impl Into<Class<O>>,
        message: impl Into<String>,
    ) -> Self {
        Self::cast_unchecked(class.into().new_instance_with_unchecked(&[
            message.into()
        ]))
    }

    /// Returns the current pending exception.
    #[inline]
    pub fn current() -> Option<AnyException> {
        unsafe {
            match ruby::rb_errinfo() {
                crate::util::NIL_VALUE => None,
                raw => Some(AnyException::from_raw(raw))
            }
        }
    }

    /// Returns the current pending exception, removing it from its global spot.
    #[inline]
    pub fn take_current() -> Option<AnyException> {
        let current = AnyException::current()?;
        unsafe { ruby::rb_set_errinfo(crate::util::NIL_VALUE) };
        Some(current)
    }
}

macro_rules! typed_exceptions {
    ($($name:ident => $is:ident $to:ident $class:ident;)+) => {
        typed_exceptions! { $(
            $name,
            stringify!($name),
            concat!("A `", stringify!($name), "` exception.")
            => $is $to $class;
        )+ }
    };
    ($($name:ident, $name_str:expr, $ty_doc:expr => $is:ident $to:ident $class:ident;)+) => {
        $(
            #[doc = $ty_doc]
            #[derive(Clone, Copy)]
            pub struct $name(AnyException);

            impl From<$name> for AnyException {
                #[inline]
                fn from(exc: $name) -> Self {
                    exc.0
                }
            }

            impl AsRef<AnyException> for $name {
                #[inline]
                fn as_ref(&self) -> &AnyException {
                    &self.0
                }
            }

            impl From<$name> for AnyObject {
                #[inline]
                fn from(exc: $name) -> Self {
                    exc.0.into()
                }
            }

            impl AsRef<AnyObject> for $name {
                #[inline]
                fn as_ref(&self) -> &AnyObject {
                    self.0.as_ref()
                }
            }

            impl<O: Object> PartialEq<O> for $name {
                #[inline]
                fn eq(&self, other: &O) -> bool {
                    self.raw() == other.raw()
                }
            }

            impl Eq for $name {}

            unsafe impl Object for $name {
                #[inline]
                fn cast<A: Object>(obj: A) -> Option<Self> {
                    if obj.class().inherits(Class::$class()) {
                        unsafe { Some(Self::from_raw(obj.raw())) }
                    } else {
                        None
                    }
                }

                #[inline]
                fn class(self) -> Class<Self> {
                    let ptr = self.raw() as *const ruby::RBasic;
                    unsafe { Class::from_raw((*ptr).klass) }
                }
            }

            impl fmt::Debug for $name {
                #[inline]
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.debug_tuple(stringify!($name))
                        .field(self.as_any_object())
                        .finish()
                }
            }

            impl fmt::Display for $name {
                #[inline]
                fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    self.as_any_object().fmt(f)
                }
            }

            impl Error for $name {}

            unsafe impl Exception for $name {
                #[inline]
                fn new(message: impl Into<String>) -> Self {
                    unsafe {
                        let class = Class::$class();
                        let any = AnyException::of_class(class, message);
                        Self::cast_unchecked(any)
                    }
                }
            }
        )+

        /// Typed exceptions.
        impl AnyException {
            $(
                /// Returns whether `self` is a `
                #[doc = $name_str]
                /// `.
                #[inline]
                pub fn $is(self) -> bool {
                    self.class().inherits(Class::$class())
                }

                /// Returns `self` as a `
                #[doc = $name_str]
                /// ` if it is one.
                #[inline]
                pub fn $to(self) -> Option<$name> {
                    if self.$is() {
                        Some($name(self))
                    } else {
                        None
                    }
                }
            )+
        }
    };
}

typed_exceptions! {
    StandardError    => is_standard_error     to_standard_error     standard_error;
    SystemExit       => is_system_exit        to_system_exit        system_exit;
    Interrupt        => is_interrupt          to_interrupt          interrupt;
    Signal           => is_signal             to_signal             signal;
    Fatal            => is_fatal              to_fatal              fatal;
    ArgumentError    => is_arg_error          to_arg_error          arg_error;
    EOFError         => is_eof_error          to_eof_error          eof_error;
    IndexError       => is_index_error        to_index_error        index_error;
    StopIteration    => is_stop_iteration     to_stop_iteration     stop_iteration;
    KeyError         => is_key_error          to_key_error          key_error;
    RangeError       => is_range_error        to_range_error        range_error;
    IOError          => is_io_error           to_io_error           io_error;
    RuntimeError     => is_runtime_error      to_runtime_error      runtime_error;
    FrozenError      => is_frozen_error       to_frozen_error       frozen_error;
    SecurityError    => is_security_error     to_security_error     security_error;
    SystemCallError  => is_system_call_error  to_system_call_error  system_call_error;
    ThreadError      => is_thread_error       to_thread_error       thread_error;
    TypeError        => is_type_error         to_type_error         type_error;
    ZeroDivError     => is_zero_div_error     to_zero_div_error     zero_div_error;
    NotImpError      => is_not_imp_error      to_not_imp_error      not_imp_error;
    NoMemError       => is_no_mem_error       to_no_mem_error       no_mem_error;
    NoMethodError    => is_no_method_error    to_no_method_error    no_method_error;
    FloatDomainError => is_float_domain_error to_float_domain_error float_domain_error;
    LocalJumpError   => is_local_jump_error   to_local_jump_error   local_jump_error;
    SysStackError    => is_sys_stack_error    to_sys_stack_error    sys_stack_error;
    RegexpError      => is_regexp_error       to_regexp_error       regexp_error;
    EncodingError    => is_encoding_error     to_encoding_error     encoding_error;
    EncCompatError   => is_enc_compat_error   to_enc_compat_error   enc_compat_error;
    ScriptError      => is_script_error       to_script_error       script_error;
    NameError        => is_name_error         to_name_error         name_error;
    SyntaxError      => is_syntax_error       to_syntax_error       syntax_error;
    LoadError        => is_load_error         to_load_error         load_error;
    MathDomainError  => is_math_domain_error  to_math_domain_error  math_domain_error;
}
