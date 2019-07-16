//! This module provides functions to find the companion library for the launcher executable
//! (`find_library`), loading the dynamic library (`load_library`),
//! and calling functions from this library via the `EclipseLauncherLib` type.
//! To create an instance of `EclipseLauncherLib`, which will allow calling 
//! library methods, use the factory function `EclipseLauncherLib::new`.
//! 
//! The `EclipseLauncherLib` type will map Rust types to native types passed 
//! via the C ABI of the native library.

//use libc::c_int;
use std::path::PathBuf;
use std::os::raw::c_int;
use std::path::Path;
use dlopen::symbor::{Library, Symbol, SymBorApi};
use dlopen_derive::*;
use crate::path_util::*;

static DEFAULT_EQUINOX_STARTUP: &str = "org.eclipse.equinox.launcher";

static DEFAULT_OS: Option<&str> = option_env!("DEFAULT_OS");

static DEFAULT_OS_ARCH: Option<&str>  = option_env!("DEFAULT_OS_ARCH");

static DEFAULT_WS: Option<&str>  = option_env!("DEFAULT_WS");

// On Windows we use 
#[cfg(windows)]
type NativeString = *const u16;

#[cfg(not(windows))]
type NativeString = *const c_char;

type RunMethod = unsafe extern "C" fn(c_int, *const NativeString, *const NativeString) -> c_int;
type SetInitialArgs = unsafe extern "C" fn(c_int, *const NativeString, NativeString) -> ();

#[derive(SymBorApi)]
struct EclipseLauncherLibApi<'a> {
    #[cfg(not(windows))]
    pub run: Symbol<'a, RunMethod>,

    /// On Windows the unicode methods 
    #[cfg(windows)]
    #[dlopen_name="runW"]
    pub run: Symbol<'a, RunMethod>,

    #[cfg(not(windows))]
    #[dlopen_name="setInitialArgs"]
    pub set_initial_args: Symbol<'a, SetInitialArgs>,

    #[cfg(windows)]
    #[dlopen_name="setInitialArgsW"]
    pub set_initial_args: Symbol<'a, SetInitialArgs>,
}

pub struct EclipseLauncherLib<'a> {
    lib_api: EclipseLauncherLibApi<'a>,
}

impl <'a> EclipseLauncherLib<'a> {
    
    pub fn new<'t>(lib: &'t Library) -> Result<EclipseLauncherLib<'a>, String> where 't : 'a {
        Ok(Self {
            lib_api: unsafe{ EclipseLauncherLibApi::load(lib) }.map_err(|_| "Could not load symbols")?,
        })
    }
    
    #[cfg(windows)]
    pub fn run<S: AsRef<str>>(&self, args: &[S], vm_args: &[S]) -> Result<(),String> {
        // Convert parameters
        let utf16_args = str_slice_to_widechar_vec(args);
        let count_args: c_int = utf16_args.len() as c_int;
        let utf16_vm_args = str_slice_to_widechar_vec(vm_args);
        let args_ptr_vec = vec_to_native_string(utf16_args);
        let args_native = args_ptr_vec.as_ptr();

        // VM args are null-terminated, so we need to add a trailing null
        let mut vm_args_ptr_vec = vec_to_native_string(utf16_vm_args);
        vm_args_ptr_vec.push(std::ptr::null());
        let vm_args_native = vm_args_ptr_vec.as_ptr();
            
        unsafe {
            let result = (self.lib_api.run)(count_args, args_native, vm_args_native);
            if result == 0 {
                Ok(())
            } else {
                // TODO: handle error codes?
                Err("TODO: something went wrong!".into())
            }
        }
    }

    #[cfg(not(windows))]
    pub fn run<S: AsRef<str>>(&self, args: &[S], vm_args: &[S]) -> Result<(),String> {
        unimplemented!()
    }

    #[cfg(windows)]
    pub fn set_initial_args<S: AsRef<str>>(&self, args: &[S], library: &str) -> Result<(), String> {
        // Convert parameters to native
        let arg_count = args.len() as c_int;

        let utf16_args = str_slice_to_widechar_vec(args);
        let args_ptr_vec = vec_to_native_string(utf16_args);
        let args_native = args_ptr_vec.as_ptr();
        
        let library_u16_vec = str_to_utf16( library );
        let library_native_str: NativeString = library_u16_vec.as_ptr();

        unsafe {
            (self.lib_api.set_initial_args)(arg_count, args_native, library_native_str);
        }
        Ok(())
    }

    #[cfg(not(windows))]
    pub fn set_initial_args<S: AsRef<str>>(&self, args: &[S], library: &str) -> Result<(), String> {
        let arg_count = args.len();
        // TODO: convert params, call lib_api.set_initial_args
        Err("not implemented yet".into())
    }
}

