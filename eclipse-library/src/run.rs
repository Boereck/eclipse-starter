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

//! This method provides the main `run_framework` method, which drives the 
//! launcher code.

use core::fmt::Debug;
use crate::eclipse_params_parse::parse_args;
use crate::errors::EclipseLibErr;
use crate::jar_lookup::find_startup_jar;
use crate::params::EclipseParams;
use crate::shared_mem::{create_shared_mem, SharedMem, MAX_SHARED_LENGTH};
use crate::vm_args_read::complete_vm_args;
use crate::vm_command::{get_vm_command, VmArgs};
use crate::vm_launch::{JavaLauncher, StopAction};
use crate::vm_lookup::{determine_vm, JvmLaunchMode};
use crate::console_detection::is_console_launcher;
use eclipse_common::name_util::get_default_official_name_from_str;
use eclipse_common::path_util::strip_unc_prefix;
use std::path::Path;

const ACTION_OPENFILE: &str = "openFile";

pub fn run_framework<S: AsRef<str> + Debug>(
    args: &[S],
    vm_args: &[S],
    initial_args: &[S],
    library: &Path,
) -> Result<(), EclipseLibErr> {
    let (mut parsed_args, remaining_args) = parse_args(args);

    let program = args.get(0).map(|s| s.as_ref()).unwrap_or_default();
    let program_path = Path::new(program);
    if parsed_args.name.is_none() {
        let default_name = get_default_official_name_from_str(&program);
        parsed_args.name = default_name;
    }
    // We prefer the library passed to this program instead parsed from parameter
    // since the launcher allready did path resolution.
    let library_str = &library.to_string_lossy();
    let library_str = strip_unc_prefix(library_str);
    parsed_args.library.replace(library_str.to_string());

    // TODO: on Mac start second thread if parsed_args.second_thread

    if let Some(action) = &parsed_args.default_action {
        let action = action.clone(); // appeases the borrow checker. OK, since inexpensive
        process_default_action(&action, initial_args, &mut parsed_args)
    }

    // TODO: initialize windowing system

    // TODO: on windows call Kernel32.dll#SetDllDirectoryW with empty str???

    // Find the directory where the Eclipse program is installed. If not able to, return Err
    let program_dir = program_path.parent().ok_or(EclipseLibErr::HomeNotFound)?;

    // find startup jar
    let jar_file = find_startup_jar(&parsed_args, program_dir)?;
    let jar_file_str = strip_unc_prefix(&jar_file.to_string_lossy()).to_string();
    parsed_args.startup = Some(jar_file_str);

    let win_console = is_console_launcher();
    let complete_vm_args = complete_vm_args(&vm_args, &parsed_args, &program_path, win_console)?;

    let vm_path = determine_vm(&parsed_args, program_dir)?;

    // TODO: reuse running eclipse if params.openfile is Some
    // TODO: on windows: if( launchMode == LAUNCH_JNI && (debug || needConsole) ) createConsole
    // TODO: show splash if needed
    // not using JNI launching, need some shared data
    let shared_data = create_shared_mem(MAX_SHARED_LENGTH)?;

    let vm_command = get_vm_command(
        &vm_path,
        &remaining_args,
        &complete_vm_args,
        &jar_file,
        &parsed_args,
        shared_data.get_id(),
        program_path,
    );

    let mut vm_launcher = JavaLauncher::new(&vm_path, &vm_command, &jar_file, &shared_data);

    // While the Java VM should be restarted
    loop {
        // TODO: store vm command as message
        // TODO: if -debug, print start command to console
        // TODO: Handle result (restart if necessary)
        match vm_launcher.launch()? {
            StopAction::Nothing => {
                // No reastart needed, stop the loop
                break;
            },
            StopAction::RestartExeNewArgs(new_args) => {
                // TODO: restart launcher with new_args
                break;
            },
            StopAction::RestartExeLastArgs => {
                // TODO: restart lauchner with current args
                break;
            },
            StopAction::RestartVM => {
                // Nothing to do, remain in restart loop
            },
        }
    }

    shared_data.close()?;

    // TODO: Port rest of run from C
    Ok(())
}

/// Based on the default `action` uses `initial_args`
/// to update the given `params`.
fn process_default_action<S: AsRef<str>>(
    action: &str,
    initial_args: &[S],
    params: &mut EclipseParams,
) {
    // We currently only support "openFile" default action
    if action != ACTION_OPENFILE {
        return;
    }
    let contains_start_dash_param = initial_args
        .iter()
        .skip(1)
        .any(|s| s.as_ref().starts_with('-'));
    // if any parameter starts with "-" we call default action parsing off
    if contains_start_dash_param {
        return;
    }

    // Set files to open to all parameters passed to executable (excluding program path)
    // As original C source, we will overwrite existing --launcher.openFile option
    // in future semantics could maybe change to merge.
    let files: Vec<String> = initial_args
        .iter()
        .skip(1)
        .map(|s| s.as_ref().to_string())
        .collect();
    params.openfile = Some(files);
}
