//! Ruby strings.

use std::{
    borrow::Cow,
    cmp::Ordering,
    convert::{TryFrom, TryInto},
    error::Error,
    ffi::{CStr, CString, FromBytesWithNulError},
    fmt,
    os::raw::{c_int, c_long},
    str::Utf8Error,
    string,
};
use crate::{
    object::{NonNullObject, Ty},
    prelude::*,
    ruby,
};

/// An instance of Ruby's `String` class.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct String(NonNullObject);

impl fmt::Display for String {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        unsafe { self.to_str_lossy().fmt(f) }
    }
}

unsafe impl Object for String {
    #[inline]
    fn unique_id() -> Option<u128> {
        Some(!(Ty::String as u128))
    }

    #[inline]
    fn cast<A: Object>(obj: A) -> Option<Self> {
        Some(obj.to_s())
    }

    #[inline]
    fn ty(self) -> Ty { Ty::String }

    #[inline]
    fn is_ty(self, ty: Ty) -> bool { ty == Ty::String }
}

impl AsRef<AnyObject> for String {
    #[inline]
    fn as_ref(&self) -> &AnyObject { self.0.as_ref() }
}

impl From<String> for AnyObject {
    #[inline]
    fn from(object: String) -> AnyObject { object.0.into() }
}

impl From<&str> for String {
    #[inline]
    fn from(s: &str) -> String {
        unsafe { String::from_raw(ruby::rb_utf8_str_new(
            s.as_ptr() as *const _,
            s.len() as _,
        )) }
    }
}

impl From<&str> for AnyObject {
    #[inline]
    fn from(s: &str) -> Self {
        String::from(s).into()
    }
}

impl From<&CStr> for String {
    #[inline]
    fn from(s: &CStr) -> String {
        s.to_bytes().into()
    }
}

impl From<&CStr> for AnyObject {
    #[inline]
    fn from(s: &CStr) -> Self {
        String::from(s).into()
    }
}

impl From<&[u8]> for String {
    #[inline]
    fn from(bytes: &[u8]) -> String {
        let ptr = bytes.as_ptr();
        let len = bytes.len();
        unsafe { String::from_raw(ruby::rb_str_new(ptr as *const _, len as _)) }
    }
}

impl From<&[u8]> for AnyObject {
    #[inline]
    fn from(s: &[u8]) -> Self {
        String::from(s).into()
    }
}

impl TryFrom<String> for std::string::String {
    type Error = Utf8Error;

    #[inline]
    fn try_from(s: String) -> Result<Self, Self::Error> {
        s.to_string()
    }
}

// Make fast byte comparison version of `PartialEq<Self>` when specialization is
// made stable
impl<'r, O: Object> PartialEq<O> for String {
    // If `obj` is not an instance of `String` but responds to `to_str`, then
    // the two strings are compared using `obj.==`.
    #[inline]
    fn eq(&self, obj: &O) -> bool {
        let this = self.raw();
        let that = obj.raw();
        unsafe { ruby::rb_str_equal(this, that) != crate::util::FALSE_VALUE }
    }
}

// Implements `PartialEq` against all relevant string-related types
macro_rules! impl_eq {
    ($($t:ty, $bytes:ident;)+) => { $(
        impl PartialEq<$t> for String {
            #[inline]
            fn eq(&self, other: &$t) -> bool {
                // Safe because no other thread can access the bytes
                unsafe { self.as_bytes() == other.$bytes() }
            }
        }

        // Needed to prevent conflict with `PartialEq<impl Object>`
        impl PartialEq<&$t> for String {
            #[inline]
            fn eq(&self, other: &&$t) -> bool {
                *self == **other
            }
        }

        impl PartialEq<String> for $t {
            #[inline]
            fn eq(&self, other: &String) -> bool {
                other == self
            }
        }

        impl PartialEq<String> for &$t {
            #[inline]
            fn eq(&self, other: &String) -> bool {
                other == self
            }
        }
    )+ }
}

impl_eq! {
    [u8],           as_ref;
    Vec<u8>,        as_slice;
    str,            as_bytes;
    string::String, as_bytes;
    CStr,           to_bytes;
    CString,        to_bytes;
}

impl<S: ?Sized + Clone> PartialEq<Cow<'_, S>> for String
    where String: PartialEq<S>
{
    #[inline]
    fn eq(&self, other: &Cow<'_, S>) -> bool {
        self == AsRef::<S>::as_ref(other)
    }
}

impl Eq for String {}

impl PartialOrd for String {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for String {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        unsafe { ruby::rb_str_cmp(self.raw(), other.raw()).cmp(&0) }
    }
}

