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

use std::path::{PathBuf, Path};
use std::io;
use eclipse_common::path_util::*;


// It seems there is no constant for paths separator in the rust standard library,
// so we define this character for ourselves.

#[cfg(target_os = "windows")]
const PATHS_SEPARATOR: char = ';';

#[cfg(not(target_os = "windows"))]
const PATHS_SEPARATOR: char = ':';

/// If `exe_path` is a symlink, the symlink is resolved 
/// (and if the resolved file is a symlink as well it will continue resolving)
/// and the resolved file will be returned.
pub fn get_exe_path() -> Result<PathBuf,io::Error> {
    // First try to find program from first command line argument
    if let Some(first_arg) = std::env::args().next() {
        let mut program_path = PathBuf::from(first_arg);
        // On windows we always use file extension .exe
        if cfg!(windows) && !has_extension_exe(&program_path) {
            program_path.set_extension("exe");
        }
        if let Some(program_location) = find_program(&program_path) {
            return Ok(program_location);
        }
    }
    // If we were not able to detect program location from arguments, we use OS functions
    let exe_path = std::env::current_exe()?;
    // Just to be sure, and to resolve symlinks, conconicalize the path
    std::fs::canonicalize(exe_path)
}

fn find_program(path: &PathBuf) -> Option<PathBuf> {
    let path = if path.is_absolute() || has_parent(path) {
        path.to_path_buf()
    } else {
        search_on_path_env(path)?
    };
	// if path is relative we make it absolute,
	// if path contains symlinks, they alre also neatly resolved
    std::fs::canonicalize(path).ok()
}

fn search_on_path_env(path: &Path) -> Option<PathBuf> {
    let path_env = std::env::var("PATH").ok()?;
	// test for every path `prefix` in the PATH environment variable
	// if the concatenation `prefix/path` exists and if so, return
	// the resulting path.
    path_env.split(PATHS_SEPARATOR)
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .map(|mut p| { 
            p.push(path); 
            p
         })
        .find(|p| p.exists())
}

