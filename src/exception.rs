//! Ruby exceptions.

use std::fmt;
use crate::{
    object::NonNullObject,
    prelude::*,
};

/// Some concrete Ruby exception.
///
/// # Safety
///
/// The implementing object type _must_ be an exception type. Otherwise, methods
/// like [`backtrace`](#method.backtrace) and [`cause`](#method.cause) will
/// cause a segmentation fault.
pub unsafe trait Exception: Object {
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
    unsafe fn raise(self) {
        ruby::rb_exc_raise(self.raw());
    }

    /// Returns a backtrace associated with `self`.
    ///
    /// The array contains strings.
    #[inline]
    fn backtrace(&self) -> Option<Array> {
        unsafe {
            let obj = self.call_unchecked("backtrace");
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
            let obj = self.call_unchecked("cause");
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
    fn cast(obj: impl Object) -> Option<Self> {
        if obj.class().inherits(Class::exception()) {
            unsafe { Some(Self::from_raw(obj.raw())) }
        } else {
            None
        }
    }
}

impl fmt::Display for AnyException {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_any_object().fmt(f)
    }
}

unsafe impl Exception for AnyException {

}

impl<E: Exception> PartialEq<E> for AnyException {
    #[inline]
    fn eq(&self, other: &E) -> bool {
        self.raw() == other.raw()
    }
}

impl Eq for AnyException {}

impl AnyException {
    // Returns the current exception without checking, clearing it globally
    #[inline]
    pub(crate) unsafe fn _take_current() -> AnyException {
        let exc = ruby::rb_errinfo();
        ruby::rb_set_errinfo(crate::util::NIL_VALUE);
        AnyException::from_raw(exc)
    }

    /// Creates a new instance of `Exception` with `message`.
    pub fn new(message: impl Into<String>) -> Self {
        unsafe { Self::of_class(Class::exception(), message) }
    }

