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
 *     IBM Corporation - Initial C implementation and documentation
 *     Max Bureck (Fraunhofer FOKUS)
 *******************************************************************************/

//! Eclipse Program Launcher
//!
//! This file forms the base of the eclipse_*.dll/so.  This dll is loaded by eclipse.exe
//! to start a Java VM, or alternatively it is loaded from Java to show the splash
//! screen or write to the shared memory.  See eclipseJNI.c for descriptions of the methods
//! exposed to the Java program using JNI.
//!
//! To display a splash screen before starting the java vm, the launcher should be started
//! with the location of the splash bitmap to use:
//! -showsplash <path/to/splash.bmp>
//! Otherwise, when the Java program starts, it should determine the location of
//! the splash bitmap to be used and use the JNI method show_splash.
//!
//! When the Java program initialization is complete, the splash window
//! is brought down by calling the JNI method takedown_splash.
//!
//! The Java program can also call the get_splash_handle method to get the handle to the splash
//! window.  This can be passed to SWT to create SWT widgets in the splash screen.
//!
//! The Java application will receive two other arguments:
//!    -exitdata <shared memory id>
//!
//! The java program can call set_exit_data with this shared-memory-id
//! to provide specific exit data to the launcher.
//!
//! The exit data size must not exceed MAX_SHARED_LENGTH which is
//! 16Kb. The interpretation of the exit data is dependent on the
//! exit value of the java application.
//!
//! The main launcher recognizes the following exit codes from the
//! Java application:
//!
//!    0    - Exit normally.
//!    RESTART_LAST_EC = 23
//!       - restart the java VM again with the same arguments as the previous one.
//!    RESTART_NEW_EC  = 24
//!       - restart the java VM again with the arguments taken from the exit data.
//!       The exit data format is a list of arguments separated by '\n'. The Java
//!       application should build this list using the arguments passed to it on
//!       startup. See below.
//!
//! Additionally, if the Java application exits with an exit code other than the
//! ones above, the main launcher will display an error message with the contents
//! of the exit data. If the exit data is empty, a generic error message is
//! displayed. The generic error message shows the exit code and the arguments
//! passed to the Java application.
//!
//! The options that can be specified by the user to the launcher are:
//!  -vm <javaVM>               the Java VM to be used
//!  -os <opSys>                the operating system being run on
//!  -arch <osArch>             the hardware architecture of the OS: x86, sparc, hp9000
//!  -ws <gui>                  the window system to be used: win32, gtk, cocoa, ...
//!  -nosplash                  do not display the splash screen. The java application will
//!                             not receive the -showsplash command.
//!  -showsplash <bitmap>       show the given bitmap in the splash screen.
//!  -name <name>               application name displayed in error message dialogs and
//!                             splash screen window. Default value is computed from the
//!                             name of the executable - with the first letter capitalized
//!                             if possible. e.g. eclipse.exe defaults to the name Eclipse.
//!  -startup <startup.jar>     the startup jar to execute. The argument is first assumed to be
//!                             relative to the path of the launcher. If such a file does not
//!                             exist, the argument is then treated as an absolute path.
//!                             The default is find the plugins/org.eclipse.equinox.launcher jar
//!                             with the highest version number.
//!                             The jar must contain an org.eclipse.equinox.launcher.Main class.
//!                             (unless JNI invocation is not being used, then the jar only needs to be
//!                             an executable jar)
//! -library                    the location of the eclipse launcher shared library (this library) to use
//!                             By default, the launcher exe (see eclipseMain.c) finds
//!  <userArgs>                 arguments that are passed along to the Java application
//!                             (i.e, -data <path>, -debug, -console, -consoleLog, etc)
//!  -vmargs <userVMargs> ...   a list of arguments for the VM itself
//!
//! The -vmargs option and all user specified VM arguments must appear
//! at the end of the command line, after all arguments that are
//! being passed to Java application.
//!
//! The argument order for the new Java VM process is as follows:
//!
//! <javaVM> <all VM args>
//!     -os <user or default OS value>
//!     -ws <user or default WS value>
//!     -arch <user or default ARCH value>
//!     -launcher <absolute launcher name>
//!     -name <application name>
//!     -library <eclipse library location>
//!     -startup <startup.jar location>
//!     [-showsplash]
//!     [-exitdata <shared memory id>]
//!     <userArgs>
//!     -vm <javaVM>
//!     -vmargs <all VM args>
//!
//! where:
//!   <all VM args> =
//!     [<defaultVMargs | <userVMargs>]
//!     -jar
//!     <startup jar full path>
//!
//! The startup jar must be an executable jar.
//!
//!
//! See "Main.java" for a simple implementation of the Java
//! application.
//!
//! Configuration file
//!   The launcher gets arguments from the command line and/or from a configuration file.
//! The configuration file must have the same name and location as the launcher executable
//! and the extension .ini. For example, the eclipse.ini configuration file must be
//! in the same folder as the eclipse.exe or eclipse executable (except in the case of
//! Mac OS X where the eclipse.ini can be read from a Mac specific configuration folder
//! recommended by the OS, see bugs 461725 and 461728 for more details).
//!
//!   The format of the ini file matches that of the command line arguments - one
//! argument per line.
//!   In general, the settings of the config file are expected to be overriden by the
//! command line.
//!   - launcher arguments (-os, -arch...) set in the config file are overriden by the command line
//!   - the -vmargs from the command line replaces in its entirety the -vmargs from the config file.
//!   - user arguments from the config file are prepended to the user arguments defined in the
//!     config file. This is consistent with the java behaviour in the following case:
//!     java -Dtest="one" -Dtest="two" ...  : test is set to the value "two"

