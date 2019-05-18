//! Ruby hash tables.

use std::{
    fmt,
    iter::FromIterator,
    marker::PhantomData,
};
use crate::{
    object::{NonNullObject, Ty},
    prelude::*,
    ruby,
};

/// An instance of Ruby's `Hash` class.
#[repr(transparent)]
pub struct Hash<K = AnyObject, V = AnyObject> {
    inner: NonNullObject,
    _marker: PhantomData<(*mut K, *mut V)>
}

impl<K, V> Clone for Hash<K, V> {
    #[inline]
    fn clone(&self) -> Self { *self }
}

impl<K, V> Copy for Hash<K, V> {}

impl<K: Object, V: Object> AsRef<AnyObject> for Hash<K, V> {
    #[inline]
    fn as_ref(&self) -> &AnyObject { self.inner.as_ref() }
}

impl<K: Object, V: Object> From<Hash<K, V>> for AnyObject {
    #[inline]
    fn from(object: Hash<K, V>) -> AnyObject { object.inner.into() }
}

impl<K: Object, V: Object> PartialEq<AnyObject> for Hash<K, V> {
    #[inline]
    fn eq(&self, obj: &AnyObject) -> bool {
        self.as_any_object() == obj
    }
}

unsafe impl<K: Object, V: Object> Object for Hash<K, V> {
    #[inline]
    fn unique_id() -> Option<u128> {
        Some(!(Ty::Hash as u128))
    }

    #[inline]
    fn cast<A: Object>(obj: A) -> Option<Self> {
        if obj.is_ty(Ty::Hash) {
            unsafe { Some(Self::from_raw(obj.raw())) }
        } else {
            None
        }
    }

    #[inline]
    fn ty(self) -> Ty { Ty::Hash }

    #[inline]
    fn is_ty(self, ty: Ty) -> bool { ty == Ty::Hash }
}

impl<K: Object, V: Object> fmt::Display for Hash<K, V> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_any_object().fmt(f)
    }
}

impl<K: Object, V: Object> fmt::Debug for Hash<K, V> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_tuple("Hash")
            .field(&self.inner)
            .finish()
    }
}

#[cfg(feature = "ruby_2_6")]
impl<K: Object, V: Object> From<&[(K, V)]> for Hash<K, V> {
    #[inline]
    fn from(pairs: &[(K, V)]) -> Self {
        Self::from_pairs(pairs)
    }
}

impl<K1, K2, V1, V2> FromIterator<(K2, V2)> for Hash<K1, V1>
    where K1: Object, K2: Into<K1>, V1: Object, V2: Into<V1>
{
    #[inline]
    fn from_iter<I: IntoIterator<Item = (K2, V2)>>(iter: I) -> Self {
        let hash = Self::new();
        for (key, val) in iter {
            unsafe { hash.insert(key.into(), val.into()) };
        }
        hash
    }
}

impl<K: Object, V: Object> Hash<K, V> {
    /// Creates a new hash table.
    #[inline]
    pub fn new() -> Self {
        unsafe { Self::from_raw(ruby::rb_hash_new()) }
    }

    /// Creates an instance from the key-value pairs in `map`.
    ///
    /// # Examples
    ///
    /// This initializer is general enough to work with most map types. The most
    /// common use case would probably be interacting with Rust's [`HashMap`].
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use std::collections::HashMap;
    /// use rosy::prelude::*;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("is_working", true);
    ///
    /// let hash = Hash::<String, AnyObject>::from_map(&map);
    /// assert_eq!(hash.get("is_working").unwrap(), true);
    /// ```
    ///
    /// [`HashMap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
    #[inline]
    pub fn from_map<'a, M, MK, MV>(map: M) -> Self
    where
        M: IntoIterator<Item = (&'a MK, &'a MV)>,
        MK: Copy + Into<K> + 'a,
        MV: Copy + Into<V> + 'a,
    {
        map.into_iter().map(|(&k, &v)| (k, v)).collect()
    }

    /// Creates a new hash table from `pairs`.
    ///
    #[cfg_attr(not(nightly), doc = "**Requires:** Ruby 2.6+")]
    ///
    /// # Examples
    ///
    /// Although this may insert the objects efficiently, it requires a bit more
    /// verbose code with explicit
    /// [`Into`](https://doc.rust-lang.org/std/convert/trait.Into.html)
    /// conversions from non-Ruby types.
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::{Hash, String};
    ///
    /// let hash = Hash::<String, String>::from_pairs(&[
    ///     ("user".into(), "nvzqz".into()),
    ///     ("name".into(), "Nikolai Vazquez".into()),
    /// ]);
    ///
    /// assert_eq!(hash.get("user").unwrap(), "nvzqz");
    /// assert_eq!(hash.get("name").unwrap(), "Nikolai Vazquez");
    /// ```
    #[cfg(feature = "ruby_2_6")]
    #[cfg_attr(nightly, doc(cfg(feature = "ruby_2_6")))]
    #[inline]
    pub fn from_pairs(pairs: &[(K, V)]) -> Self {
        let hash = Self::new();
        unsafe { hash.insert_pairs(pairs) };
        hash
    }

    /// Associates `val` with `key`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised.
    ///
    /// # Examples
    ///
    /// Rust types can automagically be converted to keys and values:
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::prelude::*;
    ///
    /// let hash = Hash::<String, AnyObject>::new();
    /// unsafe { hash.insert("should_eat", true) };
    ///
    /// assert_eq!(hash.to_s(), r#"{"should_eat"=>true}"#);
    /// ```
    #[inline]
    pub unsafe fn insert(self, key: impl Into<K>, val: impl Into<V>) {
        let key = key.into().raw();
        let val = val.into().raw();
        ruby::rb_hash_aset(self.raw(), key, val);
    }

    /// Inserts `pairs` into `self` in bulk.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised.
    #[cfg(feature = "ruby_2_6")]
    #[cfg_attr(nightly, doc(cfg(feature = "ruby_2_6")))]
    #[inline]
    pub unsafe fn insert_pairs(self, pairs: &[(K, V)]) {
        ruby::rb_hash_bulk_insert_into_st_table(
            (pairs.len() * 2) as _,
            pairs.as_ptr() as *const _,
            self.raw(),
        );
    }

    /// Returns the value for `key`.
    #[inline]
    pub fn get(self, key: impl Into<K>) -> Option<V> {
        let key = key.into().raw();
        unsafe {
            let val = AnyObject::from_raw(ruby::rb_hash_aref(self.raw(), key));
            if val.is_nil() {
                None
            } else {
                Some(V::cast_unchecked(val))
            }
        }
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
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::prelude::*;
    ///
    /// let hash = Hash::<String, AnyObject>::new();
    ///
    /// unsafe {
    ///     assert!(hash.remove("not_here").is_none());
    ///     hash.insert("is_here", true);
    ///     assert_eq!(hash.remove("is_here").unwrap(), true);
    /// }
    /// ```
    #[inline]
    pub unsafe fn remove(self, key: impl Into<K>) -> Option<V> {
        let key = key.into().raw();
        let val = AnyObject::from_raw(ruby::rb_hash_delete(self.raw(), key));
        if val.is_nil() {
            None
        } else {
            Some(V::cast_unchecked(val))
        }
    }

    /// Removes all elements from `self` in-place.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised.
    #[inline]
    pub unsafe fn clear(self) {
        ruby::rb_hash_clear(self.raw());
    }
}