impl String {
    #[inline]
    pub(crate) fn rstring(self) -> *mut ruby::RString {
        self.as_any_object()._ptr() as _
    }

    /// Returns a new instance from `s` encoded as `enc`.
    ///
    /// # Safety
    ///
    /// Care must be taken to ensure that the bytes are actually encoded this
    /// way. Otherwise, Ruby may make incorrect assumptions about the underlying
    /// data.
    #[inline]
    pub unsafe fn with_encoding(s: impl AsRef<[u8]>, enc: Encoding) -> Self {
        let s = s.as_ref();
        String::from_raw(ruby::rb_external_str_new_with_enc(
            s.as_ptr() as *const _,
            s.len() as _,
            enc._enc(),
        ))
    }

    /// Returns how the bytes of `self` are encoded.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// let string = rosy::String::from("¡Hola!");
    /// assert!(string.encoding().is_utf8());
    /// ```
    #[inline]
    pub fn encoding(self) -> Encoding {
        unsafe { Encoding::_from_index(ruby::rb_enc_get_index(self.raw())) }
    }

    /// Returns a reference to the underlying bytes in `self`.
    ///
    /// # Safety
    ///
    /// Care must be taken to ensure that the length of `self` and the bytes
    /// pointed to by `self` are not changed through the VM or otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// let rs = "Hey, I just met you, and this is crazy;
    ///           but here's my number, so call me maybe.";
    /// let rb = rosy::String::from(rs);
    ///
    /// unsafe { assert_eq!(rs.as_bytes(), rb.as_bytes()) };
    /// ```
    #[inline]
    pub unsafe fn as_bytes(&self) -> &[u8] {
        let ptr = (*self.rstring()).start() as *const u8;
        std::slice::from_raw_parts(ptr, self.len())
    }

    /// Returns a buffer of the underlying bytes in `self`.
    #[inline]
    pub fn to_bytes(self) -> Vec<u8> {
        unsafe { self.as_bytes().into() }
    }

    /// Returns a reference to the underlying UTF-8 encoded string in `self`.
    ///
    /// # Safety
    ///
    /// Care must be taken to ensure that the length of `self` and the
    /// characters pointed to by `self` are not changed through the VM or
    /// otherwise.
    ///
    /// If Ruby believes that the underlying encoding is indeed UTF-8, then we
    /// return the bytes directly without any further checking. However, if the
    /// method `force_encoding` has been called on `self`, then we are
    /// susceptible to getting invalid UTF-8 in a `str` instance, which is UB.
    /// To force a check, one should call
    /// [`str::from_utf8`](https://doc.rust-lang.org/std/str/fn.from_utf8.html)
    /// on the result of [`as_bytes`](#method.as_bytes).
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// let rs = "Somebody once told me the world is gonna roll me...";
    /// let rb = rosy::String::from(rs);
    ///
    /// unsafe { assert_eq!(rb.to_str().unwrap(), rs) };
    /// ```
    pub unsafe fn to_str(&self) -> Result<&str, Utf8Error> {
        if self.encoding().is_utf8() {
            return Ok(self.to_str_unchecked());
        }
        std::str::from_utf8(self.as_bytes())
    }

