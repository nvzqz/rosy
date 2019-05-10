//! Ruby mixins.

use crate::{
    object::{Object, AnyObject, Array, symbol::SymbolId},
    util::Sealed,
};

mod class;
mod module;
pub use self::{class::*, module::*};

#[inline]
fn _get_const(m: ruby::VALUE, name: SymbolId) -> Option<AnyObject> {
    unsafe {
        if ruby::rb_const_defined(m, name.raw()) != 0 {
            Some(AnyObject(ruby::rb_const_get(m, name.raw())))
        } else {
            None
        }
    }
}

#[inline]
fn _attr(m: ruby::VALUE, name: SymbolId, read: bool, write: bool) {
    unsafe { ruby::rb_attr(m, name.raw(), read as _, write as _, 0) };
}

/// A type that supports mixins (see [`Class`](struct.Class.html) and
/// [`Module`](struct.Module.html)).
pub trait Mixin: Object + Sealed {
    /// Embeds the contents of `module` in `self`.
    #[inline]
    fn include(self, module: Module) {
        unsafe { ruby::rb_include_module(self.raw(), module.raw()) };
    }

    /// Returns whether `self` or one of its ancestors includes `module`.
    ///
    /// This is equivalent to the `include?` method.
    #[inline]
    #[must_use]
    fn includes(self, module: Module) -> bool {
        unsafe { ruby::rb_mod_include_p(self.raw(), module.raw()) != 0 }
    }

    /// Returns an array of the modules included in `self`.
    #[inline]
    fn included_modules(self) -> Array {
        unsafe { Array::_new(ruby::rb_mod_included_modules(self.raw())) }
    }

    /// Prepends `module` in `self`.
    #[inline]
    fn prepend(self, module: Module) {
        unsafe { ruby::rb_prepend_module(self.raw(), module.raw()) };
    }

    /// Defines a new class under `self` with `name`.
    #[inline]
    fn def_class(
        self,
        name: impl Into<SymbolId>,
    ) -> Result<Class, DefMixinError> {
        Class::_def_under(self.raw(), Class::object(), name.into())
    }

    /// Defines a new subclass of `superclass` under `self` with `name`.
    #[inline]
    fn def_subclass(
        self,
        superclass: Class,
        name: impl Into<SymbolId>,
    ) -> Result<Class, DefMixinError> {
        Class::_def_under(self.raw(), superclass, name.into())
    }

    /// Returns the existing `Class` with `name` in `self`.
    #[inline]
    fn get_class(
        self,
        name: impl Into<SymbolId>,
    ) -> Option<Class> {
        _get_const(self.raw(), name.into())?.to_class()
    }

    /// Defines a new module under `self` with `name`.
    #[inline]
    fn def_module(
        self,
        name: impl Into<SymbolId>,
    ) -> Result<Module, DefMixinError> {
        Module::_def_under(self.raw(), name.into())
    }

    /// Returns the existing `Module` with `name` in `self`.
    #[inline]
    fn get_module(
        self,
        name: impl Into<SymbolId>,
    ) -> Option<Module> {
        _get_const(self.raw(), name.into())?.to_module()
    }

    /// Returns whether a constant for `name` is defined in `self`, or in some
    /// parent class if not `self`.
    #[inline]
    fn has_const(self, name: impl Into<SymbolId>) -> bool {
        unsafe { ruby::rb_const_defined(self.raw(), name.into().raw()) != 0 }
    }

    /// Returns the constant value for `name` in `self`, or in some parent class
    /// if not `self`.
    ///
    /// # Exception Handling
    ///
    /// If `name` is an uninitialized variable, a `NameError` exception will be
    /// raised. If you're unsure whether `name` exists, either check
    /// [`has_const`](#method.has_const) or surround a call to this method in a
    /// `protected` closure.
    #[inline]
    fn get_const(self, name: impl Into<SymbolId>) -> AnyObject {
        unsafe { AnyObject(ruby::rb_const_get(self.raw(), name.into().raw())) }
    }

    /// Sets the value a constant for `name` in `self` to `val`.
    #[inline]
    fn set_const(self, name: impl Into<SymbolId>, val: impl Object) {
        unsafe { ruby::rb_const_set(self.raw(), name.into().raw(), val.raw()) };
    }

