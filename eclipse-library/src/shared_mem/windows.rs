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
 *     IBM Corporation - Initial C implementation and documentation
 *     Max Bureck (Fraunhofer FOKUS)
 *******************************************************************************/

use super::common::{SharedMem, SharedMemRef, ECLIPSE_UNITIALIZED};
use crate::errors::EclipseLibErr;
use crate::native_str_read::utf8_str_to_string;
use std::os::raw::c_char;
use winapi::shared::minwindef::DWORD;
use winapi::shared::ntdef::DWORDLONG;
use winapi::shared::ntdef::HANDLE;
use winapi::um::handleapi::CloseHandle;
use winapi::um::handleapi::DuplicateHandle;
use winapi::um::handleapi::INVALID_HANDLE_VALUE;
use winapi::um::memoryapi::CreateFileMappingW;
use winapi::um::memoryapi::MapViewOfFile;
use winapi::um::memoryapi::UnmapViewOfFile;
use winapi::um::memoryapi::FILE_MAP_WRITE;
use winapi::um::processthreadsapi::GetCurrentProcess;
use winapi::um::processthreadsapi::OpenProcess;
use winapi::um::winnt::DUPLICATE_SAME_ACCESS;
use winapi::um::winnt::PROCESS_ALL_ACCESS;
use winapi::um::winnt::PAGE_READWRITE;

pub struct SharedMemOS {
    map_handle: HANDLE,
    id: String,
    max_len: usize,
}

impl SharedMem for SharedMemOS {
    fn create(mem_size: usize) -> Result<Self, EclipseLibErr> {
        let null_sec = std::ptr::null_mut();
        let null_str = std::ptr::null_mut();
        let max_size = mem_size as DWORD;
        let map_handle_param = unsafe {
            CreateFileMappingW(
                INVALID_HANDLE_VALUE,
                null_sec,
                PAGE_READWRITE,
                0,
                max_size,
                null_str,
            )
        };
        if map_handle_param.is_null() {
            return Err(EclipseLibErr::SharedMemoryInitFail);
        }
        let process_id_param = std::process::id();
        let id_str = if cfg!(target_arch = "x86_64") {
            format!("{:x}_{:x}", process_id_param, map_handle_param as DWORDLONG)
        } else {
            format!("{:x}_{:x}", process_id_param, map_handle_param as DWORD)
        };

        let result = SharedMemOS {
            map_handle: map_handle_param,
            id: id_str,
            max_len: mem_size,
        };

        // Initialize memory
        result.write(ECLIPSE_UNITIALIZED)?;

        Ok(result)
    }

    fn read(&self) -> Result<String, EclipseLibErr> {
        let shared_data = unsafe { MapViewOfFile(self.map_handle, FILE_MAP_WRITE, 0, 0, 0) };

        if shared_data.is_null() {
            return Err(EclipseLibErr::SharedMemoryReadFail);
        }

        let opt_res = utf8_str_to_string(shared_data as *const c_char);
        let res = opt_res.ok_or(EclipseLibErr::SharedMemoryReadInvalidStr)?;

        if unsafe { UnmapViewOfFile(shared_data) } == 0 {
            return Err(EclipseLibErr::SharedMemoryReadFail);
        }

        Ok(res)
    }

    fn write(&self, s: &str) -> Result<(), EclipseLibErr> {
        write_str_to_shared_data(s, self.map_handle, self.max_len)
    }

    fn get_id(&self) -> &str {
        &self.id
    }

    fn close(mut self) -> Result<(), EclipseLibErr> {
        self.close_internal()
    }
}

fn write_str_to_shared_data(
    s: &str,
    map_handle: HANDLE,
    max_len: usize,
) -> Result<(), EclipseLibErr> {
    let shared_data = unsafe { MapViewOfFile(map_handle, FILE_MAP_WRITE, 0, 0, 0) };
    if shared_data.is_null() {
        return Err(EclipseLibErr::SharedMemoryWriteFail);
    }

    let s_raw = s.as_ptr();
    // We do not want to write more bytes than shared mem size and
    // we need space for the terminating 0
    let size_bytes = std::cmp::min(s.len(), max_len - 1);

    let shared_data_chars = shared_data as *mut u8;
    if size_bytes != 0 {
        unsafe { std::ptr::copy(s_raw, shared_data_chars, size_bytes) };
        // rusts string does not end with 0, lets terminate
        let null_pos = shared_data_chars.wrapping_add(size_bytes);
        unsafe { std::ptr::write_bytes(null_pos, 0, 1) };
    } else {
        unsafe { std::ptr::write_bytes(shared_data, 0, 1) };
    }

    if unsafe { UnmapViewOfFile(shared_data) } == 0 {
        return Err(EclipseLibErr::SharedMemoryWriteFail);
    }

    Ok(())
}

