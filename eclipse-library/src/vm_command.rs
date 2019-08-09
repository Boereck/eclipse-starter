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

use crate::vm_lookup::JvmLaunchMode;
use std::path::Path;

// Get the command and arguments to start the Java VM.
pub fn get_vm_command<'a, 'b, 'c, 'd, 'e, S: AsRef<str>>(launch_mode: &JvmLaunchMode,
    args: &'a [S],
    vm_args: &'b [S],
    initial_args: &'c [S],
    jar_file: &'d Path,) -> Vec<&'e str> 
    where 'a: 'e, 'b: 'e, 'c: 'e, 'd: 'e {
    let result: Vec<&str> = Vec::new();
    unimplemented!();
}