//! Ruby hash tables.

use crate::object::{Object, AnyObject, Ty};
use std::{
    fmt,
    iter::FromIterator,
    collections::HashMap,
};

/// An instance of Ruby's `Hash` class.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Hash(AnyObject);

impl AsRef<AnyObject> for Hash {
    #[inline]
    fn as_ref(&self) -> &AnyObject { &self.0 }
}

impl From<Hash> for AnyObject {
    #[inline]
    fn from(object: Hash) -> AnyObject { object.0 }
}

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

impl fmt::Display for Hash {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_any_object().fmt(f)
    }
}

impl<K: Into<AnyObject>, V: Into<AnyObject>> FromIterator<(K, V)> for Hash {
    #[inline]
    fn from_iter<I: IntoIterator<Item = (K, V)>>(iter: I) -> Self {
        let mut hash = Hash::new();
        hash.extend(iter);
        hash
    }
}

impl<K: Into<AnyObject>, V: Into<AnyObject>> Extend<(K, V)> for Hash {
    fn extend<I: IntoIterator<Item = (K, V)>>(&mut self, iter: I) {
        for (key, val) in iter {
            self.insert(key, val);
        }
    }
}

impl Hash {
    #[inline]
    pub(crate) fn _new(raw: ruby::VALUE) -> Self {
        Self(AnyObject(raw))
    }

    /// Creates a new hash table.
    #[inline]
    pub fn new() -> Self {
        unsafe { Self::_new(ruby::rb_hash_new()) }
    }

    /// Creates a new hash table from `pairs`.
    ///
    /// The caller should ensure that `pairs` is an even number of values. Not
    /// upholding this constraint is considered a programming error.
    #[inline]
    pub fn from_pairs(pairs: &[impl Object]) -> Self {
        let hash = Self::new();
        hash.insert_pairs(pairs);
        hash
    }

    /// Associates `val` with `key`.
    ///
    /// # Examples
    ///
    /// Rust types can automagically be converted to keys and values:
    ///
    /// ```
    /// # rosy::init().unwrap();
    /// use rosy::{Hash, Object};
    ///
    /// let hash = Hash::new();
    /// hash.insert("should_eat", true);
    ///
    /// assert_eq!(hash.to_s(), r#"{"should_eat"=>true}"#);
    /// ```
    #[inline]
    pub fn insert(self, key: impl Into<AnyObject>, val: impl Into<AnyObject>) {
        let key = key.into().raw();
        let val = val.into().raw();
        unsafe { ruby::rb_hash_aset(self.raw(), key, val) };
    }

    /// Inserts `pairs` into `self` in bulk.
    ///
    /// The caller should ensure that `pairs` is an even number of values. Not
    /// upholding this constraint is considered a programming error.
    #[inline]
    pub fn insert_pairs(self, pairs: &[impl Object]) {
        unsafe { ruby::rb_hash_bulk_insert_into_st_table(
            pairs.len() as _,
            pairs.as_ptr() as *const _,
            self.raw(),
        ) };
    }

    /// Returns the value for `key`.
    #[inline]
    pub fn get(self, key: impl Into<AnyObject>) -> AnyObject {
        let key = key.into().raw();
        unsafe { AnyObject::from_raw(ruby::rb_hash_aref(self.raw(), key)) }
    }

    /// Returns the number of key-value pairs in `self`.
    #[inline]
    pub fn len(self) -> usize {
        unsafe { ruby::rb_hash_size_num(self.raw()) }
    }

    /// Returns whether `self` is empty.
    #[inline]
    pub fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Removes the value associated with `key` from `self` and returns it.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::init().unwrap();
    /// let hash = rosy::Hash::new();
    /// assert!(hash.remove("not_here").is_nil());
    ///
    /// hash.insert("is_here", true);
    /// assert_eq!(hash.remove("is_here"), true);
    /// ```
    #[inline]
    pub fn remove(self, key: impl Into<AnyObject>) -> AnyObject {
        let key = key.into().raw();
        unsafe { AnyObject::from_raw(ruby::rb_hash_delete(self.raw(), key)) }
    }

    /// Removes all elements from `self` in-place.
    #[inline]
    pub fn clear(self) {
        unsafe { ruby::rb_hash_clear(self.raw()) };
    }
}
