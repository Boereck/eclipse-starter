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
/// This struct holds all information needed by the launcher executable
/// that needs to be parsed from the command line and configuration ini file.
#[derive(Debug, Default)]
pub struct EclipseLauncherParams {
    pub name: Option<String>,
    pub eclipse_library: Option<String>,
    pub suppress_errors: bool,
    pub protect: Option<String>,
    pub launcher_ini: Option<String>,
    pub vm_args: Option<Vec<String>>,
}