    /// Returns the underlying string lossy-encoded as UTF-8. See
    /// [`String::from_utf8_lossy`](https://doc.rust-lang.org/std/string/struct.String.html#method.from_utf8_lossy)
    /// for more details.
    ///
    /// # Safety
    ///
    /// Care must be taken to ensure that, if the returned value is a reference
    /// to `self`, the length of `self` and the characters pointed to by `self`
    /// are not changed through the VM or otherwise.
    ///
    /// If Ruby believes that the underlying encoding is indeed UTF-8, then we
    /// return the bytes directly without any further checking. However, if the
    /// method `force_encoding` has been called on `self`, then we are
    /// susceptible to getting invalid UTF-8 in a `str` instance, which is UB.
    /// To force a check, one should call
    /// [`str::from_utf8`](https://doc.rust-lang.org/std/str/fn.from_utf8.html)
    /// on the result of [`as_bytes`](#method.as_bytes).
    pub unsafe fn to_str_lossy(&self) -> Cow<'_, str> {
        if self.encoding().is_utf8() {
            return Cow::Borrowed(self.to_str_unchecked());
        }
        std::string::String::from_utf8_lossy(self.as_bytes())
    }

    /// Returns a reference to the underlying bytes of `self` as if they were
    /// UTF-8 encoded.
    ///
    /// # Safety
    ///
    /// Same reasons as [`to_str`](#method.to_str) as well as that no UTF-8
    /// checking is performed.
    #[inline]
    pub unsafe fn to_str_unchecked(&self) -> &str {
        std::str::from_utf8_unchecked(self.as_bytes())
    }

    /// Returns a buffer of the underlying UTF-8 encoded string of `self`.
    #[inline]
    pub fn to_string(self) -> Result<std::string::String, Utf8Error> {
        unsafe { Ok(self.to_str()?.into()) }
    }

    /// Returns the number of bytes in `self`.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// let s1 = "Í'm in Rüby!";
    /// let s2 = "I'm in Ruby!";
    /// let s3 = rosy::String::from(s1);
    ///
    /// assert_eq!(s3.len(), s1.len());
    /// assert_ne!(s3.len(), s2.len());
    /// ```
    #[inline]
    pub fn len(self) -> usize {
        unsafe { (*self.rstring()).len() }
    }

    /// Returns the number of characters in `self`.
    ///
    /// # Examples
    ///
    /// This is a [Unicode](https://en.wikipedia.org/wiki/Unicode)-aware method:
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// let s1 = "Í'm in Rüby!";
    /// let s2 = "I'm in Ruby!";
    /// let s3 = rosy::String::from(s1);
    ///
    /// assert_eq!(s3.char_len(), s1.chars().count());
    /// assert_eq!(s3.char_len(), s2.chars().count());
    /// ```
    #[inline]
    pub fn char_len(self) -> usize {
        unsafe { ruby::rb_str_strlen(self.raw()) as usize }
    }

    /// Concatenates `c` to `self`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised.
    #[inline]
    pub unsafe fn push(self, c: char) {
        self.push_str(c.encode_utf8(&mut [0; 4]))
    }

    /// Concatenates `s` to `self`.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `self` is not frozen or else a `FrozenError`
    /// exception will be raised.
    #[inline]
    pub unsafe fn push_str(self, s: &str) {
        ruby::rb_str_cat(self.raw(), s.as_ptr() as _, s.len() as _);
    }

    /// Duplicates the contents of `self` into a new instance.
    #[inline]
    pub fn duplicate(self) -> Self {
        unsafe { Self::from_raw(ruby::rb_str_dup(self.raw())) }
    }

    /// Returns the contents of `self` with an ellipsis (three dots) if it's
    /// longer than `len` _characters_.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// let s1 = rosy::String::from("Hello, there!");
    /// let s2 = s1.ellipsized(8);
    ///
    /// assert_eq!(s2, "Hello...");
    /// ```
    #[inline]
    pub fn ellipsized(self, len: usize) -> Self {
        if len > c_long::max_value() as usize {
            // Avoid an exception for going negative
            return self.duplicate();
        }
        let len = len as c_long;
        unsafe { Self::from_raw(ruby::rb_str_ellipsize(self.raw(), len)) }
    }

    /// Returns whether the string is locked by the VM.
    #[inline]
    pub fn is_locked(self) -> bool {
        unsafe { (*self.rstring()).is_locked() }
    }

    /// Attempts to call `f` if a lock on `self` can be acquired, returning its
    /// output on success.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// let s = rosy::String::from("Hello!");
    /// let result = s.with_lock(|s| s.is_locked());
    ///
    /// assert_eq!(result, Some(true));
    /// ```
    #[inline]
    #[must_use]
    pub fn with_lock<F, O>(self, f: F) -> Option<O>
        where F: FnOnce(Self) -> O
    {
        if self.is_locked() {
            return None;
        }
        unsafe { self.raw_lock() };
        let output = f(self);
        unsafe { self.raw_unlock() };
        Some(output)
    }

    /// Locks the string, preventing others from writing to it.
    ///
    /// # Safety
    ///
    /// The exception raised by the VM must be handled if the string is already
    /// locked.
    #[inline]
    pub unsafe fn raw_lock(self) {
        ruby::rb_str_locktmp(self.raw());
    }

    /// Unlocks the string, allowing others to write to it.
    ///
    /// # Safety
    ///
    /// The exception raised by the VM must be handled if the string is already
    /// unlocked.
    #[inline]
    pub unsafe fn raw_unlock(self) {
        ruby::rb_str_unlocktmp(self.raw());
    }
}

/// An encoding for `String`.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Encoding(NonNullObject);

impl AsRef<AnyObject> for Encoding {
    #[inline]
    fn as_ref(&self) -> &AnyObject { self.0.as_ref() }
}

impl From<Encoding> for AnyObject {
    #[inline]
    fn from(object: Encoding) -> AnyObject { object.0.into() }
}

