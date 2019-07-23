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

//! This module holds declarations and error messages common for the OS specific
//! launcher_lib implementations in `crate::launcher_lib::os`, which are located
//! in `windows.rs` and `unix.rs`.

use crate::errors::LauncherError;
pub(crate) use eclipse_common::native_str::NativeString;
use dlopen::symbor::Library;
use std::os::raw::c_int;

pub static MSG_LOAD_LIB_SYMBOL_RESOLVE_ERROR: &str =
    "There was a problem loading the shared library and finding the entry point.";
pub static MSG_ERROR_CALLING_RUN: &str = "Error calling the run function on the launcher library.";

/// API of the native companion library's "run" method. Before calling this method, the user
/// has to call method "setInitialArgs".
///
/// The first parameter represents the count of strings that are passed as the second argument.
/// As the second parameter an array of native strings of the merged parameters from ini config file,
/// followed by command line parameters is expected. The third parameter is an array of JVM parameters read
/// from the command line. Note that the VM argument array must be terminated with `null` element.
pub(super) type RunMethod =
    unsafe extern "C" fn(c_int, *const NativeString, *const NativeString) -> c_int;

/// API of the native companion library's "setInitialArgs" method. This method has to be called
/// before calling the `run` method.
///
/// The first parameter represents
/// the count of strings passed as the second argument. The caller shall pass the original
/// command line paramers as an array of strings to this method. The third parameter must
/// be the absolute file path to the native library that is called.
///
/// *Important*: The caller is responsible for the lifetime of all passed pointers. All arrays
/// and strings must be kept in memory until after the "run" method returned and then freed.
pub(super) type SetInitialArgs =
    unsafe extern "C" fn(c_int, *const NativeString, NativeString) -> ();

/// Type holding inital parameters needed to call `EclipseLauncher::set_initial_args`.
pub trait InitialArgs<'b> {
    /// Creates a new instance of a concrete `InitialArgs` implementation.
    /// Note that users of this module shall use the function `EclipseLauncher::new_initial_args`
    /// to create an instance of `InitialArgs`.
    fn new<S: AsRef<str>>(args: &'b [S], library: &'b str) -> Self;
}

/// This trait represents the API surface of the launcers companion dynamic library.
/// To craete an instance of this type, use function `new_launcher`.
pub trait EclipseLauncher<'a, 'b>: Sized
where
    'b: 'a,
{
    type InitialArgsParams: InitialArgs<'b>;

    /// Creates a new instance of a concrete `EclipseLauncher` implementation.
    /// Note that users of this module shall use the function `new_launcher`
    /// to craete an instance of `EclipseLauncher`.
    fn new(lib: &'a Library) -> Result<Self, LauncherError>;

    /// Starts the main application. The caller has to provide the merged
    /// start parameters (first from config file, followed by arguments from command line)
    /// without the JVM parameters. The JVM arguments from the command line are
    /// passed by the `vm_args` parameter.
    ///
    /// *Note*: `set_initial_args` has to be called before calling this function.
    fn run<S: AsRef<str>>(&self, args: &[S], vm_args: &[S]) -> Result<(), LauncherError>;

    /// Creates a `InitialArgsParams` value holding the information about
    /// the initial command line arguments `args` and the file path to the
    /// dynamic companion library via `library` parameter.
    #[inline]
    fn new_initial_args<S: AsRef<str>>(
        &self,
        args: &'b [S],
        library: &'b str,
    ) -> Self::InitialArgsParams {
        Self::InitialArgsParams::new(args, library)
    }

    /// Sets the initial command line arguments and the location of the
    /// companion dynamic library. The `params` parameter value has to be created
    /// by the caller via the `new_initial_args` function.
    /// This function needs to be called before calling the `run` function.
    fn set_initial_args(&self, params: &Self::InitialArgsParams) -> Result<(), String>;
}
