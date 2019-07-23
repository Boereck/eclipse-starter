#[cfg(not(windows))]
use std::os::raw::c_char;

// On Windows we use UTF-16 chars
#[cfg(target_os = "windows")]
pub type NativeString = *const u16;

#[cfg(not(target_os = "windows"))]
pub type NativeString = *const c_char;


/// Converts a Rust UTF-8 string into a tuple of
/// a vector holding the UTF-16 characters and a 
/// `NativeString`, which is a pointer to these characters.
/// Note that the `NativeString` *must not* outlive the
/// vector. Unfortunately, this cannot be represented in the 
/// type system when using raw pointers.
#[cfg(target_os="windows")]
pub fn to_native_str(s: &str) -> (Vec<u16>, NativeString) {
    let vec = str_to_utf16(s);
    let ptr = vec.as_ptr();
    (vec, ptr)
}

/// Converts a Rust string into a Vec of 
/// null-terminated UTF-16 characters
#[cfg(target_os="windows")]
pub fn str_to_utf16(s: &str) -> Vec<u16> {
    let mut vec: Vec<u16> = s.encode_utf16().collect();
    vec.push(0);
    vec
}