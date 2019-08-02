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

//! This module provides the public function `complete_vm_args` which allows
//! to combine the JVM arguments specified via command line and ini file(s).

#[cfg(target_os = "macos")]
use eclipse_common::ini_reader::read_ini_lines;
use eclipse_common::ini_reader::read_ini;
use std::path::Path;
#[cfg(target_os = "macos")]
use std::path::PathBuf;
use crate::params::EclipseParams;
use crate::errors::EclipseLibErr;

static VM_ARGS_PARAM: &str = "-vmargs";

/// Returns all JVM paramters needed to start the framework. This will take 
/// the given `vm_args` from command line into account as well as the parameters
/// specified in ini files. If `params.append_vmargs` is `false` only the VM arguments
/// from `vm_args` are returned (except if `vm_args` is empty, in this case the arguments
/// from ini file(s) are taken). If `params.append_vmargs` is `true` the vm args are read from
/// ini file(s) and the `vm_args` are appended.
pub fn complete_vm_args<S: AsRef<str>>(
    vm_args: &[S],
    params: &EclipseParams,
    program: &Path,
) -> Result<Vec<String>, EclipseLibErr> {
    let mut result: Vec<String> = Vec::new();

    // VM args from command line
    let vm_args_iter = vm_args.iter().map(AsRef::as_ref).map(ToOwned::to_owned);
    let vm_args_present = !vm_args.is_empty();

    // If we have command line VM args and ini vm args are not appended,
    // take command line VM args
    // So we do not need to parse ini files if we don't have to.
    if !params.append_vmargs && vm_args_present {
        result.extend(vm_args_iter);
        return Ok(result);
    }

    // Read VM args from ini file(s)
    if cfg!(target_os = "macos") {
        let ini_params = vm_args_from_launcher_ini_from_config(params, program);
        result.extend(ini_params);
    }
    result.extend(vm_args_from_config(params, program));

    // Add VM args from command-line (may be empty)
    if vm_args_present {
        result.extend(vm_args_iter);
    }
    // TODO: on mac add -Xdock:icon -> APP_ICON_<pid> and -Xdock:name -> APP_NAME_<pid>, set env variables for app-name and app icon

    Ok(result)
}

#[inline] // only called in one place, won't lead to code bloat.
#[cfg(target_os = "macos")]
fn vm_args_from_launcher_ini_from_config(params: &EclipseParams, program: &Path) -> Vec<String> {
    // Unfortunately this functionality is not documented, see:
    // https://bugs.eclipse.org/bugs/show_bug.cgi?id=509087
    // and
    // https://help.eclipse.org/index.jsp?topic=%2Forg.eclipse.platform.doc.isv%2Freference%2Fmisc%2Flauncher.html&cp=2_1_5_1
    if !params.protect.as_ref().map_or(false, |s| s == "base") {
        return Vec::new();
    }

    let ini_path = if let Some(ini) = get_launcher_file_path_from_configuration(program) {
        ini
    } else {
        return Vec::new();
    };

    if let Some(lines_iter) = read_ini_lines(ini_path).ok() {
        vm_args_from_params(lines_iter).collect()
    } else {
        Vec::new()
    }
}

#[cfg(target_os = "macos")]
fn get_launcher_file_path_from_configuration(program: &Path) -> Option<PathBuf> {
    
    let mut program_name = program.file_stem()?.to_str()?.to_string();
    program_name.push_str(".ini");
    // TODO: implement fn get_folder_for_application_data() -> Option<PathBuf>
    // let mut ini_path = get_folder_for_application_data();
    // ini_path.push(program_name);
    // Ok(ini_path)
    unimplemented!();
}

/// This method will never be called and is only here to make conditional compilation
/// work in complete_vm_args
#[cfg(not(target_os = "macos"))]
fn vm_args_from_launcher_ini_from_config(_params: &EclipseParams, _program: &Path) -> Vec<String> {
    unimplemented!();
}

fn vm_args_from_config(params : &EclipseParams, program: &Path) -> Vec<String> {
    if let Ok(lines_iter) = read_ini(&params.ini, program) {
        let vm_args_iter = vm_args_from_params(lines_iter);
        vm_args_iter.collect()
    } else {
        Vec::new()
    }
}

fn vm_args_from_params(iter: impl Iterator<Item = String>) -> impl Iterator<Item = String> {
    iter.skip_while(|s| s != VM_ARGS_PARAM).skip(1)
}