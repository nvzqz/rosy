//! Ruby modules.

use std::fmt;
use crate::{
    mixin::DefMixinError,
    object::{NonNullObject, Ty},
    prelude::*,
    ruby,
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
    fn unique_id() -> Option<u128> {
        Some(!(Ty::MODULE.id() as u128))
    }

    #[inline]
    fn cast<A: Object>(obj: A) -> Option<Self> {
        if obj.is_ty(Ty::MODULE) {
            unsafe { Some(Self::cast_unchecked(obj)) }
        } else {
            None
        }
    }

    #[inline]
    fn ty(self) -> Ty { Ty::MODULE }

    #[inline]
    fn is_ty(self, ty: Ty) -> bool { ty == Ty::MODULE }
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
        m: impl Mixin,
        name: SymbolId,
    ) -> Result<Self, DefMixinError> {
        if let Some(err) = DefMixinError::_get(m, name) {
            return Err(err);
        } else if m.is_frozen() {
            return Err(DefMixinError::_frozen(m));
        }
        unsafe {
            let raw = ruby::rb_define_module_id_under(m.raw(), name.raw());
            Ok(Self::from_raw(raw))
        }
    }

    // monomorphization
    fn _extend(self, object: AnyObject) -> Result {
        unsafe { crate::protected_no_panic(|| self.extend_unchecked(object)) }
    }

    /// Extends `object` with the contents of `self`.
    #[inline]
    pub fn extend(self, object: impl Object) -> Result {
        self._extend(object.into())
    }

    /// Extends `object` with the contents of `self`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised.
    #[inline]
    pub unsafe fn extend_unchecked(self, object: impl Object) {
        ruby::rb_extend_object(object.raw(), self.raw());
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
    /// assert_eq!(Module::math(), math.unwrap());
    /// ```
    #[inline]
    pub fn def(name: impl Into<SymbolId>) -> Result<Self, DefMixinError> {
        Class::object().def_module(name)
    }

    /// Retrieves an existing top-level `Module` defined by `name`.
    #[inline]
    pub fn get(name: impl Into<SymbolId>) -> Option<Self> {
        Class::object().get_module(name)
    }

    /// Retrieves an existing top-level `Module` defined by `name`.
    ///
    /// # Safety
    ///
    /// This method does not:
    /// - Check whether an item for `name` exists (an exception will be thrown
    ///   if this is the case)
    /// - Check whether the returned item for `name` is actually a `Module`
    #[inline]
    pub unsafe fn get_unchecked(name: impl Into<SymbolId>) -> Self {
        Class::object().get_module_unchecked(name)
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
    pub fn ancestors(self) -> Array<Module> {
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
