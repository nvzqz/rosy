//! Ruby mixins.

use crate::{
    prelude::*,
    ruby,
    util::Sealed,
};

mod class;
mod method;
mod module;
pub use self::{class::*, method::*, module::*};

#[inline]
fn _get_const(m: impl Mixin, name: SymbolId) -> Option<AnyObject> {
    unsafe {
        if ruby::rb_const_defined(m.raw(), name.raw()) != 0 {
            Some(_get_const_unchecked(m, name))
        } else {
            None
        }
    }
}

#[inline]
unsafe fn _get_const_unchecked(m: impl Mixin, name: impl Into<SymbolId>) -> AnyObject {
    AnyObject::from_raw(ruby::rb_const_get(m.raw(), name.into().raw()))
}

#[inline]
fn _attr(m: ruby::VALUE, name: SymbolId, read: bool, write: bool) {
    unsafe { ruby::rb_attr(m, name.raw(), read as _, write as _, 0) };
}

/// A type that supports mixins (see [`Class`](struct.Class.html) and
/// [`Module`](struct.Module.html)).
pub trait Mixin: Object + Sealed {
    /// Returns `self` as a `Class` if it is one or a `Module` otherwise.
    fn to_class(self) -> Result<Class, Module>;

    /// Returns `self` as a `Module` if it is one or a `Class` otherwise.
    fn to_module(self) -> Result<Module, Class>;

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
        unsafe { Array::from_raw(ruby::rb_mod_included_modules(self.raw())) }
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
        Class::_def_under(self, Class::object(), name.into())
    }

    /// Defines a new subclass of `superclass` under `self` with `name`.
    #[inline]
    fn def_subclass(
        self,
        superclass: Class,
        name: impl Into<SymbolId>,
    ) -> Result<Class, DefMixinError> {
        Class::_def_under(self, superclass, name.into())
    }

    /// Returns the existing `Class` with `name` in `self`.
    #[inline]
    fn get_class(
        self,
        name: impl Into<SymbolId>,
    ) -> Option<Class> {
        _get_const(self, name.into())?.to_class()
    }

    /// Returns the existing `Class` with `name` in `self`.
    ///
    /// # Safety
    ///
    /// This method does not:
    /// - Check whether an item for `name` exists (an exception will be thrown
    ///   if this is the case)
    /// - Check whether the returned item for `name` is actually a `Class`
    #[inline]
    unsafe fn get_class_unchecked(
        self,
        name: impl Into<SymbolId>,
    ) -> Class {
        Class::cast_unchecked(_get_const_unchecked(self, name))
    }

    /// Defines a new module under `self` with `name`.
    #[inline]
    fn def_module(
        self,
        name: impl Into<SymbolId>,
    ) -> Result<Module, DefMixinError> {
        Module::_def_under(self, name.into())
    }

    /// Returns the existing `Module` with `name` in `self`.
    #[inline]
    fn get_module(
        self,
        name: impl Into<SymbolId>,
    ) -> Option<Module> {
        _get_const(self, name.into())?.to_module()
    }

    /// Returns the existing `Module` with `name` in `self`.
    ///
    /// # Safety
    ///
    /// This method does not:
    /// - Check whether an item for `name` exists (an exception will be thrown
    ///   if this is the case)
    /// - Check whether the returned item for `name` is actually a `Module`
    #[inline]
    unsafe fn get_module_unchecked(
        self,
        name: impl Into<SymbolId>,
    ) -> Module {
        Module::cast_unchecked(_get_const_unchecked(self, name))
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
        let name = name.into().raw();
        unsafe { AnyObject::from_raw(ruby::rb_const_get(self.raw(), name)) }
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
        unsafe { AnyObject::from_raw(ruby::rb_const_remove(self.raw(), name)) }
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
    /// # rosy::vm::init().unwrap();
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

    /// Evaluates `args` in the context of `self`.
    ///
    /// See the docs for `EvalArgs` for more info.
    ///
    /// # Safety
    ///
    /// An exception may be raised by the code or by `args` being invalid.
    #[inline]
    unsafe fn eval_unchecked(self, args: impl EvalArgs) -> AnyObject {
        args.eval_in_unchecked(self)
    }

    /// Evaluates `args` in the context of `self`, returning any raised
    /// exceptions.
    ///
    /// See the docs for `EvalArgs` for more info.
    #[inline]
    fn eval(self, args: impl EvalArgs) -> Result<AnyObject, AnyException> {
        args.eval_in(self)
    }
}

impl Mixin for Class {
    #[inline]
    fn to_class(self) -> Result<Class, Module> {
        Ok(self)
    }

