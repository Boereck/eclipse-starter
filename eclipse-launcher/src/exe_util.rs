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

//! This module provides the public function `get_exe_path()` which provides the 
//! Path to the current executable. The algorithm was ported directly from the 
//! original C version, it should be investigated if a simple call to `std::fs::canonicalize`
//! would suffice.

use std::path::PathBuf;
use std::io;
use eclipse_common::exe_util::find_program;


/// If `exe_path` is a symlink, the symlink is resolved 
/// (and if the resolved file is a symlink as well it will continue resolving)
/// and the resolved file will be returned.
pub fn get_exe_path() -> Result<PathBuf,io::Error> {
    // First try to find program from first command line argument
    if let Some(first_arg) = std::env::args().next() {
        let program_path = PathBuf::from(first_arg);
        if let Some(program_location) = find_program(&program_path) {
            return Ok(program_location);
        }
    }
    // If we were not able to detect program location from arguments, we use OS functions
    let exe_path = std::env::current_exe()?;
    // Just to be sure, and to resolve symlinks, conconicalize the path
    std::fs::canonicalize(exe_path)
}


