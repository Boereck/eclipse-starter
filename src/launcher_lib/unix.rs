use dlopen::symbor::{Library, SymBorApi, Symbol};
use dlopen_derive::*;
use std::marker::PhantomData;
use std::os::raw::c_int;
use super::{EclipseLauncher, InitialArgs};
use super::common::{RunMethod, SetInitialArgs, NativeString};
use std::ffi::CString;

#[derive(SymBorApi)]
struct EclipseLauncherLibApi<'a> {
	#[cfg(not(windows))]
	pub run: Symbol<'a, RunMethod>,

	#[cfg(not(windows))]
	#[dlopen_name = "setInitialArgs"]
	pub set_initial_args: Symbol<'a, SetInitialArgs>,
}

pub struct EclipseLauncherNix<'a> {
	lib_api: EclipseLauncherLibApi<'a>,
}

impl<'a,'b> EclipseLauncher<'a,'b> for EclipseLauncherNix<'a> 
	where 'b: 'a {
	type InitialArgsParams = SetInitialArgsParams<'b>;
	
	fn new(lib: &'a Library) -> Result<Self, String>
	{
		Ok(Self {
			lib_api: unsafe { EclipseLauncherLibApi::load(lib) }
				.map_err(|_| "Could not load symbols")?,
		})
	}

	fn run<S: AsRef<str>>(&self, args: &[S], vm_args: &[S]) -> Result<(), String> {
		let count_args = args.len() as c_int;

		// convert args to native types
		let args_vec_cstring = str_slice_to_cstring_vec(args);
		let args_vec_nativestring = cstring_vec_to_nativestr_vec(&args_vec_cstring);
		let args_ptr_nativestring = args_vec_nativestring.as_ptr();

		// convert vm_args to native types
		let vm_args_vec_cstring = str_slice_to_cstring_vec(vm_args);
		let mut vm_args_vec_nativestring = cstring_vec_to_nativestr_vec(&vm_args_vec_cstring);
		// VM args are null-terminated, so we need to add a trailing null
		vm_args_vec_nativestring.push(std::ptr::null());
		let vm_args_ptr_nativestring = vm_args_vec_nativestring.as_ptr();
		unsafe {
			let result = (self.lib_api.run)(count_args, args_ptr_nativestring, vm_args_ptr_nativestring);
			if result == 0 {
				Ok(())
			} else {
				// TODO: handle error codes?
				Err("TODO: something went wrong!".into())
			}
		}
	}

	fn set_initial_args(&self, params: &SetInitialArgsParams) -> Result<(), String> {
		// Convert parameters to native
		let arg_count = params.arg_count;
		let args_native = params.args_ptr_nativestr;
		let library_native_str = params.library_native_str;
		unsafe {
			(self.lib_api.set_initial_args)(arg_count, args_native, library_native_str);
		}
		Ok(())
	}
}

#[allow(dead_code)] // needed, since some fields are only used to hold date pointers are pointing to
pub struct SetInitialArgsParams<'a> {
	arg_count: c_int,
	args_vec_cstring: Vec<CString>,
	args_vec_nativestr: Vec<NativeString>,
	args_ptr_nativestr: *const NativeString,
	library_cstring: CString,
	library_native_str: NativeString,
	// phantom needed to make use of lifetime 'a
	phantom: PhantomData<&'a NativeString>,
}

impl<'b> InitialArgs<'b> for SetInitialArgsParams<'b> {
	fn new<S: AsRef<str>>(args: &'b [S], library: &'b str) -> Self {
		let args_vec_cstring_param = str_slice_to_cstring_vec(args);
		let args_vec_nativestr_param = cstring_vec_to_nativestr_vec(&args_vec_cstring_param);
		let args_ptr_nativestr_param = args_vec_nativestr_param.as_ptr();

		let library_cstring_param = CString::new(library).unwrap_or_default();
		let library_ptr_param: NativeString = library_cstring_param.as_ptr();
		Self {
			arg_count: args.len() as c_int,
			args_vec_cstring: args_vec_cstring_param,
			args_vec_nativestr: args_vec_nativestr_param,
			args_ptr_nativestr: args_ptr_nativestr_param,
			library_cstring: library_cstring_param,
			library_native_str: library_ptr_param,
			phantom: PhantomData,
		}
	}
}

fn str_slice_to_cstring_vec<S: AsRef<str>>(args: &[S]) -> Vec<CString> {
	args
		.iter()
		.filter_map(|s| CString::new(s.as_ref()).ok())
		.collect()
}

fn cstring_vec_to_nativestr_vec(args: &Vec<CString>) -> Vec<NativeString> {
	args.iter()
		.map(|s| s.as_ptr())
		.collect()
}