    /// Creates a new instance of `class` with `message`.
    ///
    /// # Safety
    ///
    /// The `class` argument must inherit from an exception class.
    pub unsafe fn of_class(
        class: impl Into<Class>,
        message: impl Into<String>,
    ) -> Self {
        Self::cast_unchecked(class.into().new_instance(&[message.into()]))
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

    /// Returns whether `self` is a `StandardError`.
    #[inline]
    pub fn is_standard_error(self) -> bool {
        self.class() == Class::standard_error()
    }

    /// Returns whether `self` is a `SystemExit`.
    #[inline]
    pub fn is_system_exit(self) -> bool {
        self.class() == Class::system_exit()
    }

    /// Returns whether `self` is a `Interrupt`.
    #[inline]
    pub fn is_interrupt(self) -> bool {
        self.class() == Class::interrupt()
    }

    /// Returns whether `self` is a `Signal`.
    #[inline]
    pub fn is_signal(self) -> bool {
        self.class() == Class::signal()
    }

    /// Returns whether `self` is a `Fatal`.
    #[inline]
    pub fn is_fatal(self) -> bool {
        self.class() == Class::fatal()
    }

    /// Returns whether `self` is a `ArgumentError`.
    #[inline]
    pub fn is_arg_error(self) -> bool {
        self.class() == Class::arg_error()
    }

    /// Returns whether `self` is a `EOFError`.
    #[inline]
    pub fn is_eof_error(self) -> bool {
        self.class() == Class::eof_error()
    }

    /// Returns whether `self` is a `IndexError`.
    #[inline]
    pub fn is_index_error(self) -> bool {
        self.class() == Class::index_error()
    }

    /// Returns whether `self` is a `StopIteration`.
    #[inline]
    pub fn is_stop_iteration(self) -> bool {
        self.class() == Class::stop_iteration()
    }

    /// Returns whether `self` is a `KeyError`.
    #[inline]
    pub fn is_key_error(self) -> bool {
        self.class() == Class::key_error()
    }

    /// Returns whether `self` is a `RangeError`.
    #[inline]
    pub fn is_range_error(self) -> bool {
        self.class() == Class::range_error()
    }

    /// Returns whether `self` is a `IOError`.
    #[inline]
    pub fn is_io_error(self) -> bool {
        self.class() == Class::io_error()
    }

    /// Returns whether `self` is a `RuntimeError`.
    #[inline]
    pub fn is_runtime_error(self) -> bool {
        self.class() == Class::runtime_error()
    }

    /// Returns whether `self` is a `FrozenError`.
    #[inline]
    pub fn is_frozen_error(self) -> bool {
        self.class() == Class::frozen_error()
    }

    /// Returns whether `self` is a `SecurityError`.
    #[inline]
    pub fn is_security_error(self) -> bool {
        self.class() == Class::security_error()
    }

    /// Returns whether `self` is a `SystemCallError`.
    #[inline]
    pub fn is_system_call_error(self) -> bool {
        self.class() == Class::system_call_error()
    }

    /// Returns whether `self` is a `ThreadError`.
    #[inline]
    pub fn is_thread_error(self) -> bool {
        self.class() == Class::thread_error()
    }

    /// Returns whether `self` is a `TypeError`.
    #[inline]
    pub fn is_type_error(self) -> bool {
        self.class() == Class::type_error()
    }

    /// Returns whether `self` is a `ZeroDivError`.
    #[inline]
    pub fn is_zero_div_error(self) -> bool {
        self.class() == Class::zero_div_error()
    }

    /// Returns whether `self` is a `NotImpError`.
    #[inline]
    pub fn is_not_imp_error(self) -> bool {
        self.class() == Class::not_imp_error()
    }

    /// Returns whether `self` is a `NoMemError`.
    #[inline]
    pub fn is_no_mem_error(self) -> bool {
        self.class() == Class::no_mem_error()
    }

    /// Returns whether `self` is a `NoMethodError`.
    #[inline]
    pub fn is_no_method_error(self) -> bool {
        self.class() == Class::no_method_error()
    }

    /// Returns whether `self` is a `FloatDomainErr`.
    #[inline]
    pub fn is_float_domain_error(self) -> bool {
        self.class() == Class::float_domain_error()
    }

    /// Returns whether `self` is a `LocalJumpError`.
    #[inline]
    pub fn is_local_jump_error(self) -> bool {
        self.class() == Class::local_jump_error()
    }

    /// Returns whether `self` is a `SysStackError`.
    #[inline]
    pub fn is_sys_stack_error(self) -> bool {
        self.class() == Class::sys_stack_error()
    }

    /// Returns whether `self` is a `RegexpError`.
    #[inline]
    pub fn is_regexp_error(self) -> bool {
        self.class() == Class::regexp_error()
    }

    /// Returns whether `self` is a `EncodingError`.
    #[inline]
    pub fn is_encoding_error(self) -> bool {
        self.class() == Class::encoding_error()
    }

    /// Returns whether `self` is a `EncCompatError`.
    #[inline]
    pub fn is_enc_compat_error(self) -> bool {
        self.class() == Class::enc_compat_error()
    }

    /// Returns whether `self` is a `ScriptError`.
    #[inline]
    pub fn is_script_error(self) -> bool {
        self.class() == Class::script_error()
    }

    /// Returns whether `self` is a `NameError`.
    #[inline]
    pub fn is_name_error(self) -> bool {
        self.class() == Class::name_error()
    }

    /// Returns whether `self` is a `SyntaxError`.
    #[inline]
    pub fn is_syntax_error(self) -> bool {
        self.class() == Class::syntax_error()
    }

    /// Returns whether `self` is a `LoadError`.
    #[inline]
    pub fn is_load_error(self) -> bool {
        self.class() == Class::load_error()
    }

    /// Returns whether `self` is a `MathDomainError`.
    #[inline]
    pub fn is_math_domain_error(self) -> bool {
        self.class() == Class::math_domain_error()
    }
}
