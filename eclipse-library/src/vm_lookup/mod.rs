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

#[cfg_attr(target_os = "macos", path = "macos.rs")]
#[cfg_attr(target_os = "linux", path = "linux.rs")]
#[cfg_attr(target_os = "windows", path = "windows.rs")]
mod os;

use std::path::{Path, PathBuf};
use crate::params::EclipseParams;
use crate::errors::EclipseLibErr;
use eclipse_common::path_util::check_path;
use eclipse_common::exe_util::find_program;
use eclipse_common::path_buf;

static DEFAULT_EE: &str = "default.ee";

pub fn determine_vm(params: &EclipseParams, program_dir: &Path) -> Result<JvmLaunchMode, EclipseLibErr> {
    let default_vm = os::get_default_vm(params);
    if let Some(vm_name) = params.vm.as_ref() {
        let vm_path = Path::new(vm_name);
        let check_program_dir_first = true;
        let vm_path = check_path(vm_path, program_dir, check_program_dir_first);
        let vm_type = determine_provided_vm_type(&vm_path);

        use VmType::*; 
        match vm_type {
            VM_DIRECTORY => get_vm_from_dir(vm_name, program_dir),
            VM_EE_PROPS => get_ee_vm(vm_name),
            VM_LIBRARY => get_vm_library(vm_name),
            _ => get_vm_exe(vm_name),
        }
    } else {
        find_jvm(false)
    }
}

fn get_vm_from_dir(vm_name: &str, program_dir: &Path) -> Result<JvmLaunchMode, EclipseLibErr> {
    // look for default.ee
    let mut vm_path = path_buf![vm_name, DEFAULT_EE,];
    let vm_program = find_program(&vm_path);

    if let Some(vm_program_path) = vm_program {
        let launch_mode = launch_mode_from_jvm_exe_path(vm_program_path, program_dir);
        Ok(launch_mode)
    } else {
        find_jvm(false)
    }
}

fn get_ee_vm(vm_name: &str) -> Result<JvmLaunchMode, EclipseLibErr> {
    unimplemented!()
}

fn get_vm_library(vm_name: &str) -> Result<JvmLaunchMode, EclipseLibErr> {
    unimplemented!()
}

fn get_vm_exe(vm_name: &str) -> Result<JvmLaunchMode, EclipseLibErr> {
    // TODO: if not found, call find_jvm(true)
    unimplemented!()
}

fn launch_mode_from_jvm_exe_path(jvm_exe_path: PathBuf, program_dir: &Path) -> JvmLaunchMode {
    if cfg!(not(feature = "default_java_exec")) {
        if let Some(jvm_lib_path) = os::find_vm_library(&jvm_exe_path, program_dir) {
            return JvmLaunchMode::LAUNCH_JNI {
                jni_lib: jvm_lib_path,
            }
        }
    }
    JvmLaunchMode::LAUNCH_EXE {
        exe: jvm_exe_path,
    }
}

#[allow(clippy::if_same_then_else)] // concatenating these large expressions with an || is not readable
fn determine_provided_vm_type(vm_path: &Path) -> VmType {
    use VmType::*; 

    if vm_path.is_dir() {
        return VM_DIRECTORY;
    }

    let ext = vm_path.extension().unwrap_or_default().to_str().unwrap_or_default();
    let is_win = cfg!(target_os = "windows");

    if is_win && ext == "dll" {
        VM_LIBRARY
    } else if (!is_win) && (ext == "so" || ext == "jnilib" || ext == "dylib") {
        VM_LIBRARY
    } else if ext == "ee" {
        VM_EE_PROPS
    } else {
        VmType::VM_OTHER
    }
}

fn find_jvm(user_specified_vm: bool) -> Result<JvmLaunchMode, EclipseLibErr> {
    
    unimplemented!()
}

pub enum JvmLaunchMode {
    LAUNCH_JNI {jni_lib: PathBuf},
    LAUNCH_EXE {exe: PathBuf}
}

enum VmType {
    VM_DIRECTORY,
    VM_NOTHING,
    VM_LIBRARY,
    VM_EE_PROPS,
    VM_OTHER,
}