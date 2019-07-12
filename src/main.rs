//#![windows_subsystem = "windows"]
mod windows;

mod arg_parser;
mod ini_reader;
mod params;
mod launcher_lib;

use arg_parser::*;
use ini_reader::*;
use params::EclipseLauncherParams;
use unicode_segmentation::UnicodeSegmentation;


// Arguments

const NAME_ARG: &str = "-name";
const LAUNCHER_LIB_ARG: &str = "--launcher.library";
const SUPPRESS_ERRORS_ARGS: &str = "--launcher.suppressErrors";
const PROTECT_ARG: &str = "-protect";
const LAUNCHER_INI_ARG: &str = "--launcher.ini";
const VMARGS_ARG: &str = "-vmargs";



fn main() {
    println!("Hello Eclipse");
    let mut params = EclipseLauncherParams::default();
    let command_line_args: Vec<String> = std::env::args().collect();
    let mut ini_file_args = Vec::<String>::new();
    // arguments without program location
    parse_arguments(
        &mut params,
        command_line_args.iter().map(String::as_str).skip(1),
    );

    // read ini, only set params not already defined by program arguments
    if let Ok(ini_file_lines) = read_ini(&params.launcher_ini) {
        // store ini lines in vector for later usage
        ini_file_args.extend(ini_file_lines);
        parse_arguments(&mut params, ini_file_args.iter().map(String::as_str));
    }

    // get default name if not yet set
    if params.name.is_none() {
        params.name = get_default_official_name()
    }

    // TODO: Root check

    // TODO: Find launcher library, load and promt msg on failure
    load_library(params.library);

    // TODO: call SET_INITIAL_ARGS

    // TODO: call RUN_METHOD

    println!("{:#?}", params);
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
    let library = parser.add_option(LAUNCHER_LIB_ARG);
    let protect = parser.add_option(PROTECT_ARG);
    let suppress_errors = parser.add_flag(SUPPRESS_ERRORS_ARGS);
    let launcher_ini = parser.add_option(LAUNCHER_INI_ARG);
    let vm_args = parser.add_list(VMARGS_ARG);

    let mut parse_result = parser.parse(args);

    // Extract parsed parameters and set in `params` if they were found and not already set
    set_if_none(&mut params.name, parse_result.take_option(name));
    set_if_none(&mut params.library, parse_result.take_option(library));
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

/// Searches for
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

