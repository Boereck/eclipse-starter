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

//! This module is the entrypoint to the eclipse launcher executable.
//! See the `main()` method, for the entrypoint for the executable.

// Turn on/of console creation on windows based on "win_console" feature
#![cfg_attr(
    all(target_os = "windows", feature = "win_console"),
    windows_subsystem = "console"
)]
#![cfg_attr(
    all(target_os = "windows", not(feature = "win_console")),
    windows_subsystem = "windows"
)]

mod compile_params;
mod errors;
mod exe_util;
mod launcher_lib;
mod params;

use eclipse_common::arg_parser::*;
use eclipse_common::ini_reader::*;
use eclipse_common::option_util::opt_str;
use eclipse_common::name_util::get_default_official_name;
use eclipse_common::messagebox::display_message;
use eclipse_common::path_util::strip_unc_prefix;
use errors::LauncherError;
use exe_util::get_exe_path;
use launcher_lib::{find_library, load_library, new_launcher, EclipseLauncher};
use params::EclipseLauncherParams;
use std::path::Path;

#[cfg(not(target_os = "windows"))]
use libc::geteuid;

// Error messages

static MSG_EXE_LOCATION_NOT_FOUND: &str = "Determining the launcher location failed.";
static MSG_EXE_PARENT_NOT_FOUND: &str = "Parent directory of launcher not found.";
static MSG_LIB_PATH_CONVERSION_ERR: &str = "Converting path name of companion library failed.";
static MSG_EXE_PATH_CONVERSION_ERR: &str = "Converting path name of launcher failed.";
static MSG_ROOT_ERR: &str =
    "executable launcher is configured to not start with administrative privileges.";

static ROOT: &str = "root";

// Possible launcher executable arguments

const NAME_ARG: &str = "-name";
const LAUNCHER_LIB_ARG: &str = "--launcher.library";
const SUPPRESS_ERRORS_ARGS: &str = "--launcher.suppressErrors";
const PROTECT_ARG: &str = "-protect";
const LAUNCHER_INI_ARG: &str = "--launcher.ini";
const VMARGS_ARG: &str = "-vmargs";

fn main() {
    let mut params = EclipseLauncherParams::default();
    let result = fallible_main(&mut params);
    if let Err(ref err) = &result {
        if params.suppress_errors {
            // Do not show dialog, just print to stdout
            eprintln!("{}\nDetails: \n{:#?}", err, err);
        } else {
            let title = opt_str(&params.name).unwrap_or_else(|| "");
            let msg = format!("{}", err);
            // message dialog failed, print the error to stderr
            if let Err(msg) = display_message(&msg, &title) {
                eprintln!("{}", msg);
            }
        }
        let err_code = match err {
            LauncherError::LibraryLookupError(_) => 1,
            LauncherError::SecurityError(_) => 2,
            LauncherError::GeneralError(_) => 3,
            LauncherError::RunError(_, i) => *i as i32,
        };
        std::process::exit(err_code);
    }
}

#[inline]
fn fallible_main(params: &mut EclipseLauncherParams) -> Result<(), LauncherError> {
    let command_line_args: Vec<String> = std::env::args().collect();
    let mut ini_file_args = Vec::<String>::new();
    // parse arguments without program location
    parse_arguments(params, command_line_args.iter().map(String::as_str).skip(1));

    // Determine the full pathname of this program.
    let exe_path = get_exe_path().map_err(|_| MSG_EXE_LOCATION_NOT_FOUND)?;
    // read ini, only set params not already defined by program arguments
    if let Ok(ini_file_lines) = read_ini(&params.launcher_ini, &exe_path) {
        // we strip vmargs off (since the original launcher had this behavior,
        // see eclipseMain.c main calling parseArgs with useVMargs = 0)
        let ini_lines_no_vmargs = ini_file_lines.take_while(|s| s != VMARGS_ARG);
        // store ini lines in vector for later usage
        ini_file_args.extend(ini_lines_no_vmargs);
        parse_arguments(params, ini_file_args.iter().map(String::as_str));
    }

    // get default name if not yet set
    if params.name.is_none() {
        // Initialize official program name
        params.name = get_default_official_name();
    }

    // If config prohibits root to start the application and
    // the user is root, then stop the application.
    if cfg!(not(target_os = "windows")) && perform_root_check(params) && is_root() {
        let name = opt_str(&params.name).unwrap_or_else(|| "");
        return Err(LauncherError::SecurityError(format!(
            "{} {}",
            name, MSG_ROOT_ERR
        )));
    }

    // load and promt msg on failure
    load_lib_and_run(&params, &command_line_args, &ini_file_args, &exe_path)
}

