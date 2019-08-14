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
use eclipse_common::arg_parser::{OptionId, ParseResult, Parser};
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

pub fn read_ee_file(ee_file: &Path) -> Result<Vec<String>, Error> {
    // Read EE file
    let lines = read_ini_lines(ee_file)?.collect();
    Ok(lines)
}

pub fn parse_ee_params<S: AsRef<str>>(
    ee_file: &Path,
    ee_file_lines: &[S],
) -> Result<EclipseEEProps, Error> {
    let ee_dir_cow: Option<Cow<'_, str>> = ee_file.parent().map(Path::to_string_lossy);
    // we need to strip UNC prefix on windows, because creating a Path from a string with UNC
    // prefix leads to problems down the road.
    let ee_dir: Option<&str> = ee_dir_cow.as_ref().map(|s| &**s).map(strip_unc_prefix);

    // Strategy: split at '=' and use command line parser to work for us
    let split_lines = ee_file_lines
        .iter()
        .map(AsRef::as_ref)
        .flat_map(|line| line.splitn(2, '='));

    // Configure parser
    let mut parser = Parser::new();
    let exe_id = parser.add_option(EE_EXECUTABLE);
    let console_id = parser.add_option(EE_CONSOLE);
    let vm_lib_id = parser.add_option(EE_VM_LIBRARY);
    let lib_path_id = parser.add_option(EE_LIBRARY_PATH);

    // parse params from lines
    let mut parse_result = parser.parse(split_lines);

    let exe_param = get_parsed_path(&mut parse_result, exe_id, ee_dir);
    let console_param = get_parsed_path(&mut parse_result, console_id, ee_dir);
    let vm_lib_parm = get_parsed_path(&mut parse_result, vm_lib_id, ee_dir);
    let lib_path_param = get_parsed_path(&mut parse_result, lib_path_id, ee_dir);
    let lib_path_param_split = lib_path_param.map(|paths| -> Vec<String> {
        paths
            .split(PATHS_SEPARATOR)
            .map(ToOwned::to_owned)
            .collect()
    });
    let ee_dir_str = ee_dir.unwrap_or_default().to_string();
    let ee_file_cow = ee_file.to_string_lossy();
    let ee_file_str = strip_unc_prefix(&ee_file_cow).to_string();

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

/// Takes the result, replaces `${ee.home}` with the given
/// `ee_dir` (if speficied) and canonicalizes the path.
fn get_parsed_path(
    parse_result: &mut ParseResult,
    param_id: OptionId,
    ee_dir: Option<&str>,
) -> Option<String> {
    parse_result
        .take_option(param_id)
        .map(|s| replace_ee_home(s, ee_dir))
        .map(canonicalize)
}

fn canonicalize(path_str: String) -> String {
    let path = Path::new(&path_str);
    std::fs::canonicalize(path)
        .map(|p| {
            let p_cow = p.to_string_lossy();
            let p_str = strip_unc_prefix(&p_cow);
            p_str.to_string()
        })
        .unwrap_or(path_str)
}

impl EclipseEEProps {
    pub fn to_vm_command_line_args(&self) -> Vec<String> {
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
    for item in iter {
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
