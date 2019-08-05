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

//! TODO: document

use std::path::{PathBuf, Path};
use crate::path_util::*;

pub fn find_program<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
    let path = path.as_ref();
    
    find_program_internal(path).or_else(|| {
        // If the command does not end with .exe, append it an try again.
        if cfg!(target_os = "windows") && !has_extension_exe(path) {
            let mut path = path.to_path_buf();
            path.set_extension("exe");
            find_program_internal(&path)
        } else {
            None
        }
    })
}

fn find_program_internal(path: &Path) -> Option<PathBuf> {
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