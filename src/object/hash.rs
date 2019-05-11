//! Ruby hash tables.

use crate::object::{Object, AnyObject, NonNullObject, Ty};
use std::{
    fmt,
    iter::FromIterator,
};

/// An instance of Ruby's `Hash` class.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Hash(NonNullObject);

impl AsRef<AnyObject> for Hash {
    #[inline]
    fn as_ref(&self) -> &AnyObject { self.0.as_ref() }
}

impl From<Hash> for AnyObject {
    #[inline]
    fn from(object: Hash) -> AnyObject { object.0.into() }
}

unsafe impl Object for Hash {
    #[inline]
    fn cast(obj: impl Object) -> Option<Self> {
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

impl fmt::Display for Hash {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_any_object().fmt(f)
    }
}

impl<K: Object, V: Object> From<&[(K, V)]> for Hash {
    #[inline]
    fn from(pairs: &[(K, V)]) -> Self {
        Self::from_pairs(pairs)
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
    /// # rosy::init().unwrap();
    /// use std::collections::HashMap;
    /// use rosy::Hash;
    ///
    /// let mut map = HashMap::new();
    /// map.insert("is_working", true);
    ///
    /// let hash = Hash::from_map(&map);
    /// assert_eq!(hash.get("is_working"), true);
    /// ```
    ///
    /// [`HashMap`]: https://doc.rust-lang.org/std/collections/struct.HashMap.html
    #[inline]
    pub fn from_map<'a, I, K, V>(map: I) -> Self
    where
        I: IntoIterator<Item = (&'a K, &'a V)>,
        K: Copy + Into<AnyObject> + 'a,
        V: Copy + Into<AnyObject> + 'a,
    {
        map.into_iter().map(|(&k, &v)| (k, v)).collect()
    }

    /// Creates a new hash table from `pairs`.
    ///
    /// # Examples
    ///
    /// Although this may insert the objects efficiently, it requires a bit more
    /// verbose code.
    ///
    /// However, one can use the [turbofish (`::<>`) syntax](https://turbo.fish)
    /// to allow the compiler to infer types more easily:
    ///
    /// ```
    /// # rosy::init().unwrap();
    /// use rosy::{Hash, String};
    ///
    /// let hash = Hash::from_pairs::<String, String>(&[
    ///     ("user".into(), "nvzqz".into()),
    ///     ("name".into(), "Nikolai Vazquez".into()),
    /// ]);
    ///
    /// assert_eq!(hash.get("user"), "nvzqz");
    /// assert_eq!(hash.get("name"), "Nikolai Vazquez");
    /// ```
    #[inline]
    pub fn from_pairs<K: Object, V: Object>(pairs: &[(K, V)]) -> Self {
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
    #[inline]
    pub fn insert_pairs<K: Object, V: Object>(self, pairs: &[(K, V)]) {
        unsafe { ruby::rb_hash_bulk_insert_into_st_table(
            (pairs.len() * 2) as _,
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
