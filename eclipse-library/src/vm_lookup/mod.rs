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

mod common;
#[cfg_attr(target_os = "macos", path = "macos.rs")]
#[cfg_attr(target_os = "linux", path = "linux.rs")]
#[cfg_attr(target_os = "windows", path = "windows.rs")]
mod os;

use crate::ee_params_parse::*;
use crate::errors::EclipseLibErr;
use crate::params::EclipseParams;
use common::is_vm_library_ext;
use eclipse_common::exe_util::{find_program, find_program_path, strip_path};
use eclipse_common::option_util::opt_str;
use eclipse_common::path_buf;
use eclipse_common::path_util::check_path;
use os::{console_needed, find_vm_library, SHIPPED_VM_DIR, VM_LIBRARY};
use std::path::{Path, PathBuf};

static DEFAULT_EE: &str = "default.ee";
static MSG_VM_REQUIRED: &str = r"A Java Runtime Environment (JRE) or Java Development Kit (JDK)
must be available in order to run";
static MSG_NO_VM_FOUND_AT: &str = r"No Java virtual machine
was found after searching the following locations:";

/// Tries to find a JVM based on the `params` and the `program_dir`
/// (where the launcher executable is located). If a JVM was found, a
/// `JvmLaunchMode` will be returned holding the information of the
/// JVM to start. If no JVM was found the function will return a
/// `EclipseLibErr::JvmNotFound`.
pub fn determine_vm(
    params: &EclipseParams,
    program_dir: &Path,
) -> Result<JvmLaunchMode, EclipseLibErr> {
    if let Some(vm_name) = opt_str(&params.vm) {
        let vm_path = Path::new(strip_path(vm_name));
        let check_program_dir_first = true;
        let vm_path = check_path(vm_path, program_dir, check_program_dir_first);
        let vm_type = determine_provided_vm_type(&vm_path);

        use VmType::*;
        match vm_type {
            Directory => get_vm_from_dir(&vm_path, program_dir, params),
            EeProps => get_ee_vm(&vm_path, program_dir, params),
            Library => get_vm_library(vm_name, &vm_path, program_dir, params),
            // otherwise, assume executable
            _ => get_vm_exe(vm_name, &vm_path, program_dir, params),
        }
    } else {
        let default_vm = os::get_default_vm(params);
        find_jvm(program_dir, None, default_vm, params)
    }
}

fn get_vm_from_dir(
    vm_dir: &Path,
    program_dir: &Path,
    params: &EclipseParams,
) -> Result<JvmLaunchMode, EclipseLibErr> {
    // look for default.ee
    let ee_vm_path = path_buf![vm_dir, DEFAULT_EE,];
    let ee_vm_program = find_program_path(&ee_vm_path);

    if let Some(ee_vm_program_path) = ee_vm_program {
        // default.ee does exist
        return get_ee_vm(ee_vm_program_path, program_dir, params);
    }
    // No default.ee file, look for default VM

    let default_vm = os::get_default_vm(params);
    let default_vm_command = path_buf![vm_dir, default_vm,];
    if let Ok(java_vm) = std::fs::canonicalize(&default_vm_command) {
        // go on with java exe command
        let result = launch_mode_from_jvm_exe_path(java_vm, program_dir, params);
        Ok(result)
    } else {
        // No vm executable, look for library
        let lib_path = path_buf![vm_dir, VM_LIBRARY,];
        let found_lib_path = find_vm_library(&lib_path, program_dir, params, None);
        found_lib_path
            .map(|p| {
                // JNI library found
                JvmLaunchMode::LaunchJni { jni_lib: p, add_vm_args: Vec::new() }
            })
            .ok_or_else(|| {
                // found nothing, return error
                no_vm_found_err(params, &[&ee_vm_path, &default_vm_command, &lib_path])
            })
    }
}

