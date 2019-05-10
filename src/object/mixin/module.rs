//! Ruby modules.

use crate::object::{
    AnyObject,
    mixin::{Mixin, Class, DefMixinError},
    Object,
    symbol::SymbolId,
    Ty,
};
use std::fmt;

/// An instance of Ruby's `Module` type.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Module(AnyObject);

unsafe impl Object for Module {
    #[inline]
    fn cast(obj: impl Object) -> Option<Self> {
        if obj.is_ty(Ty::Module) {
            Some(Self::_new(obj.raw()))
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
        self.as_any().fmt(f)
    }
}

impl AsRef<AnyObject> for Module {
    #[inline]
    fn as_ref(&self) -> &AnyObject { &self.0 }
}

impl From<Module> for AnyObject {
    #[inline]
    fn from(object: Module) -> AnyObject { object.0 }
}

impl<O: Object> PartialEq<O> for Module {
    #[inline]
    fn eq(&self, other: &O) -> bool {
        self.raw() == other.raw()
    }
}

impl Eq for Module {}

impl Module {
    #[inline]
    pub(crate) fn _new(raw: ruby::VALUE) -> Self {
        Self(AnyObject(raw))
    }

    pub(crate) fn _def_under(
        m: ruby::VALUE,
        name: SymbolId,
    ) -> Result<Module, DefMixinError> {
        if let Some(err) = DefMixinError::_get(m, name) {
            return Err(err);
        }
        let raw = unsafe { ruby::rb_define_module_id_under(m, name.raw()) };
        Ok(Module::_new(raw))
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
    /// # rosy::init().unwrap();
    /// let my_mod = rosy::Module::def("MyMod").unwrap();
    /// ```
    ///
    /// Attempting to define an existing module will result in an error:
    ///
    /// ```
    /// use rosy::Module;
    /// # rosy::init().unwrap();
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
                Self::_new(unsafe { ruby::$konst })
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
