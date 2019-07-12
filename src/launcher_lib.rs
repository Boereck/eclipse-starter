use libc::c_int;
use dlopen::symbor::{Library, Symbol, SymBorApi};
use dlopen_derive::*;

#[cfg(windows)]
type NativeString = *const u16;

#[cfg(not(windows))]
type NativeString = *const c_char;

type RunMethod = unsafe extern "C" fn(c_int, *const NativeString, *const NativeString) -> f64;
type SetInitialArgs = unsafe extern "C" fn(c_int, *const NativeString, NativeString) -> f64;

#[derive(SymBorApi)]
struct EclipseLauncherLibApi<'a> {
    pub run_method: Symbol<'a, RunMethod>,
    pub set_inial_arg: Symbol<'a, SetInitialArgs>,
}

pub struct EclipseLauncherLib {
    
}


pub fn load_library() -> Result<EclipseLauncherLib,String>{
    unimplemented!()
}