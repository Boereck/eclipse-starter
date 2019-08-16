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

use crate::errors::{VmLaunchErr, VmRunErr, VmStartErr};
use crate::vm_command::VmArgs;
use crate::vm_lookup::JvmLaunchMode;
use std::borrow::Cow;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

/// Based on return code and shared data written by the started JVM,
/// a `StopAction` is derived, which may demand a restart.
pub enum StopAction {
    RestartLastEc,
    RestartNewEc,
    Nothing,
}

/// Data type for actually starting a JVM and interpreting the 
/// return code and shared data, written by the JVM.
#[derive(Debug)]
pub enum JavaLauncher<'a> {
    ExeLaunch {
        exe_path: &'a Path,
        all_args: Vec<&'a str>,
    },
    JniLaunch {
        jni_lib: &'a Path,
        jar_file: &'a Path,
        args: &'a VmArgs<'a>,
    },
}

impl<'a> JavaLauncher<'a> {

    /// Starts the JVM according to the parameters passed to the `new` function.
    /// If the VM terminates gracefully, a `StopAction` is provided which may
    /// demand a restart. If the JVM fails to start or terminates unsuccessfully,
    /// an `Err` holding a `VmLaunchErr` is returned.
    pub fn launch(&self) -> Result<StopAction, VmLaunchErr> {
        match self {
            JavaLauncher::JniLaunch {
                jni_lib,
                jar_file,
                args,
            } => launch_jni(jni_lib, jar_file, args),
            JavaLauncher::ExeLaunch { exe_path, all_args } => launch_exe(exe_path, &all_args),
        }
    }

    /// Creates a new instance of `JavaLauncher`. This launcher must not outlive
    /// any of the paramters passed to it.
    pub fn new<'t>(
        launch_mode: &'t JvmLaunchMode,
        vm_args: &'t VmArgs<'t>,
        jar_file_param: &'t Path,
    ) -> JavaLauncher<'t> {
        use JvmLaunchMode::*;
        match launch_mode {
            LaunchExe { exe, .. } => JavaLauncher::ExeLaunch {
                exe_path: exe,
                all_args: concat_args(vm_args),
            },
            LaunchJni {
                jni_lib: ref lib_path,
                ..
            } => JavaLauncher::JniLaunch {
                jni_lib: lib_path,
                jar_file: jar_file_param,
                args: vm_args,
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

fn launch_jni(
    jni_lib: &Path,
    jar_file: &Path,
    args: &VmArgs<'_>,
) -> Result<StopAction, VmLaunchErr> {
    // TODO implement
    unimplemented!();
}

fn launch_exe(exe_path: &Path, all_args: &[&str]) -> Result<StopAction, VmLaunchErr> {
    use VmLaunchErr::*;
    use VmRunErr::*;
    use VmStartErr::*;
    let mut child = Command::new(exe_path)
        .args(all_args.iter())
        .spawn()
        .map_err(|e| StartFail(ExeStartErr(e)))?;

    let mut last_check = Instant::now();
    let timeout = Duration::from_millis(100);
    let mut return_code = None;

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
                Ok(Some(status)) => {
                    return_code = status.code();
                    true
                }
                _ => {
                    // Some error ocurred, we try to kill the program, if not terminated already
                    child.kill();
                    true
                }
            }
        }
    };
    os::program_loop(is_terminated_callback);
    // TODO: handle return_code
    Ok(StopAction::Nothing)
}