impl PartialEq<AnyObject> for Encoding {
    #[inline]
    fn eq(&self, obj: &AnyObject) -> bool {
        self.as_any_object() == obj
    }
}

unsafe impl Object for Encoding {
    #[inline]
    fn cast<A: Object>(obj: A) -> Option<Self> {
        if obj.class().inherits(Class::encoding()) {
            unsafe { Some(Self::cast_unchecked(obj)) }
        } else {
            None
        }
    }
}

impl fmt::Display for Encoding {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_any_object().fmt(f)
    }
}

impl PartialEq for Encoding {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self._enc() == other._enc()
    }
}

impl Eq for Encoding {}

impl TryFrom<&CStr> for Encoding {
    type Error = EncodingLookupError;

    #[inline]
    fn try_from(s: &CStr) -> Result<Self, Self::Error> {
        let index = unsafe { ruby::rb_enc_find_index(s.as_ptr()) };
        if index < 0 {
            Err(EncodingLookupError::UnknownName)
        } else {
            Ok(Encoding::_from_index(index))
        }
    }
}

impl TryFrom<&[u8]> for Encoding {
    type Error = EncodingLookupError;

    #[inline]
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        CStr::from_bytes_with_nul(bytes)?.try_into()
    }
}

impl TryFrom<&str> for Encoding {
    type Error = EncodingLookupError;

    #[inline]
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.as_bytes().try_into()
    }
}

impl Encoding {
    #[inline]
    pub(crate) fn _from_enc(enc: *mut ruby::rb_encoding) -> Self {
        unsafe { Self::from_raw(ruby::rb_enc_from_encoding(enc)) }
    }

    #[inline]
    pub(crate) fn _from_index(i: c_int) -> Self {
        unsafe { Self::_from_enc(ruby::rb_enc_from_index(i)) }
    }

    #[inline]
    pub(crate) fn _rdata(self) -> *mut ruby::RData {
        self.as_any_object()._ptr() as _
    }

    #[inline]
    pub(crate) fn _enc(self) -> *mut ruby::rb_encoding {
        unsafe {
            let enc = (*self._rdata()).data as *mut ruby::rb_encoding;
            debug_assert_eq!(enc, ruby::rb_to_encoding(self.raw()));
            enc
        }
    }

    #[inline]
    pub(crate) fn _index(self) -> c_int {
        unsafe { ruby::rb_enc_to_index(self._enc()) }
    }

    /// Returns the `ASCII-8BIT` encoding.
    ///
    /// # Examples
    ///
    /// This is essentially an "anything goes" encoding:
    ///
    /// ```
    /// use rosy::string::{String, Encoding};
    ///
    /// # rosy::vm::init().unwrap();
    /// let bytes: &[u8] = &[b'a', b'z', 0, 255];
    /// let string = String::from(bytes);
    ///
    /// assert_eq!(string.encoding(), Encoding::ascii_8bit());
    /// ```
    #[inline]
    pub fn ascii_8bit() -> Encoding {
        unsafe { Encoding::_from_enc(ruby::rb_ascii8bit_encoding()) }
    }

    /// Returns the `UTF-8` encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::string::Encoding;
    ///
    /// let utf8 = Encoding::find("UTF-8\0").unwrap();
    /// assert_eq!(utf8, Encoding::utf8());
    /// ```
    #[inline]
    pub fn utf8() -> Encoding {
        unsafe { Encoding::_from_enc(ruby::rb_utf8_encoding()) }
    }

    /// Returns the `US-ASCII` encoding.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::string::Encoding;
    ///
    /// let ascii = Encoding::find("US-ASCII\0").unwrap();
    /// assert_eq!(ascii, Encoding::us_ascii());
    /// ```
    #[inline]
    pub fn us_ascii() -> Encoding {
        unsafe { Encoding::_from_enc(ruby::rb_usascii_encoding()) }
    }

    /// Attempts to find `encoding`, returning an error if either:
    /// - `encoding` cannot be passed in as a nul-terminated C string.
    /// - The requested encoding was not found.
    ///
    /// # Examples
    ///
    /// Looking up an encoding is straightforward since Rust allows for
    /// embedding nul bytes in its UTF-8 strings:
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::string::Encoding;
    ///
    /// let utf8 = Encoding::find("UTF-8\0").unwrap();
    /// let ascii = Encoding::find("US-ASCII\0").unwrap();
    ///
    /// assert_ne!(utf8, ascii);
    /// ```
    #[inline]
    pub fn find<E>(encoding: E) -> Result<Self, EncodingLookupError>
        where E: TryInto<Self, Error=EncodingLookupError>
    {
        encoding.try_into()
    }

