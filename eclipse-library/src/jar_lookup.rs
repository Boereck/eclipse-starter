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
 *     IBM Corporation - Initial C implementation and documentation
 *     Max Bureck (Fraunhofer FOKUS)
 *******************************************************************************/
use crate::params::EclipseParams;
use crate::errors::EclipseLibErr;
use std::path::{Path, PathBuf};
use eclipse_common::path_util::{check_path, find_file};

const DEFAULT_EQUINOX_STARTUP: &str = "org.eclipse.equinox.launcher";
const OLD_STARTUP: &str = "startup.jar";

pub fn find_startup_jar(params: &EclipseParams, program_dir: &Path) -> Result<PathBuf,EclipseLibErr> {
    // First see if command line argument is set
    if let Some(jar_path_str) = params.startup.as_ref() {
        let jar_path = Path::new(jar_path_str); 
        let jar_pathbuf = check_path(jar_path, program_dir, true);
        if jar_pathbuf.exists() {
            return Ok(jar_pathbuf);
        }
    }

    let mut plugins_dir = PathBuf::from(program_dir);
    if cfg!(target_os = "macos") {
        plugins_dir.push("../../../");
    }
    plugins_dir.push("plugins");

    // equinox startup jar?
    let file = find_file(&plugins_dir, DEFAULT_EQUINOX_STARTUP);
    if let Some(jar_path) = file {
        return Ok(jar_path);
    }

    // old startup.jar?
    let file = check_path(Path::new(OLD_STARTUP), program_dir, true);
    if file.exists() {
        return Ok(file);
    }

    // No Jar found. Sad.
    Err(EclipseLibErr::NoStartupJarFound)
}