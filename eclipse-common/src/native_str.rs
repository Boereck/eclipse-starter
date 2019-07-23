#[cfg(not(windows))]
use std::os::raw::c_char;

// On Windows we use UTF-16 chars
#[cfg(target_os = "windows")]
pub type NativeString = *const u16;

#[cfg(not(target_os = "windows"))]
pub type NativeString = *const c_char;


#[cfg(target_os="windows")]
pub fn to_native_str(s: &str) -> (Vec<u16>, NativeString) {
    let mut vec: Vec<u16> = s.encode_utf16().collect();
    vec.push(0);
    let ptr = vec.as_ptr();
    (vec, ptr)
}