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

//! This module provides the type `JavaLauncher`, which allows starting a JVM
//! from all given parameters, JVM location and jar file to launch.

#[cfg_attr(target_os = "macos", path = "macos.rs")]
#[cfg_attr(target_os = "linux", path = "linux.rs")]
#[cfg_attr(target_os = "windows", path = "windows.rs")]
mod os;

use crate::errors::{EclipseLibErr, VmLaunchErr, VmRunErr, VmStartErr};
use crate::shared_mem::SharedMem;
use crate::vm_command::VmArgs;
use crate::vm_lookup::JvmLaunchMode;
use std::borrow::Cow;
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

const RESTART_LAST_EC: i32 = 23;
const RESTART_NEW_EC: i32 = 24;

/// Wraps around a list of either owned, or borrowed strings
#[derive(Debug)]
pub enum ArgList<'a> {
    Owned(Vec<String>),
    Borrowed(Vec<&'a str>),
}

impl<'a> From<Vec<&'a str>> for ArgList<'a> {
    fn from(args: Vec<&'a str>) -> ArgList<'a> {
        ArgList::Borrowed(args)
    }
}

impl<'a> From<Vec<String>> for ArgList<'a> {
    fn from(args: Vec<String>) -> ArgList<'a> {
        ArgList::Owned(args)
    }
}

impl<'a> ArgList<'a> {
    /// Adds the argument strings to the given `cmd`
    fn add_to<'b>(&'a self, cmd: &'b mut Command) -> &'b mut Command {
        match self {
            ArgList::Owned(vec) => cmd.args(vec.iter()),
            ArgList::Borrowed(vec) => cmd.args(vec.iter()),
        }
    }
}

/// Based on return code and shared data written by the started JVM,
/// a `StopAction` is derived, which may demand a restart.
#[derive(Debug)]
pub enum StopAction {
    /// Restart VM via the JavaLauncher
    RestartVM,

    /// Restart the launcher executable with the same arguments
    RestartExeLastArgs,

    /// Restart the launcher executable with arguments in this variant
    RestartExeNewArgs(Vec<String>),

    /// Do not restart, exit program
    Nothing,
}

/// Data type for actually starting a JVM and interpreting the
/// return code and shared data, written by the JVM.
#[derive(Debug)]
pub enum JavaLauncher<'a, S: SharedMem> {
    ExeLaunch {
        exe_path: OsString,
        all_args: ArgList<'a>,
        shared_mem: &'a S,
    },
    JniLaunch {
        jni_lib: &'a Path,
        jar_file: &'a Path,
        args: &'a VmArgs<'a>,
        shared_mem: &'a S,
    },
}

impl<'a, S: SharedMem> JavaLauncher<'a, S> {

    /// Starts the JVM according to the parameters passed to the `new` function.
    /// If the VM terminates gracefully, a `StopAction` is provided which may
    /// demand a restart. If the JVM fails to start or terminates unsuccessfully,
    /// an `Err` holding a `VmLaunchErr` is returned. If the started VM requests 
    /// a VM restart (`StopAction::RestartVM`), the launcher will be altered, 
    /// so that the launch method can simply be invoked again.
    pub fn launch(&mut self) -> Result<StopAction, EclipseLibErr> {
        match self {
            JavaLauncher::JniLaunch {
                jni_lib,
                jar_file,
                args,
                shared_mem,
            } => launch_jni(jni_lib, jar_file, args, *shared_mem),
            JavaLauncher::ExeLaunch {
                exe_path,
                all_args,
                shared_mem,
            } => launch_exe(exe_path, all_args, *shared_mem),
        }
    }

