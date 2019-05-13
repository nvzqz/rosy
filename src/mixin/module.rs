//! Ruby modules.

use std::fmt;
use crate::{
    mixin::DefMixinError,
    prelude::*,
    object::{NonNullObject, Ty},
};

/// An instance of Ruby's `Module` type.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Module(NonNullObject);

impl AsRef<AnyObject> for Module {
    #[inline]
    fn as_ref(&self) -> &AnyObject { self.0.as_ref() }
}

impl From<Module> for AnyObject {
    #[inline]
    fn from(object: Module) -> AnyObject { object.0.into() }
}

unsafe impl Object for Module {
    #[inline]
    fn cast(obj: impl Object) -> Option<Self> {
        if obj.is_ty(Ty::Module) {
            unsafe { Some(Self::cast_unchecked(obj)) }
        } else {
            None
        }
    }

    #[inline]
    fn ty(self) -> Ty { Ty::Module }

    #[inline]
    fn is_ty(self, ty: Ty) -> bool { ty == Ty::Module }
}

impl crate::util::Sealed for Module {}

impl fmt::Display for Module {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_any_object().fmt(f)
    }
}

impl<O: Object> PartialEq<O> for Module {
    #[inline]
    fn eq(&self, other: &O) -> bool {
        self.raw() == other.raw()
    }
}

impl Eq for Module {}

impl Module {
    pub(crate) fn _def_under(
        m: ruby::VALUE,
        name: SymbolId,
    ) -> Result<Self, DefMixinError> {
        if let Some(err) = DefMixinError::_get(m, name) {
            return Err(err);
        }
        let name = name.raw();
        unsafe { Ok(Self::from_raw(ruby::rb_define_module_id_under(m, name))) }
    }

    /// Extends `object` with the contents of `self`.
    #[inline]
    pub fn extend(self, object: impl Object) {
        unsafe { ruby::rb_extend_object(object.raw(), self.raw()) };
    }

    /// Defines a new top-level module with `name`.
    ///
    /// # Examples
    ///
    /// Defining a new module is straightforward:
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// let my_mod = rosy::Module::def("MyMod").unwrap();
    /// ```
    ///
    /// Attempting to define an existing module will result in an error:
    ///
    /// ```
    /// use rosy::Module;
    /// # rosy::vm::init().unwrap();
    ///
    /// let math = Module::def("Math").unwrap_err().existing_object();
    /// assert_eq!(Module::math(), math);
    /// ```
    #[inline]
    pub fn def(name: impl Into<SymbolId>) -> Result<Self, DefMixinError> {
        Class::object().def_module(name)
    }

    /// Retrieves an existing top-level `module` defined by `name`.
    #[inline]
    pub fn get(name: impl Into<SymbolId>) -> Option<Self> {
        Class::object().get_module(name)
    }

    /// Retrieves an existing top-level `Module` defined by `name` or defines
    /// one if it doesn't exist.
    #[inline]
    pub fn get_or_def(name: impl Into<SymbolId>) -> Result<Self, DefMixinError> {
        match Module::def(name) {
            Ok(module) => Ok(module),
            Err(error) => if let Some(module) = error.existing_module() {
                Ok(module)
            } else {
                Err(error)
            }
        }
    }

    /// Returns the name of `self` or `nil` if anonymous.
    #[inline]
    pub fn name(self) -> Option<String> {
        unsafe {
            match ruby::rb_mod_name(self.raw()) {
                crate::util::NIL_VALUE => None,
                raw => Some(String::from_raw(raw)),
            }
        }
    }

    /// Returns the ancestors of this module, including itself.
    #[inline]
    pub fn ancestors(self) -> Array {
        unsafe { Array::from_raw(ruby::rb_mod_ancestors(self.raw())) }
    }
}

macro_rules! built_in_modules {
    ($($vm_name:expr, $method:ident, $konst:ident;)+) => {
        /// Built-in modules.
        impl Module {$(
            /// The `
            #[doc = $vm_name]
            ///` module.
            #[inline]
            pub fn $method() -> Self {
                unsafe { Self::from_raw(ruby::$konst) }
            }
        )+}
    }
}

built_in_modules! {
    "Kernel",       kernel,        rb_mKernel;
    "Comparable",   comparable,    rb_mComparable;
    "Enumerable",   enumerable,    rb_mEnumerable;
    "Errno",        errno,         rb_mErrno;
    "FileTest",     file_test,     rb_mFileTest;
    "GC",           gc,            rb_mGC;
    "Math",         math,          rb_mMath;
    "Process",      process,       rb_mProcess;
    "WaitReadable", wait_readable, rb_mWaitReadable;
    "WaitWritable", wait_writable, rb_mWaitWritable;
}