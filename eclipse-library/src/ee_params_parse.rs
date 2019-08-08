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
//! This module reads ee parameters into an `EclipseEEProps` instance.

use crate::params::EclipseEEProps;
use eclipse_common::arg_parser::Parser;
use eclipse_common::ini_reader::read_ini_lines;
use eclipse_common::option_util::opt_str;
use eclipse_common::path_util::{strip_unc_prefix, PATHS_SEPARATOR};
use std::borrow::Cow;
use std::io::Error;
use std::path::Path;

// constants for ee options file
const EE_HOME_VAR: &str = "${ee.home}";
const EE_EXECUTABLE: &str = "-Dee.executable";
const EE_CONSOLE: &str = "-Dee.executable.console";
const EE_VM_LIBRARY: &str = "-Dee.vm.library";
const EE_LIBRARY_PATH: &str = "-Dee.library.path";
const EE_FILENAME: &str = "-Dee.filename";
const EE_HOME: &str = "-Dee.home";

pub fn parse_ee_params(ee_file: &Path) -> Result<EclipseEEProps, Error> {
    let ee_file_str = ee_file.to_string_lossy().to_string();

    let ee_dir_cow: Option<Cow<'_, str>> = ee_file.parent().map(Path::to_string_lossy);
    // we need to strip UNC prefix on windows, because creating a Path from a string with UNC
    // prefix leads to problems down the road.
    let ee_dir: Option<&str> = ee_dir_cow.as_ref().map(|s| &**s).map(strip_unc_prefix);

    // Read EE file
    let lines: Vec<String> = read_ini_lines(ee_file)?.collect();

    // Strategy: split at '=' and use command line parser to work for us
    let split_lines = lines.iter().flat_map(|line| line.splitn(2, '='));

    // Configure parser
    let mut parser = Parser::new();
    let exe_id = parser.add_option(EE_EXECUTABLE);
    let console_id = parser.add_option(EE_CONSOLE);
    let vm_lib_id = parser.add_option(EE_VM_LIBRARY);
    let lib_path_id = parser.add_option(EE_LIBRARY_PATH);

    // parse params from lines
    let mut parse_result = parser.parse(split_lines);

    let exe_param = parse_result
        .take_option(exe_id)
        .map(|s| replace_ee_home(s, ee_dir));
    let console_param = parse_result
        .take_option(console_id)
        .map(|s| replace_ee_home(s, ee_dir));
    let vm_lib_parm = parse_result
        .take_option(vm_lib_id)
        .map(|s| replace_ee_home(s, ee_dir));
    let lib_path_param = parse_result
        .take_option(lib_path_id)
        .map(|s| replace_ee_home(s, ee_dir));
    let lib_path_param_split = lib_path_param.map(|paths| -> Vec<String> {
        paths
            .split(PATHS_SEPARATOR)
            .map(ToOwned::to_owned)
            .collect()
    });
    let ee_dir_str = ee_dir.unwrap_or_default().to_string();

    let result = EclipseEEProps {
        ee_console: console_param,
        ee_executable: exe_param,
        ee_filename: ee_file_str,
        ee_home: ee_dir_str,
        ee_lib_path: lib_path_param_split,
        ee_vm_libary: vm_lib_parm,
    };
    Ok(result)
}

impl EclipseEEProps {
    pub fn to_vm_command_line_args(&self, console: bool) -> Vec<String> {
        // Paths in ee_lib_path need to be joined with separating PATHS_SEPARATOR
        let paths_joined = self.ee_lib_path.as_ref().map(|p| join_paths(&p));

        // Now make a list of "-Dkey=value" pair for optional values that exist
        let params = [
            (EE_EXECUTABLE, &self.ee_executable),
            (EE_CONSOLE, &self.ee_console),
            (EE_VM_LIBRARY, &self.ee_vm_libary),
            (EE_LIBRARY_PATH, &paths_joined),
        ];
        let mut result: Vec<String> = params.iter().filter_map(to_parameter_opt).collect();
        // now add parameters that always exist
        result.push(to_parameter(EE_FILENAME, &self.ee_filename));
        result.push(to_parameter(EE_HOME, &self.ee_home));
        result
    }
}

fn to_parameter_opt(param_desc: &(&str, &Option<String>)) -> Option<String> {
    let (key, val_opt) = param_desc;
    opt_str(val_opt).map(|ref val| to_parameter(key, val))
}

fn to_parameter(key: &str, value: &str) -> String {
    format!("{}={}", key, value)
}

fn join_paths<S: AsRef<str>>(paths: &[S]) -> String {
    let mut result = String::new();
    let mut iter = paths.iter();
    if let Some(first) = iter.next() {
        result.push_str(first.as_ref());
    }
    while let Some(item) = iter.next() {
        result.push(PATHS_SEPARATOR);
        result.push_str(item.as_ref());
    }
    result
}

fn replace_ee_home(to_replace_in: String, ee_home_dir_opt: Option<&str>) -> String {
    if let Some(ee_home_dir) = ee_home_dir_opt {
        to_replace_in.replace(EE_HOME_VAR, ee_home_dir)
    } else {
        to_replace_in
    }
}
