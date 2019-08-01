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

use crate::params::EclipseParams;
use eclipse_common::native_str::to_native_str;
use eclipse_common::path_buf;
use std::ffi::OsString;
use std::os::windows::prelude::*;
use std::path::{Path, PathBuf};
use winapi::shared::minwindef::{DWORD, HKEY, LPBYTE, LPDWORD, MAX_PATH, PFILETIME};
use winapi::shared::ntdef::LPWSTR;
use winapi::shared::winerror::ERROR_SUCCESS;
use winapi::um::wincon;
use winapi::um::winnt::KEY_READ;
use winapi::um::winreg::{
    RegCloseKey, RegEnumKeyExW, RegOpenKeyExW, RegQueryValueExW, HKEY_LOCAL_MACHINE, LSTATUS,
};

static DEFAULT_VM: &str = "javaw.exe";
static CONSOLE_VM: &str = "java.exe";
pub const VM_LIBRARY: &str = "jvm.dll";
pub const SHIPPED_VM_DIR: &str = r"jre\bin\";

/// Defines default locations in which to find the jvm shared library
/// these are paths relative to the java exe, the shared library is
const JVM_LOCATIONS: [&str; 10] = [
    "j9vm",
    r"..\jre\bin\j9vm",
    "client",
    r"..\jre\bin\client",
    "server",
    r"..\jre\bin\server",
    "classic",
    r"..\jre\bin\classic",
    "jrockit",
    r"..\jre\bin\jrockit",
];

pub fn get_default_vm(params: &EclipseParams) -> &'static str {
    if console_needed(params) {
        CONSOLE_VM
    } else {
        DEFAULT_VM
    }
}

pub fn find_vm_library(vm_exe_path: &Path, exe_dir: &Path) -> Option<PathBuf> {
    let lib = find_lib(vm_exe_path, exe_dir);
    if let Some(lib_path) = lib.as_ref() {
        adjust_search_path(lib_path);
    }
    lib
}

/// Find the VM shared library starting from the java executable.
fn find_lib(vm_exe_path: &Path, exe_dir: &Path) -> Option<PathBuf> {
    // First check if we point to library allready
    if is_vm_library(vm_exe_path) {
        let result = if vm_exe_path.is_file() {
            Some(vm_exe_path.to_path_buf())
        } else {
            None
        };
        return result;
    }

    // Now check if DLL is at known location relative to exe
    if let Some(vm_exe_dir) = vm_exe_path.parent() {
        for vm_location in JVM_LOCATIONS.iter() {
            let vm_path = path_buf![vm_exe_dir, vm_location, VM_LIBRARY,];
            if vm_path.exists() {
                // found library!
                return Some(vm_path);
            }
        }
    }

    // if command is eclipse/jre, return. TODO: why???
    if path_buf![exe_dir, SHIPPED_VM_DIR,] == vm_exe_path {
        return None;
    }

    find_lib_from_registry()
}

fn find_lib_from_registry() -> Option<PathBuf> {
    // Not found yet, try the registry, we will use the first vm >= 1.6
    let (_key_name_vec, jre_key_name) =
        to_native_str(r"Software\JavaSoft\Java Runtime Environment");
    let mut jre_key: HKEY = std::ptr::null_mut();
    let success = (ERROR_SUCCESS as LSTATUS);

    let open_result =
        unsafe { RegOpenKeyExW(HKEY_LOCAL_MACHINE, jre_key_name, 0, KEY_READ, &mut jre_key) };
    if open_result == success {
        let (_current_version_vec, current_vesion_str) = to_native_str("CurrentVersion");
        let null_word: LPDWORD = std::ptr::null_mut();
        let null_str: LPWSTR = std::ptr::null_mut();
        let null_time: PFILETIME = std::ptr::null_mut();
        let mut key_name_buf = [0u16; MAX_PATH];
        let mut length = MAX_PATH as DWORD;
        let key_name_buf_ptr = key_name_buf.as_mut_ptr();

        let query_result = unsafe {
            RegQueryValueExW(
                jre_key,
                current_vesion_str,
                null_word,
                null_word,
                key_name_buf_ptr as LPBYTE,
                &mut length,
            )
        };
        if query_result == success {
            if let Some(path) = check_vm_registry_key(jre_key, key_name_buf) {
                unsafe {
                    RegCloseKey(jre_key);
                }
                return Some(path);
            }
        }

        length = MAX_PATH as DWORD;
        let mut j: DWORD = 0;
        while unsafe {
            RegEnumKeyExW(
                jre_key,
                j,
                key_name_buf_ptr,
                &mut length,
                null_word,
                null_str,
                null_word,
                null_time,
            )
        } == success
        {
            j += 1;
            // look for a 1.6+ vm
            let osstr = OsString::from_wide(&key_name_buf[..(length as usize)]);
            let first_three_chars = &osstr.to_string_lossy()[0..3];
            if first_three_chars >= "1.6" {
                if let Some(path) = check_vm_registry_key(jre_key, key_name_buf) {
                    unsafe {
                        RegCloseKey(jre_key);
                    }
                    return Some(path);
                }
            }
        }

        unsafe {
            RegCloseKey(jre_key);
        }
    }
    None
}

fn check_vm_registry_key(jre_key: HKEY, key_name_buf: [u16; MAX_PATH]) -> Option<PathBuf> {
    unimplemented!()
}

fn is_vm_library(path: &Path) -> bool {
    unimplemented!()
}

fn adjust_search_path(lib_path: &Path) {
    unimplemented!()
}

#[cfg(target_os = "windows")]
fn console_needed(params: &EclipseParams) -> bool {
    params.console.is_set() || params.console_log || is_console_launcher()
}

fn is_console_launcher() -> bool {
    let console_hwnd = unsafe { wincon::GetConsoleWindow() };
    !console_hwnd.is_null()
}