fn str_slice_to_widechar_vec<S: AsRef<str>>(slice: &[S]) -> Vec<Vec<u16>> {
    slice.iter().map(|s| {
        let s = s.as_ref();
        str_to_utf16(s) 
    }).collect()
}

fn str_to_utf16(s: &str) -> Vec<u16> {
    let mut vec: Vec<u16> = s.encode_utf16().collect();
    vec.push(0);
    vec
}

fn vec_to_native_string(utf16args: Vec<Vec<u16>>) -> Vec<NativeString> {
    utf16args.iter().map(|v| v.as_ptr()).collect()
}

pub fn load_library(lib_path: &Path) -> Result<Library,String> {
    Library::open(lib_path).map_err(|_| "Could not load library".into())
}

/// Looks for the companion library to the executable in the `library_dir`
/// if given. If the dir is relative, the search for the library will be look in the 
/// working directory and the given `program` directory. 
/// If no `library_dir` is given, this function will try to locate the plugin folder
/// containing the library, relative to the given `program_dir` and search for the
/// library in there.
pub fn find_library(library_dir: &Option<String>, program_dir: &Path) -> Result<PathBuf,String> {
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
        let plugin_dir_opt = find_file(&plugin_path, &fragment);
        let plugin_dir = plugin_dir_opt.ok_or_else(|| format!("Plugin not found in path {:?}", &plugin_path))?;
        find_file(&plugin_dir, "eclipse").ok_or_else(|| format!("Compagnion library not found in path {:?}", &plugin_dir))
    }
}

// make const as soon as get_default_os and get_default_arch are const
fn is_macos_non_x86_64() -> bool {
    get_default_os() == "macosx" && get_default_arch() != "x86_64" 
}

// make this fn const as soon as Option::unwrap_or is a const fn.
#[cfg(target_os = "windows")]
fn get_default_os() -> &'static str {
     DEFAULT_OS.unwrap_or("win32")
}

// make this fn const as soon as Option::unwrap_or is a const fn.
#[cfg(target_os = "macos")]
fn get_default_os() -> &'static str {
     DEFAULT_OS.unwrap_or("macosx")
}

// make this fn const as soon as Option::unwrap_or is a const fn.
#[cfg(target_os = "linux")]
fn get_default_os() -> &'static str {
     DEFAULT_OS.unwrap_or("linux")
}


// make this fn const as soon as Option::unwrap_or is a const fn.
#[cfg(target_arch = "x86")]
fn get_default_arch() -> &'static str {
     DEFAULT_ARCH.unwrap_or("x86")
}

// make this fn const as soon as Option::unwrap_or is a const fn.
#[cfg(target_arch = "x86_64")]
fn get_default_arch() -> &'static str {
     DEFAULT_OS_ARCH.unwrap_or("x86_64")
}

// make this fn const as soon as Option::unwrap_or is a const fn.
#[cfg(target_arch = "powerpc")]
fn get_default_arch() -> &'static str {
     DEFAULT_OS_ARCH.unwrap_or("ppc")
}

// make this fn const as soon as Option::unwrap_or is a const fn.
#[cfg(target_arch = "arm")]
fn get_default_arch() -> &'static str {
     DEFAULT_OS_ARCH.unwrap_or("arm")
}

// make this fn const as soon as Option::unwrap_or is a const fn.
#[cfg(target_arch = "aarch64")]
fn get_default_arch() -> &'static str {
     DEFAULT_OS_ARCH.unwrap_or("aarch64")
}


// make this fn const as soon as Option::unwrap_or is a const fn.
#[cfg(target_os = "windows")]
fn get_default_ws() -> &'static str {
     DEFAULT_WS.unwrap_or("win32")
}

// make this fn const as soon as Option::unwrap_or is a const fn.
#[cfg(target_os = "macos")]
fn get_default_ws() -> &'static str {
     DEFAULT_WS.unwrap_or("cocoa")
}

// make this fn const as soon as Option::unwrap_or is a const fn.
#[cfg(target_os = "linux")]
fn get_default_ws() -> &'static str {
     DEFAULT_WS.unwrap_or("gtk")
}
