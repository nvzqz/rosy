//! Metadata for Ruby.

use std::{
    ffi::CStr,
    str,
};
use crate::ruby;

#[inline]
unsafe fn convert_c_str<'a>(s: &'a CStr, debug_message: &str) -> &'a str {
    let bytes = s.to_bytes();
    if cfg!(debug_assertions) {
        str::from_utf8(bytes).expect(debug_message)
    } else {
        str::from_utf8_unchecked(bytes)
    }
}

/// Ruby's API version.
///
/// Note that this may differ from the result of `version_str`.
#[inline]
pub fn api_version() -> (u16, u16, u16) {
    let [major, minor, teeny] = unsafe { ruby::ruby_api_version };
    (major as u16, minor as u16, teeny as u16)
}

/// Ruby's version info as a UTF-8 string.
#[inline]
pub fn version_str<'a>() -> &'a str {
    unsafe { convert_c_str(version_c_str(), "Ruby version is not UTF-8") }
}

/// Ruby's version info as a C string.
#[inline]
pub fn version_c_str<'a>() -> &'a CStr {
    unsafe { CStr::from_ptr(ruby::ruby_version.as_ptr()) }
}

/// Ruby's release date info as a UTF-8 string.
#[inline]
pub fn release_date_str<'a>() -> &'a str {
    unsafe {
        convert_c_str(release_date_c_str(), "Ruby release date is not UTF-8")
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
    unsafe { convert_c_str(platform_c_str(), "Ruby platform is not UTF-8") }
}

/// Ruby's platform info as a C string.
#[inline]
pub fn platform_c_str<'a>() -> &'a CStr {
    unsafe { CStr::from_ptr(ruby::ruby_platform.as_ptr()) }
}

/// Ruby's description info as a UTF-8 string.
#[inline]
pub fn description_str<'a>() -> &'a str {
    unsafe {
        convert_c_str(description_c_str(), "Ruby description is not UTF-8")
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
    unsafe { convert_c_str(copyright_c_str(), "Ruby copyright is not UTF-8") }
}

/// Ruby's copyright info as a C string.
#[inline]
pub fn copyright_c_str<'a>() -> &'a CStr {
    unsafe { CStr::from_ptr(ruby::ruby_copyright.as_ptr()) }
}

/// Ruby's engine info as a UTF-8 string.
#[inline]
pub fn engine_str<'a>() -> &'a str {
    unsafe { convert_c_str(engine_c_str(), "Ruby engine is not UTF-8") }
}

/// Ruby's engine info as a C string.
#[inline]
pub fn engine_c_str<'a>() -> &'a CStr {
    unsafe { CStr::from_ptr(ruby::ruby_engine.as_ptr()) }
}
