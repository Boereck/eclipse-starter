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

//! This is a sub-module of either module "linux" or module "macos"
//! It exposes the function `find_vm_library` for both of those modules.


use std::path::{Path, PathBuf};
use crate::params::{EclipseParams, EclipseEEProps};

pub fn find_vm_library(exe_path: &Path, exe_dir: &Path, params: &EclipseParams, ee_props: Option<&EclipseEEProps>,) -> Option<PathBuf> {
    unimplemented!();
}

pub fn console_needed(params: &EclipseParams) -> bool {
    params.console.is_set() || params.console_log
}