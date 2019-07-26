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

use eclipse_common::arg_parser::OptionalParam;

#[derive(Default, Debug)]
pub struct EclipseParams {
    pub console: OptionalParam,
    pub console_log: bool,
    pub debug: OptionalParam,
    pub os: Option<String>,
    pub arch: Option<String>,
    pub nosplash: bool,
    pub showsplash: OptionalParam,
    pub startup: Option<String>,
    pub vm: Option<String>,
    pub ws: Option<String>,
    pub name: Option<String>,
    pub protect: Option<String>,
    pub openfile: Option<Vec<String>>,
    pub default_action: Option<String>,
    pub timeout: Option<String>,
    pub suppress_errors: bool,
    pub library: Option<String>,
    pub ini: Option<String>,
    pub append_vmargs: bool,
    pub override_vmargs: bool,
    pub second_thread: bool,
    pub perm_gen: bool,
    pub gtk_version: Option<String>,
}
