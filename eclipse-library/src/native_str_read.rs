/*******************************************************************************
 * Copyright (c) 2019 Fraunhofer FOKUS and others.
 *
 * This program and the accompanying materials
 * are made available under the terms of the Eclipse Public License 2.0
 * which accompanies this distribution, and is available at
 * https://www.eclipse.org/legal/epl-2.0/
 *
 * SPDX-License-Identifier: EPL-2.0
 *
 * Contributors:
 *     Max Bureck (Fraunhofer FOKUS)
 *******************************************************************************/

use crate::iter_ptr::iter;
use eclipse_common::native_str::NativeString;
use std::ffi::OsString;
use std::ffi::CStr;
use std::ptr;
use std::os::raw::c_char;
#[cfg(target_os = "windows")]
use std::os::windows::prelude::*;

#[cfg(not(target_os = "windows"))]
pub fn utf8_str_array_to_string_vec(
    native_strings: *mut NativeString,
    count: usize,
) -> Vec<String> {
    iter(native_strings)
        .take(count)
        .filter_map(utf8_str_to_string)
        .collect()
}

#[cfg(not(target_os = "windows"))]
pub fn null_term_utf8_str_array_to_string_vec(native_strings: *mut NativeString) -> Vec<String> {
    if native_strings.is_null() {
        return Vec::default();
    }
    let count = count_ptrs_null_term(native_strings);
    println!("Count is {}", count);
    utf8_str_array_to_string_vec(native_strings, count)
}

pub fn utf8_str_to_string(native_string: *const c_char) -> Option<String> {
    if native_string.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(native_string) }
        .to_str()
        .ok()
        .map(ToOwned::to_owned)
}

#[cfg(target_os = "windows")]
pub fn utf16_str_array_to_string_vec(
    native_strings: *mut NativeString,
    count: usize,
) -> Vec<String> {
    let args_slice: &[NativeString] = unsafe { std::slice::from_raw_parts(native_strings, count) };
    let args_vec: Vec<String> = args_slice.iter().filter_map(utf16_to_string).collect();
    args_vec
}

#[cfg(target_os = "windows")]
pub fn null_term_utf16_str_array_to_string_vec(native_strings: *mut NativeString) -> Vec<String> {
    if native_strings.is_null() {
        return Vec::default();
    }
    let count = count_ptrs_null_term(native_strings);
    utf16_str_array_to_string_vec(native_strings, count)
}

#[cfg(target_os = "windows")]
#[allow(clippy::trivially_copy_pass_by_ref)] // needed to allow function reference in filter_map (see utf16_str_array_to_string_vec)
pub fn utf16_to_string(native_str: &NativeString) -> Option<String> {
    if native_str.is_null() {
        return None;
    }
    // Count character until 0, since string is 0 terminated
    // We have to cast away const-ness to access characters in string
    let str_len = count_u16_null_term(*native_str as *mut u16);
    let char_slice: &[u16] = unsafe { std::slice::from_raw_parts(*native_str, str_len) };
    OsString::from_wide(char_slice)
        .to_str()
        .map(|s| s.to_string())
}

#[cfg(target_os = "windows")]
pub fn count_u16_null_term(elem: *mut u16) -> usize {
    count_elem_null_term(elem, 0)
}

pub fn count_ptrs_null_term<T>(elem: *mut *const T) -> usize {
    count_elem_null_term(elem, ptr::null())
}

#[inline]
pub fn count_elem_null_term<T: Eq>(elem: *mut T, nil: T) -> usize {
    iter(elem).take_while(|c| c != &nil).count()
}