fn get_ee_vm<P: AsRef<Path>>(
    ee_path_source: P,
    program_dir: &Path,
    params: &EclipseParams,
) -> Result<JvmLaunchMode, EclipseLibErr> {
    let ee_path = ee_path_source.as_ref();
    let ee_lines = read_ee_file(ee_path).map_err(|_| no_vm_found_err(params, &[ee_path]))?;
    let parsed_props = parse_ee_params(ee_path, &ee_lines).map_err(|_| no_vm_found_err(params, &[ee_path]))?;
    let ee_vm_args = parsed_props.to_vm_command_line_args();
    
    if let Some(lib_path_str) = opt_str(&parsed_props.ee_vm_libary) {
        let lib_path = Path::new(lib_path_str);
        let vm_lib_opt = os::find_vm_library(lib_path, program_dir, params, Some(&parsed_props));
        if let Some(vm_lib) = vm_lib_opt {
            let result = JvmLaunchMode::LaunchJni { jni_lib: vm_lib, add_vm_args:  ee_vm_args};
            return Ok(result);
        }
    }

    if console_needed(params) {
        if let Some(console_path_str) = opt_str(&parsed_props.ee_console) {
            let vm_path_opt = find_program(console_path_str);
            if let Some(vm_path) = vm_path_opt {
                let result = JvmLaunchMode::LaunchExe { exe: vm_path, add_vm_args: ee_vm_args };
                return Ok(result);
            }
        }
    }
    if let Some(vm_path_str) = opt_str(&parsed_props.ee_executable) {
        let vm_path_opt = find_program(vm_path_str);
        if let Some(vm_path) = vm_path_opt {
            let result = JvmLaunchMode::LaunchExe { exe: vm_path, add_vm_args: ee_vm_args };
            return Ok(result);
        }
    }

    let err = no_vm_found_err(params, &[ee_path]);
    Err(err)
}

fn get_vm_library(
    vm_name: &str,
    lib_path: &Path,
    program_dir: &Path,
    params: &EclipseParams,
) -> Result<JvmLaunchMode, EclipseLibErr> {
    let lib_path_resolved = find_program_path(lib_path)
        .and_then(|path| os::find_vm_library(&path, program_dir, params, None));
    let result_lib_path = lib_path_resolved.ok_or_else(|| {
        let lookup_path = if !vm_name.contains(std::path::MAIN_SEPARATOR) {
            Path::new(vm_name)
        } else {
            lib_path
        };
        no_vm_found_err(params, &[lookup_path])
    })?;
    let result = JvmLaunchMode::LaunchJni {
        jni_lib: result_lib_path,
        add_vm_args: Vec::new(),
    };
    Ok(result)
}

fn get_vm_exe(
    vm_name: &str,
    vm_path: &Path,
    program_dir: &Path,
    params: &EclipseParams,
) -> Result<JvmLaunchMode, EclipseLibErr> {
    let resolved_vm_path = find_program_path(vm_path).ok_or_else(|| {
        // file didn't exist, error
        // if vm_name doesn't contain a dirSeparator, we looked on the path
        let lookup_path = if !vm_name.contains(std::path::MAIN_SEPARATOR) {
            Path::new(vm_name)
        } else {
            vm_path
        };
        no_vm_found_err(params, &[lookup_path])
    })?;
    // right now, we are always doing JNI on Mac
    if cfg!(target_os = "macos") {
        let result = launch_mode_from_jvm_exe_path(resolved_vm_path, program_dir, params);
        Ok(result)
    } else {
        let result = JvmLaunchMode::LaunchExe {
            exe: resolved_vm_path,
            add_vm_args: Vec::new(),
        };
        Ok(result)
    }
}

fn launch_mode_from_jvm_exe_path(
    jvm_exe_path: PathBuf,
    program_dir: &Path,
    params: &EclipseParams,
) -> JvmLaunchMode {
    if cfg!(not(feature = "default_java_exec")) {
        if let Some(jvm_lib_path) = os::find_vm_library(&jvm_exe_path, program_dir, params, None) {
            return JvmLaunchMode::LaunchJni {
                jni_lib: jvm_lib_path,
                add_vm_args: Vec::new(),
            };
        }
    }
    JvmLaunchMode::LaunchExe { exe: jvm_exe_path,
        add_vm_args: Vec::new(), }
}