impl SharedMemOS {
    fn close_internal(&mut self) -> Result<(), EclipseLibErr> {
        if self.map_handle.is_null() {
            return Ok(());
        }
        let close_result = unsafe { CloseHandle(self.map_handle) };
        self.map_handle = std::ptr::null_mut();
        if close_result != 0 {
            Ok(())
        } else {
            Err(EclipseLibErr::SharedMemoryCloseFail)
        }
    }
}

#[allow(unused_must_use)] // we cannot handle errors in drop
impl Drop for SharedMemOS {
    fn drop(&mut self) {
        self.close_internal();
    }
}

pub struct SharedMemRefOS {
    map_handle: HANDLE,
    max_size: usize,
    close_handle_on_drop: bool,
}

impl SharedMemRef for SharedMemRefOS {
    fn from_id(id: &str, size: usize) -> Result<Self, EclipseLibErr> {
        let mut split = id.splitn(2, '_');

        // Restore process_id
        let first = split.next().ok_or(EclipseLibErr::SharedMemoryIdParseFail)?;
        let process_id =
            u32::from_str_radix(first, 16).map_err(|_| EclipseLibErr::SharedMemoryIdParseFail)?;

        // Restore map_handle
        let second = split.next().ok_or(EclipseLibErr::SharedMemoryIdParseFail)?;
        let map_handle: HANDLE = if cfg!(target_arch = "x86_64") {
            u64::from_str_radix(second, 16).map_err(|_| EclipseLibErr::SharedMemoryIdParseFail)?
                as HANDLE
        } else {
            u32::from_str_radix(second, 16).map_err(|_| EclipseLibErr::SharedMemoryIdParseFail)?
                as HANDLE
        };

        // If this process did not create the map_handle, duplicate the hanlde over
        let current_proc_id = std::process::id();
        let handle = if current_proc_id == process_id {
            map_handle
        } else {
            let mut new_handle: HANDLE = std::ptr::null_mut();
            let new_process_handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, 0, process_id) };
            if new_process_handle.is_null() {
                return Err(EclipseLibErr::SharedMemoryIdParseFail);
            }
            unsafe {
                DuplicateHandle(
                    new_process_handle,
                    map_handle,
                    GetCurrentProcess(),
                    &mut new_handle,
                    DUPLICATE_SAME_ACCESS,
                    0,
                    DUPLICATE_SAME_ACCESS,
                )
            };
            unsafe {
                CloseHandle(new_process_handle);
            }
            new_handle
        };

        if handle.is_null() {
            return Err(EclipseLibErr::SharedMemoryIdParseFail);
        }

        let result = SharedMemRefOS {
            map_handle: handle,
            max_size: size,
            close_handle_on_drop: (handle != map_handle),
        };
        Ok(result)
    }

    fn write(&self, s: &str) -> Result<(), EclipseLibErr> {
        write_str_to_shared_data(s, self.map_handle, self.max_size)
    }

    fn close(mut self) -> Result<(), EclipseLibErr> {
        self.close_internal()
    }

}

impl SharedMemRefOS {
    fn close_internal(&mut self) -> Result<(), EclipseLibErr> {
        if !self.close_handle_on_drop || self.map_handle.is_null() {
            return Ok(());
        }
        let close_result = unsafe { CloseHandle(self.map_handle) };
        self.map_handle = std::ptr::null_mut();
        if close_result != 0 {
            Ok(())
        } else {
            Err(EclipseLibErr::SharedMemoryCloseFail)
        }
    }
}

#[allow(unused_must_use)] // we cannot handle errors in drop
impl Drop for SharedMemRefOS {
    fn drop(&mut self) {
        self.close_internal();
    }
}