mod eclipse_params_parse;
mod ee_params_parse;
mod errors;
mod iter_ptr;
mod java;
mod native_str_read;
mod params;
mod run;
mod vm_args_read;
mod vm_lookup;
mod jar_lookup;

use eclipse_common::native_str::NativeString;
use lazy_static::lazy_static;
use native_str_read::*;
use std::os::raw::c_int;
use std::path::PathBuf;
use std::sync::Mutex;

static MSG_SETTING_INITIAL_ARGS_FAIL: &str = "Accessing intial arguments failed.";
static LOCK_ERR_CODE: i32 = 2;

#[derive(Default)]
struct InitialArgs {
    args: Vec<String>,
    library: PathBuf,
}

lazy_static! {
    static ref INITIAL_ARGS: Mutex<InitialArgs> = Mutex::default();
}

#[cfg(not(target_os = "windows"))]
#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn setInitialArgs(args_size: c_int, args: *mut NativeString, library: NativeString) {
    let arg_strings = utf8_str_array_to_string_vec(args, args_size as usize);
    let library_str = utf8_str_to_string(library).unwrap_or_default();
    let library_path = PathBuf::from(&library_str);
    set_initial_args_internal(arg_strings, library_path)
}

#[cfg(target_os = "windows")]
#[no_mangle]
#[allow(non_snake_case)]
pub extern "C" fn setInitialArgsW(
    args_size: c_int,
    args: *mut NativeString,
    library: NativeString,
) {
    let arg_strings = utf16_str_array_to_string_vec(args, args_size as usize);
    let library_str = utf16_to_string(&library).unwrap_or_default();
    let library_path = PathBuf::from(&library_str);
    set_initial_args_internal(arg_strings, library_path)
}

fn set_initial_args_internal(args: Vec<String>, library: PathBuf) {
    let lock = INITIAL_ARGS.lock();
    if let Ok(mut guard) = lock {
        guard.args = args;
        guard.library = library;
    } else {
        // We have no information about suppressing errors, let's just print to syserr
        eprintln!("{}", MSG_SETTING_INITIAL_ARGS_FAIL);
    }
}

#[cfg(not(target_os = "windows"))]
#[no_mangle]
pub unsafe extern "C" fn run(
    args_size: c_int,
    args: *mut NativeString,
    vm_args: *mut NativeString,
) -> c_int {
    let arg_strings = utf8_str_array_to_string_vec(args, args_size as usize);
    let vm_arg_strings = null_term_utf8_str_array_to_string_vec(vm_args);
    run_internal(arg_strings, vm_arg_strings)
}

#[cfg(target_os = "windows")]
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "C" fn runW(
    args_size: c_int,
    args: *mut NativeString,
    vm_args: *mut NativeString,
) -> c_int {
    let arg_strings = utf16_str_array_to_string_vec(args, args_size as usize);
    let vm_arg_strings = null_term_utf16_str_array_to_string_vec(vm_args);
    run_internal(arg_strings, vm_arg_strings)
}

fn run_internal(args: Vec<String>, vm_args: Vec<String>) -> i32 {
    let lock = INITIAL_ARGS.lock();
    let mut initial_args = match lock {
        Ok(guard) => guard,
        Err(e) => {
            // TODO: if !args.suppress_errors call eclipse_common::messagebox::display_message
            // We have no information about suppressing errors, let's just print to syserr
            eprintln!("{}", MSG_SETTING_INITIAL_ARGS_FAIL);
            eprintln!("{:?}", e);
            return LOCK_ERR_CODE;
        }
    };

    let result = run::run_framework(&args, &vm_args, &initial_args.args, &initial_args.library);

    // TODO: turn result into return code and show error messages
    if let Err(e) = result {
        eprintln!("{:#?}", e);
    }

    // Free global memory
    initial_args.args = Vec::new();
    initial_args.library = PathBuf::new();
    0
}
