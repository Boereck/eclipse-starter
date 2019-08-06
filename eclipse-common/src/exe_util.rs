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

use crate::path_util::*;
use std::path::{Path, PathBuf};

pub fn find_program(mut path: &str) -> Option<PathBuf> {
    path = strip_path(path);
    find_program_path(path)
}

// Remove quotes and interpret empty path as current dir
pub fn strip_path(mut path: &str) -> &str {
    if cfg!(target_os = "windows") {
        // remove quotes
        let quote = '"';
        path = path.trim_start_matches(quote).trim_end_matches(quote);
        // If we now have an empty string, interpret as current directory
    }
    if path.is_empty() {
        path = ".";
    }
    path
}

pub fn find_program_path<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
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
    path_env
        .split(PATHS_SEPARATOR)
        .map(strip_path)
        .map(PathBuf::from)
        .filter(|p| p.is_dir())
        .map(|mut p| {
            p.push(path);
            p
        })
        .find(|p| p.exists())
}
