//! Ruby hash tables.

use crate::object::{Object, AnyObject, Ty};

/// An instance of Ruby's `Hash` class.
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct Hash(AnyObject);

unsafe impl Object for Hash {
    #[inline]
    fn cast(obj: impl Object) -> Option<Self> {
        if obj.is_ty(Ty::Hash) {
            Some(Self::_new(obj.raw()))
        } else {
            None
        }
    }

    #[inline]
    fn ty(self) -> Ty { Ty::Hash }

    #[inline]
    fn is_ty(self, ty: Ty) -> bool { ty == Ty::Hash }
}

impl AsRef<AnyObject> for Hash {
    #[inline]
    fn as_ref(&self) -> &AnyObject { &self.0 }
}

impl From<Hash> for AnyObject {
    #[inline]
    fn from(object: Hash) -> AnyObject { object.0 }
}

impl Hash {
    #[inline]
    pub(crate) fn _new(raw: ruby::VALUE) -> Self {
        Self(AnyObject(raw))
    }
}