/// Detects the location of the companion shared library library,
/// loads it, and calls `setInitialArgs` and `run` on the library.
fn load_lib_and_run(
    params: &EclipseLauncherParams,
    command_line_args: &[String],
    ini_file_args: &[String],
    exe_path: &Path,
) -> Result<(), LauncherError> {
    let exe_parent = exe_path
        .parent()
        .ok_or_else(|| MSG_EXE_PARENT_NOT_FOUND.to_string())?;
    // Find the eclipse library, load and initalize callable API
    let lib_path = find_library(&params.eclipse_library, exe_parent)?;
    let lib = load_library(&lib_path)?;
    let lib_api = new_launcher(&lib)?;
    // If no VM args are set use empty slice
    let lib_path_str = &lib_path
        .to_str()
        .ok_or_else(|| MSG_LIB_PATH_CONVERSION_ERR.to_string())?;
    let initial_args_params = lib_api.new_initial_args(command_line_args, lib_path_str);
    lib_api.set_initial_args(&initial_args_params)?;

    // call run on the library
    let exe_path_str = exe_path
        .to_str()
        .ok_or_else(|| MSG_EXE_PATH_CONVERSION_ERR.to_string())?;
    let merged_args: Vec<&str> = merge_parameters(exe_path_str, ini_file_args, command_line_args);
    let vm_args: Vec<&str> = params
        .vm_args
        .iter()
        .flatten()
        .map(String::as_str)
        .collect();
    lib_api.run(&merged_args, &vm_args)
}

/// Cretes a vector with the value of `exe_path` followed by references
/// to the elements of `ini_file_args` followed by the elements of
/// `command_line_args` (where the first argument is dropped).
fn merge_parameters<'a>(
    exe_path: &'a str,
    ini_file_args: &'a [String],
    command_line_args: &'a [String],
) -> Vec<&'a str> {
    let mut result: Vec<&'a str> = Vec::new();
    let exe_path_param = if cfg!(target_os = "windows") {
        // TODO: if library supports long path names, we don't have to strip
        strip_unc_prefix(exe_path)
    } else {
        exe_path
    };
    result.push(exe_path_param);
    result.extend(ini_file_args.iter().map(String::as_str));
    result.extend(command_line_args.iter().skip(1).map(String::as_str));
    result
}

/// Reads arguments from `args` and sets values in `params`, if they were not
/// set already.
fn parse_arguments<'a, 'b>(
    params: &'a mut EclipseLauncherParams,
    args: impl Iterator<Item = &'b str>,
) {
    let mut parser: Parser = Parser::new();

    // Define parameters to parse
    let name = parser.add_option(NAME_ARG);
    let eclipse_library = parser.add_option(LAUNCHER_LIB_ARG);
    let protect = parser.add_option(PROTECT_ARG);
    let suppress_errors = parser.add_flag(SUPPRESS_ERRORS_ARGS);
    let launcher_ini = parser.add_option(LAUNCHER_INI_ARG);
    let vm_args = parser.add_list(VMARGS_ARG, ListParseStyle::AllRemaining);

    let mut parse_result = parser.parse(args);

    // Extract parsed parameters and set in `params` if they were found and not already set
    set_if_none(&mut params.name, parse_result.take_option(name));
    set_if_none(
        &mut params.eclipse_library,
        parse_result.take_option(eclipse_library),
    );
    params.suppress_errors |= parse_result.take_flag(suppress_errors);
    set_if_none(&mut params.protect, parse_result.take_option(protect));
    set_if_none(
        &mut params.launcher_ini,
        parse_result.take_option(launcher_ini),
    );
    set_if_none(&mut params.vm_args, parse_result.take_list(vm_args));
}

/// If `target` is `None` and `from` is `Some(t)`, this
/// function will make `target` a `Some(t)`. Otherwise
/// `target` will stay unchanged.
fn set_if_none<T>(target: &mut Option<T>, from: Option<T>) {
    if let Some(from_val) = from {
        target.get_or_insert(from_val);
    }
}


/// Determins if the configuration demands a check
/// if the user staring this executable is the root user
fn perform_root_check(params: &EclipseLauncherParams) -> bool {
    opt_str(&params.protect) == Some(ROOT)
}

/// No root-check support for Windows
#[cfg(target_os = "windows")]
fn is_root() -> bool {
    false
}

/// Tests if the effective user is root
#[cfg(not(target_os = "windows"))]
fn is_root() -> bool {
    unsafe { geteuid() == 0 }
}