    /// Creates a new instance of `JavaLauncher`. This launcher must not outlive
    /// any of the paramters passed to it.
    pub fn new<'t>(
        launch_mode: &'t JvmLaunchMode,
        vm_args: &'t VmArgs<'t>,
        jar_file_param: &'t Path,
        shared_mem_param: &'t S,
    ) -> JavaLauncher<'t, S> {
        use JvmLaunchMode::*;
        match launch_mode {
            LaunchExe { exe, .. } => JavaLauncher::ExeLaunch {
                exe_path: exe.into(),
                all_args: concat_args(vm_args).into(),
                shared_mem: shared_mem_param,
            },
            LaunchJni {
                jni_lib: ref lib_path,
                ..
            } => JavaLauncher::JniLaunch {
                jni_lib: lib_path,
                jar_file: jar_file_param,
                args: vm_args,
                shared_mem: shared_mem_param,
            },
        }
    }
}

/// Concatenates the elements of `all_args.vm_args` and
/// `all_args.program_args` into a list of all element.
fn concat_args<'a>(all_args: &'a VmArgs<'a>) -> Vec<&'a str> {
    let mut result: Vec<&'a str> = to_str_iter(&all_args.vm_args).collect();

    let prog_args_iter = to_str_iter(&all_args.program_args);
    result.extend(prog_args_iter);

    result
}

fn to_str_iter<'e>(cows: &'e [Cow<'e, str>]) -> impl Iterator<Item = &'e str> {
    cows.iter().map(|s| -> &'e str { &s })
}

// Actual launching

fn launch_jni<S: SharedMem>(
    jni_lib: &Path,
    jar_file: &Path,
    args: &VmArgs<'_>,
    shared_mem: &S,
) -> Result<StopAction, EclipseLibErr> {
    // TODO implement
    unimplemented!();
}

fn launch_exe<S: SharedMem>(
    exe_path: &mut OsString,
    all_args: &mut ArgList,
    shared_mem: &S,
) -> Result<StopAction, EclipseLibErr> {
    use VmStartErr::*;
    let mut command = Command::new(&exe_path);
    let mut child = all_args
        .add_to(&mut command)
        .spawn()
        .map_err(ExeStartErr)?;

    let mut last_check = Instant::now();
    let timeout = Duration::from_millis(100);

    // Callback, letting the program loop know if the JVM is already terminated.
    // Only does the check every 100 milliseconds, which is hopefully cheaper than
    // querying if process is still running.
    let is_terminated_callback = || {
        if last_check.elapsed() < timeout {
            false
        } else {
            last_check = Instant::now();
            match child.try_wait() {
                Ok(None) => false,
                _ => true,
            }
        }
    };
    os::program_loop(is_terminated_callback);

    // Check why we terminated
    match child.try_wait() {
        // Regular termination
        Ok(Some(exit_status)) => match exit_status.code() {
            Some(return_code) => {
                result_from_exe_exit_code(exe_path, all_args, shared_mem, return_code)
            }
            None => Err(VmRunErr::UnknownErr)?,
        },
        // Not terminated?
        #[allow(unused_must_use)] // may fail if already terminated, we don't care
        Ok(None) => {
            // This should not happen, maybe bug in ps::program_loop?
            child.kill();
            Err(VmRunErr::UnknownErr)?
        }
        // Error on termination check
        #[allow(unused_must_use)] // may fail if already terminated, we don't care
        Err(e) => {
            // Some error ocurred, we try to kill the program, if not terminated already
            child.kill();
            Err(VmRunErr::TerminationErr(e))?
        }
    }
}

fn result_from_exe_exit_code(
    exe_path: &mut OsString,
    arg_list: &mut ArgList<'_>,
    shared_mem: &impl SharedMem,
    code: i32,
) -> Result<StopAction, EclipseLibErr> {
    match code {
        0 => Ok(StopAction::Nothing),
        RESTART_LAST_EC => Ok(StopAction::RestartVM),
        RESTART_NEW_EC => {
            // Update list of commands from shared memory
            let shared_str = shared_mem.read()?;
            let mut str_iter = shared_str.lines();
            // first parameter is the path to the java executable
            *exe_path = str_iter.next().unwrap_or_default().into();
            // the rest are parameters
            let new_args: Vec<String> = str_iter.map(String::from).collect();
            *arg_list = new_args.into();

            Ok(StopAction::RestartVM)
        }
        failure_code => Err(VmRunErr::FailureReturnCode(failure_code))?,
    }
}
