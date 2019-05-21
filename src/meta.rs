//! Metadata for Ruby.

use std::{
    ffi::CStr,
    str,
};
use crate::ruby;

/// Ruby's version info as a UTF-8 string.
#[inline]
pub fn version_str<'a>() -> &'a str {
    let bytes = version_c_str().to_bytes();
    if cfg!(debug_assertions) {
        str::from_utf8(bytes).expect("Ruby version is not UTF-8")
    } else {
        unsafe { str::from_utf8_unchecked(bytes) }
    }
}

/// Ruby's version info as a C string.
#[inline]
pub fn version_c_str<'a>() -> &'a CStr {
    unsafe { CStr::from_ptr(ruby::ruby_version.as_ptr()) }
}

/// Ruby's release date info as a UTF-8 string.
#[inline]
pub fn release_date_str<'a>() -> &'a str {
    let bytes = release_date_c_str().to_bytes();
    if cfg!(debug_assertions) {
        str::from_utf8(bytes).expect("Ruby release date is not UTF-8")
    } else {
        unsafe { str::from_utf8_unchecked(bytes) }
    }
}

/// Ruby's release date info as a C string.
#[inline]
pub fn release_date_c_str<'a>() -> &'a CStr {
    unsafe { CStr::from_ptr(ruby::ruby_release_date.as_ptr()) }
}

/// Ruby's platform info as a UTF-8 string.
#[inline]
pub fn platform_str<'a>() -> &'a str {
    let bytes = platform_c_str().to_bytes();
    if cfg!(debug_assertions) {
        str::from_utf8(bytes).expect("Ruby platform is not UTF-8")
    } else {
        unsafe { str::from_utf8_unchecked(bytes) }
    }
}

/// Ruby's platform info as a C string.
#[inline]
pub fn platform_c_str<'a>() -> &'a CStr {
    unsafe { CStr::from_ptr(ruby::ruby_platform.as_ptr()) }
}

/// Ruby's description info as a UTF-8 string.
#[inline]
pub fn description_str<'a>() -> &'a str {
    let bytes = description_c_str().to_bytes();
    if cfg!(debug_assertions) {
        str::from_utf8(bytes).expect("Ruby description is not UTF-8")
    } else {
        unsafe { str::from_utf8_unchecked(bytes) }
    }
}

/// Ruby's description info as a C string.
#[inline]
pub fn description_c_str<'a>() -> &'a CStr {
    unsafe { CStr::from_ptr(ruby::ruby_description.as_ptr()) }
}

/// Ruby's copyright info as a UTF-8 string.
#[inline]
pub fn copyright_str<'a>() -> &'a str {
    let bytes = copyright_c_str().to_bytes();
    if cfg!(debug_assertions) {
        str::from_utf8(bytes).expect("Ruby copyright is not UTF-8")
    } else {
        unsafe { str::from_utf8_unchecked(bytes) }
    }
}

/// Ruby's copyright info as a C string.
#[inline]
pub fn copyright_c_str<'a>() -> &'a CStr {
    unsafe { CStr::from_ptr(ruby::ruby_copyright.as_ptr()) }
}

/// Ruby's engine info as a UTF-8 string.
#[inline]
pub fn engine_str<'a>() -> &'a str {
    let bytes = engine_c_str().to_bytes();
    if cfg!(debug_assertions) {
        str::from_utf8(bytes).expect("Ruby engine is not UTF-8")
    } else {
        unsafe { str::from_utf8_unchecked(bytes) }
    }
}

/// Ruby's engine info as a C string.
#[inline]
pub fn engine_c_str<'a>() -> &'a CStr {
    unsafe { CStr::from_ptr(ruby::ruby_engine.as_ptr()) }
}
