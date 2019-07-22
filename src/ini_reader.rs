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

//! This module provides the public method `read_ini`, which allows reading the
//! launcher configuration file, which is either located relative to the executable,
//! or on a user specified location.

use std::fs::File;
use std::io::prelude::*;
use std::io::{BufReader, Error};
use std::path::{Path, PathBuf};
use unicode_segmentation::UnicodeSegmentation;

/// Reads the ini config file either from a user specified location,
/// if `user_defined_config` is `Some`, or otherwise from a location relative 
/// to the executable. The path to the executable is passed via the parameter
/// `exe_path`. If the operation succeeds, the returned result will contain
/// an iterator over the lines of the config file. If the operation fails, the
/// result will contain an IO error.
pub fn read_ini(
    user_defined_config: &Option<String>,
    exe_path: &PathBuf,
) -> Result<impl Iterator<Item = String>, Error> {
    let ini_path = if let Some(user_ini) = user_defined_config {
        PathBuf::from(user_ini)
    } else {
        exe_to_ini_path(&exe_path)
    };
    let ini_file = File::open(ini_path)?;
    let reader = BufReader::new(ini_file);
    // Only take successfuly read lines, omit IO errors
    let result = reader
        .lines()
        .filter_map(Result::ok)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| line.trim_end().to_string());
    Ok(result)
}

/// Removes a possible file extension off the given `exe_path`
/// and adds the file extension `ini`. If the filename starts with
/// `c` on windows, the prefix is removed. The prefix is used
/// for launchers creating a console window on the win32 windowsing system.
fn exe_to_ini_path(exe_path: &PathBuf) -> PathBuf {
    let mut ini_path = exe_path.clone();
    ini_path.set_extension("ini");

    if cfg!(target_os = "macos") {
        let adjusted = adjust_macos_ini_path(&ini_path);
        if let Some(mac_path) = adjusted {
            return mac_path;
        }
        // If the MacOS path cannot be determined
        // the default location next to the launcher is returned
    }

    // We are the (win) console version, if the ini file does not exist, try
    // removing the 'c' from the end of the program name
    if cfg!(all(target_os = "windows", feature = "win_console")) {
        let adjusted_file_name_opt = adjust_console_file_name(&ini_path);
        if let Some(adjusted_file_name) = adjusted_file_name_opt {
            ini_path.set_file_name(adjusted_file_name);
        }
    }

    ini_path
}

/// On MacOSX, the eclipse.ini is not a sibling of the executable.
/// It is in `../Eclipse/<launcherName>.ini` relatively to the executable.
/// This method assumes the given `ini_path` references to the ini file
/// next to the launcher executable. This method adjusts the path and
/// returns a `PathBuf` pointing to the correct location.
/// If the location cannot be determined, an empty `Option` is returned.
fn adjust_macos_ini_path(ini_path: &Path) -> Option<PathBuf> {
    let parent = ini_path.parent()?.parent()?;
    let file_name = ini_path.file_name()?;
    let mut result = parent.to_path_buf();
    result.push("Eclipse");
    result.push(file_name);
    Some(result)
}

fn adjust_console_file_name(ini_path: &Path) -> Option<String> {
    let file_stem = ini_path.file_stem()?.to_str()?;
    let file_ext = ini_path.extension()?.to_str()?;
    let extended = true;
    let graphemes = file_stem.graphemes(extended);
    let (index, last_char) = graphemes.enumerate().last()?;
    if last_char == "c" {
        // get all elements except for last character
        let mut stripped = file_stem.graphemes(extended).take(index).collect::<String>();
        stripped.push('.');
        stripped.push_str(file_ext);
        Some(stripped)
    } else {
        // Nothing to change
        None
    }
}