/// Depending if the given `vm_path` is a directory,
/// or the file extension, a `VmType` is selected and returned.
fn determine_provided_vm_type(vm_path: &Path) -> VmType {
    use VmType::*;

    if vm_path.is_dir() {
        return Directory;
    }

    let ext = vm_path
        .extension()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();

    if is_vm_library_ext(ext) {
        Library
    } else if ext == "ee" {
        EeProps
    } else {
        Other
    }
}

/// Tries to look up JVM in `program_dir` or globally on search path, if the given `user_specified_vm`
/// is `None`.
/// Note that if `user_specified_vm` is `Some`, that path must exist.
fn find_jvm(
    program_dir: &Path,
    user_specified_vm: Option<PathBuf>,
    default_vm: &str,
    params: &EclipseParams,
) -> Result<JvmLaunchMode, EclipseLibErr> {
    let (vm_path, vm_loopup_path) = if user_specified_vm == None {
        // no vm specified, Try to find the VM shipped with eclipse.
        let default_ee_path = path_buf![program_dir, DEFAULT_EE,];
        if let Some(ee_path) = find_program_path(default_ee_path) {
            let ee_result = get_ee_vm(ee_path, program_dir, params);
            if ee_result.is_ok() {
                return ee_result;
            }
        }
        // not found yet: then look for java(w).exe
        let lookup_path = path_buf![program_dir, SHIPPED_VM_DIR, default_vm,];
        let vm_path = find_program_path(&lookup_path);
        (vm_path, Some(lookup_path))
    } else {
        (user_specified_vm, None)
    };

    let default_vm_path = Path::new(default_vm);
    let java_vm_result = vm_path
        .or_else(|| {
            // vm not found yet, look for one on the search path
            find_program_path(default_vm_path)
        })
        .ok_or_else(|| {
            // JVM nowhere found, construct error with paths we looked at
            let mut lookup_paths = vec![default_vm_path];
            if let Some(ref path) = vm_loopup_path {
                lookup_paths.push(path);
            }
            no_vm_found_err(params, &lookup_paths)
        });

    // last straw on windows: lookup library, we may find DLL in registry
    if cfg!(target_os = "windows") && java_vm_result.is_err() {
        let lib_result = os::find_vm_library(Path::new(""), program_dir, params, None);
        if let Some(lib_path) = lib_result {
            let result = JvmLaunchMode::LaunchJni { jni_lib: lib_path, add_vm_args: Vec::new(), };
            return Ok(result);
        }
    }

    let java_vm = java_vm_result?;
    let result = launch_mode_from_jvm_exe_path(java_vm, program_dir, params);
    Ok(result)
}

/// Creates a `EclipseLibErr::JvmNotFound` holding the error message to display.
/// The message will show all the paths from `search_path` that were visited
/// looking for a JVM.
fn no_vm_found_err(params: &EclipseParams, search_paths: &[&Path]) -> EclipseLibErr {
    let program_name = opt_str(&params.name).unwrap_or_default();
    let paths: String = search_paths
        .iter()
        .map(|search_path| search_path.to_string_lossy() + "\n")
        .collect();
    let msg = format!(
        "{} {}. {} {}",
        MSG_VM_REQUIRED, program_name, MSG_NO_VM_FOUND_AT, paths
    );
    EclipseLibErr::JvmNotFound(msg)
}

#[derive(Debug)]
pub enum JvmLaunchMode {
    LaunchJni { jni_lib: PathBuf, add_vm_args: Vec<String> },
    LaunchExe { exe: PathBuf, add_vm_args: Vec<String> },
}

#[derive(Debug)]
enum VmType {
    Directory,
    Nothing,
    Library,
    EeProps,
    Other,
}
