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

use crate::errors::{EclipseLibErr, VmRunErr, VmStartErr};
use crate::shared_mem::SharedMem;
use std::ffi::OsString;
use std::process::Command;
use std::time::{Duration, Instant};
use super::{ArgList, StopAction, os, RESTART_LAST_EC, RESTART_NEW_EC, };

/// Launches the VM found under `exe_path`, with the arguments provided
/// by `all_args`. When finished, the method will interpret the return
/// code, and may read from the shared memory that access is provided 
/// to via `shared_mem`. If needed, this method will adjust `exe_path` and
/// `all_args` for a subsequent launch. The return value will give the
/// caller instructions on how to proceed.
pub(super) fn launch_exe<S: SharedMem>(
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
