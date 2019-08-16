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

use crate::errors::{VmLaunchErr, VmStartErr, VmRunErr};
use crate::vm_command::VmArgs;
use crate::vm_lookup::JvmLaunchMode;
use std::borrow::Cow;
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

pub enum StopAction {
    RestartLastEc,
    RestartNewEc,
    Nothing,
}

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
    pub fn launch(&self) -> Result<StopAction, VmLaunchErr> {
        // TODO: return type
        match self {
            JavaLauncher::JniLaunch {
                jni_lib,
                jar_file,
                args,
            } => launch_jni(jni_lib, jar_file, args),
            JavaLauncher::ExeLaunch { exe_path, all_args } => launch_exe(exe_path, &all_args),
        }
    }

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

fn launch_jni(jni_lib: &Path, jar_file: &Path, args: &VmArgs<'_>) -> Result<StopAction, VmLaunchErr> {
    // TODO implement
    unimplemented!();
}

fn launch_exe(exe_path: &Path, all_args: &[&str]) -> Result<StopAction, VmLaunchErr> {
    use VmLaunchErr::*;
    use VmStartErr::*;
    use VmRunErr::*;
    let mut child = Command::new(exe_path)
        .args(all_args.iter())
        .spawn()
        .map_err(|e| StartFail(ExeStartErr(e)))?;

    let mut last_check = Instant::now();
    let timeout = Duration::from_millis(100);
    let mut return_code = None;
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
                    // Some error ocurred, we try to kill the program, if not closed already
                    child.kill();
                    true
                }
            }
        }
    };
    program_loop(is_terminated_callback);
    // TODO: handle return_code
    Ok(StopAction::Nothing)
}

fn program_loop(mut is_term_callback: impl FnMut() -> bool) {
    let dur = Duration::from_millis(50);
    loop {
        if is_term_callback() {
            return;
        }
        std::thread::sleep(dur);
    }
}
