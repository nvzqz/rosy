use std::{
    ffi::c_void,
    fmt,
    marker::PhantomData,
    ptr,
};
use crate::{
    object::NonNullObject,
    prelude::*,
    ruby::{self, rb_data_type_t, rb_data_type_t_function},
};

/// An instance of a Ruby object that wraps around Rust data.
///
/// See the documentation for `Rosy` for more information.
#[repr(transparent)]
pub struct RosyObject<R: Rosy> {
    inner: NonNullObject,
    _marker: PhantomData<R>,
}

impl<R: Rosy> Clone for RosyObject<R> {
    #[inline]
    fn clone(&self) -> Self { *self }
}

impl<R: Rosy> Copy for RosyObject<R> {}

impl<R: Rosy> AsRef<AnyObject> for RosyObject<R> {
    #[inline]
    fn as_ref(&self) -> &AnyObject {
        self.inner.as_ref()
    }
}

impl<R: Rosy> From<RosyObject<R>> for AnyObject {
    #[inline]
    fn from(obj: RosyObject<R>) -> Self {
        obj.inner.into()
    }
}

unsafe impl<R: Rosy> Object for RosyObject<R> {
    #[inline]
    fn cast(obj: impl Object) -> Option<Self> {
        if obj.class() == R::class() {
            unsafe { Some(Self::cast_unchecked(obj)) }
        } else {
            None
        }
    }

    #[inline]
    fn class(self) -> Class {
        unsafe { Class::from_raw((*self.rdata()).basic.klass) }
    }
}

impl<R: Rosy> From<Box<R>> for RosyObject<R> {
    #[inline]
    fn from(rosy: Box<R>) -> Self {
        let rosy = Box::into_raw(rosy) as *mut c_void;
        let ty = RosyObject::<R>::data_type();
        let class = R::class().raw();
        unsafe {
            Self::from_raw(ruby::rb_data_typed_object_wrap(class, rosy, ty))
        }
    }
}

impl<R: Rosy> From<R> for RosyObject<R> {
    #[inline]
    fn from(rosy: R) -> Self {
        Box::new(rosy).into()
    }
}

impl<R: Rosy> fmt::Debug for RosyObject<R> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_any_object().fmt(f)
    }
}

impl<R: Rosy> fmt::Display for RosyObject<R> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_any_object().fmt(f)
    }
}

impl<R: Rosy> RosyObject<R> {
    #[inline]
    pub(crate) fn data_type() -> &'static rb_data_type_t {
        unsafe extern "C" fn dmark<R: Rosy>(rosy: *mut c_void) {
            (&mut *(rosy as *mut R)).mark();
        }
        unsafe extern "C" fn dfree<R: Rosy>(rosy: *mut c_void) {
            Box::from_raw(rosy as *mut R).free();
        }
        unsafe extern "C" fn dsize<R: Rosy>(rosy: *const c_void) -> usize {
            (&*(rosy as *const R)).size()
        }
        &rb_data_type_t {
            wrap_struct_name: R::ID,
            function: rb_data_type_t_function {
                dmark: Some(dmark::<R>),
                dfree: Some(dfree::<R>),
                dsize: Some(dsize::<R>),
                reserved: [ptr::null_mut(); 2],
            },
            parent: ptr::null(),
            data: ptr::null_mut(),
            flags: ruby::RUBY_TYPED_FREE_IMMEDIATELY,
        }
    }

    #[inline]
    fn rdata(self) -> *mut ruby::RData {
        self.raw() as *mut ruby::RData
    }

    #[inline]
    fn data(self) -> *mut R {
        unsafe { (*self.rdata()).data as *mut R }
    }

    /// Returns a reference to the inner `Rosy` value.
    #[inline]
    pub fn as_data(&self) -> &R {
        unsafe { &*self.data() }
    }
}
