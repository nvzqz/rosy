use std::{
    ffi::c_void,
    fmt,
    marker::PhantomData,
    num::NonZeroUsize,
};
use crate::{
    AnyObject,
    ruby,
};

// An `AnyObject` instance that doesn't use 0 (reserved for `false`). This type
// is used as a memory optimization for objects wrapped in `Option`.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct NonNullObject {
    raw: NonZeroUsize,
    // !Send + !Sync
    _marker: PhantomData<*const c_void>,
}

impl From<NonNullObject> for AnyObject {
    #[inline]
    fn from(obj: NonNullObject) -> Self {
        unsafe { AnyObject::from_raw(obj.raw.into()) }
    }
}

impl AsRef<AnyObject> for NonNullObject {
    #[inline]
    fn as_ref(&self) -> &AnyObject {
        unsafe { &*(self as *const Self as *const AnyObject) }
    }
}

impl fmt::Debug for NonNullObject {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        AnyObject::from(*self).fmt(f)
    }
}

impl NonNullObject {
    #[inline]
    pub const unsafe fn from_raw(raw: ruby::VALUE) -> Self {
        NonNullObject {
            raw: NonZeroUsize::new_unchecked(raw),
            _marker: PhantomData,
        }
    }

    #[inline]
    pub const fn raw(self) -> usize {
        self.raw.get()
    }
}
