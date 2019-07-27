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

//! This module provides functionality to provide  

use std::path::{Path, PathBuf};
use unicode_segmentation::UnicodeSegmentation;

/// Determine the default official application name
///
/// This function provides the default application name that appears in a variety of
/// places such as: title of message dialog, title of splash screen window
/// that shows up in Windows task bar.
/// It is computed from the name of the launcher executable and
/// by capitalizing the first letter. e.g. "c:/ide/eclipse.exe" provides
/// a default name of "Eclipse".
pub fn get_default_official_name() -> Option<String> {
    let exe_path = std::env::current_exe().ok()?;
    get_default_official_name_from_path(&exe_path)
}

pub fn get_default_official_name_from_str(exe_path: &str) -> Option<String> {
    let exe_path = PathBuf::from(exe_path);
    get_default_official_name_from_path(&exe_path)
}

pub fn get_default_official_name_from_path(exe_path: &Path) -> Option<String> {
    let file_name = exe_path.file_stem()?.to_str()?;
    // TODO on win_console trim trailing 'c'
    first_to_uppercase(file_name).into()
}

fn first_to_uppercase(input: &str) -> String {
    // split characters at unicode "grapheme cluster"
    let extended = true;
    let mut graphemes = input.graphemes(extended);
    // Uppercase only first cluster
    let mut result = String::new();
    if let Some(first_char) = graphemes.next() {
        result.push_str(&first_char.to_uppercase());
    }
    // push the rest "as is" into the result
    result.push_str(graphemes.as_str());
    result
}
