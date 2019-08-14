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
use crate::errors::EclipseLibErr;
use crate::params::EclipseParams;
use eclipse_common::path_util::{check_path, find_file};
use std::path::{Path, PathBuf};

const DEFAULT_EQUINOX_STARTUP: &str = "org.eclipse.equinox.launcher";
const OLD_STARTUP: &str = "startup.jar";

pub fn find_startup_jar(
    params: &EclipseParams,
    program_dir: &Path,
) -> Result<PathBuf, EclipseLibErr> {
    // First see if command line argument is set
    if let Some(jar_path_str) = params.startup.as_ref() {
        let jar_path = Path::new(jar_path_str);
        let jar_pathbuf = check_path(jar_path, program_dir, true);

        // canonicalization will fail if file does not exist
        if let Ok(jar_pathbuf_result) = std::fs::canonicalize(jar_pathbuf) {
            return Ok(jar_pathbuf_result);
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
        if let Ok(jar_path_can) = std::fs::canonicalize(jar_path) {
            return Ok(jar_path_can);
        }
    }

    // old startup.jar?
    let file = check_path(Path::new(OLD_STARTUP), program_dir, true);

    // canonicalization will fail if file does not exist
    std::fs::canonicalize(file).map_err(|_| EclipseLibErr::NoStartupJarFound)
}
