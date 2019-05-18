use std::{
    convert::{TryFrom, TryInto},
    error::Error,
    ffi::{CStr, CString, FromBytesWithNulError},
    fmt,
    os::raw::c_int,
};
use crate::{
    object::NonNullObject,
    prelude::*,
    ruby,
};

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

impl TryFrom<&CString> for Encoding {
    type Error = EncodingLookupError;

    #[inline]
    fn try_from(s: &CString) -> Result<Self, Self::Error> {
        s.as_c_str().try_into()
    }
}

impl TryFrom<&[u8]> for Encoding {
    type Error = EncodingLookupError;

    #[inline]
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        CStr::from_bytes_with_nul(bytes)?.try_into()
    }
}

impl TryFrom<&Vec<u8>> for Encoding {
    type Error = EncodingLookupError;

    #[inline]
    fn try_from(bytes: &Vec<u8>) -> Result<Self, Self::Error> {
        bytes.as_slice().try_into()
    }
}

impl TryFrom<&str> for Encoding {
    type Error = EncodingLookupError;

    #[inline]
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        s.as_bytes().try_into()
    }
}

impl TryFrom<&std::string::String> for Encoding {
    type Error = EncodingLookupError;

    #[inline]
    fn try_from(s: &std::string::String) -> Result<Self, Self::Error> {
        s.as_str().try_into()
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
        unsafe { (*self._enc()).index() }
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
        self._index() == ruby::rb_encoding::ascii_8bit_index()
    }

    /// Returns whether `self` is `UTF-8`.
    #[inline]
    pub fn is_utf8(self) -> bool {
        self._index() == ruby::rb_encoding::utf8_index()
    }

    /// Returns whether `self` is `US-ASCII`.
    #[inline]
    pub fn is_us_ascii(self) -> bool {
        self._index() == ruby::rb_encoding::us_ascii_index()
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
