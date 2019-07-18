//! This module provides functions to find the companion library for the launcher executable
//! (`find_library`), loading the dynamic library (`load_library`),
//! and calling functions from this library via the `EclipseLauncher` type.
//! To create an instance of `EclipseLauncherLib`, which will allow calling
//! library methods, use the factory function `new_launcher`.
//!
//! The `EclipseLauncher` implementation will map Rust types to native types passed
//! via the C ABI of the native library.
#[cfg(windows)]
mod windows;

#[cfg(not(windows))]
mod unix;

mod common;

#[cfg(windows)]
use self::windows as os;
#[cfg(windows)]
use os::EclipseLauncherWin as EclipseLauncherOs;

#[cfg(not(windows))]
use self::unix as os;
#[cfg(not(windows))]
use os::EclipseLauncherNix as EclipseLauncherOs;

use crate::compile_params::*;
use crate::path_util::*;
use dlopen::symbor::Library;
use std::path::Path;
use std::path::PathBuf;

static DEFAULT_EQUINOX_STARTUP: &str = "org.eclipse.equinox.launcher";

pub trait InitialArgs<'b> {
	fn new<S: AsRef<str>>(args: &'b [S], library: &'b str) -> Self;
}

pub trait EclipseLauncher<'a, 'b>: Sized
where
	'b : 'a,
{
	type InitialArgsParams: InitialArgs<'b>;
	fn new(lib: &'a Library) -> Result<Self, String>;
	fn run<S: AsRef<str>>(&self, args: &[S], vm_args: &[S]) -> Result<(), String>;
	#[inline]
	fn new_initial_args<S: AsRef<str>>(
		&self,
		args: &'b [S],
		library: &'b str,
	) -> Self::InitialArgsParams {
		Self::InitialArgsParams::new(args, library)
	}
	fn set_initial_args(&self, params: &Self::InitialArgsParams) -> Result<(), String>;
}

/// Creates an instance of `EclipseLauncher` for the given `library`, which allows to
/// call functions on the library. 
/// The library instance needs to be obtained using the `load_library` function.
pub fn new_launcher<'a, 'b>(lib: &'a Library) -> Result<impl EclipseLauncher<'a, 'b>, String>
where
	'b: 'a,
{
	EclipseLauncherOs::new(lib)
}

/// Loads the launcher companion library.
/// The path to the library, needed as paramter, can be detected
/// using the `find_library` function.
pub fn load_library(lib_path: &Path) -> Result<Library, String> {
	Library::open(lib_path).map_err(|_| "Could not load library".into())
}

/// Looks for the companion library to the executable in the `library_dir`
/// if given. If the dir is relative, the search for the library will be look in the
/// working directory and the given `program` directory.
/// If no `library_dir` is given, this function will try to locate the plugin folder
/// containing the library, relative to the given `program_dir` and search for the
/// library in there.
pub fn find_library(library_dir: &Option<String>, program_dir: &Path) -> Result<PathBuf, String> {
	if let Some(library_location) = library_dir {
		let lib_dir_path = Path::new(library_location);
		let lib_dir_path = check_path(&lib_dir_path, program_dir, true);
		let result_path = if lib_dir_path.as_path().is_dir() {
			// directory, find the highest version eclipse_* library
			find_file(&lib_dir_path, "eclipse").ok_or_else(|| "library not found")?
		} else {
			// file, return it
			lib_dir_path
		};
		Ok(result_path)
	} else {
		// build the equinox.launcher fragment name
		let dot = ".";
		let mut fragment = DEFAULT_EQUINOX_STARTUP.to_string();
		fragment.push_str(dot);
		fragment.push_str(get_default_ws());
		fragment.push_str(dot);
		fragment.push_str(get_default_os());
		if !is_macos_non_x86_64() {
			// The Mac fragment covers both archs and does not have that last segment
			fragment.push_str(dot);
			fragment.push_str(get_default_arch());
		}

		let mut plugin_path = program_dir.to_path_buf();
		if cfg!(macos) {
			plugin_path.push("../../../")
		}
		plugin_path.push("plugins");

		// find equinox.launcher plugin directory containing the companion dynamic library
		let plugin_dir_opt = find_file(&plugin_path, &fragment);
		let plugin_dir =
			plugin_dir_opt.ok_or_else(|| format!("Plugin not found in path {:?}", &plugin_path))?;

		// Find companion dynamic library in plugins folder
		find_file(&plugin_dir, "eclipse")
			.filter(|path| path.is_file())
			.ok_or_else(|| format!("Companion library not found in path {:?}", &plugin_dir))
	}
}
