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

use crate::params::{EclipseEEProps, EclipseParams};
use std::path::{Path, PathBuf};
use eclipse_common::path_buf;

pub fn is_vm_library(path: &Path) -> bool {
    let ext = if let Some(ext) = path.extension() {
        ext.to_str().unwrap_or_default()
    } else {
        return false;
    };
    is_vm_library_ext(ext)
}

#[allow(clippy::if_same_then_else)] // concatenating these large expressions with an || is not readable
pub fn is_vm_library_ext(file_ext: &str) -> bool {
    let is_win = cfg!(target_os = "windows");

    (is_win && file_ext == "dll")
        || ((!is_win) && (file_ext == "so" || file_ext == "jnilib" || file_ext == "dylib"))
}

pub fn get_vm_library_search_path(
    lib_path: &Path,
    params: &EclipseParams,
    ee_props: Option<&EclipseEEProps>,
) -> Vec<PathBuf> {
    // TODO: do we have to replace \ by / on windows?
    let paths = if let Some(EclipseEEProps {
        ee_lib_path: Some(paths),
        ..
    }) = ee_props
    {
        paths.iter().map(PathBuf::from).collect()
    } else {
        // add lib_path, its parent and grandparent/lib/arg
        let mut paths = vec![lib_path.to_path_buf()];
        if let Some(parent) = lib_path.parent() {
            paths.push(parent.to_path_buf());
            if let Some(grandparent) = parent.parent() {
                // trying grandparent/lib/arch
                let vm_arch = get_vm_arch(params);
                let path = path_buf![grandparent,"lib", vm_arch,];
                if path.exists() {
                    paths.push(path);
                }
            }
        }
        paths
    };
    paths
}

fn get_vm_arch(params: &EclipseParams) -> &str {
    let os_str: &str = if let Some(os_str) = &params.os {
        os_str
    } else {
        return "";
    };
    match os_str {
        "x86_64" => "amd64",
        "x86" => "i386",
        _ => os_str,
    }
}
