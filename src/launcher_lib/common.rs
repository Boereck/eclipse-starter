use std::os::raw::c_int;

#[cfg(not(windows))]
use std::os::raw::c_char;

// On Windows we use UTF-16 chars
#[cfg(windows)]
pub(super) type NativeString = *const u16;

#[cfg(not(windows))]
pub(super) type NativeString = *const c_char;

pub(super) type RunMethod = unsafe extern "C" fn(c_int, *const NativeString, *const NativeString) -> c_int;
pub(super) type SetInitialArgs = unsafe extern "C" fn(c_int, *const NativeString, NativeString) -> ();