    #[inline]
    fn to_module(self) -> Result<Module, Class> {
        Err(self)
    }
}

impl Mixin for Module {
    #[inline]
    fn to_class(self) -> Result<Class, Module> {
        Err(self)
    }

    #[inline]
    fn to_module(self) -> Result<Module, Class> {
        Ok(self)
    }
}

/// A type that can be used as one or more arguments for evaluating code within
/// the context of a [`Mixin`](trait.Mixin.html).
///
/// See the documentation of [its implementors](#foreign-impls) for much more
/// detailed information.
pub trait EvalArgs: Sized {
    /// Evaluates `self` in the context of `mixin`, returning any thrown
    /// exceptions.
    #[inline]
    fn eval_in(self, mixin: impl Mixin) -> Result<AnyObject, AnyException> {
        crate::protected(|| unsafe { self.eval_in_unchecked(mixin) })
    }

    /// Evaluates `self` in the context of `mixin`.
    ///
    /// # Safety
    ///
    /// If an exception is thrown due to an argument error or from evaluating
    /// the script itself, it should be caught.
    unsafe fn eval_in_unchecked(self, mixin: impl Mixin) -> AnyObject;
}

/// Unchecked arguments directly to the evaluation function.
impl<O: Object> EvalArgs for &[O] {
    #[inline]
    unsafe fn eval_in_unchecked(self, mixin: impl Mixin) -> AnyObject {
        let raw = ruby::rb_mod_module_eval(
            self.len() as _,
            self.as_ptr() as *const ruby::VALUE,
            mixin.raw(),
        );
        AnyObject::from_raw(raw)
    }
}

/// The script argument without any extra information.
impl EvalArgs for String {
    #[inline]
    unsafe fn eval_in_unchecked(self, mixin: impl Mixin) -> AnyObject {
        self.as_any_slice().eval_in_unchecked(mixin)
    }
}

/// The script argument as a UTF-8 string, without any extra information.
// TODO: Impl for `Into<String>` when specialization stabilizes
impl EvalArgs for &str {
    #[inline]
    unsafe fn eval_in_unchecked(self, mixin: impl Mixin) -> AnyObject {
        String::from(self).eval_in_unchecked(mixin)
    }
}

/// The script and filename arguments.
impl<S: Into<String>, F: Into<String>> EvalArgs for (S, F) {
    #[inline]
    unsafe fn eval_in_unchecked(self, mixin: impl Mixin) -> AnyObject {
        let (s, f) = self;
        [s.into(), f.into()].eval_in_unchecked(mixin)
    }
}

/// The script, filename, and line number arguments.
impl<S: Into<String>, F: Into<String>, L: Into<u32>> EvalArgs for (S, F, L) {
    #[inline]
    unsafe fn eval_in_unchecked(self, _mixin: impl Mixin) -> AnyObject {
        unimplemented!("TODO: Convert u32 to object");
    }
}

/// An error when attempting to define a [`Mixin`](trait.Mixin.html) type.
#[derive(Debug)]
pub enum DefMixinError {
    /// A class already exists with the same name in the same namespace.
    ExistingClass(Class),
    /// A module already exists with the same name in the same namespace.
    ExistingModule(Module),
    /// Some other constant already exists.
    ExistingConst(AnyObject),
    /// The given class is frozen and can't have items defined under it.
    FrozenClass(Class),
    /// The given module is frozen and can't have items defined under it.
    FrozenModule(Module),
}

impl DefMixinError {
    #[cold]
    #[inline]
    pub(crate) fn _frozen(m: impl Mixin) -> Self {
        match m.to_class() {
            Ok(class) => DefMixinError::FrozenClass(class),
            Err(module) => DefMixinError::FrozenModule(module),
        }
    }

    #[inline]
    fn _get(m: impl Mixin, name: SymbolId) -> Option<Self> {
        use ruby::value_type::*;
        use DefMixinError::*;

        let existing = _get_const(m, name)?;
        let raw = existing.raw();
        let err = match crate::util::value_built_in_type(raw) {
            Some(MODULE) => unsafe {
                ExistingModule(Module::from_raw(raw))
            },
            Some(CLASS) => unsafe {
                ExistingClass(Class::from_raw(raw))
            },
            Some(_) | None => ExistingConst(existing),
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
    pub fn existing_object(&self) -> Option<AnyObject> {
        use DefMixinError::*;
        match *self {
            ExistingModule(m) => Some(m.into_any_object()),
            ExistingClass(c)  => Some(c.into_any_object()),
            ExistingConst(c)  => Some(c),
            _ => None,
        }
    }
}
