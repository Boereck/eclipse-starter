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

use crate::eclipse_params_parse::parse_args;
use crate::errors::EclipseLibErr;
use crate::params::EclipseParams;
use crate::vm_args_read::complete_vm_args;
use crate::vm_lookup::{JvmLaunchMode, determine_vm};
use eclipse_common::name_util::get_default_official_name_from_str;
use std::path::{Path, PathBuf};

static ACTION_OPENFILE: &str = "openFile";

pub fn run_framework<S: AsRef<str>>(
    args: &[S],
    vm_args: &[S],
    initial_args: &[S],
    library: &Path,
) -> Result<(), EclipseLibErr> {
    let mut parsed_args = parse_args(args);

    let program = args.get(0).map(|s| s.as_ref()).unwrap_or_default();
    let program_path = Path::new(program);
    if parsed_args.name.is_none() {
        let default_name = get_default_official_name_from_str(&program);
        parsed_args.name = default_name;
    }

    // TODO: on Mac start second thread if parsed_args.second_thread

    if let Some(action) = &parsed_args.default_action {
        let action = action.clone(); // appeases the borrow checker. OK, since inexpensive
        process_default_action(&action, initial_args, &mut parsed_args)
    }

    // TODO: initialize windowing system

    // TODO: on windows call Kernel32.dll#SetDllDirectoryW with empty str???
    // Find the directory where the Eclipse program is installed. If not able to, return Err
    let program_dir = program_path.parent()
        .ok_or(EclipseLibErr::HomeNotFound)?;

    let complete_vm_args = complete_vm_args(vm_args, &parsed_args, &program_path);

    let vm_path = determine_vm(&parsed_args, program_dir)?;
    dbg!(vm_path);
    // TODO: determine the vm to use.
    // TODO: find startup jar

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

