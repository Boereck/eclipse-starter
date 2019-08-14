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

use crate::vm_lookup::JvmLaunchMode;
use eclipse_common::native_str::to_native_str;
use eclipse_common::path_util::strip_unc_prefix;
use std::borrow::Cow;
use std::convert::TryFrom;
use winapi::ctypes::{c_uint, c_void};
use winapi::shared::minwindef::{DWORD, LPVOID};
use winapi::um::winver::{GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW};

/// Holds (language and code page independent) version information of a file.
/// See https://docs.microsoft.com/en-us/windows/win32/api/verrsrc/ns-verrsrc-vs_fixedfileinfo
#[repr(C)]
struct VsFixedFileInfo {
    dw_signature: DWORD,
    dw_struc_version: DWORD,
    dw_file_version_ms: DWORD,
    dw_file_version_ls: DWORD,
    dw_product_version_ms: DWORD,
    dw_product_version_ls: DWORD,
    dw_file_flags_mask: DWORD,
    dw_file_flags: DWORD,
    dw_file_os: DWORD,
    dw_file_type: DWORD,
    dw_file_subtype: DWORD,
    dw_file_date_ms: DWORD,
    dw_file_date_ls: DWORD,
}

// returns true if the JVM version is >= 9, false otherwise
pub fn is_modular_vm(vm_path: &JvmLaunchMode) -> bool {
    let vm = match vm_path {
        JvmLaunchMode::LaunchJni { jni_lib, .. } => jni_lib,
        JvmLaunchMode::LaunchExe { exe, .. } => exe,
    };
    let vm_cow = vm.to_string_lossy();
    let vm_str = strip_unc_prefix(&vm_cow);
    let (_char_vec, vm_native_str) = to_native_str(vm_str);

    let mut handle: DWORD = 0;
    let info_size = unsafe { GetFileVersionInfoSizeW(vm_native_str, &mut handle) };
    if info_size > 0 {
        let size = usize::try_from(info_size).unwrap_or(0);
        let mut info = vec![0u8; size];
        let info_ptr = info.as_mut_ptr() as *mut c_void;
        let success =
            unsafe { GetFileVersionInfoW(vm_native_str, handle, info_size, info_ptr) } != 0;
        if success {
            let mut buffer: LPVOID = info.as_mut_ptr() as LPVOID;
            let mut version_info_size: c_uint = 0;
            let root_block_str = r"\";
            let (_root_block_vec, root_block_native_str) = to_native_str(root_block_str);

            // This does not seem to work for Oracle Java 7 java.dll, but this is not supported by equinox anymore.
            let version_read_succ = unsafe {
                VerQueryValueW(
                    info_ptr,
                    root_block_native_str,
                    &mut buffer,
                    &mut version_info_size,
                )
            } != 0;

            // Did we read at least the size of the struct we expect?
            let buf_is_struct_size =
                std::mem::size_of::<VsFixedFileInfo>() >= version_info_size as usize;

            if version_read_succ && !buffer.is_null() && buf_is_struct_size {
                let file_info_ptr = buffer as *mut VsFixedFileInfo;
                let file_info = unsafe { file_info_ptr.read() };
                let major_version = file_info.dw_product_version_ms >> 16 & 0xffff;
                if major_version >= 9 {
                    return true;
                }
            }
        }
    }
    false
}

pub fn default_vm_args() -> Vec<Cow<'static, str>> {
    Vec::new()
}
