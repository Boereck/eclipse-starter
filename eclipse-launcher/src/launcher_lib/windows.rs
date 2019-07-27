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

//! This module contains Windows specific functionality to load the launcher
//! companion library and call methods on it.

use eclipse_common::path_util::strip_unc_prefix;
use eclipse_common::native_str::{str_to_utf16, to_native_str};
use super::common::{EclipseLauncher, InitialArgs, NativeString, RunMethod, SetInitialArgs, MSG_LOAD_LIB_SYMBOL_RESOLVE_ERROR, MSG_ERROR_CALLING_RUN};
use dlopen::symbor::{Library, SymBorApi, Symbol};
use dlopen_derive::*;
use std::marker::PhantomData;
use std::os::raw::c_int;
use crate::errors::LauncherError;

#[derive(SymBorApi)]
struct EclipseLauncherLibWin<'a> {
	/// On Windows we use unicode methods (suffix 'W')
	#[dlopen_name = "runW"]
	pub run: Symbol<'a, RunMethod>,

	#[dlopen_name = "setInitialArgsW"]
	pub set_initial_args: Symbol<'a, SetInitialArgs>,
}

pub(super) struct EclipseLauncherOs<'a> {
	lib_api: EclipseLauncherLibWin<'a>,
}

impl<'a, 'b> EclipseLauncher<'a, 'b> for EclipseLauncherOs<'a>
where
	'b: 'a,
{
	type InitialArgsParams = SetInitialArgsParams<'b>;

	fn new(lib: &'a Library) -> Result<Self, LauncherError> {
		Ok(Self {
			lib_api: unsafe { EclipseLauncherLibWin::load(lib) }
				.map_err(|e| {
                    let msg = MSG_LOAD_LIB_SYMBOL_RESOLVE_ERROR;
                    let result = format!("{}\n{}", msg, e);
                    LauncherError::LibraryLookupError(result)
                 })?,
		})
	}

	fn run<S: AsRef<str>>(&self, args: &[S], vm_args: &[S]) -> Result<(), LauncherError> {
		// Convert parameters
		let utf16_args = str_slice_to_widechar_vec(args);
		let count_args: c_int = utf16_args.len() as c_int;
		let utf16_vm_args = str_slice_to_widechar_vec(vm_args);
		let args_ptr_vec = vec_to_native_string(&utf16_args);
		let args_native = args_ptr_vec.as_ptr();

		// VM args are null-terminated, so we need to add a trailing null
		let mut vm_args_ptr_vec = vec_to_native_string(&utf16_vm_args);
		vm_args_ptr_vec.push(std::ptr::null());
		let vm_args_native = vm_args_ptr_vec.as_ptr();
		unsafe {
			let result = (self.lib_api.run)(count_args, args_native, vm_args_native);
            match result {
                0 => Ok(()),
                i => Err(LauncherError::RunError(MSG_ERROR_CALLING_RUN.to_string(), i))
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
	args_vec_vec_u16: Vec<Vec<u16>>,
	args_vec_nativestr: Vec<NativeString>,
	args_ptr_nativestr: *const NativeString,
	library_vec_u16: Vec<u16>,
	library_native_str: NativeString,
	// phantom needed to make use of lifetime 'a
	phantom: PhantomData<&'a NativeString>,
}

impl<'a> InitialArgs<'a> for SetInitialArgsParams<'a> {
	fn new<S: AsRef<str>>(args: &'a [S], library: &'a str) -> Self {
		let args_vec_vec_u16_param = str_slice_to_widechar_vec(args);
		let args_vec_nativestr_param = vec_to_native_string(&args_vec_vec_u16_param);
		let args_ptr_nativestr_param = args_vec_nativestr_param.as_ptr();

		let (library_vec_u16_param, library_native_str_param) = to_native_str(strip_unc_prefix(library));
		Self {
			arg_count: args.len() as c_int,
			args_vec_vec_u16: args_vec_vec_u16_param,
			args_vec_nativestr: args_vec_nativestr_param,
			args_ptr_nativestr: args_ptr_nativestr_param,
			library_vec_u16: library_vec_u16_param,
			library_native_str: library_native_str_param,
			phantom: PhantomData,
		}
	}
}

fn str_slice_to_widechar_vec<S: AsRef<str>>(slice: &[S]) -> Vec<Vec<u16>> {
	slice
		.iter()
		.map(|s| {
			let s = s.as_ref();
			str_to_utf16(s)
		})
		.collect()
}

fn vec_to_native_string(utf16args: &[Vec<u16>]) -> Vec<NativeString> {
	utf16args.iter().map(|v| v.as_ptr()).collect()
}
