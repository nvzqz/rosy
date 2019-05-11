use std::{
    fmt,
    num::NonZeroUsize,
    mem,
};
use crate::AnyObject;

// A `AnyObject` instance that doesn't use 0 (reserved for `false`). This type
// is used as a memory optimization for objects wrapped in `Option`.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct NonNullObject(NonZeroUsize);

impl From<NonNullObject> for AnyObject {
    #[inline]
    fn from(obj: NonNullObject) -> Self {
        // Transmute to ensure the same size is used
        unsafe { mem::transmute(obj) }
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
