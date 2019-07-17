//! This module is the entrypoint to the eclipse launcher executable.
//! 

//#![windows_subsystem = "windows"]
mod windows;

mod arg_parser;
mod ini_reader;
mod params;
mod launcher_lib;
mod exe_util;
mod path_util;
mod compile_params;

use std::path::Path;
use arg_parser::*;
use ini_reader::*;
use params::EclipseLauncherParams;
use unicode_segmentation::UnicodeSegmentation;
use launcher_lib::{find_library, load_library, EclipseLauncherLib, SetInitialArgsParams};
use exe_util::get_exe_path;


// Arguments

const NAME_ARG: &str = "-name";
const LAUNCHER_LIB_ARG: &str = "--launcher.library";
const SUPPRESS_ERRORS_ARGS: &str = "--launcher.suppressErrors";
const PROTECT_ARG: &str = "-protect";
const LAUNCHER_INI_ARG: &str = "--launcher.ini";
const VMARGS_ARG: &str = "-vmargs";


fn main() {
    let mut params = EclipseLauncherParams::default();
    let command_line_args: Vec<String> = std::env::args().collect();
    let mut ini_file_args = Vec::<String>::new();
    // parse arguments without program location
    parse_arguments(
        &mut params,
        command_line_args.iter().map(String::as_str).skip(1),
    );

    // TODO: handle errors here!
    // Determine the full pathname of this program.
    let exe_path = get_exe_path().unwrap();
    // read ini, only set params not already defined by program arguments
    if let Ok(ini_file_lines) = read_ini(&params.launcher_ini, &exe_path) {
        // we strip vmargs off (since the original launcher had this behavior, 
        // see eclipseMain.c main calling parseArgs with useVMargs = 0)
        let ini_lines_no_vmargs = ini_file_lines.take_while(|s| s != VMARGS_ARG);
        // store ini lines in vector for later usage
        ini_file_args.extend(ini_lines_no_vmargs);
        parse_arguments(&mut params, ini_file_args.iter().map(String::as_str));
    }

    // get default name if not yet set
    if params.name.is_none() {
        // Initialize official program name
        params.name = get_default_official_name()
    }

    // TODO: Root check on Mac OS

    // load and promt msg on failure
    let result = load_lib_and_run(&params, &command_line_args, &ini_file_args, &exe_path);
    // TODO: proper error handling, if !params.suppress_errors
    result.unwrap();
}

fn load_lib_and_run(params: &EclipseLauncherParams, command_line_args : &[String], ini_file_args: &[String], exe_path: &Path) -> Result<(),String> {
    let exe_parent = exe_path.parent().ok_or_else(|| "Parent dir of executable not found".to_string())?;
    // Find the eclipse library, load and initalize callable API
    let lib_path = find_library(&params.eclipse_library, exe_parent)?;
    let lib = load_library(&lib_path)?;
    let lib_api = EclipseLauncherLib::new(&lib)?;
    
    // If no VM args are set use empty slice
    let lib_path_str = &lib_path.to_str().ok_or_else(|| "Converting library path name failed".to_string())?;
    let initial_args_params = SetInitialArgsParams::new(command_line_args, lib_path_str);
    lib_api.set_initial_args(&initial_args_params)?;

    // call run on the library
    let exe_path_str = exe_path.to_str().ok_or_else(|| "Converting exe path name failed".to_string())?;
    let merged_args: Vec<&str> = merge_parameters(exe_path_str, ini_file_args, command_line_args);
    let vm_args : Vec<&str> = params.vm_args.iter().flatten().map(String::as_str).collect();
    lib_api.run(&merged_args, &vm_args)
}

/// Cretes a vector with the value of `exe_path` followed by references
/// to the elements of `ini_file_args` followed by the elements of 
/// `command_line_args` (where the first argument is dropped).
fn merge_parameters<'a>(exe_path: &'a str, ini_file_args: &'a [String], command_line_args : &'a [String]) -> Vec<&'a str> {
    let mut result : Vec<&'a str> = Vec::new();
    let exe_path_param = if cfg!(windows) {
        // TODO: if library supports long path names, we don't have to strip
        exe_path.trim_start_matches(r"\\?\")
    } else {
        exe_path
    };
    result.push(exe_path);
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
    let vm_args = parser.add_list(VMARGS_ARG);

    let mut parse_result = parser.parse(args);

    // Extract parsed parameters and set in `params` if they were found and not already set
    set_if_none(&mut params.name, parse_result.take_option(name));
    set_if_none(&mut params.eclipse_library, parse_result.take_option(eclipse_library));
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

/// Determine the default official application name
///
/// This function provides the default application name that appears in a variety of
/// places such as: title of message dialog, title of splash screen window
/// that shows up in Windows task bar.
/// It is computed from the name of the launcher executable and
/// by capitalizing the first letter. e.g. "c:/ide/eclipse.exe" provides
/// a default name of "Eclipse".
fn get_default_official_name() -> Option<String> {
    let exe_path = std::env::current_exe().ok()?;
    let file_name = exe_path.file_stem()?.to_str()?;
    first_to_uppercase(file_name).into()
}

fn first_to_uppercase(input: &str) -> String {
    // split characters at unicode "grapheme cluster"
    let extended = true;
    let mut graphemes = input.graphemes(extended);
    // Uppercase only first cluster
    let mut result = String::new();
    if let Some(first_char) = graphemes.next() {
        result.push_str(&first_char.to_uppercase());
    }
    // push the rest "as is" into the result
    result.push_str(graphemes.as_str());
    result
}