    /// Removes the constant value for `name`, returning it.
    ///
    /// # Exception Handling
    ///
    /// If the constant for `name` cannot be removed, an exception is raised.
    #[inline]
    fn remove_const(self, name: impl Into<SymbolId>) -> AnyObject {
        let name = name.into().raw();
        unsafe { AnyObject(ruby::rb_const_remove(self.raw(), name)) }
    }

    /// Returns whether the class-level `var` is defined in `self`.
    #[inline]
    fn has_class_var(self, var: impl Into<SymbolId>) -> bool {
        let t = unsafe { ruby::rb_cvar_defined(self.raw(), var.into().raw()) };
        t == crate::util::TRUE_VALUE
    }

    /// Returns the class-level `var` in `self`.
    ///
    /// # Exception Handling
    ///
    /// If `var` is an uninitialized variable, a `NameError` exception will be
    /// raised. If you're unsure whether `var` exists, either check
    /// [`has_class_var`](#method.has_class_var) or surround a call to this
    /// method in a `protected` closure.
    ///
    /// ```
    /// use rosy::{Class, Object, Mixin, protected};
    /// # rosy::init().unwrap();
    ///
    /// let class = Class::array();
    /// let error = protected(|| class.get_class_var("@@hello")).unwrap_err();
    ///
    /// assert!(error.is_name_error());
    /// ```
    #[inline]
    fn get_class_var(self, var: impl Into<SymbolId>) -> AnyObject {
        let var = var.into().raw();
        unsafe { AnyObject::from_raw(ruby::rb_cvar_get(self.raw(), var)) }
    }

    /// Sets the class-level `var` in `self` to `val`.
    #[inline]
    fn set_class_var(self, var: impl Into<SymbolId>, val: impl Object) {
        unsafe { ruby::rb_cvar_set(self.raw(), var.into().raw(), val.raw()) };
    }

    /// Defines an read-only attribute on `self` with `name`.
    #[inline]
    fn attr_reader(self, name: impl Into<SymbolId>) {
        _attr(self.raw(), name.into(), true, false);
    }

    /// Defines a write-only attribute on `self` with `name`.
    #[inline]
    fn attr_writer(self, name: impl Into<SymbolId>) {
        _attr(self.raw(), name.into(), false, true);
    }

    /// Defines a read-write attribute on `self` with `name`.
    #[inline]
    fn attr_accessor(self, name: impl Into<SymbolId>) {
        _attr(self.raw(), name.into(), true, true);
    }
}

impl Mixin for Class {}

impl Mixin for Module {}

/// An error when attempting to define a [`Mixin`](trait.Mixin.html) type.
#[derive(Debug)]
pub enum DefMixinError {
    /// A class already exists with the same name in the same namespace.
    ExistingClass(Class),
    /// A module already exists with the same name in the same namespace.
    ExistingModule(Module),
    /// Some other constant already exists.
    ExistingConst(AnyObject),
}

impl DefMixinError {
    #[inline]
    fn _get(m: ruby::VALUE, name: SymbolId) -> Option<Self> {
        use ruby::ruby_value_type::*;
        use DefMixinError::*;

        let existing = _get_const(m, name)?;
        let raw = existing.raw();
        let err = match crate::util::value_built_in_type(raw) {
            Some(RUBY_T_MODULE) => ExistingModule(Module::_new(raw)),
            Some(RUBY_T_CLASS)  => ExistingClass(Class::_new(raw)),
            Some(_) | None      => ExistingConst(existing),
        };
        Some(err)
    }

    /// Returns the existing class that was found.
    #[inline]
    pub fn existing_class(&self) -> Option<Class> {
        match *self {
            DefMixinError::ExistingClass(c) => Some(c),
            _ => None,
        }
    }

    /// Returns the existing module that was found.
    #[inline]
    pub fn existing_module(&self) -> Option<Module> {
        match *self {
            DefMixinError::ExistingModule(m) => Some(m),
            _ => None,
        }
    }

    /// Returns the existing constant that was found.
    #[inline]
    pub fn existing_const(&self) -> Option<AnyObject> {
        match *self {
            DefMixinError::ExistingConst(m) => Some(m),
            _ => None,
        }
    }

    /// Returns the existing object that was found.
    #[inline]
    pub fn existing_object(&self) -> AnyObject {
        use DefMixinError::*;
        match *self {
            ExistingModule(m) => m.into_any(),
            ExistingClass(c)  => c.into_any(),
            ExistingConst(c)  => c,
        }
    }
}