    /// Returns the encoding's name.
    ///
    /// # Examples
    ///
    /// ```
    /// # rosy::vm::init().unwrap();
    /// use rosy::string::Encoding;
    ///
    /// assert_eq!(Encoding::utf8().name().to_bytes(), b"UTF-8");
    /// ```
    #[inline]
    pub fn name(&self) -> &CStr {
        unsafe { CStr::from_ptr((*self._enc()).name) }
    }

    /// Returns whether `self` is `ASCII-8BIT`.
    #[inline]
    pub fn is_ascii_8bit(self) -> bool {
        unsafe { self._index() == ruby::rb_ascii8bit_encindex() }
    }

    /// Returns whether `self` is `UTF-8`.
    #[inline]
    pub fn is_utf8(self) -> bool {
        unsafe { self._index() == ruby::rb_utf8_encindex() }
    }

    /// Returns whether `self` is `US-ASCII`.
    #[inline]
    pub fn is_us_ascii(self) -> bool {
        unsafe { self._index() == ruby::rb_usascii_encindex() }
    }

    /// Returns whether `self` is the locale encoding.
    #[inline]
    pub fn is_locale(self) -> bool {
        unsafe { self._index() == ruby::rb_locale_encindex() }
    }

    /// Returns whether `self` is the filesystem encoding.
    #[inline]
    pub fn is_filesystem(self) -> bool {
        unsafe { self._index() == ruby::rb_filesystem_encindex() }
    }

    /// Returns whether `self` is the default external encoding.
    #[inline]
    pub fn is_default_external(self) -> bool {
        unsafe { self._enc() == ruby::rb_default_external_encoding() }
    }

    /// Returns whether `self` is the default internal encoding.
    #[inline]
    pub fn is_default_internal(self) -> bool {
        unsafe { self._enc() == ruby::rb_default_internal_encoding() }
    }
}

/// The error returned when [`Encoding::find`](struct.Encoding.html#method.find)
/// fails.
#[derive(Debug)]
pub enum EncodingLookupError {
    /// The encoding name could not be found.
    UnknownName,
    /// The encoding name string was not C-compatible.
    InvalidCStr(FromBytesWithNulError),
}

impl Error for EncodingLookupError {
    #[inline]
    fn description(&self) -> &str {
        use EncodingLookupError::*;
        match self {
            UnknownName => "Unknown encoding name",
            InvalidCStr(error) => error.description(),
        }
    }
}

impl fmt::Display for EncodingLookupError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use EncodingLookupError::*;
        match self {
            UnknownName => self.description().fmt(f),
            InvalidCStr(error) => error.fmt(f),
        }
    }
}

impl From<FromBytesWithNulError> for EncodingLookupError {
    #[inline]
    fn from(error: FromBytesWithNulError) -> Self {
        EncodingLookupError::InvalidCStr(error)
    }
}

#[cfg(all(test, nightly))]
mod benches {
    use test::{Bencher, black_box};
    use super::*;

    const STRING_MULTIPLE: usize = 10;

    fn create_string() -> String {
        let mut string = std::string::String::new();
        for _ in 0..STRING_MULTIPLE {
            string.push('a');
            string.push('ñ');
            string.push('ß');
        }
        String::from(&*string)
    }

    #[bench]
    fn to_str(b: &mut Bencher) {
        crate::vm::init().unwrap();

        let string = create_string();

        b.bytes = string.len() as u64;
        b.iter(move || unsafe {
            let f = black_box(String::to_str);
            let _ = black_box(f(&black_box(string)));
        });
    }

    #[bench]
    fn to_str_checked(b: &mut Bencher) {
        crate::vm::init().unwrap();

        let string = create_string();
        let enc = String::from("ASCII-8BIT");
        string.call_with("force_encoding", &[enc]).unwrap();

        b.bytes = string.len() as u64;
        b.iter(move || unsafe {
            let f = black_box(String::to_str);
            let _ = black_box(f(&black_box(string)));
        });
    }

    #[bench]
    fn to_str_no_lookup(b: &mut Bencher) {
        crate::vm::init().unwrap();

        unsafe fn to_str(s: &String) -> Result<&str, Utf8Error> {
            std::str::from_utf8(s.as_bytes())
        }

        let string = create_string();

        b.bytes = string.len() as u64;
        b.iter(move || unsafe {
            let f = black_box(to_str);
            let _ = black_box(f(&black_box(string)));
        });
    }
}
