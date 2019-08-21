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
mod exe_launch;
mod jni_launch;

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
            } => jni_launch::launch_jni(jni_lib, jar_file, args, *shared_mem),
            JavaLauncher::ExeLaunch {
                exe_path,
                all_args,
                shared_mem,
            } => exe_launch::launch_exe(exe_path, all_args, *shared_mem),
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
