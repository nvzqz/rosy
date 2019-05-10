//! Ruby exceptions.

use crate::object::{Object, AnyObject, Array, Class};
use std::fmt;

/// Some concrete Ruby exception.
pub trait Exception: Object {
    /// Returns `self` as an [`AnyException`](struct.AnyException.html).
    #[inline]
    fn into_any_exception(self) -> AnyException { *self.as_any_exception() }

    /// Returns a reference to `self` as an `AnyObject`.
    #[inline]
    fn as_any_exception(&self) -> &AnyException {
        unsafe { &*(self as *const Self as *const AnyException) }
    }

    /// Returns a backtrace associated with `self`.
    ///
    /// The array contains strings
    #[inline]
    fn backtrace(&self) -> Option<Array> {
        let value = self.call("backtrace");
        if value.is_nil() { None } else { Some(Array::_new(value.raw())) }
    }

    /// The underlying exception that caused `self`.
    #[inline]
    fn cause(&self) -> Option<AnyException> {
        let cause = self.call("cause");
        if cause.is_nil() { None } else { Some(AnyException(cause)) }
    }
}

/// Any Ruby exception.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct AnyException(AnyObject);

unsafe impl Object for AnyException {
    #[inline]
    fn cast(obj: impl Object) -> Option<Self> {
        if obj.class().inherits(Class::exception()) {
            Some(Self::_new(obj.raw()))
        } else {
            None
        }
    }
}

impl Exception for AnyException {

}

impl<E: Exception> PartialEq<E> for AnyException {
    #[inline]
    fn eq(&self, other: &E) -> bool {
        self.raw() == other.raw()
    }
}

impl Eq for AnyException {}

impl fmt::Debug for AnyException {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("AnyException")
            .field(&self.0.raw())
            .finish()
    }
}

impl AnyException {
    #[inline]
    pub(crate) fn _new(raw: ruby::VALUE) -> Self {
        Self(AnyObject(raw))
    }

    // Returns the current exception without checking, clearing it globally
    #[inline]
    pub(crate) unsafe fn _take_current() -> AnyException {
        let exc = ruby::rb_errinfo();
        ruby::rb_set_errinfo(crate::util::NIL_VALUE);
        AnyException::_new(exc)
    }

    /// Returns the current pending exception.
    #[inline]
    pub fn current() -> Option<AnyException> {
        match unsafe { ruby::rb_errinfo() } {
            crate::util::NIL_VALUE => None,
            raw => Some(AnyException::_new(raw)),
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
